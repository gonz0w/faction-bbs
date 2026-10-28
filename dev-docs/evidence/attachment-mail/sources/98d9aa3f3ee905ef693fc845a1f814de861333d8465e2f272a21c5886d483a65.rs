#![cfg(feature = "native-mail")]
use faction_client::{
    attachment_save::SaveError,
    mail_attachments::{Error, save_received, verify_received},
    mail_checkpoint::{CheckpointVault, SESSION_AGE, Wire},
    mail_data::Data,
    mail_vault::Store,
};
use faction_omemo::{attachment::Descriptor, blob};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, MetadataExt},
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;
const NOW: u64 = 1_792_886_400;
const PASS: &[u8] = b"synthetic issue 258 bridge password";
const BYTES: &[u8] = b"retained authenticated attachment bytes\0\xff";
const OWNER: &[u8] = b"Owned issue 258 disposable bridge fixture";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let mut nonce = [0; 16];
        getrandom::fill(&mut nonce).unwrap();
        let suffix = nonce.iter().map(|v| format!("{v:02x}")).collect::<String>();
        let root = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("faction258-bridge-{suffix}"));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        fs::write(root.join("OWNER"), OWNER).unwrap();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(root.join("selected"))
            .unwrap();
        Self(root)
    }
    fn create(&self, name: &str, home: &str, device: u32) -> CheckpointVault {
        CheckpointVault::create(
            Store::at(&self.0.join(name), true).unwrap(),
            Data::new(home.into()).unwrap(),
            PASS,
            device,
            NOW,
        )
        .unwrap()
        .0
    }
    fn reopen(&self, name: &str) -> CheckpointVault {
        CheckpointVault::unlock(Store::at(&self.0.join(name), false).unwrap(), PASS, false).unwrap()
    }
    fn destination(&self) -> PathBuf {
        self.0.join("selected/explicit-choice")
    }
    fn untouched(&self) {
        assert_eq!(fs::read_dir(self.0.join("selected")).unwrap().count(), 0);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert!(fs::read(self.0.join("OWNER")).unwrap() == OWNER);
        fs::remove_dir_all(&self.0).unwrap();
        assert!(!self.0.exists());
    }
}
fn pair(f: &Fixture) -> (CheckpointVault, CheckpointVault) {
    let mut alice = f.create("alice", "alice@home.test", 101);
    let mut bob = f.create("bob", "bob@away.test", 202);
    alice.enable_attachments().unwrap();
    bob.enable_attachments().unwrap();
    let ab = alice.bundle().unwrap();
    let bb = bob.bundle().unwrap();
    alice
        .enroll(
            "bob@away.test",
            202,
            &format!("ed25519:{}", faction_omemo::encode(&bb.identity)),
            NOW,
        )
        .unwrap();
    bob.enroll(
        "alice@home.test",
        101,
        &format!("ed25519:{}", faction_omemo::encode(&ab.identity)),
        NOW,
    )
    .unwrap();
    alice.start(&bb, NOW).unwrap();
    (alice, bob)
}
fn encrypted() -> (Descriptor, Vec<u8>) {
    let e = blob::encrypt(BYTES).unwrap();
    (
        Descriptor {
            name: Zeroizing::new("remote-name-never-selects-path.bin".into()),
            media_type: Zeroizing::new("application/octet-stream".into()),
            size: e.size,
            hash: e.hash,
            key: e.key,
            iv: e.iv,
            locator: Zeroizing::new("https://blob.invalid/inert-never-fetch".into()),
        },
        e.ciphertext,
    )
}
fn received(bob: &CheckpointVault, index: usize) -> [u8; 32] {
    bob.history().unwrap()[index].received_id().unwrap()
}
fn queue(alice: &mut CheckpointVault, id: u8, descriptor: &Descriptor) -> Wire {
    alice
        .queue_attachment(
            [id; 16],
            "attachment subject",
            "attachment body",
            descriptor,
            NOW,
            NOW,
        )
        .unwrap()
}
fn saved(f: &Fixture, bob: &CheckpointVault, frame: [u8; 32], bytes: &[u8]) {
    save_received(bob, 0, frame, bytes, &f.destination()).unwrap();
    assert!(fs::read(f.destination()).unwrap() == BYTES);
    assert_eq!(
        fs::metadata(f.destination()).unwrap().mode() & 0o7777,
        0o600
    );
    assert!(
        !f.0.join("selected/remote-name-never-selects-path.bin")
            .exists()
    );
}
#[test]
fn durable_received_history_binds_real_blob_verification_and_explicit_private_save() {
    let f = Fixture::new();
    let (mut alice, mut bob) = pair(&f);
    let (descriptor, ciphertext) = encrypted();
    let wire = queue(&mut alice, 1, &descriptor);
    drop(alice);
    let mut alice = f.reopen("alice");
    assert!(alice.pending(NOW).unwrap() == [wire.clone()]);
    assert!(queue(&mut alice, 1, &descriptor) == wire);
    #[cfg(feature = "network-mail")]
    let wire = {
        let xml = faction_client::mail_wire::message(&wire, [1; 16]).unwrap();
        faction_client::mail_wire::decode_message(
            &xml,
            faction_client::mail_wire::Binding {
                peer: "alice@home.test",
                peer_device: 101,
                peer_identity: wire.identity,
                own: "bob@away.test",
                own_device: 202,
            },
        )
        .unwrap()
    };
    #[cfg(not(feature = "network-mail"))]
    let wire = Wire { sent_at: 0, ..wire };
    assert!(!bob.receive_network(&wire, NOW).unwrap().duplicate);
    let frame = received(&bob, 0);
    drop(bob);
    let mut bob = f.reopen("bob");
    assert!(bob.receive_network(&wire, NOW).unwrap().duplicate);
    assert_eq!(bob.history().unwrap().len(), 1);
    assert!(
        verify_received(&bob, 0, frame, &ciphertext)
            .unwrap()
            .as_bytes()
            == BYTES
    );
    saved(&f, &bob, frame, &ciphertext);
    assert_eq!(
        save_received(&bob, 0, frame, &ciphertext, &f.destination()),
        Err(Error::Save(SaveError::BeforePublication))
    );
    assert!(fs::read(f.destination()).unwrap() == BYTES);
    alice.acknowledge([1; 16]).unwrap();
    assert!(
        alice
            .queue_attachment(
                [1; 16],
                "attachment subject",
                "attachment body",
                &descriptor,
                NOW,
                NOW
            )
            .is_err()
    );
}
#[test]
fn selection_must_match_retained_received_attachment_before_any_file_action() {
    let f = Fixture::new();
    let (mut alice, mut bob) = pair(&f);
    let (descriptor, ciphertext) = encrypted();
    let wire = queue(&mut alice, 2, &descriptor);
    bob.receive(&wire, NOW).unwrap();
    let frame = received(&bob, 0);
    let ordinary = alice
        .queue([3; 16], "plain subject", "plain body", NOW, NOW)
        .unwrap();
    bob.receive(&ordinary, NOW).unwrap();
    for (index, expected) in [(usize::MAX, frame), (0, [0; 32]), (1, received(&bob, 1))] {
        assert_eq!(
            save_received(&bob, index, expected, &ciphertext, &f.destination()),
            Err(Error::Record)
        );
        f.untouched();
    }
    assert_eq!(
        save_received(&alice, 0, frame, &ciphertext, &f.destination()),
        Err(Error::Record)
    );
    let legacy = f.create("legacy", "legacy@home.test", 303);
    assert_eq!(
        save_received(&legacy, 0, frame, &ciphertext, &f.destination()),
        Err(Error::State)
    );
    f.untouched();
    // A deliberate canceled destination performs no save call, even after verification.
    let _verified = verify_received(&bob, 0, frame, &ciphertext).unwrap();
    let selection: Option<&Path> = None;
    if let Some(destination) = selection {
        save_received(&bob, 0, frame, &ciphertext, destination).unwrap();
    }
    f.untouched();
}
#[test]
fn corrupt_wrong_truncated_and_oversize_bytes_never_reach_a_save() {
    let f = Fixture::new();
    let (mut alice, mut bob) = pair(&f);
    let (descriptor, ciphertext) = encrypted();
    bob.receive(&queue(&mut alice, 4, &descriptor), NOW)
        .unwrap();
    let frame = received(&bob, 0);
    let mut damaged = ciphertext.clone();
    damaged[0] ^= 1;
    let wrong = blob::encrypt(BYTES).unwrap().ciphertext;
    let truncated = ciphertext[..ciphertext.len() - 1].to_vec();
    let oversize = vec![0; blob::MAX_BLOB_BYTES + 17];
    for bytes in [&damaged, &wrong, &truncated, &oversize, &vec![]] {
        assert_eq!(
            save_received(&bob, 0, frame, bytes, &f.destination()),
            Err(Error::Blob)
        );
        f.untouched();
    }
    assert_eq!(
        save_received(&bob, 0, frame, &ciphertext, Path::new("relative")),
        Err(Error::Save(SaveError::BeforePublication))
    );
    f.untouched();
}
#[test]
fn accepted_hash_and_size_assertions_are_still_checked_against_supplied_bulk_bytes() {
    let f = Fixture::new();
    let (mut alice, mut bob) = pair(&f);
    for (index, failure) in ["hash", "size"].into_iter().enumerate() {
        let (mut descriptor, ciphertext) = encrypted();
        if failure == "hash" {
            descriptor.hash[0] ^= 1;
        } else {
            descriptor.size += 1;
        }
        bob.receive(&queue(&mut alice, 5 + index as u8, &descriptor), NOW)
            .unwrap();
        assert_eq!(
            save_received(
                &bob,
                index,
                received(&bob, index),
                &ciphertext,
                &f.destination()
            ),
            Err(Error::Blob)
        );
        f.untouched();
    }
}
#[test]
fn retained_history_remains_consumable_after_expiry_revocation_recovery_and_fresh_device() {
    let f = Fixture::new();
    let (mut alice, mut bob) = pair(&f);
    let (descriptor, ciphertext) = encrypted();
    let wire = queue(&mut alice, 7, &descriptor);
    bob.receive(&wire, NOW).unwrap();
    let frame = received(&bob, 0);
    let expired = NOW + SESSION_AGE + 1;
    assert!(
        bob.queue_attachment([8; 16], "new", "new", &descriptor, expired, expired)
            .is_err()
    );
    assert!(bob.receive(&wire, expired).is_err());
    assert!(verify_received(&bob, 0, frame, &ciphertext).is_ok());
    bob.revoke().unwrap();
    assert!(bob.receive(&wire, NOW).is_err());
    assert!(
        bob.queue_attachment([8; 16], "new", "new", &descriptor, NOW, NOW)
            .is_err()
    );
    assert!(verify_received(&bob, 0, frame, &ciphertext).is_ok());
    let recovery = bob.reset(PASS).unwrap();
    bob.backup(&f.0.join("backup")).unwrap();
    let backup = f.reopen("backup");
    assert!(backup.inspection().unwrap().history_only);
    assert!(backup.pending(NOW).is_err());
    assert!(backup.publication(NOW).is_err());
    saved(&f, &backup, frame, &ciphertext);
    drop(backup);
    drop(bob);
    let mut bob = CheckpointVault::unlock(
        Store::at(&f.0.join("bob"), false).unwrap(),
        &*recovery,
        true,
    )
    .unwrap();
    assert!(bob.inspection().unwrap().history_only);
    assert!(bob.receive(&wire, NOW).is_err());
    assert!(
        bob.queue_attachment([8; 16], "new", "new", &descriptor, NOW, NOW)
            .is_err()
    );
    assert!(verify_received(&bob, 0, frame, &ciphertext).is_ok());
    bob.fresh_device(404, NOW).unwrap();
    assert!(
        bob.queue_attachment([8; 16], "new", "new", &descriptor, NOW, NOW)
            .is_err()
    );
    drop(bob);
    let bob = f.reopen("bob");
    assert!(
        verify_received(&bob, 0, frame, &ciphertext)
            .unwrap()
            .as_bytes()
            == BYTES
    );
}
#[test]
fn diagnostics_are_fixed_and_preserve_save_publication_phase() {
    for (error, expected) in [
        (Error::State, "attachment history unavailable"),
        (Error::Record, "attachment selection unavailable"),
        (Error::Blob, "attachment verification failed"),
        (
            Error::Save(SaveError::BeforePublication),
            "attachment save failed before publication",
        ),
        (
            Error::Save(SaveError::PublicationUncertain),
            "attachment publication may have completed",
        ),
    ] {
        assert_eq!(error.to_string(), expected);
    }
}
