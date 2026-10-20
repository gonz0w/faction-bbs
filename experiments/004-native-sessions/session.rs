//! Research only: synthetic approval controls over private local Unix sockets.
//! No authentication, crypto, persistence, TCP or production protocol.
#![forbid(unsafe_code)]
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::time::Duration;

struct Session { stream: UnixStream, input: Vec<u8>, output: Vec<u8>, approved: bool }
fn main() -> io::Result<()> {
    let path = std::env::args().nth(1).expect("socket path required");
    let listener = UnixListener::bind(path)?;
    listener.set_nonblocking(true)?;
    let mut sessions: Vec<Session> = Vec::with_capacity(250);
    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                if sessions.len() >= 250 { let _ = stream.write_all(b"FULL\n"); }
                else {
                    stream.set_nonblocking(true)?;
                    sessions.push(Session { stream, input: Vec::with_capacity(1024),
                        output: Vec::with_capacity(8192), approved: false });
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => return Err(e),
        }
        sessions.retain_mut(|s| {
            let mut block = [0u8; 256];
            match s.stream.read(&mut block) {
                Ok(0) => return false,
                Ok(n) => for &b in &block[..n] {
                    if b == b'\n' {
                        let response: &[u8] = match s.input.as_slice() {
                            b"ALLOW" => { s.approved = true; b"OK\n" },
                            b"REVOKE" => { s.approved = false; b"OK\n" },
                            b"READ" | b"POST" | b"CHAT" if !s.approved => b"DENIED\n",
                            b"READ" => b"ROOM synthetic local message\n",
                            b"POST" => b"POSTED synthetic only\n",
                            b"CHAT" => b"CHAT synthetic only\n",
                            _ => b"ERROR\n",
                        };
                        if s.output.len() + response.len() > 8192 { return false; }
                        s.output.extend_from_slice(response); s.input.clear();
                    } else {
                        if !(32..=126).contains(&b) || s.input.len() >= 1024 { return false; }
                        s.input.push(b);
                    }
                },
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => (),
                Err(_) => return false,
            }
            if !s.output.is_empty() {
                match s.stream.write(&s.output) {
                    Ok(0) => return false,
                    Ok(n) => { s.output.drain(..n); },
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => (),
                    Err(_) => return false,
                }
            }
            true
        });
        std::thread::sleep(Duration::from_millis(1));
    }
}
