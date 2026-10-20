//! Opt-in shared terminal experiment. Never installed by the normal service.
#![forbid(unsafe_code)]
use faction_access::{Result, check_directory, denied, receive_terminal};
use faction_core::{Board, ChatCommand, ChatKind, ChatLease, Error, Session, Status};
use faction_terminal::{Action, Decoder, Key, LineEditor, LineEvent, Menu, Theme};
mod ui;
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    fs::{OFlags, fcntl_getfl, fcntl_setfl},
    termios::{self, OptionalActions, Termios},
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    os::unix::{
        fs::{FileTypeExt, MetadataExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, SyncSender, TrySendError},
    },
    thread,
    time::{Duration, Instant},
};
use zeroize::{Zeroize, Zeroizing};

const CLIENTS: usize = 250;
const OUTPUT: usize = 32768;
const INPUT: usize = 1152;
enum State {
    Choice,
    Handle(bool),
    Password(bool, String),
    Confirm(String, Zeroizing<String>),
    Authenticating,
    Identity,
    Menu,
    Pause,
    RoomsMore(usize),
    RoomId(bool),
    ReadMore(i64, i64),
    PostRoom,
    PostReply(i64),
    PostBody(i64, Option<i64>),
    PostRetry {
        room: i64,
        reply: Option<i64>,
        body: String,
        key: String,
    },
    Sysop,
    MembersMore(usize),
    MemberStatus(bool),
    RoomSlug,
    RoomTitle(String),
    Chat,
    Closing,
}
struct Request {
    connection: u64,
    register: bool,
    name: String,
    secret: Zeroizing<String>,
}
// A permit covers the full queued -> worker -> unconsumed-result lifetime.
// A disconnected client cannot free a slot while its credential work remains.
struct CredentialPermit(Arc<AtomicUsize>);
impl Drop for CredentialPermit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Release);
    }
}
struct Job {
    request: Request,
    permit: CredentialPermit,
}
struct CredentialJobs {
    sender: SyncSender<Job>,
    pending: Arc<AtomicUsize>,
}
impl CredentialJobs {
    fn new() -> (Self, mpsc::Receiver<Job>) {
        let (sender, receiver) = mpsc::sync_channel(2);
        (
            Self {
                sender,
                pending: Arc::new(AtomicUsize::new(0)),
            },
            receiver,
        )
    }
    fn try_send(&self, request: Request) -> std::result::Result<(), TrySendError<Request>> {
        if self
            .pending
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < 2).then_some(n + 1)
            })
            .is_err()
        {
            return Err(TrySendError::Full(request));
        }
        let job = Job {
            request,
            permit: CredentialPermit(self.pending.clone()),
        };
        match self.sender.try_send(job) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(job)) => Err(TrySendError::Full(job.request)),
            Err(TrySendError::Disconnected(job)) => Err(TrySendError::Disconnected(job.request)),
        }
    }
}
struct Auth {
    _permit: CredentialPermit,
    connection: u64,
    result: faction_core::Result<Option<Session>>,
}
struct Client {
    connection: u64,
    control: UnixStream,
    tty: File,
    saved: Termios,
    flags: OFlags,
    state: State,
    session: Option<Session>,
    lease: Option<ChatLease>,
    channel: String,
    cursor: i64,
    request: i64,
    input: Zeroizing<Vec<u8>>,
    decoder: Decoder,
    output: Vec<u8>,
    written: usize,
    output_started: Instant,
    started: Instant,
    prompt_started: Instant,
    heartbeat: Instant,
    next_poll: Instant,
    focus: usize,
    sysop_display: bool,
    mark_after_flush: Option<(i64, i64)>,
    close: bool,
    success: bool,
    ansi_allowed: bool,
    ansi: bool,
    dimensions: (u16, u16),
    geometry_check: Instant,
    editor: LineEditor,
    skip_lf: bool,
    next_input: Instant,
}
impl Drop for Client {
    fn drop(&mut self) {
        // Best effort while still nonblocking: never hang cleanup behind a peer.
        if self.ansi {
            let _ = self.tty.write(b"\x1b[0m\x1b[?25h\r\n");
        }
        let _ = termios::tcsetattr(&self.tty, OptionalActions::Now, &self.saved);
        let _ = fcntl_setfl(&self.tty, self.flags);
        let _ = self.control.write(&[u8::from(!self.success)]);
    }
}
impl Client {
    fn new(
        connection: u64,
        mut control: UnixStream,
        tty: File,
        term: &str,
        ansi: bool,
    ) -> Result<Self> {
        control.set_nonblocking(true)?;
        let saved = termios::tcgetattr(&tty)?;
        let flags = fcntl_getfl(&tty)?;
        let mut mode = saved.clone();
        mode.make_raw();
        termios::tcsetattr(&tty, OptionalActions::Flush, &mode)?;
        if let Err(e) = fcntl_setfl(&tty, flags | OFlags::NONBLOCK) {
            let _ = termios::tcsetattr(&tty, OptionalActions::Now, &saved);
            return Err(e.into());
        }
        // Control is not a member RPC. EOF or any extra byte terminates this connection.
        let _ = &mut control;
        let now = Instant::now();
        let mut c = Self {
            connection,
            control,
            tty,
            saved,
            flags,
            state: State::Choice,
            session: None,
            lease: None,
            channel: "#lobby".into(),
            cursor: 0,
            request: 1,
            input: Zeroizing::new(Vec::with_capacity(INPUT)),
            decoder: Decoder::new(),
            output: Vec::with_capacity(1024),
            written: 0,
            output_started: now,
            started: now,
            prompt_started: now,
            heartbeat: now,
            next_poll: now,
            focus: 0,
            sysop_display: false,
            mark_after_flush: None,
            close: false,
            success: false,
            ansi_allowed: ansi && !term.is_empty() && term != "dumb",
            ansi: false,
            dimensions: (0, 0),
            geometry_check: now,
            editor: LineEditor::default(),
            skip_lf: false,
            next_input: now,
        };
        c.queue("[L]ogin [R]egister [Q]uit: ");
        Ok(c)
    }
    fn queue(&mut self, text: &str) {
        // Count the encoded size before mutating the queue, including CRLF.
        let mut encoded = 0usize;
        let mut previous = 0;
        for byte in text.bytes() {
            encoded += 1 + usize::from(byte == b'\n' && previous != b'\r');
            previous = byte;
        }
        if self.output.len().saturating_add(encoded) > OUTPUT {
            self.close = true;
            return;
        }
        if self.output.is_empty() {
            self.output_started = Instant::now();
        }
        previous = 0;
        for byte in text.bytes() {
            if byte == b'\n' && previous != b'\r' {
                self.output.push(b'\r');
            }
            self.output.push(byte);
            previous = byte;
        }
    }
    fn safe(text: &str) -> String {
        text.chars()
            .flat_map(|c| {
                if c.is_ascii_graphic() || c == ' ' || c == '\n' {
                    vec![c]
                } else {
                    c.escape_unicode().collect()
                }
            })
            .collect()
    }
    fn error(&mut self, e: Error) {
        self.queue(&format!("Action failed: {e}\n"));
    }
    fn flush(&mut self, board: &mut Board) -> Result<()> {
        if self.written < self.output.len() {
            match self.tty.write(&self.output[self.written..]) {
                Ok(0) => self.close = true,
                Ok(n) => self.written += n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(e.into()),
            }
        }
        if self.written == self.output.len() {
            self.output.clear();
            self.written = 0;
            if let Some((room, through)) = self.mark_after_flush.take() {
                board.mark_read(self.session.as_ref().ok_or_else(denied)?, room, through)?;
            }
            if matches!(self.state, State::Closing) {
                self.close = true;
            }
        }
        Ok(())
    }
}
struct SocketGuard {
    path: PathBuf,
    metadata: fs::Metadata,
}
impl Drop for SocketGuard {
    fn drop(&mut self) {
        if let Ok(m) = fs::symlink_metadata(&self.path)
            && m.file_type().is_socket()
            && m.dev() == self.metadata.dev()
            && m.ino() == self.metadata.ino()
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}
fn bind(path: &Path) -> Result<(UnixListener, File, SocketGuard)> {
    let owner = rustix::process::geteuid().as_raw();
    check_directory(path.parent().ok_or_else(denied)?, owner)?;
    rustix::process::umask(rustix::fs::Mode::from_raw_mode(0o077));
    let mut pid = File::from(rustix::fs::openat(
        rustix::fs::CWD,
        path.with_extension("pid"),
        OFlags::CREATE | OFlags::RDWR | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        rustix::fs::Mode::from_raw_mode(0o600),
    )?);
    let m = pid.metadata()?;
    if !m.is_file() || m.uid() != owner || m.mode() & 0o077 != 0 || m.nlink() != 1 {
        return Err(denied());
    }
    rustix::fs::flock(&pid, rustix::fs::FlockOperation::NonBlockingLockExclusive)?;
    if let Ok(m) = fs::symlink_metadata(path) {
        if !m.file_type().is_socket() || m.uid() != owner {
            return Err(denied());
        }
        match UnixStream::connect(path) {
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => fs::remove_file(path)?,
            _ => return Err(denied()),
        }
    }
    let listener = UnixListener::bind(path)?;
    let guard = SocketGuard {
        path: path.into(),
        metadata: fs::symlink_metadata(path)?,
    };
    fs::set_permissions(path, fs::Permissions::from_mode(0o660))?;
    listener.set_nonblocking(true)?;
    pid.set_len(0)?;
    writeln!(pid, "{}", std::process::id())?;
    pid.sync_all()?;
    Ok((listener, pid, guard))
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 6
        || args[0] != "--socket"
        || args[2] != "--terminal-uid"
        || args[4] != "--state-dir"
    {
        return Err(denied());
    }
    let terminal_uid: u32 = args[3].parse()?;
    let owner = rustix::process::geteuid().as_raw();
    if owner == 0 || terminal_uid == 0 || terminal_uid == owner {
        return Err(denied());
    }
    let state = PathBuf::from(&args[5]);
    if !state.join("faction.sqlite3").is_file() {
        return Err(denied());
    }
    let mut board = Board::open(&state)?;
    // Owner-only startup options. No terminal input can select paths or policy.
    let mut theme = Theme::default();
    let mut ansi = true;
    let mut options = args[6..].iter();
    while let Some(option) = options.next() {
        match option.as_str() {
            "--theme" => {
                let path = options.next().ok_or_else(denied)?;
                theme = Theme::load(Path::new(path)).map_err(|_| denied())?;
            }
            "--season" => {
                let path = options.next().ok_or_else(denied)?;
                theme = theme.with_season(Path::new(path)).map_err(|_| denied())?;
            }
            "--plain" => ansi = false,
            "--ansi" => ansi = true,
            _ => return Err(denied()),
        }
    }

    let (listener, pid, _guard) = bind(Path::new(&args[1]))?;
    faction_unix::install_shutdown()?;
    let stop = Arc::new(AtomicBool::new(false));
    let (admit_tx, admit_rx) = mpsc::sync_channel(8);
    let stopping = stop.clone();
    let handoff = thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(move || {
            let mut tokens = 8;
            let mut refill = Instant::now();
            while !stopping.load(Ordering::SeqCst) {
                let stream = match listener.accept() {
                    Ok((s, _)) => s,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(_) => break,
                };
                let elapsed = refill.elapsed().as_millis() / 100;
                if elapsed > 0 {
                    tokens = (tokens + elapsed.min(8) as usize).min(8);
                    refill = Instant::now();
                }
                if tokens == 0 {
                    continue;
                }
                tokens -= 1;
                if stream.set_nonblocking(false).is_err() {
                    continue;
                }
                if let Ok((tty, term)) = receive_terminal(&stream, terminal_uid) {
                    let _ = admit_tx.try_send((stream, tty, term));
                }
            }
        })?;
    let (jobs, job_rx) = CredentialJobs::new();
    let job_rx = Arc::new(Mutex::new(job_rx));
    let (auth_tx, auth_rx) = mpsc::sync_channel(2);
    let mut workers = Vec::new();
    for _ in 0..2 {
        let mut b = Board::open(&state)?;
        let rx = job_rx.clone();
        let tx = auth_tx.clone();
        workers.push(
            thread::Builder::new()
                .stack_size(256 * 1024)
                .spawn(move || {
                    loop {
                        let job = match rx.lock().expect("credential receiver").recv() {
                            Ok(j) => j,
                            Err(_) => break,
                        };
                        let result = if job.request.register {
                            b.register(&job.request.name, &job.request.secret)
                                .map(|()| None)
                        } else {
                            b.authenticate(&job.request.name, &job.request.secret)
                                .map(Some)
                        };
                        if tx
                            .send(Auth {
                                _permit: job.permit,
                                connection: job.request.connection,
                                result,
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                })?,
        );
    }
    drop(auth_tx);
    let mut clients: Vec<Client> = Vec::with_capacity(CLIENTS);
    let mut generation = 0u64;
    let mut turn = 0usize;
    while !faction_unix::shutdown_requested() {
        while let Ok((control, tty, term)) = admit_rx.try_recv() {
            if clients.len() >= CLIENTS {
                continue;
            }
            let metadata = tty.metadata()?;
            if clients.iter().any(|c| {
                c.tty
                    .metadata()
                    .is_ok_and(|m| m.dev() == metadata.dev() && m.ino() == metadata.ino())
            }) {
                continue;
            }
            generation = generation.checked_add(1).ok_or_else(denied)?;
            if let Ok(client) = Client::new(generation, control, tty, &term, ansi) {
                clients.push(client);
            }
        }
        while let Ok(auth) = auth_rx.try_recv() {
            // IDs are assigned only here, never accepted from terminals, never reused.
            if let Some(c) = clients.iter_mut().find(|c| {
                c.connection == auth.connection && matches!(c.state, State::Authenticating)
            }) {
                c.state = State::Closing;
                match auth.result {
                    Ok(Some(s)) => match board.whoami(&s) {
                        Ok(m) if m.status == Status::Approved => {
                            c.session = Some(s);
                            c.sysop_display = m.role == faction_core::Role::Sysop;
                            c.identity(&theme, &m.handle);
                            c.state = State::Identity;
                        }
                        Ok(_) => {
                            c.success = true;
                            c.queue("Sysop approval required. No community access.\n");
                        }
                        Err(e) => c.error(e),
                    },
                    Ok(None) => {
                        c.success = true;
                        c.queue("Registration pending; sysop approval required.\n");
                    }
                    Err(e) => c.error(e),
                }
                c.prompt_started = Instant::now();
            }
        }
        let ready = {
            let mut fds = Vec::with_capacity(clients.len() * 2);
            for c in &clients {
                let mut events = PollFlags::empty();
                // Backpressure: one command frame/one response at a time.
                if c.output.is_empty()
                    && Instant::now() >= c.next_input
                    && !matches!(c.state, State::Authenticating | State::Closing)
                {
                    events |= PollFlags::IN;
                }
                if !c.output.is_empty() {
                    events |= PollFlags::OUT;
                }
                fds.push(PollFd::new(&c.tty, events));
                fds.push(PollFd::new(&c.control, PollFlags::IN));
            }
            match poll(
                &mut fds,
                Some(&Timespec {
                    tv_sec: 0,
                    tv_nsec: 10_000_000,
                }),
            ) {
                Ok(_) => {}
                Err(rustix::io::Errno::INTR) => continue,
                Err(e) => return Err(e.into()),
            }
            fds.iter().map(PollFd::revents).collect::<Vec<_>>()
        };
        // Rotating start + a global work budget bounds each sweep. Every client
        // gets at most one input event or background page, never an unbounded
        // pasted sequence of database actions. Slow output never holds a txn.
        let mut work = 16usize;
        let count = clients.len();
        for offset in 0..count {
            let i = (turn + offset) % count;
            let c = &mut clients[i];
            let now = Instant::now();
            if !ready[i * 2 + 1].is_empty()
                || ready[i * 2].intersects(PollFlags::ERR | PollFlags::HUP | PollFlags::NVAL)
            {
                c.close = true;
            }
            c.decoder.expire(now);
            if work > 0 && c.geometry_check.elapsed() >= Duration::from_millis(100) {
                work -= 1;
                c.geometry_check = now;
                if c.resize(&theme, &mut board).is_err() {
                    c.close = true;
                }
            }
            if work > 0 && !c.close && ready[i * 2].contains(PollFlags::IN) {
                work -= 1;
                c.next_input = now + Duration::from_millis(20);
                if c.input(&mut board, &jobs, &theme).is_err() {
                    c.close = true;
                }
            }
            if !c.output.is_empty() && c.output_started.elapsed() > Duration::from_secs(5) {
                c.close = true;
            }
            if c.started.elapsed() > Duration::from_secs(1800)
                || (!matches!(c.state, State::Menu | State::Chat)
                    && c.prompt_started.elapsed() > Duration::from_secs(120))
            {
                c.close = true;
            }
            if work > 0
                && matches!(c.state, State::Chat)
                && now >= c.next_poll
                && !c.close
                && c.output.is_empty()
            {
                work -= 1;
                c.next_poll = now + Duration::from_millis(250);
                let result = (|| -> faction_core::Result<()> {
                    if c.heartbeat.elapsed() >= Duration::from_secs(10) {
                        board.chat_heartbeat(c.lease.as_ref().ok_or(Error::Denied)?)?;
                        c.heartbeat = now;
                    }
                    if c.editor.expire(now).is_some() {
                        c.queue("Draft discarded; press Enter to recover.\n");
                        c.chat_prompt();
                    }
                    if c.ansi {
                        c.chat_page(&mut board, None)
                    } else {
                        c.approved(&mut board)
                    }
                })();
                if let Err(e) = result {
                    c.recover(e, &mut board, &theme);
                }
            }
            if ready[i * 2].contains(PollFlags::OUT) && c.flush(&mut board).is_err() {
                c.close = true;
            }
        }
        turn = if count == 0 { 0 } else { (turn + 16) % count };
        let mut i = 0;
        while i < clients.len() {
            if clients[i].close {
                let mut c = clients.swap_remove(i);
                c.input.zeroize();
                if let Some(lease) = c.lease.take() {
                    let _ = board.chat_close(&lease);
                }
            } else {
                i += 1;
            }
        }
    }
    stop.store(true, Ordering::SeqCst);
    let _ = handoff.join();
    drop(jobs);
    drop(auth_rx);
    for worker in workers {
        let _ = worker.join();
    }
    for c in &mut clients {
        if let Some(l) = c.lease.take() {
            let _ = board.chat_close(&l);
        }
    }
    drop(clients);
    pid.set_len(0)?;
    Ok(())
}
fn main() {
    if run().is_err() {
        eprintln!("Shared-board experiment: local setup or transport failed.");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod dispatch_tests {
    use super::*;
    fn request(connection: u64) -> Request {
        Request {
            connection,
            register: false,
            name: "member".into(),
            secret: Zeroizing::new("private fixture".into()),
        }
    }
    #[test]
    fn two_jobs_before_receivers_wait_and_permits_cover_work_and_results() {
        let (jobs, receiver) = CredentialJobs::new();
        // The old rendezvous queue rejected even the first submission here.
        assert!(jobs.try_send(request(1)).is_ok());
        assert!(jobs.try_send(request(2)).is_ok());
        assert!(matches!(
            jobs.try_send(request(3)),
            Err(TrySendError::Full(_))
        ));
        let a = receiver.recv().expect("first job");
        let b = receiver.recv().expect("second job");
        // Empty queue and a departed connection still cannot admit a third job.
        assert!(matches!(
            jobs.try_send(request(3)),
            Err(TrySendError::Full(_))
        ));
        let result = Auth {
            connection: a.request.connection,
            result: Err(Error::Credentials),
            _permit: a.permit,
        };
        drop(a.request);
        assert!(matches!(
            jobs.try_send(request(3)),
            Err(TrySendError::Full(_))
        ));
        drop(result); // Delivered or discarded result releases exactly one slot.
        assert!(jobs.try_send(request(3)).is_ok());
        assert!(matches!(
            jobs.try_send(request(4)),
            Err(TrySendError::Full(_))
        ));
        drop(b); // Worker/queue shutdown must also release its owned permit.
        assert!(jobs.try_send(request(4)).is_ok());
        drop(receiver);
        assert!(matches!(
            jobs.try_send(request(5)),
            Err(TrySendError::Disconnected(_))
        ));
        assert!(matches!(
            jobs.try_send(request(6)),
            Err(TrySendError::Disconnected(_))
        ));
    }
}
