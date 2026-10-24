//! Shared terminal service. Local member authority stays in faction-core.
#![forbid(unsafe_code)]
use faction_access::{Result, check_directory, denied, receive_terminal};
use faction_core::{Board, ChatCommand, ChatKind, ChatLease, Error, Session, Status};
use faction_terminal::{Action, Decoder, Key, LineEditor, LineEvent, Menu, Theme};
mod catchup_ui;
mod classic;
mod classic_ui;
mod config;
mod directory_ui;
mod door_ui;
mod doors;
mod file_ui;
mod find_ui;
mod gallery_ui;
mod gatherings_ui;
mod library_ui;
mod native_files;
mod operator;
mod pages_ui;
mod policy;
mod polls_ui;
mod rooms_ui;
mod social_ui;
mod ui;
mod wall_ui;
mod writing_ui;
use config::Config;
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    fs::{OFlags, fcntl_getfl, fcntl_setfl},
    termios::{self, OptionalActions, Termios},
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    os::fd::AsFd,
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

const OUTPUT: usize = 32768;
const INPUT: usize = 1152;
enum State {
    Choice,
    Handle(bool),
    Password(bool, String),
    Confirm(String, Zeroizing<String>),
    Authenticating,
    Identity,
    IdentityArt(usize),
    Menu,
    Lounge(Box<faction_pages::social::Desk>),
    Who(Box<faction_pages::who::Desk>),
    Terminal,
    Directory(faction_terminal::directory::View),
    Gallery(Box<faction_gallery::Desk>),
    Library(Box<faction_library::Desk>),
    RoomBrowse(faction_terminal::rooms::View),
    GraphicsProbe(faction_terminal::graphics::Probe),
    Graphics(usize, Instant),
    Probe(faction_terminal::capabilities::Probe),
    Pause,
    RoomsMore(usize),
    RoomId(bool),
    ReadMore(i64, i64, usize),
    PostRoom,
    DraftChoice(i64, faction_core::SavedRoomDraft),
    PostReply(i64),
    PostSubject(i64, Option<i64>),
    PostBody(i64, Option<i64>, String),
    Writing {
        room: i64,
        reply: Option<i64>,
        subject: String,
        draft: faction_terminal::composer::Draft,
        preview: Option<usize>,
        key: Option<String>,
        screen: Option<faction_terminal::composer::Screen>,
        confirm: bool,
    },
    PostRetry {
        room: i64,
        reply: Option<i64>,
        subject: String,
        body: String,
        key: String,
    },
    Wall(Box<faction_wall::Desk>),
    Pages(Box<faction_pages::Desk>),
    Polls(Box<faction_polls::Desk>),
    Catchup(Box<faction_catchup::Desk>),
    Find(Box<faction_find::Desk>),
    Gatherings(Box<faction_gatherings::Desk>),
    Sysop,
    MembersMore(usize),
    MemberStatus(bool),
    RoomSlug,
    RoomTitle(String),
    Chat,
    Closing,
    DoorPending,
    DoorMenu,
    DoorInput,
    Classic,
    Files,
    NativeFiles,
    FileUploadKey {
        area: i64,
        name: String,
        description: String,
        bytes: u32,
    },
    FileDownloadKey(String),
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
    theme_index: usize,
    sysop_display: bool,
    mark_after_flush: Option<(i64, i64)>,
    close: bool,
    success: bool,
    ansi_allowed: bool,
    native_text_hint: bool,
    ansi: bool,
    dimensions: (u16, u16),
    geometry_check: Instant,
    graphics_active: bool,
    last_graphics: Option<Instant>,
    last_graphics_frame: Option<Instant>,
    capabilities: faction_terminal::capabilities::Capabilities,
    editor: LineEditor,
    skip_lf: bool,
    next_input: Instant,
    last_input: Instant,
    published_activity: Option<faction_core::VisitActivity>,
    activity_retry: Instant,
    notifications: faction_pages::notifications::Notifications,
    doors_uid: Option<u32>,
    door: Option<doors::Door>,
    classic_games: Option<Arc<Vec<classic::Game>>>,
    classic: Option<classic::Connection>,
    files_enabled: bool,
    native: Option<native_files::Native>,
    directory: Option<Arc<faction_terminal::directory::Directory>>,
    gallery: Option<Arc<faction_gallery::Catalog>>,
    library: Option<Arc<faction_library::Catalog>>,
}
impl Drop for Client {
    fn drop(&mut self) {
        // Best effort while still nonblocking: never hang cleanup behind a peer.
        if self.graphics_active {
            let _ = self
                .tty
                .write(faction_terminal::graphics::CLEANUP.as_bytes());
        }
        if self.ansi {
            if self.capabilities.synchronized {
                let _ = self.tty.write(b"\x1b[?2026l");
            }
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
            theme_index: 0,
            sysop_display: false,
            mark_after_flush: None,
            close: false,
            success: false,
            ansi_allowed: ansi && !term.is_empty() && !matches!(term, "dumb" | "faction-compact"),
            native_text_hint: term == "faction-compact",
            ansi: false,
            dimensions: (0, 0),
            geometry_check: now,
            graphics_active: false,
            last_graphics: None,
            last_graphics_frame: None,
            capabilities: Default::default(),
            editor: LineEditor::default(),
            skip_lf: false,
            next_input: now,
            last_input: now,
            published_activity: None,
            activity_retry: now,
            notifications: Default::default(),
            doors_uid: None,
            door: None,
            classic_games: None,
            classic: None,
            files_enabled: false,
            native: None,
            directory: None,
            gallery: None,
            library: None,
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
            if let State::GraphicsProbe(probe) = &mut self.state {
                probe.begin(Instant::now());
            }
            if let State::Probe(probe) = &mut self.state {
                probe.begin(Instant::now());
            }
            if !self.close
                && let Some((room, through)) = self.mark_after_flush.take()
            {
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
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 || (args[0] != "--config" && args[0] != "--check-config") {
        return Err(denied());
    }
    let config = Config::load(Path::new(&args[1]))?;
    let base = if let Some(path) = &config.theme {
        Theme::load(path).map_err(|_| denied())?
    } else {
        Theme::default()
    };
    let mut themes = faction_terminal::seasons::Themes::load(
        base,
        config.season.as_deref(),
        config.theme_schedule.as_deref(),
    )
    .map_err(|_| denied())?;
    themes
        .set_services(
            config.files_uid.is_some(),
            config.doors_uid.is_some() || !config.classic_games.is_empty(),
        )
        .map_err(|_| denied())?;
    let library = config
        .library
        .as_ref()
        .map(|p| faction_library::Catalog::load(p).map(Arc::new))
        .transpose()
        .map_err(|_| denied())?;
    themes
        .set_library_enabled(library.as_ref().is_some_and(|c| c.enabled()))
        .map_err(|_| denied())?;
    let gallery = config
        .gallery
        .as_ref()
        .map(|p| faction_gallery::Catalog::load(p).map(Arc::new))
        .transpose()
        .map_err(|_| denied())?;
    themes
        .set_gallery_enabled(gallery.as_ref().is_some_and(|c| c.enabled()))
        .map_err(|_| denied())?;
    let directory = config
        .directory
        .as_ref()
        .map(|p| faction_terminal::directory::Directory::load(p).map(Arc::new))
        .transpose()
        .map_err(|_| denied())?;
    themes
        .set_directory_enabled(directory.is_some())
        .map_err(|_| denied())?;
    let classic_games = Arc::new(config.classic_games.clone());
    if args[0] == "--check-config" {
        return Ok(());
    }
    let terminal_uid = config.terminal_uid;
    let owner = rustix::process::geteuid().as_raw();
    if owner == 0
        || terminal_uid == owner
        || config
            .doors_uid
            .is_some_and(|u| u == owner || u == terminal_uid)
    {
        return Err(denied());
    }
    faction_unix::prepare_process()?;
    let state = config.state_dir.clone();
    if !state.join("faction.sqlite3").is_file() {
        return Err(denied());
    }
    let mut board = Board::open(&state)?;
    let ansi = config.ansi;
    let max_sessions = config.max_sessions;
    let launch_ms = u128::from(config.launch_ms);
    let session_seconds = config.session_seconds;
    let (listener, pid, _guard) = bind(&config.socket)?;
    let mut file_policy = match (config.files_uid, config.files_auth_uid) {
        (Some(worker), Some(auth)) => Some(policy::Policy::bind(worker, auth)?),
        _ => None,
    };
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
                let elapsed = refill.elapsed().as_millis() / launch_ms;
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
                if faction_unix::peer_uid(stream.as_fd()).ok() != Some(terminal_uid) {
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
    faction_unix::process_limits(max_sessions as u32)?;
    let mut clients: Vec<Client> = Vec::with_capacity(max_sessions);
    let mut generation = 0u64;
    eprintln!(
        "Faction shared service ready; session limit {max_sessions}, admission refill {launch_ms} ms."
    );
    let mut operator = operator::Publisher::new(config.operator_status, state);
    let mut turn = 0usize;
    while !faction_unix::shutdown_requested() {
        operator.tick(&board, &clients, &jobs, max_sessions);
        if let Some(p) = &mut file_policy {
            p.pump(&mut board);
        }
        while let Ok((control, tty, term)) = admit_rx.try_recv() {
            if clients.len() >= max_sessions {
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
            if let Ok(mut client) = Client::new(generation, control, tty, &term, ansi) {
                client.files_enabled = config.files_uid.is_some();
                client.directory = directory.as_ref().map(Arc::clone);
                client.gallery = gallery.as_ref().map(Arc::clone);
                client.library = library.as_ref().map(Arc::clone);
                client.doors_uid = config.doors_uid;
                client.classic_games = Some(Arc::clone(&classic_games));
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
                            c.theme_index = themes.select_now();
                            if let Err(e) =
                                c.identity(themes.get(c.theme_index), &m.handle, &mut board)
                            {
                                c.error(e)
                            }
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
            let theme = themes.get(c.theme_index);
            let now = Instant::now();
            if !ready[i * 2 + 1].is_empty()
                || ready[i * 2].intersects(PollFlags::ERR | PollFlags::HUP | PollFlags::NVAL)
            {
                c.close = true;
            }
            c.decoder.expire(now);
            if work > 0 && matches!(c.state, State::DoorPending) && c.output.is_empty() && !c.close
            {
                work -= 1;
                if let Err(e) = c.door_poll(theme, &mut board) {
                    c.recover(e, &mut board, theme);
                    c.door = None;
                }
            }
            if work > 0 && matches!(c.state, State::Classic) && c.output.is_empty() && !c.close {
                work -= 1;
                if let Err(e) = c.classic_poll(theme, &mut board) {
                    c.classic = None;
                    c.recover(e, &mut board, theme);
                }
            }
            if work > 0 && !c.close && matches!(&c.state,State::Probe(p) if p.expired(now)) {
                work -= 1;
                if c.probe_finish(&mut board).is_err() {
                    c.close = true;
                }
            }
            if work > 0
                && !c.close
                && c.output.is_empty()
                && (matches!(&c.state, State::GraphicsProbe(p) if p.expired(now))
                    || matches!(&c.state, State::Graphics(..)))
            {
                work -= 1;
                if c.graphics_poll(&mut board).is_err() {
                    c.close = true;
                }
            }
            if work > 0 && c.geometry_check.elapsed() >= Duration::from_millis(100) {
                work -= 1;
                c.geometry_check = now;
                if c.resize(theme, &mut board).is_err() {
                    c.close = true;
                }
            }
            if work > 0 && !c.close && ready[i * 2].contains(PollFlags::IN) {
                work -= 1;
                c.next_input = now + Duration::from_millis(20);
                if c.input(&mut board, &jobs, theme).is_err() {
                    c.close = true;
                }
            }
            if !c.output.is_empty() && c.output_started.elapsed() > Duration::from_secs(5) {
                c.close = true;
            }
            if matches!(c.state, State::Classic)
                && c.prompt_started.elapsed() > Duration::from_secs(120)
                && c.classic_return("Classic game input timed out.", theme, &mut board)
                    .is_err()
            {
                c.close = true;
            }
            if c.started.elapsed() > Duration::from_secs(session_seconds)
                || (!matches!(c.state, State::Menu | State::Chat)
                    && c.prompt_started.elapsed() > Duration::from_secs(120))
            {
                c.close = true;
            }
            if work > 0
                && !c.close
                && c.lease.is_some()
                && now >= c.activity_retry
                && c.activity() != c.published_activity
            {
                work -= 1;
                if let Some(activity) = c.activity() {
                    match board.visit_activity(c.lease.as_ref().ok_or(Error::Denied)?, activity) {
                        Ok(()) => c.published_activity = Some(activity),
                        Err(Error::Busy) => c.activity_retry = now + Duration::from_secs(1),
                        Err(e) => c.recover(e, &mut board, theme),
                    }
                }
            }
            // One connection lease, including while a member reads, composes or plays.
            // This produces no TTY output and never changes the active UI state.
            if work > 0
                && !c.close
                && c.lease.is_some()
                && c.heartbeat.elapsed() >= Duration::from_secs(10)
            {
                work -= 1;
                match board.chat_heartbeat(c.lease.as_ref().ok_or(Error::Denied)?) {
                    Ok(()) => c.heartbeat = now,
                    Err(Error::Busy) => (),
                    Err(e) => c.recover(e, &mut board, theme),
                }
            }
            if work > 0
                && matches!(c.state, State::Chat)
                && now >= c.next_poll
                && !c.close
                && c.output.is_empty()
            {
                work -= 1;
                c.next_poll = now + Duration::from_millis(250);
                let result = {
                    if c.editor.expire(now).is_some() {
                        c.queue("Draft discarded; press Enter to recover.\n");
                        c.chat_prompt();
                    }
                    if c.ansi {
                        c.chat_page(&mut board, None)
                    } else {
                        c.approved(&mut board)
                    }
                };
                if let Err(e) = result {
                    c.recover(e, &mut board, theme);
                }
            }
            if work > 0
                && !c.close
                && matches!(c.state, State::Menu)
                && c.output.is_empty()
                && c.input.is_empty()
                && c.decoder.idle()
                && !ready[i * 2].contains(PollFlags::IN)
                && c.last_input.elapsed() >= faction_pages::notifications::INTERVAL
                && c.notifications.due(now)
            {
                work -= 1;
                if let Err(e) = c.idle_notifications(&mut board, theme, now) {
                    c.recover(e, &mut board, theme);
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
    operator.stopped(&board, max_sessions);
    pid.set_len(0)?;
    Ok(())
}
fn main() {
    if run().is_err() {
        eprintln!(
            "Faction shared service: unsafe/invalid configuration, local setup or transport failed."
        );
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
