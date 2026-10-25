//! Synthetic member-only archive storage controls; no network or archive authority.
use super::*;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};
const PASS: &[u8] = b"archive checkpoint synthetic passphrase";
const NOW: u64 = 1_792_886_400;
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        Self(std::env::temp_dir().canonicalize().unwrap().join(format!(
            "faction211-storage-{}",
            hex(&random::<16>().unwrap())
        )))
    }
    fn store(&self, create: bool) -> Store {
        Store::at(&self.0, create).unwrap()
    }
    fn reopen(&self) -> CheckpointVault {
        CheckpointVault::unlock(self.store(false), PASS, false).unwrap()
    }
    fn bytes(&self) -> Vec<u8> {
        fs::read(self.0.join(NAME)).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn fresh(f: &Fixture, home: &str, id: u32) -> (CheckpointVault, Zeroizing<[u8; 32]>) {
    CheckpointVault::create(
        f.store(true),
        Data::new(home.into()).unwrap(),
        PASS,
        id,
        NOW,
    )
    .unwrap()
}
fn enroll(a: &mut CheckpointVault, b: &CheckpointVault, home: &str, id: u32) {
    a.enroll(
        home,
        id,
        &format!("ed25519:{}", hex(&b.bundle().unwrap().identity)),
        NOW,
    )
    .unwrap();
}
fn pair() -> (Fixture, Fixture, CheckpointVault, CheckpointVault) {
    let a = Fixture::new();
    let b = Fixture::new();
    let (mut av, _) = fresh(&a, "alice@home.test", 101);
    let (mut bv, _) = fresh(&b, "bob@away.test", 202);
    enroll(&mut av, &bv, "bob@away.test", 202);
    enroll(&mut bv, &av, "alice@home.test", 101);
    av.start(&bv.bundle().unwrap(), NOW).unwrap();
    (a, b, av, bv)
}
fn queued(a: &mut CheckpointVault, id: u8) -> Wire {
    let mut wire = a
        .queue(
            [id; 16],
            "synthetic archive subject",
            "synthetic archive body",
            NOW,
            NOW,
        )
        .unwrap();
    wire.sent_at = 0;
    wire
}
fn unchanged(f: &Fixture, before: &[u8]) {
    assert!(
        f.bytes() == before,
        "canonical ciphertext changed on refusal"
    );
}
fn schema(v: &CheckpointVault) -> [u8; 8] {
    v.state.encode().unwrap()[..8].try_into().unwrap()
}

#[test]
fn explicit_archive_migration_preserves_v2_state_wraps_and_recovery() {
    let (a, b, mut av, mut bv) = pair();
    let first = queued(&mut av, 1);
    bv.receive_network(&first, NOW).unwrap();
    let reply = queued(&mut bv, 2);
    av.receive_network(&reply, NOW).unwrap();
    let before = av.state.encode().unwrap();
    let old_header = av.header.clone();
    let old_key = Zeroizing::new(*av.key);
    let pending = av.pending_deliveries(NOW).unwrap();
    let old_revision = av.state.revision;
    assert_eq!(&before[..8], b"FACTCP02");
    assert!(av.archive_checkpoint(NOW).is_err());
    av.enable_archive(NOW).unwrap();
    assert_eq!(schema(&av), *b"FACTCP03");
    assert!(
        av.header == old_header && *av.key == *old_key,
        "migration changed existing wrapping keys"
    );
    assert_eq!(av.state.revision, old_revision + 1);
    let token = av.archive_checkpoint(NOW).unwrap();
    assert_eq!(token.home, "alice@home.test");
    assert_eq!(token.local_device, 101);
    assert_ne!(token.epoch, [0; 16]);
    assert_eq!(token.after, None);
    let mut old_view = av.state.duplicate().unwrap();
    old_view.archive = None;
    old_view.revision = old_revision;
    assert!(
        old_view.encode().unwrap().as_slice() == before.as_slice(),
        "native state/history/trust/outbox changed in migration"
    );
    assert_eq!(
        av.pending_deliveries(NOW).unwrap()[0].operation,
        pending[0].operation
    );
    assert_eq!(av.pending(NOW).unwrap()[0], pending[0].wire);
    assert_eq!(&a.bytes()[..8], b"FACTVLT2");
    let canonical = a.bytes();
    av.enable_archive(NOW).unwrap();
    unchanged(&a, &canonical);
    drop(av);
    let av = a.reopen();
    assert_eq!(av.archive_checkpoint(NOW).unwrap(), token);
    drop(av);
    drop(bv);
    drop(b);
}

#[test]
fn schema02_stays_exact_and_schema03_rejects_malformed_or_foreign_scope() {
    let (_a, _b, mut av, _bv) = pair();
    let original = av.state.encode().unwrap();
    assert!(
        State::decode(&original)
            .unwrap()
            .encode()
            .unwrap()
            .as_slice()
            == original.as_slice()
    );
    assert_eq!(schema(&av), *b"FACTCP02");
    av.enable_archive(NOW).unwrap();
    let enabled = av.state.encode().unwrap();
    for end in [enabled.len() - 1, enabled.len() - 16] {
        assert!(State::decode(&enabled[..end]).is_err());
    }
    let mut trailing = enabled.to_vec();
    trailing.push(0);
    assert!(State::decode(&trailing).is_err());
    let mut wrong_magic = enabled.to_vec();
    wrong_magic[..8].copy_from_slice(b"FACTCP02");
    assert!(State::decode(&wrong_magic).is_err());
    let mut missing_trailer = original.to_vec();
    missing_trailer[..8].copy_from_slice(b"FACTCP03");
    assert!(State::decode(&missing_trailer).is_err());
    for choice in 0..5 {
        let mut state = av.state.duplicate().unwrap();
        let cursor = state.archive.as_mut().unwrap();
        match choice {
            0 => cursor.home = "foreign@home.test".into(),
            1 => cursor.local_device = 999,
            2 => cursor.epoch = [0; 16],
            3 => cursor.after = Some("bad id".into()),
            _ => cursor.after = Some("x".repeat(257)),
        }
        assert!(state.encode().is_err());
    }
}

#[test]
fn archive_receive_commits_authenticated_time_cursor_and_reopen_duplicate_once() {
    let (_a, b, mut av, mut bv) = pair();
    let wire = queued(&mut av, 1);
    bv.enable_archive(NOW).unwrap();
    let first = bv.archive_checkpoint(NOW).unwrap();
    let receipt = bv
        .receive_archive(&first, "archive-1", &wire, NOW + 17)
        .unwrap();
    assert!(!receipt.duplicate);
    assert_eq!(bv.history().unwrap().len(), 1);
    assert_eq!(bv.history().unwrap()[0].sent_at, NOW);
    let next = bv.archive_checkpoint(NOW).unwrap();
    assert_eq!(next.after.as_deref(), Some("archive-1"));
    let snapshot = Zeroizing::new(bv.state.snapshot.to_vec());
    drop(bv);
    let mut bv = b.reopen();
    assert_eq!(bv.archive_checkpoint(NOW).unwrap(), next);
    assert!(
        bv.receive_archive(&next, "archive-2", &wire, NOW + 99)
            .unwrap()
            .duplicate
    );
    assert_eq!(bv.history().unwrap().len(), 1);
    assert!(bv.state.snapshot.as_slice() == snapshot.as_slice());
    assert_eq!(
        bv.archive_checkpoint(NOW).unwrap().after.as_deref(),
        Some("archive-2")
    );
    // Live receive retains its own exact-duplicate semantics and never moves the cursor.
    let before = b.bytes();
    assert!(bv.receive_network(&wire, NOW).unwrap().duplicate);
    unchanged(&b, &before);
}

#[test]
fn archive_cas_binding_ids_and_tamper_stop_before_checkpoint_mutation() {
    let (_a, b, mut av, mut bv) = pair();
    let wire = queued(&mut av, 1);
    bv.enable_archive(NOW).unwrap();
    let token = bv.archive_checkpoint(NOW).unwrap();
    let before = b.bytes();
    for choice in 0..4 {
        let mut wrong = token.clone();
        match choice {
            0 => wrong.home = "foreign@home.test".into(),
            1 => wrong.local_device += 1,
            2 => wrong.epoch[0] ^= 1,
            _ => wrong.after = Some("not-current".into()),
        }
        assert!(bv.receive_archive(&wrong, "item", &wire, NOW).is_err());
        unchanged(&b, &before);
    }
    for id in [
        "".to_owned(),
        " ".into(),
        "bad\tvalue".into(),
        "bad\nvalue".into(),
        "é".into(),
        "\u{7f}".into(),
        "x".repeat(257),
    ] {
        assert!(bv.receive_archive(&token, &id, &wire, NOW).is_err());
        unchanged(&b, &before);
    }
    for choice in 0..7 {
        let mut bad = wire.clone();
        match choice {
            0 => bad.payload[0] ^= 1,
            1 => bad.identity[0] ^= 1,
            2 => bad.device += 1,
            3 => bad.to_device += 1,
            4 => bad.to = "foreign@home.test".into(),
            5 => bad.from = "foreign@home.test".into(),
            _ => bad.sent_at = NOW,
        }
        assert!(bv.receive_archive(&token, "item", &bad, NOW).is_err());
        unchanged(&b, &before);
    }
    bv.receive_archive(&token, &"x".repeat(256), &wire, NOW)
        .unwrap();
    let now = bv.archive_checkpoint(NOW).unwrap();
    let accepted = b.bytes();
    assert!(bv.receive_archive(&token, "later", &wire, NOW).is_err());
    assert!(
        bv.receive_archive(&now, now.after.as_deref().unwrap(), &wire, NOW)
            .is_err()
    );
    unchanged(&b, &accepted);
}

#[test]
fn known_own_archive_echo_advances_only_cursor_even_after_retirement() {
    let (a, _b, mut av, _bv) = pair();
    let wire = queued(&mut av, 1);
    av.enable_archive(NOW).unwrap();
    let snapshot = Zeroizing::new(av.state.snapshot.to_vec());
    let before_pending = av.pending(NOW).unwrap();
    let token = av.archive_checkpoint(NOW).unwrap();
    let receipt = av.receive_archive(&token, "own-1", &wire, NOW).unwrap();
    assert!(receipt.duplicate);
    assert_eq!(av.pending(NOW).unwrap(), before_pending);
    assert_eq!(av.history().unwrap().len(), 1);
    assert!(av.state.snapshot.as_slice() == snapshot.as_slice());
    av.acknowledge([1; 16]).unwrap();
    drop(av);
    let mut av = a.reopen();
    let token = av.archive_checkpoint(NOW).unwrap();
    assert!(
        av.receive_archive(&token, "own-2", &wire, NOW)
            .unwrap()
            .duplicate
    );
    assert!(av.pending(NOW).unwrap().is_empty());
    assert_eq!(av.history().unwrap().len(), 1);
    let token = av.archive_checkpoint(NOW).unwrap();
    let before = a.bytes();
    for choice in 0..5 {
        let mut unknown = wire.clone();
        match choice {
            0 => unknown.payload[0] ^= 1,
            1 => unknown.identity[0] ^= 1,
            2 => unknown.to_device += 1,
            3 => unknown.device += 1,
            _ => unknown.to = "foreign@home.test".into(),
        }
        assert!(
            av.receive_archive(&token, "unknown", &unknown, NOW)
                .is_err()
        );
        unchanged(&a, &before);
    }
}

#[test]
fn full_history_or_outbox_blocks_archive_duplicates_and_echoes() {
    for history_full in [false, true] {
        let (_a, b, mut av, mut bv) = pair();
        let wire = queued(&mut av, 1);
        bv.enable_archive(NOW).unwrap();
        let token = bv.archive_checkpoint(NOW).unwrap();
        bv.receive_archive(&token, "received", &wire, NOW).unwrap();
        let token = bv.archive_checkpoint(NOW).unwrap();
        let count = if history_full {
            MAX_HISTORY - 1
        } else {
            MAX_OUTBOX
        };
        for i in 0..count {
            bv.queue([(i + 1) as u8; 16], "bounded", "synthetic", NOW, NOW)
                .unwrap();
            if history_full {
                bv.acknowledge([(i + 1) as u8; 16]).unwrap();
            }
        }
        let before = b.bytes();
        assert!(bv.archive_checkpoint(NOW).is_err());
        assert!(bv.enable_archive(NOW).is_err());
        assert!(bv.receive_archive(&token, "duplicate", &wire, NOW).is_err());
        unchanged(&b, &before);
        if !history_full {
            let mut own = bv.pending(NOW).unwrap()[0].clone();
            own.sent_at = 0;
            assert!(bv.receive_archive(&token, "own", &own, NOW).is_err());
            unchanged(&b, &before);
        }
    }
}

#[test]
fn inactive_revoked_expired_and_poisoned_archive_operations_refuse() {
    let (a, _b, mut av, _bv) = pair();
    let wire = queued(&mut av, 1);
    av.enable_archive(NOW).unwrap();
    let token = av.archive_checkpoint(NOW).unwrap();
    let before = a.bytes();
    for time in [NOW - 1, NOW + SESSION_AGE + 1] {
        assert!(av.archive_checkpoint(time).is_err());
        assert!(av.enable_archive(time).is_err());
        assert!(av.receive_archive(&token, "own", &wire, time).is_err());
        unchanged(&a, &before);
    }
    av.revoke().unwrap();
    let before = a.bytes();
    assert!(av.archive_checkpoint(NOW).is_err());
    assert!(av.enable_archive(NOW).is_err());
    assert!(av.receive_archive(&token, "own", &wire, NOW).is_err());
    unchanged(&a, &before);
    drop(av);
    let fixture = Fixture::new();
    let (mut v, _) = fresh(&fixture, "empty@home.test", 303);
    assert!(v.enable_archive(NOW).is_err());
    assert!(v.archive_checkpoint(NOW).is_err());
}

#[test]
fn recovery_backup_and_reset_preserve_inert_cursor_and_fresh_device_rescopes() {
    let (a, _b, mut av, _bv) = pair();
    let wire = queued(&mut av, 1);
    av.enable_archive(NOW).unwrap();
    let token = av.archive_checkpoint(NOW).unwrap();
    av.receive_archive(&token, "own", &wire, NOW).unwrap();
    let token = av.archive_checkpoint(NOW).unwrap();
    let recovery = av.reset(PASS).unwrap();
    assert_eq!(av.archive_checkpoint(NOW).unwrap(), token);
    let backup = Fixture::new();
    av.backup(&backup.0).unwrap();
    let mut backed = backup.reopen();
    assert_eq!(backed.state.archive.as_ref().unwrap(), &token);
    assert!(backed.archive_checkpoint(NOW).is_err());
    assert!(backed.receive_archive(&token, "next", &wire, NOW).is_err());
    backed.reset(PASS).unwrap();
    assert!(backed.state.history_only);
    assert_eq!(backed.state.archive.as_ref().unwrap(), &token);
    drop(av);
    let mut av = CheckpointVault::unlock(a.store(false), &*recovery, true).unwrap();
    assert_eq!(av.state.archive.as_ref().unwrap(), &token);
    assert!(av.archive_checkpoint(NOW).is_err());
    assert!(av.receive_archive(&token, "next", &wire, NOW).is_err());
    av.reset(PASS).unwrap();
    assert!(av.state.history_only);
    assert_eq!(av.state.archive.as_ref().unwrap(), &token);
    assert!(av.fresh_device(404, NOW).is_err());
    av.discard_pending().unwrap();
    av.fresh_device(404, NOW).unwrap();
    assert_eq!(schema(&av), *b"FACTCP03");
    let fresh = av.state.archive.as_ref().unwrap();
    assert_eq!(fresh.local_device, 404);
    assert_eq!(fresh.home, token.home);
    assert_ne!(fresh.epoch, token.epoch);
    assert_eq!(fresh.after, None);
    assert_eq!(av.history().unwrap().len(), 1);
    assert!(av.archive_checkpoint(NOW).is_err()); // deliberate peer reenrollment is still required
    assert!(av.receive_archive(&token, "stale", &wire, NOW).is_err());
}

#[test]
fn archive_write_failures_poison_and_preserve_atomic_candidate() {
    for fault in [
        Fault::BeforeWrite,
        Fault::BeforeFileSync,
        Fault::BeforePublish,
        Fault::AfterPublish,
        Fault::DirectorySync,
    ] {
        let (_a, b, mut av, mut bv) = pair();
        let wire = queued(&mut av, 1);
        bv.enable_archive(NOW).unwrap();
        let token = bv.archive_checkpoint(NOW).unwrap();
        bv.fail_once(fault);
        assert!(bv.receive_archive(&token, "atomic", &wire, NOW).is_err());
        assert!(bv.archive_checkpoint(NOW).is_err());
        assert!(bv.enable_archive(NOW).is_err());
        assert!(bv.receive_archive(&token, "retry", &wire, NOW).is_err());
        drop(bv);
        let bv = b.reopen();
        let accepted = matches!(fault, Fault::AfterPublish | Fault::DirectorySync);
        assert_eq!(bv.history().unwrap().len(), usize::from(accepted));
        assert_eq!(
            bv.archive_checkpoint(NOW).unwrap().after.as_deref(),
            if accepted { Some("atomic") } else { None }
        );
    }
}

#[test]
#[ignore = "owned subprocess helper, invoked only by archive checkpoint crash controls"]
fn archive_checkpoint_crash_child() {
    let owned = |name| {
        let p = PathBuf::from(std::env::var_os(name).unwrap());
        assert!(
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("faction211-storage-")
        );
        p
    };
    let receiver = owned("FACTION211_TEST_RECEIVER");
    let sender = owned("FACTION211_TEST_SENDER");
    let mut bv =
        CheckpointVault::unlock(Store::at(&receiver, false).unwrap(), PASS, false).unwrap();
    let mut wire = if receiver == sender {
        bv.pending(NOW).unwrap()[0].clone()
    } else {
        let av = CheckpointVault::unlock(Store::at(&sender, false).unwrap(), PASS, false).unwrap();
        av.pending(NOW).unwrap()[0].clone()
    };
    wire.sent_at = 0;
    let token = bv.archive_checkpoint(NOW).unwrap();
    bv.fail_once(
        if std::env::var("FACTION211_TEST_BOUNDARY").unwrap() == "before" {
            Fault::CrashBeforePublish
        } else {
            Fault::CrashAfterPublish
        },
    );
    let _ = bv.receive_archive(&token, "crash-item", &wire, NOW);
    std::process::exit(76);
}
#[test]
fn archive_process_death_couples_message_cursor_and_native_snapshot() {
    for mode in ["incoming", "duplicate", "own"] {
        for boundary in ["before", "after"] {
            let (a, b, mut av, mut bv) = pair();
            let wire = queued(&mut av, 1);
            if mode == "own" {
                av.enable_archive(NOW).unwrap();
            } else {
                bv.enable_archive(NOW).unwrap();
            }
            if mode == "duplicate" {
                let token = bv.archive_checkpoint(NOW).unwrap();
                bv.receive_archive(&token, "prior-item", &wire, NOW)
                    .unwrap();
            }
            let (target, v) = if mode == "own" { (&a, &av) } else { (&b, &bv) };
            let before = target.bytes();
            let snapshot = Zeroizing::new(v.state.snapshot.to_vec());
            let prior = v.archive_checkpoint(NOW).unwrap();
            let history = v.history().unwrap().len();
            drop(av);
            drop(bv);
            let status = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "mail_checkpoint::archive_checkpoint_tests::archive_checkpoint_crash_child",
                    "--ignored",
                    "--test-threads=1",
                ])
                .env("FACTION211_TEST_RECEIVER", &target.0)
                .env("FACTION211_TEST_SENDER", &a.0)
                .env("FACTION211_TEST_BOUNDARY", boundary)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap();
            assert_eq!(status.code(), Some(75));
            let mut v = target.reopen();
            let token = v.archive_checkpoint(NOW).unwrap();
            if boundary == "before" {
                unchanged(target, &before);
                assert_eq!(v.history().unwrap().len(), history);
                assert_eq!(token, prior);
                assert!(v.state.snapshot.as_slice() == snapshot.as_slice());
                assert_eq!(
                    v.receive_archive(&token, "retry", &wire, NOW)
                        .unwrap()
                        .duplicate,
                    mode != "incoming"
                );
            } else {
                assert_eq!(
                    v.history().unwrap().len(),
                    history + usize::from(mode == "incoming")
                );
                assert_eq!(token.after.as_deref(), Some("crash-item"));
                if mode != "incoming" {
                    assert!(v.state.snapshot.as_slice() == snapshot.as_slice());
                }
                let committed = Zeroizing::new(v.state.snapshot.to_vec());
                assert!(
                    v.receive_archive(&token, "duplicate", &wire, NOW)
                        .unwrap()
                        .duplicate
                );
                assert!(v.state.snapshot.as_slice() == committed.as_slice());
            }
            if mode == "own" {
                assert_eq!(v.pending_deliveries(NOW).unwrap()[0].operation, [1; 16]);
            }
        }
    }
}

#[test]
fn partial_archive_page_preserves_last_good_cursor_when_next_item_fails() {
    let (_a, b, mut av, mut bv) = pair();
    let first = queued(&mut av, 1);
    let second = queued(&mut av, 2);
    bv.enable_archive(NOW).unwrap();
    let token = bv.archive_checkpoint(NOW).unwrap();
    bv.receive_archive(&token, "first", &first, NOW).unwrap();
    let next = bv.archive_checkpoint(NOW).unwrap();
    let before = b.bytes();
    let mut tampered = second.clone();
    tampered.payload[0] ^= 1;
    assert!(bv.receive_archive(&next, "second", &tampered, NOW).is_err());
    unchanged(&b, &before);
    drop(bv);
    let mut bv = b.reopen();
    assert_eq!(bv.archive_checkpoint(NOW).unwrap(), next);
    assert_eq!(bv.history().unwrap().len(), 1);
    assert!(
        !bv.receive_archive(&next, "second", &second, NOW)
            .unwrap()
            .duplicate
    );
    assert_eq!(bv.history().unwrap().len(), 2);
}

#[test]
fn archive_inspection_exposes_only_enabled_and_cursor_presence_even_when_inactive() {
    let (_a, _b, mut av, _bv) = pair();
    let status = av.inspection().unwrap();
    assert!(!status.archive_enabled && !status.archive_cursor_present);
    let wire = queued(&mut av, 1);
    av.enable_archive(NOW).unwrap();
    let status = av.inspection().unwrap();
    assert!(status.archive_enabled && !status.archive_cursor_present);
    let token = av.archive_checkpoint(NOW).unwrap();
    av.receive_archive(&token, "inert-private-id", &wire, NOW)
        .unwrap();
    let status = av.inspection().unwrap();
    assert!(status.archive_enabled && status.archive_cursor_present);
    assert!(av.archive_checkpoint(NOW + SESSION_AGE + 1).is_err());
    av.revoke().unwrap();
    let status = av.inspection().unwrap();
    assert!(status.archive_enabled && status.archive_cursor_present);
    let backup = Fixture::new();
    av.backup(&backup.0).unwrap();
    let backed = backup.reopen();
    let status = backed.inspection().unwrap();
    assert!(status.history_only && status.archive_enabled && status.archive_cursor_present);
    assert!(backed.archive_checkpoint(NOW).is_err());
}

#[test]
fn original_recovery_secret_remains_valid_after_explicit_archive_migration() {
    let a = Fixture::new();
    let b = Fixture::new();
    let (mut av, original_recovery) = fresh(&a, "alice@home.test", 101);
    let (bv, _) = fresh(&b, "bob@away.test", 202);
    enroll(&mut av, &bv, "bob@away.test", 202);
    av.enable_archive(NOW).unwrap();
    let token = av.archive_checkpoint(NOW).unwrap();
    drop(av);
    let recovered = CheckpointVault::unlock(a.store(false), &*original_recovery, true).unwrap();
    assert!(recovered.inspection().unwrap().history_only);
    assert_eq!(recovered.state.archive.as_ref().unwrap(), &token);
    assert!(recovered.archive_checkpoint(NOW).is_err());
}
