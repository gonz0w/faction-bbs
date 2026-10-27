#![cfg(feature = "native-mail")]
use faction_client::{mail_checkpoint::{CheckpointVault, Wire},mail_data::Data,mail_vault::Store};
use faction_omemo::{Device,attachment,sce};
use std::{fs,path::PathBuf};
use zeroize::Zeroizing;
const SENT:u64=1_709_164_800;
#[test]
fn authentic_candidate_is_rejected_without_checkpoint_ciphertext_or_metadata_change() {
    let root=PathBuf::from(std::env::var_os("FACTION252_FIXTURE_ROOT").expect("explicit owned fixture root"));
    assert_eq!(fs::read_to_string(root.join("OWNER")).unwrap(),"Faction issue 252 disposable implementation evidence. Keep until separate review.\n");
    let path=root.join(format!("checkpoint-{}",std::process::id()));
    let store=Store::at(&path,true).unwrap();
    let mut data=Data::new("bob@example".into()).unwrap();
    data.home="bob@example".into();
    let (mut vault,_recovery)=CheckpointVault::create(store,data,b"synthetic password",202,SENT).unwrap();
    let mut sender=Device::fresh().unwrap();
    let sender_bundle=sender.bundle().unwrap();
    vault.enroll("alice@example",101,&format!("ed25519:{}",faction_omemo::encode(&sender_bundle.identity)),SENT).unwrap();
    sender.trust(&format!("ed25519:{}",faction_omemo::encode(&vault.bundle().unwrap().identity))).unwrap();
    sender.start(&vault.bundle().unwrap()).unwrap();
    let d=attachment::Descriptor{name:Zeroizing::new("public.txt".into()),media_type:Zeroizing::new("text/plain".into()),size:0,hash:Zeroizing::new([0;32]),key:Zeroizing::new([1;32]),iv:Zeroizing::new([2;12]),locator:Zeroizing::new("https://blob.test/opaque".into())};
    let plain=attachment::encode("alice@example","bob@example",SENT,"subject","body",&d).unwrap();
    let mut verifier=Device::import(&sender.export().unwrap()).unwrap();
    // Sender seals a real authenticated descriptor, not a claimed plaintext hook.
    let msg=sender.seal(&plain).unwrap();
    // Explicitly establish that this XML is valid under the candidate interface.
    assert!(attachment::decode(&plain,"alice@example","bob@example",SENT).is_ok());
    let wire=Wire{namespace:"urn:xmpp:omemo:2".into(),from:"alice@example".into(),to:"bob@example".into(),device:101,to_device:202,identity:sender_bundle.identity,sent_at:SENT,key:msg.key,prekey:msg.prekey,payload:msg.payload};
    let canonical=fs::read(path.join("mail.fkv")).unwrap();
    let before=vault.inspection().unwrap();
    let metadata=(before.local_device,*before.identity,before.created,before.expires,before.history_only,before.started);
    assert!(vault.receive(&wire,SENT).is_err());
    assert!(fs::read(path.join("mail.fkv")).unwrap()==canonical);
    assert!(vault.history().unwrap().is_empty());
    assert!(vault.pending(SENT).unwrap().is_empty());
    let after=vault.inspection().unwrap();
    assert!(metadata==(after.local_device,*after.identity,after.created,after.expires,after.history_only,after.started));
    // The same sender state can produce ordinary text accepted after rejection.
    let ordinary=sce::encode("alice@example","bob@example",SENT,"subject","body").unwrap();
    let msg=verifier.seal(&ordinary).unwrap();
    let valid=Wire{key:msg.key,prekey:msg.prekey,payload:msg.payload,..wire};
    assert!(!vault.receive(&valid,SENT).unwrap().duplicate);
    assert_eq!(vault.history().unwrap().len(),1);
    drop(vault);
    fs::remove_dir_all(&path).unwrap();
    assert!(!path.exists());
}
