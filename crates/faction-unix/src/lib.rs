//! Narrow native descriptor/signal bridge. Application crates forbid unsafe.
#![deny(unsafe_op_in_unsafe_fn)]
pub mod operator;
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

unsafe extern "C" {
    fn faction_install_shutdown() -> i32;
    fn faction_shutdown_requested() -> i32;
}
/// Install process-global TERM/INT/HUP handlers. Call once during broker startup.
/// A handler only stores into a C11 lock-free atomic; no Rust executes in it.
pub fn install_shutdown() -> std::io::Result<()> {
    // SAFETY: no pointers or borrowed state cross this call. C owns the static
    // atomic/handlers for process lifetime. On partial failure the broker exits.
    if unsafe { faction_install_shutdown() } == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
pub fn shutdown_requested() -> bool {
    // SAFETY: reads the process-lifetime C lock-free atomic, with no pointers.
    unsafe { faction_shutdown_requested() != 0 }
}

unsafe extern "C" {
    fn faction_prepare_process() -> i32;
    fn faction_process_limits(service: i32, sessions: u32) -> i32;
    fn faction_peer_uid(socket: i32, uid: *mut u32) -> i32;
}
/// Sanitize a dedicated process before opening state or starting threads.
/// Checks inherited descriptors/identity, sets cwd/CLOEXEC and hard core/memlock limits.
/// Invoke once only from the single-threaded service/frontend startup path.
pub fn prepare_process() -> std::io::Result<()> {
    // SAFETY: no pointers cross the call. C rejects unknown inherited descriptors
    // rather than closing Rust-owned files, and never mutates the environment.
    if unsafe { faction_prepare_process() } == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
/// Lock the existing fixed-worker service to its process/thread/FD envelope.
pub fn process_limits(sessions: u32) -> std::io::Result<()> {
    // SAFETY: scalar arguments only. C validates range and verifies hard limits.
    if unsafe { faction_process_limits(1, sessions) } == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
/// Kernel-recorded connected Unix peer UID; independent of socket file ownership.
pub fn peer_uid(socket: std::os::fd::BorrowedFd<'_>) -> std::io::Result<u32> {
    use std::os::fd::AsRawFd;
    let mut uid = 0;
    // SAFETY: borrowed FD remains live; output points to one writable u32 for the
    // duration of the call. No pointer/descriptor is retained or transferred.
    if unsafe { faction_peer_uid(socket.as_raw_fd(), &mut uid) } == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(uid)
    }
}

#[cfg(test)]
mod peer_tests {
    #[test]
    fn peer_identity_comes_from_connected_kernel_socket() {
        use std::os::{
            fd::AsFd,
            unix::net::{UnixListener, UnixStream},
        };
        let path = std::env::temp_dir().join(format!("faction-peer-{}", std::process::id()));
        let listener = UnixListener::bind(&path).expect("private test socket");
        let client = UnixStream::connect(&path).expect("connect");
        let server = listener.accept().expect("accept").0;
        // Equal on both directions; compare to the OS-created file owner rather
        // than a user-supplied wire field.
        use std::os::unix::fs::MetadataExt;
        let owner = std::fs::metadata(&path).expect("socket metadata").uid();
        assert_eq!(super::peer_uid(client.as_fd()).unwrap(), owner);
        assert_eq!(super::peer_uid(server.as_fd()).unwrap(), owner);
        assert!(super::peer_uid(std::fs::File::open("/dev/null").unwrap().as_fd()).is_err());
        std::fs::remove_file(path).unwrap();
    }
}

unsafe extern "C" {
    fn faction_calendar_init();
    fn faction_calendar_date(
        seconds: i64,
        local: i32,
        year: *mut i32,
        month: *mut i32,
        day: *mut i32,
    ) -> i32;
}
static CALENDAR_READY: std::sync::Once = std::sync::Once::new();
/// Initialize trusted process/OS timezone rules once, before shared helper threads.
/// Does not change the environment, clock, host configuration or retain Rust data.
pub fn init_calendar() {
    CALENDAR_READY.call_once(|| {
        // SAFETY: scalar/no-pointer call; libc owns its process-lifetime timezone
        // state. The application never mutates TZ while reading calendar dates.
        unsafe { faction_calendar_init() };
    });
}
/// Presentation-only date. Local conversion requires startup initialization;
/// failure/out-of-range time returns None and cannot affect account authority.
pub fn calendar_date(seconds: i64, local: bool) -> Option<(u16, u8, u8)> {
    if local && !CALENDAR_READY.is_completed() {
        return None;
    }
    let (mut year, mut month, mut day) = (0, 0, 0);
    // SAFETY: three distinct writable i32 locals remain live for this call; C
    // writes only on success and retains no pointer. Timestamp/local are scalars.
    let status = unsafe {
        faction_calendar_date(seconds, i32::from(local), &mut year, &mut month, &mut day)
    };
    if status != 0 {
        return None;
    }
    Some((
        year.try_into().ok()?,
        month.try_into().ok()?,
        day.try_into().ok()?,
    ))
}
#[cfg(test)]
mod calendar_tests {
    #[test]
    fn utc_calendar_boundaries_and_invalid_times() {
        assert_eq!(super::calendar_date(0, false), Some((1970, 1, 1)));
        assert_eq!(super::calendar_date(1709164800, false), Some((2024, 2, 29)));
        assert_eq!(
            super::calendar_date(1798761599, false),
            Some((2026, 12, 31))
        );
        assert_eq!(super::calendar_date(1798761600, false), Some((2027, 1, 1)));
        assert!(super::calendar_date(-1, false).is_none());
        assert!(super::calendar_date(i64::MAX, false).is_none());
        super::init_calendar();
        assert!(super::calendar_date(1709164800, true).is_some());
    }
}
