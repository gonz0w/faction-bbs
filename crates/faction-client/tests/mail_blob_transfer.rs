#![cfg(feature = "network-mail")]
use faction_client::{
    mail_blob_transfer::{self, Selection, Spool},
    mail_checkpoint::CheckpointVault,
    mail_data::Data,
    mail_vault::Store,
};
use std::{fs, os::unix::fs::DirBuilderExt, path::PathBuf};
const NOW: u64 = 1_792_886_400;
const PASS: &[u8] = b"owned synthetic transfer password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let mut r = [0; 16];
        getrandom::fill(&mut r).unwrap();
        let p = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "faction261-local-{}",
            faction_client::mail_data::hex(&r)
        ));
        fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        fs::write(p.join("OWNED"), b"Owned transfer test fixture").unwrap();
        Self(p)
    }
    fn vault(&self, name: &str, home: &str, id: u32) -> CheckpointVault {
        CheckpointVault::create(
            Store::at(&self.0.join(name), true).unwrap(),
            Data::new(home.into()).unwrap(),
            PASS,
            id,
            NOW,
        )
        .unwrap()
        .0
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert_eq!(
            fs::read(self.0.join("OWNED")).unwrap(),
            b"Owned transfer test fixture"
        );
        fs::remove_dir_all(&self.0).unwrap();
        assert!(!self.0.exists());
    }
}
fn selection<'a>(bytes: &'a [u8]) -> Selection<'a> {
    Selection {
        name: "secret-name.bin",
        media_type: "application/octet-stream",
        bytes,
        subject: "secret-subject",
        body: "secret-body",
        sent_at: NOW,
    }
}
#[test]
fn preparation_spools_before_queue_and_reopens_exact_intent_without_reencrypting() {
    let f = Fixture::new();
    let mut a = f.vault("alice", "alice@home.test", 101);
    let mut b = f.vault("bob", "bob@away.test", 202);
    a.enable_attachments().unwrap();
    b.enable_attachments().unwrap();
    let bb = b.bundle().unwrap();
    a.enroll(
        "bob@away.test",
        202,
        &format!("ed25519:{}", faction_omemo::encode(&bb.identity)),
        NOW,
    )
    .unwrap();
    a.start(&bb, NOW).unwrap();
    let mut spool = Spool::open(&f.0.join("spool")).unwrap();
    let first = mail_blob_transfer::prepare(
        &mut a,
        &mut spool,
        [1; 16],
        selection(b"actual private bytes"),
        NOW,
    )
    .unwrap();
    let descriptor = a.history().unwrap()[0].attachment().unwrap();
    assert!(descriptor.locator.starts_with("urn:faction:blob:1:"));
    let locator = descriptor.locator.to_string();
    drop(a);
    drop(spool);
    let mut a = CheckpointVault::unlock(Store::at(&f.0.join("alice"), false).unwrap(), PASS, false)
        .unwrap();
    let mut spool = Spool::open(&f.0.join("spool")).unwrap();
    assert!(
        mail_blob_transfer::prepare(
            &mut a,
            &mut spool,
            [1; 16],
            selection(b"actual private bytes"),
            NOW
        )
        .unwrap()
            == first
    );
    assert_eq!(
        a.history().unwrap()[0]
            .attachment()
            .unwrap()
            .locator
            .as_str(),
        locator
    );
    assert!(
        mail_blob_transfer::prepare(
            &mut a,
            &mut spool,
            [1; 16],
            selection(b"changed bytes"),
            NOW
        )
        .is_err()
    );
    let mut changed = selection(b"actual private bytes");
    changed.name = "changed.bin";
    assert!(mail_blob_transfer::prepare(&mut a, &mut spool, [1; 16], changed, NOW).is_err());
    let mut changed = selection(b"actual private bytes");
    changed.subject = "changed";
    assert!(mail_blob_transfer::prepare(&mut a, &mut spool, [1; 16], changed, NOW).is_err());
    let mut changed = selection(b"actual private bytes");
    changed.sent_at += 1;
    assert!(mail_blob_transfer::prepare(&mut a, &mut spool, [1; 16], changed, NOW).is_err());
    assert!(
        mail_blob_transfer::prepare(
            &mut a,
            &mut spool,
            [0; 16],
            selection(b"invalid zero operation"),
            NOW
        )
        .is_err()
    );
    for entry in fs::read_dir(f.0.join("spool")).unwrap() {
        let p = entry.unwrap().path();
        if p.is_file() {
            let bytes = fs::read(p).unwrap();
            for secret in [
                b"actual private bytes".as_slice(),
                b"secret-name.bin",
                b"secret-subject",
                b"secret-body",
            ] {
                assert!(!bytes.windows(secret.len()).any(|v| v == secret));
            }
        }
    }
    assert_eq!(a.pending(NOW).unwrap().len(), 1);
    let second = mail_blob_transfer::prepare(
        &mut a,
        &mut spool,
        [2; 16],
        selection(b"actual private bytes"),
        NOW,
    )
    .unwrap();
    assert!(second != first);
    assert_ne!(
        a.history().unwrap()[0]
            .attachment()
            .unwrap()
            .locator
            .as_str(),
        a.history().unwrap()[1]
            .attachment()
            .unwrap()
            .locator
            .as_str()
    );
    a.revoke().unwrap();
    assert!(
        mail_blob_transfer::prepare(
            &mut a,
            &mut spool,
            [1; 16],
            selection(b"actual private bytes"),
            NOW
        )
        .is_err()
    );
}
#[test]
fn explicit_invited_domains_and_invalid_local_selection_are_refused() {
    use mail_blob_transfer::InvitedHomes;
    assert!(InvitedHomes::new(&["home.test"]).is_ok());
    for home in [
        "https://home.test",
        "alice@home.test",
        "evil.test/path",
        "HOME.test",
        "",
        "home.test:123",
    ] {
        assert!(InvitedHomes::new(&[home]).is_err());
    }
    let f = Fixture::new();
    let a = f.vault("alice", "alice@home.test", 101);
    let mut spool = Spool::open(&f.0.join("spool")).unwrap();
    assert!(mail_blob_transfer::verify_fetched(&a, &mut spool, 0, [0; 32], NOW).is_err());
}

#[test]
fn a_failed_queue_retains_only_bounded_ciphertext_orphans_and_preserves_existing_pending_mail() {
    let f = Fixture::new();
    let mut a = f.vault("alice", "alice@home.test", 101);
    let b = f.vault("bob", "bob@away.test", 202);
    a.enable_attachments().unwrap();
    let bb = b.bundle().unwrap();
    a.enroll(
        "bob@away.test",
        202,
        &format!("ed25519:{}", faction_omemo::encode(&bb.identity)),
        NOW,
    )
    .unwrap();
    a.start(&bb, NOW).unwrap();
    for i in 1..=8 {
        a.queue([i; 16], "existing text", "keep pending", NOW, NOW)
            .unwrap();
    }
    let pending = a.pending(NOW).unwrap();
    let checkpoint = fs::read(f.0.join("alice/mail.fkv")).unwrap();
    for attempt in 0..=8 {
        let mut spool = Spool::open(&f.0.join("spool")).unwrap();
        let result = mail_blob_transfer::prepare(
            &mut a,
            &mut spool,
            [99; 16],
            selection(b"private orphan selected bytes"),
            NOW,
        );
        if attempt < 8 {
            assert!(matches!(result, Err(mail_blob_transfer::Error::State)));
        } else {
            assert!(matches!(
                result,
                Err(mail_blob_transfer::Error::Storage(
                    faction_files::private_blobs::Error::Quota
                ))
            ));
        }
        assert_eq!(a.pending(NOW).unwrap(), pending);
        assert_eq!(a.history().unwrap().len(), 8);
        assert_eq!(fs::read(f.0.join("alice/mail.fkv")).unwrap(), checkpoint);
        for entry in fs::read_dir(f.0.join("spool")).unwrap() {
            let p = entry.unwrap().path();
            let bytes = fs::read(p).unwrap();
            for secret in [
                b"private orphan selected bytes".as_slice(),
                b"secret-name.bin",
                b"secret-subject",
                b"secret-body",
            ] {
                assert!(!bytes.windows(secret.len()).any(|v| v == secret));
            }
        }
        drop(spool);
    }
}

#[test]
fn fetched_ciphertext_requires_exact_history_and_verification_before_deliberate_save() {
    use faction_files::private_blobs::{
        self as blobs, Access, Action, Binding, Principal, Request,
    };
    use faction_omemo::{attachment::Descriptor, blob};
    use sha2::{Digest, Sha256};
    use zeroize::Zeroizing;
    struct Local;
    impl Access for Local {
        fn require(
            &self,
            _: &Principal,
            _: &Binding,
            _: Action,
            _: u64,
        ) -> Result<(), blobs::Error> {
            Ok(())
        }
    }
    for wrong_key in [false, true] {
        let f = Fixture::new();
        let mut a = f.vault("alice", "alice@home.test", 101);
        let mut b = f.vault("bob", "bob@away.test", 202);
        a.enable_attachments().unwrap();
        b.enable_attachments().unwrap();
        let ab = a.bundle().unwrap();
        let bb = b.bundle().unwrap();
        a.enroll(
            "bob@away.test",
            202,
            &format!("ed25519:{}", faction_omemo::encode(&bb.identity)),
            NOW,
        )
        .unwrap();
        b.enroll(
            "alice@home.test",
            101,
            &format!("ed25519:{}", faction_omemo::encode(&ab.identity)),
            NOW,
        )
        .unwrap();
        a.start(&bb, NOW).unwrap();
        let encrypted = blob::encrypt(b"fetched private byte selection").unwrap();
        let id: [u8; 32] = Sha256::digest(&encrypted.ciphertext).into();
        let mut descriptor = Descriptor {
            name: Zeroizing::new("inert.bin".into()),
            media_type: Zeroizing::new("application/octet-stream".into()),
            size: encrypted.size,
            hash: encrypted.hash,
            key: encrypted.key,
            iv: encrypted.iv,
            locator: Zeroizing::new(format!(
                "urn:faction:blob:1:{}",
                faction_client::mail_data::hex(&id)
            )),
        };
        if wrong_key {
            descriptor.key[0] ^= 1;
        }
        let wire = a
            .queue_attachment([7; 16], "subject", "body", &descriptor, NOW, NOW)
            .unwrap();
        b.receive(&wire, NOW).unwrap();
        let frame = b.history().unwrap()[0].received_id().unwrap();
        let own = Principal {
            account: "bob@away.test".into(),
            device: 202,
            identity: bb.identity,
        };
        let binding = Binding {
            sender: own.clone(),
            recipient: own.clone(),
            id,
            size: encrypted.ciphertext.len() as u64,
            created: NOW,
            expires: NOW + blobs::MAX_RETENTION,
        };
        let path = f.0.join("spool");
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        let mut local = blobs::Store::open(&path).unwrap();
        local
            .dispatch(&own, &Local, Request::Begin(binding), NOW)
            .unwrap();
        local
            .dispatch(
                &own,
                &Local,
                Request::PutChunk {
                    id,
                    offset: 0,
                    bytes: encrypted.ciphertext,
                },
                NOW,
            )
            .unwrap();
        local
            .dispatch(&own, &Local, Request::Commit { id }, NOW)
            .unwrap();
        drop(local);
        let mut spool = Spool::open(&path).unwrap();
        let directory = f.0.join("save");
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .unwrap();
        let destination = directory.join("deliberate.bin");
        assert!(mail_blob_transfer::verify_fetched(&b, &mut spool, 0, [0; 32], NOW).is_err());
        if wrong_key {
            assert!(matches!(
                mail_blob_transfer::verify_fetched(&b, &mut spool, 0, frame, NOW),
                Err(mail_blob_transfer::Error::Blob)
            ));
            assert!(
                mail_blob_transfer::save_fetched(&b, &mut spool, 0, frame, NOW, &destination)
                    .is_err()
            );
            assert!(!destination.exists());
        } else {
            let verified =
                mail_blob_transfer::verify_fetched(&b, &mut spool, 0, frame, NOW).unwrap();
            assert_eq!(verified.as_bytes(), b"fetched private byte selection");
            assert!(
                !destination.exists(),
                "verification and canceled save must not publish plaintext"
            );
            b.revoke().unwrap();
            assert_eq!(
                mail_blob_transfer::verify_fetched(&b, &mut spool, 0, frame, NOW)
                    .unwrap()
                    .as_bytes(),
                b"fetched private byte selection"
            );
            mail_blob_transfer::save_fetched(&b, &mut spool, 0, frame, NOW, &destination).unwrap();
            assert!(
                mail_blob_transfer::save_fetched(&b, &mut spool, 0, frame, NOW, &destination)
                    .is_err()
            );
            assert_eq!(
                fs::read(destination).unwrap(),
                b"fetched private byte selection"
            );
        }
    }
}
