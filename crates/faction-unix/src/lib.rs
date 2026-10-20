//! Narrow NetBSD CMSG ABI bridge. All application crates still forbid unsafe.
#![deny(unsafe_op_in_unsafe_fn)]
#[cfg(target_os = "netbsd")]
mod netbsd {
    use std::{
        io,
        os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd},
    };
    unsafe extern "C" {
        fn faction_send_fd(socket: i32, terminal: i32, bytes: *const u8, length: usize) -> isize;
        fn faction_recv_fd(socket: i32, bytes: *mut u8, length: usize, terminal: *mut i32)
        -> isize;
    }
    pub fn send(
        socket: BorrowedFd<'_>,
        terminal: BorrowedFd<'_>,
        bytes: &[u8],
    ) -> io::Result<usize> {
        // SAFETY: borrowed descriptors stay live; bytes points to length readable
        // bytes for this call. C borrows both descriptors and retains no pointers.
        let n = unsafe {
            faction_send_fd(
                socket.as_raw_fd(),
                terminal.as_raw_fd(),
                bytes.as_ptr(),
                bytes.len(),
            )
        };
        if n < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(n as usize)
        }
    }
    pub fn receive(socket: BorrowedFd<'_>, bytes: &mut [u8]) -> io::Result<(usize, OwnedFd)> {
        let mut fd = -1;
        // SAFETY: writable byte/output pointers have the stated capacity and live
        // until return. C closes every rejected right; success transfers exactly
        // one newly received, CLOEXEC descriptor to the caller, never a borrowed fd.
        let n = unsafe {
            faction_recv_fd(socket.as_raw_fd(), bytes.as_mut_ptr(), bytes.len(), &mut fd)
        };
        if n < 0 {
            return Err(io::Error::last_os_error());
        }
        if fd < 0 {
            return Err(io::Error::other("missing terminal right"));
        }
        // SAFETY: the C success contract above transfers unique ownership here.
        Ok((n as usize, unsafe { OwnedFd::from_raw_fd(fd) }))
    }
}
#[cfg(target_os = "netbsd")]
pub use netbsd::{receive, send};

#[cfg(all(test, target_os = "netbsd"))]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        os::{fd::AsFd, unix::net::UnixStream},
    };
    #[test]
    fn native_descriptor_roundtrip_preserves_the_actual_open_file() {
        let (control_a, control_b) = UnixStream::pair().unwrap();
        let (mut data_a, mut data_b) = UnixStream::pair().unwrap();
        assert_eq!(
            send(control_a.as_fd(), data_a.as_fd(), b"frame").unwrap(),
            5
        );
        let mut bytes = [0u8; 16];
        let (n, fd) = receive(control_b.as_fd(), &mut bytes).unwrap();
        assert_eq!(&bytes[..n], b"frame");
        let mut transferred = UnixStream::from(fd);
        data_b.write_all(b"actual descriptor").unwrap();
        let mut proof = [0; 17];
        transferred.read_exact(&mut proof).unwrap();
        assert_eq!(&proof, b"actual descriptor");
        drop(transferred);
        data_b.write_all(b"original alive").unwrap();
        let mut proof = [0; 14];
        data_a.read_exact(&mut proof).unwrap();
        assert_eq!(&proof, b"original alive");
    }
}
