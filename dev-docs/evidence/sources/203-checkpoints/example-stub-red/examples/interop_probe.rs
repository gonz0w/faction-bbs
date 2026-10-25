//! Laboratory-only private-pipe adapter for the unchanged independent reference.
//! Never install this raw-state protocol as a member UI or product transport.
use faction_omemo::{Bundle, Device, Error, Message, Result, convert, decode, encode, sce};
use std::{fs::File, io::{self, Read, Write}, os::unix::fs::FileTypeExt};
use zeroize::Zeroizing;
const MAX_LINE: usize = 65568;
const STAMP: u64 = 1792843200;
#[derive(Default)]
struct State { device: Option<Device>, pending: bool }
fn command(_: &mut State, _: &str) -> Result<Zeroizing<String>> { Err(Error) }
fn run(_: impl Read, _: impl Write) -> io::Result<()> { Ok(()) }
fn private_pipe(file: &File) -> io::Result<bool> { let kind=file.metadata()?.file_type(); Ok(kind.is_fifo() || kind.is_socket()) }
fn main() -> io::Result<()> { Ok(()) }
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_product_sce_is_pending_until_commit_and_bad_sce_aborts() {
        let mut sender=Device::fresh().unwrap(); let mut receiver=Device::fresh().unwrap();
        sender.trust(&format!("ed25519:{}",encode(&receiver.bundle().unwrap().identity))).unwrap();
        receiver.trust(&format!("ed25519:{}",encode(&sender.bundle().unwrap().identity))).unwrap();
        sender.start(&receiver.bundle().unwrap()).unwrap();
        let mut state=State {device:Some(receiver),pending:false};
        let raw=sce::encode("reference@b.test","native@a.test",STAMP,"subject","body").unwrap();
        let valid=sender.seal(&raw).unwrap();
        let line=format!("open {} {} {}",encode(&valid.key),u8::from(valid.prekey),encode(&valid.payload));
        let response=command(&mut state,&line).unwrap();
        assert_eq!(response.as_str(),format!("ok {}",encode(&raw)));
        assert!(state.pending);assert!(command(&mut state,"export").is_err());
        command(&mut state,"commit").unwrap();
        let before=state.device.as_ref().unwrap().export().unwrap();
        let invalid=sender.seal(b"authenticated but invalid SCE").unwrap();
        let line=format!("open {} {} {}",encode(&invalid.key),u8::from(invalid.prekey),encode(&invalid.payload));
        assert!(command(&mut state,&line).is_err());
        assert!(!state.pending);
        assert_eq!(before,state.device.as_ref().unwrap().export().unwrap());
        let successor=sender.seal(&raw).unwrap();
        let line=format!("open {} {} {}",encode(&successor.key),u8::from(successor.prekey),encode(&successor.payload));
        assert!(command(&mut state,&line).is_ok());
        command(&mut state,"abort").unwrap();
    }
    #[test]
    fn oversized_lines_drain_before_next_command_and_incomplete_eof_refuses() {
        let mut input=vec![b'x';MAX_LINE+3];input.extend_from_slice(b"\nnew\n");
        let mut output=Zeroizing::new(Vec::new());run(input.as_slice(),&mut *output).unwrap();
        let text=std::str::from_utf8(&output).unwrap();
        let lines:Vec<_>=text.lines().collect();assert_eq!(lines.len(),2);assert_eq!(lines[0],"err");assert!(lines[1].starts_with("ok "));
        let mut output=Vec::new();run(b"new".as_slice(),&mut output).unwrap();assert_eq!(output,b"err\n");
        assert!(!private_pipe(&File::open("/dev/null").unwrap()).unwrap());
    }
}
