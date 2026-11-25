//! Real vault authority and PTY Screen controls; transport qualification is separate.
#[path = "checkpoint_ui_conversation_actions_tests.rs"]
mod actions;
#[path = "checkpoint_ui_archive_review_tests.rs"]
mod archive_review_tests;
#[path = "checkpoint_ui_brief_review_tests.rs"]
mod brief_review_tests;
#[path = "checkpoint_ui_password_cancel_tests.rs"]
mod password_cancel_tests;
#[path = "checkpoint_ui_pending_tests.rs"]
mod pending_tests;
use super::*;
use faction_client::{
    mail_checkpoint::{Delivery, Wire},
    mail_network::{ForegroundAction, ForegroundReport, ForegroundWrite, Settings},
};
use faction_omemo::{Device, sce};
use std::{io::Write, os::unix::fs::PermissionsExt};

const SECRET: &[u8] = b"synthetic foreground local password";
struct Fixture {
    root: std::path::PathBuf,
    vault: CheckpointVault,
    remote: Device,
    settings: Settings,
    time: u64,
}
impl Fixture {
    fn new() -> Self {
        let mut random = [0; 16];
        getrandom::fill(&mut random).unwrap();
        let root = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("faction356-ui-OWNED-{}", hex(&random)));
        std::fs::create_dir(&root).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::write(root.join("OWNERSHIP"), b"synthetic foreground UI control\n").unwrap();
        let ca = root.join("ca");
        std::fs::write(&ca, b"unused controlled-cycle CA").unwrap();
        std::fs::set_permissions(&ca, std::fs::Permissions::from_mode(0o600)).unwrap();
        let config = root.join("home.toml");
        std::fs::write(&config, format!("version=1\naccount=\"alice@home.test\"\nendpoint=\"127.0.0.1:9\"\ndomain=\"home.test\"\nca={:?}\npin=\"{}\"\n", ca.to_str().unwrap(), "00".repeat(32))).unwrap();
        std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o600)).unwrap();
        let time = now().unwrap();
        let (mut vault, _) = CheckpointVault::create(
            Store::at(&root.join("vault"), true).unwrap(),
            Data::new("alice@home.test".into()).unwrap(),
            SECRET,
            101,
            time,
        )
        .unwrap();
        let mut remote = Device::fresh().unwrap();
        let peer = remote.bundle().unwrap();
        let local = vault.bundle().unwrap();
        vault
            .enroll(
                "bob@away.test",
                202,
                &format!("ed25519:{}", hex(&peer.identity)),
                time,
            )
            .unwrap();
        remote
            .trust(&format!("ed25519:{}", hex(&local.identity)))
            .unwrap();
        remote.start(&local).unwrap();
        let mut fixture = Self {
            root,
            vault,
            remote,
            settings: Settings::load(&config).unwrap(),
            time,
        };
        let wire = fixture.wire("keep original");
        fixture.vault.receive(&wire, time).unwrap();
        fixture.save("cold");
        fixture.vault.enable_archive(time).unwrap();
        fixture.save("empty-cursor");
        let mut wire = fixture.wire("keep archived");
        wire.sent_at = 0;
        let checkpoint = fixture.vault.archive_checkpoint(time).unwrap();
        fixture
            .vault
            .receive_archive(&checkpoint, "saved-position", &wire, time)
            .unwrap();
        fixture.vault.enable_read_tracking().unwrap();
        fixture
            .vault
            .queue(
                [1; 16],
                "keep selected",
                "private selected body",
                time,
                time,
            )
            .unwrap();
        fixture
            .vault
            .queue(
                [2; 16],
                "keep other pending",
                "private other body",
                time,
                time,
            )
            .unwrap();
        fixture
    }
    fn save(&self, name: &str) {
        let path = self.root.join(name);
        std::fs::create_dir(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::copy(self.root.join("vault/mail.fkv"), path.join("mail.fkv")).unwrap();
    }
    fn open(&self, name: &str) -> CheckpointVault {
        CheckpointVault::unlock(
            Store::at(&self.root.join(name), false).unwrap(),
            SECRET,
            false,
        )
        .unwrap()
    }
    fn wire(&mut self, subject: &str) -> Wire {
        let state = self.vault.inspection().unwrap();
        let peer = state.peer.unwrap();
        let from = peer.address.to_owned();
        let device = peer.device;
        let to_device = state.local_device;
        let plain = sce::encode(
            &from,
            "alice@home.test",
            self.time,
            subject,
            "private received body",
        )
        .unwrap();
        let sealed = self.remote.seal(&plain).unwrap();
        Wire {
            namespace: "urn:xmpp:omemo:2".into(),
            from,
            to: "alice@home.test".into(),
            device,
            to_device,
            identity: self.remote.bundle().unwrap().identity,
            sent_at: self.time,
            key: sealed.key,
            prekey: sealed.prekey,
            payload: sealed.payload,
        }
    }
    fn rotate(&mut self, address: &str, device: u32) {
        self.vault.discard_pending().unwrap();
        self.vault.fresh_device(102, self.time).unwrap();
        self.remote = Device::fresh().unwrap();
        let peer = self.remote.bundle().unwrap();
        let local = self.vault.bundle().unwrap();
        self.vault
            .enroll(
                address,
                device,
                &format!("ed25519:{}", hex(&peer.identity)),
                self.time,
            )
            .unwrap();
        self.remote
            .trust(&format!("ed25519:{}", hex(&local.identity)))
            .unwrap();
        self.remote.start(&local).unwrap();
        let mut wire = self.wire("current replacement");
        wire.sent_at = 0;
        let checkpoint = self.vault.archive_checkpoint(self.time).unwrap();
        self.vault
            .receive_archive(&checkpoint, "replacement-position", &wire, self.time)
            .unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        actions::retain_finished(self);
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}

// Native allocations disable process core dumps. Each fixture executes in a child.
fn child(test: &str) {
    let before = rustix::process::getrlimit(rustix::process::Resource::Core);
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--ignored", "--nocapture"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let after = rustix::process::getrlimit(rustix::process::Resource::Core);
    assert_eq!(
        (before.current, before.maximum),
        (after.current, after.maximum)
    );
}

fn capture_mail_menu(f: &Fixture, name: &str, online: bool, unicode: bool) -> String {
    let vault = f.open(name);
    let canonical = f.root.join(name).join("mail.fkv");
    let before = std::fs::read(&canonical).unwrap();
    let (master, slave) = faction_client_pty::open(24, 80).unwrap();
    let saved = rustix::termios::tcgetattr(&slave).unwrap();
    let mut raw = saved.clone();
    raw.make_raw();
    rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw).unwrap();
    let capture = f.root.join("mail-menu-output");
    let mut term = LocalTerminal {
        input: slave.into(),
        output: std::fs::File::create(&capture).unwrap(),
        saved,
    };
    let mut driver = Driver {
        input: master.into(),
        capture: capture.clone(),
        offset: 0,
    };
    std::thread::scope(|scope| {
        let input = scope.spawn(move || {
            driver.wait("[L] Lock and exit");
            driver.send(b"?o");
            driver
        });
        let mut screen = Screen {
            term: &mut term,
            activity: Instant::now(),
            unicode,
        };
        let key = if online {
            screen.online_menu(&vault)
        } else {
            screen.menu(&vault)
        };
        input.join().unwrap();
        assert_eq!(key.unwrap(), b'O');
    });
    drop(term);
    assert_eq!(std::fs::read(&canonical).unwrap(), before);
    let output = std::fs::read_to_string(capture).unwrap();
    assert!(output.contains("[E] Enable local read tracking\r\n"));
    assert!(output.contains("[I] Local inbox\r\n"));
    output
}

#[test]
fn mail_menu_local_restrictions() {
    child("checkpoint_ui::foreground_tests::mail_menu_local_restrictions_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn mail_menu_local_restrictions_helper() {
    let mut f = Fixture::new();
    f.save("healthy");
    f.vault.revoke().unwrap();
    f.save("revoked");
    let backup = f.root.join("history-only");
    f.vault.backup(&backup).unwrap();

    let prefixes = [
        "HISTORY ONLY:",
        "EXPIRED /",
        "REVOKED:",
        "HISTORY FULL:",
        "OUTBOX FULL:",
    ];
    let revoked = "REVOKED: peer blocked locally; no protocol retry.";
    let history_only = "HISTORY ONLY: backup/recovery cannot resume old ratchets.";
    let next_step =
        "Retire pending ciphertext, then create a fresh device and verify the peer again.";
    let mut differences = Vec::new();
    for (name, expected) in [
        ("healthy", &[][..]),
        ("revoked", &[revoked, next_step][..]),
        ("history-only", &[history_only, revoked, next_step][..]),
    ] {
        for unicode in [false, true] {
            for online in [false, true] {
                let output = capture_mail_menu(&f, name, online, unicode);
                let rows: Vec<_> = output
                    .lines()
                    .filter(|row| {
                        prefixes.iter().any(|prefix| row.starts_with(prefix)) || *row == next_step
                    })
                    .collect();
                if rows != expected {
                    differences.push(format!(
                        "{name} online={online} unicode={unicode}: expected {expected:?}, got {rows:?}"
                    ));
                }
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn mail_menu_local_archive_position() {
    child("checkpoint_ui::foreground_tests::mail_menu_local_archive_position_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn mail_menu_local_archive_position_helper() {
    let f = Fixture::new();
    f.save("retained");
    let mut differences = Vec::new();
    for (name, archive, summary, send) in [
        (
            "cold",
            "disabled; explicit upgrade required",
            "Local retained: 1; received 1; pending 0",
            "[N] Connect / sync",
        ),
        (
            "empty-cursor",
            "enabled; no saved position",
            "Local retained: 1; received 1; pending 0",
            "[N] Send pending",
        ),
        (
            "retained",
            "enabled; saved position present",
            "Local unread: 2 / received 2; retained 4; pending 2",
            "[N] Send pending",
        ),
    ] {
        let expected = format!("Archive checkpoints: {archive}");
        for unicode in [false, true] {
            for online in [false, true] {
                let output = capture_mail_menu(&f, name, online, unicode);
                assert!(!output.contains("saved-position"));
                assert!(output.lines().any(|row| row == summary));
                if online {
                    assert!(output.contains(send));
                    assert!(output.contains("[A] Catch up one archive page\r\n"));
                }
                let rows: Vec<_> = output
                    .lines()
                    .filter(|row| row.starts_with("Archive checkpoints:"))
                    .collect();
                if rows != [expected.as_str()] {
                    differences.push(format!(
                        "{name} online={online} unicode={unicode}: expected {expected:?}, got {rows:?}"
                    ));
                }
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn foreground_ui_warm_target() {
    child("checkpoint_ui::foreground_tests::foreground_ui_target_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn foreground_ui_target_helper() {
    let fixture = Fixture::new();
    let target =
        foreground::Target::prepare(&fixture.vault, &fixture.settings, 0, fixture.time).unwrap();
    assert_eq!(target.address.as_str(), "bob@away.test");
    assert_eq!(target.device, 202);
    assert_eq!(target.identity, fixture.remote.bundle().unwrap().identity);
}

#[test]
fn foreground_ui_target_refuses_unsupported_contexts() {
    child("checkpoint_ui::foreground_tests::foreground_ui_refusal_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn foreground_ui_refusal_helper() {
    let mut f = Fixture::new();
    let before = std::fs::read(f.root.join("vault/mail.fkv")).unwrap();
    let mut accepted = Vec::new();
    for (name, group, time) in [
        ("missing-group", usize::MAX, f.time),
        ("past", 0, f.time - 1),
        (
            "expired",
            0,
            f.time + faction_client::mail_checkpoint::SESSION_AGE + 1,
        ),
    ] {
        if foreground::Target::prepare(&f.vault, &f.settings, group, time).is_ok() {
            accepted.push(name);
        }
    }
    f.settings.account = "mallory@home.test".into();
    if foreground::Target::prepare(&f.vault, &f.settings, 0, f.time).is_ok() {
        accepted.push("foreign-home");
    }
    f.settings.account = "alice@home.test".into();
    for name in ["cold", "empty-cursor"] {
        let v = f.open(name);
        if foreground::Target::prepare(&v, &f.settings, 0, f.time).is_ok() {
            accepted.push(name);
        }
    }
    assert_eq!(
        std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
        before
    );
    f.vault.revoke().unwrap();
    if foreground::Target::prepare(&f.vault, &f.settings, 0, f.time).is_ok() {
        accepted.push("revoked");
    }
    assert!(
        accepted.is_empty(),
        "unsupported contexts offered foreground authority: {accepted:?}"
    );
}

#[test]
fn foreground_ui_requires_exact_current_peer_context() {
    child("checkpoint_ui::foreground_tests::foreground_ui_peer_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn foreground_ui_peer_helper() {
    let mut accepted = Vec::new();
    for (name, address, device) in [
        ("account", "carol@away.test", 202),
        ("device", "bob@away.test", 203),
        ("identity", "bob@away.test", 202),
    ] {
        let mut f = Fixture::new();
        f.rotate(address, device);
        assert!(foreground::Target::prepare(&f.vault, &f.settings, 4, f.time).is_ok());
        if foreground::Target::prepare(&f.vault, &f.settings, 0, f.time).is_ok() {
            accepted.push(name);
        }
    }
    assert!(
        accepted.is_empty(),
        "retained peer context replaced current trust: {accepted:?}"
    );
}

#[test]
fn foreground_ui_selects_exact_pending_text() {
    child("checkpoint_ui::foreground_tests::foreground_ui_selection_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn foreground_ui_selection_helper() {
    let mut f = Fixture::new();
    let target = foreground::Target::prepare(&f.vault, &f.settings, 0, f.time).unwrap();
    assert_eq!(target.selected(&f.vault, Some(2)).unwrap(), Some([1; 16]));
    assert_eq!(target.selected(&f.vault, Some(3)).unwrap(), Some([2; 16]));
    let mut wrongly_eligible = Vec::new();
    for (name, index) in [
        ("received", Some(0)),
        ("no-selection", None),
        ("missing", Some(99)),
    ] {
        if target.selected(&f.vault, index).unwrap().is_some() {
            wrongly_eligible.push(name);
        }
    }
    f.vault.acknowledge([1; 16]).unwrap();
    if target.selected(&f.vault, Some(2)).unwrap().is_some() {
        wrongly_eligible.push("retired");
    }
    f.vault.enable_attachments().unwrap();
    let attachment = faction_omemo::attachment::Descriptor {
        name: Zeroizing::new("private.txt".into()),
        media_type: Zeroizing::new("text/plain".into()),
        size: 1,
        hash: Zeroizing::new([3; 32]),
        key: Zeroizing::new([4; 32]),
        iv: Zeroizing::new([5; 12]),
        locator: Zeroizing::new("https://blob.test/private".into()),
    };
    f.vault
        .queue_attachment([3; 16], "attachment", "body", &attachment, f.time, f.time)
        .unwrap();
    if target.selected(&f.vault, Some(4)).unwrap().is_some() {
        wrongly_eligible.push("attachment");
    }
    assert!(
        wrongly_eligible.is_empty(),
        "invalid selected release: {wrongly_eligible:?}"
    );
    assert_eq!(f.vault.inspect_outbox().unwrap().len(), 2);
}

#[test]
fn outbox_shows_matching_local_subject() {
    child("checkpoint_ui::foreground_tests::outbox_subject_helper");
}

#[test]
#[ignore = "owned subprocess helper"]
fn outbox_subject_helper() {
    let mut f = Fixture::new();
    f.vault.acknowledge([1; 16]).unwrap();
    let hostile = format!("λ\n\u{202e} {} HIDDEN", "wide".repeat(20));
    f.vault
        .queue([3; 16], &hostile, "private hostile body", f.time, f.time)
        .unwrap();
    {
        let history = f.vault.history().unwrap();
        let pending = f.vault.inspect_outbox().unwrap();
        let sent = &history[3];
        assert_eq!(sent.operation(), [2; 16]);
        assert_eq!(outbox_subject(&pending[0], [sent, sent]), None);
        for (operation, address, device) in [
            ([9; 16], "bob@away.test", 202),
            ([2; 16], "carol@away.test", 202),
            ([2; 16], "bob@away.test", 203),
            ([0; 16], "bob@away.test", 202),
        ] {
            let item = OutboxInspection {
                operation,
                address,
                device,
                sent_at: f.time,
            };
            assert_eq!(outbox_subject(&item, history), None);
        }
    }
    let checkpoint = f.root.join("vault/mail.fkv");
    let before = std::fs::read(&checkpoint).unwrap();
    for online in [false, true] {
        for unicode in [false, true] {
            let (master, slave) = faction_client_pty::open(24, 40).unwrap();
            let saved = rustix::termios::tcgetattr(&slave).unwrap();
            let mut raw = saved.clone();
            raw.make_raw();
            rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw)
                .unwrap();
            let capture = f.root.join("outbox-output");
            let mut term = LocalTerminal {
                input: slave.into(),
                output: std::fs::File::create(&capture).unwrap(),
                saved,
            };
            let mut driver = Driver {
                input: master.into(),
                capture: capture.clone(),
                offset: 0,
            };
            std::thread::scope(|scope| {
                let driver = scope.spawn(move || {
                    driver.wait("Enter to return: ");
                    driver.send(b"\r");
                    driver
                });
                let mut screen = Screen {
                    term: &mut term,
                    activity: Instant::now(),
                    unicode,
                };
                let result = if online {
                    screen.online_action(&mut f.vault, &f.settings, &f.root, b'O', None, None)
                } else {
                    screen.action(&mut f.vault, b'O')
                };
                driver.join().unwrap();
                result.unwrap();
            });
            drop(term);
            let output = std::fs::read_to_string(capture).unwrap();
            assert_eq!(std::fs::read(&checkpoint).unwrap(), before);
            let details = |id| {
                let operation = hex(&[id; 16]);
                if online {
                    format!(
                        "Operation: {operation} / bob@away.test / UTC {}\r\n",
                        f.time
                    )
                } else {
                    format!(
                        "Operation: {operation}\r\nTo: bob@away.test / device 202 / UTC {}\r\n",
                        f.time
                    )
                }
            };
            if online {
                assert!(output.contains("Durable pending ciphertext; delivery remains unproven."));
                assert!(output.contains("A socket write never retires these IDs."));
            } else {
                assert!(output.contains("Durable outbox: 2/8"));
                assert!(output.contains("Remote delivery is unknown."));
                assert!(output.contains("retirement is a LOCAL decision."));
            }
            assert!(!output.contains(&hex(&[1; 16])));
            for private in [
                "keep selected",
                "keep original",
                "keep archived",
                "private",
                "HIDDEN",
            ] {
                assert!(!output.contains(private), "{private} appeared in O");
            }
            assert_eq!(output.matches("Operation: ").count(), 2);
            assert!(output.contains(&details(2)) && output.contains(&details(3)));
            assert!(
                output.contains(&format!("{}Subject: keep other pending\r\n", details(2))),
                "pending ID lost its local subject (online={online}, unicode={unicode}): {output}"
            );
            let prefix = if unicode { "λ" } else { "\\u{3bb}" };
            let expected = format!("{}Subject: {prefix}\\u{{a}}\\u{{202e}} wide", details(3));
            assert!(
                output.contains(&expected),
                "wrong hostile subject: {output}"
            );
            let row = output
                .lines()
                .find(|row| row.starts_with(&format!("Subject: {prefix}")))
                .unwrap();
            assert!(row.ends_with('~'));
            assert!(faction_terminal::text::width(row) <= 39);
            assert!(!output.contains('\u{202e}'));
        }
    }
}

#[test]
fn foreground_ui_no_arrival_report_is_honest() {
    let text = foreground::report(&ForegroundReport {
        write: ForegroundWrite::NotAttempted,
        accepted: 0,
        duplicates: 0,
        error: None,
    });
    assert!(text.contains("No selected text write attempted."));
    assert!(text.contains("No new arrival observed."));
    assert!(text.contains("does not prove offline"));
    assert!(!text.contains("delivered"));
}

#[test]
fn foreground_ui_report_preserves_each_write_and_partial_progress() {
    use faction_client::mail_network::Error;
    let mut missing = Vec::new();
    for (write, label) in [
        (
            ForegroundWrite::NotAttempted,
            "No selected text write attempted.",
        ),
        (
            ForegroundWrite::Unconfirmed,
            "Selected text write is unconfirmed.",
        ),
        (
            ForegroundWrite::Completed,
            "Selected text local write completed.",
        ),
    ] {
        let text = foreground::report(&ForegroundReport {
            write,
            accepted: 2,
            duplicates: 3,
            error: Some(Error::Protocol),
        });
        for expected in [
            label,
            "Accepted on this device: 2.",
            "Duplicates: 3.",
            "Brief exchange stopped.",
            "Earlier accepted mail remains saved.",
            "Pending ciphertext is retained.",
            "A write is not delivery or reading.",
        ] {
            if !text.contains(expected) {
                missing.push((write, expected));
            }
        }
        if text.contains("No new arrival") {
            missing.push((write, "must not erase accepted arrivals"));
        }
        if text.contains("Protocol") {
            missing.push((write, "must not print diagnostic details"));
        }
        for row in text.lines() {
            if !row.is_ascii() || row.len() > 39 {
                missing.push((write, "classic report row must fit"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "write/progress observations erased: {missing:?}"
    );
}

struct Driver {
    input: std::fs::File,
    capture: std::path::PathBuf,
    offset: usize,
}
impl Driver {
    fn wait(&mut self, needle: &str) {
        let limit = Instant::now() + Duration::from_secs(15);
        loop {
            let text = std::fs::read_to_string(&self.capture).unwrap();
            if let Some(at) = text[self.offset..].find(needle) {
                self.offset += at + needle.len();
                return;
            }
            assert!(
                Instant::now() < limit,
                "foreground screen did not reach {needle}; actual: {text}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    fn send(&mut self, input: &[u8]) {
        self.input.write_all(input).unwrap();
    }
}

fn drive_arrival_reply(d: &mut Driver, subject: &str, identity: &[u8; 32], queue: bool) -> bool {
    let capture = std::fs::read_to_string(&d.capture).unwrap();
    let menu = &capture[capture.rfind("[LOCAL NEWLY RECEIVED TEXT]").unwrap()..];
    d.send(b"w");
    if !menu.contains("[W] Write multiline reply") {
        // The baseline ignores W. The caller finishes the existing route before failing.
        return false;
    }
    d.wait("CURRENT recipient: bob@away.test");
    d.wait(&format!("CURRENT device: 202 / ed25519:{}", hex(identity)));
    d.wait(&format!("Suggested subject: {subject}"));
    d.wait("Subject (Enter keeps suggestion;");
    d.send(b"\r");
    d.wait("write> ");
    d.send(b"arrival reply first\r");
    d.wait("write> ");
    d.send(b"arrival reply second\r");
    d.wait("write> ");
    d.send(b".done\r");
    d.wait("Review local multiline intent:");
    d.wait("CURRENT recipient: bob@away.test");
    d.wait(&format!("CURRENT device: 202 / ed25519:{}", hex(identity)));
    d.wait(&format!("Subject: {subject}"));
    d.wait("arrival reply first");
    d.wait("arrival reply second");
    d.wait("Type queue to persist encrypted ciphertext: ");
    d.send(if queue { b"queue\r" } else { b"\r" });
    d.wait(if queue {
        "Queued durably; no delivery or read receipt."
    } else {
        "Writing cancelled; no ciphertext queued."
    });
    d.wait("[LOCAL NEWLY RECEIVED TEXT]");
    true
}

fn check_arrival_reply(
    f: &Fixture,
    index: usize,
    subject: &str,
    pending: &[Delivery],
    queued: bool,
) {
    let deliveries = f.vault.pending_deliveries(f.time).unwrap();
    assert_eq!(deliveries.len(), pending.len() + usize::from(queued));
    for (delivery, before) in deliveries.iter().zip(pending) {
        assert_eq!(delivery.operation, before.operation);
        assert_eq!(delivery.wire, before.wire);
    }
    let history = f.vault.history().unwrap();
    assert_eq!(history.len(), index + usize::from(queued));
    if queued {
        let sent = &history[index];
        let delivery = deliveries.last().unwrap();
        assert_eq!(sent.operation(), delivery.operation);
        assert_eq!(sent.subject.as_str(), subject);
        assert_eq!(
            sent.body.as_str(),
            "arrival reply first\narrival reply second"
        );
        assert_eq!(sent.address, "bob@away.test");
        assert_eq!(sent.device, 202);
        assert_eq!(sent.identity, f.remote.bundle().unwrap().identity);
        assert_eq!(delivery.wire.to, sent.address);
        assert_eq!(delivery.wire.to_device, sent.device);
        assert!(sent.received_id().is_none());
    }
}

fn view(
    f: &mut Fixture,
    online: bool,
    unicode: bool,
    selected: usize,
    drive: impl FnOnce(&mut Driver) + Send,
) -> (io::Result<()>, String, inbox::Inbox) {
    view_controlled(
        f,
        online,
        unicode,
        selected,
        inbox::Filter::All,
        None,
        drive,
    )
}
fn view_controlled(
    f: &mut Fixture,
    online: bool,
    unicode: bool,
    selected: usize,
    filter: inbox::Filter,
    cycle: Option<&foreground::ControlledForeground<'_>>,
    drive: impl FnOnce(&mut Driver) + Send,
) -> (io::Result<()>, String, inbox::Inbox) {
    let (master, slave) = faction_client_pty::open(24, 40).unwrap();
    let saved = rustix::termios::tcgetattr(&slave).unwrap();
    let mut raw = saved.clone();
    raw.make_raw();
    rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw).unwrap();
    let capture = f.root.join("capture");
    let mut term = LocalTerminal {
        input: slave.into(),
        output: std::fs::File::create(&capture).unwrap(),
        saved,
    };
    let mut driver = Driver {
        input: master.into(),
        capture: capture.clone(),
        offset: 0,
    };
    let mut list = inbox::Inbox::new(Screen::inbox_group_entries(&f.vault, Some(0)).unwrap());
    list.filter(filter);
    let search = inbox::Search::parse(b"keep").unwrap();
    list.search(Screen::inbox_matches(&f.vault, &search).unwrap());
    list.select_index(selected);
    let result = std::thread::scope(|scope| {
        let driver = scope.spawn(move || {
            drive(&mut driver);
            driver
        });
        let mut exchange = HomeExchange::new(&f.settings);
        exchange.foreground = cycle;
        let result = Screen {
            term: &mut term,
            activity: Instant::now(),
            unicode,
        }
        .conversation(
            &mut f.vault,
            online,
            online.then_some(&exchange),
            0,
            &mut list,
            &search,
        );
        driver.join().unwrap();
        result
    });
    drop(term);
    (result, std::fs::read_to_string(capture).unwrap(), list)
}

#[test]
fn foreground_ui_conversation_entry_is_deliberate_and_fits_classic() {
    child("checkpoint_ui::foreground_tests::foreground_ui_entry_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn foreground_ui_entry_helper() {
    let mut f = Fixture::new();
    for unicode in [false, true] {
        let before = std::fs::read(f.root.join("vault/mail.fkv")).unwrap();
        let (result, text, list) = view(&mut f, true, unicode, 2, |d| {
            d.wait("[B] Back");
            let text = std::fs::read_to_string(&d.capture).unwrap();
            assert!(
                text.contains("[E] Brief exchange"),
                "matching warm conversation has no deliberate foreground entry"
            );
            d.send(b"b");
        });
        result.unwrap();
        let start = text.find("[LOCAL CONVERSATION]").unwrap();
        let end = start + text[start..].find("[B] Back\r\n").unwrap() + "[B] Back\r\n".len();
        let screen = &text[start..end];
        assert!(screen.lines().count() <= 23);
        for row in screen.lines() {
            assert!(
                faction_terminal::text::width(row) <= 39,
                "wide conversation: {row}"
            );
        }
        assert_eq!(list.selected().unwrap().index, 2);
        assert_eq!(
            std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
            before
        );
    }
    let (result, text, _) = view(&mut f, false, false, 2, |d| {
        d.wait("[B] Back");
        d.send(b"eb");
    });
    result.unwrap();
    assert!(!text.contains("[E] Brief exchange"));
}

#[test]
fn foreground_ui_review_and_fresh_confirmation_cancel_in_place() {
    child("checkpoint_ui::foreground_tests::foreground_ui_review_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn foreground_ui_review_helper() {
    let mut f = Fixture::new();
    for unicode in [false, true] {
        for action in [b'r', b's', b'b'] {
            let before = std::fs::read(f.root.join("vault/mail.fkv")).unwrap();
            let (result, text, list) = view(&mut f, true, unicode, 2, move |d| {
                d.wait("[B] Back");
                d.send(b"esexchange\rqueued password\r");
                d.wait("[LOCAL BRIEF EXCHANGE]");
                d.wait("[B] Return");
                let review = std::fs::read_to_string(&d.capture).unwrap();
                for required in [
                    "Home: alice@home.test",
                    "Peer: bob@away.test",
                    "Peer device: 202",
                    "Current identity:",
                    "At most 10 seconds; idle wait 2 seconds.",
                    "At most 32 stanzas; no automatic retry.",
                    "A write is not delivery or reading.",
                    "No arrival does not prove offline.",
                    "[R] Receive briefly",
                    "[S] Send selected text",
                ] {
                    assert!(
                        review.contains(required),
                        "missing consequential review: {required}"
                    );
                }
                assert!(!review.contains("HOME account password"));
                d.send(&[action]);
                if action != b'b' {
                    d.wait("Type exchange to connect, or Enter: ");
                    let review = std::fs::read_to_string(&d.capture).unwrap();
                    if action == b's' {
                        assert!(
                            review.contains("Retained record: 3")
                                && review.contains(&hex(&[1; 16]))
                        );
                    }
                    d.send(b"\r");
                }
                d.wait("[LOCAL CONVERSATION]");
                d.wait("[B] Back");
                d.send(b"b");
            });
            result.unwrap();
            assert!(!text.contains("HOME account password"));
            assert_eq!(list.selected().unwrap().index, 2);
            assert_eq!(
                std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
                before
            );
        }
    }
}

#[test]
fn foreground_ui_fresh_dispatch_retains_progress_and_exact_view() {
    child("checkpoint_ui::foreground_tests::foreground_ui_dispatch_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn foreground_ui_dispatch_helper() {
    use faction_client::mail_network::Error;
    use std::cell::{Cell, RefCell};
    let mut f = Fixture::new();
    let read = f.vault.history().unwrap()[1].received_id().unwrap();
    f.vault.mark_read(1, read).unwrap();
    for unicode in [false, true] {
        for (send, write, accepted, failure) in [
            (false, ForegroundWrite::NotAttempted, false, false),
            (false, ForegroundWrite::NotAttempted, true, false),
            (false, ForegroundWrite::NotAttempted, true, true),
            (true, ForegroundWrite::Completed, true, false),
            (true, ForegroundWrite::Unconfirmed, true, true),
            (true, ForegroundWrite::NotAttempted, false, true),
        ] {
            let cursor = f.vault.archive_checkpoint(f.time).unwrap();
            let pending = f.vault.pending(f.time).unwrap();
            let summary = f.vault.read_summary().unwrap();
            let identity = f.remote.bundle().unwrap().identity;
            let mut wire = f.wire("keep newly accepted");
            wire.sent_at = 0;
            let wire = RefCell::new(Some(wire));
            let calls = Cell::new(0);
            let cycle = |v: &mut CheckpointVault,
                         password: &str,
                         peer: faction_client::mail_network::ForegroundPeer<'_>,
                         action: ForegroundAction| {
                assert_eq!(password, "fresh synthetic home password");
                assert_eq!(
                    (peer.address, peer.device, peer.identity),
                    ("bob@away.test", 202, identity)
                );
                assert_eq!(
                    action,
                    if send {
                        ForegroundAction::SendSelected([1; 16])
                    } else {
                        ForegroundAction::ReceiveOnly
                    }
                );
                calls.set(calls.get() + 1);
                if accepted {
                    v.receive_network(&wire.borrow_mut().take().unwrap(), now().unwrap())
                        .unwrap();
                }
                ForegroundReport {
                    write,
                    accepted: usize::from(accepted),
                    duplicates: 2,
                    error: failure.then_some(Error::Protocol),
                }
            };
            let filter = if send {
                inbox::Filter::All
            } else {
                inbox::Filter::Unread
            };
            let selected = if send { 2 } else { 0 };
            let (result, text, list) = view_controlled(
                &mut f,
                true,
                unicode,
                selected,
                filter,
                Some(&cycle),
                move |d| {
                    d.wait("[B] Back");
                    d.send(b"e");
                    d.wait("[B] Return");
                    d.send(if send { b"s" } else { b"r" });
                    d.wait("Type exchange to connect, or Enter: ");
                    d.send(b"exchange\r");
                    d.wait("HOME account password (hidden; never sent to a BBS): ");
                    d.send(b"fresh synthetic home password\r");
                    d.wait("Pending ciphertext is retained.");
                    d.wait("Enter to return: ");
                    d.send(b"\r");
                    d.wait("[LOCAL CONVERSATION]");
                    d.wait("[B] Back");
                    d.send(b"b");
                },
            );
            result.unwrap();
            assert_eq!(calls.get(), 1);
            assert_eq!(f.vault.archive_checkpoint(f.time).unwrap(), cursor);
            assert_eq!(f.vault.pending(f.time).unwrap(), pending);
            assert_eq!(
                f.vault.read_summary().unwrap().unread,
                summary.unread.map(|n| n + usize::from(accepted))
            );
            assert_eq!(f.vault.received_read(1, read).unwrap(), Some(true));
            assert_eq!(list.selected().unwrap().index, selected);
            assert_eq!(list.mode(), filter);
            assert_eq!(
                list.eligible().len(),
                if send {
                    f.vault.history().unwrap().len()
                } else {
                    f.vault.read_summary().unwrap().unread.unwrap()
                }
            );
            let tail = &text[text.rfind("[LOCAL CONVERSATION]").unwrap()..];
            assert!(tail.contains(if send {
                "All / Search"
            } else {
                "Unread / Search"
            }));
            assert!(!text.contains("fresh synthetic home password"));
            assert_eq!(text.matches("HOME account password").count(), 1);
            assert!(text.contains(if accepted {
                "Accepted on this device: 1."
            } else {
                "No new arrival observed."
            }));
            if failure {
                assert!(text.contains("Brief exchange stopped."));
            }
        }
    }
}

#[test]
fn foreground_ui_typeahead_fresh_password_and_predispatch_controls() {
    child("checkpoint_ui::foreground_tests::foreground_ui_fresh_input_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn foreground_ui_fresh_input_helper() {
    use std::cell::Cell;
    let mut f = Fixture::new();
    for unicode in [false, true] {
        let calls = Cell::new(0);
        let cycle = |_v: &mut CheckpointVault,
                     password: &str,
                     _peer: faction_client::mail_network::ForegroundPeer<'_>,
                     action: ForegroundAction| {
            assert_eq!(action, ForegroundAction::ReceiveOnly);
            assert_eq!(password, "fresh synthetic home password");
            calls.set(calls.get() + 1);
            ForegroundReport {
                write: ForegroundWrite::NotAttempted,
                accepted: 0,
                duplicates: 0,
                error: None,
            }
        };
        let before = std::fs::read(f.root.join("vault/mail.fkv")).unwrap();
        let (result, text, _) = view_controlled(
            &mut f,
            true,
            unicode,
            0,
            inbox::Filter::Unread,
            Some(&cycle),
            |d| {
                d.wait("[B] Back");
                d.send(b"e");
                d.wait("[B] Return");
                let review = std::fs::read_to_string(&d.capture).unwrap();
                assert!(!review.contains("[S] Send selected text"));
                d.send(b"srexchange\rqueued home password\r");
                d.wait("Type exchange to connect, or Enter: ");
                std::thread::sleep(Duration::from_millis(75));
                assert!(
                    !std::fs::read_to_string(&d.capture)
                        .unwrap()
                        .contains("HOME account password"),
                    "typeahead authorized connection"
                );
                d.send(b"\r");
                d.wait("[LOCAL CONVERSATION]");
                d.wait("[B] Back");
                for _ in 0..2 {
                    d.send(b"e");
                    d.wait("[B] Return");
                    d.send(b"r");
                    d.wait("Type exchange to connect, or Enter: ");
                    d.send(b"exchange\rqueued home password\r");
                    d.wait("HOME account password (hidden; never sent to a BBS): ");
                    d.send(b"fresh synthetic home password\r");
                    d.wait("Enter to return: ");
                    d.send(b"\r");
                    d.wait("[LOCAL CONVERSATION]");
                    d.wait("[B] Back");
                }
                d.send(b"b");
            },
        );
        result.unwrap();
        assert_eq!(calls.get(), 2);
        assert_eq!(text.matches("HOME account password").count(), 2);
        assert!(
            !text.contains("queued home password")
                && !text.contains("fresh synthetic home password")
        );
        assert_eq!(
            std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
            before
        );
        for control in [3, 4] {
            for password in [false, true] {
                let (result, text, _) = view_controlled(
                    &mut f,
                    true,
                    unicode,
                    0,
                    inbox::Filter::Unread,
                    Some(&cycle),
                    move |d| {
                        d.wait("[B] Back");
                        d.send(b"e");
                        d.wait("[B] Return");
                        d.send(b"r");
                        d.wait("Type exchange to connect, or Enter: ");
                        if password {
                            d.send(b"exchange\r");
                            d.wait("HOME account password (hidden; never sent to a BBS): ");
                        }
                        d.send(&[control]);
                    },
                );
                assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Interrupted);
                assert_eq!(calls.get(), 2, "cancelled credential input dispatched");
                assert_eq!(text.matches("[LOCAL CONVERSATION]").count(), 1);
                assert_eq!(
                    std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
                    before
                );
            }
        }
    }
}

#[test]
fn foreground_ui_poisoned_checkpoint_locks_and_reopens_canonical() {
    child("checkpoint_ui::foreground_tests::foreground_ui_poison_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn foreground_ui_poison_helper() {
    use faction_client::mail_network::Error;
    use std::cell::RefCell;
    for unicode in [false, true] {
        let mut f = Fixture::new();
        let cursor = f.vault.archive_checkpoint(f.time).unwrap();
        let pending = f.vault.pending(f.time).unwrap();
        let mut wire = f.wire("keep accepted before poison");
        wire.sent_at = 0;
        let wire = RefCell::new(Some(wire));
        let canonical = f.root.join("vault/mail.fkv");
        let saved = f.root.join("vault/accepted-canonical");
        let cycle = |v: &mut CheckpointVault,
                     _password: &str,
                     _peer: faction_client::mail_network::ForegroundPeer<'_>,
                     _action: ForegroundAction| {
            v.receive_network(&wire.borrow_mut().take().unwrap(), now().unwrap())
                .unwrap();
            std::fs::rename(&canonical, &saved).unwrap();
            assert!(
                v.queue(
                    [9; 16],
                    "fault",
                    "not published",
                    now().unwrap(),
                    now().unwrap()
                )
                .is_err()
            );
            std::fs::rename(&saved, &canonical).unwrap();
            assert!(v.inspection().is_err());
            ForegroundReport {
                write: ForegroundWrite::NotAttempted,
                accepted: 1,
                duplicates: 0,
                error: Some(Error::State),
            }
        };
        let (result, text, _) = view_controlled(
            &mut f,
            true,
            unicode,
            0,
            inbox::Filter::Unread,
            Some(&cycle),
            |d| {
                d.wait("[B] Back");
                d.send(b"e");
                d.wait("[B] Return");
                d.send(b"r");
                d.wait("Type exchange to connect, or Enter: ");
                d.send(b"exchange\r");
                d.wait("HOME account password (hidden; never sent to a BBS): ");
                d.send(b"fresh synthetic home password\r");
                d.wait("Lock and reopen the canonical store.");
            },
        );
        assert!(result.is_err());
        assert!(f.vault.inspection().is_err());
        assert!(!text.contains("Enter to return:") && !text.contains("Accepted on this device:"));
        assert_eq!(text.matches("[LOCAL CONVERSATION]").count(), 1);
        let placeholder = f.open("cold");
        drop(std::mem::replace(&mut f.vault, placeholder));
        f.vault = CheckpointVault::unlock(
            Store::at(&f.root.join("vault"), false).unwrap(),
            SECRET,
            false,
        )
        .unwrap();
        assert_eq!(f.vault.archive_checkpoint(f.time).unwrap(), cursor);
        assert_eq!(f.vault.pending(f.time).unwrap(), pending);
        assert_eq!(f.vault.read_summary().unwrap().unread, Some(3));
    }
}

#[test]
fn foreground_ui_production_pending_connection_ctrl_c_d_closes() {
    child("checkpoint_ui::foreground_tests::foreground_ui_active_cancel_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn foreground_ui_active_cancel_helper() {
    use std::{io::Read, net::TcpListener, sync::mpsc};
    let mut f = Fixture::new();
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let ca = f.root.join("cancel-ca.pem");
    let key = f.root.join("cancel-key.pem");
    let openssl_config = f.root.join("cancel-openssl.cnf");
    std::fs::write(&openssl_config, b"[req]\ndistinguished_name=dn\n[dn]\n").unwrap();
    std::fs::set_permissions(&openssl_config, std::fs::Permissions::from_mode(0o600)).unwrap();
    let output = std::process::Command::new("openssl")
        .args(["req", "-config"])
        .arg(&openssl_config)
        .args(["-x509", "-newkey", "rsa:2048", "-nodes", "-keyout"])
        .arg(&key)
        .arg("-out")
        .arg(&ca)
        .args(["-days", "1", "-subj", "/CN=synthetic.invalid"])
        .output()
        .unwrap();
    assert!(output.status.success(), "fixture CA generation failed");
    for path in [&ca, &key] {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let config = f.root.join("cancel-home.toml");
    std::fs::write(&config, format!("version=1\naccount=\"alice@home.test\"\nendpoint=\"{}\"\ndomain=\"home.test\"\nca={:?}\npin=\"{}\"\n", listener.local_addr().unwrap(), ca.to_str().unwrap(), "00".repeat(32))).unwrap();
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o600)).unwrap();
    f.settings = Settings::load(&config).unwrap();
    for unicode in [false, true] {
        for control in [3, 4] {
            let before = std::fs::read(f.root.join("vault/mail.fkv")).unwrap();
            let (accepted, ready) = mpsc::channel();
            std::thread::scope(|scope| {
                let server = scope.spawn(|| {
                    let deadline = Instant::now() + Duration::from_secs(5);
                    let mut socket = loop {
                        match listener.accept() {
                            Ok((socket, peer)) => {
                                assert!(peer.ip().is_loopback());
                                break socket;
                            }
                            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                                assert!(
                                    Instant::now() < deadline,
                                    "production connection never arrived"
                                );
                                std::thread::sleep(Duration::from_millis(10));
                            }
                            Err(error) => panic!("controlled accept failed: {error}"),
                        }
                    };
                    socket.set_nonblocking(false).unwrap();
                    socket
                        .set_read_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    accepted.send(()).unwrap();
                    let mut bytes = 0;
                    let mut buffer = [0; 1024];
                    loop {
                        match socket.read(&mut buffer) {
                            Ok(0) => break,
                            Ok(n) => {
                                bytes += n;
                                assert!(bytes <= 65536);
                            }
                            Err(error) if error.kind() == io::ErrorKind::ConnectionReset => break,
                            Err(error) => panic!("cancelled connection did not close: {error}"),
                        }
                    }
                });
                let (result, text, _) = view(&mut f, true, unicode, 0, move |d| {
                    d.wait("[B] Back");
                    d.send(b"e");
                    d.wait("[B] Return");
                    d.send(b"r");
                    d.wait("Type exchange to connect, or Enter: ");
                    d.send(b"exchange\r");
                    d.wait("HOME account password (hidden; never sent to a BBS): ");
                    d.send(b"fresh synthetic home password\r");
                    d.wait("Connecting directly to your home.");
                    ready.recv_timeout(Duration::from_secs(3)).unwrap();
                    let pending = std::fs::read_to_string(&d.capture).unwrap();
                    assert!(
                        !pending.contains("Pending ciphertext is retained.")
                            && !pending.contains("Enter to return:"),
                        "production future finished before cancellation"
                    );
                    d.send(&[control]);
                });
                assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Interrupted);
                assert_eq!(text.matches("[LOCAL CONVERSATION]").count(), 1);
                assert!(
                    !text.contains("Pending ciphertext is retained.")
                        && !text.contains("fresh synthetic home password")
                );
                server.join().unwrap();
            });
            assert_eq!(
                std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
                before
            );
        }
    }
    drop(listener);
}
