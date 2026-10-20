//! Synthetic core authorization dispatcher + separate Lua worker; not a BBS.
#![forbid(unsafe_code)]
use std::{env, process::{Command,Stdio}, thread, time::{Duration,Instant}};
#[derive(Clone,Copy,PartialEq)]enum Status{Pending,Approved,Revoked}
struct Member{status:Status,menu:bool,post:bool}
fn dispatch(member:&Member, bytes:&[u8])->Result<(usize,usize),&'static str>{
    if bytes.len()>2048{return Err("protocol limit")}
    let text=std::str::from_utf8(bytes).map_err(|_|"encoding")?;
    let mut menus=0;let mut posts=0;
    // Validate a complete batch before any synthetic mutation (no partial dispatch).
    for line in text.lines(){
        let (verb,body)=line.split_once('|').ok_or("protocol")?;
        if body.len()>120 || !body.bytes().all(|b|(32..=126).contains(&b)){return Err("content")}
        if member.status!=Status::Approved{return Err("approval")}
        match verb{
            "MENU" if member.menu=>menus+=1,
            "ROOM_POST" if member.post=>posts+=1,
            _=>return Err("capability"),
        }
        if menus+posts>8{return Err("event limit")}
    }
    Ok((menus,posts))
}
fn run(worker:&str,fixture:&str)->(bool,bool,Vec<u8>,String){
    let mut child=Command::new(worker).arg(fixture).env_clear().stdin(Stdio::null())
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("spawn worker");
    let start=Instant::now();let mut killed=false;
    loop{
        if child.try_wait().expect("wait").is_some(){break}
        if start.elapsed()>Duration::from_secs(2){child.kill().expect("kill");killed=true;break}
        thread::sleep(Duration::from_millis(5));
    }
    let out=child.wait_with_output().expect("reap");
    (out.status.success(),killed,out.stdout,String::from_utf8(out.stderr).expect("worker status"))
}
fn main(){
    let a:Vec<String>=env::args().collect();assert_eq!(a.len(),3);
    let worker=&a[1];let fixtures=&a[2];
    let cases=[("menu",Status::Approved,true,false,Ok((1,0))),
        ("post",Status::Approved,false,true,Ok((0,1))),
        ("post",Status::Pending,false,true,Err("approval")),
        ("post",Status::Revoked,false,true,Err("approval")),
        ("post",Status::Approved,false,false,Err("capability")),
        ("apis",Status::Approved,true,false,Ok((1,0)))];
    for (i,(fixture,status,menu,post,want)) in cases.into_iter().enumerate(){
        let (ok,killed,out,diagnostic)=run(worker,&format!("{fixtures}/{fixture}.lua"));
        assert!(ok && !killed && diagnostic.contains("live_after_close=0"));
        let member=Member{status,menu,post};assert_eq!(dispatch(&member,&out),want);
        println!("PASS dispatch_{i} {diagnostic}");
    }
    for (fixture,reason) in [("memory","memory"),("loop","instruction"),("controls","controls"),
                             ("length","length"),("events","events"),("badapi","script")]{
        let (ok,killed,_,diagnostic)=run(worker,&format!("{fixtures}/{fixture}.lua"));
        assert!(!ok && !killed && diagnostic.contains(&format!("result={reason} ")));
        assert!(diagnostic.contains("live_after_close=0"));
        // Discard ALL proposals from a failed worker; never commit partial output.
        println!("PASS reject_{fixture} {diagnostic}");
    }
    let (ok,killed,_,_)=run(worker,"--crash");assert!(!ok && !killed);println!("PASS child_crash_reaped");
    let (ok,killed,_,_)=run(worker,"--stall");assert!(!ok && killed);println!("PASS child_deadline_reaped");
    let (ok,killed,out,_)=run(worker,&format!("{fixtures}/menu.lua"));
    assert!(ok && !killed);assert_eq!(dispatch(&Member{status:Status::Approved,menu:true,post:false},&out),Ok((1,0)));
    println!("PASS healthy_after_failures");
    let approved=Member{status:Status::Approved,menu:true,post:true};
    assert_eq!(dispatch(&approved,b"APPROVE|self\n"),Err("capability"));
    assert_eq!(dispatch(&approved,b"ROOM_POST|bad\x1b[2J\n"),Err("content"));
    println!("PASS forged_api_and_control_bytes_denied");
}
