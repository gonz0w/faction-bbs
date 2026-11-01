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
    certificate_named(key, ca, server, wrong_name, expired, None)
}
fn certificate_named(
    key: &PKey<Private>,
    ca: Option<(&X509, &PKey<Private>)>,
    server: bool,
    wrong_name: bool,
    expired: bool,
    home: Option<&str>,
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
            home.unwrap_or("synthetic-ca")
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
        if server || home.is_some() {
            let san = SubjectAlternativeName::new()
                .dns(if wrong_name {
                    "wrong.test"
                } else {
                    home.unwrap_or("home.test")
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
impl Fixture {
    fn retain(&self, retention: &std::path::Path) -> PathBuf {
        fn save(source: &std::path::Path, target: &std::path::Path) {
            let metadata = fs::symlink_metadata(source).unwrap();
            if metadata.file_type().is_symlink() {
                symlink(fs::read_link(source).unwrap(), target).unwrap();
            } else if metadata.is_dir() {
                fs::DirBuilder::new().mode(0o700).create(target).unwrap();
                for child in fs::read_dir(source).unwrap() {
                    let child = child.unwrap();
                    save(&child.path(), &target.join(child.file_name()));
                }
            } else {
                fs::copy(source, target).unwrap();
                fs::set_permissions(target, fs::Permissions::from_mode(0o600)).unwrap();
            }
        }
        let target = retention.join(self.root.file_name().unwrap());
        assert!(!target.starts_with(&self.root));
        save(&self.root, &target);
        // Capture the final vector after owned provider teardown, never into its
        // live configuration/Store roots or the command output.
        let inputs = target.join("device-client-inputs");
        fs::DirBuilder::new().mode(0o700).create(&inputs).unwrap();
        for (index, (certificate, key)) in self.devices.iter().enumerate() {
            for (name, bytes) in [
                (format!("device-{index}.pem"), certificate.to_pem().unwrap()),
                (
                    format!("device-{index}.key"),
                    key.private_key_to_pem_pkcs8().unwrap(),
                ),
            ] {
                let path = inputs.join(name);
                fs::write(&path, bytes).unwrap();
                fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
            }
        }
        let scope = inputs.join("SCOPE");
        fs::write(
            &scope,
            b"final synthetic device vector; capture-only, not live server files\n",
        )
        .unwrap();
        fs::set_permissions(scope, fs::Permissions::from_mode(0o600)).unwrap();
        target
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert_eq!(
            fs::read(self.root.join("OWNED")).unwrap(),
            b"issue284 maintained operator tests"
        );
        if let Some(retention) = std::env::var_os("FACTION_321_FIXTURE_RETENTION") {
            self.retain(std::path::Path::new(&retention));
        }
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
                fs::set_permissions(
                    f.root.join("unsafe-output"),
                    fs::Permissions::from_mode(0o755),
                )
                .unwrap();
                assert_eq!(
                    fs::metadata(f.root.join("unsafe-output"))
                        .unwrap()
                        .permissions()
                        .mode()
                        & 0o7777,
                    0o755
                );
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

#[cfg(feature = "delegation-verifier")]
fn composed_proposal(f: &Fixture, generation: u64, reset: u64) -> String {
    let mut p = f
        .policy(generation, reset, [true, true])
        .replacen("version=1", "version=2", 1)
        .replace(
            "homes=[\"home.test\"]",
            "homes=[\"home.test\",\"peer.test\"]",
        );
    p = p.replacen(
        "[[bindings]]",
        "local_home=\"home.test\"\nissuer=\"home.test\"\n\n[[bindings]]",
        1,
    );
    let entries = [
        ("home.test", "relay", "relay", 10),
        ("peer.test", "inspector", "introspection", 11),
        ("peer.test", "relay", "resource-client", 12),
        ("home.test", "inspector", "outbound-introspection", 13),
        ("peer.test", "endpoint", "issuer-endpoint", 14),
        ("home.test", "publisher", "outbound-observation", 15),
        ("peer.test", "publisher", "observation", 16),
    ];
    for (home, id, purpose, value) in entries {
        p += &format!(
            "\n[[services]]\nhome={home:?}\nclient_id={id:?}\npurpose={purpose:?}\ndns_san={home:?}\ncertificate_sha256={:?}\nenabled=true\nvalid_from={}\nvalid_until={}\n",
            hex(&[value; 32]),
            now() - 60,
            now() + 3600
        );
    }
    p += "\n[[issuer_routes]]\nsender_home=\"peer.test\"\naudience=\"peer.test/attachments\"\nrelay_client_id=\"relay\"\nintrospection_client_id=\"inspector\"\n";
    p += &format!(
        "\n[[resources]]\nrecipient_home=\"peer.test\"\nissuer=\"peer.test\"\naudience=\"home.test/attachments\"\nrelay_client_id=\"relay\"\noutbound_client_id=\"inspector\"\nendpoint_id=\"endpoint\"\naddress=\"127.0.0.1:49152\"\nhttp_host=\"peer.test\"\nca_sha256={:?}\nremote_generation=1\nremote_reset=1\n",
        hex(&[17; 32])
    );
    p += &format!(
        "\n[[publications]]\nrecipient_home=\"peer.test\"\noutbound_client_id=\"publisher\"\nendpoint_id=\"endpoint\"\naddress=\"127.0.0.1:49152\"\nhttp_host=\"peer.test\"\nca_sha256={:?}\nremote_generation=1\nremote_reset=1\n",
        hex(&[17; 32])
    );
    p
}
#[cfg(feature = "delegation-verifier")]
fn composed_config(f: &Fixture) -> String {
    let mut c = f.config.replacen("version=1", "version=2", 1);
    c += &format!(
        "client_cas=[{:?}]\npublication_clients=[\"publisher\"]\n",
        f.root.join("ca.pem").to_str().unwrap()
    );
    let endpoint = |prefix: &str, home: &str| {
        format!(
            "\n[{prefix}]\naddress=\"127.0.0.1:49152\"\nserver_name={home:?}\nhttp_host={home:?}\nleaf_pin={:?}\nca={:?}\ncertificate={:?}\nkey={:?}\n",
            hex(&[18; 32]),
            f.root.join("ca.pem").to_str().unwrap(),
            f.root.join("server.pem").to_str().unwrap(),
            f.root.join("server.key").to_str().unwrap()
        )
    };
    c += &endpoint("local_relay", "home.test");
    c += "\n[[peers]]\nhome=\"peer.test\"\n";
    for field in ["publication", "introspection", "resource"] {
        c += &endpoint(&format!("peers.{field}"), "peer.test");
    }
    c
}
#[test]
#[cfg(feature = "delegation-verifier")]
fn composed_commands_initialize_complete_policy_and_preserve_legacy_refusal() {
    let f = Fixture::new();
    f.write("config.toml", composed_config(&f).as_bytes());
    f.write("proposal.toml", composed_proposal(&f, 1, 1).as_bytes());
    assert!(
        !f.run("init", Some(f.root.join("proposal.toml").to_str().unwrap()))
            .status
            .success()
    );
    assert_eq!(fs::read_dir(f.root.join("policy")).unwrap().count(), 0);
    let output = f.run(
        "init-composed",
        Some(f.root.join("proposal.toml").to_str().unwrap()),
    );
    assert!(
        output.status.success(),
        "explicit complete composed initialization: {output:?}"
    );
    assert_eq!(output.stdout, b"initialized\n");
    assert!(f.snapshot().len() > 100);
    let before = f.snapshot();
    f.write("config.toml", f.config.as_bytes());
    assert!(!f.run("serve", None).status.success());
    assert_eq!(f.snapshot(), before);
    assert!(f.run("inspect", None).status.success());
    f.write("config.toml", composed_config(&f).as_bytes());
    assert!(
        !f.run(
            "apply-composed",
            Some(f.root.join("proposal.toml").to_str().unwrap())
        )
        .status
        .success()
    );
    assert_eq!(f.snapshot(), before);
    f.write("proposal.toml", composed_proposal(&f, 2, 1).as_bytes());
    assert!(
        f.run(
            "apply-composed",
            Some(f.root.join("proposal.toml").to_str().unwrap())
        )
        .status
        .success()
    );
    assert_ne!(f.snapshot(), before);
    assert_eq!(
        f.run("inspect-composed", None).stdout,
        b"policy generation=2 reset=1 available=1 bindings=9 history=9\n"
    );
    assert!(f.run("reset-composed", Some("2")).status.success());
    assert_eq!(
        f.run("inspect-composed", None).stdout,
        b"policy generation=3 reset=2 available=1 bindings=0 history=9\n"
    );
    let disabled = f.snapshot();
    assert!(!f.run("serve-composed", None).status.success());
    assert!(
        !f.run(
            "apply-composed",
            Some(f.root.join("proposal.toml").to_str().unwrap())
        )
        .status
        .success()
    );
    assert_eq!(f.snapshot(), disabled);
    f.write("proposal.toml", composed_proposal(&f, 4, 2).as_bytes());
    assert!(
        f.run(
            "apply-composed",
            Some(f.root.join("proposal.toml").to_str().unwrap())
        )
        .status
        .success()
    );
    assert_eq!(
        f.run("inspect-composed", None).stdout,
        b"policy generation=4 reset=2 available=1 bindings=9 history=9\n"
    );
}
#[cfg(feature = "delegation-verifier")]
mod composed_relay {
    use super::*;
    use faction_provider::{
        delivery::{CarrierEndpoint, Collection},
        tls::ClientCredentials,
    };
    #[test]
    fn composed_fixed_routes_refuse_changed_credentials_or_endpoints_before_listen() {
        for case in 0..4 {
            let a = Home::new("home.test");
            let b = Home::new("peer.test");
            a.configure(&b);
            let before = a.f.snapshot();
            let mut config = a.config(&b).parse::<toml::Table>().unwrap();
            let peers = config.get_mut("peers").unwrap().as_array_mut().unwrap();
            let peer = peers[0].as_table_mut().unwrap();
            let publication = peer.get_mut("publication").unwrap().as_table_mut().unwrap();
            match case {
                0 => {
                    publication.insert("leaf_pin".into(), toml::Value::String(hex(&[91; 32])));
                }
                1 => {
                    publication.insert(
                        "ca".into(),
                        toml::Value::String(a.f.root.join("ca.pem").to_str().unwrap().into()),
                    );
                }
                2 => {
                    publication.insert("address".into(), toml::Value::String("127.0.0.1:9".into()));
                }
                _ => {
                    publication.insert(
                        "key".into(),
                        toml::Value::String(a.f.root.join("relay.key").to_str().unwrap().into()),
                    );
                }
            }
            a.f.write("config.toml", toml::to_string(&config).unwrap().as_bytes());
            let refused = a.f.run("serve-composed", None);
            assert!(!refused.status.success(), "changed fixed route case {case}");
            assert!(refused.stdout.is_empty());
            assert_eq!(refused.stderr, b"private provider command refused\n");
            assert_eq!(a.f.snapshot(), before);
        }
    }
    struct Home {
        f: Fixture,
        name: &'static str,
        address: SocketAddr,
        relay: (X509, PKey<Private>),
        inspector: (X509, PKey<Private>),
        publisher: (X509, PKey<Private>),
    }
    impl Home {
        fn new(name: &'static str) -> Self {
            let mut f = Fixture::new();
            let ca_key = key();
            f.ca = certificate_named(&ca_key, None, false, false, false, Some(name));
            let leaf = |server| {
                let k = key();
                let c =
                    certificate_named(&k, Some((&f.ca, &ca_key)), server, false, false, Some(name));
                (c, k)
            };
            let (server, server_key) = leaf(true);
            let relay = leaf(false);
            let inspector = leaf(false);
            let publisher = leaf(false);
            let device = leaf(false);
            f.server = server;
            f.devices = vec![device];
            f.write("ca.pem", &f.ca.to_pem().unwrap());
            f.write("server.pem", &f.server.to_pem().unwrap());
            f.write(
                "server.key",
                &server_key.private_key_to_pem_pkcs8().unwrap(),
            );
            for (id, (c, k)) in [
                ("relay", &relay),
                ("inspector", &inspector),
                ("publisher", &publisher),
            ] {
                f.write(&format!("{id}.pem"), &c.to_pem().unwrap());
                f.write(&format!("{id}.key"), &k.private_key_to_pem_pkcs8().unwrap());
            }
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            drop(listener);
            Self {
                f,
                name,
                address,
                relay,
                inspector,
                publisher,
            }
        }
        fn principal(&self) -> Principal {
            Principal {
                account: format!("member@{}", self.name),
                device: 1,
                identity: if self.name == "home.test" {
                    [1; 32]
                } else {
                    [2; 32]
                },
            }
        }
        fn endpoint(&self, section: &str, remote: &Home, credential: &str) -> String {
            format!(
                "\n[{section}]\naddress={:?}\nhttp_host={:?}\nserver_name={:?}\nleaf_pin={:?}\nca={:?}\ncertificate={:?}\nkey={:?}\n",
                remote.address.to_string(),
                remote.name,
                remote.name,
                hex(&sha256(&remote.f.server.to_der().unwrap())),
                self.f
                    .root
                    .join(if remote.name == self.name {
                        "ca.pem"
                    } else {
                        "peer-ca.pem"
                    })
                    .to_str()
                    .unwrap(),
                self.f
                    .root
                    .join(format!("{credential}.pem"))
                    .to_str()
                    .unwrap(),
                self.f
                    .root
                    .join(format!("{credential}.key"))
                    .to_str()
                    .unwrap()
            )
        }
        fn config(&self, peer: &Home) -> String {
            let mut c = self
                .f
                .config
                .replacen("version=1", "version=2", 1)
                .replace("home.test", self.name)
                .replace("127.0.0.1:0", &self.address.to_string());
            c += &format!(
                "client_cas=[{:?},{:?}]\npublication_clients=[\"publisher\"]\n",
                self.f.root.join("ca.pem").to_str().unwrap(),
                self.f.root.join("peer-ca.pem").to_str().unwrap()
            );
            c += &self.endpoint("local_relay", self, "relay");
            c += &format!("\n[[peers]]\nhome={:?}\n", peer.name);
            c += &self.endpoint("peers.publication", peer, "publisher");
            c += &self.endpoint("peers.introspection", peer, "inspector");
            c += &self.endpoint("peers.resource", peer, "relay");
            c
        }
        fn proposal(&self, peer: &Home, generation: u64, reset: u64) -> String {
            let p = self.principal();
            let mut t = format!(
                "version=2\ngeneration={generation}\nreset_epoch={reset}\nhomes=[{:?},{:?}]\nlocal_home={:?}\nissuer={:?}\n\n[[bindings]]\ncertificate_sha256={:?}\naccount={:?}\ndevice=1\nidentity={:?}\nvalid_from={}\nvalid_until={}\napproved=true\naccount_enabled=true\ndevice_enabled=true\ngeneration={generation}\n",
                self.name,
                peer.name,
                self.name,
                self.name,
                hex(&sha256(&self.f.devices[0].0.to_der().unwrap())),
                p.account,
                hex(&p.identity),
                now() - 60,
                now() + 3600
            );
            for (home, id, purpose, cert) in [
                (self.name, "relay", "relay", &self.relay.0),
                (peer.name, "inspector", "introspection", &peer.inspector.0),
                (peer.name, "relay", "resource-client", &peer.relay.0),
                (
                    self.name,
                    "inspector",
                    "outbound-introspection",
                    &self.inspector.0,
                ),
                (peer.name, "endpoint", "issuer-endpoint", &peer.f.server),
                (
                    self.name,
                    "publisher",
                    "outbound-observation",
                    &self.publisher.0,
                ),
                (peer.name, "publisher", "observation", &peer.publisher.0),
            ] {
                t += &format!(
                    "\n[[services]]\nhome={home:?}\nclient_id={id:?}\npurpose={purpose:?}\ndns_san={home:?}\ncertificate_sha256={:?}\nenabled=true\nvalid_from={}\nvalid_until={}\n",
                    hex(&sha256(&cert.to_der().unwrap())),
                    now() - 60,
                    now() + 3600
                );
            }
            t += &format!(
                "\n[[issuer_routes]]\nsender_home={:?}\naudience={:?}\nrelay_client_id=\"relay\"\nintrospection_client_id=\"inspector\"\n",
                peer.name,
                format!("{}/attachments", peer.name)
            );
            t += &format!(
                "\n[[resources]]\nrecipient_home={:?}\nissuer={:?}\naudience={:?}\nrelay_client_id=\"relay\"\noutbound_client_id=\"inspector\"\nendpoint_id=\"endpoint\"\naddress={:?}\nhttp_host={:?}\nca_sha256={:?}\nremote_generation=1\nremote_reset=1\n",
                peer.name,
                peer.name,
                format!("{}/attachments", self.name),
                peer.address.to_string(),
                peer.name,
                hex(&sha256(&peer.f.ca.to_der().unwrap()))
            );
            t += &format!(
                "\n[[publications]]\nrecipient_home={:?}\noutbound_client_id=\"publisher\"\nendpoint_id=\"endpoint\"\naddress={:?}\nhttp_host={:?}\nca_sha256={:?}\nremote_generation=1\nremote_reset=1\n",
                peer.name,
                peer.address.to_string(),
                peer.name,
                hex(&sha256(&peer.f.ca.to_der().unwrap()))
            );
            t
        }
        fn configure(&self, peer: &Home) {
            self.f.write("peer-ca.pem", &peer.f.ca.to_pem().unwrap());
            self.f.write("config.toml", self.config(peer).as_bytes());
            self.f
                .write("proposal.toml", self.proposal(peer, 1, 1).as_bytes());
            let output = self.f.run(
                "init-composed",
                Some(self.f.root.join("proposal.toml").to_str().unwrap()),
            );
            assert!(output.status.success(), "composed init: {output:?}");
        }
        fn endpoint_client(&self, who: &(X509, PKey<Private>)) -> CarrierEndpoint {
            CarrierEndpoint::new(
                self.address,
                self.name.into(),
                sha256(&self.f.server.to_der().unwrap()),
                ClientCredentials::from_pem(
                    &self.f.ca.to_pem().unwrap(),
                    &who.0.to_pem().unwrap(),
                    &who.1.private_key_to_pem_pkcs8().unwrap(),
                )
                .unwrap(),
            )
            .unwrap()
        }
        fn start(&self) -> Serving {
            Serving::start(self.f.command("serve-composed"))
        }
    }
    fn collect(
        operation: &mut faction_provider::delivery::Operation,
        action: &Request,
    ) -> Response {
        // Explicit finite polling consumes the original eight operation attempts.
        for _ in 0..6 {
            match operation
                .collect_paced()
                .unwrap_or_else(|error| panic!("collect {action:?}: {error:?}"))
            {
                Collection::Complete(response) => return response,
                Collection::Busy => thread::sleep(Duration::from_millis(100)),
            }
        }
        panic!("bounded collection did not complete");
    }
    fn publication(home: &Home, binding: &Binding, request: Request) -> Response {
        let mut operation = home
            .endpoint_client(&home.f.devices[0])
            .operation(Arc::new(AtomicBool::new(false)));
        operation
            .submit_publication(binding.clone(), request.clone())
            .unwrap();
        collect(&mut operation, &request)
    }
    fn retrieval(home: &Home, binding: &Binding, request: Request) -> Response {
        let mut operation = home
            .endpoint_client(&home.f.devices[0])
            .operation(Arc::new(AtomicBool::new(false)));
        operation
            .submit_own_home(binding.clone(), request.clone())
            .unwrap();
        collect(&mut operation, &request)
    }
    #[test]
    fn independent_operator_homes_publish_and_retrieve_exact_ciphertext_both_directions() {
        let a = Home::new("home.test");
        let b = Home::new("peer.test");
        a.configure(&b);
        b.configure(&a);
        let sa = a.start();
        let sb = b.start();
        let denied = Binding {
            sender: a.principal(),
            recipient: b.principal(),
            id: [99; 32],
            size: 32,
            created: now(),
            expires: now() + 3600,
        };
        // Registered home service credentials cannot become a local device actor.
        let mut wrong_purpose = a
            .endpoint_client(&a.relay)
            .operation(Arc::new(AtomicBool::new(false)));
        assert!(
            wrong_purpose
                .submit_publication(denied.clone(), Request::Begin(denied.clone()))
                .is_err()
        );
        // A real peer-device certificate chains to an approved CA but has no
        // local device binding at this home.
        let mut foreign_device = a
            .endpoint_client(&b.f.devices[0])
            .operation(Arc::new(AtomicBool::new(false)));
        assert!(
            foreign_device
                .submit_publication(denied.clone(), Request::Begin(denied.clone()))
                .is_err()
        );
        for (sender, recipient, byte) in [(&a, &b, 61), (&b, &a, 62)] {
            let binding = Binding {
                sender: sender.principal(),
                recipient: recipient.principal(),
                id: sha256(&[byte; 32]),
                size: 32,
                created: now(),
                expires: now() + 3600,
            };
            let progress = |offset, committed| Response::Progress {
                binding: binding.clone(),
                offset,
                committed,
            };
            assert_eq!(
                publication(sender, &binding, Request::Begin(binding.clone())),
                progress(0, false)
            );
            assert_eq!(
                publication(
                    sender,
                    &binding,
                    Request::PutChunk {
                        id: binding.id,
                        offset: 0,
                        bytes: vec![byte; 32]
                    }
                ),
                progress(32, false)
            );
            assert_eq!(
                publication(sender, &binding, Request::Commit { id: binding.id }),
                progress(32, true)
            );
            // A new Operation reconnects and reauthorizes this exact retry.
            assert_eq!(
                publication(sender, &binding, Request::Commit { id: binding.id }),
                progress(32, true)
            );
            assert_eq!(
                retrieval(recipient, &binding, Request::Status { id: binding.id }),
                progress(32, true)
            );
            assert_eq!(
                retrieval(
                    recipient,
                    &binding,
                    Request::GetChunk {
                        id: binding.id,
                        offset: 0
                    }
                ),
                Response::Chunk {
                    offset: 0,
                    bytes: vec![byte; 32],
                    total: 32
                }
            );
            // Exact ciphertext is unchanged through another fresh own-home cycle.
            assert_eq!(
                retrieval(
                    recipient,
                    &binding,
                    Request::GetChunk {
                        id: binding.id,
                        offset: 0
                    }
                ),
                Response::Chunk {
                    offset: 0,
                    bytes: vec![byte; 32],
                    total: 32
                }
            );
            assert_eq!(
                retrieval(
                    recipient,
                    &binding,
                    Request::GetChunk {
                        id: binding.id,
                        offset: 32
                    }
                ),
                Response::Chunk {
                    offset: 32,
                    bytes: vec![],
                    total: 32
                }
            );
            // Test observer: retain the actual committed bytes outside the Store
            // root before the successful sender Delete removes that object.
            let committed = fs::read(sender.f.root.join("blobs/private-blobs.snapshot")).unwrap();
            assert!(committed.windows(32).any(|w| w.iter().all(|b| *b == byte)));
            sender.f.write("committed-store.snapshot", &committed);
            assert_eq!(
                publication(sender, &binding, Request::Delete { id: binding.id }),
                Response::Deleted
            );
        }
        sa.stop(rustix::process::Signal::TERM);
        sb.stop(rustix::process::Signal::TERM);
    }
}

#[test]
#[cfg(feature = "delegation-verifier")]
fn composed_proposal_refusals_leave_an_empty_policy_root() {
    for case in 0..8 {
        let f = Fixture::new();
        f.write("config.toml", composed_config(&f).as_bytes());
        let good = composed_proposal(&f, 1, 1);
        let bad = match case {
            0 => good.replacen("version=2", "version=3", 1),
            1 => good.replace("local_home=\"home.test\"", "local_home=\"other.test\""),
            2 => good.replace("purpose=\"relay\"", "purpose=\"device\""),
            3 => good.replace("remote_generation=1", "remote_generation=0"),
            4 => good.replace("member1@home.test", "member1@peer.test"),
            5 => good.replacen("remote_reset=1", "remote_reset=2", 1),
            6 => good.replace("address=\"127.0.0.1:49152\"", "address=\"192.0.2.1:49152\""),
            _ => format!("unexpected=true\n{good}"),
        };
        f.write("proposal.toml", bad.as_bytes());
        let output = f.run(
            "init-composed",
            Some(f.root.join("proposal.toml").to_str().unwrap()),
        );
        assert!(
            !output.status.success(),
            "invalid complete proposal case {case}"
        );
        assert_eq!(output.stderr, b"private provider command refused\n");
        assert_eq!(fs::read_dir(f.root.join("policy")).unwrap().count(), 0);
        assert_eq!(fs::read_dir(f.root.join("blobs")).unwrap().count(), 0);
    }
}
#[test]
#[cfg(feature = "delegation-verifier")]
fn composed_inspect_and_reset_use_explicit_v2_config_without_a_legacy_workaround() {
    let f = Fixture::new();
    f.init();
    f.write("config.toml", composed_config(&f).as_bytes());
    assert!(!f.run("inspect", None).status.success());
    assert!(!f.run("reset", Some("2")).status.success());
    let inspect = f.run("inspect-composed", None);
    assert!(inspect.status.success(), "composed inspection: {inspect:?}");
    assert_eq!(
        inspect.stdout,
        b"policy generation=1 reset=1 available=1 bindings=2 history=2\n"
    );
    let reset = f.run("reset-composed", Some("2"));
    assert!(reset.status.success(), "composed reset: {reset:?}");
    assert_eq!(reset.stdout, b"reset\n");
    let inspect = f.run("inspect-composed", None);
    assert!(inspect.status.success());
    assert_eq!(
        inspect.stdout,
        b"policy generation=2 reset=2 available=1 bindings=0 history=2\n"
    );
    let after = f.snapshot();
    assert!(!f.run("reset-composed", Some("2")).status.success());
    assert_eq!(f.snapshot(), after);
}
#[test]
#[cfg(feature = "delegation-verifier")]
fn composed_configuration_accepts_exact_bound_and_refuses_an_extra_byte() {
    let f = Fixture::new();
    let mut config = composed_config(&f);
    assert!(config.len() + 2 < 65_536);
    config.push('#');
    config.push_str(&"x".repeat(65_536 - config.len() - 1));
    config.push('\n');
    assert_eq!(config.len(), 65_536);
    f.write("proposal.toml", composed_proposal(&f, 1, 1).as_bytes());
    f.write("config.toml", format!("{config}\n").as_bytes());
    let oversized = f.run(
        "init-composed",
        Some(f.root.join("proposal.toml").to_str().unwrap()),
    );
    assert!(!oversized.status.success());
    assert_eq!(oversized.stderr, b"private provider command refused\n");
    assert_eq!(fs::read_dir(f.root.join("policy")).unwrap().count(), 0);
    f.write("config.toml", config.as_bytes());
    let exact = f.run(
        "init-composed",
        Some(f.root.join("proposal.toml").to_str().unwrap()),
    );
    assert!(exact.status.success(), "exact v2 config bound: {exact:?}");
    assert_eq!(exact.stdout, b"initialized\n");
}

#[test]
fn explicit_fixture_retention_characterizes_disk_copy() {
    let f = Fixture::new();
    let name = f.root.file_name().unwrap().to_owned();
    let live = f.root.clone();
    let expected = ["config.toml", "proposal.toml", "server.key"]
        .map(|name| (name, fs::read(f.root.join(name)).unwrap()));
    drop(f);
    if let Some(retention) = std::env::var_os("FACTION_321_FIXTURE_RETENTION") {
        let saved = PathBuf::from(retention).join(name);
        for (name, bytes) in expected {
            assert!(
                fs::read(saved.join(name)).unwrap() == bytes,
                "retained disk fixture bytes differ"
            );
            assert_eq!(
                fs::metadata(saved.join(name)).unwrap().permissions().mode() & 0o7777,
                0o600
            );
        }
        assert_eq!(
            fs::metadata(saved).unwrap().permissions().mode() & 0o7777,
            0o700
        );
    }
    assert!(!live.exists());
}

#[test]
fn fixture_retention_preserves_final_device_vector_privately() {
    let sink = Fixture::new();
    let capture = sink.root.join("capture");
    fs::DirBuilder::new().mode(0o700).create(&capture).unwrap();
    let mut f = Fixture::new();
    f.devices.remove(0);
    assert_eq!(f.devices.len(), 1);
    let expected_cert = f.devices[0].0.to_pem().unwrap();
    let expected_key = f.devices[0].1.private_key_to_pem_pkcs8().unwrap();
    // Independent expected input bytes stay outside the provider fixture root.
    sink.write("expected-device.pem", &expected_cert);
    sink.write("expected-device.key", &expected_key);
    let saved = f.retain(&capture);
    assert!(!saved.starts_with(&f.root));
    let inputs = saved.join("device-client-inputs");
    assert!(
        inputs.is_dir(),
        "final in-memory device inputs were not retained"
    );
    let certificate = fs::read(inputs.join("device-0.pem")).unwrap();
    let private_key = fs::read(inputs.join("device-0.key")).unwrap();
    assert!(
        certificate == expected_cert,
        "retained certificate bytes differ"
    );
    assert!(
        private_key == expected_key,
        "retained private-key bytes differ"
    );
    let certificate = X509::from_pem(&certificate).unwrap();
    let private_key = PKey::private_key_from_pem(&private_key).unwrap();
    assert!(private_key.public_eq(&certificate.public_key().unwrap()));
    assert_eq!(
        fs::metadata(&inputs).unwrap().permissions().mode() & 0o7777,
        0o700
    );
    for name in ["device-0.pem", "device-0.key", "SCOPE"] {
        assert_eq!(
            fs::metadata(inputs.join(name))
                .unwrap()
                .permissions()
                .mode()
                & 0o7777,
            0o600
        );
    }
    assert_eq!(
        fs::read(inputs.join("SCOPE")).unwrap(),
        b"final synthetic device vector; capture-only, not live server files\n"
    );
    assert!(!inputs.join("device-1.pem").exists());
    assert!(!f.root.join("device-client-inputs").exists());
}
