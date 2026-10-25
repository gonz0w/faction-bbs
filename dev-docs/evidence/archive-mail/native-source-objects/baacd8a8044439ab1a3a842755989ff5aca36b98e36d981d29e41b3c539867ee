#![cfg(feature = "native")]
// Fault injection in this test executable: the OS refuses the limit change.
#[unsafe(no_mangle)]
pub extern "C" fn setrlimit(_: i32, _: *const std::ffi::c_void) -> i32 {
    -1
}
#[test]
fn refusing_core_dump_suppression_refuses_private_device_creation() {
    assert!(faction_omemo::Device::fresh().is_err());
}
