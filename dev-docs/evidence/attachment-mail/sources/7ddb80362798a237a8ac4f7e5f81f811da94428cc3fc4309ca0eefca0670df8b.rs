//! Private schema/failure controls for the explicitly adopted attachment format.
use super::*;
use faction_omemo::{attachment::Descriptor, blob::MAX_BLOB_BYTES};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};
const NOW: u64 = 1_792_886_400;
const PASS: &[u8] = b"synthetic attachment schema password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        Self(std::env::temp_dir().canonicalize().unwrap().join(format!(
            "faction257-schema-{}",
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
fn fresh(f: &Fixture, who: &str, id: u32) -> (CheckpointVault, Zeroizing<[u8; 32]>) {
    CheckpointVault::create(f.store(true), Data::new(who.into()).unwrap(), PASS, id, NOW).unwrap()
}
fn descriptor() -> Descriptor {
    Descriptor {
        name: Zeroizing::new("schema-private.txt".into()),
        media_type: Zeroizing::new("text/plain".into()),
        size: MAX_BLOB_BYTES as u64,
        hash: Zeroizing::new([3; 32]),
        key: Zeroizing::new([4; 32]),
        iv: Zeroizing::new([5; 12]),
        locator: Zeroizing::new("https://blob.test/private".into()),
    }
}
fn assert_descriptor(a: &Descriptor, b: &Descriptor) {
    assert!(
        *a.name == *b.name
            && *a.media_type == *b.media_type
            && a.size == b.size
            && *a.hash == *b.hash
            && *a.key == *b.key
            && *a.iv == *b.iv
            && *a.locator == *b.locator,
        "private descriptor mismatch"
    );
}
fn pair() -> (Fixture, Fixture, CheckpointVault, CheckpointVault) {
    let a = Fixture::new();
    let b = Fixture::new();
    let (mut av, _) = fresh(&a, "alice@home.test", 101);
    let (mut bv, _) = fresh(&b, "bob@away.test", 202);
    let ab = av.bundle().unwrap();
    let bb = bv.bundle().unwrap();
    av.enroll(
        "bob@away.test",
        202,
        &format!("ed25519:{}", hex(&bb.identity)),
        NOW,
    )
    .unwrap();
    bv.enroll(
        "alice@home.test",
        101,
        &format!("ed25519:{}", hex(&ab.identity)),
        NOW,
    )
    .unwrap();
    av.start(&bb, NOW).unwrap();
    (a, b, av, bv)
}
#[test]
fn attachment_schema_upgrade_preserves_each_old_format_and_independent_archive_read_flags() {
    for archive in [false, true] {
        for read in [false, true] {
            let (a, _b, mut av, mut bv) = pair();
            let sent = av.queue([1; 16], "subject", "body", NOW, NOW).unwrap();
            bv.receive(&sent, NOW).unwrap();
            let reply = bv.queue([2; 16], "reply", "reply body", NOW, NOW).unwrap();
            let receipt = av.receive(&reply, NOW).unwrap();
            if archive {
                av.enable_archive(NOW).unwrap();
            }
            if read {
                av.enable_read_tracking().unwrap();
                av.mark_read(1, receipt.frame).unwrap();
            }
            let old = av.state.encode().unwrap();
            assert!(old.starts_with(if read {
                READ_SCHEMA
            } else if archive {
                ARCHIVE_SCHEMA
            } else {
                SCHEMA
            }));
            assert!(State::decode(&old).unwrap().encode().unwrap() == old);
            let native = av.state.snapshot.clone();
            let header = av.header.clone();
            let key = Zeroizing::new(*av.key);
            let archive_before = av.state.archive.clone();
            let read_before = av.state.read.clone();
            let legacy = av.state.legacy.encode().unwrap();
            let revision = av.state.revision;
            av.enable_attachments().unwrap();
            assert!(av.state.encode().unwrap().starts_with(b"FACTCP05"));
            assert!(av.state.revision == revision + 1);
            assert!(av.header == header && *av.key == *key && *av.state.snapshot == *native);
            assert!(
                av.state.archive == archive_before
                    && av.state.read == read_before
                    && av.state.legacy.encode().unwrap() == legacy
            );
            assert!(av.pending(NOW).unwrap() == vec![sent]);
            assert!(
                av.history().unwrap().len() == 2
                    && av
                        .history()
                        .unwrap()
                        .iter()
                        .all(|r| r.attachment().is_none())
            );
            let before = a.bytes();
            av.enable_attachments().unwrap();
            assert!(a.bytes() == before);
            drop(av);
            let av = a.reopen();
            assert!(av.inspection().unwrap().attachments_enabled);
            assert!(av.state.archive == archive_before && av.state.read == read_before);
            assert!(av.state.encode().unwrap().starts_with(b"FACTCP05"));
        }
    }
}
#[test]
fn attachment_schema_exact_eof_flags_counts_and_descriptor_bounds_reject() {
    let f = Fixture::new();
    let (mut v, _) = fresh(&f, "alice@home.test", 101);
    v.enable_attachments().unwrap();
    let raw = v.state.encode().unwrap();
    for n in 0..raw.len() {
        assert!(
            State::decode(&raw[..n]).is_err(),
            "truncated schema accepted"
        );
    }
    let mut bad = raw.clone();
    bad.push(0);
    assert!(State::decode(&bad).is_err());
    let mut bad = raw.clone();
    bad[..8].copy_from_slice(b"FACTCP06");
    assert!(State::decode(&bad).is_err());
    for offset in [raw.len() - 2, raw.len() - 1] {
        let mut bad = raw.clone();
        bad[offset] = 2;
        assert!(State::decode(&bad).is_err());
    }
    for (offset, count) in [
        (raw.len() - 10, (MAX_HISTORY + 1) as u32),
        (raw.len() - 6, (MAX_OUTBOX + 1) as u32),
    ] {
        let mut bad = raw.clone();
        bad[offset..offset + 4].copy_from_slice(&count.to_be_bytes());
        assert!(State::decode(&bad).is_err());
    }
    let (_a, _b, mut av, _) = pair();
    av.enable_attachments().unwrap();
    av.queue_attachment([1; 16], "subject", "body", &descriptor(), NOW, NOW)
        .unwrap();
    let mut state = av.state.duplicate().unwrap();
    state.history[0].attachment.as_mut().unwrap().size = MAX_BLOB_BYTES as u64 + 1;
    assert!(state.encode().is_err());
    let mut state = av.state.duplicate().unwrap();
    state.attachments = false;
    assert!(state.encode().is_err());
    let mut state = av.state.duplicate().unwrap();
    state.read = Some(vec![]);
    assert!(state.encode().is_err());
    let mut state = av.state.duplicate().unwrap();
    state.outbox[0].wire.to_device += 1;
    assert!(state.encode().is_err());
}
#[test]
fn attachment_descriptor_fields_reset_backup_recovery_and_fresh_device_preserve_history() {
    let (a, b, mut av, mut bv) = pair();
    av.enable_attachments().unwrap();
    bv.enable_attachments().unwrap();
    bv.enable_read_tracking().unwrap();
    bv.enable_archive(NOW).unwrap();
    let d = descriptor();
    let mut wire = av
        .queue_attachment([1; 16], "subject", "body", &d, NOW, NOW)
        .unwrap();
    wire.sent_at = 0;
    let token = bv.archive_checkpoint(NOW).unwrap();
    let receipt = bv.receive_archive(&token, "accepted", &wire, NOW).unwrap();
    bv.mark_read(0, receipt.frame).unwrap();
    let pending = bv.queue([2; 16], "reply", "body", NOW, NOW).unwrap();
    let backup = Fixture::new();
    bv.backup(&backup.0).unwrap();
    let mut backup_v = backup.reopen();
    assert!(backup_v.inspection().unwrap().history_only);
    assert_descriptor(backup_v.history().unwrap()[0].attachment().unwrap(), &d);
    assert!(backup_v.pending(NOW).is_err());
    backup_v.discard_pending().unwrap();
    backup_v.fresh_device(303, NOW + 1).unwrap();
    assert!(backup_v.inspection().unwrap().attachments_enabled);
    assert_descriptor(backup_v.history().unwrap()[0].attachment().unwrap(), &d);
    assert!(backup_v.state.read.as_ref().unwrap()[0]);
    assert!(backup_v.archive_checkpoint(NOW + 1).is_err());
    let recovery = bv.reset(PASS).unwrap();
    let history_frame = bv.history().unwrap()[0].received_id().unwrap();
    assert!(bv.pending(NOW).unwrap() == vec![pending]);
    let path = b.0.clone();
    drop(bv);
    let mut recovered =
        CheckpointVault::unlock(Store::at(&path, false).unwrap(), &*recovery, true).unwrap();
    assert!(recovered.inspection().unwrap().history_only);
    assert_descriptor(recovered.history().unwrap()[0].attachment().unwrap(), &d);
    assert!(recovered.received_read(0, history_frame).unwrap() == Some(true));
    assert!(recovered.receive_network(&wire, NOW).is_err());
    assert!(recovered.pending(NOW).is_err());
    assert!(recovered.inspection().unwrap().archive_cursor_present);
    recovered.discard_pending().unwrap();
    recovered.fresh_device(404, NOW + 1).unwrap();
    assert!(recovered.inspection().unwrap().attachments_enabled);
    assert_descriptor(recovered.history().unwrap()[0].attachment().unwrap(), &d);
    assert!(
        recovered.inspection().unwrap().archive_enabled
            && !recovered.inspection().unwrap().archive_cursor_present
    );
    drop(av);
    let av = a.reopen();
    assert_descriptor(av.history().unwrap()[0].attachment().unwrap(), &d);
}
#[test]
fn attachment_migration_write_failures_poison_and_only_canonical_reopen_is_usable() {
    for fault in [
        Fault::BeforeWrite,
        Fault::BeforeFileSync,
        Fault::BeforePublish,
        Fault::AfterPublish,
        Fault::DirectorySync,
    ] {
        let f = Fixture::new();
        let (mut v, _) = fresh(&f, "alice@home.test", 101);
        let before = f.bytes();
        v.fail_once(fault);
        assert!(v.enable_attachments().is_err());
        assert!(v.history().is_err() && v.inspection().is_err() && v.enable_attachments().is_err());
        drop(v);
        let v = f.reopen();
        let published = matches!(fault, Fault::AfterPublish | Fault::DirectorySync);
        assert!(v.inspection().unwrap().attachments_enabled == published);
        assert!((f.bytes() != before) == published);
    }
}
#[test]
#[ignore = "invoked only by attachment migration process-death parent"]
fn attachment_migration_crash_child() {
    let path = PathBuf::from(std::env::var_os("FACTION257_TEST_STORE").unwrap());
    assert!(
        path.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("faction257-schema-")
    );
    let mut v = CheckpointVault::unlock(Store::at(&path, false).unwrap(), PASS, false).unwrap();
    let after = std::env::var("FACTION257_TEST_AFTER").unwrap() == "yes";
    v.fail_once(if after {
        Fault::CrashAfterPublish
    } else {
        Fault::CrashBeforePublish
    });
    let _ = v.enable_attachments();
    std::process::exit(76);
}
#[test]
fn attachment_migration_process_death_keeps_only_authenticated_canonical_version() {
    for after in [false, true] {
        let f = Fixture::new();
        let (v, _) = fresh(&f, "alice@home.test", 101);
        let before = f.bytes();
        drop(v);
        let child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "mail_checkpoint::attachment_checkpoint_tests::attachment_migration_crash_child",
                "--ignored",
                "--test-threads=1",
            ])
            .env("FACTION257_TEST_STORE", &f.0)
            .env("FACTION257_TEST_AFTER", if after { "yes" } else { "no" })
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut child = ChildGuard(child);
        let status = bounded_child_wait(&mut child.0, std::time::Duration::from_secs(5)).unwrap();
        assert!(status.code() == Some(75));
        let v = f.reopen();
        assert!(v.inspection().unwrap().attachments_enabled == after);
        assert!((f.bytes() != before) == after);
        assert!(v.history().unwrap().is_empty());
    }
}

#[test]
fn authentic_malformed_attachment_and_wrong_wire_bindings_abort_before_any_durable_change() {
    let (_a, b, mut av, mut bv) = pair();
    av.enable_attachments().unwrap();
    bv.enable_attachments().unwrap();
    let d = descriptor();
    let valid = attachment::encode(
        "alice@home.test",
        "bob@away.test",
        NOW,
        "subject",
        "body",
        &d,
    )
    .unwrap();
    let text = std::str::from_utf8(&valid).unwrap();
    let mut sender = av.state.device().unwrap();
    for (from, to) in [
        ("text/plain", "text/plain;parameter"),
        ("<size>1048576</size>", "<size>1048577</size>"),
        ("<size>1048576</size>", "<size>01</size>"),
        (
            "https://blob.test/private",
            "https://blob.test/private?secret",
        ),
        ("bob@away.test", "other@away.test"),
    ] {
        let changed = Zeroizing::new(text.replace(from, to));
        assert!(changed.as_bytes() != valid.as_slice());
        let message = sender.seal(changed.as_bytes()).unwrap();
        let wire = Wire {
            namespace: "urn:xmpp:omemo:2".into(),
            from: "alice@home.test".into(),
            to: "bob@away.test".into(),
            device: 101,
            to_device: 202,
            identity: av.state.identity,
            sent_at: NOW,
            key: message.key,
            prekey: message.prekey,
            payload: message.payload,
        };
        let before = b.bytes();
        assert!(bv.receive(&wire, NOW).is_err());
        assert!(b.bytes() == before && bv.history().unwrap().is_empty());
    }
    let message = sender.seal(&valid).unwrap();
    let wire = Wire {
        namespace: "urn:xmpp:omemo:2".into(),
        from: "alice@home.test".into(),
        to: "bob@away.test".into(),
        device: 101,
        to_device: 202,
        identity: av.state.identity,
        sent_at: NOW,
        key: message.key,
        prekey: message.prekey,
        payload: message.payload,
    };
    for field in 0..6 {
        let mut bad = wire.clone();
        match field {
            0 => bad.device += 1,
            1 => bad.to_device += 1,
            2 => bad.identity[0] ^= 1,
            3 => bad.key[0] ^= 1,
            4 => bad.sent_at += 301,
            _ => bad.payload[0] ^= 1,
        };
        let before = b.bytes();
        assert!(bv.receive(&bad, NOW).is_err());
        assert!(b.bytes() == before);
    }
    assert!(!bv.receive(&wire, NOW).unwrap().duplicate);
    assert_descriptor(bv.history().unwrap()[0].attachment().unwrap(), &d);
}
#[test]
fn descriptor_binary_contract_roundtrips_and_rejects_each_truncation_and_malformed_field() {
    let d = descriptor();
    let mut raw = Zeroizing::new(Vec::new());
    descriptor_fields(&mut raw, &d);
    let decoded = Reader(&raw).descriptor().unwrap();
    assert_descriptor(&decoded, &d);
    for n in 0..raw.len() {
        assert!(Reader(&raw[..n]).descriptor().is_err());
    }
    let name_end = 4 + d.name.len();
    let media_end = name_end + 4 + d.media_type.len();
    let size_start = media_end;
    let locator_start = size_start + 8 + 32 + 32 + 12;
    for (offset, value) in [(0, 256_u32), (name_end, 128), (locator_start, 513)] {
        let mut bad = raw.clone();
        bad[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
        assert!(Reader(&bad).descriptor().is_err());
    }
    let mut bad = raw.clone();
    bad[size_start..size_start + 8].copy_from_slice(&(MAX_BLOB_BYTES as u64 + 1).to_be_bytes());
    assert!(Reader(&bad).descriptor().is_err());
    let mut bad = raw.clone();
    bad[4] = b'/';
    assert!(Reader(&bad).descriptor().is_err());
    let mut zero = descriptor();
    zero.size = 0;
    let mut raw = Zeroizing::new(Vec::new());
    descriptor_fields(&mut raw, &zero);
    assert_descriptor(&Reader(&raw).descriptor().unwrap(), &zero);
}

#[test]
fn every_old_schema_rejects_authentic_attachments_without_advancing_native_history_or_cursor() {
    for (archive, read) in [(false, false), (true, false), (false, true), (true, true)] {
        let (_a, b, mut av, mut bv) = pair();
        av.enable_attachments().unwrap();
        if archive {
            bv.enable_archive(NOW).unwrap();
        }
        if read {
            bv.enable_read_tracking().unwrap();
        }
        let expected = if read {
            READ_SCHEMA
        } else if archive {
            ARCHIVE_SCHEMA
        } else {
            SCHEMA
        };
        assert!(bv.state.encode().unwrap().starts_with(expected));
        let wire = av
            .queue_attachment([1; 16], "subject", "body", &descriptor(), NOW, NOW)
            .unwrap();
        let before = b.bytes();
        let native = bv.state.snapshot.clone();
        let cursor = bv.state.archive.clone();
        assert!(bv.receive(&wire, NOW).is_err());
        let mut network = wire;
        network.sent_at = 0;
        assert!(bv.receive_network(&network, NOW).is_err());
        if archive {
            let token = bv.archive_checkpoint(NOW).unwrap();
            assert!(
                bv.receive_archive(&token, "rejected", &network, NOW)
                    .is_err()
            );
        }
        assert!(
            b.bytes() == before
                && *bv.state.snapshot == *native
                && bv.state.archive == cursor
                && bv.history().unwrap().is_empty()
        );
        let text = av
            .queue([2; 16], "ordinary subject", "ordinary body", NOW, NOW)
            .unwrap();
        assert!(!bv.receive(&text, NOW).unwrap().duplicate);
        assert!(bv.history().unwrap()[0].attachment().is_none());
    }
}
#[test]
fn attachment_retry_intent_binds_current_peer_address_device_and_full_identity() {
    let (a, _b, mut av, _bv) = pair();
    av.enable_attachments().unwrap();
    let d = descriptor();
    av.queue_attachment([1; 16], "subject", "body", &d, NOW, NOW)
        .unwrap();
    let before = a.bytes();
    let intent = av.attachment_intent("subject", "body", &d, NOW).unwrap();
    for field in 0..3 {
        let original = av.state.duplicate().unwrap();
        let peer = av.state.peer.as_mut().unwrap();
        match field {
            0 => peer.address = "other@away.test".into(),
            1 => peer.device += 1,
            _ => peer.identity[31] ^= 1,
        };
        assert!(av.attachment_intent("subject", "body", &d, NOW).unwrap() != intent);
        assert!(
            av.queue_attachment([1; 16], "subject", "body", &d, NOW, NOW)
                .is_err()
        );
        assert!(a.bytes() == before);
        av.state = original;
    }
    assert!(
        av.queue_attachment([1; 16], "subject", "body", &d, NOW, NOW)
            .is_ok()
    );
    assert!(a.bytes() == before);
}
fn first_descriptor_offset(raw: &[u8]) -> usize {
    let mut r = Reader(raw);
    r.take(8).unwrap();
    r.big().unwrap();
    r.bytes(crate::mail_data::MAX_PLAIN).unwrap();
    r.number().unwrap();
    r.take(32).unwrap();
    r.bytes(32768).unwrap();
    if r.boolean().unwrap() {
        r.bytes(254).unwrap();
        r.number().unwrap();
        r.take(32).unwrap();
        r.boolean().unwrap();
    }
    r.boolean().unwrap();
    r.big().unwrap();
    r.big().unwrap();
    r.boolean().unwrap();
    assert!(r.number().unwrap() == 1);
    r.boolean().unwrap();
    r.bytes(254).unwrap();
    r.number().unwrap();
    r.take(32).unwrap();
    r.big().unwrap();
    r.bytes(128).unwrap();
    r.bytes(1024).unwrap();
    r.take(16 + 32 + 32).unwrap();
    raw.len() - r.0.len()
}
#[test]
fn whole_attachment_state_rejects_record_flag_and_nonzero_malformed_descriptor_bytes() {
    let (_a, _b, mut av, _bv) = pair();
    av.enable_attachments().unwrap();
    let d = descriptor();
    av.queue_attachment([1; 16], "subject", "body", &d, NOW, NOW)
        .unwrap();
    let raw = av.state.encode().unwrap();
    let flag = first_descriptor_offset(&raw);
    assert!(raw[flag] == 1);
    let name = flag + 1;
    let media = name + 4 + d.name.len();
    let size = media + 4 + d.media_type.len();
    let locator = size + 8 + 32 + 32 + 12;
    let mut bad = raw.clone();
    bad[flag] = 2;
    assert!(State::decode(&bad).is_err());
    for (offset, n) in [(name, 256_u32), (media, 128), (locator, 513)] {
        let mut bad = raw.clone();
        bad[offset..offset + 4].copy_from_slice(&n.to_be_bytes());
        assert!(State::decode(&bad).is_err());
    }
    let mut bad = raw.clone();
    bad[name + 4] = b'/';
    assert!(State::decode(&bad).is_err());
    let mut bad = raw.clone();
    bad[media + 4] = b'/';
    assert!(State::decode(&bad).is_err());
    let mut bad = raw.clone();
    bad[size..size + 8].copy_from_slice(&(MAX_BLOB_BYTES as u64 + 1).to_be_bytes());
    assert!(State::decode(&bad).is_err());
    let mut bad = raw.clone();
    bad[locator + 4] = b'x';
    assert!(State::decode(&bad).is_err());
    for end in 0..raw.len() {
        assert!(
            State::decode(&raw[..end]).is_err(),
            "truncated attachment state accepted"
        );
    }
    let decoded = State::decode(&raw).unwrap();
    assert_descriptor(decoded.history[0].attachment.as_ref().unwrap(), &d);
}
#[test]
fn original_recovery_secret_remains_valid_after_explicit_attachment_upgrade() {
    let f = Fixture::new();
    let (mut v, recovery) = fresh(&f, "alice@home.test", 101);
    v.enable_attachments().unwrap();
    drop(v);
    let v = CheckpointVault::unlock(f.store(false), &*recovery, true).unwrap();
    assert!(v.inspection().unwrap().attachments_enabled && v.inspection().unwrap().history_only);
    assert!(v.pending(NOW).is_err());
}

struct ChildGuard(std::process::Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[test]
#[ignore = "invoked only by bounded attachment child-wait regression"]
fn attachment_stalled_child() {
    let root = PathBuf::from(std::env::var_os("FACTION257_STALLED_CHILD").unwrap());
    assert!(
        root.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("faction257-schema-")
    );
    fs::write(root.join("READY"), b"synthetic stalled child ready").unwrap();
    std::thread::sleep(std::time::Duration::from_secs(30));
}
#[test]
fn attachment_child_wait_times_out_and_reaps_observed_stalled_child() {
    use std::{
        os::unix::{fs::DirBuilderExt, process::ExitStatusExt},
        time::{Duration, Instant},
    };
    let f = Fixture::new();
    fs::DirBuilder::new().mode(0o700).create(&f.0).unwrap();
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "mail_checkpoint::attachment_checkpoint_tests::attachment_stalled_child",
            "--ignored",
            "--test-threads=1",
        ])
        .env("FACTION257_STALLED_CHILD", &f.0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut child = ChildGuard(child);
    let ready_deadline = Instant::now() + Duration::from_secs(5);
    while !f.0.join("READY").exists() {
        assert!(child.0.try_wait().unwrap().is_none());
        assert!(
            Instant::now() < ready_deadline,
            "stalled child readiness deadline expired"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let started = Instant::now();
    let error = bounded_child_wait(&mut child.0, Duration::from_millis(100)).unwrap_err();
    assert!(error.kind() == io::ErrorKind::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(5));
    let status = child
        .0
        .try_wait()
        .unwrap()
        .expect("timed-out child must be reaped");
    assert!(status.signal() == Some(9));
    assert!(child.0.wait().unwrap() == status);
}

fn bounded_child_wait(
    child: &mut std::process::Child,
    timeout: std::time::Duration,
) -> io::Result<std::process::ExitStatus> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if std::time::Instant::now() >= deadline {
            child.kill()?;
            child.wait()?;
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "bounded attachment child wait expired",
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
