//! Actual TLS wiring plus a private post-handshake response-BIO gate.
//! The gate is deterministic scheduling, not observed kernel backpressure.
use super::super::issuer_lifecycle_tests::cert;
use super::*;
use crate::{
    authority::{CredentialBinding, Registry},
    issuer::OperationReference,
    retrieval::{RetrievalAction, RetrievalPolicy},
    services::{IssuerRoute, ServicePolicy, ServiceRegistration, ServiceUse},
};
use faction_files::private_blobs::{Binding, Principal};
use std::sync::{Mutex, mpsc};
struct Real {
    runtime: Runtime,
    ca: (X509, PKey<Private>),
    server: (X509, PKey<Private>),
    device: (X509, PKey<Private>),
    relay: (X509, PKey<Private>),
}
impl Real {
    fn new() -> Self {
        let t = wall_now() as i64;
        let ca = cert(None, "CA", 1, t - 60, t + 3600);
        let server = cert(Some((&ca.0, &ca.1)), "provider.test", 2, t - 60, t + 3600);
        let device = cert(Some((&ca.0, &ca.1)), "device", 3, t - 60, t + 3600);
        let relay = cert(Some((&ca.0, &ca.1)), "relay", 4, t - 60, t + 3600);
        let inspector = cert(Some((&ca.0, &ca.1)), "inspector", 5, t - 60, t + 3600);
        let registration = |leaf: &X509, home: &str, id: &str| ServiceRegistration {
            home: home.into(),
            client_id: id.into(),
            dns_san: "provider.test".into(),
            certificate_sha256: sha256(&leaf.to_der().unwrap()),
            enabled: true,
            valid_from: (t - 60) as u64,
            valid_until: (t + 3600) as u64,
        };
        let policy = ServicePolicy::new_issuer(
            "home.test".into(),
            "issuer.test".into(),
            vec![
                (
                    registration(&relay.0, "home.test", "relay"),
                    ServiceUse::Relay,
                ),
                (
                    registration(&inspector.0, "foreign.test", "inspector"),
                    ServiceUse::Introspection,
                ),
            ],
            vec![IssuerRoute {
                sender_home: "foreign.test".into(),
                audience: "foreign.test/attachments".into(),
                relay_client_id: "relay".into(),
                introspection_client_id: "inspector".into(),
            }],
        )
        .unwrap();
        let runtime = Runtime::new(None);
        let real = Self {
            runtime,
            ca,
            server,
            device,
            relay,
        };
        real.runtime
            .control()
            .replace_delegation_policy(real.registry(1), policy)
            .unwrap();
        real.runtime
            .control()
            .replace_retrieval_policy(
                RetrievalPolicy::new(
                    "home.test".into(),
                    vec![("foreign.test".into(), "foreign.test/attachments".into())],
                )
                .unwrap(),
            )
            .unwrap();
        real
    }
    fn recipient() -> Principal {
        Principal {
            account: "recipient@home.test".into(),
            device: 1,
            identity: [1; 32],
        }
    }
    fn registry(&self, generation: u64) -> Registry {
        let t = wall_now();
        Registry::new(
            generation,
            0,
            vec!["home.test".into(), "foreign.test".into()],
            vec![CredentialBinding {
                certificate_sha256: sha256(&self.device.0.to_der().unwrap()),
                principal: Self::recipient(),
                valid_from: t - 60,
                valid_until: t + 3600,
                approved: true,
                account_enabled: true,
                device_enabled: true,
                generation,
            }],
        )
        .unwrap()
    }
    fn server(&self) -> Server {
        Server::new(&self.server.0, &self.server.1, &self.ca.0).unwrap()
    }
    fn client(&self, address: SocketAddr, who: &(X509, PKey<Private>)) -> Client {
        Client::connect(
            address,
            "provider.test",
            sha256(&self.server.0.to_der().unwrap()),
            &self.ca.0,
            Some((&who.0, Some(&who.1))),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
    }
    fn reference(&self) -> (OperationReference, Instant) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::scope(|scope| {
            let peer = scope.spawn(|| self.client(address, &self.device));
            let mut session = self
                .server()
                .accept(&listener, &self.runtime, Arc::new(AtomicBool::new(false)))
                .unwrap();
            let _peer = peer.join().unwrap();
            let t = wall_now();
            let binding = Binding {
                sender: Principal {
                    account: "sender@foreign.test".into(),
                    device: 2,
                    identity: [2; 32],
                },
                recipient: Self::recipient(),
                id: [7; 32],
                size: 32,
                created: t,
                expires: t + 100,
            };
            let deadline = session.deadline;
            let record = session
                .retrieval_intent(binding, RetrievalAction::Status)
                .unwrap();
            drop(session);
            (
                self.runtime.control().provision_retrieval(&record).unwrap(),
                deadline,
            )
        })
    }
}
#[derive(Clone, Copy)]
enum Case {
    Healthy,
    Malformed,
    Cancelled,
    Deadline,
    PolicyStop,
    Late,
}
fn gated(case: Case) {
    let real = Real::new();
    let (reference, record_deadline) = real.reference();
    let request = if matches!(case, Case::Malformed) {
        b"POST /oauth/token HTTP/1.1\r\nHost: issuer.test\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 0\r\n\r\n".to_vec()
    } else {
        codec::encode_token_http(
            "issuer.test",
            &codec::UnverifiedTokenRequest {
                client_id: "relay".into(),
                scope: codec::UnverifiedReference::from_bytes(*reference.expose_bytes()),
            },
        )
        .unwrap()
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (late_tx, late_rx) = mpsc::channel();
    let (ack_tx, ack_rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    std::thread::scope(|scope| {
        let view = &real;
        let request_view = &request;
        let client = scope.spawn(move || {
            let mut client = view.client(address, &view.relay);
            client.stream.write_all(request_view).unwrap();
            if matches!(case, Case::Late) {
                late_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                client.stream.write_all(request_view).unwrap();
                ack_tx.send(()).unwrap();
            }
            let mut bytes = Vec::new();
            let mut buffer = [0; 1024];
            loop {
                match client.stream.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        assert!(bytes.len() + n <= codec::MAX_HEADER + codec::MAX_JSON);
                        bytes.extend_from_slice(&buffer[..n]);
                    }
                }
            }
            bytes
        });
        let mut session = real
            .server()
            .accept_service(&listener, &real.runtime, cancel.clone())
            .unwrap();
        session.stream.get_mut().response_write_gate = Some(Arc::new(ResponseWriteGate {
            entered: entered_tx,
            release: Mutex::new(release_rx),
        }));
        let service_deadline = session.deadline;
        let handler = scope.spawn(move || {
            let result = session.serve_http_inner();
            let attempted = session.attempted;
            drop(session);
            (result, attempted)
        });
        let deadline = entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        assert!(deadline <= service_deadline);
        if !matches!(case, Case::Malformed) {
            assert!(deadline <= record_deadline);
        }
        assert!(!handler.is_finished());
        let counts = real
            .runtime
            .state
            .authority
            .lock()
            .unwrap()
            .test_issuer_counts();
        assert_eq!(
            counts,
            if matches!(case, Case::Malformed) {
                (1, 1, 0)
            } else {
                (1, 1, 1)
            }
        );
        match case {
            Case::PolicyStop => {
                let start = Instant::now();
                real.runtime
                    .control()
                    .replace_registry(real.registry(2))
                    .unwrap();
                assert!(start.elapsed() < Duration::from_secs(1));
                assert!(!handler.is_finished());
                let start = Instant::now();
                real.runtime.control().stop().unwrap();
                assert!(start.elapsed() < Duration::from_secs(1));
                assert!(!handler.is_finished());
            }
            Case::Cancelled => cancel.store(true, Ordering::SeqCst),
            Case::Deadline => {
                while Instant::now() < deadline {
                    std::thread::sleep(
                        deadline
                            .saturating_duration_since(Instant::now())
                            .min(Duration::from_millis(100)),
                    );
                }
            }
            Case::Late => {
                late_tx.send(()).unwrap();
                ack_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                assert!(!handler.is_finished());
            }
            _ => (),
        }
        let released = Instant::now();
        release_tx.send(()).unwrap();
        let (result, attempted) = handler.join().unwrap();
        let bytes = client.join().unwrap();
        assert_eq!(attempted, 1);
        assert!(released.elapsed() < Duration::from_secs(2));
        assert!(!real.runtime.state.active.load(Ordering::SeqCst));
        match case {
            Case::Healthy | Case::Late => {
                assert!(result.is_ok());
                assert!(codec::decode_token_response_http(&bytes).is_ok());
                assert_eq!(
                    real.runtime
                        .state
                        .authority
                        .lock()
                        .unwrap()
                        .test_issuer_counts(),
                    (1, 1, 1)
                );
            }
            Case::Malformed => {
                assert!(result.is_err());
                assert!(bytes.starts_with(b"HTTP/1.1 400"));
            }
            Case::Cancelled => {
                assert!(matches!(result, Err(Error::Transport(Stage::Cancelled))));
                assert!(bytes.is_empty());
            }
            Case::Deadline => {
                assert!(matches!(result, Err(Error::Transport(Stage::Deadline))));
                assert!(bytes.is_empty());
            }
            Case::PolicyStop => {
                assert!(result.is_err());
                assert!(bytes.is_empty());
            }
        }
    });
}
#[test]
fn actual_tls_response_bio_success_charges_once() {
    gated(Case::Healthy)
}
#[test]
fn actual_tls_response_bio_malformed_request_charges_once() {
    gated(Case::Malformed)
}
#[test]
fn policy_replace_and_stop_progress_while_actual_response_bio_is_held() {
    gated(Case::PolicyStop)
}
#[test]
fn actual_tls_response_bio_checks_cancellation_after_gate_release() {
    gated(Case::Cancelled)
}
#[test]
fn actual_tls_response_bio_checks_original_token_deadline_after_gate_release() {
    gated(Case::Deadline)
}
#[test]
fn late_second_request_never_dispatches_or_receives_a_second_response() {
    gated(Case::Late)
}
#[test]
fn actual_stalled_http_read_allows_policy_replace_and_stop_after_observed_socket_retry() {
    use std::sync::atomic::AtomicUsize;
    let real = Real::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let retries = Arc::new(AtomicUsize::new(0));
    std::thread::scope(|scope| {
        let peer = scope.spawn(|| {
            let mut peer = real.client(address, &real.relay);
            peer.stream
                .write_all(b"POST /oauth/token HTTP/1.1\r\n")
                .unwrap();
            let mut scratch = [0; 32];
            assert!(peer.stream.read(&mut scratch).is_err() || scratch == [0; 32]);
        });
        let mut session = real
            .server()
            .accept_service(&listener, &real.runtime, Arc::new(AtomicBool::new(false)))
            .unwrap();
        session.stream.get_mut().read_retries = Some(retries.clone());
        let handler = scope.spawn(move || {
            let result = session.serve_http_inner();
            let attempts = session.attempted;
            drop(session);
            (result, attempts)
        });
        let end = Instant::now() + Duration::from_secs(3);
        while retries.load(Ordering::SeqCst) == 0 && Instant::now() < end {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(
            retries.load(Ordering::SeqCst) > 0,
            "no actual socket read WouldBlock/TimedOut observed"
        );
        assert!(!handler.is_finished());
        let start = Instant::now();
        real.runtime
            .control()
            .replace_registry(real.registry(2))
            .unwrap();
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(!handler.is_finished());
        real.runtime.control().stop().unwrap();
        let (result, attempts) = handler.join().unwrap();
        assert!(result.is_err());
        assert_eq!(attempts, 1);
        assert!(start.elapsed() < Duration::from_secs(2));
        assert!(!real.runtime.state.active.load(Ordering::SeqCst));
        peer.join().unwrap();
    });
}
#[test]
fn wire_bio_precise_wall_expiry_and_rollback_are_sticky_without_host_clock_changes() {
    for rollback in [false, true] {
        let mut lifetime = WireLifetime {
            expires: Duration::from_secs(110),
            highwater: Duration::from_millis(100900),
            invalid: false,
        };
        assert!(lifetime.check(Duration::from_millis(101100)).is_ok());
        assert!(
            lifetime
                .check(if rollback {
                    Duration::from_millis(101000)
                } else {
                    Duration::from_secs(110)
                })
                .is_err()
        );
        assert!(lifetime.check(Duration::from_secs(102)).is_err());
    }
}
fn publication_fixture() -> (
    Real,
    (X509, PKey<Private>),
    codec::UnverifiedPublicationQuery,
) {
    let real = Real::new();
    let now = wall_now();
    let observer = cert(
        Some((&real.ca.0, &real.ca.1)),
        "observer",
        301,
        now as i64 - 60,
        now as i64 + 3600,
    );
    real.runtime
        .control()
        .replace_delegation_policy(
            real.registry(2),
            ServicePolicy::new(
                "home.test".into(),
                vec![ServiceRegistration {
                    home: "foreign.test".into(),
                    client_id: "observer".into(),
                    dns_san: "provider.test".into(),
                    certificate_sha256: sha256(&observer.0.to_der().unwrap()),
                    enabled: true,
                    valid_from: now - 60,
                    valid_until: now + 3600,
                }],
            )
            .unwrap(),
        )
        .unwrap();
    let p = |account: &str| codec::UnverifiedPrincipal {
        account: account.into(),
        device: 1,
        identity: "01".repeat(32),
    };
    let query = codec::UnverifiedPublicationQuery {
        client_id: "observer".into(),
        binding: codec::UnverifiedBinding {
            sender: p("sender@foreign.test"),
            recipient: p("recipient@home.test"),
            id: "22".repeat(32),
            size: 32,
            created: now - 1,
            expires: now + 100,
        },
        action: codec::UnverifiedPublicationAction::Begin,
    };
    (real, observer, query)
}
#[test]
fn publication_actual_tls_response_bio_gate_rechecks_policy_stop_cancel_and_original_cycle_deadline()
 {
    use crate::services::PublicationWireProfile;
    for case in 0..5 {
        let (real, observer, query) = publication_fixture();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let request = codec::encode_publication_query_http("home.test", &query).unwrap();
        std::thread::scope(|scope| {
            let peer = scope.spawn(|| {
                let mut client = real.client(addr, &observer);
                client.stream.write_all(&request).unwrap();
                let mut bytes = Vec::new();
                let mut scratch = [0; 2048];
                loop {
                    match client.stream.read(&mut scratch) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            assert!(bytes.len() + n <= codec::MAX_HEADER + codec::MAX_JSON);
                            bytes.extend_from_slice(&scratch[..n]);
                        }
                    }
                }
                bytes
            });
            let mut session = real
                .server()
                .accept_service(&listener, &real.runtime, cancel.clone())
                .unwrap();
            let original = session.deadline;
            session.stream.get_mut().response_write_gate = Some(Arc::new(ResponseWriteGate {
                entered: entered_tx,
                release: Mutex::new(release_rx),
            }));
            let handler = scope.spawn(move || {
                session.serve_publication_http_once(
                    &PublicationWireProfile::new("home.test".into(), vec!["observer".into()])
                        .unwrap(),
                )
            });
            assert_eq!(
                entered_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
                original
            );
            assert!(!handler.is_finished());
            let changed = Instant::now();
            // This lock acquisition is an explicit lock-freedom oracle at the BIO gate.
            drop(real.runtime.state.authority.lock().unwrap());
            match case {
                0 => real
                    .runtime
                    .control()
                    .replace_registry(real.registry(3))
                    .unwrap(),
                1 => real.runtime.control().stop().unwrap(),
                2 => cancel.store(true, Ordering::SeqCst),
                3 => {
                    while Instant::now() < original {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
                _ => (),
            };
            if case != 3 {
                assert!(changed.elapsed() < Duration::from_secs(1));
            }
            assert!(!handler.is_finished());
            release_tx.send(()).unwrap();
            let result = handler.join().unwrap();
            let bytes = peer.join().unwrap();
            if case == 4 {
                assert!(result.is_ok());
                match codec::decode_publication_response_http(&bytes).unwrap() {
                    codec::UnverifiedPublicationObservation::Current { query: got, .. } => {
                        assert_eq!(*got, query)
                    }
                    _ => panic!("current metadata missing"),
                };
            } else {
                assert!(result.is_err());
                assert!(bytes.is_empty());
                if case == 3 {
                    assert_eq!(result, Err(Error::Transport(Stage::Deadline)));
                }
            }
            assert!(!real.runtime.state.active.load(Ordering::SeqCst));
        });
    }
}
#[test]
fn publication_actual_partial_tls_request_observes_retry_then_policy_cancel_stop_release_admission()
{
    use crate::services::PublicationWireProfile;
    use std::sync::atomic::AtomicUsize;
    for case in 0..3 {
        let (real, observer, _) = publication_fixture();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let retries = Arc::new(AtomicUsize::new(0));
        let cancel = Arc::new(AtomicBool::new(false));
        std::thread::scope(|scope| {
            let peer = scope.spawn(|| {
                let mut client = real.client(addr, &observer);
                client
                    .stream
                    .write_all(b"POST /faction/publication-current HTTP/1.1\r\n")
                    .unwrap();
                let mut scratch = [0; 8192];
                assert!(client.stream.read(&mut scratch).is_err() || scratch == [0; 8192]);
            });
            let mut session = real
                .server()
                .accept_service(&listener, &real.runtime, cancel.clone())
                .unwrap();
            session.stream.get_mut().read_retries = Some(retries.clone());
            let handler = scope.spawn(move || {
                session.serve_publication_http_once(
                    &PublicationWireProfile::new("home.test".into(), vec!["observer".into()])
                        .unwrap(),
                )
            });
            let end = Instant::now() + Duration::from_secs(2);
            while retries.load(Ordering::SeqCst) == 0 && Instant::now() < end {
                std::thread::sleep(Duration::from_millis(5));
            }
            assert!(retries.load(Ordering::SeqCst) > 0);
            assert!(!handler.is_finished());
            let changed = Instant::now();
            match case {
                0 => real
                    .runtime
                    .control()
                    .replace_registry(real.registry(3))
                    .unwrap(),
                1 => real.runtime.control().stop().unwrap(),
                _ => cancel.store(true, Ordering::SeqCst),
            };
            assert!(handler.join().unwrap().is_err());
            assert!(changed.elapsed() < Duration::from_secs(2));
            assert!(!real.runtime.state.active.load(Ordering::SeqCst));
            peer.join().unwrap();
        });
    }
}
