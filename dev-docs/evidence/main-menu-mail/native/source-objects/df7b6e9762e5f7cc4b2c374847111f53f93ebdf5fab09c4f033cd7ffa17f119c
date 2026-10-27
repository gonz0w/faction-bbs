//! Inert connection-bound mail controls. These carry no identity or file authority.
use crate::Result;
pub const PREFIX: &str = "7770;FACTION-MAIL1;";
pub const MAX_PAYLOAD: usize = 80;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Frame {
    Offer,
    Parked(String),
    Returned(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Park(String),
    Back(String),
}
pub fn is_valid_nonce(s: &str) -> bool {
    s.len() == 32
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn nonce(s: &str) -> Result<String> {
    if is_valid_nonce(s) {
        Ok(s.into())
    } else {
        Err("invalid mail nonce")
    }
}
pub fn parse_frame(payload: &str) -> Result<Frame> {
    if payload.len() > MAX_PAYLOAD {
        return Err("mail frame too large");
    }
    let fields: Vec<_> = payload.split(';').collect();
    match fields.as_slice() {
        ["offer"] => Ok(Frame::Offer),
        ["parked", n] => Ok(Frame::Parked(nonce(n)?)),
        ["returned", n] => Ok(Frame::Returned(nonce(n)?)),
        _ => Err("invalid mail frame"),
    }
}
pub fn encode_frame(frame: &Frame) -> Result<String> {
    let payload = match frame {
        Frame::Offer => "offer".into(),
        Frame::Parked(n) => format!("parked;{}", nonce(n)?),
        Frame::Returned(n) => format!("returned;{}", nonce(n)?),
    };
    Ok(format!("\x1b]{PREFIX}{payload}\x07"))
}
pub fn parse_request(line: &str) -> Result<Request> {
    if line.len() > MAX_PAYLOAD {
        return Err("mail request too large");
    }
    let line = line
        .strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .or_else(|| line.strip_suffix('\r'))
        .unwrap_or(line);
    let fields: Vec<_> = line.split(' ').collect();
    match fields.as_slice() {
        ["@MAIL1", "park", n] => Ok(Request::Park(nonce(n)?)),
        ["@MAIL1", "back", n] => Ok(Request::Back(nonce(n)?)),
        _ => Err("invalid mail request"),
    }
}
pub fn encode_request(request: &Request) -> Result<String> {
    match request {
        Request::Park(n) => Ok(format!("@MAIL1 park {}", nonce(n)?)),
        Request::Back(n) => Ok(format!("@MAIL1 back {}", nonce(n)?)),
    }
}
