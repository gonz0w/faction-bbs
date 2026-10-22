//! Client-only PTY bridge. No fork, credential or BBS authority.
#![deny(unsafe_op_in_unsafe_fn)]
use std::{
    io,
    os::fd::{FromRawFd, OwnedFd},
};
unsafe extern "C" {
    fn faction_client_openpty(master: *mut i32, slave: *mut i32, rows: u16, cols: u16) -> i32;
}
pub fn open(rows: u16, cols: u16) -> io::Result<(OwnedFd, OwnedFd)> {
    let (mut master, mut slave) = (-1, -1);
    // SAFETY: outputs are writable for the call. C either closes all rejected
    // descriptors or transfers two distinct newly allocated CLOEXEC descriptors.
    if unsafe { faction_client_openpty(&mut master, &mut slave, rows, cols) } < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful allocation transfers exclusive ownership exactly once.
    Ok(unsafe { (OwnedFd::from_raw_fd(master), OwnedFd::from_raw_fd(slave)) })
}
