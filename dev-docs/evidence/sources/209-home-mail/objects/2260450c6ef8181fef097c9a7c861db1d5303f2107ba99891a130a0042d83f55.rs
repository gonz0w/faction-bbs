use crate::{mail_checkpoint::CheckpointVault, mail_data::{Data, hex}, mail_vault::Store};
use std::{fs, path::PathBuf};
const PASS: &[u8] = b"synthetic network checkpoint passphrase";
const NOW: u64 = 1_792_886_400;
struct Owned(PathBuf);
impl Owned {
    fn new() -> Self { let mut id=[0;16];getrandom::fill(&mut id).unwrap();Self(std::env::temp_dir().canonicalize().unwrap().join(format!("faction209-checkpoint-{}",hex(&id)))) }
    fn open(&self) -> CheckpointVault { CheckpointVault::unlock(Store::at(&self.0,false).unwrap(),PASS,false).unwrap() }
    fn bytes(&self)->Vec<u8> { fs::read(self.0.join("mail.fkv")).unwrap() }
}
impl Drop for Owned { fn drop(&mut self){let _=fs::remove_dir_all(&self.0);} }
fn pair()->(Owned,Owned,CheckpointVault,CheckpointVault){
    let a=Owned::new();let b=Owned::new();
    let mut av=CheckpointVault::create(Store::at(&a.0,true).unwrap(),Data::new("alice@home.test".into()).unwrap(),PASS,101,NOW).unwrap().0;
    let mut bv=CheckpointVault::create(Store::at(&b.0,true).unwrap(),Data::new("bob@away.test".into()).unwrap(),PASS,202,NOW).unwrap().0;
    let ab=av.bundle().unwrap();let bb=bv.bundle().unwrap();
    av.enroll("bob@away.test",202,&format!("ed25519:{}",hex(&bb.identity)),NOW).unwrap();
    bv.enroll("alice@home.test",101,&format!("ed25519:{}",hex(&ab.identity)),NOW).unwrap();
    av.start(&bb,NOW).unwrap();(a,b,av,bv)
}
#[test]
fn authenticated_network_timestamp_and_delayed_duplicate_survive_reopen(){
    let (_a,b,mut av,mut bv)=pair();
    let mut wire=av.queue([9;16],"delayed subject λ","private body <&>",NOW,NOW).unwrap();
    wire.sent_at=0;
    assert!(!bv.receive_network(&wire,NOW+86_400).unwrap().duplicate);
    assert_eq!(bv.history().unwrap()[0].sent_at,NOW);
    assert_eq!(bv.history().unwrap()[0].subject.as_str(),"delayed subject λ");
    drop(bv);let mut bv=b.open();let before=b.bytes();
    assert!(bv.receive_network(&wire,NOW+172_800).unwrap().duplicate);
    assert_eq!(bv.history().unwrap().len(),1);assert_eq!(b.bytes(),before);
}
#[test]
fn unauthenticated_timestamp_substitution_and_tampering_never_mutate_checkpoint(){
    let (_a,b,mut av,mut bv)=pair();
    let wire=av.queue([7;16],"subject","body",NOW,NOW).unwrap();let before=b.bytes();
    assert!(bv.receive_network(&wire,NOW).is_err());assert_eq!(b.bytes(),before);
    let mut network=wire.clone();network.sent_at=0;
    let mut bad=network.clone();bad.payload[0]^=1;
    assert!(bv.receive_network(&bad,NOW).is_err());assert_eq!(b.bytes(),before);
    let mut bad=network.clone();bad.identity[0]^=1;
    assert!(bv.receive_network(&bad,NOW).is_err());assert_eq!(b.bytes(),before);
    assert!(!bv.receive_network(&network,NOW).unwrap().duplicate);
    let accepted=b.bytes();
    assert!(bv.receive_network(&bad,NOW).is_err());assert_eq!(b.bytes(),accepted);
}

#[test]
fn published_prekeys_come_from_the_durable_current_identity_only(){
    let (_a,_b,av,mut bv)=pair();
    let (bundle,keys)=av.publication(NOW).unwrap();
    assert_eq!(keys.len(),32);assert_eq!(bundle.identity,*av.inspection().unwrap().identity);
    assert!(av.publication(NOW+crate::mail_checkpoint::SESSION_AGE+1).is_err());
    bv.revoke().unwrap();assert!(bv.publication(NOW).is_err());
}
