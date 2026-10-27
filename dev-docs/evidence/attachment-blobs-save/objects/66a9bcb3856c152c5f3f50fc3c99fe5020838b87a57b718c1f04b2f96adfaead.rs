#![cfg(feature = "native")]
use faction_omemo::{Device, attachment, blob};
use zeroize::Zeroizing;
const SENT: u64 = 1_709_164_800;

fn paired() -> (Device, Device) {
    let mut a = Device::fresh().unwrap();
    let mut b = Device::fresh().unwrap();
    a.trust(&format!("ed25519:{}", faction_omemo::encode(&b.bundle().unwrap().identity))).unwrap();
    b.trust(&format!("ed25519:{}", faction_omemo::encode(&a.bundle().unwrap().identity))).unwrap();
    a.start(&b.bundle().unwrap()).unwrap();
    (a, b)
}

#[test]
fn authenticated_descriptor_bad_blob_aborts_then_valid_blob_commits() {
    let encrypted = blob::encrypt(b"actual private blob bytes\0<&>").unwrap();
    let descriptor = attachment::Descriptor {
        name: Zeroizing::new("private.txt".into()),
        media_type: Zeroizing::new("text/plain".into()),
        size: encrypted.size,
        hash: encrypted.hash,
        key: encrypted.key,
        iv: encrypted.iv,
        locator: Zeroizing::new("https://blob.test/opaque".into()),
    };
    let encoded = attachment::encode("alice@example", "bob@example", SENT, "subject", "body", &descriptor).unwrap();
    let (mut sender, mut recipient) = paired();
    let message = sender.seal(&encoded).unwrap();
    let before = recipient.export().unwrap();
    for change in ["ciphertext", "tag", "truncated"] {
        let mut bad = encrypted.ciphertext.clone();
        match change {
            "ciphertext" => bad[0] ^= 1,
            "tag" => *bad.last_mut().unwrap() ^= 1,
            _ => { bad.pop(); }
        }
        let authenticated = recipient.open_pending(&message).unwrap();
        let parsed = attachment::decode(&authenticated, "alice@example", "bob@example", SENT).unwrap();
        assert!(blob::verify(&bad, &parsed.attachment).is_err());
        recipient.abort().unwrap();
        assert!(*recipient.export().unwrap() == *before);
    }
    let authenticated = recipient.open_pending(&message).unwrap();
    let parsed = attachment::decode(&authenticated, "alice@example", "bob@example", SENT).unwrap();
    let verified = blob::verify(&encrypted.ciphertext, &parsed.attachment).unwrap();
    assert_eq!(verified.as_bytes(), b"actual private blob bytes\0<&>");
    recipient.commit().unwrap();
    assert!(*recipient.export().unwrap() != *before);
}
