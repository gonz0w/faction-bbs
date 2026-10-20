//! Trusted local worker supervisor. Does not speak SSH or accept owner RPC commands.
#![forbid(unsafe_code)]
use faction_access::{Result, check_directory, denied, receive_terminal};
use std::{
    fs,
    io::{Read, Write},
    os::unix::{
        fs::{MetadataExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
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
) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    // Atomic CMSG_CLOEXEC on NetBSD/Linux; also serialize receive->spawn for
    // platforms without it, so no transient foreign descriptor reaches a child.
    let spawn_guard = SPAWN_LOCK.lock().map_err(|_| denied())?;
    let (tty, terminal) = receive_terminal(&stream, uid)?;
    let mut child = ChildGuard(
        Command::new(worker)
            .arg("--state-dir")
            .arg(state)
            .args(theme_args)
            .env_clear()
            .env("TERM", terminal)
            .stdin(Stdio::from(tty.try_clone()?))
            .stdout(Stdio::from(tty.try_clone()?))
            .stderr(Stdio::from(tty))
            .spawn()?,
    );
    drop(spawn_guard);
    stream.set_read_timeout(Some(Duration::from_millis(200)))?;
    let deadline = Instant::now() + Duration::from_secs(1800);
    let mut byte = [0];
    loop {
        if let Some(status) = child.0.try_wait()? {
            stream.write_all(&[u8::from(!status.success())])?;
            return Ok(());
        }
        if Instant::now() >= deadline {
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
    let theme_args = args[8..].to_vec();
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
    if fs::symlink_metadata(&path).is_ok() {
        return Err(denied());
    }
    rustix::process::umask(rustix::fs::Mode::from_raw_mode(0o077));
    let listener = UnixListener::bind(&path)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o660))?;
    let active = Arc::new(AtomicUsize::new(0));
    let mut tokens = LIMIT;
    let mut refill = Instant::now();
    for incoming in listener.incoming() {
        let stream = incoming?;
        let elapsed = refill.elapsed().as_secs() / 5;
        if elapsed > 0 {
            tokens = (tokens + elapsed.min(LIMIT as u64) as usize).min(LIMIT);
            refill = Instant::now();
        }
        if tokens == 0 || active.load(Ordering::SeqCst) >= LIMIT {
            drop(stream);
            continue;
        }
        tokens -= 1;
        active.fetch_add(1, Ordering::SeqCst);
        let slot = Slot(active.clone());
        let worker = worker.clone();
        let state = state.clone();
        let theme_args = theme_args.clone();
        let _ = thread::Builder::new().name("bbs-session".into()).stack_size(128 * 1024).spawn(move || {
            let _slot = slot;
            if session(stream, terminal_uid, worker, state, theme_args).is_err() { /* no content/credentials in operational logs */
            }
        });
    }
    Ok(())
}
fn main() {
    if run().is_err() {
        eprintln!("Faction broker: configuration or local transport failed.");
        std::process::exit(1)
    }
}
