#![cfg(feature = "native")]
use faction_omemo::{Device, encode, sce};

#[test]
fn native_clone_is_an_independent_transaction_and_sce_abort_keeps_state() {
    let mut alice = Device::fresh().unwrap();
    let mut bob = Device::fresh().unwrap();
    alice
        .trust(&format!(
            "ed25519:{}",
            encode(&bob.bundle().unwrap().identity)
        ))
        .unwrap();
    bob.trust(&format!(
        "ed25519:{}",
        encode(&alice.bundle().unwrap().identity)
    ))
    .unwrap();
    alice.start(&bob.bundle().unwrap()).unwrap();
    let before = alice.export().unwrap();
    let mut staged = alice.try_clone().unwrap();
    let invalid = staged.seal(b"not an SCE envelope").unwrap();
    assert_eq!(before, alice.export().unwrap());
    let old_bob = bob.export().unwrap();
    let plain = bob.open_pending(&invalid).unwrap();
    assert!(sce::decode(&plain, "alice@example", "bob@example", 0).is_err());
    bob.abort().unwrap();
    assert_eq!(old_bob, bob.export().unwrap());
    let raw = sce::encode("alice@example", "bob@example", 0, "subject", "body").unwrap();
    let valid = alice.seal(&raw).unwrap();
    let plain = bob.open_pending(&valid).unwrap();
    let content = sce::decode(&plain, "alice@example", "bob@example", 0).unwrap();
    assert_eq!(content.body.as_str(), "body");
    bob.commit().unwrap();
    assert!(bob.open_pending(&valid).is_err());
}
