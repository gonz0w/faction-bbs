//! One pending TCP connection, using the original admission clock.
use super::*;
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    net::{self, AddressFamily, SocketFlags, SocketType},
};
pub(super) fn connect_budget(deadline: Instant) -> Duration {
    deadline.saturating_duration_since(Instant::now())
}
fn current(deadline: Instant, cancel: &AtomicBool, probe: &impl Fn() -> bool) -> Result<(), Stage> {
    check(deadline, cancel)?;
    if !probe() {
        return Err(Stage::Denied);
    }
    check(deadline, cancel)
}
pub(super) fn connect(
    address: &SocketAddr,
    deadline: Instant,
    cancel: &AtomicBool,
    probe: impl Fn() -> bool,
) -> Result<TcpStream, Stage> {
    current(deadline, cancel, &probe)?;
    let domain = if address.is_ipv4() {
        AddressFamily::INET
    } else {
        AddressFamily::INET6
    };
    #[cfg(any(target_os = "linux", target_os = "netbsd"))]
    let flags = SocketFlags::CLOEXEC | SocketFlags::NONBLOCK;
    #[cfg(not(any(target_os = "linux", target_os = "netbsd")))]
    let flags = SocketFlags::empty();
    let fd = net::socket_with(domain, SocketType::STREAM, flags, Some(net::ipproto::TCP))
        .map_err(|_| Stage::Admission)?;
    #[cfg(not(any(target_os = "linux", target_os = "netbsd")))]
    {
        // macOS has no atomic SOCK_CLOEXEC/SOCK_NONBLOCK. The foreground
        // operator does not exec while a worker is active; set both immediately.
        // This fallback has a small creation-to-CLOEXEC window.
        rustix::io::fcntl_setfd(&fd, rustix::io::FdFlags::CLOEXEC).map_err(|_| Stage::Admission)?;
        let flags = rustix::fs::fcntl_getfl(&fd).map_err(|_| Stage::Admission)?;
        rustix::fs::fcntl_setfl(&fd, flags | rustix::fs::OFlags::NONBLOCK)
            .map_err(|_| Stage::Admission)?;
    }
    current(deadline, cancel, &probe)?;
    // There is exactly one connect syscall. Every retry below polls this same
    // pending socket; it never restarts TCP or either original deadline.
    match net::connect(&fd, address) {
        Ok(()) => (),
        Err(error)
            if error == rustix::io::Errno::INPROGRESS || error == rustix::io::Errno::INTR =>
        {
            loop {
                current(deadline, cancel, &probe)?;
                let timeout = Timespec::try_from(connect_budget(deadline).min(CANCEL_SLICE))
                    .map_err(|_| Stage::Admission)?;
                let mut pending = [PollFd::new(&fd, PollFlags::OUT)];
                let ready = poll(&mut pending, Some(&timeout));
                current(deadline, cancel, &probe)?;
                match ready {
                    Err(error) if error == rustix::io::Errno::INTR => continue,
                    Err(_) => return Err(Stage::Admission),
                    Ok(0) => continue,
                    Ok(_) => (),
                }
                // Both the getsockopt operation and its reported socket error
                // must succeed. Readiness alone does not establish TCP.
                net::sockopt::socket_error(&fd)
                    .map_err(|_| Stage::Admission)?
                    .map_err(|_| Stage::Admission)?;
                if !pending[0].revents().contains(PollFlags::OUT) {
                    return Err(Stage::Admission);
                }
                break;
            }
        }
        Err(_) => return Err(classify(deadline, cancel, Stage::Admission)),
    }
    current(deadline, cancel, &probe)?;
    let socket = TcpStream::from(fd);
    if socket.peer_addr().map_err(|_| Stage::Admission)? != *address {
        return Err(Stage::Admission);
    }
    socket
        .set_nonblocking(false)
        .map_err(|_| Stage::Admission)?;
    current(deadline, cancel, &probe)?;
    Ok(socket)
}
