#![cfg(feature = "device-provider")]
//! Actual offline command/PTY controls using existing encrypted checkpoints.
//! Execute a separately materialized client via FACTION_SETUP_CLIENT_EXE.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use faction_client::{
    mail_checkpoint::CheckpointVault,
    mail_data::{Data, hex},
    mail_vault::Store,
};
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    termios,
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
const PASS: &str = "synthetic offline preparation password";
const PROMPT: &[u8] = b"LOCAL checkpoint passphrase (hidden): ";
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
fn private(path: &Path, bytes: &[u8]) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
}
struct Fixture {
    root: PathBuf,
    capture: PathBuf,
    identity: String,
}
impl Fixture {
    fn new(created: u64, history: bool) -> Self {
        let capture = PathBuf::from(
            std::env::var_os("FACTION_SETUP_CAPTURE_DIR")
                .expect("explicit private capture directory required"),
        );
        let m = fs::symlink_metadata(&capture).unwrap();
        assert!(
            m.is_dir()
                && m.uid() == rustix::process::geteuid().as_raw()
                && m.mode() & 0o7777 == 0o700
        );
        let mut random = [0; 16];
        getrandom::fill(&mut random).unwrap();
        let root = capture.join(format!("faction286-OWNED-{}", hex(&random)));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        private(&root.join("OWNED"), b"issue286 offline setup acceptance");
        let v = CheckpointVault::create(
            Store::at(&root.join("vault"), true).unwrap(),
            Data::new("alice@home.test".into()).unwrap(),
            PASS.as_bytes(),
            101,
            created,
        )
        .unwrap()
        .0;
        let identity = hex(v.inspection().unwrap().identity);
        if history {
            v.backup(&root.join("history")).unwrap();
        }
        drop(v);
        Self {
            root,
            capture,
            identity,
        }
    }
    fn credentials(&self) {
        let openssl = if cfg!(target_os = "macos") {
            "/opt/homebrew/opt/openssl@3/bin/openssl"
        } else {
            "openssl"
        };
        let mut child = Command::new(openssl)
            .args([
                "req",
                "-x509",
                "-newkey",
                "ec",
                "-pkeyopt",
                "ec_paramgen_curve:P-256",
                "-nodes",
                "-days",
                "2",
                "-subj",
                "/CN=synthetic-device",
                "-keyout",
            ])
            .arg(self.root.join("key.pem"))
            .arg("-out")
            .arg(self.root.join("cert.pem"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let start = Instant::now();
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            if start.elapsed() > Duration::from_secs(10) {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("bounded synthetic certificate fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        for name in ["key.pem", "cert.pem"] {
            fs::set_permissions(self.root.join(name), fs::Permissions::from_mode(0o600)).unwrap();
        }
        private(&self.root.join("template.toml"), self.template().as_bytes());
        private(&self.root.join("selected.toml"), self.profile().as_bytes());
    }
    fn template(&self) -> String {
        format!(
            "version = 1\nhome = \"home.test\"\nendpoint = \"127.0.0.1:9\"\nname = \"home.test\"\npin = \"{}\"\nca = \"{}\"\ncertificate = \"{}\"\nkey = \"{}\"\n",
            "01".repeat(32),
            self.root.join("cert.pem").display(),
            self.root.join("cert.pem").display(),
            self.root.join("key.pem").display()
        )
    }
    fn profile(&self) -> String {
        format!(
            "{}account = \"alice@home.test\"\ndevice = 101\nidentity = \"{}\"\n",
            self.template(),
            self.identity
        )
    }
    fn snapshot(&self, name: &str) -> Vec<(String, Vec<u8>)> {
        let mut files = fs::read_dir(self.root.join(name))
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                assert!(
                    entry.file_type().unwrap().is_file(),
                    "unexpected spool or directory creation"
                );
                (
                    entry.file_name().into_string().unwrap(),
                    fs::read(entry.path()).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        files.sort();
        files
    }
    fn command(
        &self,
        action: &str,
        vault: &str,
        input: &str,
        output: &str,
        secret: Option<&[u8]>,
        unicode: bool,
    ) -> (bool, Vec<u8>) {
        let binary = PathBuf::from(
            std::env::var_os("FACTION_SETUP_CLIENT_EXE")
                .expect("stage-unique retained client executable required"),
        );
        assert!(binary.is_absolute());
        let (master, slave) = faction_client_pty::open(25, 80).unwrap();
        let mut master = File::from(master);
        let slave = File::from(slave);
        let original = termios::tcgetattr(&slave).unwrap();
        let mut cmd = Command::new(binary);
        cmd.arg(action)
            .arg(self.root.join(vault))
            .arg(self.root.join(input))
            .arg(self.root.join(output));
        if unicode {
            cmd.arg("--unicode");
        }
        let mut child = cmd
            .stdin(Stdio::from(slave.try_clone().unwrap()))
            .stdout(Stdio::from(slave.try_clone().unwrap()))
            .stderr(Stdio::from(slave.try_clone().unwrap()))
            .spawn()
            .unwrap();
        let start = Instant::now();
        let mut transcript = Vec::new();
        let mut sent = false;
        let status = loop {
            if !sent && transcript.windows(PROMPT.len()).any(|v| v == PROMPT) {
                if let Some(secret) = secret {
                    master.write_all(secret).unwrap();
                }
                sent = true;
            }
            let mut fds = [PollFd::new(&master, PollFlags::IN)];
            poll(
                &mut fds,
                Some(&Timespec {
                    tv_sec: 0,
                    tv_nsec: 20_000_000,
                }),
            )
            .unwrap();
            if fds[0].revents().contains(PollFlags::IN) {
                let mut bytes = [0; 4096];
                let count = master.read(&mut bytes).unwrap_or(0);
                transcript.extend_from_slice(&bytes[..count]);
                assert!(transcript.len() <= 16_384, "setup diagnostic bound");
            }
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if start.elapsed() > Duration::from_secs(10) {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("bounded setup command failed to exit");
            }
        };
        let restored = termios::tcgetattr(&slave).unwrap();
        // Match the maintained terminal restoration oracle: Darwin/NetBSD set
        // the kernel's PENDIN queue-state bit when restoring configured modes.
        #[cfg(any(target_os = "macos", target_os = "netbsd"))]
        let queue_state = termios::LocalModes::PENDIN;
        #[cfg(not(any(target_os = "macos", target_os = "netbsd")))]
        let queue_state = termios::LocalModes::empty();
        assert_eq!(
            restored.local_modes & !queue_state,
            original.local_modes & !queue_state,
            "terminal flags were not restored"
        );
        assert_eq!(restored.input_modes, original.input_modes);
        assert_eq!(restored.output_modes, original.output_modes);
        assert_eq!(restored.control_modes, original.control_modes);
        let mut random = [0; 16];
        getrandom::fill(&mut random).unwrap();
        private(
            &self
                .capture
                .join(format!("setup-transcript-{}.private", hex(&random))),
            &transcript,
        );
        assert!(
            sent,
            "missing maintained setup command: no normal hidden unlock prompt"
        );
        assert!(
            !transcript.windows(PASS.len()).any(|v| v == PASS.as_bytes()),
            "hidden passphrase leaked"
        );
        for private in [
            "alice@home.test",
            self.identity.as_str(),
            self.root.to_str().unwrap(),
        ] {
            assert!(
                !transcript
                    .windows(private.len())
                    .any(|v| v == private.as_bytes()),
                "private identifiers or paths appeared in diagnostics"
            );
        }
        (status.success(), transcript)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert_eq!(
            fs::read(self.root.join("OWNED")).unwrap(),
            b"issue286 offline setup acceptance"
        );
        fs::remove_dir_all(&self.root).unwrap();
        assert!(!self.root.exists());
    }
}
#[test]
fn maintained_prepare_command_derives_identity_and_preserves_checkpoint() {
    let f = Fixture::new(now(), false);
    f.credentials();
    let before = f.snapshot("vault");
    let (ok, _) = f.command(
        "--device-provider-prepare",
        "vault",
        "template.toml",
        "prepared.toml",
        Some(format!("{PASS}\n").as_bytes()),
        false,
    );
    assert!(ok);
    let raw = fs::read(f.root.join("prepared.toml")).unwrap();
    assert!(raw.len() <= 8192);
    let fields: toml::Table = std::str::from_utf8(&raw).unwrap().parse().unwrap();
    assert_eq!(fields.len(), 11);
    assert_eq!(fields["account"].as_str(), Some("alice@home.test"));
    assert_eq!(fields["device"].as_integer(), Some(101));
    assert_eq!(fields["identity"].as_str(), Some(f.identity.as_str()));
    assert_eq!(
        fs::metadata(f.root.join("prepared.toml")).unwrap().mode() & 0o7777,
        0o600
    );
    assert_eq!(f.snapshot("vault"), before);
    let original = raw;
    let (ok, _) = f.command(
        "--device-provider-prepare",
        "vault",
        "template.toml",
        "prepared.toml",
        Some(format!("{PASS}\n").as_bytes()),
        true,
    );
    assert!(!ok);
    assert_eq!(fs::read(f.root.join("prepared.toml")).unwrap(), original);
    assert_eq!(f.snapshot("vault"), before);
}
#[test]
fn maintained_card_command_exports_only_public_fields_without_vault_mutation() {
    let f = Fixture::new(now(), false);
    f.credentials();
    let before = f.snapshot("vault");
    let (ok, _) = f.command(
        "--device-provider-card",
        "vault",
        "selected.toml",
        "card.toml",
        Some(format!("{PASS}\n").as_bytes()),
        true,
    );
    assert!(ok);
    let raw = fs::read(f.root.join("card.toml")).unwrap();
    let fields: toml::Table = std::str::from_utf8(&raw).unwrap().parse().unwrap();
    assert_eq!(fields.len(), 5);
    assert_eq!(fields["version"].as_integer(), Some(1));
    assert_eq!(fields["account"].as_str(), Some("alice@home.test"));
    assert_eq!(fields["device"].as_integer(), Some(101));
    assert_eq!(fields["identity"].as_str(), Some(f.identity.as_str()));
    assert_eq!(fields["certificate_sha256"].as_str().unwrap().len(), 64);
    let pem = fs::read_to_string(f.root.join("cert.pem")).unwrap();
    let body = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<String>();
    let der = STANDARD.decode(body).unwrap();
    let expected = hex(&Sha256::digest(&der));
    assert_eq!(
        fields["certificate_sha256"].as_str(),
        Some(expected.as_str())
    );
    for forbidden in [
        PASS,
        "PRIVATE KEY",
        "checkpoint passphrase",
        "127.0.0.1",
        f.root.to_str().unwrap(),
    ] {
        assert!(
            !raw.windows(forbidden.len())
                .any(|v| v == forbidden.as_bytes())
        );
    }
    assert!(raw.len() <= 8192);
    assert_eq!(
        fs::metadata(f.root.join("card.toml")).unwrap().mode() & 0o7777,
        0o600
    );
    assert_eq!(f.snapshot("vault"), before);
}
#[test]
fn wrong_secret_and_cancel_keys_leave_checkpoint_and_outputs_unchanged() {
    let f = Fixture::new(now(), false);
    f.credentials();
    let before = f.snapshot("vault");
    for secret in [b"wrong synthetic password\n".as_slice(), b"\x03", b"\x04"] {
        let (ok, _) = f.command(
            "--device-provider-prepare",
            "vault",
            "template.toml",
            "absent.toml",
            Some(secret),
            false,
        );
        assert!(!ok);
        assert!(!f.root.join("absent.toml").exists());
        assert_eq!(f.snapshot("vault"), before);
    }
}
#[test]
fn expired_history_and_hostile_template_refuse_without_publication() {
    for (created, history) in [(now() - 31 * 24 * 3600, false), (now(), true)] {
        let f = Fixture::new(created, history);
        f.credentials();
        let store = if history { "history" } else { "vault" };
        let before = f.snapshot(store);
        let (ok, _) = f.command(
            "--device-provider-prepare",
            store,
            "template.toml",
            "absent.toml",
            Some(format!("{PASS}\n").as_bytes()),
            false,
        );
        assert!(!ok);
        assert!(!f.root.join("absent.toml").exists());
        assert_eq!(f.snapshot(store), before);
    }
    let f = Fixture::new(now(), false);
    f.credentials();
    let before = f.snapshot("vault");
    for changed in [
        f.template().replace("home.test", "away.test"),
        format!("{}account=\"mallory@home.test\"\n", f.template()),
    ] {
        fs::write(f.root.join("template.toml"), changed).unwrap();
        let (ok, _) = f.command(
            "--device-provider-prepare",
            "vault",
            "template.toml",
            "absent.toml",
            Some(format!("{PASS}\n").as_bytes()),
            false,
        );
        assert!(!ok);
        assert!(!f.root.join("absent.toml").exists());
        assert_eq!(f.snapshot("vault"), before);
    }
    fs::write(f.root.join("template.toml"), f.template()).unwrap();
    symlink(f.root.join("template.toml"), f.root.join("linked.toml")).unwrap();
    let (ok, _) = f.command(
        "--device-provider-prepare",
        "vault",
        "linked.toml",
        "absent.toml",
        Some(format!("{PASS}\n").as_bytes()),
        false,
    );
    assert!(!ok);
    assert!(!f.root.join("absent.toml").exists());
    assert_eq!(f.snapshot("vault"), before);
}
#[test]
fn revoked_device_and_each_profile_binding_mismatch_refuse_card_export() {
    let f = Fixture::new(now(), false);
    f.credentials();
    let before = f.snapshot("vault");
    for changed in [
        f.profile().replace("alice@home.test", "other@home.test"),
        f.profile().replace("device = 101", "device = 202"),
        f.profile().replace(&f.identity, &"bc".repeat(32)),
    ] {
        fs::write(f.root.join("selected.toml"), changed).unwrap();
        let (ok, _) = f.command(
            "--device-provider-card",
            "vault",
            "selected.toml",
            "absent.toml",
            Some(format!("{PASS}\n").as_bytes()),
            false,
        );
        assert!(!ok);
        assert!(!f.root.join("absent.toml").exists());
        assert_eq!(f.snapshot("vault"), before);
    }
    fs::write(f.root.join("selected.toml"), f.profile()).unwrap();
    let mut vault = CheckpointVault::unlock(
        Store::at(&f.root.join("vault"), false).unwrap(),
        PASS.as_bytes(),
        false,
    )
    .unwrap();
    vault
        .enroll(
            "bob@home.test",
            202,
            &format!("ed25519:{}", "ab".repeat(32)),
            now(),
        )
        .unwrap();
    vault.revoke().unwrap();
    drop(vault);
    let revoked = f.snapshot("vault");
    for (action, input) in [
        ("--device-provider-prepare", "template.toml"),
        ("--device-provider-card", "selected.toml"),
    ] {
        let (ok, _) = f.command(
            action,
            "vault",
            input,
            "absent.toml",
            Some(format!("{PASS}\n").as_bytes()),
            true,
        );
        assert!(!ok);
        assert!(!f.root.join("absent.toml").exists());
        assert_eq!(f.snapshot("vault"), revoked);
    }
}
#[test]
fn unsafe_ambiguous_and_mismatched_credentials_cannot_produce_profile() {
    let f = Fixture::new(now(), false);
    f.credentials();
    let before = f.snapshot("vault");
    let cert = fs::read(f.root.join("cert.pem")).unwrap();
    for changed in [
        [cert.clone(), cert.clone()].concat(),
        [cert.clone(), b"trailing untrusted text".to_vec()].concat(),
    ] {
        fs::write(f.root.join("cert.pem"), changed).unwrap();
        let (ok, _) = f.command(
            "--device-provider-prepare",
            "vault",
            "template.toml",
            "absent.toml",
            Some(format!("{PASS}\n").as_bytes()),
            false,
        );
        assert!(!ok);
        assert!(!f.root.join("absent.toml").exists());
        assert_eq!(f.snapshot("vault"), before);
    }
    fs::write(f.root.join("cert.pem"), &cert).unwrap();
    let other = Fixture::new(now(), false);
    other.credentials();
    let key = fs::read(f.root.join("key.pem")).unwrap();
    fs::write(
        f.root.join("key.pem"),
        fs::read(other.root.join("key.pem")).unwrap(),
    )
    .unwrap();
    let (ok, _) = f.command(
        "--device-provider-prepare",
        "vault",
        "template.toml",
        "absent.toml",
        Some(format!("{PASS}\n").as_bytes()),
        false,
    );
    assert!(!ok);
    assert!(!f.root.join("absent.toml").exists());
    assert_eq!(f.snapshot("vault"), before);
    fs::write(f.root.join("key.pem"), &key).unwrap();
    fs::set_permissions(f.root.join("key.pem"), fs::Permissions::from_mode(0o644)).unwrap();
    let (ok, _) = f.command(
        "--device-provider-prepare",
        "vault",
        "template.toml",
        "absent.toml",
        Some(format!("{PASS}\n").as_bytes()),
        false,
    );
    assert!(!ok);
    assert!(!f.root.join("absent.toml").exists());
    assert_eq!(f.snapshot("vault"), before);
    fs::set_permissions(f.root.join("key.pem"), fs::Permissions::from_mode(0o600)).unwrap();
    fs::hard_link(f.root.join("key.pem"), f.root.join("hard-key.pem")).unwrap();
    let (ok, _) = f.command(
        "--device-provider-prepare",
        "vault",
        "template.toml",
        "absent.toml",
        Some(format!("{PASS}\n").as_bytes()),
        true,
    );
    assert!(!ok);
    assert!(!f.root.join("absent.toml").exists());
    assert_eq!(f.snapshot("vault"), before);
}
