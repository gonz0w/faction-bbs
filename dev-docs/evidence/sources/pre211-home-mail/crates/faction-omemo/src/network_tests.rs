//! Public-only discovery exports must reflect committed native state exactly.
use super::*;

#[test]
fn network_public_prekeys_are_bounded_distinct_and_survive_import() {
    let d = Device::fresh().unwrap();
    let before = d.export().unwrap();
    let keys = d.public_prekeys().unwrap();
    assert_eq!(keys.len(), 32);
    assert!(
        keys.iter()
            .all(|(id, _)| (1..=i32::MAX as u32).contains(id))
    );
    for (i, key) in keys.iter().enumerate() {
        assert!(!keys[..i].iter().any(|k| k.0 == key.0));
    }
    let b = d.bundle().unwrap();
    assert_eq!(keys[0], (b.prekey_id, b.prekey));
    assert_eq!(d.export().unwrap().as_slice(), before.as_slice());
    assert_eq!(d.try_clone().unwrap().public_prekeys().unwrap(), keys);
    assert_eq!(
        Device::import(&before).unwrap().public_prekeys().unwrap(),
        keys
    );
}
#[test]
fn network_public_prekeys_active_role_preserves_own_equal_id() {
    let mut active = Device::fresh().unwrap();
    let passive = Device::fresh().unwrap();
    let before = active.public_prekeys().unwrap();
    let bundle = passive.bundle().unwrap();
    assert!(before.iter().any(|p| p.0 == bundle.prekey_id));
    active
        .trust(&format!("ed25519:{}", encode(&bundle.identity)))
        .unwrap();
    active.start(&bundle).unwrap();
    assert_eq!(active.public_prekeys().unwrap(), before);
}
#[test]
fn network_public_prekeys_all_open_and_used_key_disappears_only_after_commit() {
    let receiver = Device::fresh().unwrap();
    let keys = receiver.public_prekeys().unwrap();
    let base = receiver.bundle().unwrap();
    for (id, key) in keys {
        let mut sender = Device::fresh().unwrap();
        sender
            .trust(&format!("ed25519:{}", encode(&base.identity)))
            .unwrap();
        let bundle = Bundle {
            prekey_id: id,
            prekey: key,
            ..base.clone()
        };
        sender.start(&bundle).unwrap();
        let mut target = receiver.try_clone().unwrap();
        target
            .trust(&format!(
                "ed25519:{}",
                encode(&sender.bundle().unwrap().identity)
            ))
            .unwrap();
        let encrypted = sender.seal(b"synthetic public prekey proof").unwrap();
        let plain = target.open_pending(&encrypted).unwrap();
        assert_eq!(plain.as_slice(), b"synthetic public prekey proof");
        assert!(target.public_prekeys().is_err()); // no staged-state publication
        target.abort().unwrap();
        assert!(target.public_prekeys().unwrap().iter().any(|p| p.0 == id));
        target.open_pending(&encrypted).unwrap();
        target.commit().unwrap();
        let after = target.public_prekeys().unwrap();
        assert_eq!(after.len(), 32);
        assert!(!after.iter().any(|p| p.0 == id));
        assert_eq!(
            Device::import(&target.export().unwrap())
                .unwrap()
                .public_prekeys()
                .unwrap(),
            after
        );
    }
}
