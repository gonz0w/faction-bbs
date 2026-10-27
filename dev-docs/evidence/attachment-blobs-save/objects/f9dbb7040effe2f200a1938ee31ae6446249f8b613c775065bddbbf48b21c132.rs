#![cfg(feature = "native-mail")]
use faction_client::attachment_save::{SaveError, save_verified};
use faction_omemo::{attachment::Descriptor, blob};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt, symlink},
    path::PathBuf,
};
use zeroize::Zeroizing;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let mut nonce = [0; 16];
        getrandom::fill(&mut nonce).unwrap();
        let suffix = nonce.iter().map(|v| format!("{v:02x}")).collect::<String>();
        let path = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("faction255-save-{suffix}"));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        fs::write(
            path.join("OWNER"),
            b"Owned issue 255 disposable save fixture",
        )
        .unwrap();
        Self(path)
    }
    fn parent(&self) -> PathBuf {
        let path = self.0.join("selected");
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert_eq!(
            fs::read(self.0.join("OWNER")).unwrap(),
            b"Owned issue 255 disposable save fixture"
        );
        fs::remove_dir_all(&self.0).unwrap();
        assert!(!self.0.exists());
    }
}
fn verified(bytes: &[u8]) -> blob::VerifiedBlob {
    let encrypted = blob::encrypt(bytes).unwrap();
    let d = Descriptor {
        name: Zeroizing::new("remote-name-never-selected.txt".into()),
        media_type: Zeroizing::new("application/octet-stream".into()),
        size: encrypted.size,
        hash: encrypted.hash,
        key: encrypted.key,
        iv: encrypted.iv,
        locator: Zeroizing::new("https://blob.test/never-fetch".into()),
    };
    blob::verify(&encrypted.ciphertext, &d).unwrap()
}
#[test]
fn explicit_private_save_preserves_exact_zero_small_and_cap_bytes() {
    let f = Fixture::new();
    let parent = f.parent();
    for (index, size) in [0, 17, blob::MAX_BLOB_BYTES].into_iter().enumerate() {
        let bytes = (0..size).map(|i| (i % 251) as u8).collect::<Vec<_>>();
        let v = verified(&bytes);
        let destination = parent.join(format!("chosen-{index}"));
        save_verified(&v, &destination).unwrap();
        assert_eq!(fs::read(&destination).unwrap(), bytes);
        assert_eq!(fs::metadata(&destination).unwrap().mode() & 0o7777, 0o600);
        assert_eq!(fs::metadata(&parent).unwrap().mode() & 0o7777, 0o700);
    }
    assert_eq!(fs::read_dir(&parent).unwrap().count(), 3);
    assert!(!parent.join("remote-name-never-selected.txt").exists());
}
#[test]
fn existing_objects_and_unsafe_paths_remain_untouched() {
    let f = Fixture::new();
    let parent = f.parent();
    let v = verified(b"verified new bytes");
    fs::write(parent.join("existing"), b"keep original").unwrap();
    fs::create_dir(parent.join("directory")).unwrap();
    symlink("existing", parent.join("symlink")).unwrap();
    symlink("absent", parent.join("dangling")).unwrap();
    for name in ["existing", "directory", "symlink", "dangling"] {
        assert_eq!(
            save_verified(&v, &parent.join(name)),
            Err(SaveError::BeforePublication)
        );
    }
    assert_eq!(fs::read(parent.join("existing")).unwrap(), b"keep original");
    assert!(
        fs::symlink_metadata(parent.join("dangling"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    symlink("selected", f.0.join("alias")).unwrap();
    for path in [
        PathBuf::from("relative"),
        PathBuf::from("/"),
        parent.join("../escape"),
        f.0.join("alias/out"),
        f.0.join("missing/out"),
    ] {
        assert_eq!(save_verified(&v, &path), Err(SaveError::BeforePublication));
    }
    for mode in [0o755, 0o770, 0o1700] {
        fs::set_permissions(&parent, fs::Permissions::from_mode(mode)).unwrap();
        assert_eq!(
            save_verified(&v, &parent.join("unsafe")),
            Err(SaveError::BeforePublication)
        );
    }
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(fs::read_dir(&parent).unwrap().count(), 4);
    assert!(!f.0.join("escape").exists());
}
#[test]
fn invalid_blob_and_canceled_selection_make_no_save_call() {
    let f = Fixture::new();
    let parent = f.parent();
    let encrypted = blob::encrypt(b"private bytes").unwrap();
    let d = Descriptor {
        name: Zeroizing::new("remote.txt".into()),
        media_type: Zeroizing::new("text/plain".into()),
        size: encrypted.size,
        hash: encrypted.hash,
        key: encrypted.key,
        iv: encrypted.iv,
        locator: Zeroizing::new("https://blob.test/opaque".into()),
    };
    let mut corrupt = encrypted.ciphertext;
    corrupt[0] ^= 1;
    assert!(blob::verify(&corrupt, &d).is_err());
    let _canceled_selection = verified(b"verified but caller canceled");
    assert_eq!(fs::read_dir(&parent).unwrap().count(), 0);
}

#[test]
fn authenticated_peer_descriptor_then_real_blob_verification_then_explicit_save() {
    use faction_omemo::{Device, Message, attachment};
    const SENT: u64 = 1_709_164_800;
    let f = Fixture::new();
    let parent = f.parent();
    let mut sender = Device::fresh().unwrap();
    let mut recipient = Device::fresh().unwrap();
    sender
        .trust(&format!(
            "ed25519:{}",
            faction_omemo::encode(&recipient.bundle().unwrap().identity)
        ))
        .unwrap();
    recipient
        .trust(&format!(
            "ed25519:{}",
            faction_omemo::encode(&sender.bundle().unwrap().identity)
        ))
        .unwrap();
    sender.start(&recipient.bundle().unwrap()).unwrap();
    for (index, failure) in ["authentication", "blob", "hash", "size"]
        .into_iter()
        .enumerate()
    {
        let bytes = b"actual encrypted authenticated descriptor to verified saved bytes\0\xff";
        let encrypted = blob::encrypt(bytes).unwrap();
        let mut d = Descriptor {
            name: Zeroizing::new("remote-name.txt".into()),
            media_type: Zeroizing::new("application/octet-stream".into()),
            size: encrypted.size,
            hash: encrypted.hash,
            key: encrypted.key,
            iv: encrypted.iv,
            locator: Zeroizing::new("https://blob.test/opaque".into()),
        };
        if failure == "hash" {
            d.hash[0] ^= 1;
        }
        if failure == "size" {
            d.size += 1;
        }
        let raw = attachment::encode("alice@example", "bob@example", SENT, "subject", "body", &d)
            .unwrap();
        let message = sender.seal(&raw).unwrap();
        let before = recipient.export().unwrap();
        if failure == "authentication" {
            let mut corrupt = Message {
                key: message.key.clone(),
                prekey: message.prekey,
                payload: message.payload.clone(),
            };
            corrupt.payload[0] ^= 1;
            assert!(recipient.open_pending(&corrupt).is_err());
        } else {
            let authenticated = recipient.open_pending(&message).unwrap();
            let descriptor =
                attachment::decode(&authenticated, "alice@example", "bob@example", SENT)
                    .unwrap()
                    .attachment;
            let mut ciphertext = encrypted.ciphertext.clone();
            if failure == "blob" {
                ciphertext[0] ^= 1;
            }
            assert!(blob::verify(&ciphertext, &descriptor).is_err());
            recipient.abort().unwrap();
        }
        assert!(*recipient.export().unwrap() == *before);
        assert_eq!(fs::read_dir(&parent).unwrap().count(), index);
        // A valid authenticated candidate follows the rejected one. Saving
        // consumes only the actual verified type; metadata alone cannot write.
        if failure == "hash" {
            d.hash[0] ^= 1;
        }
        if failure == "size" {
            d.size -= 1;
        }
        let raw = attachment::encode("alice@example", "bob@example", SENT, "subject", "body", &d)
            .unwrap();
        let valid = sender.seal(&raw).unwrap();
        let authenticated = recipient.open_pending(&valid).unwrap();
        let descriptor = attachment::decode(&authenticated, "alice@example", "bob@example", SENT)
            .unwrap()
            .attachment;
        let verified = blob::verify(&encrypted.ciphertext, &descriptor).unwrap();
        let destination = parent.join(format!("explicit-{index}"));
        save_verified(&verified, &destination).unwrap();
        assert_eq!(fs::read(&destination).unwrap(), bytes);
        recipient.commit().unwrap();
        assert!(*recipient.export().unwrap() != *before);
    }
    assert!(!parent.join("remote-name.txt").exists());
    assert_eq!(fs::read_dir(&parent).unwrap().count(), 4);
}
