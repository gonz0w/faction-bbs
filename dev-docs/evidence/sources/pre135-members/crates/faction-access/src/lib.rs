//! Local Unix terminal handoff only; no database, SSH crypto or member authority.
#![forbid(unsafe_code)]
#[cfg(not(target_os = "netbsd"))]
use rustix::{
    fd::AsFd,
    net::{
        self, RecvAncillaryBuffer, RecvAncillaryMessage, RecvFlags, ReturnFlags,
        SendAncillaryBuffer, SendAncillaryMessage, SendFlags,
    },
};
#[cfg(target_os = "netbsd")]
use std::os::fd::AsFd;
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    os::unix::{fs::MetadataExt, net::UnixStream},
    path::Path,
};
#[cfg(not(target_os = "netbsd"))]
use std::{
    io::{IoSlice, IoSliceMut},
    mem::MaybeUninit,
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub fn denied() -> Box<dyn std::error::Error + Send + Sync> {
    io::Error::new(io::ErrorKind::PermissionDenied, "terminal admission denied").into()
}
pub fn term(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_+.".contains(&b))
}
pub fn check_directory(path: &Path, owner: u32) -> Result<()> {
    let m = fs::symlink_metadata(path)?;
    if !m.is_dir() || m.uid() != owner || m.mode() & 0o027 != 0 {
        return Err(denied());
    } // owner rwx, optional group search/read only
    Ok(())
}
pub fn send_terminal(stream: &UnixStream, tty: &File, terminal: &str) -> Result<()> {
    if !term(terminal) {
        return Err(denied());
    }
    let mut packet = [0u8; 72];
    packet[..8].copy_from_slice(b"FBSTTY01");
    packet[8] = terminal.len() as u8;
    packet[9..9 + terminal.len()].copy_from_slice(terminal.as_bytes());
    #[cfg(target_os = "netbsd")]
    let n = faction_unix::send(stream.as_fd(), tty.as_fd(), &packet)?;
    #[cfg(not(target_os = "netbsd"))]
    let n = {
        let mut space = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
        let mut control = SendAncillaryBuffer::new(&mut space);
        let fds = [tty.as_fd()];
        if !control.push(SendAncillaryMessage::ScmRights(&fds)) {
            return Err(denied());
        }
        net::sendmsg(
            stream,
            &[IoSlice::new(&packet)],
            &mut control,
            SendFlags::empty(),
        )?
    };
    if n == 0 {
        return Err(denied());
    }
    (&*stream).write_all(&packet[n..])?;
    Ok(())
}
pub fn receive_terminal(stream: &UnixStream, terminal_uid: u32) -> Result<(File, String)> {
    let mut packet = [0u8; 72];
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    stream.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;
    #[cfg(target_os = "netbsd")]
    let (used, fd) = faction_unix::receive(stream.as_fd(), &mut packet)?;
    #[cfg(not(target_os = "netbsd"))]
    let (used, fd) = {
        let mut space = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(2))];
        let mut control = RecvAncillaryBuffer::new(&mut space);
        #[cfg(any(
            target_os = "linux",
            target_os = "netbsd",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "dragonfly"
        ))]
        let flags = RecvFlags::CMSG_CLOEXEC;
        #[cfg(not(any(
            target_os = "linux",
            target_os = "netbsd",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "dragonfly"
        )))]
        let flags = RecvFlags::empty();
        let msg = net::recvmsg(
            stream,
            &mut [IoSliceMut::new(&mut packet)],
            &mut control,
            flags,
        )?;
        let mut received = Vec::new();
        for item in control.drain() {
            match item {
                RecvAncillaryMessage::ScmRights(fds) => received.extend(fds),
                _ => return Err(denied()),
            }
        }
        if msg.bytes == 0
            || msg
                .flags
                .intersects(ReturnFlags::CTRUNC | ReturnFlags::TRUNC)
            || received.len() != 1
        {
            return Err(denied());
        }
        (msg.bytes, received.pop().ok_or_else(denied)?)
    };
    let mut used = used;
    while used < packet.len() {
        let remaining = deadline
            .checked_duration_since(std::time::Instant::now())
            .ok_or_else(denied)?;
        stream.set_read_timeout(Some(remaining))?;
        let n = (&*stream).read(&mut packet[used..])?;
        if n == 0 {
            return Err(denied());
        }
        used += n;
    }
    let n = usize::from(packet[8]);
    if &packet[..8] != b"FBSTTY01" || n == 0 || n > 63 || packet[9 + n..].iter().any(|b| *b != 0) {
        return Err(denied());
    }
    let terminal = std::str::from_utf8(&packet[9..9 + n])?;
    if !term(terminal) {
        return Err(denied());
    }
    rustix::io::fcntl_setfd(&fd, rustix::io::FdFlags::CLOEXEC)?;
    if !rustix::termios::isatty(&fd) || rustix::fs::fstat(&fd)?.st_uid != terminal_uid {
        return Err(denied());
    }
    Ok((File::from(fd), terminal.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_names_cannot_carry_environment_or_controls() {
        for s in ["", "x\x1b[2J", "a/b", "a b", "$(id)", &"x".repeat(64)] {
            assert!(!term(s));
        }
        assert!(term("xterm-256color"));
    }
    #[test]
    fn incomplete_or_descriptorless_handoff_is_denied() {
        let (a, b) = UnixStream::pair().unwrap();
        a.shutdown(std::net::Shutdown::Write).unwrap();
        assert!(receive_terminal(&b, 0).is_err());
        let (mut a, b) = UnixStream::pair().unwrap();
        a.write_all(&[0; 72]).unwrap();
        assert!(receive_terminal(&b, 0).is_err());
    }
    #[test]
    fn nonterminal_descriptor_cannot_become_a_worker_input() {
        let (a, b) = UnixStream::pair().unwrap();
        let fd = File::open("/dev/null").unwrap();
        send_terminal(&a, &fd, "xterm").unwrap();
        assert!(receive_terminal(&b, rustix::process::geteuid().as_raw()).is_err());
    }
    #[cfg(not(target_os = "netbsd"))]
    #[test]
    fn extra_descriptors_and_invalid_metadata_are_rejected() {
        for total in [0, 2, 3] {
            let (a, b) = UnixStream::pair().unwrap();
            let file = File::open("/dev/null").unwrap();
            let fds = vec![file.as_fd(); total];
            let mut space = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(3))];
            let mut control = SendAncillaryBuffer::new(&mut space);
            if total > 0 {
                assert!(control.push(SendAncillaryMessage::ScmRights(&fds)));
            }
            net::sendmsg(
                &a,
                &[IoSlice::new(&[0; 72])],
                &mut control,
                SendFlags::empty(),
            )
            .unwrap();
            assert!(receive_terminal(&b, rustix::process::geteuid().as_raw()).is_err());
        }
    }
}
