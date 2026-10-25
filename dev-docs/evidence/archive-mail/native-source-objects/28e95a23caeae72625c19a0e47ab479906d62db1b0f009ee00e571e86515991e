#![cfg(feature = "native")]
// The executable's symbol replaces the dynamically linked major-version query.
// This exercises the real native allocation guard without loading another library.
#[unsafe(no_mangle)]
pub extern "C" fn OPENSSL_version_major() -> u32 {
    2
}

#[test]
fn incompatible_openssl_runtime_refuses_device_creation() {
    assert!(faction_omemo::Device::fresh().is_err());
}
