//! Explicit native-client file control. Public key/metadata only, no local paths.
use crate::{decode, encode, invalid};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::io;
pub const PREFIX: &[u8] = b"\x1b]7770;FACTION1;";
pub fn frame(fields: &[&str]) -> io::Result<String> {
    let encoded = encode(fields)?;
    Ok(format!(
        "\x1b]7770;FACTION1;{}\x07",
        STANDARD.encode(&encoded[4..])
    ))
}
pub fn unframe(payload: &[u8]) -> io::Result<Vec<String>> {
    if payload.len() > 5500 {
        return Err(invalid());
    }
    decode(&STANDARD.decode(payload).map_err(|_| invalid())?)
}
pub fn command(nonce: &str, seq: u32, fields: &[&str]) -> io::Result<String> {
    if !crate::id(nonce) || seq == 0 {
        return Err(invalid());
    }
    let b = encode(fields)?;
    if b.len() > 604 {
        return Err(invalid());
    }
    Ok(format!(
        "@NATIVE1 {nonce} {seq} {}\r",
        STANDARD.encode(&b[4..])
    ))
}
pub fn request(line: &str) -> io::Result<(&str, u32, Vec<String>)> {
    if line.len() > 1024 {
        return Err(invalid());
    }
    let mut p = line.split_whitespace();
    if p.next() != Some("@NATIVE1") {
        return Err(invalid());
    }
    let nonce = p.next().filter(|s| crate::id(s)).ok_or_else(invalid)?;
    let seq = p
        .next()
        .ok_or_else(invalid)?
        .parse::<u32>()
        .map_err(|_| invalid())?;
    let b = p.next().ok_or_else(invalid)?;
    if p.next().is_some() || seq == 0 || b.len() > 800 {
        return Err(invalid());
    }
    let b = STANDARD.decode(b).map_err(|_| invalid())?;
    if b.len() > 600 {
        return Err(invalid());
    }
    Ok((nonce, seq, decode(&b)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_controls_are_versioned_bounded_and_strictly_framed() {
        let nonce = "0123456789abcdef0123456789abcdef";
        let text = command(nonce, 1, &["list", "1", ""]).unwrap();
        let (n, s, fields) = request(&text).unwrap();
        assert_eq!((n, s), (nonce, 1));
        assert_eq!(fields, ["list", "1", ""]);
        assert!(command(nonce, 0, &["back"]).is_err());
        assert!(command("bad", 1, &["back"]).is_err());
        assert!(command(nonce, 1, &[&"a".repeat(601)]).is_err());
        assert!(request(&text.replace("@NATIVE1", "@NATIVE2")).is_err());
        assert!(request(&format!("{text} trailing")).is_err());
        assert!(request(&format!("@NATIVE1 {nonce} 1 !!!")).is_err());
        assert!(unframe(&vec![b'A'; 5501]).is_err());
        let wire = frame(&["ready"]).unwrap();
        assert_eq!(
            unframe(&wire.as_bytes()[PREFIX.len()..wire.len() - 1]).unwrap(),
            ["ready"]
        );
    }
}
