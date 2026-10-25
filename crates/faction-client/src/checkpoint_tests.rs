use crate::{
    mail_checkpoint::{CheckpointVault, Fault, MAX_OUTBOX, SESSION_AGE},
    mail_data::Data,
    mail_vault::{Store, Vault},
};
use std::{fs, path::PathBuf};
use zeroize::Zeroizing;
const PASS: &[u8] = b"checkpoint local testing passphrase";
const NOW: u64 = 1_792_886_400;
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let mut id = [0; 16];
        getrandom::fill(&mut id).unwrap();
        Self(
            std::env::temp_dir()
                .canonicalize()
                .unwrap()
                .join(format!("faction203-{}", crate::mail_data::hex(&id))),
        )
    }
    fn store(&self, create: bool) -> Store {
        Store::at(&self.0, create).unwrap()
    }
    fn reopen(&self) -> CheckpointVault {
        CheckpointVault::unlock(self.store(false), PASS, false).unwrap()
    }
    fn bytes(&self) -> Vec<u8> {
        fs::read(self.0.join("mail.fkv")).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn fresh(f: &Fixture, home: &str, id: u32) -> CheckpointVault {
    CheckpointVault::create(
        f.store(true),
        Data::new(home.into()).unwrap(),
        PASS,
        id,
        NOW,
    )
    .unwrap()
    .0
}
fn pair() -> (Fixture, Fixture, CheckpointVault, CheckpointVault) {
    let a = Fixture::new();
    let b = Fixture::new();
    let mut av = fresh(&a, "alice@home.test", 101);
    let mut bv = fresh(&b, "bob@away.test", 202);
    let ab = av.bundle().unwrap();
    let bb = bv.bundle().unwrap();
    av.enroll(
        "bob@away.test",
        202,
        &format!("ed25519:{}", crate::mail_data::hex(&bb.identity)),
        NOW,
    )
    .unwrap();
    bv.enroll(
        "alice@home.test",
        101,
        &format!("ed25519:{}", crate::mail_data::hex(&ab.identity)),
        NOW,
    )
    .unwrap();
    av.start(&bb, NOW).unwrap();
    (a, b, av, bv)
}
#[test]
fn encrypted_send_receive_retry_and_ack_survive_reopen() {
    let (a, b, mut av, mut bv) = pair();
    let wire = av
        .queue([1; 16], "private subject λ", "private body <&>", NOW, NOW)
        .unwrap();
    for marker in [
        b"private subject".as_slice(),
        b"private body",
        b"alice@home.test",
    ] {
        assert!(!a.bytes().windows(marker.len()).any(|v| v == marker));
    }
    drop(av);
    let mut av = a.reopen();
    assert_eq!(av.pending(NOW).unwrap(), vec![wire.clone()]);
    assert_eq!(
        av.queue(
            [1; 16],
            "private subject λ",
            "private body <&>",
            NOW,
            NOW + 1
        )
        .unwrap(),
        wire
    );
    assert!(
        av.queue([1; 16], "changed", "private body <&>", NOW, NOW)
            .is_err()
    );
    assert!(!bv.receive(&wire, NOW).unwrap().duplicate);
    drop(bv);
    let mut bv = b.reopen();
    assert!(bv.receive(&wire, NOW + 1).unwrap().duplicate);
    assert_eq!(bv.history().unwrap().len(), 1);
    assert_eq!(
        bv.history().unwrap()[0].subject.as_str(),
        "private subject λ"
    );
    av.acknowledge([1; 16]).unwrap();
    drop(av);
    let mut av = a.reopen();
    assert!(av.pending(NOW).unwrap().is_empty());
    assert!(
        av.queue([1; 16], "private subject λ", "private body <&>", NOW, NOW)
            .is_err()
    );
    assert_eq!(av.history().unwrap().len(), 1);
}
#[test]
fn delayed_keys_and_reply_ratchets_commit_with_history() {
    let (a, b, mut av, mut bv) = pair();
    let mut wires = Vec::new();
    for i in 0..3 {
        wires.push(av.queue([i + 1; 16], "late", "body", NOW, NOW).unwrap());
    }
    bv.receive(&wires[2], NOW).unwrap();
    drop(bv);
    let mut bv = b.reopen();
    let reply = bv
        .queue([9; 16], "reply", "other direction", NOW, NOW)
        .unwrap();
    av.receive(&reply, NOW).unwrap();
    drop(av);
    let mut av = a.reopen();
    bv.receive(&wires[0], NOW).unwrap();
    bv.receive(&wires[1], NOW).unwrap();
    let next = av.queue([8; 16], "after restart", "ok", NOW, NOW).unwrap();
    bv.receive(&next, NOW).unwrap();
    assert_eq!(bv.history().unwrap().len(), 5);
}
#[test]
fn validation_tamper_trust_and_duplicates_do_not_change_canonical_state() {
    let (_a, b, mut av, mut bv) = pair();
    let wire = av.queue([1; 16], "subject", "body", NOW, NOW).unwrap();
    let before = b.bytes();
    let mut bads = Vec::new();
    let mut v = wire.clone();
    v.identity[31] ^= 128;
    bads.push(v);
    let mut v = wire.clone();
    v.device += 1;
    bads.push(v);
    let mut v = wire.clone();
    v.to_device += 1;
    bads.push(v);
    let mut v = wire.clone();
    v.from = "other@home.test".into();
    bads.push(v);
    let mut v = wire.clone();
    v.payload[0] ^= 1;
    bads.push(v);
    let mut v = wire.clone();
    v.key.truncate(1);
    bads.push(v);
    let mut v = wire.clone();
    v.sent_at += 1000;
    bads.push(v);
    for v in bads {
        assert!(bv.receive(&v, NOW).is_err());
        assert_eq!(b.bytes(), before);
    }
    bv.receive(&wire, NOW).unwrap();
    let accepted = b.bytes();
    assert!(bv.receive(&wire, NOW).unwrap().duplicate);
    assert_eq!(b.bytes(), accepted);
    bv.revoke().unwrap();
    let revoked = b.bytes();
    assert!(bv.receive(&wire, NOW).is_err());
    assert!(bv.pending(NOW).is_err());
    assert_eq!(b.bytes(), revoked);
}
#[test]
fn write_errors_poison_until_durable_reopen() {
    for fault in [
        Fault::BeforeWrite,
        Fault::BeforeFileSync,
        Fault::BeforePublish,
        Fault::AfterPublish,
        Fault::DirectorySync,
    ] {
        let (a, _, mut av, _) = pair();
        let before = a.bytes();
        av.fail_once(fault);
        assert!(av.queue([3; 16], "fault", "secret", NOW, NOW).is_err());
        assert!(av.pending(NOW).is_err());
        assert!(av.history().is_err());
        assert!(av.bundle().is_err());
        assert!(av.queue([4; 16], "blocked", "secret", NOW, NOW).is_err());
        drop(av);
        let mut av = a.reopen();
        if matches!(fault, Fault::AfterPublish | Fault::DirectorySync) {
            assert_ne!(a.bytes(), before);
            assert_eq!(av.pending(NOW).unwrap().len(), 1);
            assert_eq!(
                av.queue([3; 16], "fault", "secret", NOW, NOW).unwrap(),
                av.pending(NOW).unwrap()[0]
            );
        } else {
            assert_eq!(a.bytes(), before);
            assert!(av.pending(NOW).unwrap().is_empty());
        }
        fs::write(
            a.0.join(".pending-not-authority"),
            b"not authenticated state",
        )
        .unwrap();
        drop(av);
        assert!(a.reopen().history().is_ok());
    }
}
#[test]
fn explicit_upgrade_preserves_v1_drafts_pins_and_original_backup() {
    let a = Fixture::new();
    let backup = Fixture::new();
    let mut data = Data::new("alice@home.test".into()).unwrap();
    data.draft(
        "bob@away.test".into(),
        Zeroizing::new("old draft".into()),
        Zeroizing::new("old body".into()),
    )
    .unwrap();
    data.pin("bob@away.test".into(), 202, [9; 32]).unwrap();
    let (v, old_recovery) = Vault::create(a.store(true), data, PASS).unwrap();
    v.backup(&backup.0).unwrap();
    let (mut v, recovery) = CheckpointVault::upgrade(v, PASS, 101, NOW).unwrap();
    assert_eq!(v.legacy().unwrap().drafts.len(), 1);
    assert_eq!(v.legacy().unwrap().pins[0].fingerprint, [9; 32]);
    assert!(
        v.enroll("bob@away.test", 202, &crate::mail_data::hex(&[9; 32]), NOW)
            .is_err()
    );
    assert_ne!(*recovery, *old_recovery);
    drop(v);
    assert!(Vault::unlock(a.store(false), PASS, false).is_err());
    assert!(Vault::unlock(backup.store(false), &*old_recovery, true).is_ok());
    assert!(CheckpointVault::unlock(a.store(false), &*old_recovery, true).is_err());
    assert_eq!(
        a.reopen().legacy().unwrap().drafts[0].body.as_str(),
        "old body"
    );
}
#[test]
fn backup_is_history_only_until_explicit_fresh_device() {
    let (a, _, mut av, _) = pair();
    let b = Fixture::new();
    av.queue([1; 16], "already delivered maybe", "private", NOW, NOW)
        .unwrap();
    av.backup(&b.0).unwrap();
    drop(av);
    let mut restored = b.reopen();
    assert!(restored.pending(NOW).is_err());
    assert!(restored.queue([2; 16], "no", "resume", NOW, NOW).is_err());
    assert_eq!(restored.history().unwrap().len(), 1);
    assert!(restored.fresh_device(303, NOW).is_err());
    restored.discard_pending().unwrap();
    restored.fresh_device(303, NOW).unwrap();
    assert_eq!(restored.history().unwrap().len(), 1);
    assert!(restored.pending(NOW).is_err());
    assert_eq!(a.reopen().pending(NOW).unwrap().len(), 1);
}
#[test]
fn locks_wrong_secrets_malformed_bytes_and_caps_fail_closed() {
    let (a, _, mut av, _) = pair();
    assert!(Store::at(&a.0, false).is_err());
    for i in 0..MAX_OUTBOX {
        av.queue([i as u8 + 1; 16], "bounded", "body", NOW, NOW)
            .unwrap();
    }
    let before = a.bytes();
    assert!(av.queue([99; 16], "full", "body", NOW, NOW).is_err());
    assert_eq!(a.bytes(), before);
    assert!(av.pending(NOW + SESSION_AGE + 1).is_err());
    drop(av);
    assert!(CheckpointVault::unlock(a.store(false), b"incorrect local passphrase", false).is_err());
    let original = a.bytes();
    for index in [0, 40, 185, original.len() - 1] {
        let mut bad = original.clone();
        bad[index] ^= 1;
        fs::write(a.0.join("mail.fkv"), bad).unwrap();
        assert!(CheckpointVault::unlock(a.store(false), PASS, false).is_err());
    }
    fs::write(a.0.join("mail.fkv"), &original[..original.len() - 1]).unwrap();
    assert!(CheckpointVault::unlock(a.store(false), PASS, false).is_err());
}
#[test]
fn checkpoint_review_recovery_quarantine_survives_password_reopen() {
    let a = Fixture::new();
    let (v, key) = CheckpointVault::create(
        a.store(true),
        Data::new("alice@home.test".into()).unwrap(),
        PASS,
        101,
        NOW,
    )
    .unwrap();
    assert!(v.bundle().is_ok());
    drop(v);
    let v = CheckpointVault::unlock(a.store(false), &*key, true).unwrap();
    assert!(v.bundle().is_err());
    drop(v);
    assert!(a.reopen().bundle().is_err());
}
#[test]
fn checkpoint_review_parent_sync_failure_prevents_usable_initialization() {
    let a = Fixture::new();
    crate::mail_vault::fail_parent_sync_once();
    assert!(Store::at(&a.0, true).is_err());
    assert!(!a.0.join("mail.fkv").exists());
}
#[test]
fn checkpoint_review_restart_outbox_recovers_acknowledgement_id() {
    let (a, _b, mut av, _bv) = pair();
    av.queue([21; 16], "persistent ID", "body", NOW, NOW)
        .unwrap();
    drop(av);
    let mut av = a.reopen();
    let deliveries = av.pending_deliveries(NOW).unwrap();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].operation, [21; 16]);
    av.acknowledge(deliveries[0].operation).unwrap();
    assert!(av.pending_deliveries(NOW).unwrap().is_empty());
}
#[test]
#[ignore = "owned subprocess helper, invoked only by checkpoint crash controls"]
fn checkpoint_crash_child() {
    let path = std::env::var_os("FACTION203_TEST_STORE").unwrap();
    let path = PathBuf::from(path);
    assert!(
        path.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("faction203-")
    );
    let mut v = CheckpointVault::unlock(Store::at(&path, false).unwrap(), PASS, false).unwrap();
    let case = std::env::var("FACTION203_TEST_CASE").unwrap();
    if case == "delivery" {
        let other = PathBuf::from(std::env::var_os("FACTION203_TEST_RECEIVER").unwrap());
        assert!(
            other
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("faction203-")
        );
        let mut receiver =
            CheckpointVault::unlock(Store::at(&other, false).unwrap(), PASS, false).unwrap();
        let p = v.pending_deliveries(NOW).unwrap();
        receiver.receive(&p[0].wire, NOW).unwrap();
        std::process::exit(75);
    }
    v.fail_once(if case == "before" {
        Fault::CrashBeforePublish
    } else {
        Fault::CrashAfterPublish
    });
    let _ = v.queue(
        [31; 16],
        "crash-private-marker",
        "crash-body-marker",
        NOW,
        NOW,
    );
    std::process::exit(76);
}
fn crash_child(a: &Fixture, case: &str, b: Option<&Fixture>) -> std::process::ExitStatus {
    let mut cmd = std::process::Command::new(std::env::current_exe().unwrap());
    cmd.args([
        "--exact",
        "checkpoint_tests::checkpoint_crash_child",
        "--ignored",
        "--test-threads=1",
    ])
    .env("FACTION203_TEST_STORE", &a.0)
    .env("FACTION203_TEST_CASE", case)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null());
    if let Some(b) = b {
        cmd.env("FACTION203_TEST_RECEIVER", &b.0);
    }
    cmd.status().unwrap()
}
#[test]
fn process_death_before_and_after_publication_keeps_canonical_state() {
    for case in ["before", "after"] {
        let (a, b, av, bv) = pair();
        let before = a.bytes();
        drop(av);
        drop(bv);
        assert_eq!(crash_child(&a, case, None).code(), Some(75));
        let av = a.reopen();
        let pending = av.pending_deliveries(NOW).unwrap();
        if case == "before" {
            assert_eq!(a.bytes(), before);
            assert!(pending.is_empty());
        } else {
            assert_ne!(a.bytes(), before);
            assert_eq!(pending.len(), 1);
            let mut bv = b.reopen();
            assert!(!bv.receive(&pending[0].wire, NOW).unwrap().duplicate);
        }
        for p in fs::read_dir(&a.0)
            .unwrap()
            .map(|p| p.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".checkpoint-")
            })
        {
            let raw = fs::read(p).unwrap();
            for marker in [b"crash-private-marker".as_slice(), b"crash-body-marker"] {
                assert!(!raw.windows(marker.len()).any(|v| v == marker));
            }
        }
    }
}
#[test]
fn process_death_after_delivery_before_ack_retries_identical_ciphertext() {
    let (a, b, mut av, bv) = pair();
    let original = av
        .queue([41; 16], "delivered once", "private body", NOW, NOW)
        .unwrap();
    drop(av);
    drop(bv);
    assert_eq!(crash_child(&a, "delivery", Some(&b)).code(), Some(75));
    let mut av = a.reopen();
    let mut bv = b.reopen();
    let pending = av.pending_deliveries(NOW).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].wire, original);
    assert!(bv.receive(&pending[0].wire, NOW).unwrap().duplicate);
    assert_eq!(bv.history().unwrap().len(), 1);
    av.acknowledge(pending[0].operation).unwrap();
    drop(av);
    assert!(a.reopen().pending_deliveries(NOW).unwrap().is_empty());
}
#[test]
fn reset_write_ambiguity_never_rolls_back_into_a_live_handle() {
    for fault in [
        Fault::BeforePublish,
        Fault::AfterPublish,
        Fault::DirectorySync,
    ] {
        let (a, _b, mut av, _bv) = pair();
        av.queue([51; 16], "preserved through reset", "body", NOW, NOW)
            .unwrap();
        av.fail_once(fault);
        assert!(av.reset(b"a different checkpoint passphrase").is_err());
        assert!(av.history().is_err());
        assert!(av.pending(NOW).is_err());
        drop(av);
        let (pw, bad) = if matches!(fault, Fault::BeforePublish) {
            (PASS, b"a different checkpoint passphrase".as_slice())
        } else {
            (b"a different checkpoint passphrase".as_slice(), PASS)
        };
        assert!(CheckpointVault::unlock(a.store(false), bad, false).is_err());
        let v = CheckpointVault::unlock(a.store(false), pw, false).unwrap();
        assert_eq!(v.pending_deliveries(NOW).unwrap().len(), 1);
    }
}
