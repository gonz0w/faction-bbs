//! Optional nonblocking policy endpoint; no new owner helper/process/thread.
use super::*;
use faction_files::{FRAME, decode, encode};
struct Peer {
    stream: UnixStream,
    uid: u32,
    input: Vec<u8>,
    output: Vec<u8>,
    written: usize,
    started: Instant,
    done: bool,
}
pub(super) struct Policy {
    listener: UnixListener,
    peers: Vec<Peer>,
    worker: u32,
    auth: u32,
    guard: SocketGuard,
}
impl Policy {
    pub fn bind(worker: u32, auth: u32) -> Result<Self> {
        let path = Path::new("/var/run/faction/file-policy.sock");
        if let Ok(m) = fs::symlink_metadata(path) {
            if !m.file_type().is_socket()
                || m.uid() != rustix::process::geteuid().as_raw()
                || UnixStream::connect(path).is_ok()
            {
                return Err(denied());
            }
            fs::remove_file(path)?;
        }
        let listener = UnixListener::bind(path)?;
        let guard = SocketGuard {
            path: path.into(),
            metadata: fs::symlink_metadata(path)?,
        };
        fs::set_permissions(path, fs::Permissions::from_mode(0o660))?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            peers: Vec::new(),
            worker,
            auth,
            guard,
        })
    }
    pub fn pump(&mut self, board: &mut Board) {
        let _ = &self.guard;
        for _ in 0..4 {
            match self.listener.accept() {
                Ok((s, _)) => {
                    if self.peers.len() >= 4 {
                        continue;
                    }
                    let Ok(uid) = faction_unix::peer_uid(s.as_fd()) else {
                        continue;
                    };
                    if uid != self.worker && uid != self.auth {
                        continue;
                    }
                    if s.set_nonblocking(true).is_err() {
                        continue;
                    }
                    self.peers.push(Peer {
                        stream: s,
                        uid,
                        input: Vec::new(),
                        output: Vec::new(),
                        written: 0,
                        started: Instant::now(),
                        done: false,
                    });
                }
                Err(_) => break,
            }
        }
        for p in &mut self.peers {
            if p.started.elapsed() > Duration::from_secs(3) {
                p.done = true;
                continue;
            }
            if p.output.is_empty() {
                let mut b = [0; FRAME + 5];
                match p.stream.read(&mut b) {
                    Ok(0) => p.done = true,
                    Ok(n) => {
                        p.input.extend_from_slice(&b[..n]);
                        if p.input.len() > FRAME + 4 {
                            p.done = true;
                            continue;
                        }
                        if p.input.len() >= 4 {
                            let len = u32::from_be_bytes(p.input[..4].try_into().expect("header"))
                                as usize;
                            if len > FRAME {
                                p.done = true;
                                continue;
                            }
                            if p.input.len() >= len + 4 {
                                if p.input.len() != len + 4 {
                                    p.done = true;
                                    continue;
                                }
                                let response = decode(&p.input[4..]).ok().and_then(|f| {
                                    self::dispatch(board, p.uid == self.auth, &f).ok()
                                });
                                let fields = match response {
                                    Some(mut v) => {
                                        v.insert(0, "ok".into());
                                        v
                                    }
                                    None => vec!["denied".into()],
                                };
                                p.output =
                                    encode(&fields.iter().map(String::as_str).collect::<Vec<_>>())
                                        .unwrap_or_default();
                                if p.output.is_empty() {
                                    p.done = true
                                }
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                    Err(_) => p.done = true,
                }
            }
            if !p.output.is_empty() {
                match p.stream.write(&p.output[p.written..]) {
                    Ok(0) => p.done = true,
                    Ok(n) => {
                        p.written += n;
                        if p.written == p.output.len() {
                            p.done = true
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                    Err(_) => p.done = true,
                }
            }
        }
        self.peers.retain(|p| !p.done);
    }
}
fn number(s: &str) -> faction_core::Result<u32> {
    s.parse().map_err(|_| Error::Denied)
}
fn dispatch(b: &mut Board, auth: bool, f: &[String]) -> faction_core::Result<Vec<String>> {
    let fields = f.iter().map(String::as_str).collect::<Vec<_>>();
    if auth {
        return match fields.as_slice() {
            ["lookup", key] => Ok(vec![b.file_lookup_key(key)?.ok_or(Error::Denied)?]),
            _ => Err(Error::Denied),
        };
    }
    match fields.as_slice() {
        ["start", id] => {
            let g = b.file_claim(id)?;
            Ok(vec![
                g.object,
                g.filename,
                g.direction,
                g.bytes.to_string(),
                g.sha256,
                g.lease,
            ])
        }
        ["check", id, lease] => {
            b.file_check(id, lease)?;
            Ok(vec![])
        }
        ["chunk", id, lease, offset, bytes] => {
            b.file_chunk(id, lease, number(offset)?, number(bytes)?)?;
            Ok(vec![])
        }
        ["finish", id, lease, sha] => {
            b.file_complete(id, lease, sha)?;
            Ok(vec![])
        }
        ["cancel", id, lease] => {
            b.file_cancel(id, lease)?;
            Ok(vec![])
        }
        ["removed", after] => b.file_removed(after),
        ["reclaimed", id] => {
            b.file_reclaimed(id)?;
            Ok(vec![])
        }
        _ => Err(Error::Denied),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_lookup_uid_cannot_claim_or_complete_valid_transfer_authority() {
        use std::os::unix::fs::DirBuilderExt;
        let p = std::env::temp_dir().join(format!(
            "faction-file-policy-{}",
            faction_core::request_id().unwrap()
        ));
        fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        let mut b = Board::open(&p.join("board")).unwrap();
        let secret = faction_core::request_id().unwrap();
        b.bootstrap("sysop", &secret).unwrap();
        let s = b.authenticate("sysop", &secret).unwrap();
        let area = b
            .file_create_area(&s, "files", "File policy", false, 100)
            .unwrap();
        // Structurally valid public key; signature verification belongs to SSH.
        let key = "AAAAC3NzaC1lZDI1NTE5AAAAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let grant = b
            .file_upload(
                &s,
                area,
                "test.bin",
                "test description",
                1,
                &format!("ssh-ed25519 {key}"),
            )
            .unwrap();
        let fields = |f: &[&str]| f.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        assert_eq!(
            dispatch(&mut b, true, &fields(&["lookup", key])).unwrap(),
            vec![grant.clone()]
        );
        assert!(dispatch(&mut b, true, &fields(&["start", &grant])).is_err());
        assert!(dispatch(&mut b, false, &fields(&["lookup", key])).is_err());
        let claimed = dispatch(&mut b, false, &fields(&["start", &grant])).unwrap();
        assert!(dispatch(&mut b, true, &fields(&["finish", &grant, &claimed[5], ""])).is_err());
        assert!(b.file_claim(&grant).is_err());
        drop(b);
        fs::remove_dir_all(p).unwrap();
    }
}
