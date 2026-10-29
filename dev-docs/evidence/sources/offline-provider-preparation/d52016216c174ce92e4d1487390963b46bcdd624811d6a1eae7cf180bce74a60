#![cfg(feature = "operator")]
//! Actual maintained-command process tests with owned synthetic policy and TLS.
use faction_files::private_blobs::{Binding, Principal, Request, Response};
use faction_provider::tls::Client;
use openssl::{
    asn1::Asn1Time,
    bn::{BigNum, MsbOption},
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    nid::Nid,
    pkey::{PKey, Private},
    sha::sha256,
    x509::{
        X509, X509NameBuilder,
        extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage, SubjectAlternativeName},
    },
};
use std::{
    fs,
    io::Read,
    net::{SocketAddr, TcpStream},
    os::unix::fs::{DirBuilderExt, PermissionsExt, symlink},
    path::PathBuf,
    process::{Child, Command, Output, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn key() -> PKey<Private> {
    PKey::from_ec_key(
        EcKey::generate(&EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap()).unwrap(),
    )
    .unwrap()
}
fn certificate(
    key: &PKey<Private>,
    ca: Option<(&X509, &PKey<Private>)>,
    server: bool,
    wrong_name: bool,
    expired: bool,
) -> X509 {
    let mut b = X509::builder().unwrap();
    b.set_version(2).unwrap();
    let mut serial = BigNum::new().unwrap();
    serial.rand(96, MsbOption::MAYBE_ZERO, false).unwrap();
    b.set_serial_number(&serial.to_asn1_integer().unwrap())
        .unwrap();
    let mut name = X509NameBuilder::new().unwrap();
    name.append_entry_by_text(
        "CN",
        if ca.is_none() {
            "synthetic-ca"
        } else {
            "synthetic-leaf"
        },
    )
    .unwrap();
    let name = name.build();
    b.set_subject_name(&name).unwrap();
    b.set_issuer_name(ca.map_or(name.as_ref(), |(c, _)| c.subject_name()))
        .unwrap();
    b.set_pubkey(key).unwrap();
    b.set_not_before(&Asn1Time::from_unix(now() as i64 - 3600).unwrap())
        .unwrap();
    b.set_not_after(
        &Asn1Time::from_unix(now() as i64 + if expired { -60 } else { 86400 }).unwrap(),
    )
    .unwrap();
    if ca.is_none() {
        b.append_extension(BasicConstraints::new().critical().ca().build().unwrap())
            .unwrap();
        b.append_extension(
            KeyUsage::new()
                .critical()
                .key_cert_sign()
                .crl_sign()
                .build()
                .unwrap(),
        )
        .unwrap();
    } else {
        b.append_extension(BasicConstraints::new().critical().build().unwrap())
            .unwrap();
        b.append_extension(
            KeyUsage::new()
                .critical()
                .digital_signature()
                .build()
                .unwrap(),
        )
        .unwrap();
        b.append_extension(
            if server {
                ExtendedKeyUsage::new().server_auth().build()
            } else {
                ExtendedKeyUsage::new().client_auth().build()
            }
            .unwrap(),
        )
        .unwrap();
        if server {
            let san = SubjectAlternativeName::new()
                .dns(if wrong_name {
                    "wrong.test"
                } else {
                    "home.test"
                })
                .build(&b.x509v3_context(ca.map(|(c, _)| c.as_ref()), None))
                .unwrap();
            b.append_extension(san).unwrap();
        }
    }
    b.sign(ca.map_or(key, |(_, k)| k), MessageDigest::sha256())
        .unwrap();
    b.build()
}
struct Fixture {
    root: PathBuf,
    ca: X509,
    server: X509,
    devices: Vec<(X509, PKey<Private>)>,
    config: String,
    proposal: String,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "faction284-OWNED-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        for dir in ["policy", "blobs"] {
            fs::DirBuilder::new()
                .mode(0o700)
                .create(root.join(dir))
                .unwrap();
        }
        let ca_key = key();
        let ca = certificate(&ca_key, None, false, false, false);
        let server_key = key();
        let server = certificate(&server_key, Some((&ca, &ca_key)), true, false, false);
        let devices = (0..2)
            .map(|_| {
                let k = key();
                (
                    certificate(&k, Some((&ca, &ca_key)), false, false, false),
                    k,
                )
            })
            .collect::<Vec<_>>();
        let config = format!(
            "version=1\nhome=\"home.test\"\nendpoint=\"127.0.0.1:0\"\npolicy_root=\"{}\"\nblob_root=\"{}\"\nca=\"{}\"\ncertificate=\"{}\"\nkey=\"{}\"\n",
            root.join("policy").display(),
            root.join("blobs").display(),
            root.join("ca.pem").display(),
            root.join("server.pem").display(),
            root.join("server.key").display()
        );
        let mut f = Self {
            root,
            ca,
            server,
            devices,
            config,
            proposal: String::new(),
        };
        f.write("OWNED", b"issue284 maintained operator tests");
        f.write("ca.pem", &f.ca.to_pem().unwrap());
        f.write("server.pem", &f.server.to_pem().unwrap());
        f.write(
            "server.key",
            &server_key.private_key_to_pem_pkcs8().unwrap(),
        );
        f.write("config.toml", f.config.as_bytes());
        f.proposal = f.policy(1, 1, [true, true]);
        f.write("proposal.toml", f.proposal.as_bytes());
        // Negative certificate inputs are generated outside the implementation.
        let wrong = certificate(&server_key, Some((&f.ca, &ca_key)), true, true, false);
        f.write("wrong-name.pem", &wrong.to_pem().unwrap());
        let expired = certificate(&server_key, Some((&f.ca, &ca_key)), true, false, true);
        f.write("expired.pem", &expired.to_pem().unwrap());
        let foreign_key = key();
        let foreign_ca = certificate(&foreign_key, None, false, false, false);
        f.write("foreign-ca.pem", &foreign_ca.to_pem().unwrap());
        f
    }
    fn write(&self, name: &str, bytes: &[u8]) {
        fs::write(self.root.join(name), bytes).unwrap();
        fs::set_permissions(self.root.join(name), fs::Permissions::from_mode(0o600)).unwrap();
    }
    fn policy(&self, g: u64, r: u64, approved: [bool; 2]) -> String {
        let mut s = format!("version=1\ngeneration={g}\nreset_epoch={r}\nhomes=[\"home.test\"]\n");
        for (i, (cert, _)) in self.devices.iter().enumerate() {
            s += &format!(
                "\n[[bindings]]\ncertificate_sha256=\"{}\"\naccount=\"member{}@home.test\"\ndevice={}\nidentity=\"{}\"\nvalid_from={}\nvalid_until={}\napproved={}\naccount_enabled=true\ndevice_enabled=true\ngeneration={}\n",
                hex(&sha256(&cert.to_der().unwrap())),
                i + 1,
                i + 1,
                hex(&[i as u8 + 1; 32]),
                now() - 60,
                now() + 3600,
                approved[i],
                g
            );
        }
        s
    }
    fn command(&self, action: &str) -> Command {
        let mut c = Command::new(
            std::env::var_os("FACTION_287_OPERATOR_BINARY")
                .unwrap_or_else(|| env!("CARGO_BIN_EXE_faction-provider").into()),
        );
        c.arg(action).arg(self.root.join("config.toml"));
        c
    }
    fn run(&self, action: &str, extra: Option<&str>) -> Output {
        let mut c = self.command(action);
        if let Some(extra) = extra {
            c.arg(extra);
        }
        bounded(c)
    }
    fn init(&self) {
        let o = self.run(
            "init",
            Some(self.root.join("proposal.toml").to_str().unwrap()),
        );
        assert!(o.status.success(), "{:?}", o);
        assert_eq!(o.stdout, b"initialized\n");
    }
    fn summary(&self) -> Vec<u8> {
        let o = self.run("inspect", None);
        assert!(o.status.success(), "{:?}", o);
        o.stdout
    }
    fn snapshot(&self) -> Vec<u8> {
        fs::read(self.root.join("policy/provider-policy.snapshot")).unwrap()
    }
    fn start(&self) -> Serving {
        Serving::start(self.command("serve"))
    }
    fn connect(&self, port: u16, device: usize) -> Result<Client, faction_provider::tls::Stage> {
        let (cert, key) = &self.devices[device];
        Client::connect(
            SocketAddr::from(([127, 0, 0, 1], port)),
            "home.test",
            sha256(&self.server.to_der().unwrap()),
            &self.ca,
            Some((cert, Some(key))),
            Arc::new(AtomicBool::new(false)),
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert_eq!(
            fs::read(self.root.join("OWNED")).unwrap(),
            b"issue284 maintained operator tests"
        );
        if std::thread::panicking() {
            eprintln!("retained failed synthetic fixture {:?}", self.root);
        } else {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
}
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().unwrap().is_none() {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}
fn bounded(mut c: Command) -> Output {
    let mut child = OwnedChild(
        c.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let start = Instant::now();
    while child.0.try_wait().unwrap().is_none() {
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "CLI child exceeded bound"
        );
        thread::sleep(Duration::from_millis(5));
    }
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    child
        .0
        .stdout
        .take()
        .unwrap()
        .take(8193)
        .read_to_end(&mut stdout)
        .unwrap();
    child
        .0
        .stderr
        .take()
        .unwrap()
        .take(8193)
        .read_to_end(&mut stderr)
        .unwrap();
    assert!(stdout.len() <= 8192 && stderr.len() <= 8192);
    Output {
        status: child.0.wait().unwrap(),
        stdout,
        stderr,
    }
}
struct Serving {
    child: OwnedChild,
    port: u16,
}
impl Serving {
    fn start(mut c: Command) -> Self {
        let mut child = OwnedChild(
            c.stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        let stdout = child.0.stdout.as_mut().unwrap();
        rustix::fs::fcntl_setfl(&*stdout, rustix::fs::OFlags::NONBLOCK).unwrap();
        let mut raw = Vec::new();
        let start = Instant::now();
        while !raw.ends_with(b"\n") {
            let mut b = [0; 1];
            match stdout.read(&mut b) {
                Ok(1) => raw.push(b[0]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5))
                }
                other => panic!("readiness failed {other:?}"),
            };
            assert!(
                raw.len() < 65 && start.elapsed() < Duration::from_secs(5),
                "readiness unavailable"
            );
        }
        let line = std::str::from_utf8(&raw).unwrap();
        let port = line
            .strip_prefix("ready ")
            .unwrap()
            .trim_end()
            .parse::<u16>()
            .unwrap();
        assert_ne!(port, 0);
        Self { child, port }
    }
    fn stop(mut self, signal: rustix::process::Signal) {
        let start = Instant::now();
        rustix::process::kill_process(
            rustix::process::Pid::from_raw(self.child.0.id() as i32).unwrap(),
            signal,
        )
        .unwrap();
        while self.child.0.try_wait().unwrap().is_none() {
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "orderly drain exceeded five seconds"
            );
            thread::sleep(Duration::from_millis(5));
        }
        assert!(self.child.0.wait().unwrap().success());
        let mut out = Vec::new();
        self.child
            .0
            .stdout
            .take()
            .unwrap()
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, b"stopped\n");
        let mut err = Vec::new();
        self.child
            .0
            .stderr
            .take()
            .unwrap()
            .read_to_end(&mut err)
            .unwrap();
        assert!(err.is_empty());
        assert!(TcpStream::connect(("127.0.0.1", self.port)).is_err());
    }
}
#[test]
fn explicit_offline_roundtrip_reset_and_inspection_never_need_tls_key() {
    let f = Fixture::new();
    assert!(!f.run("inspect", None).status.success());
    f.init();
    assert!(
        !f.run("init", Some(f.root.join("proposal.toml").to_str().unwrap()))
            .status
            .success()
    );
    assert_eq!(
        f.summary(),
        b"policy generation=1 reset=1 available=1 bindings=2 history=2\n"
    );
    f.write("proposal.toml", f.policy(2, 1, [false, true]).as_bytes());
    assert!(
        f.run(
            "apply",
            Some(f.root.join("proposal.toml").to_str().unwrap())
        )
        .status
        .success()
    );
    assert!(f.run("reset", Some("2")).status.success());
    assert_eq!(
        f.summary(),
        b"policy generation=3 reset=2 available=1 bindings=0 history=2\n"
    );
    let before = f.snapshot();
    assert!(!f.run("reset", Some("2")).status.success());
    assert!(
        !f.run(
            "apply",
            Some(f.root.join("proposal.toml").to_str().unwrap())
        )
        .status
        .success()
    );
    assert_eq!(before, f.snapshot());
    // Retired leaf digests cannot change exact principal after offline reset
    // and command exit. The rejected proposal leaves retained history intact.
    let remapped = f
        .policy(4, 2, [true, true])
        .replace("member1@home.test", "remapped@home.test");
    f.write("proposal.toml", remapped.as_bytes());
    assert!(
        !f.run(
            "apply",
            Some(f.root.join("proposal.toml").to_str().unwrap())
        )
        .status
        .success()
    );
    assert_eq!(before, f.snapshot());
    fs::remove_file(f.root.join("server.key")).unwrap();
    assert_eq!(
        f.summary(),
        b"policy generation=3 reset=2 available=1 bindings=0 history=2\n"
    );
}
#[test]
fn strict_configuration_and_proposal_refuse_before_initialization() {
    let f = Fixture::new();
    for text in [
        f.config.replace("version=1", "version=2"),
        format!("{}unknown=true\n", f.config),
        format!("{}version=1\n", f.config),
        f.config.replace("127.0.0.1:0", "192.0.2.1:9"),
        f.config.replace("127.0.0.1:0", "localhost:9"),
        f.config.replace("/blobs\"", "/policy\""),
        "x".repeat(8193),
    ] {
        f.write("config.toml", text.as_bytes());
        assert!(
            !f.run("init", Some(f.root.join("proposal.toml").to_str().unwrap()))
                .status
                .success()
        );
        assert!(!f.root.join("policy/provider-policy.snapshot").exists());
    }
    f.write("config.toml", f.config.as_bytes());
    for text in [
        f.proposal.replace("version=1", "version=2"),
        format!(
            "{}\nbindings=42\n",
            f.proposal.split("\n[[bindings]]").next().unwrap()
        ),
        f.proposal
            .replace("account_enabled=true", "account_enabled=true\nunknown=true"),
        f.proposal.replace("home.test", "away.test"),
        f.proposal.replace("device=1", "device=0"),
        format!("{}\n[[bindings]]\n", f.proposal),
        "x".repeat(65537),
    ] {
        f.write("proposal.toml", text.as_bytes());
        assert!(
            !f.run("init", Some(f.root.join("proposal.toml").to_str().unwrap()))
                .status
                .success()
        );
        assert!(!f.root.join("policy/provider-policy.snapshot").exists());
    }
}
#[test]
fn private_inputs_refuse_symlinks_links_types_and_modes() {
    for which in 0..7 {
        let f = Fixture::new();
        match which {
            0 => fs::set_permissions(
                f.root.join("config.toml"),
                fs::Permissions::from_mode(0o644),
            )
            .unwrap(),
            1 => {
                fs::rename(f.root.join("config.toml"), f.root.join("original")).unwrap();
                symlink("original", f.root.join("config.toml")).unwrap();
            }
            2 => fs::hard_link(f.root.join("proposal.toml"), f.root.join("alias")).unwrap(),
            3 => fs::set_permissions(&f.root, fs::Permissions::from_mode(0o755)).unwrap(),
            4 => {
                fs::remove_file(f.root.join("proposal.toml")).unwrap();
                fs::create_dir(f.root.join("proposal.toml")).unwrap();
            }
            5 => {
                fs::rename(f.root.join("policy"), f.root.join("real-policy")).unwrap();
                symlink("real-policy", f.root.join("policy")).unwrap();
            }
            _ => fs::set_permissions(f.root.join("blobs"), fs::Permissions::from_mode(0o755))
                .unwrap(),
        };
        assert!(
            !f.run("init", Some(f.root.join("proposal.toml").to_str().unwrap()))
                .status
                .success()
        );
    }
}
#[test]
fn credentials_and_service_are_validated_before_binding() {
    let f = Fixture::new();
    f.init();
    for text in [
        f.config.replace("server.pem", "wrong-name.pem"),
        f.config.replace("server.pem", "expired.pem"),
        f.config.replace("ca.pem", "foreign-ca.pem"),
        f.config.replace("server.key", "ca.pem"),
        f.config.replace("home.test", "away.test"),
    ] {
        f.write("config.toml", text.as_bytes());
        let o = f.run("serve", None);
        assert!(!o.status.success());
        assert!(o.stdout.is_empty());
        assert_eq!(o.stderr, b"private provider command refused\n");
    }
    f.write("config.toml", f.config.as_bytes());
    for bytes in [vec![b'x'; 65537], Vec::new()] {
        f.write("server.key", &bytes);
        assert!(!f.run("serve", None).status.success());
    }
}
#[test]
fn actual_service_lock_idle_handshake_and_request_stop_reopen() {
    let f = Fixture::new();
    f.init();
    for signal in [rustix::process::Signal::TERM, rustix::process::Signal::INT] {
        for stage in 0..3 {
            let server = f.start();
            let snapshot = f.snapshot();
            for action in ["inspect", "reset", "apply"] {
                let mut cmd = f.command(action);
                if action == "reset" {
                    cmd.arg("2");
                }
                if action == "apply" {
                    cmd.arg(f.root.join("proposal.toml"));
                }
                assert!(!bounded(cmd).status.success());
            }
            assert_eq!(snapshot, f.snapshot());
            let mut raw = None;
            let mut client = None;
            if stage == 1 {
                raw = Some(TcpStream::connect(("127.0.0.1", server.port)).unwrap());
                thread::sleep(Duration::from_millis(30));
            }
            if stage == 2 {
                let mut c = f.connect(server.port, 0).unwrap();
                let bytes = vec![NEXT.fetch_add(1, Ordering::Relaxed) as u8; 16];
                let binding = Binding {
                    sender: principal(1),
                    recipient: principal(2),
                    id: sha256(&bytes),
                    size: 16,
                    created: now(),
                    expires: now() + 120,
                };
                assert!(matches!(
                    c.request(&Request::Begin(binding)),
                    Ok(Response::Progress { .. })
                ));
                client = Some(c);
            }
            server.stop(signal);
            drop(raw);
            drop(client);
            assert!(f.run("inspect", None).status.success());
        }
    }
}
fn principal(n: u8) -> Principal {
    Principal {
        account: format!("member{n}@home.test"),
        device: n.into(),
        identity: [n; 32],
    }
}
#[test]
fn revoked_policy_and_reset_survive_exit_and_rightful_device_still_serves() {
    let f = Fixture::new();
    f.init();
    f.write("proposal.toml", f.policy(2, 1, [false, true]).as_bytes());
    assert!(
        f.run(
            "apply",
            Some(f.root.join("proposal.toml").to_str().unwrap())
        )
        .status
        .success()
    );
    let server = f.start();
    assert!(
        f.connect(server.port, 0)
            .and_then(|mut c| c.request(&Request::Status { id: [7; 32] }))
            .is_err()
    );
    let mut rightful = f.connect(server.port, 1).unwrap();
    let b = Binding {
        sender: principal(2),
        recipient: principal(2),
        id: sha256(&[5; 16]),
        size: 16,
        created: now(),
        expires: now() + 120,
    };
    assert!(matches!(
        rightful.request(&Request::Begin(b)),
        Ok(Response::Progress { .. })
    ));
    drop(rightful);
    server.stop(rustix::process::Signal::TERM);
    assert!(f.run("reset", Some("2")).status.success());
    let server = f.start();
    for device in 0..2 {
        assert!(
            f.connect(server.port, device)
                .and_then(|mut c| c.request(&Request::Status { id: [7; 32] }))
                .is_err()
        );
    }
    server.stop(rustix::process::Signal::INT);
    let before = f.snapshot();
    fs::write(f.root.join("policy/provider-policy.snapshot"), b"corrupt").unwrap();
    assert!(!f.run("serve", None).status.success());
    assert_eq!(
        fs::read(f.root.join("policy/provider-policy.snapshot")).unwrap(),
        b"corrupt"
    );
    assert!(!before.is_empty());
}

/// A separately selected public certificate and card grant no authority. This
/// command may only check an independently reviewed complete proposal.
#[test]
fn offline_prepare_exact_candidate_needs_separate_application() {
    use faction_provider::preparation::Card;
    use std::os::unix::fs::MetadataExt;
    let f = Fixture::new();
    let cert = f.devices[0].0.to_pem().unwrap();
    f.write("selected-device.pem", &cert);
    let card = Card::new(principal(1), sha256(&f.devices[0].0.to_der().unwrap())).unwrap();
    f.write("card.toml", &card.encode().unwrap());
    // Preparation cannot require TLS private material or retained policy bytes.
    fs::remove_file(f.root.join("server.key")).unwrap();
    fs::remove_file(f.root.join("ca.pem")).unwrap();
    f.write(
        "policy/unrelated-corrupt-state",
        b"must never be read or changed",
    );
    f.write(
        "blobs/unrelated-ciphertext",
        b"must never be read or changed",
    );
    let mut command = f.command("prepare");
    command
        .arg(f.root.join("card.toml"))
        .arg(f.root.join("selected-device.pem"))
        .arg(f.root.join("proposal.toml"))
        .arg(f.root.join("candidate.toml"));
    let result = bounded(command);
    assert!(result.status.success(), "{result:?}");
    assert_eq!(
        result.stdout,
        b"prepared candidate; not applied or approved\n"
    );
    let output = f.root.join("candidate.toml");
    let actual = fs::read_to_string(&output)
        .unwrap()
        .parse::<toml::Table>()
        .unwrap();
    let expected = f.proposal.parse::<toml::Table>().unwrap();
    assert_eq!(actual, expected);
    let m = fs::metadata(&output).unwrap();
    assert_eq!(m.mode() & 0o7777, 0o600);
    assert_eq!(m.nlink(), 1);
    assert_eq!(
        fs::read(f.root.join("policy/unrelated-corrupt-state")).unwrap(),
        b"must never be read or changed"
    );
    assert_eq!(
        fs::read(f.root.join("blobs/unrelated-ciphertext")).unwrap(),
        b"must never be read or changed"
    );
    assert_eq!(fs::read_dir(f.root.join("policy")).unwrap().count(), 1);
    assert_eq!(fs::read_dir(f.root.join("blobs")).unwrap().count(), 1);
}

fn preparation_inputs(f: &Fixture) {
    let card = faction_provider::preparation::Card::new(
        principal(1),
        sha256(&f.devices[0].0.to_der().unwrap()),
    )
    .unwrap();
    f.write("card.toml", &card.encode().unwrap());
    f.write("selected-device.pem", &f.devices[0].0.to_pem().unwrap());
}
fn prepare(f: &Fixture, card: &str, certificate: &str, proposal: &str, output: &str) -> Output {
    let mut c = f.command("prepare");
    for value in [card, certificate, proposal, output] {
        c.arg(f.root.join(value));
    }
    bounded(c)
}
#[test]
fn offline_prepare_rejects_each_card_certificate_and_proposal_mismatch() {
    let f = Fixture::new();
    preparation_inputs(&f);
    let original = fs::read(f.root.join("card.toml")).unwrap();
    let expected = faction_provider::preparation::Card::decode(&original).unwrap();
    for field in 0..4 {
        let mut card = expected.clone();
        match field {
            0 => card.principal.account = "other@home.test".into(),
            1 => card.principal.device = 3,
            2 => card.principal.identity = [3; 32],
            _ => card.certificate_sha256 = [3; 32],
        };
        f.write("card.toml", &card.encode().unwrap());
        assert!(
            !prepare(
                &f,
                "card.toml",
                "selected-device.pem",
                "proposal.toml",
                "candidate.toml"
            )
            .status
            .success()
        );
        assert!(!f.root.join("candidate.toml").exists());
    }
    f.write("card.toml", &original);
    f.write("other-device.pem", &f.devices[1].0.to_pem().unwrap());
    assert!(
        !prepare(
            &f,
            "card.toml",
            "other-device.pem",
            "proposal.toml",
            "candidate.toml"
        )
        .status
        .success()
    );
    let digest = hex(&expected.certificate_sha256);
    for text in [
        f.proposal.replace("member1@home.test", "other@home.test"),
        f.proposal.replace("device=1", "device=3"),
        f.proposal.replace(&hex(&[1; 32]), &hex(&[9; 32])),
        f.proposal.replace(&digest, &hex(&[9; 32])),
        f.proposal.replace("home.test", "away.test"),
        f.proposal.replace("generation=1", "generation=0"),
        f.proposal.replace("valid_from=", "valid_from=-"),
    ] {
        f.write("proposal.toml", text.as_bytes());
        assert!(
            !prepare(
                &f,
                "card.toml",
                "selected-device.pem",
                "proposal.toml",
                "candidate.toml"
            )
            .status
            .success()
        );
        assert!(!f.root.join("candidate.toml").exists());
    }
    assert_eq!(fs::read_dir(f.root.join("policy")).unwrap().count(), 0);
    assert_eq!(fs::read_dir(f.root.join("blobs")).unwrap().count(), 0);
}
#[test]
fn offline_prepare_preserves_complete_thirty_two_binding_false_flags_and_order() {
    use std::os::unix::fs::MetadataExt;
    let f = Fixture::new();
    preparation_inputs(&f);
    let mut table = f
        .policy(7, 9, [false, true])
        .parse::<toml::Table>()
        .unwrap();
    let bindings = table.get_mut("bindings").unwrap().as_array_mut().unwrap();
    bindings[0]["account_enabled"] = false.into();
    bindings[0]["device_enabled"] = false.into();
    let base = bindings[0].clone();
    for n in 3..=32u8 {
        let mut entry = base.clone();
        entry["account"] = format!("member{n}@home.test").into();
        entry["device"] = i64::from(n).into();
        entry["identity"] = hex(&[n; 32]).into();
        entry["certificate_sha256"] = hex(&[n; 32]).into();
        entry["approved"] = (n % 2 == 0).into();
        bindings.push(entry);
    }
    let input = table.to_string();
    f.write("proposal.toml", input.as_bytes());
    assert!(
        prepare(
            &f,
            "card.toml",
            "selected-device.pem",
            "proposal.toml",
            "candidate.toml"
        )
        .status
        .success()
    );
    let output = fs::read(f.root.join("candidate.toml")).unwrap();
    assert!(output.len() <= 65536);
    assert_eq!(
        String::from_utf8(output.clone())
            .unwrap()
            .parse::<toml::Table>()
            .unwrap(),
        table
    );
    assert_eq!(
        fs::metadata(f.root.join("candidate.toml")).unwrap().mode() & 0o7777,
        0o600
    );
    assert_eq!(
        fs::read(f.root.join("proposal.toml")).unwrap(),
        input.as_bytes()
    );
    assert_eq!(fs::read_dir(f.root.join("policy")).unwrap().count(), 0);
    assert_eq!(fs::read_dir(f.root.join("blobs")).unwrap().count(), 0);
    let mut oversized = table;
    let rows = oversized
        .get_mut("bindings")
        .unwrap()
        .as_array_mut()
        .unwrap();
    let mut extra = rows.last().unwrap().clone();
    extra["account"] = "member33@home.test".into();
    extra["device"] = 33i64.into();
    extra["identity"] = hex(&[33; 32]).into();
    extra["certificate_sha256"] = hex(&[33; 32]).into();
    rows.push(extra);
    f.write("proposal.toml", oversized.to_string().as_bytes());
    assert!(
        !prepare(
            &f,
            "card.toml",
            "selected-device.pem",
            "proposal.toml",
            "overflow-candidate.toml"
        )
        .status
        .success()
    );
    assert!(!f.root.join("overflow-candidate.toml").exists());
}
#[test]
fn offline_prepare_hostile_data_refuses_without_output_or_root_mutation() {
    let f = Fixture::new();
    preparation_inputs(&f);
    let card = fs::read_to_string(f.root.join("card.toml")).unwrap();
    for text in [
        format!("{card}\nunknown=true\n"),
        format!("{card}\nversion=1\n"),
        card.replace("version = 1", "version = 2"),
        card.replace(&hex(&[1; 32]), &"AA".repeat(32)),
        "x".repeat(8193),
    ] {
        f.write("card.toml", text.as_bytes());
        assert!(
            !prepare(
                &f,
                "card.toml",
                "selected-device.pem",
                "proposal.toml",
                "candidate.toml"
            )
            .status
            .success()
        );
        assert!(!f.root.join("candidate.toml").exists());
    }
    f.write("card.toml", card.as_bytes());
    let certificate = f.devices[0].0.to_pem().unwrap();
    for bytes in [
        Vec::new(),
        [certificate.clone(), certificate.clone()].concat(),
        [certificate.clone(), b"junk".to_vec()].concat(),
        f.devices[0].1.private_key_to_pem_pkcs8().unwrap(),
        vec![b'x'; 65537],
    ] {
        f.write("selected-device.pem", &bytes);
        assert!(
            !prepare(
                &f,
                "card.toml",
                "selected-device.pem",
                "proposal.toml",
                "candidate.toml"
            )
            .status
            .success()
        );
        assert!(!f.root.join("candidate.toml").exists());
    }
    f.write("selected-device.pem", &certificate);
    for text in [
        format!("{}\ngeneration=1\n", f.proposal),
        f.proposal
            .replace("account_enabled=true", "account_enabled=true\nextra=true"),
        format!("{}\n[[bindings]]\n", f.proposal),
        f.proposal.replace(
            "homes=[\"home.test\"]",
            "homes=[\"home.test\",\"home.test\"]",
        ),
        "x".repeat(65537),
    ] {
        f.write("proposal.toml", text.as_bytes());
        assert!(
            !prepare(
                &f,
                "card.toml",
                "selected-device.pem",
                "proposal.toml",
                "candidate.toml"
            )
            .status
            .success()
        );
        assert!(!f.root.join("candidate.toml").exists());
    }
    assert_eq!(fs::read_dir(f.root.join("policy")).unwrap().count(), 0);
    assert_eq!(fs::read_dir(f.root.join("blobs")).unwrap().count(), 0);
}
#[test]
fn offline_prepare_paths_and_no_replace_leave_all_existing_bytes_intact() {
    for case in 0..14 {
        let f = Fixture::new();
        preparation_inputs(&f);
        let output = "candidate.toml";
        let mut card = "card.toml";
        let mut certificate = "selected-device.pem";
        let mut destination = output;
        match case {
            0 => fs::set_permissions(f.root.join(card), fs::Permissions::from_mode(0o644)).unwrap(),
            1 => {
                symlink(f.root.join(card), f.root.join("card-link")).unwrap();
                card = "card-link";
            }
            2 => fs::hard_link(f.root.join(card), f.root.join("card-hard")).unwrap(),
            3 => fs::set_permissions(f.root.join(certificate), fs::Permissions::from_mode(0o644))
                .unwrap(),
            4 => {
                fs::remove_file(f.root.join(certificate)).unwrap();
                fs::create_dir(f.root.join(certificate)).unwrap();
            }
            5 => f.write(output, b"existing candidate"),
            6 => symlink(f.root.join("proposal.toml"), f.root.join(output)).unwrap(),
            7 => fs::hard_link(f.root.join("proposal.toml"), f.root.join(output)).unwrap(),
            8 => {
                symlink(&f.root, f.root.join("alias")).unwrap();
                destination = "alias/candidate.toml";
            }
            9 => destination = "policy/candidate.toml",
            10 => destination = "blobs/candidate.toml",
            11 => certificate = "server.key",
            12 => destination = "./candidate.toml",
            _ => {
                fs::DirBuilder::new()
                    .mode(0o755)
                    .create(f.root.join("unsafe-output"))
                    .unwrap();
                destination = "unsafe-output/candidate.toml";
            }
        }
        let original = fs::read(f.root.join("proposal.toml")).unwrap();
        assert!(
            !prepare(&f, card, certificate, "proposal.toml", destination)
                .status
                .success(),
            "case {case}"
        );
        assert_eq!(fs::read(f.root.join("proposal.toml")).unwrap(), original);
        if case == 5 {
            assert_eq!(
                fs::read(f.root.join(output)).unwrap(),
                b"existing candidate"
            );
        }
        if !matches!(case, 5..=7) {
            assert!(!f.root.join(output).exists());
        }
        assert_eq!(fs::read_dir(f.root.join("policy")).unwrap().count(), 0);
        assert_eq!(fs::read_dir(f.root.join("blobs")).unwrap().count(), 0);
    }
}
#[test]
fn prepared_candidate_cannot_bypass_separate_init_apply_generation_or_history() {
    let f = Fixture::new();
    preparation_inputs(&f);
    assert!(
        prepare(
            &f,
            "card.toml",
            "selected-device.pem",
            "proposal.toml",
            "candidate.toml"
        )
        .status
        .success()
    );
    assert!(!f.run("inspect", None).status.success());
    assert_eq!(fs::read_dir(f.root.join("policy")).unwrap().count(), 0);
    assert!(
        f.run(
            "init",
            Some(f.root.join("candidate.toml").to_str().unwrap())
        )
        .status
        .success()
    );
    let before = f.snapshot();
    assert!(
        !f.run(
            "apply",
            Some(f.root.join("candidate.toml").to_str().unwrap())
        )
        .status
        .success()
    );
    assert_eq!(f.snapshot(), before);
    assert!(f.run("reset", Some("2")).status.success());
    let after = f.snapshot();
    assert!(
        prepare(
            &f,
            "card.toml",
            "selected-device.pem",
            "proposal.toml",
            "still-stale.toml"
        )
        .status
        .success()
    );
    assert!(
        !f.run(
            "apply",
            Some(f.root.join("still-stale.toml").to_str().unwrap())
        )
        .status
        .success()
    );
    assert_eq!(f.snapshot(), after);
    // Data consistency deliberately does not read retained tombstones. Even a
    // matching self-asserted remap stays subject to separate durable apply.
    let remapped = Principal {
        account: "remapped@home.test".into(),
        ..principal(1)
    };
    let card = faction_provider::preparation::Card::new(
        remapped,
        sha256(&f.devices[0].0.to_der().unwrap()),
    )
    .unwrap();
    f.write("card.toml", &card.encode().unwrap());
    f.write(
        "proposal.toml",
        f.policy(3, 2, [true, true])
            .replace("member1@home.test", "remapped@home.test")
            .as_bytes(),
    );
    assert!(
        prepare(
            &f,
            "card.toml",
            "selected-device.pem",
            "proposal.toml",
            "remapped-candidate.toml"
        )
        .status
        .success()
    );
    assert!(
        !f.run(
            "apply",
            Some(f.root.join("remapped-candidate.toml").to_str().unwrap())
        )
        .status
        .success()
    );
    assert_eq!(f.snapshot(), after);
}
