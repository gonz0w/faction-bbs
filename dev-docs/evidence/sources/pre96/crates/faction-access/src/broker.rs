//! Trusted local worker supervisor. Does not speak SSH or accept owner RPC commands.
#![forbid(unsafe_code)]
use faction_access::{Result, check_directory, denied, receive_terminal};
use std::{
    fs,
    io::{Read, Write},
    os::unix::{
        fs::{FileTypeExt, MetadataExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
const LIMIT: usize = 8;
static SPAWN_LOCK: Mutex<()> = Mutex::new(());
struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
struct Slot(Arc<AtomicUsize>);
impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
fn session(
    mut stream: UnixStream,
    uid: u32,
    worker: PathBuf,
    state: PathBuf,
    theme_args: Vec<std::ffi::OsString>,
    stopping: Arc<AtomicBool>,
    session_seconds: u64,
) -> Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    // Atomic CMSG_CLOEXEC on NetBSD/Linux; also serialize receive->spawn for
    // platforms without it, so no transient foreign descriptor reaches a child.
    let spawn_guard = loop {
        if stopping.load(Ordering::SeqCst) {
            return Err(denied());
        }
        match SPAWN_LOCK.try_lock() {
            Ok(guard) => break guard,
            Err(std::sync::TryLockError::WouldBlock) => thread::sleep(Duration::from_millis(20)),
            Err(_) => return Err(denied()),
        }
    };
    let (tty, terminal) = receive_terminal(&stream, uid)?;
    if stopping.load(Ordering::SeqCst) {
        return Err(denied());
    }
    let mut child = ChildGuard(
        Command::new(worker)
            .arg("--state-dir")
            .arg(state)
            .args(theme_args)
            .env_clear()
            .env("TERM", terminal)
            .env("FACTION_BROKER_PID", std::process::id().to_string())
            .stdin(Stdio::from(tty.try_clone()?))
            .stdout(Stdio::from(tty.try_clone()?))
            .stderr(Stdio::from(tty))
            .spawn()?,
    );
    drop(spawn_guard);
    stream.set_read_timeout(Some(Duration::from_millis(200)))?;
    let deadline = Instant::now() + Duration::from_secs(session_seconds);
    let mut byte = [0];
    loop {
        if let Some(status) = child.0.try_wait()? {
            stream.write_all(&[u8::from(!status.success())])?;
            return Ok(());
        }
        if stopping.load(Ordering::SeqCst) || Instant::now() >= deadline {
            return Err(denied());
        }
        match stream.read(&mut byte) {
            Ok(_) => {
                // Allow the now-disconnected SSH PTY to deliver HUP/EOF so the
                // worker can remove chat presence, then enforce bounded teardown.
                for _ in 0..20 {
                    if child.0.try_wait()?.is_some() {
                        return Ok(());
                    }
                    thread::sleep(Duration::from_millis(50));
                }
                return Err(denied());
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(e) => return Err(e.into()),
        }
    }
}
struct SocketGuard {
    path: PathBuf,
    metadata: fs::Metadata,
}
impl Drop for SocketGuard {
    fn drop(&mut self) {
        if let Ok(now) = fs::symlink_metadata(&self.path)
            && now.file_type().is_socket()
            && now.dev() == self.metadata.dev()
            && now.ino() == self.metadata.ino()
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() < 8
        || args[0] != "--socket"
        || args[2] != "--terminal-uid"
        || args[4] != "--worker"
        || args[6] != "--state-dir"
    {
        return Err(denied());
    }
    let mut theme_args = args[8..].to_vec();
    let mut session_seconds = 1800u64;
    // Owner-only experiment controls. Normal service remains eight workers and
    // one replenished launch / five seconds; no production capacity adoption.
    let mut limit = LIMIT;
    let mut launch_ms = 5000u64;
    for flag in ["--experimental-workers", "--experimental-launch-ms"] {
        if let Some(i) = theme_args.iter().position(|a| a == flag) {
            let n: u64 = theme_args
                .get(i + 1)
                .and_then(|a| a.to_str())
                .ok_or_else(denied)?
                .parse()?;
            if flag == "--experimental-workers" {
                if !(1..=250).contains(&n) {
                    return Err(denied());
                }
                limit = n as usize;
            } else {
                if !(100..=5000).contains(&n) {
                    return Err(denied());
                }
                launch_ms = n;
            }
            theme_args.drain(i..i + 2);
        }
    }
    if let Some(i) = theme_args.iter().position(|a| a == "--session-seconds") {
        session_seconds = theme_args
            .get(i + 1)
            .and_then(|a| a.to_str())
            .ok_or_else(denied)?
            .parse()?;
        if !(1..=1800).contains(&session_seconds) {
            return Err(denied());
        }
        theme_args.drain(i..i + 2);
    }
    if !theme_args.len().is_multiple_of(2)
        || theme_args
            .chunks(2)
            .any(|a| a[0] != "--theme" && a[0] != "--season")
    {
        return Err(denied());
    }
    let owner = rustix::process::geteuid().as_raw();
    let terminal_uid: u32 = args[3].to_str().ok_or_else(denied)?.parse()?;
    if owner == 0 || terminal_uid == 0 || terminal_uid == owner {
        return Err(denied());
    }
    let path = PathBuf::from(&args[1]);
    let worker = PathBuf::from(&args[5]);
    let state = PathBuf::from(&args[7]);
    check_directory(path.parent().ok_or_else(denied)?, owner)?;
    let m = fs::symlink_metadata(&worker)?;
    if !worker.is_absolute()
        || !m.is_file()
        || ![0, owner].contains(&m.uid())
        || m.mode() & 0o022 != 0
        || m.mode() & 0o111 == 0
    {
        return Err(denied());
    }
    let m = fs::symlink_metadata(&state)?;
    if !m.is_dir()
        || m.uid() != owner
        || m.mode() & 0o077 != 0
        || !state.join("faction.sqlite3").is_file()
    {
        return Err(denied());
    }
    // The persistent pidfile doubles as a kernel-held single-instance lock.
    // It is never unlinked: removing a locked inode permits two independent locks.
    rustix::process::umask(rustix::fs::Mode::from_raw_mode(0o077));
    let mut pidfile = fs::File::from(rustix::fs::openat(
        rustix::fs::CWD,
        path.with_extension("pid"),
        rustix::fs::OFlags::CREATE
            | rustix::fs::OFlags::RDWR
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::from_raw_mode(0o600),
    )?);
    let m = pidfile.metadata()?;
    if !m.is_file() || m.uid() != owner || m.mode() & 0o077 != 0 || m.nlink() != 1 {
        return Err(denied());
    }
    rustix::fs::flock(
        &pidfile,
        rustix::fs::FlockOperation::NonBlockingLockExclusive,
    )?;
    if let Ok(m) = fs::symlink_metadata(&path) {
        // Recover only an owned, disconnected socket under the exclusive lock.
        if !m.file_type().is_socket() || m.uid() != owner {
            return Err(denied());
        }
        match UnixStream::connect(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => fs::remove_file(&path)?,
            _ => return Err(denied()),
        }
    }
    faction_unix::install_shutdown()?;
    let listener = UnixListener::bind(&path)?;
    let socket = SocketGuard {
        path: path.clone(),
        metadata: fs::symlink_metadata(&path)?,
    };
    fs::set_permissions(&path, fs::Permissions::from_mode(0o660))?;
    listener.set_nonblocking(true)?;
    pidfile.set_len(0)?;
    writeln!(pidfile, "{}", std::process::id())?;
    pidfile.sync_all()?;
    let active = Arc::new(AtomicUsize::new(0));
    let stopping = Arc::new(AtomicBool::new(false));
    let mut threads: Vec<thread::JoinHandle<()>> = Vec::with_capacity(limit);
    let mut tokens = LIMIT;
    let mut refill = Instant::now();
    let mut rejected = 0u64;
    let mut listener_failed = false;
    eprintln!("Faction broker: ready; worker limit {limit}.");
    while !faction_unix::shutdown_requested() {
        let mut index = 0;
        while index < threads.len() {
            if threads[index].is_finished() {
                let _ = threads.swap_remove(index).join();
            } else {
                index += 1;
            }
        }
        let stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => {
                eprintln!("Faction broker: listener failed: {}.", e.kind());
                listener_failed = true;
                break;
            }
        };
        let elapsed = refill.elapsed().as_millis() / u128::from(launch_ms);
        if elapsed > 0 {
            tokens = (tokens + elapsed.min(LIMIT as u128) as usize).min(LIMIT);
            refill += Duration::from_millis(elapsed as u64 * launch_ms);
        }
        if tokens == 0 || active.load(Ordering::SeqCst) >= limit {
            rejected = rejected.saturating_add(1);
            drop(stream);
            continue;
        }
        tokens -= 1;
        active.fetch_add(1, Ordering::SeqCst);
        let slot = Slot(active.clone());
        let worker = worker.clone();
        let state = state.clone();
        let theme_args = theme_args.clone();
        let stopping = stopping.clone();
        match thread::Builder::new()
            .name("bbs-session".into())
            .stack_size(128 * 1024)
            .spawn(move || {
                let _slot = slot;
                let _ = session(
                    stream,
                    terminal_uid,
                    worker,
                    state,
                    theme_args,
                    stopping,
                    session_seconds,
                );
            }) {
            Ok(handle) => threads.push(handle),
            Err(_) => eprintln!("Faction broker: supervisor thread unavailable."),
        }
    }
    // Stop new connections before draining existing sessions. On every ordinary
    // exit the thread joins reap children before the socket/instance lock release.
    drop(listener);
    let deadline = Instant::now() + Duration::from_secs(5);
    while active.load(Ordering::SeqCst) != 0 && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(50));
    }
    stopping.store(true, Ordering::SeqCst);
    for handle in threads {
        let _ = handle.join();
    }
    drop(socket);
    pidfile.set_len(0)?;
    eprintln!("Faction broker: stopped; rejected admissions {rejected}.");
    if listener_failed {
        return Err(denied());
    }
    Ok(())
}
fn main() {
    if run().is_err() {
        eprintln!("Faction broker: configuration or local transport failed.");
        std::process::exit(1)
    }
}
