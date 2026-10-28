#![cfg(feature = "native-mail")]
use faction_client::{
    mail_checkpoint::{CheckpointVault, SESSION_AGE, Wire},
    mail_data::{Data, hex},
    mail_vault::Store,
};
use faction_omemo::{attachment::Descriptor, blob};
use std::{fs, path::PathBuf};
use zeroize::Zeroizing;
const NOW: u64 = 1_792_886_400;
const PASS: &[u8] = b"synthetic attachment checkpoint password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let mut id = [0; 16];
        getrandom::fill(&mut id).unwrap();
        Self(
            std::env::temp_dir()
                .canonicalize()
                .unwrap()
                .join(format!("faction258-core-{}", hex(&id))),
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
fn pair() -> (Fixture, Fixture, CheckpointVault, CheckpointVault) {
    let a = Fixture::new();
    let b = Fixture::new();
    let mut av = CheckpointVault::create(
        a.store(true),
        Data::new("alice@home.test".into()).unwrap(),
        PASS,
        101,
        NOW,
    )
    .unwrap()
    .0;
    let mut bv = CheckpointVault::create(
        b.store(true),
        Data::new("bob@away.test".into()).unwrap(),
        PASS,
        202,
        NOW,
    )
    .unwrap()
    .0;
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
fn descriptor() -> Descriptor {
    Descriptor {
        name: Zeroizing::new("private-name.txt".into()),
        media_type: Zeroizing::new("text/plain".into()),
        size: 0,
        hash: Zeroizing::new([3; 32]),
        key: Zeroizing::new([4; 32]),
        iv: Zeroizing::new([5; 12]),
        locator: Zeroizing::new("https://blob.test/opaque".into()),
    }
}
fn equal(a: &Descriptor, b: &Descriptor) {
    assert!(
        *a.name == *b.name
            && *a.media_type == *b.media_type
            && a.size == b.size
            && *a.hash == *b.hash
            && *a.key == *b.key
            && *a.iv == *b.iv
            && *a.locator == *b.locator,
        "descriptor fields changed"
    );
}
fn queued(v: &mut CheckpointVault, d: &Descriptor) -> Wire {
    v.queue_attachment([1; 16], "private subject", "private body", d, NOW, NOW)
        .unwrap()
}
#[test]
fn attachment_queue_requires_opt_in_then_retries_exact_durable_bytes_and_binds_every_field() {
    let (a, _b, mut av, _bv) = pair();
    let d = descriptor();
    let before = a.bytes();
    assert!(
        av.queue_attachment([1; 16], "private subject", "private body", &d, NOW, NOW)
            .is_err()
    );
    assert!(a.bytes() == before);
    av.enable_attachments().unwrap();
    let wire = queued(&mut av, &d);
    let canonical = a.bytes();
    assert!(queued(&mut av, &d) == wire);
    assert!(a.bytes() == canonical);
    for field in 0..7 {
        let mut changed = descriptor();
        match field {
            0 => changed.name.push('x'),
            1 => changed.media_type.push('x'),
            2 => changed.size = 1,
            3 => changed.hash[0] ^= 1,
            4 => changed.key[0] ^= 1,
            5 => changed.iv[0] ^= 1,
            _ => changed.locator.push('x'),
        };
        assert!(
            av.queue_attachment(
                [1; 16],
                "private subject",
                "private body",
                &changed,
                NOW,
                NOW
            )
            .is_err()
        );
        assert!(a.bytes() == canonical);
    }
    for (subject, body, time) in [
        ("changed", "private body", NOW),
        ("private subject", "changed", NOW),
        ("private subject", "private body", NOW + 1),
    ] {
        assert!(
            av.queue_attachment([1; 16], subject, body, &d, time, NOW)
                .is_err()
        );
    }
    assert!(
        av.queue([1; 16], "private subject", "private body", NOW, NOW)
            .is_err()
    );
    av.queue([2; 16], "private subject", "private body", NOW, NOW)
        .unwrap();
    let before = a.bytes();
    assert!(
        av.queue_attachment([2; 16], "private subject", "private body", &d, NOW, NOW)
            .is_err()
    );
    assert!(a.bytes() == before);
    drop(av);
    let mut av = a.reopen();
    assert!(queued(&mut av, &d) == wire);
    equal(av.history().unwrap()[0].attachment().unwrap(), &d);
    av.acknowledge([1; 16]).unwrap();
    let before = a.bytes();
    assert!(
        av.queue_attachment([1; 16], "private subject", "private body", &d, NOW, NOW)
            .is_err()
    );
    assert!(a.bytes() == before);
    for marker in [
        b"private-name".as_slice(),
        b"private subject",
        b"private body",
        b"https://blob.test",
    ] {
        assert!(!a.bytes().windows(marker.len()).any(|b| b == marker));
    }
}
#[test]
fn authenticated_attachment_live_network_duplicates_and_historical_keys_survive_reopen() {
    let (a, b, mut av, mut bv) = pair();
    let d = descriptor();
    av.enable_attachments().unwrap();
    let wire = queued(&mut av, &d);
    let before = b.bytes();
    assert!(bv.receive(&wire, NOW).is_err());
    assert!(b.bytes() == before);
    bv.enable_attachments().unwrap();
    bv.enable_read_tracking().unwrap();
    let mut network = wire.clone();
    network.sent_at = 0;
    let receipt = bv.receive_network(&network, NOW).unwrap();
    assert!(!receipt.duplicate);
    equal(bv.history().unwrap()[0].attachment().unwrap(), &d);
    assert!(!bv.received_read(0, receipt.frame).unwrap().unwrap());
    bv.mark_read(0, receipt.frame).unwrap();
    drop(bv);
    let mut bv = b.reopen();
    let before = b.bytes();
    assert!(bv.receive_network(&network, NOW + 500).unwrap().duplicate);
    assert!(b.bytes() == before);
    assert!(bv.received_read(0, receipt.frame).unwrap().unwrap());
    bv.revoke().unwrap();
    equal(bv.history().unwrap()[0].attachment().unwrap(), &d);
    assert!(bv.receive_network(&network, NOW).is_err());
    assert!(bv.pending(NOW).is_err());
    av.discard_pending().unwrap();
    av.fresh_device(303, NOW + 1).unwrap();
    assert!(av.inspection().unwrap().attachments_enabled);
    equal(av.history().unwrap()[0].attachment().unwrap(), &d);
    assert!(av.pending(NOW + 1).is_err());
    drop(av);
    let av = a.reopen();
    equal(av.history().unwrap()[0].attachment().unwrap(), &d);
    assert!(bv.receive_network(&network, NOW + SESSION_AGE + 1).is_err());
}
#[test]
fn actual_zero_and_cap_blob_descriptors_are_accepted_but_oversize_and_invalid_metadata_do_not_mutate()
 {
    let (_a, b, mut av, mut bv) = pair();
    av.enable_attachments().unwrap();
    bv.enable_attachments().unwrap();
    for (i, size) in [0, blob::MAX_BLOB_BYTES].into_iter().enumerate() {
        let bytes = Zeroizing::new(vec![91; size]);
        let encrypted = blob::encrypt(&bytes).unwrap();
        let d = Descriptor {
            key: encrypted.key,
            iv: encrypted.iv,
            hash: encrypted.hash,
            size: encrypted.size,
            ..descriptor()
        };
        let wire = av
            .queue_attachment([i as u8 + 1; 16], "subject", "body", &d, NOW, NOW)
            .unwrap();
        bv.receive(&wire, NOW).unwrap();
        let stored = bv.history().unwrap()[i].attachment().unwrap();
        assert!(
            blob::verify(&encrypted.ciphertext, stored)
                .unwrap()
                .as_bytes()
                == bytes.as_slice()
        );
    }
    for variant in 0..4 {
        let mut d = descriptor();
        match variant {
            0 => d.size = blob::MAX_BLOB_BYTES as u64 + 1,
            1 => d.name = Zeroizing::new("../bad".into()),
            2 => d.locator = Zeroizing::new("https://blob.test/p?secret".into()),
            _ => d.media_type = Zeroizing::new("text/plain;extra".into()),
        };
        let before = b.bytes();
        assert!(
            bv.queue_attachment([9; 16], "subject", "body", &d, NOW, NOW)
                .is_err()
        );
        assert!(b.bytes() == before);
    }
}
#[test]
fn attachment_archive_commits_descriptor_cursor_and_known_own_echo_without_retirement() {
    let (a, b, mut av, mut bv) = pair();
    av.enable_attachments().unwrap();
    bv.enable_attachments().unwrap();
    av.enable_archive(NOW).unwrap();
    bv.enable_archive(NOW).unwrap();
    bv.enable_read_tracking().unwrap();
    let d = descriptor();
    let mut wire = queued(&mut av, &d);
    wire.sent_at = 0;
    let token = bv.archive_checkpoint(NOW).unwrap();
    let receipt = bv.receive_archive(&token, "one", &wire, NOW).unwrap();
    assert!(!receipt.duplicate);
    drop(bv);
    let mut bv = b.reopen();
    let token = bv.archive_checkpoint(NOW).unwrap();
    assert!(token.after.as_deref() == Some("one"));
    assert!(
        bv.receive_archive(&token, "two", &wire, NOW)
            .unwrap()
            .duplicate
    );
    assert!(bv.history().unwrap().len() == 1);
    equal(bv.history().unwrap()[0].attachment().unwrap(), &d);
    let token = av.archive_checkpoint(NOW).unwrap();
    assert!(
        av.receive_archive(&token, "echo", &wire, NOW)
            .unwrap()
            .duplicate
    );
    assert!(av.pending(NOW).unwrap().len() == 1);
    drop(av);
    let av = a.reopen();
    assert!(av.archive_checkpoint(NOW).unwrap().after.as_deref() == Some("echo"));
    let before = b.bytes();
    let mut damaged = wire;
    damaged.payload[0] ^= 1;
    let token = bv.archive_checkpoint(NOW).unwrap();
    assert!(bv.receive_archive(&token, "three", &damaged, NOW).is_err());
    assert!(b.bytes() == before);
}
