#![cfg(feature = "native-mail")]
use faction_client::{mail_checkpoint::CheckpointVault, mail_data::Data, mail_vault::Store};
#[test]
fn explicit_attachment_upgrade_is_idempotent_and_durable() {
    let root = std::env::temp_dir().join(format!("faction257-red-{}", std::process::id()));
    let store = Store::at(&root, true).unwrap();
    let (mut vault, _) = CheckpointVault::create(
        store,
        Data::new("alice@home.test".into()).unwrap(),
        b"synthetic checkpoint password",
        101,
        1_792_886_400,
    )
    .unwrap();
    assert!(!vault.inspection().unwrap().attachments_enabled);
    vault.enable_attachments().unwrap();
    let before = std::fs::read(root.join("mail.fkv")).unwrap();
    vault.enable_attachments().unwrap();
    assert!(std::fs::read(root.join("mail.fkv")).unwrap() == before);
    drop(vault);
    let vault = CheckpointVault::unlock(
        Store::at(&root, false).unwrap(),
        b"synthetic checkpoint password",
        false,
    )
    .unwrap();
    assert!(vault.inspection().unwrap().attachments_enabled);
    drop(vault);
    std::fs::remove_dir_all(root).unwrap();
}
