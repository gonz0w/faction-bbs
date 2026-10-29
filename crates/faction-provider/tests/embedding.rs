//! Actual disposable loopback TLS1.3 observations; synthetic identities only.
use faction_files::private_blobs::{Request, Response, Store};
use faction_provider::tls::{CYCLE_LIMIT, Client, HANDSHAKE_LIMIT, MAX_OPERATIONS, Server, Stage};
use faction_provider::{Control, Error, Runtime, Session};
use openssl::{
    asn1::Asn1Time,
    pkey::{PKey, Private},
    sha::sha256,
    x509::X509,
};
use std::{
    io::{self, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
fn wall_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
use faction_files::private_blobs::{Binding, Principal};
use faction_provider::authority::{CredentialBinding, Registry};
use openssl::{
    bn::BigNum,
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    nid::Nid,
    x509::{
        X509NameBuilder,
        extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage, SubjectAlternativeName},
    },
};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::PathBuf,
    sync::{Mutex, MutexGuard, atomic::AtomicU64},
    thread,
};
static SERIAL: Mutex<()> = Mutex::new(());
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    _serial: MutexGuard<'static, ()>,
}
impl Fixture {
    fn new() -> Self {
        let serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "faction276-owned-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            wall_now()
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join("OWNED"))
            .unwrap();
        f.write_all(b"issue276 synthetic TLS fixture").unwrap();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(root.join("store"))
            .unwrap();
        Self {
            root,
            _serial: serial,
        }
    }
    fn store(&self) -> Store {
        Store::open(&self.root.join("store")).unwrap()
    }
    fn bytes(&self) -> Vec<u8> {
        fs::read(self.root.join("store/private-blobs.snapshot")).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert_eq!(
            fs::read(self.root.join("OWNED")).unwrap(),
            b"issue276 synthetic TLS fixture"
        );
        fs::remove_dir_all(&self.root).unwrap();
        assert!(!self.root.exists());
    }
}
fn key() -> PKey<Private> {
    PKey::from_ec_key(
        EcKey::generate(&EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap()).unwrap(),
    )
    .unwrap()
}
fn cert(
    ca: Option<(&X509, &PKey<Private>)>,
    name: &str,
    serial: u32,
    start: i64,
    end: i64,
) -> (X509, PKey<Private>) {
    let key = key();
    let mut n = X509NameBuilder::new().unwrap();
    n.append_entry_by_text("CN", name).unwrap();
    let n = n.build();
    let mut b = X509::builder().unwrap();
    b.set_version(2).unwrap();
    b.set_serial_number(&BigNum::from_u32(serial).unwrap().to_asn1_integer().unwrap())
        .unwrap();
    b.set_subject_name(&n).unwrap();
    b.set_issuer_name(ca.map_or(n.as_ref(), |v| v.0.subject_name()))
        .unwrap();
    b.set_pubkey(&key).unwrap();
    b.set_not_before(&Asn1Time::from_unix(start).unwrap())
        .unwrap();
    b.set_not_after(&Asn1Time::from_unix(end).unwrap()).unwrap();
    if ca.is_none() {
        b.append_extension(BasicConstraints::new().critical().ca().build().unwrap())
            .unwrap();
        b.append_extension(KeyUsage::new().key_cert_sign().crl_sign().build().unwrap())
            .unwrap();
    } else {
        b.append_extension(BasicConstraints::new().critical().build().unwrap())
            .unwrap();
        b.append_extension(KeyUsage::new().digital_signature().build().unwrap())
            .unwrap();
        b.append_extension(
            ExtendedKeyUsage::new()
                .client_auth()
                .server_auth()
                .build()
                .unwrap(),
        )
        .unwrap();
        let ext = SubjectAlternativeName::new()
            .dns("provider.test")
            .build(&b.x509v3_context(ca.map(|v| v.0.as_ref()), None))
            .unwrap();
        b.append_extension(ext).unwrap();
    }
    b.sign(ca.map_or(&key, |v| v.1), MessageDigest::sha256())
        .unwrap();
    (b.build(), key)
}
struct Certs {
    ca: (X509, PKey<Private>),
    server: (X509, PKey<Private>),
    alice: (X509, PKey<Private>),
    bob: (X509, PKey<Private>),
}
impl Certs {
    fn new() -> Self {
        let t = wall_now() as i64;
        let ca = cert(None, "fixture CA", 1, t - 60, t + 3600);
        let server = cert(Some((&ca.0, &ca.1)), "provider.test", 2, t - 60, t + 3600);
        let alice = cert(Some((&ca.0, &ca.1)), "alice", 3, t - 60, t + 3600);
        let bob = cert(Some((&ca.0, &ca.1)), "bob", 4, t - 60, t + 3600);
        Self {
            ca,
            server,
            alice,
            bob,
        }
    }
    fn server(&self) -> Server {
        Server::new(&self.server.0, &self.server.1, &self.ca.0).unwrap()
    }
    fn client(
        &self,
        addr: SocketAddr,
        cancel: Arc<AtomicBool>,
        who: Option<&(X509, PKey<Private>)>,
    ) -> Result<Client, Stage> {
        Client::connect(
            addr,
            "provider.test",
            sha256(&self.server.0.to_der().unwrap()),
            &self.ca.0,
            who.map(|v| (&v.0, Some(&v.1))),
            cancel,
        )
    }
    fn entries(&self) -> Vec<CredentialBinding> {
        let t = wall_now();
        [(&self.alice, 1), (&self.bob, 2)]
            .into_iter()
            .map(|(pair, n)| CredentialBinding {
                certificate_sha256: sha256(&pair.0.to_der().unwrap()),
                principal: principal(n),
                valid_from: t - 60,
                valid_until: t + 3600,
                approved: true,
                account_enabled: true,
                device_enabled: true,
                generation: 1,
            })
            .collect()
    }
    fn runtime(&self) -> Runtime {
        Runtime::new(Some(
            Registry::new(1, 0, vec!["home.test".into()], self.entries()).unwrap(),
        ))
    }
}
fn principal(n: u8) -> Principal {
    Principal {
        account: if n == 1 {
            "alice@home.test"
        } else {
            "bob@home.test"
        }
        .into(),
        device: n.into(),
        identity: [n; 32],
    }
}
fn binding() -> Binding {
    let t = wall_now();
    Binding {
        sender: principal(1),
        recipient: principal(2),
        id: sha256(&[9; 32]),
        size: 32,
        created: t,
        expires: t + 100,
    }
}

fn current_registry(
    c: &Certs,
    generation: u64,
    change: impl FnOnce(&mut Vec<CredentialBinding>),
) -> Registry {
    let mut entries = c.entries();
    for e in &mut entries {
        e.generation = generation;
    }
    change(&mut entries);
    Registry::new(generation, 0, vec!["home.test".into()], entries).unwrap()
}
fn with_session<T>(
    c: &Certs,
    runtime: &Runtime,
    who: &(X509, PKey<Private>),
    action: impl FnOnce(Session) -> T,
) -> T {
    with_cancel_session(c, runtime, who, |session, _| action(session))
}
fn with_cancel_session<T>(
    c: &Certs,
    runtime: &Runtime,
    who: &(X509, PKey<Private>),
    action: impl FnOnce(Session, Arc<AtomicBool>) -> T,
) -> T {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let value = thread::scope(|scope| {
        let (done, wait) = std::sync::mpsc::channel();
        let client_cancel = cancel.clone();
        let client = scope.spawn(move || {
            let client = c.client(addr, client_cancel, Some(who)).unwrap();
            wait.recv_timeout(Duration::from_secs(15)).unwrap();
            drop(client);
        });
        let session = c
            .server()
            .accept(&listener, runtime, cancel.clone())
            .unwrap();
        let result = action(session, cancel);
        done.send(()).unwrap();
        let drain = Instant::now();
        client.join().unwrap();
        assert!(drain.elapsed() < Duration::from_secs(5));
        result
    });
    drop(listener);
    match TcpStream::connect_timeout(&addr, Duration::from_millis(100)) {
        Err(e) => assert_eq!(e.kind(), io::ErrorKind::ConnectionRefused),
        Ok(s) => assert_eq!(
            s.take_error().unwrap().unwrap().kind(),
            io::ErrorKind::ConnectionRefused
        ),
    }
    value
}
// Each object is committed through its own real certificate session. Self delivery
// makes Alice's readable object and Bob's private object distinct actor controls.
fn committed_route_objects(c: &Certs, runtime: &Runtime, store: &mut Store) -> [Binding; 2] {
    [1, 2].map(|n| {
        let mut b = binding();
        b.sender = principal(n);
        b.recipient = principal(n);
        b.id = sha256(&[n; 32]);
        with_session(
            c,
            runtime,
            if n == 1 { &c.alice } else { &c.bob },
            |mut s| {
                for request in [
                    Request::Begin(b.clone()),
                    Request::PutChunk {
                        id: b.id,
                        offset: 0,
                        bytes: vec![n; 32],
                    },
                    Request::Commit { id: b.id },
                ] {
                    assert!(s.dispatch(store, &b.sender.account, request).is_ok());
                }
            },
        );
        b
    })
}
fn committed_reads(b: &Binding) -> [(Request, Response); 2] {
    [
        (
            Request::Status { id: b.id },
            Response::Progress {
                binding: b.clone(),
                offset: 32,
                committed: true,
            },
        ),
        (
            Request::GetChunk {
                id: b.id,
                offset: 0,
            },
            Response::Chunk {
                offset: 0,
                bytes: vec![b.recipient.identity[0]; 32],
                total: 32,
            },
        ),
    ]
}
#[test]
fn committed_actor_free_routes_reject_confusion_in_dispatch() {
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let mut store = f.store();
    let [alice, bob] = committed_route_objects(&c, &runtime, &mut store);
    with_session(&c, &runtime, &c.bob, |mut s| {
        for (request, expected) in committed_reads(&bob) {
            assert_eq!(
                s.dispatch(&mut store, "bob@home.test", request),
                Ok(expected)
            );
        }
    });
    with_session(&c, &runtime, &c.alice, |mut s| {
        for (request, expected) in committed_reads(&alice) {
            assert_eq!(
                s.dispatch(&mut store, "alice@home.test", request),
                Ok(expected)
            );
        }
        for object in [&alice, &bob] {
            for (request, _) in committed_reads(object) {
                let before = f.bytes();
                assert_eq!(
                    s.dispatch(&mut store, "bob@home.test", request),
                    Err(Error::Store(faction_files::private_blobs::Error::Denied))
                );
                assert_eq!(f.bytes(), before);
            }
        }
    });
}
#[test]
fn committed_actor_free_routes_reject_confusion_on_actual_binary_wire() {
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let mut store = f.store();
    let [alice, bob] = committed_route_objects(&c, &runtime, &mut store);
    // First prove Bob's private object is actually readable on the wire.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::scope(|scope| {
        let peer = scope.spawn(|| {
            let mut client = c
                .client(addr, Arc::new(AtomicBool::new(false)), Some(&c.bob))
                .unwrap();
            for (request, expected) in committed_reads(&bob) {
                assert_eq!(client.request(&request), Ok(expected));
            }
        });
        let mut session = c
            .server()
            .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
            .unwrap();
        for _ in 0..2 {
            assert_eq!(session.serve_next(&mut store, "bob@home.test"), Ok(true));
        }
        peer.join().unwrap();
    });
    drop(listener);
    for object in [&alice, &bob] {
        for (index, (forged, _)) in committed_reads(object).into_iter().enumerate() {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            let (rightful, expected) = committed_reads(&alice)[index].clone();
            thread::scope(|scope| {
                let peer = scope.spawn(|| {
                    let mut client = c
                        .client(addr, Arc::new(AtomicBool::new(false)), Some(&c.alice))
                        .unwrap();
                    assert_eq!(client.request(&rightful), Ok(expected));
                    assert!(client.request(&forged).is_err());
                });
                let mut session = c
                    .server()
                    .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
                    .unwrap();
                assert_eq!(session.serve_next(&mut store, "alice@home.test"), Ok(true));
                let before = f.bytes();
                assert_eq!(
                    session.serve_next(&mut store, "bob@home.test"),
                    Err(Error::Store(faction_files::private_blobs::Error::Denied))
                );
                assert_eq!(f.bytes(), before);
                let drain = Instant::now();
                peer.join().unwrap();
                assert!(drain.elapsed() < Duration::from_secs(5));
            });
        }
    }
}

#[test]
fn public_embedding_contract_keeps_qualified_bounds() {
    assert_eq!(HANDSHAKE_LIMIT, Duration::from_secs(5));
    assert_eq!(CYCLE_LIMIT, Duration::from_secs(10));
    assert_eq!(MAX_OPERATIONS, 8);
    fn public_types(_: Option<(Control, Session, Client, Server)>) {}
    public_types(None);
}
#[test]
fn real_tls_upload_exact_retry_and_recipient_read_use_only_certificate_actor() {
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let mut store = f.store();
    let b = binding();
    with_session(&c, &runtime, &c.alice, |mut s| {
        for r in [
            Request::Begin(b.clone()),
            Request::PutChunk {
                id: b.id,
                offset: 0,
                bytes: vec![9; 32],
            },
            Request::PutChunk {
                id: b.id,
                offset: 0,
                bytes: vec![9; 32],
            },
            Request::Commit { id: b.id },
            Request::Commit { id: b.id },
        ] {
            assert!(s.dispatch(&mut store, "alice@home.test", r).is_ok());
        }
    });
    with_session(&c, &runtime, &c.bob, |mut s| {
        assert_eq!(
            s.dispatch(
                &mut store,
                "bob@home.test",
                Request::GetChunk {
                    id: b.id,
                    offset: 0
                }
            )
            .unwrap(),
            Response::Chunk {
                offset: 0,
                bytes: vec![9; 32],
                total: 32
            }
        );
        assert!(
            s.dispatch(&mut store, "bob@home.test", Request::Status { id: b.id })
                .is_ok()
        );
    });
}
#[test]
fn forged_route_account_device_and_full_identity_never_mutate_store() {
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let mut store = f.store();
    let before = f.bytes();
    with_session(&c, &runtime, &c.alice, |mut s| {
        for r in [
            Request::Status { id: [3; 32] },
            Request::GetChunk {
                id: [3; 32],
                offset: 0,
            },
        ] {
            assert!(s.dispatch(&mut store, "bob@home.test", r).is_err());
            assert_eq!(f.bytes(), before);
        }
        for n in 0..3 {
            let mut b = binding();
            match n {
                0 => b.sender.account = "bob@home.test".into(),
                1 => b.sender.device = 9,
                _ => b.sender.identity[31] ^= 1,
            };
            assert!(
                s.dispatch(&mut store, "alice@home.test", Request::Begin(b))
                    .is_err()
            );
            assert_eq!(f.bytes(), before);
        }
    });
}
#[test]
fn current_public_policy_controls_affect_live_sessions_and_invalid_input_preserves_snapshot() {
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let control = runtime.control();
    let mut store = f.store();
    let b = binding();
    with_session(&c, &runtime, &c.alice, |mut s| {
        assert!(
            s.dispatch(&mut store, "alice@home.test", Request::Begin(b.clone()))
                .is_ok()
        );
        assert!(
            control
                .replace_registry(current_registry(&c, 1, |_| {}))
                .is_err()
        );
        assert!(
            s.dispatch(&mut store, "alice@home.test", Request::Status { id: b.id })
                .is_ok()
        );
        let before = f.bytes();
        control
            .replace_registry(current_registry(&c, 2, |e| e[0].device_enabled = false))
            .unwrap();
        assert!(
            s.dispatch(&mut store, "alice@home.test", Request::Status { id: b.id })
                .is_err()
        );
        assert_eq!(before, f.bytes());
    });
}
#[test]
fn unavailable_reset_stop_drop_and_other_instances_cannot_reuse_live_authority() {
    let f = Fixture::new();
    let c = Certs::new();
    let mut store = f.store();
    for variant in 0..4 {
        let runtime = c.runtime();
        let control = runtime.control();
        with_session(&c, &runtime, &c.alice, |mut s| {
            let before = f.bytes();
            match variant {
                0 => control.unavailable().unwrap(),
                1 => control.reset(1).unwrap(),
                2 => runtime.stop(),
                _ => control.stop().unwrap(),
            };
            assert!(
                s.dispatch(
                    &mut store,
                    "alice@home.test",
                    Request::Status { id: [5; 32] }
                )
                .is_err()
            );
            assert_eq!(before, f.bytes());
        });
    }
    let runtime = c.runtime();
    let control = runtime.control();
    let mut old = with_session(&c, &runtime, &c.alice, |s| s);
    drop(runtime);
    let _replacement = c.runtime();
    assert!(control.reset(1).is_err());
    let before = f.bytes();
    assert!(
        old.dispatch(
            &mut store,
            "alice@home.test",
            Request::Status { id: [5; 32] }
        )
        .is_err()
    );
    assert_eq!(before, f.bytes());
}
#[test]
fn malformed_and_denied_attempts_share_eight_operation_budget_before_decode() {
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let mut store = f.store();
    let before = f.bytes();
    with_session(&c, &runtime, &c.alice, |mut s| {
        for i in 0..8 {
            let bytes = if i % 2 == 0 { vec![0; 8193] } else { vec![255] };
            assert!(
                s.dispatch_bytes(&mut store, "alice@home.test", &bytes)
                    .is_err()
            );
            assert_eq!(before, f.bytes());
        }
        assert_eq!(
            s.dispatch(&mut store, "alice@home.test", Request::Begin(binding())),
            Err(Error::Transport(Stage::Limit))
        );
    });
}
#[test]
fn live_invited_peer_revocation_reset_and_full_identity_changes_deny_next_operation() {
    let f = Fixture::new();
    let c = Certs::new();
    let mut store = f.store();
    for variant in 0..5 {
        let runtime = c.runtime();
        let control = runtime.control();
        with_session(&c, &runtime, &c.alice, |mut s| {
            let before = f.bytes();
            let entries = current_registry(&c, 2, |e| match variant {
                0 => e[0].account_enabled = false,
                1 => e[0].approved = false,
                2 => e[1].device_enabled = false,
                3 => {
                    e[1].principal.identity[31] ^= 1;
                    e[1].certificate_sha256 = [99; 32]
                }
                _ => e[0].valid_until = wall_now(),
            });
            control.replace_registry(entries).unwrap();
            assert!(
                s.dispatch(&mut store, "alice@home.test", Request::Begin(binding()))
                    .is_err()
            );
            assert_eq!(before, f.bytes());
        });
    }
}
#[test]
fn current_peer_policy_is_rechecked_for_newly_accepted_sessions() {
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let control = runtime.control();
    let mut store = f.store();
    control
        .replace_registry(current_registry(&c, 2, |e| e[1].device_enabled = false))
        .unwrap();
    let before = f.bytes();
    with_session(&c, &runtime, &c.alice, |mut s| {
        assert!(
            s.dispatch(&mut store, "alice@home.test", Request::Begin(binding()))
                .is_err()
        );
        assert_eq!(before, f.bytes());
    });
}
#[test]
fn real_cycle_expiry_and_cancellation_deny_without_store_mutation() {
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let mut store = f.store();
    let before = f.bytes();
    with_session(&c, &runtime, &c.alice, |mut s| {
        thread::sleep(CYCLE_LIMIT);
        assert_eq!(
            s.dispatch(&mut store, "alice@home.test", Request::Begin(binding())),
            Err(Error::Transport(Stage::Deadline))
        );
        assert_eq!(before, f.bytes());
    });
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let cancel = Arc::new(AtomicBool::new(true));
    assert!(matches!(
        c.server().accept(&listener, &runtime, cancel),
        Err(Stage::Cancelled)
    ));
}
#[test]
fn server_name_pin_ca_and_missing_client_key_fail_without_session_authority() {
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let _store = f.store();
    let before = f.bytes();
    assert!(matches!(
        Client::connect(
            "127.0.0.1:1".parse().unwrap(),
            "provider.test",
            [1; 32],
            &c.ca.0,
            Some((&c.alice.0, None)),
            Arc::new(AtomicBool::new(false))
        ),
        Err(Stage::Configuration)
    ));
    for variant in 0..4 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        thread::scope(|scope| {
            let worker = scope.spawn(|| c.server().accept(&listener, &runtime, cancel.clone()));
            let wrongca = cert(
                None,
                "wrong CA",
                90,
                wall_now() as i64 - 5,
                wall_now() as i64 + 100,
            );
            let result = Client::connect(
                addr,
                if variant == 0 {
                    "wrong.test"
                } else {
                    "provider.test"
                },
                if variant == 1 {
                    [4; 32]
                } else {
                    sha256(&c.server.0.to_der().unwrap())
                },
                if variant == 2 { &wrongca.0 } else { &c.ca.0 },
                if variant == 3 {
                    None
                } else {
                    Some((&c.alice.0, Some(&c.alice.1)))
                },
                cancel.clone(),
            );
            if variant < 3 {
                assert!(result.is_err());
            }
            drop(result);
            let session = worker.join().unwrap();
            if variant == 3 {
                assert!(session.is_err());
            }
            drop(session);
        });
        assert_eq!(before, f.bytes());
    }
}

#[test]
fn real_tcp_wait_and_stalled_tls_share_five_second_admission() {
    let _f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let start = Instant::now();
    thread::scope(|scope| {
        let server = scope.spawn(|| {
            c.server()
                .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
        });
        thread::sleep(Duration::from_millis(250));
        let peer = TcpStream::connect(addr).unwrap();
        assert!(matches!(server.join().unwrap(), Err(Stage::Deadline)));
        drop(peer);
    });
    assert!(start.elapsed() >= HANDSHAKE_LIMIT);
    assert!(start.elapsed() < HANDSHAKE_LIMIT + Duration::from_secs(1));
}
#[test]
fn stop_interrupts_handshake_and_live_cancel_and_second_session_deny() {
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let mut store = f.store();
    with_cancel_session(&c, &runtime, &c.alice, |mut s, cancel| {
        let before = f.bytes();
        let second = TcpListener::bind("127.0.0.1:0").unwrap();
        assert!(matches!(
            c.server()
                .accept(&second, &runtime, Arc::new(AtomicBool::new(false))),
            Err(Stage::Denied)
        ));
        cancel.store(true, Ordering::SeqCst);
        assert_eq!(
            s.dispatch(&mut store, "alice@home.test", Request::Begin(binding())),
            Err(Error::Transport(Stage::Cancelled))
        );
        assert_eq!(before, f.bytes());
    });
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let control = runtime.control();
    thread::scope(|scope| {
        let server = scope.spawn(|| {
            c.server()
                .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
        });
        let peer = TcpStream::connect(addr).unwrap();
        thread::sleep(Duration::from_millis(30));
        let drain = Instant::now();
        control.stop().unwrap();
        assert!(server.join().unwrap().is_err());
        assert!(drain.elapsed() < Duration::from_secs(5));
        drop(peer);
    });
}
#[test]
fn actual_leaf_expiry_denies_next_dispatch_and_unknown_client_ca_never_authenticates() {
    let f = Fixture::new();
    let mut c = Certs::new();
    let t = wall_now() as i64;
    c.alice = cert(Some((&c.ca.0, &c.ca.1)), "alice", 8, t - 10, t + 2);
    let runtime = c.runtime();
    let mut store = f.store();
    let before = f.bytes();
    with_session(&c, &runtime, &c.alice, |mut s| {
        thread::sleep(Duration::from_secs(3));
        assert!(
            s.dispatch(&mut store, "alice@home.test", Request::Begin(binding()))
                .is_err()
        );
        assert_eq!(before, f.bytes());
    });
    drop(runtime);
    c.alice = cert(None, "untrusted alice", 9, t - 10, t + 100);
    let runtime = c.runtime();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::scope(|scope| {
        let server = scope.spawn(|| {
            c.server()
                .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
        });
        let client = c.client(addr, Arc::new(AtomicBool::new(false)), Some(&c.alice));
        drop(client);
        assert!(server.join().unwrap().is_err());
    });
    assert_eq!(before, f.bytes());
}

#[test]
fn actual_binary_peer_bytes_and_responses_share_the_verified_session() {
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let mut store = f.store();
    let b = binding();
    for recipient in [false, true] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let requests = if recipient {
            vec![
                Request::GetChunk {
                    id: b.id,
                    offset: 0,
                },
                Request::Status { id: b.id },
            ]
        } else {
            vec![
                Request::Begin(b.clone()),
                Request::PutChunk {
                    id: b.id,
                    offset: 0,
                    bytes: vec![9; 32],
                },
                Request::Commit { id: b.id },
                Request::Commit { id: b.id },
            ]
        };
        let count = requests.len();
        thread::scope(|scope| {
            let peer = scope.spawn(|| {
                let mut client = c
                    .client(
                        addr,
                        Arc::new(AtomicBool::new(false)),
                        Some(if recipient { &c.bob } else { &c.alice }),
                    )
                    .unwrap();
                let responses: Vec<_> = requests
                    .iter()
                    .map(|r| client.request(r).unwrap())
                    .collect();
                if recipient {
                    assert_eq!(
                        responses[0],
                        Response::Chunk {
                            offset: 0,
                            bytes: vec![9; 32],
                            total: 32
                        }
                    );
                }
            });
            let mut session = c
                .server()
                .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
                .unwrap();
            for _ in 0..count {
                assert!(
                    session
                        .serve_next(
                            &mut store,
                            if recipient {
                                "bob@home.test"
                            } else {
                                "alice@home.test"
                            }
                        )
                        .unwrap()
                );
            }
            peer.join().unwrap();
        });
    }
}
#[test]
fn actual_binary_actor_free_wrong_route_closes_without_mutation() {
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let mut store = f.store();
    let before = f.bytes();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::scope(|scope| {
        let peer = scope.spawn(|| {
            let mut client = c
                .client(addr, Arc::new(AtomicBool::new(false)), Some(&c.alice))
                .unwrap();
            assert!(client.request(&Request::Status { id: [4; 32] }).is_err());
        });
        let mut session = c
            .server()
            .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
            .unwrap();
        assert!(session.serve_next(&mut store, "bob@home.test").is_err());
        peer.join().unwrap();
    });
    assert_eq!(before, f.bytes());
}

#[cfg(feature = "blob-iq")]
mod iq_contract {
    use super::*;
    use faction_provider::IqService;
    use faction_xmpp::{Element, blob_iq};
    fn query(id: &str, own: &str, request: &Request) -> Element {
        blob_iq::request(id, own, "home.test", request).unwrap()
    }
    #[test]
    fn maintained_iqs_compose_with_real_tls_principals_and_exact_retries() {
        let f = Fixture::new();
        let c = Certs::new();
        let runtime = c.runtime();
        let mut store = f.store();
        let service = IqService::new("home.test").unwrap();
        let b = binding();
        with_session(&c, &runtime, &c.alice, |mut session| {
            for (index, request) in [
                Request::Begin(b.clone()),
                Request::PutChunk {
                    id: b.id,
                    offset: 0,
                    bytes: vec![9; 32],
                },
                Request::Commit { id: b.id },
                Request::Commit { id: b.id },
                Request::Status { id: b.id },
            ]
            .into_iter()
            .enumerate()
            {
                let id = format!("iq-{index}");
                let iq = query(&id, "alice@home.test", &request);
                let response = session.dispatch_iq(&mut store, &service, &iq).unwrap();
                assert!(
                    blob_iq::decode_response(&response, &id, "alice@home.test", "home.test")
                        .is_ok()
                );
            }
        });
        with_session(&c, &runtime, &c.bob, |mut session| {
            let iq = query(
                "read",
                "bob@home.test",
                &Request::GetChunk {
                    id: b.id,
                    offset: 0,
                },
            );
            let response = session.dispatch_iq(&mut store, &service, &iq).unwrap();
            assert_eq!(
                blob_iq::decode_response(&response, "read", "bob@home.test", "home.test").unwrap(),
                Response::Chunk {
                    offset: 0,
                    bytes: vec![9; 32],
                    total: 32
                }
            );
            for (id, own, home) in [
                ("wrong", "bob@home.test", "home.test"),
                ("read", "alice@home.test", "home.test"),
                ("read", "bob@home.test", "away.test"),
            ] {
                assert!(blob_iq::decode_response(&response, id, own, home).is_err());
            }
        });
    }
    #[test]
    fn committed_actor_free_routes_reject_confusion_in_maintained_iqs() {
        let f = Fixture::new();
        let c = Certs::new();
        let runtime = c.runtime();
        let mut store = f.store();
        let [alice, bob] = committed_route_objects(&c, &runtime, &mut store);
        let service = IqService::new("home.test").unwrap();
        with_session(&c, &runtime, &c.bob, |mut session| {
            for (request, expected) in committed_reads(&bob) {
                let iq = query("bob-positive", "bob@home.test", &request);
                let reply = session.dispatch_iq(&mut store, &service, &iq).unwrap();
                assert_eq!(
                    blob_iq::decode_response(&reply, "bob-positive", "bob@home.test", "home.test"),
                    Ok(expected)
                );
            }
        });
        with_session(&c, &runtime, &c.alice, |mut session| {
            for (request, expected) in committed_reads(&alice) {
                let iq = query("alice-positive", "alice@home.test", &request);
                let reply = session.dispatch_iq(&mut store, &service, &iq).unwrap();
                assert_eq!(
                    blob_iq::decode_response(
                        &reply,
                        "alice-positive",
                        "alice@home.test",
                        "home.test"
                    ),
                    Ok(expected)
                );
            }
            for object in [&alice, &bob] {
                for (request, _) in committed_reads(object) {
                    let iq = query("forged-existing", "bob@home.test", &request);
                    let before = f.bytes();
                    let reply = session.dispatch_iq(&mut store, &service, &iq).unwrap();
                    assert_eq!(
                        blob_iq::decode_response(
                            &reply,
                            "forged-existing",
                            "bob@home.test",
                            "home.test"
                        ),
                        Err(blob_iq::Error::Remote(
                            faction_files::private_blobs::Error::Denied
                        ))
                    );
                    assert_eq!(f.bytes(), before);
                    assert_eq!(
                        reply,
                        blob_iq::response(
                            &iq,
                            "home.test",
                            Err(faction_files::private_blobs::Error::NotFound)
                        )
                        .unwrap()
                    );
                }
            }
        });
    }

    #[test]
    fn actor_free_iq_routes_are_assertions_and_denials_are_uniform() {
        let f = Fixture::new();
        let c = Certs::new();
        let runtime = c.runtime();
        let mut store = f.store();
        let before = f.bytes();
        let service = IqService::new("home.test").unwrap();
        with_session(&c, &runtime, &c.alice, |mut session| {
            for request in [
                Request::Status { id: [4; 32] },
                Request::GetChunk {
                    id: [4; 32],
                    offset: 0,
                },
            ] {
                let iq = query("forged", "bob@home.test", &request);
                let response = session.dispatch_iq(&mut store, &service, &iq).unwrap();
                assert_eq!(
                    blob_iq::decode_response(&response, "forged", "bob@home.test", "home.test"),
                    Err(blob_iq::Error::Remote(
                        faction_files::private_blobs::Error::Denied
                    ))
                );
                assert_eq!(before, f.bytes());
                let expected = blob_iq::response(
                    &iq,
                    "home.test",
                    Err(faction_files::private_blobs::Error::NotFound),
                )
                .unwrap();
                assert_eq!(response, expected);
            }
        });
    }

    #[test]
    fn local_service_foreign_routes_and_wrong_bound_principals_fail_closed() {
        let f = Fixture::new();
        let c = Certs::new();
        let runtime = c.runtime();
        let mut store = f.store();
        let before = f.bytes();
        let service = IqService::new("home.test").unwrap();
        for name in [
            "",
            "home",
            "home.test/evil",
            "HOME.test",
            "home.test@other.test",
        ] {
            assert!(IqService::new(name).is_err());
        }
        with_session(&c, &runtime, &c.alice, |mut session| {
            let away = blob_iq::request(
                "away",
                "alice@home.test",
                "away.test",
                &Request::Status { id: [4; 32] },
            )
            .unwrap();
            assert!(session.dispatch_iq(&mut store, &service, &away).is_err());
            let foreign = query("foreign", "bob@away.test", &Request::Status { id: [4; 32] });
            let reply = session.dispatch_iq(&mut store, &service, &foreign).unwrap();
            assert!(
                blob_iq::decode_response(&reply, "foreign", "bob@away.test", "home.test").is_err()
            );
            for variant in 0..3 {
                let mut b = binding();
                match variant {
                    0 => b.sender.account = "bob@home.test".into(),
                    1 => b.sender.device = 7,
                    _ => b.sender.identity[31] ^= 1,
                };
                let request = query("binding", "alice@home.test", &Request::Begin(b));
                let reply = session.dispatch_iq(&mut store, &service, &request).unwrap();
                assert!(
                    blob_iq::decode_response(&reply, "binding", "alice@home.test", "home.test")
                        .is_err()
                );
                assert_eq!(before, f.bytes());
            }
            let resource:Element="<iq xmlns='jabber:client' id='r' type='set' from='alice@home.test/claimed' to='home.test'><blob xmlns='urn:faction:blob:1'>AAAA</blob></iq>".parse().unwrap();
            assert!(
                session
                    .dispatch_iq(&mut store, &service, &resource)
                    .is_err()
            );
            assert_eq!(before, f.bytes());
        });
    }
    #[test]
    fn malformed_oversized_and_denied_iqs_share_budget_with_binary_before_decode() {
        let f = Fixture::new();
        let c = Certs::new();
        let runtime = c.runtime();
        let mut store = f.store();
        let before = f.bytes();
        let service = IqService::new("home.test").unwrap();
        with_session(&c, &runtime, &c.alice, |mut session| {
            assert!(
                session
                    .dispatch_bytes(&mut store, "alice@home.test", &[255])
                    .is_err()
            );
            let huge = Element::builder("iq", "jabber:client")
                .append("X".repeat(16385))
                .build();
            assert!(session.dispatch_iq(&mut store, &service, &huge).is_err());
            for raw in [
                "<iq xmlns='jabber:client' id='x' type='set' from='alice@home.test' to='home.test'><blob xmlns='urn:faction:blob:1'>AAAA</blob><blob xmlns='urn:faction:blob:1'>AAAA</blob></iq>",
                "<iq xmlns='jabber:client' id='x' type='set' from='alice@home.test' to='home.test'><blob xmlns='urn:faction:blob:1'>AAAAAA==</blob></iq>",
                "<iq xmlns='jabber:client' id='x' type='set' from='alice@home.test' to='home.test'><blob xmlns='urn:faction:blob:1'>AAAA </blob></iq>",
            ] {
                let e: Element = raw.parse().unwrap();
                assert!(session.dispatch_iq(&mut store, &service, &e).is_err());
            }
            for _ in 0..3 {
                let iq = query("denied", "bob@home.test", &Request::Status { id: [4; 32] });
                let reply = session.dispatch_iq(&mut store, &service, &iq).unwrap();
                assert!(
                    blob_iq::decode_response(&reply, "denied", "bob@home.test", "home.test")
                        .is_err()
                );
            }
            assert_eq!(before, f.bytes());
            let iq = query("ninth", "alice@home.test", &Request::Begin(binding()));
            assert_eq!(
                session.dispatch_iq(&mut store, &service, &iq),
                Err(Error::Transport(Stage::Limit))
            );
            assert_eq!(before, f.bytes());
        });
    }
    #[test]
    fn public_policy_changes_and_cancel_deny_next_iq_without_mutation() {
        let f = Fixture::new();
        let c = Certs::new();
        let mut store = f.store();
        let service = IqService::new("home.test").unwrap();
        let selected = binding();
        for variant in 0..5 {
            let runtime = c.runtime();
            let control = runtime.control();
            with_cancel_session(&c, &runtime, &c.alice, |mut session, cancel| {
                let first = query(
                    "first",
                    "alice@home.test",
                    &Request::Begin(selected.clone()),
                );
                assert!(session.dispatch_iq(&mut store, &service, &first).is_ok());
                let before = f.bytes();
                match variant {
                    0 => control.unavailable().unwrap(),
                    1 => control.reset(1).unwrap(),
                    2 => control
                        .replace_registry(current_registry(&c, 2, |e| e[0].approved = false))
                        .unwrap(),
                    3 => cancel.store(true, Ordering::SeqCst),
                    _ => runtime.stop(),
                };
                let iq = query(
                    "next",
                    "alice@home.test",
                    &Request::Status { id: selected.id },
                );
                if let Ok(reply) = session.dispatch_iq(&mut store, &service, &iq) {
                    assert!(
                        blob_iq::decode_response(&reply, "next", "alice@home.test", "home.test")
                            .is_err()
                    );
                }
                assert_eq!(before, f.bytes());
            });
        }
    }
    #[test]
    fn lost_typed_reply_reopen_exact_retry_and_explicit_delete_preserve_store_semantics() {
        let f = Fixture::new();
        let c = Certs::new();
        let runtime = c.runtime();
        let mut store = f.store();
        let service = IqService::new("home.test").unwrap();
        let b = binding();
        with_session(&c, &runtime, &c.alice, |mut session| {
            for (index, r) in [
                Request::Begin(b.clone()),
                Request::PutChunk {
                    id: b.id,
                    offset: 0,
                    bytes: vec![9; 32],
                },
                Request::Commit { id: b.id },
            ]
            .into_iter()
            .enumerate()
            {
                let iq = query(&format!("lost-{index}"), "alice@home.test", &r);
                drop(session.dispatch_iq(&mut store, &service, &iq).unwrap());
            }
        });
        drop(store);
        let mut store = f.store();
        with_session(&c, &runtime, &c.alice, |mut session| {
            let retry = query("retry", "alice@home.test", &Request::Commit { id: b.id });
            let reply = session.dispatch_iq(&mut store, &service, &retry).unwrap();
            assert!(matches!(
                blob_iq::decode_response(&reply, "retry", "alice@home.test", "home.test").unwrap(),
                Response::Progress {
                    committed: true,
                    ..
                }
            ));
            let delete = query("delete", "alice@home.test", &Request::Delete { id: b.id });
            let reply = session.dispatch_iq(&mut store, &service, &delete).unwrap();
            assert_eq!(
                blob_iq::decode_response(&reply, "delete", "alice@home.test", "home.test").unwrap(),
                Response::Deleted
            );
        });
    }
    #[test]
    fn store_quota_keeps_uniform_existing_failure_over_iq() {
        let f = Fixture::new();
        let c = Certs::new();
        let runtime = c.runtime();
        let mut store = f.store();
        let service = IqService::new("home.test").unwrap();
        with_session(&c, &runtime, &c.alice, |mut session| {
            for n in 1..=8 {
                let mut b = binding();
                b.id = [n; 32];
                let iq = query("reserve", "alice@home.test", &Request::Begin(b));
                let reply = session.dispatch_iq(&mut store, &service, &iq).unwrap();
                assert!(
                    blob_iq::decode_response(&reply, "reserve", "alice@home.test", "home.test")
                        .is_ok()
                );
            }
        });
        with_session(&c, &runtime, &c.alice, |mut session| {
            let mut b = binding();
            b.id = [99; 32];
            let iq = query("full", "alice@home.test", &Request::Begin(b));
            let reply = session.dispatch_iq(&mut store, &service, &iq).unwrap();
            assert_eq!(
                reply,
                blob_iq::response(
                    &iq,
                    "home.test",
                    Err(faction_files::private_blobs::Error::Denied)
                )
                .unwrap()
            );
        });
    }
    #[test]
    fn iq_calls_share_the_original_absolute_cycle_deadline() {
        let f = Fixture::new();
        let c = Certs::new();
        let runtime = c.runtime();
        let mut store = f.store();
        let before = f.bytes();
        let service = IqService::new("home.test").unwrap();
        with_session(&c, &runtime, &c.alice, |mut session| {
            thread::sleep(CYCLE_LIMIT);
            let iq = query("late", "alice@home.test", &Request::Begin(binding()));
            assert_eq!(
                session.dispatch_iq(&mut store, &service, &iq),
                Err(Error::Transport(Stage::Deadline))
            );
            assert_eq!(before, f.bytes());
        });
    }
}

#[test]
fn configured_binary_service_is_a_home_not_an_account_selector() {
    use faction_provider::BinaryService;
    assert!(BinaryService::new("home.test").is_ok());
    for home in [
        "",
        "HOME.test",
        "alice@home.test",
        "https://home.test",
        "home.test/path",
    ] {
        assert!(BinaryService::new(home).is_err());
    }
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let mut store = f.store();
    let objects = committed_route_objects(&c, &runtime, &mut store);
    let home = BinaryService::new("home.test").unwrap();
    let foreign = BinaryService::new("other.test").unwrap();
    with_session(&c, &runtime, &c.alice, |mut session| {
        for (request, expected) in committed_reads(&objects[0]) {
            assert_eq!(
                session
                    .dispatch_service(&mut store, &home, request)
                    .unwrap(),
                expected
            );
        }
        // Authorized reads can advance the durable high-water clock. The
        // denial baseline begins after these positive controls.
        for (request, _) in committed_reads(&objects[1]) {
            let before = f.bytes();
            assert!(
                session
                    .dispatch_service(&mut store, &home, request)
                    .is_err()
            );
            assert_eq!(f.bytes(), before);
        }
        let before = f.bytes();
        assert!(
            session
                .dispatch_service(&mut store, &foreign, Request::Status { id: objects[0].id })
                .is_err()
        );
        assert_eq!(f.bytes(), before);
    });
}

#[test]
fn bounded_preloaded_pem_helpers_reject_invalid_or_mismatched_credentials() {
    use faction_provider::tls::{ClientAdmission, ClientCredentials};
    let _f = Fixture::new();
    let c = Certs::new();
    let ca = c.ca.0.to_pem().unwrap();
    let cert = c.alice.0.to_pem().unwrap();
    let key = c.alice.1.private_key_to_pem_pkcs8().unwrap();
    let wrong = c.bob.1.private_key_to_pem_pkcs8().unwrap();
    assert!(ClientCredentials::from_pem(&ca, &cert, &key).is_ok());
    for (ca, cert, key) in [
        (b"".as_slice(), cert.as_slice(), key.as_slice()),
        (ca.as_slice(), b"".as_slice(), key.as_slice()),
        (ca.as_slice(), cert.as_slice(), b"".as_slice()),
        (ca.as_slice(), cert.as_slice(), wrong.as_slice()),
        (b"invalid".as_slice(), cert.as_slice(), key.as_slice()),
        (ca.as_slice(), b"invalid".as_slice(), key.as_slice()),
        (ca.as_slice(), cert.as_slice(), b"invalid".as_slice()),
    ] {
        assert!(ClientCredentials::from_pem(ca, cert, key).is_err());
        assert!(Server::from_pem(cert, key, ca).is_err());
    }
    let huge = vec![b'x'; 65_537];
    for (a, b, d) in [
        (&huge, &cert, &key),
        (&ca, &huge, &key),
        (&ca, &cert, &huge),
    ] {
        assert!(ClientCredentials::from_pem(a, b, d).is_err());
        assert!(Server::from_pem(b, d, a).is_err());
    }
    assert!(
        Server::from_pem(
            &c.server.0.to_pem().unwrap(),
            &c.server.1.private_key_to_pem_pkcs8().unwrap(),
            &ca
        )
        .is_ok()
    );
    let credentials = ClientCredentials::from_pem(&ca, &cert, &key).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    assert!(
        ClientAdmission::new(
            "provider.test",
            sha256(&c.server.0.to_der().unwrap()),
            credentials,
            cancel
        )
        .is_ok()
    );
}

#[test]
fn opaque_connected_admission_never_restarts_the_original_five_seconds() {
    use faction_provider::tls::{ClientAdmission, ClientCredentials};
    let _f = Fixture::new();
    let c = Certs::new();
    let credentials = ClientCredentials::from_pem(
        &c.ca.0.to_pem().unwrap(),
        &c.alice.0.to_pem().unwrap(),
        &c.alice.1.private_key_to_pem_pkcs8().unwrap(),
    )
    .unwrap();
    let token = ClientAdmission::new(
        "provider.test",
        sha256(&c.server.0.to_der().unwrap()),
        credentials,
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let socket = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (peer, _) = listener.accept().unwrap();
    thread::sleep(HANDSHAKE_LIMIT + Duration::from_millis(20));
    let start = Instant::now();
    assert!(matches!(token.connect(socket), Err(Stage::Deadline)));
    assert!(start.elapsed() < Duration::from_secs(1));
    drop(peer);
    drop(listener);
}

#[test]
fn local_binary_service_denies_foreign_bindings_with_existing_routed_read_controls() {
    use faction_provider::BinaryService;
    let f = Fixture::new();
    let c = Certs::new();
    let mut entries = c.entries();
    entries[1].principal.account = "bob@foreign.test".into();
    let runtime = Runtime::new(Some(
        Registry::new(
            1,
            0,
            vec!["home.test".into(), "foreign.test".into()],
            entries,
        )
        .unwrap(),
    ));
    let service = BinaryService::new("home.test").unwrap();
    let mut store = f.store();
    for foreign_sender in [false, true] {
        let mut b = binding();
        let mut foreign = principal(2);
        foreign.account = "bob@foreign.test".into();
        if foreign_sender {
            b.sender = foreign;
            b.recipient = principal(1);
        } else {
            b.recipient = foreign;
        }
        b.id = sha256(&[if foreign_sender { 5 } else { 6 }; 32]);
        let sender = if foreign_sender { &c.bob } else { &c.alice };
        let recipient = if foreign_sender { &c.alice } else { &c.bob };
        with_session(&c, &runtime, sender, |mut session| {
            for request in [
                Request::Begin(b.clone()),
                Request::PutChunk {
                    id: b.id,
                    offset: 0,
                    bytes: vec![if foreign_sender { 5 } else { 6 }; 32],
                },
                Request::Commit { id: b.id },
            ] {
                assert!(
                    session
                        .dispatch(&mut store, &b.sender.account, request)
                        .is_ok()
                );
            }
        });
        with_session(&c, &runtime, recipient, |mut session| {
            assert!(matches!(
                session.dispatch(
                    &mut store,
                    &b.recipient.account,
                    Request::GetChunk {
                        id: b.id,
                        offset: 0
                    }
                ),
                Ok(Response::Chunk { total: 32, .. })
            ));
        });
        with_session(&c, &runtime, &c.alice, |mut session| {
            for request in [
                Request::Status { id: b.id },
                Request::GetChunk {
                    id: b.id,
                    offset: 0,
                },
                Request::Begin(b.clone()),
            ] {
                let before = f.bytes();
                assert!(
                    session
                        .dispatch_service(&mut store, &service, request)
                        .is_err(),
                    "local service accepted an explicitly foreign binding"
                );
                assert_eq!(f.bytes(), before);
            }
        });
    }
}

#[test]
fn connected_client_admission_preserves_actual_ca_name_pin_and_binding_policy() {
    use faction_provider::{
        BinaryService,
        tls::{ClientAdmission, ClientCredentials},
    };
    let f = Fixture::new();
    let c = Certs::new();
    let other = Certs::new();
    let runtime = c.runtime();
    let mut store = f.store();
    let time = wall_now() as i64;
    let unknown = cert(
        Some((&c.ca.0, &c.ca.1)),
        "unknown-device",
        91,
        time - 60,
        time + 3600,
    );
    let expired = cert(
        Some((&c.ca.0, &c.ca.1)),
        "expired-device",
        92,
        time - 3600,
        time - 1,
    );
    for case in 0..7 {
        let before = f.bytes();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let ca = if case == 3 { &other.ca.0 } else { &c.ca.0 };
        let identity = match case {
            4 => &other.alice,
            5 => &unknown,
            6 => &expired,
            _ => &c.alice,
        };
        let credentials = ClientCredentials::from_pem(
            &ca.to_pem().unwrap(),
            &identity.0.to_pem().unwrap(),
            &identity.1.private_key_to_pem_pkcs8().unwrap(),
        )
        .unwrap();
        let name = if case == 1 {
            "wrong.test"
        } else {
            "provider.test"
        };
        let pin = if case == 2 {
            [7; 32]
        } else {
            sha256(&c.server.0.to_der().unwrap())
        };
        let token =
            ClientAdmission::new(name, pin, credentials, Arc::new(AtomicBool::new(false))).unwrap();
        let (client_ok, served) = thread::scope(|scope| {
            let server = scope.spawn(|| {
                let service = BinaryService::new("home.test").unwrap();
                match c
                    .server()
                    .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
                {
                    Ok(mut session) => {
                        matches!(session.serve_service_next(&mut store, &service), Ok(true))
                    }
                    Err(_) => false,
                }
            });
            let client = token.connect(TcpStream::connect(address).unwrap());
            let result = match client {
                Ok(mut client) => client.request(&Request::Begin(binding())).is_ok(),
                Err(_) => false,
            };
            (result, server.join().unwrap())
        });
        assert_eq!(client_ok, case == 0, "case {case}");
        assert_eq!(served, case == 0, "case {case}");
        if case != 0 {
            assert_eq!(f.bytes(), before);
        }
    }
}

#[test]
fn configured_service_rechecks_current_full_identity_peer_policy_and_live_lifecycle() {
    use faction_provider::BinaryService;
    let f = Fixture::new();
    let c = Certs::new();
    let runtime = c.runtime();
    let mut store = f.store();
    let b = binding();
    let service = BinaryService::new("home.test").unwrap();
    with_session(&c, &runtime, &c.alice, |mut session| {
        for request in [
            Request::Begin(b.clone()),
            Request::PutChunk {
                id: b.id,
                offset: 0,
                bytes: vec![9; 32],
            },
            Request::Commit { id: b.id },
        ] {
            assert!(
                session
                    .dispatch_service(&mut store, &service, request)
                    .is_ok()
            );
        }
    });
    with_session(&c, &runtime, &c.bob, |mut session| {
        assert!(matches!(
            session.dispatch_service(
                &mut store,
                &service,
                Request::GetChunk {
                    id: b.id,
                    offset: 0
                }
            ),
            Ok(Response::Chunk { total: 32, .. })
        ));
    });
    for case in 0..6 {
        let mut entries = c.entries();
        match case {
            0 => entries[0].principal.identity[31] ^= 1,
            1 => entries[0].device_enabled = false,
            2 => entries[0].account_enabled = false,
            3 => entries[0].approved = false,
            4 => entries[1].principal.identity[31] ^= 1,
            _ => {
                entries.remove(0);
            }
        };
        let changed = Runtime::new(Some(
            Registry::new(1, 0, vec!["home.test".into()], entries).unwrap(),
        ));
        with_session(&c, &changed, &c.bob, |mut session| {
            let before = f.bytes();
            assert!(
                session
                    .dispatch_service(
                        &mut store,
                        &service,
                        Request::GetChunk {
                            id: b.id,
                            offset: 0
                        }
                    )
                    .is_err()
            );
            assert_eq!(f.bytes(), before);
        });
    }
    for case in 0..4 {
        let current = c.runtime();
        let control = current.control();
        with_session(&c, &current, &c.bob, |mut session| {
            assert!(
                session
                    .dispatch_service(&mut store, &service, Request::Status { id: b.id })
                    .is_ok()
            );
            let before = f.bytes();
            match case {
                0 => control.unavailable().unwrap(),
                1 => control.reset(1).unwrap(),
                2 => control.stop().unwrap(),
                _ => control
                    .replace_registry(current_registry(&c, 2, |_| {}))
                    .unwrap(),
            };
            assert!(
                session
                    .dispatch_service(
                        &mut store,
                        &service,
                        Request::GetChunk {
                            id: b.id,
                            offset: 0
                        }
                    )
                    .is_err()
            );
            assert_eq!(f.bytes(), before);
        });
    }
}
