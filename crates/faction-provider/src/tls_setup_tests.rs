//! Initialization-work contract, separate from performance or historical deadlines.
use super::*;
use crate::{
    authority::{CredentialBinding, Registry},
    setup_trace::{Connection as SetupConnection, Kind, Phase, Role, Trace},
};
use faction_files::private_blobs::Principal;

#[test]
fn credential_clone_reuses_one_bootstrap_for_two_authenticated_connections() {
    let now = wall_now();
    let ca = test_certificate(
        None,
        "setup-contract-ca",
        501,
        now as i64 - 60,
        now as i64 + 3600,
    );
    let server_identity = test_certificate(
        Some((&ca.0, &ca.1)),
        "provider.test",
        502,
        now as i64 - 60,
        now as i64 + 3600,
    );
    let device = test_certificate(
        Some((&ca.0, &ca.1)),
        "setup-contract-device",
        503,
        now as i64 - 60,
        now as i64 + 3600,
    );
    if let Some(root) = std::env::var_os("FACTION336_CAPTURE_DIR") {
        use std::{
            fs,
            os::unix::fs::{OpenOptionsExt, PermissionsExt},
        };
        let root = std::path::PathBuf::from(root);
        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o7777,
            0o700
        );
        for (name, identity) in [
            ("ca", &ca),
            ("server", &server_identity),
            ("device", &device),
        ] {
            for (suffix, bytes) in [
                ("pem", identity.0.to_pem().unwrap()),
                ("key", identity.1.private_key_to_pem_pkcs8().unwrap()),
            ] {
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(root.join(format!("{name}.{suffix}")))
                    .unwrap()
                    .write_all(&bytes)
                    .unwrap();
            }
        }
    }
    let device_pin = sha256(&device.0.to_der().unwrap());
    let server_pin = sha256(&server_identity.0.to_der().unwrap());
    let credentials = ClientCredentials::from_pem(
        &ca.0.to_pem().unwrap(),
        &device.0.to_pem().unwrap(),
        &device.1.private_key_to_pem_pkcs8().unwrap(),
    )
    .unwrap();
    let cloned = credentials.clone();
    let runtime = Runtime::new(Some(
        Registry::new(
            1,
            0,
            vec!["home.test".into()],
            vec![CredentialBinding {
                certificate_sha256: device_pin,
                principal: Principal {
                    account: "device@home.test".into(),
                    device: 1,
                    identity: [1; 32],
                },
                valid_from: now - 60,
                valid_until: now + 3600,
                approved: true,
                account_enabled: true,
                device_enabled: true,
                generation: 1,
            }],
        )
        .unwrap(),
    ));
    let server = Server::new(&server_identity.0, &server_identity.1, &ca.0).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let origin = Instant::now();
    let trace = Trace::new(origin);
    trace.arm();
    let mut admission_offsets = Vec::new();
    let mut handshake_randoms = Vec::new();
    for (ordinal, credentials) in [credentials, cloned].into_iter().enumerate() {
        let (client, session, start) = std::thread::scope(|threads| {
            let peer = threads.spawn(|| {
                let _scope = trace.scope(Role::Outer);
                let setup = SetupConnection::begin(Kind::Binary).unwrap();
                let mut admission = ClientAdmission::new(
                    "provider.test",
                    server_pin,
                    credentials,
                    Arc::new(AtomicBool::new(false)),
                )
                .unwrap();
                let start = admission.start;
                let tcp = setup.interval(Phase::Tcp, 0);
                let socket = TcpStream::connect_timeout(
                    &address,
                    (start + HANDSHAKE_LIMIT).saturating_duration_since(Instant::now()),
                )
                .unwrap();
                tcp.finish();
                admission.setup_trace = Some(setup);
                let client = admission
                    .connect(socket)
                    .expect("actual authenticated client connection");
                assert_eq!(client.deadline, start + CYCLE_LIMIT);
                assert_eq!(client.stream.get_ref().deadline, start + CYCLE_LIMIT);
                (client, start)
            });
            let session = server
                .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
                .expect("actual certificate and current-registry authentication");
            let (client, start) = peer.join().unwrap();
            (client, session, start)
        });
        assert_eq!(sha256(&session.leaf.to_der().unwrap()), device_pin);
        assert_eq!(client.stream.ssl().verify_result(), X509VerifyResult::OK);
        assert_eq!(session.stream.ssl().verify_result(), X509VerifyResult::OK);
        assert!(!client.stream.ssl().session_reused());
        assert!(!session.stream.ssl().session_reused());
        assert_eq!(client.stream.ssl().version_str(), "TLSv1.3");
        let mut client_random = [0; 32];
        let mut server_observed_random = [0; 32];
        assert_eq!(client.stream.ssl().client_random(&mut client_random), 32);
        assert_eq!(
            session
                .stream
                .ssl()
                .client_random(&mut server_observed_random),
            32
        );
        assert_eq!(client_random, server_observed_random);
        if let Some(root) = std::env::var_os("FACTION336_CAPTURE_DIR") {
            use std::{fs, os::unix::fs::OpenOptionsExt};
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(std::path::PathBuf::from(root).join(format!("client-random-{ordinal}.bin")))
                .unwrap()
                .write_all(&client_random)
                .unwrap();
        }
        handshake_randoms.push(client_random);
        admission_offsets.push(start.duration_since(origin).as_micros() as u64);
        // Both sides are fully authenticated before either stream is destroyed.
        drop(client);
        drop(session);
        assert!(!runtime.state.active.load(Ordering::SeqCst));
        assert!(runtime.state.socket.lock().unwrap().is_none());
        eprintln!(
            "tls_setup_contract ordinal={ordinal} client_authenticated=true server_authenticated=true tls13=true session_reused=false original_cycle_clock=true lane_released=true"
        );
    }
    assert_ne!(
        handshake_randoms[0], handshake_randoms[1],
        "fresh handshake per connection"
    );
    drop(listener);
    assert!(crate::setup_trace::scope_is_clear());
    let snapshot = trace.snapshot();
    for point in &snapshot.points {
        eprintln!("{}", point.line());
        assert!(point.complete);
        assert!(point.start_us >= admission_offsets[point.ordinal]);
        assert!(
            point.end_us <= admission_offsets[point.ordinal] + HANDSHAKE_LIMIT.as_micros() as u64
        );
    }
    assert!(!snapshot.inconclusive, "complete actual setup chronology");
    assert_eq!(
        snapshot
            .points
            .iter()
            .filter(|p| p.phase == Phase::Ssl)
            .count(),
        2
    );
    assert_eq!(
        snapshot
            .points
            .iter()
            .filter(|p| p.phase == Phase::Postcheck)
            .count(),
        2
    );
    let bootstraps = snapshot
        .points
        .iter()
        .filter(|p| p.phase == Phase::Builder)
        .count();
    eprintln!(
        "tls_setup_contract authenticated_connections=2 fresh_handshakes=2 original_admission_clocks=2 actual_builder_calls={bootstraps}"
    );
    assert_eq!(
        bootstraps, 1,
        "one actual immutable TLS bootstrap per credential family"
    );
}

struct SetupPeer {
    ca: (X509, PKey<Private>),
    server: (X509, PKey<Private>),
    device: (X509, PKey<Private>),
}
impl SetupPeer {
    fn new() -> Self {
        let now = wall_now() as i64;
        let ca = test_certificate(None, "setup-controls-ca", 511, now - 60, now + 3600);
        let server = test_certificate(
            Some((&ca.0, &ca.1)),
            "provider.test",
            512,
            now - 60,
            now + 3600,
        );
        let device = test_certificate(Some((&ca.0, &ca.1)), "device", 513, now - 60, now + 3600);
        let peer = Self { ca, server, device };
        if let Some(root) = std::env::var_os("FACTION336_CAPTURE_DIR") {
            use std::{
                fs,
                os::unix::fs::{DirBuilderExt, OpenOptionsExt},
            };
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let root = std::path::PathBuf::from(root)
                .join(format!("peer-{}", NEXT.fetch_add(1, Ordering::SeqCst)));
            fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
            for (name, identity) in [
                ("ca", &peer.ca),
                ("server", &peer.server),
                ("device", &peer.device),
            ] {
                for (suffix, bytes) in [
                    ("pem", identity.0.to_pem().unwrap()),
                    ("key", identity.1.private_key_to_pem_pkcs8().unwrap()),
                ] {
                    fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .mode(0o600)
                        .open(root.join(format!("{name}.{suffix}")))
                        .unwrap()
                        .write_all(&bytes)
                        .unwrap();
                }
            }
            fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(root.join("test.txt"))
                .unwrap()
                .write_all(
                    std::thread::current()
                        .name()
                        .unwrap_or("unnamed focused control")
                        .as_bytes(),
                )
                .unwrap();
        }
        peer
    }
    fn credentials(&self) -> ClientCredentials {
        ClientCredentials::from_pem(
            &self.ca.0.to_pem().unwrap(),
            &self.device.0.to_pem().unwrap(),
            &self.device.1.private_key_to_pem_pkcs8().unwrap(),
        )
        .unwrap()
    }
    fn registry(&self, generation: u64, approved: bool, until: u64) -> Registry {
        Registry::new(
            generation,
            0,
            vec!["home.test".into()],
            vec![CredentialBinding {
                certificate_sha256: sha256(&self.device.0.to_der().unwrap()),
                principal: Principal {
                    account: "device@home.test".into(),
                    device: 1,
                    identity: [1; 32],
                },
                valid_from: wall_now() - 60,
                valid_until: until,
                approved,
                account_enabled: true,
                device_enabled: true,
                generation,
            }],
        )
        .unwrap()
    }
    fn runtime(&self) -> Runtime {
        Runtime::new(Some(self.registry(1, true, wall_now() + 3600)))
    }
    fn acceptor(&self) -> Server {
        Server::new(&self.server.0, &self.server.1, &self.ca.0).unwrap()
    }
    fn attempt(
        &self,
        credentials: ClientCredentials,
        name: &str,
        pin: [u8; 32],
        server: &Server,
        runtime: &Runtime,
    ) -> (Result<Client, Stage>, Result<Session, Stage>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let result = std::thread::scope(|threads| {
            let client = threads.spawn(move || {
                let admission =
                    ClientAdmission::new(name, pin, credentials, Arc::new(AtomicBool::new(false)))
                        .unwrap();
                let start = admission.start;
                let client = admission
                    .connect(TcpStream::connect_timeout(&address, HANDSHAKE_LIMIT).unwrap());
                if let Ok(client) = &client {
                    assert_eq!(client.deadline, start + CYCLE_LIMIT);
                }
                client
            });
            let session = server.accept(&listener, runtime, Arc::new(AtomicBool::new(false)));
            (client.join().unwrap(), session)
        });
        drop(listener);
        result
    }
    fn warm(&self, credentials: &ClientCredentials, runtime: &Runtime) {
        let (client, session) = self.attempt(
            credentials.clone(),
            "provider.test",
            sha256(&self.server.0.to_der().unwrap()),
            &self.acceptor(),
            runtime,
        );
        let client = client.expect("authenticated warm-up client");
        let session = session.expect("authenticated warm-up server");
        assert!(!client.stream.ssl().session_reused());
        assert!(!session.stream.ssl().session_reused());
        drop(client);
        drop(session);
        Self::lane_empty(runtime);
        assert!(credentials.connector.try_lock().unwrap().is_some());
    }
    fn lane_empty(runtime: &Runtime) {
        assert!(!runtime.state.active.load(Ordering::SeqCst));
        assert!(runtime.state.socket.lock().unwrap().is_none());
    }
}

#[test]
fn shared_setup_cold_configuration_failure_does_not_memoize_an_error() {
    let peer = SetupPeer::new();
    let mut credentials = peer.credentials();
    let original_key = credentials.key.clone();
    // Internal fault injection exercises the real OpenSSL configuration error;
    // public PEM construction itself already refuses mismatched keys.
    credentials.key = SetupPeer::new().device.1;
    assert!(matches!(
        credentials.prepare_connector(None),
        Err(Stage::Configuration)
    ));
    assert!(credentials.connector.try_lock().unwrap().is_none());
    credentials.key = original_key;
    peer.warm(&credentials, &peer.runtime());
    assert!(matches!(
        credentials.prepare_connector(None),
        Ok(PreparedConnector::Ready(_))
    ));
}

#[test]
fn shared_setup_distinct_families_and_busy_slots_keep_bounded_local_candidates() {
    let peer = SetupPeer::new();
    let credentials = peer.credentials();
    let separate = peer.credentials();
    assert!(!Arc::ptr_eq(&credentials.connector, &separate.connector));
    let lock = credentials.connector.try_lock().unwrap();
    let (sent, received) = std::sync::mpsc::sync_channel(1);
    let completed = std::thread::scope(|threads| {
        let clone = credentials.clone();
        let worker = threads.spawn(move || {
            let prepared = clone.prepare_connector(None).unwrap();
            assert!(matches!(prepared, PreparedConnector::Cold(_)));
            let connector = clone.finish_connector(prepared, None);
            assert!(
                connector
                    .configure()
                    .unwrap()
                    .into_ssl("provider.test")
                    .is_ok()
            );
            sent.send(()).unwrap();
        });
        let completed = received.recv_timeout(HANDSHAKE_LIMIT).is_ok();
        drop(lock);
        worker.join().unwrap();
        completed
    });
    assert!(
        completed,
        "busy cache lookup/publication must never wait for the owner"
    );
    assert!(credentials.connector.try_lock().unwrap().is_none());
    assert!(separate.connector.try_lock().unwrap().is_none());
    rendezvous::two_cold_candidates(&credentials)
        .outcome
        .unwrap();
    assert!(credentials.connector.try_lock().unwrap().is_some());
    assert!(matches!(
        credentials.prepare_connector(None),
        Ok(PreparedConnector::Ready(_))
    ));
    assert!(matches!(
        separate.prepare_connector(None),
        Ok(PreparedConnector::Cold(_))
    ));
    assert!(separate.connector.try_lock().unwrap().is_none());
}

#[test]
fn shared_setup_warm_name_pin_and_foreign_ca_still_refuse() {
    let peer = SetupPeer::new();
    let credentials = peer.credentials();
    let runtime = peer.runtime();
    peer.warm(&credentials, &runtime);
    let pin = sha256(&peer.server.0.to_der().unwrap());
    for (name, pin, expected) in [
        ("wrong.test", pin, Stage::Handshake),
        ("provider.test", [7; 32], Stage::Pin),
    ] {
        let (client, session) =
            peer.attempt(credentials.clone(), name, pin, &peer.acceptor(), &runtime);
        assert!(matches!(client, Err(stage) if stage == expected));
        drop(session);
        SetupPeer::lane_empty(&runtime);
    }
    let foreign = SetupPeer::new();
    let server = Server::new(&foreign.server.0, &foreign.server.1, &peer.ca.0).unwrap();
    let (client, session) = peer.attempt(
        credentials.clone(),
        "provider.test",
        sha256(&foreign.server.0.to_der().unwrap()),
        &server,
        &runtime,
    );
    assert!(matches!(client, Err(Stage::Handshake)));
    drop(session);
    SetupPeer::lane_empty(&runtime);
    let now = wall_now() as i64;
    let expired = test_certificate(
        Some((&peer.ca.0, &peer.ca.1)),
        "provider.test",
        514,
        now - 3600,
        now - 1,
    );
    if let Some(root) = std::env::var_os("FACTION336_CAPTURE_DIR") {
        use std::{fs, os::unix::fs::OpenOptionsExt};
        for (suffix, bytes) in [
            ("pem", expired.0.to_pem().unwrap()),
            ("key", expired.1.private_key_to_pem_pkcs8().unwrap()),
        ] {
            fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(std::path::PathBuf::from(&root).join(format!("expired-server.{suffix}")))
                .unwrap()
                .write_all(&bytes)
                .unwrap();
        }
    }
    let server = Server::new(&expired.0, &expired.1, &peer.ca.0).unwrap();
    let (client, session) = peer.attempt(
        credentials.clone(),
        "provider.test",
        sha256(&expired.0.to_der().unwrap()),
        &server,
        &runtime,
    );
    assert!(matches!(client, Err(Stage::Handshake)));
    drop(session);
    SetupPeer::lane_empty(&runtime);
    assert!(matches!(
        credentials.prepare_connector(None),
        Ok(PreparedConnector::Ready(_))
    ));
}

#[test]
fn shared_setup_warm_current_registry_revocation_expiry_and_reset_still_refuse() {
    let peer = SetupPeer::new();
    let credentials = peer.credentials();
    for case in 0..3 {
        let runtime = peer.runtime();
        peer.warm(&credentials, &runtime);
        match case {
            0 => runtime
                .control()
                .replace_registry(peer.registry(2, false, wall_now() + 3600))
                .unwrap(),
            1 => runtime
                .control()
                .replace_registry(peer.registry(2, true, wall_now() - 1))
                .unwrap(),
            _ => runtime.control().reset(1).unwrap(),
        }
        let (client, session) = peer.attempt(
            credentials.clone(),
            "provider.test",
            sha256(&peer.server.0.to_der().unwrap()),
            &peer.acceptor(),
            &runtime,
        );
        assert!(
            matches!(session, Err(Stage::Denied)),
            "cached context cannot authorize current registry case {case}"
        );
        drop(client);
        SetupPeer::lane_empty(&runtime);
    }
}

#[test]
fn shared_setup_family_drops_while_active_ssl_retains_only_native_context() {
    let peer = SetupPeer::new();
    let runtime = peer.runtime();
    let credentials = peer.credentials();
    let family = Arc::downgrade(&credentials.connector);
    let clone = credentials.clone();
    let (client, session) = peer.attempt(
        clone,
        "provider.test",
        sha256(&peer.server.0.to_der().unwrap()),
        &peer.acceptor(),
        &runtime,
    );
    let mut client = client.unwrap();
    let mut session = session.unwrap();
    assert!(family.upgrade().is_some());
    drop(credentials);
    assert!(
        family.upgrade().is_none(),
        "active SSL must not retain a Rust credential family/cycle"
    );
    write_frame(&mut session.stream, b"active native SSL context").unwrap();
    assert_eq!(
        read_frame(&mut client.stream).unwrap(),
        Some(b"active native SSL context".to_vec())
    );
    drop(client);
    drop(session);
    SetupPeer::lane_empty(&runtime);
    assert!(family.upgrade().is_none());
}

#[test]
fn shared_setup_cold_and_warm_admissions_keep_original_expiry_and_cancellation() {
    let peer = SetupPeer::new();
    let runtime = peer.runtime();
    let cold = peer.credentials();
    let warm = peer.credentials();
    peer.warm(&warm, &runtime);
    let mut pending = Vec::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    for credentials in [cold.clone(), warm.clone()] {
        let admission = ClientAdmission::new(
            "provider.test",
            sha256(&peer.server.0.to_der().unwrap()),
            credentials,
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        let socket = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let peer_socket = listener.accept().unwrap().0;
        pending.push((admission, socket, peer_socket));
    }
    std::thread::sleep(HANDSHAKE_LIMIT + Duration::from_millis(20));
    for (admission, socket, peer_socket) in pending {
        assert!(matches!(admission.connect(socket), Err(Stage::Deadline)));
        drop(peer_socket);
    }
    assert!(cold.connector.try_lock().unwrap().is_none());
    let cancel = Arc::new(AtomicBool::new(false));
    let admission = ClientAdmission::new(
        "provider.test",
        sha256(&peer.server.0.to_der().unwrap()),
        warm.clone(),
        cancel.clone(),
    )
    .unwrap();
    let socket = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let peer_socket = listener.accept().unwrap().0;
    cancel.store(true, Ordering::SeqCst);
    assert!(matches!(admission.connect(socket), Err(Stage::Cancelled)));
    drop(peer_socket);
    drop(listener);
    assert!(matches!(
        warm.prepare_connector(None),
        Ok(PreparedConnector::Ready(_))
    ));
}

#[cfg(feature = "delegation-wire")]
#[test]
fn shared_setup_recreated_http_endpoints_share_binary_family_and_keep_fresh_handshakes() {
    use faction_delegation_codec as codec;
    let peer = SetupPeer::new();
    let runtime = peer.runtime();
    let credentials = peer.credentials();
    // The first actual HTTP setup is cold; the recreated endpoint uses its clone.
    let query = codec::UnverifiedIntrospectionRequest {
        client_id: "setup-contract".into(),
        token: codec::UnverifiedToken::from_bytes([1; 32]),
    };
    let request = codec::encode_introspection_http("home.test", &query).unwrap();
    let response =
        codec::encode_introspection_response_http(&codec::UnverifiedIntrospection::Inactive)
            .unwrap();
    let trace = Trace::new(Instant::now());
    trace.arm();
    let mut randoms = Vec::new();
    for _ in 0..2 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = HttpEndpoint::new(
            listener.local_addr().unwrap(),
            "home.test".into(),
            "provider.test".into(),
            sha256(&peer.server.0.to_der().unwrap()),
            credentials.clone(),
        )
        .unwrap();
        let server = peer.acceptor();
        let random = std::thread::scope(|threads| {
            let server = threads.spawn(|| {
                let mut session = server
                    .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
                    .unwrap();
                assert!(!session.stream.ssl().session_reused());
                assert_eq!(session.stream.ssl().version_str(), "TLSv1.3");
                let mut random = [0; 32];
                assert_eq!(session.stream.ssl().client_random(&mut random), 32);
                let mut bytes = vec![0; request.len()];
                session.stream.read_exact(&mut bytes).unwrap();
                assert_eq!(bytes, request);
                session.stream.write_all(&response).unwrap();
                session.stream.flush().unwrap();
                random
            });
            let _scope = trace.scope(Role::OwnHome);
            let result = runtime
                .control()
                .query_introspection(
                    &endpoint,
                    &query,
                    Instant::now() + CYCLE_LIMIT,
                    Arc::new(AtomicBool::new(false)),
                )
                .unwrap();
            assert!(matches!(result, codec::UnverifiedIntrospection::Inactive));
            server.join().unwrap()
        });
        randoms.push(random);
        SetupPeer::lane_empty(&runtime);
    }
    assert_ne!(randoms[0], randoms[1]);
    let snapshot = trace.snapshot();
    assert!(!snapshot.inconclusive);
    assert_eq!(
        snapshot
            .points
            .iter()
            .filter(|p| p.phase == Phase::Builder)
            .count(),
        1
    );
    assert_eq!(
        snapshot
            .points
            .iter()
            .filter(|p| p.phase == Phase::Immutable)
            .count(),
        2
    );
    assert_eq!(
        snapshot
            .points
            .iter()
            .filter(|p| p.outcome == crate::setup_trace::SetupOutcome::Hit)
            .count(),
        1
    );
    assert_eq!(
        snapshot
            .points
            .iter()
            .filter(|p| p.outcome == crate::setup_trace::SetupOutcome::ColdPublished)
            .count(),
        1
    );
    assert_eq!(
        snapshot
            .points
            .iter()
            .filter(|p| p.phase == Phase::Ssl)
            .count(),
        2
    );
    // Binary and HTTP immutable profiles match, so the same family remains warm.
    assert!(matches!(
        credentials.prepare_connector(None),
        Ok(PreparedConnector::Ready(_))
    ));
    peer.warm(&credentials, &runtime);
    let cancel = Arc::new(AtomicBool::new(true));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = HttpEndpoint::new(
        listener.local_addr().unwrap(),
        "home.test".into(),
        "provider.test".into(),
        sha256(&peer.server.0.to_der().unwrap()),
        credentials,
    )
    .unwrap();
    assert!(matches!(
        runtime.control().query_introspection(
            &endpoint,
            &query,
            Instant::now() + CYCLE_LIMIT,
            cancel
        ),
        Err(Error::Transport(Stage::Cancelled))
    ));
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(listener.accept(), Err(error) if error.kind() == io::ErrorKind::WouldBlock));
}

#[cfg(feature = "delegation-verifier")]
#[test]
fn shared_setup_warm_binary_current_probe_and_inherited_deadline_still_refuse() {
    let peer = SetupPeer::new();
    let runtime = peer.runtime();
    let credentials = peer.credentials();
    peer.warm(&credentials, &runtime);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = peer.acceptor();
    let (client, session) = std::thread::scope(|threads| {
        let client = threads.spawn(|| {
            let mut admission = ClientAdmission::new(
                "provider.test",
                sha256(&peer.server.0.to_der().unwrap()),
                credentials.clone(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
            admission.current_probe = Some(Arc::new(|| false));
            admission.connect(TcpStream::connect(address).unwrap())
        });
        let session = server.accept(&listener, &runtime, Arc::new(AtomicBool::new(false)));
        (client.join().unwrap(), session)
    });
    assert!(client.is_err());
    assert!(session.is_err());
    SetupPeer::lane_empty(&runtime);
    let mut admission = ClientAdmission::new(
        "provider.test",
        sha256(&peer.server.0.to_der().unwrap()),
        credentials,
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    admission.operation_deadline = Some(Instant::now() - Duration::from_millis(1));
    let socket = TcpStream::connect(address).unwrap();
    let (peer_socket, _) = listener.accept().unwrap();
    assert!(matches!(admission.connect(socket), Err(Stage::Deadline)));
    drop(peer_socket);
}

#[cfg(feature = "delegation-wire")]
#[test]
fn shared_setup_warm_http_name_pin_and_ca_still_refuse() {
    use faction_delegation_codec as codec;
    let peer = SetupPeer::new();
    let foreign = SetupPeer::new();
    let runtime = peer.runtime();
    let credentials = peer.credentials();
    peer.warm(&credentials, &runtime);
    let query = codec::UnverifiedIntrospectionRequest {
        client_id: "setup-contract".into(),
        token: codec::UnverifiedToken::from_bytes([1; 32]),
    };
    for case in 0..3 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let identity = if case == 2 {
            &foreign.server
        } else {
            &peer.server
        };
        let server = Server::new(&identity.0, &identity.1, &peer.ca.0).unwrap();
        let endpoint = HttpEndpoint::new(
            listener.local_addr().unwrap(),
            "home.test".into(),
            if case == 0 {
                "wrong.test"
            } else {
                "provider.test"
            }
            .into(),
            if case == 1 {
                [7; 32]
            } else {
                sha256(&identity.0.to_der().unwrap())
            },
            credentials.clone(),
        )
        .unwrap();
        let answer = std::thread::scope(|threads| {
            let server = threads.spawn(|| {
                drop(server.accept(&listener, &runtime, Arc::new(AtomicBool::new(false))));
            });
            let answer = runtime.control().query_introspection(
                &endpoint,
                &query,
                Instant::now() + CYCLE_LIMIT,
                Arc::new(AtomicBool::new(false)),
            );
            server.join().unwrap();
            answer
        });
        let expected = if case == 1 {
            Stage::Pin
        } else {
            Stage::Handshake
        };
        assert!(
            matches!(answer, Err(Error::Transport(stage)) if stage == expected),
            "warm HTTP trust case {case}"
        );
        SetupPeer::lane_empty(&runtime);
    }
}

#[cfg(feature = "delegation-verifier")]
#[test]
fn shared_setup_warm_http_epoch_change_refuses_before_application_write() {
    use faction_delegation_codec as codec;
    let peer = SetupPeer::new();
    let runtime = peer.runtime();
    let credentials = peer.credentials();
    peer.warm(&credentials, &runtime);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut endpoint = HttpEndpoint::new(
        listener.local_addr().unwrap(),
        "home.test".into(),
        "provider.test".into(),
        sha256(&peer.server.0.to_der().unwrap()),
        credentials,
    )
    .unwrap();
    let (entered, ready) = std::sync::mpsc::channel();
    let (release, released) = std::sync::mpsc::channel();
    endpoint.hold_request(entered, released);
    let (admitted, admission) = std::sync::mpsc::channel();
    let server = peer.acceptor();
    std::thread::scope(|threads| {
        let server = threads.spawn(|| {
            let mut session = server
                .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
                .unwrap();
            admitted.send(()).unwrap();
            let mut bytes = [0; 1];
            let read = session.stream.read(&mut bytes);
            assert!(
                !matches!(read, Ok(n) if n > 0),
                "stale HTTP epoch cannot dispatch application bytes"
            );
        });
        let client = threads.spawn(|| {
            let query = codec::UnverifiedIntrospectionRequest {
                client_id: "setup-contract".into(),
                token: codec::UnverifiedToken::from_bytes([1; 32]),
            };
            runtime.control().query_introspection(
                &endpoint,
                &query,
                Instant::now() + CYCLE_LIMIT,
                Arc::new(AtomicBool::new(false)),
            )
        });
        let original = ready.recv_timeout(HANDSHAKE_LIMIT).unwrap();
        admission.recv_timeout(HANDSHAKE_LIMIT).unwrap();
        runtime
            .control()
            .replace_registry(peer.registry(2, true, wall_now() + 3600))
            .unwrap();
        release.send(()).unwrap();
        assert!(client.join().unwrap().is_err());
        assert!(Instant::now() < original);
        server.join().unwrap();
    });
    SetupPeer::lane_empty(&runtime);
}

#[path = "tls_setup_rendezvous_tests.rs"]
mod rendezvous;
