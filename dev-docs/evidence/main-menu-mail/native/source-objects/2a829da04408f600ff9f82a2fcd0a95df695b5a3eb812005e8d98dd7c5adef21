//! Owned disposable experiment fixture only. Never install as a member command.
#![forbid(unsafe_code)]
use faction_core::{Board, Status};
use std::path::Path;
fn state(board: &Board) -> faction_core::Result<()> {
    let s = board.roaming_owner_state()?;
    let identity: String = s
        .board_identity
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    print!("{{\"board_identity\":{identity:?},\"issuers\":[");
    for (n, i) in s.issuers.iter().enumerate() {
        if n > 0 {
            print!(",")
        }
        print!(
            "{{\"id\":{},\"name\":{:?},\"ca_fingerprint\":{:?},\"generation\":{},\"enabled\":{}}}",
            i.id, i.name, i.ca_fingerprint, i.generation, i.enabled
        );
    }
    print!("],\"bindings\":[");
    for (n, b) in s.bindings.iter().enumerate() {
        if n > 0 {
            print!(",")
        }
        print!(
            "{{\"id\":{},\"issuer_id\":{},\"subject\":{:?},\"principal\":{:?},\"member_id\":{},\"uid\":{},\"consent\":{:?},\"generation\":{},\"enabled\":{}}}",
            b.id,
            b.issuer_id,
            b.subject,
            b.principal,
            b.member_id,
            b.uid,
            b.consent,
            b.generation,
            b.enabled
        );
    }
    println!("]}}");
    Ok(())
}
fn on(s: &str) -> Result<bool, Box<dyn std::error::Error>> {
    match s {
        "on" => Ok(true),
        "off" => Ok(false),
        _ => Err("expected on|off".into()),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [directory, command, rest @ ..] = args.as_slice() else {
        return Err("BOARD COMMAND required".into());
    };
    let path = Path::new(directory);
    if !path.join(".roaming-fixture").is_file() {
        return Err("marked owned disposable .roaming-fixture directory required".into());
    }
    let mut board = Board::open(path)?;
    match (command.as_str(),rest) {
  ("state",[])=>{},
  ("bootstrap",[name,secret])=>board.bootstrap(name,secret)?,
  ("memberadd",[name,secret])=>board.register(name,secret)?,
  ("member",[name,status])=>{
   let value=match status.as_str(){"approved"=>Status::Approved,"pending"=>Status::Pending,"revoked"=>Status::Revoked,_=>return Err("invalid status".into())};
   // Fixture owner already holds the DB. This never becomes a product API.
   let mut db=rusqlite::Connection::open(path.join("faction.sqlite3"))?;
   db.execute_batch("PRAGMA foreign_keys=ON")?;
   let tx=db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
   let value=match value{Status::Approved=>"approved",Status::Pending=>"pending",Status::Revoked=>"revoked"};
   if tx.execute("UPDATE members SET status=?2,epoch=epoch+1 WHERE handle=?1 AND role='member'",rusqlite::params![name,value])?!=1 {return Err("member not found".into())}
   tx.execute("DELETE FROM room_drafts WHERE member_id IN(SELECT id FROM members WHERE handle=?1)",[name])?;tx.commit()?;
  },
  ("addissuer",[name,fingerprint])=>{board.roaming_add_issuer(name,fingerprint)?;},
  ("issuer",[id,generation,enabled])=>{board.roaming_set_issuer_enabled(id.parse()?,generation.parse()?,on(enabled)?)?;},
  ("link",[issuer,generation,subject,principal,member,uid,consent])=>{let member=board.roaming_owner_member_id(member)?;board.roaming_link_member(issuer.parse()?,generation.parse()?,subject,principal,member,uid.parse()?,consent)?;},
  ("enable",[id,generation])=>{board.roaming_set_binding_enabled(id.parse()?,generation.parse()?,true)?;},
  ("disable",[id,generation])=>{board.roaming_set_binding_enabled(id.parse()?,generation.parse()?,false)?;},
  _=>return Err("usage: BOARD state|bootstrap NAME SECRET|memberadd NAME SECRET|member NAME approved|pending|revoked|addissuer NAME SHA256:FINGERPRINT|issuer ID GEN on|off|link ISSUER GEN SUBJECT PRINCIPAL MEMBER UID CONSENT|enable ID GEN|disable ID GEN".into())
 }
    state(&board)?;
    Ok(())
}
