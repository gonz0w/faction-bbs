//! Opt-in shared terminal experiment. Never installed by the normal service.
#![forbid(unsafe_code)]
use faction_access::{Result, check_directory, denied, receive_terminal};
use faction_core::{Board, ChatKind, ChatLease, Error, Session, Status};
use faction_terminal::{Action, Decoder, Key, Menu, Theme};
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
const OUTPUT: usize = 8192;
const INPUT: usize = 1152;
enum State {
    Choice,
    Handle(bool),
    Password(bool, String),
    Authenticating,
    Identity,
    Menu,
    Chat,
    Post,
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
    mark_after_flush: Option<(i64, i64)>,
    close: bool,
    success: bool,
}
impl Drop for Client {
    fn drop(&mut self) {
        let _ = termios::tcsetattr(&self.tty, OptionalActions::Now, &self.saved);
        let _ = fcntl_setfl(&self.tty, self.flags);
        let _ = self.control.write(&[u8::from(!self.success)]);
    }
}
impl Client {
    fn new(connection: u64, mut control: UnixStream, tty: File) -> Result<Self> {
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
            output: Vec::with_capacity(OUTPUT),
            written: 0,
            output_started: now,
            started: now,
            prompt_started: now,
            heartbeat: now,
            next_poll: now,
            focus: 0,
            mark_after_flush: None,
            close: false,
            success: false,
        };
        c.queue("[L]ogin [R]egister [Q]uit: ");
        Ok(c)
    }
    fn queue(&mut self, text: &str) {
        // Per-connection output has a hard cap and five-second stall deadline.
        if self.output.len().saturating_add(text.len()) > OUTPUT {
            self.close = true;
            return;
        }
        if self.output.is_empty() {
            self.output_started = Instant::now();
        }
        for byte in text.bytes() {
            if byte == b'\n' {
                self.output.push(b'\r');
            }
            self.output.push(byte);
            if self.output.len() > OUTPUT {
                self.close = true;
                return;
            }
        }
    }
    fn safe(text: &str) -> String {
        text.chars()
            .flat_map(|c| {
                if c.is_ascii_graphic() || c == ' ' {
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
    fn menu(&mut self, theme: &Theme, board: &mut Board) -> faction_core::Result<()> {
        let member = board.whoami(self.session.as_ref().ok_or(Error::Denied)?)?;
        if member.status != Status::Approved {
            return Err(Error::Denied);
        }
        let mut menu = Menu::new(theme, member.role == faction_core::Role::Sysop);
        for _ in 0..self.focus {
            menu.feed(Key::Down);
        }
        self.queue(&menu.render());
        self.queue("Ctrl-C exits; Esc: Enter to recover.\n");
        self.state = State::Menu;
        Ok(())
    }
    fn approved(&mut self, board: &mut Board) -> faction_core::Result<()> {
        let m = board.whoami(self.session.as_ref().ok_or(Error::Denied)?)?;
        if m.status != Status::Approved {
            return Err(Error::Denied);
        }
        Ok(())
    }
    fn line(
        &mut self,
        text: &str,
        board: &mut Board,
        jobs: &CredentialJobs,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        let old = std::mem::replace(&mut self.state, State::Closing);
        match old {
            State::Choice => match text.to_ascii_lowercase().as_str() {
                "l" | "r" => {
                    self.state = State::Handle(text.eq_ignore_ascii_case("r"));
                    self.queue("Handle: ");
                }
                _ => {
                    self.success = true;
                    self.queue("Logged out.\n");
                }
            },
            State::Handle(register) => {
                if !(3..=24).contains(&text.len())
                    || !text
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
                {
                    return Err(Error::Credentials);
                }
                self.state = State::Password(register, text.into());
                self.queue("Password (hidden): ");
            }
            State::Password(register, name) => {
                let job = Request {
                    connection: self.connection,
                    register,
                    name,
                    secret: Zeroizing::new(text.to_owned()),
                };
                match jobs.try_send(job) {
                    Ok(()) => self.state = State::Authenticating,
                    Err(TrySendError::Full(_)) => {
                        self.queue("credential work busy; reconnect later\n")
                    }
                    Err(TrySendError::Disconnected(_)) => return Err(Error::Denied),
                }
            }
            State::Identity => self.menu(theme, board)?,
            State::Post => {
                self.approved(board)?;
                let key = faction_core::request_id()?;
                let p = board.post(
                    self.session.as_ref().ok_or(Error::Denied)?,
                    1,
                    &key,
                    text,
                    None,
                )?;
                self.queue(&format!("Posted #{}.\n", p.id));
                self.menu(theme, board)?;
            }
            State::Chat => {
                self.state = State::Chat;
                self.approved(board)?;
                if text == "/quit" {
                    board.chat_close(self.lease.as_ref().ok_or(Error::Denied)?)?;
                    self.lease = None;
                    self.queue("Chat closed.\n");
                    self.menu(theme, board)?;
                } else if let Some(name) = text.strip_prefix("/join ") {
                    let ch = board.chat_join(self.lease.as_ref().ok_or(Error::Denied)?, name)?;
                    self.channel = ch.name;
                    self.cursor = ch.sequence;
                    self.queue("Joined.\n");
                } else if text == "/poll" || text == "/history" {
                    self.chat_page(board)?;
                } else if text == "/names" {
                    let page = board.chat_names(
                        self.lease.as_ref().ok_or(Error::Denied)?,
                        &self.channel,
                        None,
                    )?;
                    for name in page.handles {
                        self.queue(&format!("{}\n", Self::safe(&name)));
                    }
                } else if !text.starts_with('/') || text.starts_with("//") {
                    let body = text
                        .strip_prefix('/')
                        .filter(|_| text.starts_with("//"))
                        .unwrap_or(text);
                    board.chat_send(
                        self.lease.as_ref().ok_or(Error::Denied)?,
                        &self.channel,
                        self.request,
                        body,
                        ChatKind::Say,
                    )?;
                    self.request = self.request.checked_add(1).ok_or(Error::Denied)?;
                } else {
                    self.queue("Probe commands: /join /poll /history /names /quit\n");
                }
                if matches!(self.state, State::Chat) {
                    self.queue(&format!("chat[{}]> ", self.channel));
                }
            }
            _ => return Err(Error::Denied),
        }
        self.prompt_started = Instant::now();
        Ok(())
    }
    fn action(&mut self, key: Key, board: &mut Board, theme: &Theme) -> faction_core::Result<()> {
        self.approved(board)?;
        let member = board.whoami(self.session.as_ref().ok_or(Error::Denied)?)?;
        let mut menu = Menu::new(theme, member.role == faction_core::Role::Sysop);
        for _ in 0..self.focus {
            menu.feed(Key::Down);
        }
        let before = menu.focus();
        let action = menu.feed(key);
        self.focus = menu.focus();
        if let Some(action) = action {
            match action {
                Action::Quit => {
                    self.queue("\x1b[0m\x1b[?25h\nLogged out.\n");
                    self.success = true;
                    self.state = State::Closing;
                }
                Action::Rooms => {
                    for room in board.rooms(self.session.as_ref().ok_or(Error::Denied)?)? {
                        self.queue(&format!(
                            "{} {} - {} ({} unread)\n",
                            room.id,
                            Self::safe(&room.slug),
                            Self::safe(&room.title),
                            room.unread
                        ));
                    }
                    self.menu(theme, board)?;
                }
                Action::History | Action::New => {
                    // Focused experiment: fixed Lobby, one bounded page. Full navigation remains in the normal worker.
                    let s = self.session.as_ref().ok_or(Error::Denied)?;
                    let after = if action == Action::New {
                        board.unread_cursor(s, 1)?
                    } else {
                        0
                    };
                    let rows = board.posts(s, 1, after)?;
                    let through = rows.last().map(|p| p.id);
                    for p in rows {
                        self.queue(&format!(
                            "#{} {}: {}\n",
                            p.id,
                            Self::safe(&p.author),
                            Self::safe(&p.body)
                        ));
                    }
                    if let Some(n) = through {
                        self.mark_after_flush = Some((1, n));
                    }
                    self.menu(theme, board)?;
                }
                Action::Post => {
                    self.state = State::Post;
                    self.queue("Lobby message (up to 1024 bytes): ");
                }
                Action::Chat => {
                    let lease = board.chat_open(self.session.as_ref().ok_or(Error::Denied)?)?;
                    let ch = board.chat_join(&lease, "#lobby")?;
                    self.lease = Some(lease);
                    self.request = 1; // New lease starts a new core request sequence.
                    self.channel = ch.name;
                    self.cursor = ch.sequence;
                    self.heartbeat = Instant::now();
                    self.next_poll = Instant::now();
                    self.state = State::Chat;
                    self.queue("\x1b[0m\x1b[?25h\nchat[#lobby]> ");
                }
                _ => {
                    self.queue("This action is outside the shared-board experiment.\n");
                    self.menu(theme, board)?;
                }
            }
        } else if self.focus != before {
            self.queue(&menu.redraw(before));
        }
        Ok(())
    }
    fn chat_page(&mut self, board: &mut Board) -> faction_core::Result<()> {
        let page = board.chat_read(
            self.lease.as_ref().ok_or(Error::Denied)?,
            &self.channel,
            self.cursor,
        )?;
        if page.gap {
            self.queue("History gap.\n");
        }
        for m in page.messages {
            self.queue(&format!(
                "\n{}: {}\n",
                Self::safe(&m.author),
                Self::safe(&m.body)
            ));
        }
        self.cursor = page.cursor;
        Ok(())
    }
    fn input(&mut self, board: &mut Board, jobs: &CredentialJobs, theme: &Theme) -> Result<()> {
        // One read and bounded processing per ready connection per loop.
        let mut bytes = Zeroizing::new([0u8; 256]);
        let n = match self.tty.read(&mut *bytes) {
            Ok(0) => {
                self.close = true;
                return Ok(());
            }
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        for &b in &bytes[..n] {
            if matches!(self.state, State::Closing | State::Authenticating) {
                break;
            }
            if matches!(self.state, State::Menu) {
                if let Some(k) = self.decoder.feed(b, Instant::now()) {
                    if k == Key::Cancel {
                        self.close = true;
                        break;
                    }
                    if let Err(e) = self.action(k, board, theme) {
                        self.error(e);
                        self.state = State::Closing;
                        break;
                    }
                }
                continue;
            }
            let hidden = matches!(self.state, State::Password(..));
            if b == 3 || b == 4 {
                self.close = true;
                break;
            }
            if b == b'\n' || b == b'\r' {
                let text = Zeroizing::new(String::from_utf8(std::mem::take(&mut *self.input))?);
                self.queue("\n");
                if let Err(e) = self.line(&text, board, jobs, theme) {
                    self.error(e);
                    self.state = State::Closing;
                }
            } else if b == 8 || b == 127 {
                self.input.pop();
                if !hidden {
                    self.queue("\x08 \x08");
                }
            } else if b >= 32 && b != 127 && (hidden || b.is_ascii()) {
                let cap = if hidden { 128 } else { INPUT };
                if self.input.len() >= cap {
                    self.close = true;
                    break;
                }
                self.input.push(b);
                if !hidden {
                    self.queue(&(b as char).to_string());
                }
            } else {
                self.close = true;
                break;
            }
        }
        Ok(())
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
    if args.len() != 6
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
    let theme = Theme::default();
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
                if let Ok((tty, _term)) = receive_terminal(&stream, terminal_uid) {
                    let _ = admit_tx.try_send((stream, tty));
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
    while !faction_unix::shutdown_requested() {
        while let Ok((control, tty)) = admit_rx.try_recv() {
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
            if let Ok(client) = Client::new(generation, control, tty) {
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
                            c.queue(&format!(
                                "{}\n{}\nWelcome, {}.\nPress Enter to continue: ",
                                theme.board(),
                                theme.welcome(),
                                Client::safe(&m.handle)
                            ));
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
                if c.output.is_empty() && !matches!(c.state, State::Authenticating | State::Closing)
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
        for (i, c) in clients.iter_mut().enumerate() {
            let now = Instant::now();
            if !ready[i * 2 + 1].is_empty()
                || ready[i * 2].intersects(PollFlags::ERR | PollFlags::HUP | PollFlags::NVAL)
            {
                c.close = true;
            }
            if ready[i * 2].contains(PollFlags::IN) && c.input(&mut board, &jobs, &theme).is_err() {
                c.close = true;
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
            if matches!(c.state, State::Chat)
                && now >= c.next_poll
                && !c.close
                && c.output.is_empty()
            {
                c.next_poll = now + Duration::from_millis(250);
                let result = (|| -> faction_core::Result<()> {
                    if c.heartbeat.elapsed() >= Duration::from_secs(10) {
                        board.chat_heartbeat(c.lease.as_ref().ok_or(Error::Denied)?)?;
                        c.heartbeat = now;
                    }
                    c.chat_page(&mut board)
                })();
                if let Err(e) = result {
                    c.error(e);
                    c.state = State::Closing;
                }
            }
            if ready[i * 2].contains(PollFlags::OUT) && c.flush(&mut board).is_err() {
                c.close = true;
            }
        }
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
