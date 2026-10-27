#![cfg(feature = "native")]
use faction_omemo::{Device, Message, attachment};
use zeroize::Zeroizing;
const SENT: u64 = 1_709_164_800;
fn xml() -> Zeroizing<Vec<u8>> {
    let d = attachment::Descriptor {
        name: Zeroizing::new("public.txt".into()),
        media_type: Zeroizing::new("text/plain".into()),
        size: 17,
        hash: Zeroizing::new([0; 32]),
        key: Zeroizing::new([1; 32]),
        iv: Zeroizing::new([2; 12]),
        locator: Zeroizing::new("https://blob.test/opaque".into()),
    };
    attachment::encode("alice@example", "bob@example", SENT, "subject", "body", &d).unwrap()
}
fn paired() -> (Device, Device) {
    let mut a = Device::fresh().unwrap();
    let mut b = Device::fresh().unwrap();
    a.trust(&format!(
        "ed25519:{}",
        faction_omemo::encode(&b.bundle().unwrap().identity)
    ))
    .unwrap();
    b.trust(&format!(
        "ed25519:{}",
        faction_omemo::encode(&a.bundle().unwrap().identity)
    ))
    .unwrap();
    a.start(&b.bundle().unwrap()).unwrap();
    (a, b)
}
fn copy(m: &Message) -> Message {
    Message {
        key: m.key.clone(),
        prekey: m.prekey,
        payload: m.payload.clone(),
    }
}
#[test]
fn authenticated_descriptor_commits_and_authentic_malformed_input_aborts() {
    let (mut a, mut b) = paired();
    for malformed in ["descriptor", "jid", "time"] {
        let original = xml();
        let raw = std::str::from_utf8(&original).unwrap();
        let bad = Zeroizing::new(match malformed {
            "descriptor" => raw.replace("sha-256", "sha-512"),
            "jid" => raw.replace("bob@example", "mallory@example"),
            _ => raw.replace("2024-02-29T00:00:00Z", "2024-02-29T00:10:01Z"),
        });
        let msg = a.seal(bad.as_bytes()).unwrap();
        let before = b.export().unwrap();
        let authenticated = b.open_pending(&msg).unwrap();
        assert!(attachment::decode(&authenticated, "alice@example", "bob@example", SENT).is_err());
        b.abort().unwrap();
        assert!(*b.export().unwrap() == *before);
        let valid = a.seal(&xml()).unwrap();
        let authenticated = b.open_pending(&valid).unwrap();
        let parsed =
            attachment::decode(&authenticated, "alice@example", "bob@example", SENT).unwrap();
        assert_eq!(parsed.attachment.size, 17);
        b.commit().unwrap();
        assert!(*b.export().unwrap() != *before);
    }
}
#[test]
fn untrusted_wrong_peer_and_corrupt_messages_never_release_candidate_plaintext() {
    let (mut a, mut b) = paired();
    let msg = a.seal(&xml()).unwrap();
    for change in ["payload", "truncated", "key"] {
        let before = b.export().unwrap();
        let mut bad = copy(&msg);
        match change {
            "payload" => bad.payload[0] ^= 1,
            "key" => bad.key[0] ^= 1,
            _ => {
                bad.payload.pop();
            }
        }
        assert!(b.open_pending(&bad).is_err());
        assert!(*b.export().unwrap() == *before);
    }
    let mut untrusted = Device::fresh().unwrap();
    let before = untrusted.export().unwrap();
    assert!(untrusted.open_pending(&msg).is_err());
    assert!(*untrusted.export().unwrap() == *before);
    let mut wrong = Device::fresh().unwrap();
    wrong
        .trust(&format!(
            "ed25519:{}",
            faction_omemo::encode(&Device::fresh().unwrap().bundle().unwrap().identity)
        ))
        .unwrap();
    let before = wrong.export().unwrap();
    assert!(wrong.open_pending(&msg).is_err());
    assert!(*wrong.export().unwrap() == *before);
    let plain = b.open_pending(&msg).unwrap();
    assert!(attachment::decode(&plain, "alice@example", "bob@example", SENT).is_ok());
    b.commit().unwrap();
}
