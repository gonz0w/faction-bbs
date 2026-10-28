//! Actual disposable loopback TLS1.3 observations; synthetic identities only.
use super::*;
use crate::authority::{CredentialBinding, Registry};
use faction_files::private_blobs::{Binding, Principal};
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
            "faction271-owned-{}-{}-{}",
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
        f.write_all(b"issue271 synthetic TLS fixture").unwrap();
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
            b"issue271 synthetic TLS fixture"
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
    fn authority(&self) -> Authority {
        Authority::new(Some(
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
fn connection_is_refused(addr: SocketAddr) -> bool {
    TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_err()
}
#[test]
fn refused_connection_observer_rejects_live_and_retained_listeners() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    assert!(!connection_is_refused(addr), "live listener is reachable");
    let retained = listener.try_clone().unwrap();
    drop(listener);
    assert!(
        !connection_is_refused(addr),
        "retained listener is still reachable"
    );
    drop(retained);
    assert!(connection_is_refused(addr), "final listener is closed");
}
fn cycle<T>(
    server: &mut Server,
    a: &mut Authority,
    s: &mut Store,
    action: impl FnOnce(SocketAddr, Arc<AtomicBool>) -> T,
) -> (T, Report) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let result = thread::scope(|scope| {
        let c = cancel.clone();
        let handler = scope.spawn(|| server.serve_once(&listener, a, s, c));
        let value = action(addr, cancel.clone());
        let drain = Instant::now();
        let report = handler.join().unwrap();
        assert!(
            drain.elapsed() < Duration::from_secs(5),
            "bounded handler drain"
        );
        (value, report)
    });
    drop(listener);
    assert!(connection_is_refused(addr));
    result
}
#[test]
fn actual_verified_sender_upload_then_exact_recipient_download_and_retry() {
    let f = Fixture::new();
    let c = Certs::new();
    let mut a = c.authority();
    let mut s = f.store();
    let mut server = c.server();
    let b = binding();
    let (_, report) = cycle(&mut server, &mut a, &mut s, |addr, cancel| {
        let mut client = c.client(addr, cancel, Some(&c.alice)).unwrap();
        assert_eq!(
            client
                .stream
                .ssl()
                .ssl_context()
                .cert_store()
                .all_certificates()
                .len(),
            1,
            "only the fixture CA may be trusted"
        );
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
            client.request(&r).unwrap();
        }
    });
    assert_eq!(report.stage, Stage::Complete);
    assert_eq!(report.dispatched, 5);
    let (_, report) = cycle(&mut server, &mut a, &mut s, |addr, cancel| {
        let mut client = c.client(addr, cancel, Some(&c.bob)).unwrap();
        assert_eq!(
            client
                .request(&Request::GetChunk {
                    id: b.id,
                    offset: 0
                })
                .unwrap(),
            Response::Chunk {
                offset: 0,
                bytes: vec![9; 32],
                total: 32
            }
        );
        client.request(&Request::Status { id: b.id }).unwrap();
    });
    assert_eq!(report.dispatched, 2);
}
#[test]
fn tls_refusals_have_actual_stages_and_zero_application_dispatch() {
    let f = Fixture::new();
    let c = Certs::new();
    let mut s = f.store();
    let before = f.bytes();
    for variant in 0..11 {
        let mut a = c.authority();
        if variant == 8 {
            a.unavailable();
        }
        if variant == 10 {
            let mut entries = c.entries();
            entries[0].approved = false;
            a = Authority::new(Some(
                Registry::new(1, 0, vec!["home.test".into()], entries).unwrap(),
            ));
        }
        let mut server = c.server();
        let now = wall_now() as i64;
        let rogue = cert(None, "rogue", 20, now - 60, now + 3600);
        let expired = cert(Some((&c.ca.0, &c.ca.1)), "expired", 21, now - 100, now - 1);
        let future = cert(Some((&c.ca.0, &c.ca.1)), "future", 22, now + 100, now + 200);
        let unmapped = cert(
            Some((&c.ca.0, &c.ca.1)),
            "unmapped",
            23,
            now - 60,
            now + 100,
        );
        let (result, report) = cycle(&mut server, &mut a, &mut s, |addr, cancel| {
            let name = if variant == 5 {
                "wrong.test"
            } else {
                "provider.test"
            };
            let pin = if variant == 6 {
                [0; 32]
            } else {
                sha256(&c.server.0.to_der().unwrap())
            };
            let identity = match variant {
                0 => None,
                1 => Some((&rogue.0, Some(&rogue.1))),
                2 => Some((&expired.0, Some(&expired.1))),
                3 => Some((&future.0, Some(&future.1))),
                4 => Some((&unmapped.0, Some(&unmapped.1))),
                7 => Some((&c.alice.0, Some(&c.bob.1))),
                _ => Some((&c.alice.0, Some(&c.alice.1))),
            };
            match Client::connect(
                addr,
                name,
                pin,
                if variant == 9 { &rogue.0 } else { &c.ca.0 },
                identity,
                cancel.clone(),
            ) {
                Ok(mut client) => client.request(&Request::Begin(binding())).map(|_| ()),
                Err(stage) => {
                    cancel.store(true, Ordering::Relaxed);
                    Err(stage)
                }
            }
        });
        println!("TLS refusal variant={variant} client={result:?} server={report:?}");
        assert!(result.is_err(), "variant {variant}");
        assert_eq!(report.dispatched, 0, "variant {variant}: {report:?}");
        if variant <= 3 {
            assert_eq!(report.stage, Stage::Handshake);
        }
        if [5, 9].contains(&variant) {
            assert_eq!(result, Err(Stage::Handshake));
        }
        if [4, 8, 10].contains(&variant) {
            assert_eq!(report.stage, Stage::Denied);
        }
        if variant == 6 {
            assert_eq!(result, Err(Stage::Pin));
        }
        if variant == 7 {
            assert_eq!(result, Err(Stage::Configuration));
        }
        assert_eq!(f.bytes(), before);
    }
    // A copied public certificate without a private key is a configuration refusal.
    let (result, report) = cycle(
        &mut c.server(),
        &mut c.authority(),
        &mut s,
        |addr, cancel| {
            let r = Client::connect(
                addr,
                "provider.test",
                sha256(&c.server.0.to_der().unwrap()),
                &c.ca.0,
                Some((&c.alice.0, None)),
                cancel.clone(),
            );
            cancel.store(true, Ordering::Relaxed);
            r.err()
        },
    );
    println!("copied-certificate-without-key client={result:?} server={report:?}");
    assert_eq!(result, Some(Stage::Configuration));
    assert_eq!(report.dispatched, 0);
}
#[test]
fn forged_binding_and_malformed_oversized_or_trailing_frames_do_not_mutate() {
    let f = Fixture::new();
    let c = Certs::new();
    let mut s = f.store();
    let before = f.bytes();
    for variant in 0..7 {
        let mut a = c.authority();
        let mut server = c.server();
        let (_, report) = cycle(&mut server, &mut a, &mut s, |addr, cancel| {
            let mut client = c.client(addr, cancel, Some(&c.alice)).unwrap();
            if variant < 3 {
                let mut b = binding();
                match variant {
                    0 => b.sender.account = "mallory@home.test".into(),
                    1 => b.sender.device = 99,
                    _ => b.sender.identity[31] ^= 1,
                }
                assert!(client.request(&Request::Begin(b)).is_err());
            } else {
                let bytes = if variant == 3 {
                    vec![0; MAX_WIRE + 1]
                } else if variant == 4 {
                    vec![1, 2, 3]
                } else {
                    let mut raw = encode_request(&Request::Begin(binding())).unwrap();
                    raw.push(0);
                    raw
                };
                client
                    .stream
                    .write_all(&(bytes.len() as u32).to_be_bytes())
                    .unwrap();
                if variant != 3 {
                    client.stream.write_all(&bytes).unwrap();
                }
                let mut byte = [0];
                let _ = client.stream.read(&mut byte);
            }
        });
        assert_eq!(
            report.stage,
            if variant < 3 {
                Stage::Denied
            } else {
                Stage::Frame
            }
        );
        assert_eq!(f.bytes(), before);
    }
}
#[test]
fn live_policy_changes_and_actual_leaf_expiry_deny_the_next_operation() {
    let f = Fixture::new();
    let c = Certs::new();
    let mut s = f.store();
    for variant in 0..8 {
        let mut a = c.authority();
        let mut server = c.server();
        let b = binding();
        // Only the live leaf-expiry control advances an explicitly synthetic wall clock.
        let clock = Arc::new(AtomicU64::new(wall_now()));
        let observed = clock.clone();
        server.clock = Arc::new(move || observed.load(Ordering::Relaxed));
        let mut entries = c.entries();
        server.hook = Some(Box::new(move |a, index| {
            if index == 1 {
                match variant {
                    0 => entries[0].account_enabled = false,
                    1 => entries[0].device_enabled = false,
                    2 => entries[1].device_enabled = false,
                    3 => {
                        a.reset(1).unwrap();
                        return;
                    }
                    4 => {
                        a.unavailable();
                        return;
                    }
                    5 => {
                        clock.store(wall_now() + 4000, Ordering::Relaxed);
                        return;
                    }
                    6 => {
                        entries.remove(1);
                    }
                    _ => {}
                }
                for e in &mut entries {
                    e.generation = 2;
                }
                a.replace_registry(
                    Registry::new(2, 0, vec!["home.test".into()], entries.clone()).unwrap(),
                )
                .unwrap();
            }
        }));
        let (_, report) = cycle(&mut server, &mut a, &mut s, |addr, cancel| {
            let mut client = c.client(addr, cancel, Some(&c.alice)).unwrap();
            client.request(&Request::Begin(b.clone())).unwrap();
            assert!(
                client
                    .request(&Request::PutChunk {
                        id: b.id,
                        offset: 0,
                        bytes: vec![9; 32]
                    })
                    .is_err()
            );
        });
        println!("current-policy variant={variant} server={report:?}");
        assert_eq!(report.dispatched, if variant == 5 { 1 } else { 2 });
        assert_eq!(report.stage, Stage::Denied);
        // Same id exact retry in next cycle permits cleanup under restored explicit authority.
        a = c.authority();
        cycle(&mut c.server(), &mut a, &mut s, |addr, cancel| {
            let mut client = c.client(addr, cancel, Some(&c.alice)).unwrap();
            client.request(&Request::Delete { id: b.id }).unwrap();
        });
    }
}
#[test]
fn eight_attempts_quota_conflicting_duplicate_and_lost_response_recovery() {
    let f = Fixture::new();
    let c = Certs::new();
    let mut s = f.store();
    let mut a = c.authority();
    let b = binding();
    let (_, r) = cycle(&mut c.server(), &mut a, &mut s, |addr, cancel| {
        let mut client = c.client(addr, cancel, Some(&c.alice)).unwrap();
        for _ in 0..8 {
            client.request(&Request::Begin(b.clone())).unwrap();
        }
        assert_eq!(
            client.request(&Request::Begin(b.clone())),
            Err(Stage::Limit)
        );
    });
    assert_eq!(r.dispatched, 8);
    assert_eq!(r.stage, Stage::Limit);
    let before = f.bytes();
    let (_, r) = cycle(&mut c.server(), &mut a, &mut s, |addr, cancel| {
        let mut client = c.client(addr, cancel, Some(&c.alice)).unwrap();
        let mut altered = b.clone();
        altered.expires += 1;
        assert!(client.request(&Request::Begin(altered)).is_err());
    });
    assert_eq!(r.stage, Stage::Denied);
    assert_eq!(f.bytes(), before);
    // Drop the response deliberately only after dispatch has succeeded, then reopen and retry exact bytes.
    let mut server = c.server();
    server.lose_response = true;
    let (_, r) = cycle(&mut server, &mut a, &mut s, |addr, cancel| {
        let mut client = c.client(addr, cancel, Some(&c.alice)).unwrap();
        assert!(
            client
                .request(&Request::PutChunk {
                    id: b.id,
                    offset: 0,
                    bytes: vec![9; 32]
                })
                .is_err()
        );
    });
    assert_eq!(r.dispatched, 1);
    drop(s);
    let mut s = f.store();
    cycle(&mut c.server(), &mut a, &mut s, |addr, cancel| {
        let mut client = c.client(addr, cancel, Some(&c.alice)).unwrap();
        assert!(matches!(
            client.request(&Request::PutChunk {
                id: b.id,
                offset: 0,
                bytes: vec![9; 32]
            }),
            Ok(Response::Progress { offset: 32, .. })
        ));
    });
    cycle(&mut c.server(), &mut a, &mut s, |addr, cancel| {
        let mut client = c.client(addr, cancel, Some(&c.alice)).unwrap();
        for n in 0..7 {
            client
                .request(&Request::Begin(Binding {
                    id: [n; 32],
                    size: 1_048_592,
                    ..b.clone()
                }))
                .unwrap();
        }
    });
    let before = f.bytes();
    let (_, r) = cycle(&mut c.server(), &mut a, &mut s, |addr, cancel| {
        let mut client = c.client(addr, cancel, Some(&c.alice)).unwrap();
        assert!(
            client
                .request(&Request::Begin(Binding {
                    id: [99; 32],
                    ..b.clone()
                }))
                .is_err()
        );
    });
    assert_eq!(r.stage, Stage::Denied);
    assert_eq!(r.dispatched, 1);
    assert_eq!(f.bytes(), before);
}
#[test]
fn real_stalled_handshake_and_frame_observe_five_and_ten_second_absolute_limits() {
    let f = Fixture::new();
    let c = Certs::new();
    let mut s = f.store();
    let mut a = c.authority();
    let started = Instant::now();
    let (_, r) = cycle(&mut c.server(), &mut a, &mut s, |addr, _| {
        let mut socket = TcpStream::connect(addr).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(7)))
            .unwrap();
        let mut byte = [0];
        assert_eq!(socket.read(&mut byte).unwrap(), 0);
    });
    println!(
        "stalled-handshake elapsed_ms={} report={r:?}",
        started.elapsed().as_millis()
    );
    assert_eq!(r.stage, Stage::Deadline);
    assert!(started.elapsed() >= Duration::from_secs(5));
    assert!(started.elapsed() < Duration::from_secs(7));
    let started = Instant::now();
    let (_, r) = cycle(&mut c.server(), &mut a, &mut s, |addr, cancel| {
        let mut client = c.client(addr, cancel, Some(&c.alice)).unwrap();
        client.stream.get_mut().deadline = Instant::now() + Duration::from_secs(12);
        client.stream.write_all(&[0, 0]).unwrap();
        let mut byte = [0];
        let _ = client.stream.read(&mut byte);
    });
    println!(
        "stalled-frame elapsed_ms={} report={r:?}",
        started.elapsed().as_millis()
    );
    assert_eq!(r.stage, Stage::Deadline);
    assert!(started.elapsed() >= Duration::from_secs(10));
    assert!(started.elapsed() < Duration::from_secs(12));
}
#[test]
fn cancellation_and_trickle_recompute_the_same_absolute_io_deadline() {
    let f = Fixture::new();
    let c = Certs::new();
    let mut s = f.store();
    let mut a = c.authority();
    let started = Instant::now();
    let (_, r) = cycle(&mut c.server(), &mut a, &mut s, |addr, cancel| {
        let _socket = TcpStream::connect(addr).unwrap();
        cancel.store(true, Ordering::Relaxed);
    });
    assert_eq!(r.stage, Stage::Cancelled);
    assert!(started.elapsed() < Duration::from_secs(1));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    thread::scope(|scope| {
        let producer = scope.spawn(|| {
            let mut socket = TcpStream::connect(addr).unwrap();
            for _ in 0..20 {
                if socket.write_all(&[1]).is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(20));
            }
        });
        let (socket, _) = listener.accept().unwrap();
        let start = Instant::now();
        let mut io = DeadlineIo::new(socket, start + Duration::from_millis(120), cancel);
        let mut bytes = [0; 20];
        assert!(io.read_exact(&mut bytes).is_err());
        assert!(start.elapsed() < Duration::from_millis(250));
        drop(io);
        producer.join().unwrap();
    });
}

// Controlled post-BIO scheduling delays: these do not measure a slow TLS provider.
#[test]
fn deadline_regression_server_rechecks_after_successful_tls_before_authority() {
    let f = Fixture::new();
    let c = Certs::new();
    let mut s = f.store();
    let before = f.bytes();
    let mut a = c.authority();
    let mut server = c.server();
    server.post_handshake = Some(Box::new(|| thread::sleep(Duration::from_millis(5100))));
    let start = Instant::now();
    let (result, report) = cycle(&mut server, &mut a, &mut s, |addr, cancel| {
        let mut client = c.client(addr, cancel, Some(&c.alice)).unwrap();
        client.request(&Request::Begin(binding()))
    });
    println!(
        "controlled-server-post-BIO elapsed_ms={} result={result:?} report={report:?}",
        start.elapsed().as_millis()
    );
    assert_eq!(report.stage, Stage::Deadline);
    assert_eq!(report.dispatched, 0);
    assert!(result.is_err());
    assert_eq!(f.bytes(), before);
}
#[test]
fn deadline_regression_client_rechecks_after_successful_tls_before_pin_admission() {
    let f = Fixture::new();
    let c = Certs::new();
    let mut s = f.store();
    let before = f.bytes();
    let mut a = c.authority();
    CLIENT_POST_HANDSHAKE.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(|| thread::sleep(Duration::from_millis(5100))))
    });
    let start = Instant::now();
    let (result, report) = cycle(&mut c.server(), &mut a, &mut s, |addr, cancel| {
        match c.client(addr, cancel, Some(&c.alice)) {
            Ok(mut client) => client.request(&Request::Begin(binding())).map(|_| ()),
            Err(stage) => Err(stage),
        }
    });
    println!(
        "controlled-client-post-BIO elapsed_ms={} result={result:?} report={report:?}",
        start.elapsed().as_millis()
    );
    assert_eq!(result, Err(Stage::Deadline));
    assert_eq!(report.dispatched, 0);
    assert_eq!(f.bytes(), before);
}
#[test]
fn deadline_regression_tcp_wait_and_tls_share_one_five_second_admission_budget() {
    let f = Fixture::new();
    let c = Certs::new();
    let mut s = f.store();
    let mut a = c.authority();
    let start = Instant::now();
    let (_, report) = cycle(&mut c.server(), &mut a, &mut s, |addr, _| {
        thread::sleep(Duration::from_secs(2));
        let mut socket = TcpStream::connect(addr).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(7)))
            .unwrap();
        let mut byte = [0];
        assert_eq!(socket.read(&mut byte).unwrap(), 0);
    });
    let elapsed = start.elapsed();
    println!(
        "delayed-TCP-plus-stalled-TLS elapsed_ms={} report={report:?}",
        elapsed.as_millis()
    );
    assert_eq!(report.stage, Stage::Deadline);
    assert_eq!(report.dispatched, 0);
    assert!(elapsed >= Duration::from_secs(5));
    assert!(elapsed < Duration::from_secs(6));
}
