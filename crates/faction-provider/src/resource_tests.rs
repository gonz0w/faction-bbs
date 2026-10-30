//! Actual TLS request BIO scheduling, not kernel pressure; clocks are synthetic where marked.
use super::*;
use crate::{
    Runtime,
    authority::{CredentialBinding, Registry},
    tls::{Client, ClientCredentials, HttpEndpoint, Server, test_certificate as cert},
};
use openssl::{
    pkey::{PKey, Private},
    ssl::{SslAcceptor, SslMethod, SslVerifyMode, SslVersion},
};
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    os::unix::fs::DirBuilderExt,
    path::PathBuf,
    sync::mpsc,
    thread,
};
struct Fixture {
    ca: (X509, PKey<Private>),
    server: (X509, PKey<Private>),
    peer: (X509, PKey<Private>),
    relay: (X509, PKey<Private>),
    outgoing: (X509, PKey<Private>),
    device: (X509, PKey<Private>),
    runtime: Option<Runtime>,
    listener: TcpListener,
    root: PathBuf,
    binding: Binding,
}
impl Fixture {
    fn runtime(&self) -> &Runtime {
        self.runtime.as_ref().unwrap()
    }
    #[cfg(feature = "durable-policy")]
    fn make_durable(&mut self) {
        drop(self.runtime.take());
        fs::DirBuilder::new()
            .mode(0o700)
            .create(self.root.join("policy"))
            .unwrap();
        self.runtime =
            Some(Runtime::initialize_durable(&self.root.join("policy"), self.registry(1)).unwrap());
        self.install(2);
    }

    fn new() -> Self {
        let t = tls::wall_now() as i64;
        let ca = cert(None, "CA", 501, t - 60, t + 3600);
        let leaf = |id, name| cert(Some((&ca.0, &ca.1)), name, id, t - 60, t + 3600);
        let server = leaf(502, "server");
        let peer = leaf(503, "issuer");
        let relay = leaf(504, "relay");
        let outgoing = leaf(505, "outgoing");
        let device = leaf(506, "device");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let runtime = Runtime::new(None);
        let root = std::env::temp_dir().join(format!(
            "f315-resource-OWNED-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        fs::write(root.join("OWNED"), b"Faction315 resource TLS test fixture").unwrap();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(root.join("store"))
            .unwrap();
        let binding = Binding {
            sender: Principal {
                account: "sender@local.test".into(),
                device: 1,
                identity: [1; 32],
            },
            recipient: Principal {
                account: "recipient@remote.test".into(),
                device: 2,
                identity: [2; 32],
            },
            id: sha256(&[9; 32]),
            size: 32,
            created: t as u64 - 5,
            expires: t as u64 + 100,
        };
        let f = Self {
            ca,
            server,
            peer,
            relay,
            outgoing,
            device,
            runtime: Some(runtime),
            listener,
            root,
            binding,
        };
        f.install(2);
        f
    }
    fn registry(&self, g: u64) -> Registry {
        Registry::new(
            g,
            0,
            vec!["local.test".into(), "remote.test".into()],
            vec![CredentialBinding {
                certificate_sha256: sha256(&self.device.0.to_der().unwrap()),
                principal: self.binding.sender.clone(),
                valid_from: tls::wall_now() - 60,
                valid_until: tls::wall_now() + 3600,
                approved: true,
                account_enabled: true,
                device_enabled: true,
                generation: g,
            }],
        )
        .unwrap()
    }
    fn policy(&self) -> ServicePolicy {
        let e = |leaf: &X509, home: &str, id: &str| ServiceRegistration {
            home: home.into(),
            client_id: id.into(),
            dns_san: "provider.test".into(),
            certificate_sha256: sha256(&leaf.to_der().unwrap()),
            enabled: true,
            valid_from: tls::wall_now() - 60,
            valid_until: tls::wall_now() + 3600,
        };
        ServicePolicy::new_resource(
            "local.test".into(),
            vec![
                (
                    e(&self.relay.0, "remote.test", "relay"),
                    ServiceUse::ResourceClient,
                ),
                (
                    e(&self.outgoing.0, "local.test", "outgoing"),
                    ServiceUse::OutboundIntrospection,
                ),
                (
                    e(&self.peer.0, "remote.test", "endpoint"),
                    ServiceUse::IssuerEndpoint,
                ),
            ],
            vec![ResourceAssociation {
                recipient_home: "remote.test".into(),
                issuer: "issuer.test".into(),
                audience: "local.test/attachments".into(),
                relay_client_id: "relay".into(),
                outbound_client_id: "outgoing".into(),
                endpoint_id: "endpoint".into(),
                address: self.listener.local_addr().unwrap(),
                http_host: "issuer.test".into(),
                ca_sha256: sha256(&self.ca.0.to_der().unwrap()),
                remote_generation: 2,
                remote_reset: 0,
            }],
        )
        .unwrap()
    }
    fn install(&self, g: u64) {
        self.runtime()
            .control()
            .replace_delegation_policy(self.registry(g), self.policy())
            .unwrap();
    }
    fn client(
        &self,
        address: std::net::SocketAddr,
        who: &(X509, PKey<Private>),
    ) -> Result<Client, tls::Stage> {
        Client::connect(
            address,
            "provider.test",
            sha256(&self.server.0.to_der().unwrap()),
            &self.ca.0,
            Some((&who.0, Some(&who.1))),
            Arc::new(AtomicBool::new(false)),
        )
    }
    fn prepare(&self, attempts: usize) -> Result<PendingRetrieval, Error> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::scope(|scope| {
            let client = scope.spawn(|| self.client(address, &self.relay));
            let mut session = Server::new(&self.server.0, &self.server.1, &self.ca.0)
                .unwrap()
                .accept_service(&listener, self.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            let _client = client.join().unwrap().unwrap();
            for _ in 0..attempts {
                assert!(
                    session
                        .observe_recipient(self.binding.clone(), PublicationAction::Begin)
                        .is_err()
                );
            }
            session.prepare_retrieval(
                self.binding.clone(),
                Request::GetChunk {
                    id: self.binding.id,
                    offset: 0,
                },
                codec::UnverifiedToken::from_bytes([42; 32]),
            )
        })
    }
    fn endpoint(&self) -> HttpEndpoint {
        HttpEndpoint::new(
            self.listener.local_addr().unwrap(),
            "issuer.test".into(),
            "provider.test".into(),
            sha256(&self.peer.0.to_der().unwrap()),
            ClientCredentials::from_pem(
                &self.ca.0.to_pem().unwrap(),
                &self.outgoing.0.to_pem().unwrap(),
                &self.outgoing.1.private_key_to_pem_pkcs8().unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn acceptor(&self) -> SslAcceptor {
        let mut b = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls_server()).unwrap();
        b.set_min_proto_version(Some(SslVersion::TLS1_3)).unwrap();
        b.set_max_proto_version(Some(SslVersion::TLS1_3)).unwrap();
        b.set_certificate(&self.peer.0).unwrap();
        b.set_private_key(&self.peer.1).unwrap();
        b.cert_store_mut().add_cert(self.ca.0.clone()).unwrap();
        b.set_verify(SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT);
        b.build()
    }
    fn store(&self) -> Mutex<Store> {
        struct Setup<'a>(&'a Binding);
        impl Access for Setup<'_> {
            fn require(
                &self,
                a: &Principal,
                b: &Binding,
                action: Action,
                _now: u64,
            ) -> Result<(), BlobError> {
                if a == &self.0.sender
                    && b == self.0
                    && matches!(action, Action::Begin | Action::PutChunk | Action::Commit)
                {
                    Ok(())
                } else {
                    Err(BlobError::Denied)
                }
            }
        }
        let mut s = Store::open(&self.root.join("store")).unwrap();
        let setup = Setup(&self.binding);
        for r in [
            Request::Begin(self.binding.clone()),
            Request::PutChunk {
                id: self.binding.id,
                offset: 0,
                bytes: vec![9; 32],
            },
            Request::Commit {
                id: self.binding.id,
            },
        ] {
            s.dispatch(&self.binding.sender, &setup, r, tls::wall_now())
                .unwrap();
        }
        drop(s);
        Mutex::new(Store::open(&self.root.join("store")).unwrap())
    }
    fn ordinary_progress(&self, store: &Mutex<Store>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::scope(|scope| {
            let peer = scope.spawn(|| self.client(address, &self.device));
            let mut session = Server::new(&self.server.0, &self.server.1, &self.ca.0)
                .unwrap()
                .accept(&listener, self.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            let _peer = peer.join().unwrap().unwrap();
            let mut b = self.binding.clone();
            b.recipient = b.sender.clone();
            b.id = sha256(&[7; 32]);
            let mut s = store.try_lock().expect("callback held Store lock");
            assert!(
                session
                    .dispatch(&mut s, &b.sender.account.clone(), Request::Begin(b))
                    .is_ok()
            );
        });
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(runtime) = &self.runtime {
            runtime.stop();
        }
        assert_eq!(
            fs::read(self.root.join("OWNED")).unwrap(),
            b"Faction315 resource TLS test fixture"
        );
        fs::remove_dir_all(&self.root).unwrap();
        assert!(!self.root.exists());
    }
}
fn gated(case: usize) {
    gated_inner(case, false)
}
fn gated_inner(case: usize, durable: bool) {
    let f = Fixture::new();
    #[cfg(feature = "durable-policy")]
    let f = {
        let mut candidate = f;
        if durable {
            candidate.make_durable();
        }
        candidate
    };
    #[cfg(not(feature = "durable-policy"))]
    assert!(!durable);
    let store = f.store();
    let pending = f.prepare(0).unwrap();
    assert_eq!(pending.context.attempted, 1);
    let original = pending.context.deadline;
    let cancel = pending.context.cancel.clone();
    let context = pending.context.clone();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let mut endpoint = f.endpoint();
    endpoint.hold_request(entered_tx, release_rx);
    thread::scope(|scope| {
        let server = scope.spawn(|| {
            let (socket, _) = f.listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut peer = f.acceptor().accept(socket).unwrap();
            let mut scratch = [0; 1024];
            let _ = peer.read(&mut scratch);
        });
        let operation = scope.spawn(|| pending.execute(&endpoint, &store));
        assert_eq!(
            entered_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            original
        );
        assert!(!operation.is_finished());
        f.ordinary_progress(&store);
        let before = fs::read(f.root.join("store/private-blobs.snapshot")).unwrap();
        let start = Instant::now();
        match case {
            0 => f.install(3),
            1 => f.runtime().control().stop().unwrap(),
            2 => cancel.store(true, Ordering::SeqCst),
            3 => {
                while Instant::now() < original {
                    thread::sleep(Duration::from_millis(10));
                }
            }
            _ => {
                context.clock.lock().unwrap().highwater =
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::from_secs(60);
            }
        }
        if case != 3 {
            assert!(start.elapsed() < Duration::from_secs(1));
        }
        assert!(!operation.is_finished());
        assert_eq!(
            f.runtime()
                .state
                .authority
                .lock()
                .unwrap()
                .records
                .charged_len(),
            1
        );
        if case != 1 {
            assert!(matches!(f.prepare(0), Err(Error::Store(BlobError::Busy))));
        }
        release_tx.send(()).unwrap();
        assert!(operation.join().unwrap().is_err());
        server.join().unwrap();
        assert_eq!(
            f.runtime()
                .state
                .authority
                .lock()
                .unwrap()
                .records
                .charged_len(),
            0
        );
        assert_eq!(
            fs::read(f.root.join("store/private-blobs.snapshot")).unwrap(),
            before
        );
        if case == 4 {
            let mut clock = context.clock.lock().unwrap();
            assert!(clock.invalid);
            clock.highwater = Duration::ZERO;
            drop(clock);
            assert!(matches!(
                context.moment(),
                Err(Error::Store(BlobError::Clock))
            ));
        }
        if case != 1 {
            assert!(f.prepare(0).is_ok());
        }
    });
}
#[test]
fn actual_callback_request_bio_hold_keeps_physical_reservation_during_policy_invalidation_and_allows_ordinary_ingress_store()
 {
    gated(0)
}
#[test]
fn actual_callback_request_bio_hold_keeps_physical_reservation_during_stop() {
    gated(1)
}
#[test]
fn actual_callback_request_bio_hold_keeps_physical_reservation_during_cancellation() {
    gated(2)
}
#[test]
fn actual_callback_request_bio_hold_retains_original_deadline_without_a_fresh_cycle() {
    gated(3)
}
#[test]
fn actual_callback_request_bio_hold_refuses_synthetic_clock_rollback() {
    gated(4)
}
#[test]
fn original_incoming_attempt_budget_is_captured_and_eighth_ninth_operations_remain_distinct() {
    let f = Fixture::new();
    let p = f.prepare(7).unwrap();
    assert_eq!(p.context.attempted, 8);
    drop(p);
    assert!(matches!(
        f.prepare(8),
        Err(Error::Transport(tls::Stage::Limit))
    ));
}
#[cfg(feature = "durable-policy")]
#[test]
fn actual_durable_policy_install_does_not_clear_a_live_physical_callback_reservation() {
    gated_inner(0, true)
}
#[test]
fn pending_retains_no_runtime_owner_and_cannot_connect_after_original_runtime_drop() {
    let mut f = Fixture::new();
    let store = f.store();
    let pending = f.prepare(0).unwrap();
    let endpoint = f.endpoint();
    drop(f.runtime.take());
    f.listener.set_nonblocking(true).unwrap();
    assert!(pending.execute(&endpoint, &store).is_err());
    assert!(matches!(f.listener.accept(),Err(ref e) if e.kind()==std::io::ErrorKind::WouldBlock));
}
#[cfg(feature = "durable-policy")]
#[test]
fn durable_restart_preserves_public_permissions_but_no_old_continuation_or_bearer() {
    let mut f = Fixture::new();
    f.make_durable();
    let store = f.store();
    let pending = f.prepare(0).unwrap();
    let endpoint = f.endpoint();
    drop(f.runtime.take());
    let bytes = fs::read(f.root.join("policy/provider-policy.snapshot")).unwrap();
    assert_eq!(&bytes[..8], b"FPPY0004");
    assert!(!bytes.windows(32).any(|v| v == [42; 32]));
    let text = codec::UnverifiedToken::from_bytes([42; 32]);
    assert!(
        !bytes
            .windows(text.expose().len())
            .any(|v| v == text.expose().as_bytes())
    );
    f.runtime = Some(Runtime::reopen_durable(&f.root.join("policy")).unwrap());
    assert_eq!(f.runtime().policy_summary().unwrap().binding_count, 4);
    assert!(pending.execute(&endpoint, &store).is_err());
    assert!(f.prepare(0).is_ok());
}
fn read_stall(case: usize) {
    use std::sync::atomic::AtomicUsize;
    let f = Fixture::new();
    if case == 4 {
        let mut entries = f.registry(3).bindings().to_vec();
        entries[0].valid_until = tls::wall_now() + 2;
        f.runtime()
            .control()
            .replace_delegation_policy(
                Registry::new(
                    3,
                    0,
                    vec!["local.test".into(), "remote.test".into()],
                    entries,
                )
                .unwrap(),
                f.policy(),
            )
            .unwrap();
    }
    let store = f.store();
    let pending = f.prepare(0).unwrap();
    let original = pending.context.deadline;
    let cancel = pending.context.cancel.clone();
    let retries = Arc::new(AtomicUsize::new(0));
    let mut endpoint = f.endpoint();
    endpoint.observe_resource_reads(retries.clone());
    let (release_tx, release_rx) = mpsc::channel();
    thread::scope(|scope| {
        let view = &f;
        let server = scope.spawn(move || {
            let (socket, _) = view.listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut peer = view.acceptor().accept(socket).unwrap();
            let mut scratch = [0; 8192];
            assert!(peer.read(&mut scratch).unwrap() > 0);
            peer.write_all(b"HTTP/1.1 200 OK\r\n").unwrap();
            release_rx.recv_timeout(Duration::from_secs(12)).unwrap();
        });
        let operation = scope.spawn(|| pending.execute(&endpoint, &store));
        let end = Instant::now() + Duration::from_secs(2);
        while retries.load(Ordering::SeqCst) == 0 && Instant::now() < end {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            retries.load(Ordering::SeqCst) > 0,
            "no actual callback socket read retry observed"
        );
        assert!(!operation.is_finished());
        assert!(matches!(f.prepare(0), Err(Error::Store(BlobError::Busy))));
        f.ordinary_progress(&store);
        let before = fs::read(f.root.join("store/private-blobs.snapshot")).unwrap();
        let changed = Instant::now();
        match case {
            0 => {
                let mut entries = f.registry(3).bindings().to_vec();
                entries[0].approved = false;
                f.runtime()
                    .control()
                    .replace_delegation_policy(
                        Registry::new(
                            3,
                            0,
                            vec!["local.test".into(), "remote.test".into()],
                            entries,
                        )
                        .unwrap(),
                        f.policy(),
                    )
                    .unwrap();
            }
            1 => f.runtime().control().stop().unwrap(),
            2 => cancel.store(true, Ordering::SeqCst),
            _ => (),
        };
        assert!(operation.join().unwrap().is_err());
        if case < 3 {
            assert!(changed.elapsed() < Duration::from_secs(2));
        }
        assert!(Instant::now() < original + Duration::from_secs(1));
        assert_eq!(
            f.runtime()
                .state
                .authority
                .lock()
                .unwrap()
                .records
                .charged_len(),
            0
        );
        assert_eq!(
            fs::read(f.root.join("store/private-blobs.snapshot")).unwrap(),
            before
        );
        release_tx.send(()).unwrap();
        server.join().unwrap();
    });
}
#[test]
fn actual_partial_callback_read_allows_ordinary_ingress_store_then_current_sender_revocation_refuses()
 {
    read_stall(0)
}
#[test]
fn actual_partial_callback_read_allows_ordinary_ingress_store_then_stop_refuses() {
    read_stall(1)
}
#[test]
fn actual_partial_callback_read_allows_ordinary_ingress_store_then_cancel_refuses() {
    read_stall(2)
}
#[test]
fn actual_partial_callback_read_spends_original_deadline_without_renewal() {
    read_stall(3)
}
#[test]
fn actual_partial_callback_read_refuses_current_sender_expiry_without_a_policy_update() {
    read_stall(4)
}
impl Fixture {
    fn metadata(&self) -> codec::UnverifiedIntrospection {
        let b = &self.binding;
        let principal = |p: &Principal| codec::UnverifiedPrincipal {
            account: p.account.clone(),
            device: p.device,
            identity: hex(&p.identity),
        };
        codec::UnverifiedIntrospection::Retrieval {
            claims: codec::UnverifiedClaims {
                issuer: "issuer.test".into(),
                issuer_home: "remote.test".into(),
                client_id: "relay".into(),
                audience: "local.test/attachments".into(),
                subject: b.recipient.account.clone(),
                issued_at: tls::wall_now(),
                expires_at: tls::wall_now() + 5,
                generation: 2,
                reset: 0,
                certificate_thumbprint: codec::UnverifiedReference::from_bytes(sha256(
                    &self.relay.0.to_der().unwrap(),
                )),
                binding: codec::UnverifiedBinding {
                    sender: principal(&b.sender),
                    recipient: principal(&b.recipient),
                    id: hex(&b.id),
                    size: b.size,
                    created: b.created,
                    expires: b.expires,
                },
            },
            action: codec::UnverifiedRetrievalAction::GetChunk { offset: 0 },
        }
    }
    fn complete(&self, pending: PendingRetrieval, store: &Mutex<Store>) -> Result<Response, Error> {
        thread::scope(|scope| {
            let issuer = scope.spawn(|| {
                let (socket, _) = self.listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut peer = self.acceptor().accept(socket).unwrap();
                let mut scratch = [0; 8192];
                assert!(peer.read(&mut scratch).unwrap() > 0);
                peer.write_all(
                    &codec::encode_introspection_response_http(&self.metadata()).unwrap(),
                )
                .unwrap();
            });
            let result = pending.execute(&self.endpoint(), store);
            issuer.join().unwrap();
            result
        })
    }
}
#[test]
fn private_dispatch_checks_actual_stored_full_binding_and_exact_immutable_request_offset() {
    for changed_store in [false, true] {
        let mut f = Fixture::new();
        let original = f.binding.clone();
        if changed_store {
            f.binding.expires += 1;
        }
        let store = f.store();
        f.binding = original;
        let mut pending = f.prepare(0).unwrap();
        if !changed_store {
            pending.request = Request::GetChunk {
                id: f.binding.id,
                offset: 1,
            };
        }
        let before = fs::read(f.root.join("store/private-blobs.snapshot")).unwrap();
        assert!(f.complete(pending, &store).is_err());
        assert_eq!(
            fs::read(f.root.join("store/private-blobs.snapshot")).unwrap(),
            before
        );
    }
}
#[test]
fn prepare_does_not_return_a_continuation_until_exact_original_tracked_tls_cleanup_finishes() {
    let f = Fixture::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let state = f.runtime().state.clone();
    thread::scope(|scope| {
        let peer = scope.spawn(|| f.client(address, &f.relay));
        let session = Server::new(&f.server.0, &f.server.1, &f.ca.0)
            .unwrap()
            .accept_service(&listener, f.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let mut peer = peer.join().unwrap().unwrap();
        let tracking = state.socket.lock().unwrap();
        let (returned_tx, returned_rx) = mpsc::channel();
        let binding_copy = f.binding.clone();
        let prepare = scope.spawn(move || {
            let id = binding_copy.id;
            let result = session.prepare_retrieval(
                binding_copy,
                Request::GetChunk { id, offset: 0 },
                codec::UnverifiedToken::from_bytes([42; 32]),
            );
            returned_tx.send(result).unwrap();
        });
        let end = Instant::now() + Duration::from_secs(2);
        while state.authority.lock().unwrap().records.charged_len() == 0 && Instant::now() < end {
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(state.authority.lock().unwrap().records.charged_len(), 1);
        assert!(state.active.load(Ordering::SeqCst));
        assert!(returned_rx.try_recv().is_err());
        drop(tracking);
        prepare.join().unwrap();
        let pending = returned_rx
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .unwrap();
        assert!(!state.active.load(Ordering::SeqCst));
        assert!(state.socket.lock().unwrap().is_none());
        assert!(peer.request(&Request::Status { id: f.binding.id }).is_err());
        drop(pending);
        assert_eq!(state.authority.lock().unwrap().records.charged_len(), 0);
    });
}
