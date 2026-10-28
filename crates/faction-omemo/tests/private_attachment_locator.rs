#![cfg(feature = "native")]
use faction_omemo::attachment::Descriptor;
use zeroize::Zeroizing;
#[test]
fn only_exact_private_ciphertext_digest_locator_is_accepted_alongside_inert_https() {
    let mut d = Descriptor {
        name: Zeroizing::new("a.bin".into()),
        media_type: Zeroizing::new("application/octet-stream".into()),
        size: 0,
        hash: Zeroizing::new([0; 32]),
        key: Zeroizing::new([1; 32]),
        iv: Zeroizing::new([2; 12]),
        locator: Zeroizing::new(format!("urn:faction:blob:1:{}", "ab".repeat(32))),
    };
    assert!(d.validate().is_ok());
    for locator in [
        format!("urn:faction:blob:1:{}", "AB".repeat(32)),
        format!("urn:faction:blob:1:{}", "a".repeat(63)),
        format!("urn:faction:blob:2:{}", "ab".repeat(32)),
        format!("urn:faction:blob:1:{}?host=evil", "ab".repeat(32)),
        "urn:faction:blob:1:home.test/opaque".into(),
    ] {
        *d.locator = locator;
        assert!(d.validate().is_err());
    }
    *d.locator = "https://blob.test/inert".into();
    assert!(d.validate().is_ok());
}
