//! Readiness-based follow-up to the scan probe; research, not authentication.
#![forbid(unsafe_code)]
use std::io::{self, Read, Write};
use mio::{Events, Interest, Poll, Token};
use mio::net::{UnixListener, UnixStream};
struct Session { stream: UnixStream, input: Vec<u8>, output: Vec<u8>, approved: bool }
fn service(s: &mut Session) -> bool {
    let mut block = [0u8;256];
    loop {
        match s.stream.read(&mut block) {
            Ok(0) => return false,
            Ok(n) => for &b in &block[..n] {
                if b == b'\n' {
                    let response: &[u8] = match s.input.as_slice() {
                        b"ALLOW" => { s.approved=true; b"OK\n" },
                        b"REVOKE" => { s.approved=false; b"OK\n" },
                        b"READ" | b"POST" | b"CHAT" if !s.approved => b"DENIED\n",
                        b"READ" => b"ROOM synthetic local message\n",
                        b"POST" => b"POSTED synthetic only\n",
                        b"CHAT" => b"CHAT synthetic only\n",
                        _ => b"ERROR\n",
                    };
                    if s.output.len()+response.len()>8192 { return false; }
                    s.output.extend_from_slice(response); s.input.clear();
                } else {
                    if !(32..=126).contains(&b) || s.input.len()>=1024 { return false; }
                    s.input.push(b);
                }
            },
            Err(e) if e.kind()==io::ErrorKind::WouldBlock => break,
            Err(_) => return false,
        }
    }
    while !s.output.is_empty() {
        match s.stream.write(&s.output) {
            Ok(0) => return false,
            Ok(n) => { s.output.drain(..n); },
            Err(e) if e.kind()==io::ErrorKind::WouldBlock => break,
            Err(_) => return false,
        }
    }
    true
}
fn main() -> io::Result<()> {
    let path=std::env::args().nth(1).expect("socket path required");
    let mut listener=UnixListener::bind(path)?;
    let mut poll=Poll::new()?;
    poll.registry().register(&mut listener,Token(250),Interest::READABLE)?;
    let mut sessions: Vec<Option<Session>>=(0..250).map(|_|None).collect();
    let mut events=Events::with_capacity(512);
    loop {
        poll.poll(&mut events,None)?;
        for event in &events {
            if event.token()==Token(250) {
                loop {
                    match listener.accept() {
                        Ok((mut stream,_)) => {
                            if let Some(index)=sessions.iter().position(Option::is_none) {
                                poll.registry().register(&mut stream,Token(index),Interest::READABLE.add(Interest::WRITABLE))?;
                                sessions[index]=Some(Session {stream,input:Vec::with_capacity(1024),output:Vec::with_capacity(8192),approved:false});
                            } else { let _=stream.write_all(b"FULL\n"); }
                        },
                        Err(e) if e.kind()==io::ErrorKind::WouldBlock => break,
                        Err(e) => return Err(e),
                    }
                }
            } else if let Some(session)=sessions[event.token().0].as_mut() {
                if !service(session) { sessions[event.token().0]=None; }
            }
        }
    }
}
