//! Versioned bounded policy IPC. Never carries file bytes or BBS credentials.
#![forbid(unsafe_code)]
pub mod native;
pub mod private_blobs;
use std::{
    io::{self, Read, Write},
    os::{fd::AsFd, unix::net::UnixStream},
    path::Path,
    time::Duration,
};
pub const FRAME: usize = 4096;
pub fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid file operation")
}
pub fn id(s: &str) -> bool {
    s.len() == 32
        && s.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
pub fn encode(fields: &[&str]) -> io::Result<Vec<u8>> {
    if fields.len() > 24 {
        return Err(invalid());
    }
    let mut b = vec![1, fields.len() as u8];
    for f in fields {
        if f.len() > 1024 {
            return Err(invalid());
        }
        b.extend_from_slice(&(f.len() as u16).to_be_bytes());
        b.extend_from_slice(f.as_bytes());
    }
    if b.len() > FRAME {
        return Err(invalid());
    }
    let mut out = (b.len() as u32).to_be_bytes().to_vec();
    out.extend(b);
    Ok(out)
}
pub fn decode(b: &[u8]) -> io::Result<Vec<String>> {
    if b.len() < 2 || b.len() > FRAME || b[0] != 1 || b[1] > 24 {
        return Err(invalid());
    }
    let mut n = 2;
    let mut v = Vec::new();
    for _ in 0..b[1] {
        if n + 2 > b.len() {
            return Err(invalid());
        }
        let len = u16::from_be_bytes([b[n], b[n + 1]]) as usize;
        n += 2;
        if len > 1024 || n + len > b.len() {
            return Err(invalid());
        }
        v.push(
            std::str::from_utf8(&b[n..n + len])
                .map_err(|_| invalid())?
                .into(),
        );
        n += len;
    }
    if n != b.len() {
        return Err(invalid());
    }
    Ok(v)
}
pub fn rpc(socket: &Path, owner: u32, fields: &[&str]) -> io::Result<Vec<String>> {
    let mut s = UnixStream::connect(socket)?;
    s.set_read_timeout(Some(Duration::from_secs(3)))?;
    s.set_write_timeout(Some(Duration::from_secs(3)))?;
    if faction_unix::peer_uid(s.as_fd())? != owner {
        return Err(invalid());
    }
    s.write_all(&encode(fields)?)?;
    let mut len = [0; 4];
    s.read_exact(&mut len)?;
    let n = u32::from_be_bytes(len) as usize;
    if n > FRAME {
        return Err(invalid());
    }
    let mut b = vec![0; n];
    s.read_exact(&mut b)?;
    let mut end = [0];
    if s.read(&mut end)? != 0 {
        return Err(invalid());
    }
    let mut v = decode(&b)?;
    if v.first().map(String::as_str) != Some("ok") {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "file operation denied",
        ));
    }
    v.remove(0);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_or_unbounded_policy_frames_have_no_interpretation() {
        for fields in [vec![], vec!["start", "0123456789abcdef0123456789abcdef"]] {
            let frame = encode(&fields).unwrap();
            assert_eq!(decode(&frame[4..]).unwrap(), fields);
            for n in 0..frame.len() - 4 {
                assert!(decode(&frame[4..4 + n]).is_err());
            }
            let mut extra = frame[4..].to_vec();
            extra.push(0);
            assert!(decode(&extra).is_err());
        }
        assert!(encode(&["x".repeat(1025).as_str()]).is_err());
        assert!(encode(&["x"; 25]).is_err());
        assert!(decode(&[2, 0]).is_err());
        assert!(decode(&[1, 1, 0, 1, 255]).is_err());
    }
}
