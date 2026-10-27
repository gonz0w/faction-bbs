//! Laboratory-only private-pipe adapter for the unchanged independent reference.
//! Never install this raw-state protocol as a member UI or product transport.
use faction_omemo::{
    Bundle, Device, Error, MAX_KEY, MAX_PAYLOAD, MAX_PLAINTEXT, MAX_SNAPSHOT, Message, Result,
    convert, decode, encode, sce,
};
use std::{
    fs::File,
    io::{self, Read, Write},
    os::{fd::AsFd, unix::fs::FileTypeExt},
};
use zeroize::{Zeroize, Zeroizing};
const MAX_LINE: usize = 65568;
const STAMP: u64 = 1792843200; // 2026-10-24T12:00:00Z, fixed synthetic matrix.
#[derive(Default)]
struct State {
    device: Option<Device>,
    pending: bool,
}
fn fixed<const N: usize>(text: &str) -> Result<[u8; N]> {
    decode(text, N)?.try_into().map_err(|_| Error)
}
fn id(text: &str) -> Result<u32> {
    if text.is_empty() || text.len() > 10 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error);
    }
    let n = text.parse().map_err(|_| Error)?;
    if n == 0 || n > 0x7fffffff {
        Err(Error)
    } else {
        Ok(n)
    }
}
fn ok_hex(bytes: &[u8]) -> Zeroizing<String> {
    // Build secret-bearing responses directly into the buffer main wipes.
    let mut out = Zeroizing::new(String::with_capacity(3 + bytes.len() * 2));
    out.push_str("ok ");
    if bytes.is_empty() {
        out.push('-');
    } else {
        const HEX: &[u8] = b"0123456789abcdef";
        for b in bytes {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 15) as usize] as char);
        }
    }
    out
}
fn bundle_line(bundle: &Bundle) -> String {
    format!(
        "ok {} {} {} {} {} {}",
        encode(&bundle.identity),
        encode(&bundle.signed_prekey),
        encode(&bundle.signature),
        bundle.signed_prekey_id,
        encode(&bundle.prekey),
        bundle.prekey_id
    )
}
fn command(state: &mut State, line: &str) -> Result<Zeroizing<String>> {
    if line.len() > MAX_LINE || line.bytes().any(|b| !(0x20..=0x7e).contains(&b)) {
        return Err(Error);
    }
    // At most seven tokens; reject repeated spaces and ambiguous framing.
    let tokens: Vec<_> = line.split(' ').take(8).collect();
    if tokens.len() > 7 || tokens.iter().any(|s| s.is_empty()) {
        return Err(Error);
    }
    if state.pending && !matches!(tokens[0], "commit" | "abort") {
        return Err(Error);
    }
    match tokens.as_slice() {
        ["new"] if state.device.is_none() => {
            let device = Device::fresh()?;
            let reply = bundle_line(&device.bundle()?);
            state.device = Some(device);
            Ok(Zeroizing::new(reply))
        }
        ["import", snapshot] if state.device.is_none() => {
            let bytes = Zeroizing::new(decode(snapshot, MAX_SNAPSHOT)?);
            state.device = Some(Device::import(&bytes)?);
            Ok(Zeroizing::new("ok".into()))
        }
        ["convert", ed] => Ok(Zeroizing::new(format!(
            "ok {}",
            encode(&convert(&fixed(ed)?))
        ))),
        ["trust", pin] => {
            state.device.as_mut().ok_or(Error)?.trust(pin)?;
            Ok(Zeroizing::new("ok".into()))
        }
        ["start", ik, spk, sig, spkid, pk, pkid] => {
            let bundle = Bundle {
                identity: fixed(ik)?,
                signed_prekey: fixed(spk)?,
                signature: fixed(sig)?,
                signed_prekey_id: id(spkid)?,
                prekey: fixed(pk)?,
                prekey_id: id(pkid)?,
            };
            state.device.as_mut().ok_or(Error)?.start(&bundle)?;
            Ok(Zeroizing::new("ok".into()))
        }
        ["seal", plain] => {
            let bytes = Zeroizing::new(decode(plain, MAX_PLAINTEXT)?);
            let message = state.device.as_mut().ok_or(Error)?.seal(&bytes)?;
            Ok(Zeroizing::new(format!(
                "ok {} {} {}",
                encode(&message.key),
                u8::from(message.prekey),
                encode(&message.payload)
            )))
        }
        ["open", key, prekey, payload] => {
            let prekey = match *prekey {
                "0" => false,
                "1" => true,
                _ => return Err(Error),
            };
            let message = Message {
                key: decode(key, MAX_KEY)?,
                prekey,
                payload: decode(payload, MAX_PAYLOAD)?,
            };
            let device = state.device.as_mut().ok_or(Error)?;
            let plaintext = device.open_pending(&message)?;
            state.pending = true;
            // This is the product codec, not the reference's independent oracle.
            // No candidate plaintext leaves this process before local acceptance.
            if sce::decode(&plaintext, "reference@b.test", "native@a.test", STAMP).is_err() {
                device.abort()?;
                state.pending = false;
                return Err(Error);
            }
            Ok(ok_hex(&plaintext))
        }
        ["commit"] => {
            state.device.as_mut().ok_or(Error)?.commit()?;
            state.pending = false;
            Ok(Zeroizing::new("ok".into()))
        }
        ["abort"] => {
            state.device.as_mut().ok_or(Error)?.abort()?;
            state.pending = false;
            Ok(Zeroizing::new("ok".into()))
        }
        ["export"] => {
            let snapshot = state.device.as_ref().ok_or(Error)?.export()?;
            Ok(ok_hex(&snapshot))
        }
        _ => Err(Error),
    }
}

struct Frame {
    line: Zeroizing<Vec<u8>>,
    rejected: bool,
    eof: bool,
}
/// The fixed scratch buffer and copied lines are zeroized on consumption/drop.
struct Reader<R> {
    input: R,
    scratch: Zeroizing<[u8; 4096]>,
    pos: usize,
    end: usize,
}
impl<R: Read> Reader<R> {
    fn new(input: R) -> Self {
        Self {
            input,
            scratch: Zeroizing::new([0; 4096]),
            pos: 0,
            end: 0,
        }
    }
    fn frame(&mut self) -> io::Result<Option<Frame>> {
        let mut line = Zeroizing::new(Vec::with_capacity(MAX_LINE + 1));
        let mut rejected = false;
        loop {
            if self.pos == self.end {
                self.scratch.zeroize();
                self.pos = 0;
                self.end = loop {
                    match self.input.read(&mut *self.scratch) {
                        Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                        result => break result?,
                    }
                };
                if self.end == 0 {
                    return if line.is_empty() && !rejected {
                        Ok(None)
                    } else {
                        Ok(Some(Frame {
                            line,
                            rejected: true,
                            eof: true,
                        }))
                    };
                }
            }
            let available = &self.scratch[self.pos..self.end];
            let newline = available.iter().position(|b| *b == b'\n');
            let take = newline.map_or(available.len(), |n| n + 1);
            if !rejected {
                if line.len() + take > MAX_LINE + 1 {
                    rejected = true;
                    line.zeroize();
                } else {
                    line.extend_from_slice(&available[..take]);
                }
            }
            self.scratch[self.pos..self.pos + take].zeroize();
            self.pos += take;
            if newline.is_some() {
                if !rejected {
                    line.pop();
                }
                return Ok(Some(Frame {
                    line,
                    rejected,
                    eof: false,
                }));
            }
        }
    }
}
fn run(input: impl Read, mut output: impl Write) -> io::Result<()> {
    let mut reader = Reader::new(input);
    let mut state = State::default();
    while let Some(frame) = reader.frame()? {
        let reply = if frame.rejected {
            Zeroizing::new("err".to_owned())
        } else {
            std::str::from_utf8(&frame.line)
                .ok()
                .and_then(|line| command(&mut state, line).ok())
                .unwrap_or_else(|| Zeroizing::new("err".to_owned()))
        };
        output.write_all(reply.as_bytes())?;
        output.write_all(b"\n")?;
        output.flush()?;
        if frame.eof {
            break;
        }
    }
    Ok(())
}
fn private_pipe(file: &File) -> io::Result<bool> {
    let kind = file.metadata()?.file_type();
    Ok(kind.is_fifo() || kind.is_socket())
}
fn main() -> io::Result<()> {
    if std::env::args_os().len() != 1 {
        return Err(io::Error::other("no arguments accepted"));
    }
    let input = File::from(io::stdin().as_fd().try_clone_to_owned()?);
    let output = File::from(io::stdout().as_fd().try_clone_to_owned()?);
    if !private_pipe(&input)? || !private_pipe(&output)? {
        return Err(io::Error::other("private input/output pipes required"));
    }
    run(input, output)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_product_sce_is_pending_until_commit_and_bad_sce_aborts() {
        let mut sender = Device::fresh().unwrap();
        let mut receiver = Device::fresh().unwrap();
        sender
            .trust(&format!(
                "ed25519:{}",
                encode(&receiver.bundle().unwrap().identity)
            ))
            .unwrap();
        receiver
            .trust(&format!(
                "ed25519:{}",
                encode(&sender.bundle().unwrap().identity)
            ))
            .unwrap();
        sender.start(&receiver.bundle().unwrap()).unwrap();
        let mut state = State {
            device: Some(receiver),
            pending: false,
        };
        let raw = sce::encode(
            "reference@b.test",
            "native@a.test",
            STAMP,
            "subject",
            "body",
        )
        .unwrap();
        let valid = sender.seal(&raw).unwrap();
        let line = format!(
            "open {} {} {}",
            encode(&valid.key),
            u8::from(valid.prekey),
            encode(&valid.payload)
        );
        let response = command(&mut state, &line).unwrap();
        assert_eq!(response.as_str(), format!("ok {}", encode(&raw)));
        assert!(state.pending);
        assert!(command(&mut state, "export").is_err());
        command(&mut state, "commit").unwrap();
        let before = state.device.as_ref().unwrap().export().unwrap();
        let invalid = sender.seal(b"authenticated but invalid SCE").unwrap();
        let line = format!(
            "open {} {} {}",
            encode(&invalid.key),
            u8::from(invalid.prekey),
            encode(&invalid.payload)
        );
        assert!(command(&mut state, &line).is_err());
        assert!(!state.pending);
        assert_eq!(before, state.device.as_ref().unwrap().export().unwrap());
        let successor = sender.seal(&raw).unwrap();
        let line = format!(
            "open {} {} {}",
            encode(&successor.key),
            u8::from(successor.prekey),
            encode(&successor.payload)
        );
        assert!(command(&mut state, &line).is_ok());
        command(&mut state, "abort").unwrap();
    }
    #[test]
    fn oversized_lines_drain_before_next_command_and_incomplete_eof_refuses() {
        let mut input = vec![b'x'; MAX_LINE + 3];
        input.extend_from_slice(b"\nnew\n");
        let mut output = Zeroizing::new(Vec::new());
        run(input.as_slice(), &mut *output).unwrap();
        let text = std::str::from_utf8(&output).unwrap();
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "err");
        assert!(lines[1].starts_with("ok "));
        let mut output = Vec::new();
        run(b"new".as_slice(), &mut output).unwrap();
        assert_eq!(output, b"err\n");
        assert!(!private_pipe(&File::open("/dev/null").unwrap()).unwrap());
    }
}
