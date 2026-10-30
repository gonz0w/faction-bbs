//! Actual outbound TLS/read retries; ordinary ingress uses a distinct tracked socket.
use super::*;
use crate::{
    authority::{CredentialBinding, Registry},
    tls::issuer_lifecycle_tests::cert,
};
use std::sync::{atomic::AtomicUsize, mpsc};
struct Peer {
    ca: (X509, PKey<Private>),
    server: (X509, PKey<Private>),
    client: (X509, PKey<Private>),
}
impl Peer {
    fn new() -> Self {
        let now = wall_now() as i64;
        let ca = cert(None, "CA", 201, now - 60, now + 3600);
        let server = cert(
            Some((&ca.0, &ca.1)),
            "provider.test",
            202,
            now - 60,
            now + 3600,
        );
        let client = cert(Some((&ca.0, &ca.1)), "client", 203, now - 60, now + 3600);
        Self { ca, server, client }
    }
    fn registry(&self, generation: u64) -> Registry {
        Registry::new(
            generation,
            0,
            vec!["local.test".into()],
            vec![CredentialBinding {
                certificate_sha256: sha256(&self.client.0.to_der().unwrap()),
                principal: faction_files::private_blobs::Principal {
                    account: "device@local.test".into(),
                    device: 1,
                    identity: [1; 32],
                },
                valid_from: wall_now() - 60,
                valid_until: wall_now() + 3600,
                approved: true,
                account_enabled: true,
                device_enabled: true,
                generation,
            }],
        )
        .unwrap()
    }
    fn endpoint(&self, address: SocketAddr, retries: Arc<AtomicUsize>) -> HttpEndpoint {
        let credentials = ClientCredentials::from_pem(
            &self.ca.0.to_pem().unwrap(),
            &self.client.0.to_pem().unwrap(),
            &self.client.1.private_key_to_pem_pkcs8().unwrap(),
        )
        .unwrap();
        let mut ep = HttpEndpoint::new(
            address,
            "remote.test".into(),
            "provider.test".into(),
            sha256(&self.server.0.to_der().unwrap()),
            credentials,
        )
        .unwrap();
        ep.read_retries = Some(retries);
        ep
    }
    fn acceptor(&self) -> SslAcceptor {
        let mut b = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls_server()).unwrap();
        b.set_min_proto_version(Some(SslVersion::TLS1_3)).unwrap();
        b.set_max_proto_version(Some(SslVersion::TLS1_3)).unwrap();
        b.set_certificate(&self.server.0).unwrap();
        b.set_private_key(&self.server.1).unwrap();
        b.cert_store_mut().add_cert(self.ca.0.clone()).unwrap();
        b.set_verify(SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT);
        b.build()
    }
}
fn query() -> codec::UnverifiedPublicationQuery {
    let p = |home: &str| codec::UnverifiedPrincipal {
        account: format!("member@{home}"),
        device: 1,
        identity: "11".repeat(32),
    };
    codec::UnverifiedPublicationQuery {
        client_id: "observer".into(),
        binding: codec::UnverifiedBinding {
            sender: p("local.test"),
            recipient: p("remote.test"),
            id: "22".repeat(32),
            size: 32,
            created: wall_now() - 1,
            expires: wall_now() + 100,
        },
        action: codec::UnverifiedPublicationAction::Begin,
    }
}
fn read_stall(case: usize, handshake: bool) {
    let local = Peer::new();
    let remote = Peer::new();
    let runtime = Runtime::new(Some(local.registry(1)));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let retries = Arc::new(AtomicUsize::new(0));
    let ep = remote.endpoint(listener.local_addr().unwrap(), retries.clone());
    let (ready_tx, ready_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let deadline = Instant::now()
        + if case == 3 {
            Duration::from_millis(700)
        } else {
            Duration::from_secs(4)
        };
    std::thread::scope(|scope| {
        let server = scope.spawn(move || {
            let (socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(4)))
                .unwrap();
            if handshake {
                ready_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(4)).unwrap();
                drop(socket);
            } else {
                let mut stream = remote.acceptor().accept(socket).unwrap();
                let mut scratch = [0; 16384];
                assert!(stream.read(&mut scratch).unwrap() > 0);
                stream.write_all(b"HTTP/1.1 200 OK\r\n").unwrap();
                ready_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(4)).unwrap();
            }
        });
        let result = scope.spawn(|| {
            runtime
                .control()
                .query_publication(&ep, &query(), deadline, cancel.clone())
        });
        ready_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let end = Instant::now() + Duration::from_secs(2);
        while retries.load(Ordering::SeqCst) == 0 && Instant::now() < end {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(
            retries.load(Ordering::SeqCst) > 0,
            "no actual outbound socket read retry observed"
        );
        assert!(!result.is_finished());
        // Actual ordinary TLS admission on the SAME Runtime while callback reads wait.
        let ordinary = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = ordinary.local_addr().unwrap();
        let local_view = &local;
        let device = scope.spawn(move || {
            Client::connect(
                addr,
                "provider.test",
                sha256(&local_view.server.0.to_der().unwrap()),
                &local_view.ca.0,
                Some((&local_view.client.0, Some(&local_view.client.1))),
                Arc::new(AtomicBool::new(false)),
            )
        });
        let incoming = Server::new(&local.server.0, &local.server.1, &local.ca.0)
            .unwrap()
            .accept(&ordinary, &runtime, Arc::new(AtomicBool::new(false)))
            .unwrap();
        drop(device.join().unwrap().unwrap());
        drop(incoming);
        assert!(!runtime.state.active.load(Ordering::SeqCst));
        let changed = Instant::now();
        match case {
            0 => runtime
                .control()
                .replace_registry(local.registry(2))
                .unwrap(),
            1 => runtime.control().stop().unwrap(),
            2 => cancel.store(true, Ordering::SeqCst),
            _ => (),
        }
        let answer = result.join().unwrap();
        assert!(answer.is_err());
        assert!(changed.elapsed() < Duration::from_secs(2));
        assert!(Instant::now() < deadline + Duration::from_secs(1));
        release_tx.send(()).unwrap();
        server.join().unwrap();
    });
}
#[test]
fn actual_outbound_partial_tls_read_releases_for_current_policy_change_and_ordinary_admission() {
    read_stall(0, false)
}
#[test]
fn actual_outbound_partial_tls_read_releases_for_stop_and_ordinary_admission() {
    read_stall(1, false)
}
#[test]
fn actual_outbound_partial_tls_read_releases_for_cancellation_and_ordinary_admission() {
    read_stall(2, false)
}
#[test]
fn actual_outbound_partial_tls_read_spends_original_deadline_and_allows_ordinary_admission() {
    read_stall(3, false)
}
#[test]
fn actual_outbound_handshake_read_observes_retry_and_current_policy_invalidation() {
    read_stall(0, true)
}
