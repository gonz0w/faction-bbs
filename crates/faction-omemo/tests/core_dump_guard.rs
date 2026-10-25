#![cfg(feature = "native")]
unsafe extern "C" {
    fn proof_core_dumps_disabled() -> i32;
}
#[test]
fn creating_private_native_state_disables_both_core_dump_limits() {
    let _device = faction_omemo::Device::fresh().unwrap();
    assert_eq!(unsafe { proof_core_dumps_disabled() }, 1);
}
