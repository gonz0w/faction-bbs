//! Nonblocking presentation for the isolated shared owner. Domain authorization
//! remains in faction-core; no owner bootstrap/export/recovery command exists.
use super::*;

impl Client {
    pub(super) fn session(&self) -> faction_core::Result<&Session> {
        self.session.as_ref().ok_or(Error::Denied)
    }
    fn lease(&self) -> faction_core::Result<&ChatLease> {
        self.lease.as_ref().ok_or(Error::Denied)
    }
    pub(super) fn size(&self) -> (u16, u16) {
        termios::tcgetwinsize(&self.tty)
            .map(|w| (w.ws_col, w.ws_row))
            .unwrap_or((0, 0))
    }
    fn capable(&self) -> bool {
        self.ansi_allowed && matches!(self.size(), (40..=240, 18..=100))
    }
    pub(super) fn approved(&self, board: &mut Board) -> faction_core::Result<()> {
        if board.whoami(self.session()?)?.status != Status::Approved {
            return Err(Error::Denied);
        }
        Ok(())
    }
    pub(super) fn identity(&mut self, theme: &Theme, handle: &str) {
        self.ansi = self.capable();
        self.dimensions = self.size();
        if self.ansi && matches!(self.dimensions, (80..=240, 24..=100)) {
            self.queue("\x1b[0;37;40m\x1b[2J\x1b[H");
            if let Some(art) = theme.identity_ansi() {
                self.queue(&art.render());
            } else {
                self.queue(theme.identity_ascii().unwrap_or(theme.board()));
            }
            self.queue("\x1b[18;1H");
        } else {
            self.queue(theme.identity_ascii().unwrap_or(theme.board()));
        }
        self.queue(&format!(
            "\n{}\nWelcome, {}.\nPress Enter to continue: ",
            theme.welcome(),
            Self::safe(handle)
        ));
    }
    pub(super) fn menu(&mut self, theme: &Theme, board: &mut Board) -> faction_core::Result<()> {
        self.approved(board)?;
        let sysop = board.whoami(self.session()?)?.role == faction_core::Role::Sysop;
        self.sysop_display = sysop;
        self.ansi = self.capable();
        self.dimensions = self.size();
        self.decoder = Decoder::new();
        self.input.clear();
        self.state = State::Menu;
        if self.ansi {
            let menu = self.focused(theme, sysop);
            self.focus = menu.focus();
            self.queue("\x1b[?25l");
            self.queue(&self.capabilities.frame(&menu.render(), self.ansi));
        } else {
            self.queue(&theme.plain_menu(sysop));
            self.queue("Command (then Enter): ");
        }
        Ok(())
    }
    fn terminal_screen(&mut self, board: &mut Board) -> faction_core::Result<()> {
        self.approved(board)?;
        self.state = State::Terminal;
        self.prompt_started = Instant::now();
        self.input.clear();
        let (cols, rows) = self.size();
        self.queue(
            &self
                .capabilities
                .frame(&self.capabilities.screen(cols, rows, self.ansi), self.ansi),
        );
        Ok(())
    }
    pub(super) fn probe_finish(&mut self, board: &mut Board) -> faction_core::Result<()> {
        if let State::Probe(p) = &self.state {
            self.capabilities.synchronized = p.result().map_err(Error::Invalid)?;
        }
        termios::tcflush(&self.tty, termios::QueueSelector::IFlush)
            .map_err(|_| Error::Invalid("terminal flush"))?;
        self.terminal_screen(board)
    }
    fn focused<'a>(&self, theme: &'a Theme, sysop: bool) -> Menu<'a> {
        let mut menu = Menu::new(theme, sysop);
        for _ in 0..theme.entries().len() {
            if menu.focus() == self.focus {
                break;
            }
            menu.feed(Key::Down);
        }
        menu
    }
    pub(super) fn pause(&mut self) {
        self.state = State::Pause;
        self.queue("Press Enter to return to menu: ");
    }
    pub(super) fn prompt(&mut self, state: State, label: &str) {
        self.state = state;
        self.prompt_started = Instant::now();
        self.queue(label);
    }
    fn action(&mut self, key: Key, board: &mut Board, theme: &Theme) -> faction_core::Result<bool> {
        // Cached presentation never grants an action; open/core recheck authority.
        let sysop = self.sysop_display;
        let mut menu = self.focused(theme, sysop);
        let previous = menu.focus();
        let action = menu.feed(key);
        self.focus = menu.focus();
        if let Some(action) = action {
            self.open(action, board, theme)?;
            return Ok(true);
        }
        if self.focus != previous {
            self.queue(&self.capabilities.frame(&menu.redraw(previous), self.ansi));
        }
        Ok(false)
    }
    fn open(
        &mut self,
        action: Action,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        let title = match action {
            Action::Rooms => "Rooms",
            Action::New | Action::History => "Read messages",
            Action::Post => "Post a message",
            Action::Sysop => "Sysop controls",
            Action::Chat => "Local chat",
            Action::Files => "File areas",
            Action::Doors => "Doors",
            Action::Quit => "Disconnect",
            Action::Terminal => "Terminal settings",
        };
        self.queue(&self.capabilities.section(theme, title, self.ansi));
        match action {
            Action::Terminal => self.terminal_screen(board)?,
            Action::Quit => {
                self.queue(&format!("{}\nLogged out.\n", theme.logout()));
                self.success = true;
                self.state = State::Closing;
            }
            Action::Rooms => self.rooms(board, 0)?,
            Action::New | Action::History => {
                self.prompt(State::RoomId(action == Action::New), "Room ID: ")
            }
            Action::Post => self.prompt(State::PostRoom, "Room ID: "),
            Action::Sysop => {
                if board.whoami(self.session()?)?.role != faction_core::Role::Sysop {
                    return Err(Error::Denied);
                }
                self.prompt(
                    State::Sysop,
                    "Sysop: [L]ist [A]pprove [V]revoke [C]reate room: ",
                );
            }
            Action::Chat => {
                let lease = board.chat_open(self.session()?)?;
                self.lease = Some(lease); // Cleanup also owns the lease if joining fails.
                let channel = board.chat_join(self.lease()?, "#lobby")?;
                self.channel = channel.name;
                self.cursor = channel.sequence;
                self.request = 1;
                self.heartbeat = Instant::now();
                self.next_poll = Instant::now() + Duration::from_millis(250);
                self.editor = LineEditor::default();
                self.state = State::Chat;
                self.queue("Local chat. /help lists commands. All channels stay local.\n");
                self.chat_prompt();
            }
            Action::Files => self.file_lobby(board)?,
            Action::Doors => self.door_open("doors", board, theme)?,
        }
        Ok(())
    }
    fn rooms(&mut self, board: &mut Board, offset: usize) -> faction_core::Result<()> {
        let rooms = board.rooms(self.session()?)?;
        for r in rooms.iter().skip(offset).take(10) {
            self.queue(&format!(
                "{}  {} - {} ({} unread)\n",
                r.id,
                Self::safe(&r.slug),
                Self::safe(&r.title),
                r.unread
            ));
        }
        if offset + 10 < rooms.len() {
            self.prompt(State::RoomsMore(offset + 10), "More rooms? [y/N]: ");
        } else {
            self.queue("Room listing is limited to the first 100 rooms.\n");
            self.pause();
        }
        Ok(())
    }
    fn members(&mut self, board: &mut Board, offset: usize) -> faction_core::Result<()> {
        let members = board.members(self.session()?)?;
        for m in members.iter().skip(offset).take(10) {
            self.queue(&format!(
                "{} {:?} {:?}\n",
                Self::safe(&m.handle),
                m.status,
                m.role
            ));
        }
        if offset + 10 < members.len() {
            self.prompt(State::MembersMore(offset + 10), "More accounts? [y/N]: ");
        } else {
            self.queue("First 100 accounts; exact-handle approval works beyond this list.\n");
            self.pause();
        }
        Ok(())
    }
    fn posts(
        &mut self,
        board: &mut Board,
        room: i64,
        after: i64,
        offset: usize,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        // Re-fetch under current approval and per-viewer file permissions on
        // every screen. Retain only cursors, never a per-client history cache.
        let rows = board.posts(self.session()?, room, after)?;
        let Some(post) = rows.first() else {
            self.queue("No more messages.\n");
            self.pause();
            return Ok(());
        };
        let metadata = board.post_metadata(self.session()?, post.id)?;
        let (columns, height) = self.size();
        let message = faction_terminal::Message {
            room,
            id: post.id,
            author: &post.author,
            reply_to: post.reply_to,
            body: &post.body,
        };
        let page = message
            .page_with_capabilities(
                columns,
                height,
                offset,
                faction_terminal::MessageDetails {
                    subject: &metadata.subject,
                    posted: metadata
                        .created_utc
                        .as_deref()
                        .unwrap_or("Unknown (legacy)"),
                },
                self.capabilities.unicode,
            )
            .map_err(Error::Invalid)?;
        self.queue(&self.capabilities.frame(
            &format!(
                "{}{}",
                self.capabilities.section(theme, "Read messages", self.ansi),
                page.text
            ),
            self.ansi,
        ));
        let (through, next) = match page.next {
            Some(next) => (after, next),
            None => {
                // Only a complete message may advance durable unread state,
                // and only after all its output has drained successfully.
                self.mark_after_flush = Some((room, post.id));
                (post.id, 0)
            }
        };
        self.prompt(State::ReadMore(room, through, next), "More? [y/N]: ");
        Ok(())
    }
    fn submit(
        &mut self,
        board: &mut Board,
        room: i64,
        reply: Option<i64>,
        subject: String,
        body: String,
        key: String,
    ) -> faction_core::Result<()> {
        match board.post_article(
            self.session()?,
            room,
            &key,
            faction_core::PostDraft {
                subject: &subject,
                body: &body,
                reply_to: reply,
            },
        ) {
            Ok(post) => {
                self.queue(&format!("Posted #{}.\n", post.id));
                self.pause();
            }
            Err(Error::Busy) => self.prompt(
                State::PostRetry {
                    room,
                    reply,
                    subject,
                    body,
                    key,
                },
                "Database busy. Retry this same submission? [y/N]: ",
            ),
            Err(e) => return Err(e),
        }
        Ok(())
    }
    fn positive(text: &str) -> faction_core::Result<i64> {
        text.parse::<i64>()
            .ok()
            .filter(|n| *n > 0)
            .ok_or(Error::Invalid("positive ID required"))
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
            State::NativeFiles => self.native_line(text, board, theme)?,
            State::Files | State::FileUploadKey { .. } | State::FileDownloadKey(_) => {
                self.file_line(old, text, board, theme)?
            }
            State::DoorInput => self.door_submit(text, board, theme)?,
            State::DoorMenu => {
                if text.len() == 1 {
                    self.door_key(
                        Key::Hotkey(text.as_bytes()[0].to_ascii_uppercase()),
                        theme,
                        board,
                    )?;
                } else {
                    // Keep a plain menu usable after an empty or invalid line.
                    // The same path still rechecks current membership.
                    self.door_key(Key::Hotkey(0), theme, board)?;
                }
            }
            State::DoorPending | State::Classic => (),
            State::Choice => match text.to_ascii_lowercase().as_str() {
                "l" | "r" => self.prompt(State::Handle(text.eq_ignore_ascii_case("r")), "Handle: "),
                "q" => {
                    self.success = true;
                    self.queue("Logged out.\n");
                }
                _ => self.prompt(State::Choice, "[L]ogin [R]egister [Q]uit: "),
            },
            State::Handle(register) => {
                if !(3..=24).contains(&text.len())
                    || !text
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
                {
                    return Err(Error::Credentials);
                }
                self.prompt(
                    State::Password(register, text.into()),
                    "Password (hidden): ",
                );
            }
            State::Password(true, name) => self.prompt(
                State::Confirm(name, Zeroizing::new(text.into())),
                "Confirm password (hidden): ",
            ),
            State::Password(false, name) => {
                self.credential(false, name, Zeroizing::new(text.into()), jobs)?
            }
            State::Confirm(name, secret) => {
                if *secret != text {
                    return Err(Error::Invalid("passwords do not match"));
                }
                self.credential(true, name, secret, jobs)?;
            }
            State::Identity | State::Pause => self.menu(theme, board)?,
            State::Terminal => {
                self.approved(board)?;
                if text.eq_ignore_ascii_case("d") {
                    self.menu(theme, board)?;
                } else if text.eq_ignore_ascii_case("p") && self.ansi {
                    self.state = State::Probe(Default::default());
                    self.queue(faction_terminal::capabilities::SYNC_QUERY);
                } else {
                    self.capabilities.choose(text);
                    self.terminal_screen(board)?;
                }
            }
            State::Probe(_) => return Err(Error::Invalid("probe input state")),
            State::Menu => {
                let sysop = board.whoami(self.session()?)?.role == faction_core::Role::Sysop;
                if text.len() == 1
                    && let Some(action) = theme.hotkey(text.as_bytes()[0], sysop)
                {
                    self.open(action, board, theme)?;
                } else {
                    self.menu(theme, board)?;
                }
            }
            State::RoomsMore(offset) => {
                if text.eq_ignore_ascii_case("y") {
                    self.rooms(board, offset)?;
                } else {
                    self.pause();
                }
            }
            State::MembersMore(offset) => {
                if text.eq_ignore_ascii_case("y") {
                    self.members(board, offset)?;
                } else {
                    self.pause();
                }
            }
            State::RoomId(new) => {
                let room = Self::positive(text)?;
                let after = if new {
                    board.unread_cursor(self.session()?, room)?
                } else {
                    0
                };
                self.posts(board, room, after, 0, theme)?;
            }
            State::ReadMore(room, after, offset) => {
                if text.eq_ignore_ascii_case("y") {
                    self.posts(board, room, after, offset, theme)?;
                } else {
                    self.pause();
                }
            }
            State::PostRoom => self.prompt(
                State::PostReply(Self::positive(text)?),
                "Reply to post ID (blank for new message): ",
            ),
            State::PostReply(room) => {
                let reply = if text.is_empty() {
                    None
                } else {
                    Some(Self::positive(text)?)
                };
                if let Some(parent) = reply {
                    let default = board.reply_subject(self.session()?, room, parent)?;
                    let preview = faction_terminal::subject::preview_with(
                        &format!("Reply subject: {default}"),
                        usize::from(self.size().0).saturating_sub(1),
                        self.capabilities.unicode,
                    );
                    self.queue(&format!("{preview}\n"));
                }
                self.prompt(
                    State::PostSubject(room, reply),
                    "Subject (blank for generated/reply default): ",
                );
            }
            State::PostSubject(room, reply) => self.prompt(
                State::PostBody(room, reply, text.into()),
                "Message (one line, up to 1024 bytes): ",
            ),
            State::PostBody(room, reply, subject) => {
                let key = faction_core::request_id()?;
                self.queue(&format!("Submission ID: {key}\n"));
                self.submit(board, room, reply, subject, text.into(), key)?;
            }
            State::PostRetry {
                room,
                reply,
                subject,
                body,
                key,
            } => {
                if text.eq_ignore_ascii_case("y") {
                    self.submit(board, room, reply, subject, body, key)?;
                } else {
                    self.pause();
                }
            }
            State::Sysop => match text.to_ascii_lowercase().as_str() {
                "l" => self.members(board, 0)?,
                "a" | "v" => self.prompt(
                    State::MemberStatus(text.eq_ignore_ascii_case("a")),
                    "Member handle: ",
                ),
                "c" => self.prompt(State::RoomSlug, "Room slug: "),
                _ => {
                    self.queue("Unknown sysop command.\n");
                    self.pause();
                }
            },
            State::MemberStatus(approve) => {
                board.set_status(
                    self.session()?,
                    text,
                    if approve {
                        Status::Approved
                    } else {
                        Status::Revoked
                    },
                )?;
                self.queue("Status updated; previous sessions expired.\n");
                self.pause();
            }
            State::RoomSlug => self.prompt(State::RoomTitle(text.into()), "Room title: "),
            State::RoomTitle(slug) => {
                let id = board.create_room(self.session()?, &slug, text)?;
                self.queue(&format!("Created room #{id}.\n"));
                self.pause();
            }
            State::Chat => {
                self.state = State::Chat;
                self.chat_command(text, board, theme)?;
            }
            _ => return Err(Error::Denied),
        }
        self.prompt_started = Instant::now();
        Ok(())
    }
    fn credential(
        &mut self,
        register: bool,
        name: String,
        secret: Zeroizing<String>,
        jobs: &CredentialJobs,
    ) -> faction_core::Result<()> {
        match jobs.try_send(Request {
            connection: self.connection,
            register,
            name,
            secret,
        }) {
            Ok(()) => self.state = State::Authenticating,
            Err(TrySendError::Full(_)) => {
                self.queue("credential work busy; reconnect later\n");
                self.state = State::Closing;
            }
            Err(TrySendError::Disconnected(_)) => return Err(Error::Denied),
        }
        Ok(())
    }
    fn channel(&self) -> faction_core::Result<&str> {
        if self.channel.is_empty() {
            Err(Error::Invalid("join a channel first"))
        } else {
            Ok(&self.channel)
        }
    }
    pub(super) fn chat_prompt(&mut self) {
        let label = format!(
            "chat[{}]> ",
            if self.channel.is_empty() {
                "-"
            } else {
                &self.channel
            }
        );
        if self.ansi {
            let width = usize::from(self.dimensions.0).saturating_sub(label.len() + 1);
            self.queue(&format!("\r\x1b[2K{label}{}", self.editor.viewport(width)));
        } else {
            self.queue(&label);
        }
    }
    pub(super) fn chat_page(
        &mut self,
        board: &mut Board,
        after: Option<i64>,
    ) -> faction_core::Result<()> {
        if self.channel.is_empty() {
            return Ok(());
        }
        let page = board.chat_read(self.lease()?, self.channel()?, after.unwrap_or(self.cursor))?;
        if page.messages.is_empty() && !page.gap {
            return Ok(());
        }
        if self.ansi {
            self.queue("\r\x1b[2K");
        }
        if page.gap {
            self.queue("History gap: older chat expired; showing retained messages.\n");
        }
        for m in page.messages {
            self.queue(&format!(
                "{} #{} {}{}{}\n",
                if m.kind == ChatKind::Emote { "*" } else { "<" },
                m.sequence,
                Self::safe(&m.author),
                if m.kind == ChatKind::Emote { " " } else { "> " },
                Self::safe(&m.body)
            ));
        }
        if page.more {
            self.queue(&format!(
                "More retained history: /history {}\n",
                page.cursor
            ));
        }
        self.cursor = self.cursor.max(page.cursor);
        if self.ansi {
            self.chat_prompt();
        }
        Ok(())
    }
    fn chat_command(
        &mut self,
        text: &str,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        let command = if text.trim().is_empty() {
            ChatCommand::Poll
        } else {
            ChatCommand::parse(text)?
        };
        match command {
            ChatCommand::Help => self.queue("/list /join #channel /part /names [after-handle] /history [after-sequence] /poll /me text /quit\nPlain text sends; //text sends a literal slash.\nSysop: /create #channel board|private /allow #channel handle /deny #channel handle /kick #channel handle\nAll channels are local; history begins at your join.\n"),
            ChatCommand::List => for c in board.chat_channels(self.lease()?)? { self.queue(&format!("{} ({}, local)\n", Self::safe(&c.name), if c.private { "private" } else { "board" })); },
            ChatCommand::Join(name) => {
                let channel = board.chat_join(self.lease()?, name)?;
                self.channel = channel.name; self.cursor = channel.sequence;
                self.queue("Joined; new messages start here. /history replays eligible retained chat.\n");
            }
            ChatCommand::Part => { board.chat_part(self.lease()?, self.channel()?)?; self.channel.clear(); self.cursor = 0; self.queue("Left channel.\n"); }
            ChatCommand::Names(after) => {
                let names = board.chat_names(self.lease()?, self.channel()?, after)?;
                self.queue(&format!("Members: {}\n", names.handles.iter().map(|h| Self::safe(h)).collect::<Vec<_>>().join(" ")));
                if let Some(next) = names.next { self.queue(&format!("More: /names {next}\n")); }
            }
            ChatCommand::History(after) => self.chat_page(board, Some(after))?,
            ChatCommand::Poll => self.chat_page(board, None)?,
            ChatCommand::Say(body) | ChatCommand::Emote(body) => {
                let kind = if matches!(command, ChatCommand::Emote(_)) { ChatKind::Emote } else { ChatKind::Say };
                board.chat_send(self.lease()?, self.channel()?, self.request, body, kind)?;
                self.request = self.request.checked_add(1).ok_or(Error::Invalid("request sequence exhausted"))?;
                self.chat_page(board, None)?;
            }
            ChatCommand::Create(name, private) => { board.chat_create(self.session()?, name, private)?; self.queue("Channel created.\n"); }
            ChatCommand::Allow(name, member, allow) => { board.chat_allow(self.session()?, name, member, allow)?; self.queue("Channel grant updated; target must join again.\n"); }
            ChatCommand::Kick(name, member) => { board.chat_kick(self.session()?, name, member)?; self.queue("Member parted; a kick does not remove a private grant.\n"); }
            ChatCommand::Quit => {
                board.chat_close(self.lease()?)?; self.lease = None; self.editor.clear();
                self.queue("Chat closed.\n"); self.menu(theme, board)?; return Ok(());
            }
        }
        self.chat_prompt();
        Ok(())
    }
    pub(super) fn recover(&mut self, error: Error, board: &mut Board, _theme: &Theme) {
        if matches!(
            error,
            Error::StaleSession
                | Error::UnsafeState
                | Error::UnsupportedSchema
                | Error::Io(_)
                | Error::Entropy
                | Error::PasswordHash
                | Error::Credentials
        ) || (self.session.is_some() && self.approved(board).is_err())
            || self.session.is_none()
        {
            self.error(error);
            self.state = State::Closing;
            return;
        }
        if matches!(self.state, State::Chat) {
            if matches!(error, Error::Denied) {
                self.channel.clear();
                self.cursor = 0;
                self.editor.clear();
                self.queue("Channel access changed or action denied; draft discarded. Join explicitly before sending.\n");
            } else {
                self.error(error);
            }
            self.chat_prompt();
        } else {
            self.error(error);
            self.pause();
        }
    }
    pub(super) fn resize(&mut self, theme: &Theme, board: &mut Board) -> faction_core::Result<()> {
        if matches!(
            self.state,
            State::DoorMenu | State::DoorInput | State::DoorPending | State::Classic
        ) && self.size() != self.dimensions
        {
            self.door = None;
            self.ansi = false;
            self.dimensions = self.size();
            self.classic = None;
            self.queue("Terminal resized; game closed.\n");
            self.menu(theme, board)?;
            return Ok(());
        }
        if !matches!(self.state, State::Menu | State::Chat) || self.size() == self.dimensions {
            return Ok(());
        }
        self.dimensions = self.size();
        self.decoder = Decoder::new();
        self.input.zeroize();
        self.input.clear();
        self.editor.clear();
        let _ = termios::tcflush(&self.tty, termios::QueueSelector::IFlush);
        if self.ansi {
            self.queue("\x1b[0m\x1b[?25h\n");
        }
        // This menu action falls back to plain even when the new size is capable.
        self.ansi = false;
        self.queue("Terminal resized; draft discarded. Plain input until the next menu.\n");
        if matches!(self.state, State::Menu) {
            let sysop = board.whoami(self.session()?)?.role == faction_core::Role::Sysop;
            self.queue(&theme.plain_menu(sysop));
            self.queue("Command (then Enter): ");
        } else {
            self.chat_prompt();
        }
        Ok(())
    }
    pub(super) fn input(
        &mut self,
        board: &mut Board,
        jobs: &CredentialJobs,
        theme: &Theme,
    ) -> Result<()> {
        let mut bytes = Zeroizing::new([0u8; 64]);
        let n = match self.tty.read(&mut *bytes) {
            Ok(0) => {
                self.close = true;
                return Ok(());
            }
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        if matches!(self.state, State::Classic) {
            self.approved(board)?;
            for &byte in &bytes[..n] {
                if byte == 29 {
                    self.classic_return("Classic door closed.", theme, board)?;
                    termios::tcflush(&self.tty, termios::QueueSelector::IFlush)?;
                    break;
                }
                if self.classic.as_mut().ok_or_else(denied)?.key(byte).is_err() {
                    self.classic_return("Classic door unavailable.", theme, board)?;
                    break;
                }
                self.prompt_started = Instant::now();
            }
            return Ok(());
        }
        if let State::Probe(probe) = &mut self.state {
            for &byte in &bytes[..n] {
                probe.feed(byte).map_err(Error::Invalid)?;
            }
            return Ok(());
        }
        let mut redraw = false;
        for &byte in &bytes[..n] {
            if matches!(self.state, State::Closing | State::Authenticating) {
                break;
            }
            if self.skip_lf {
                self.skip_lf = false;
                if byte == b'\n' {
                    continue;
                }
            }
            if byte == 3 || byte == 4 {
                if matches!(
                    self.state,
                    State::DoorMenu | State::DoorInput | State::DoorPending
                ) {
                    self.door = None;
                    self.menu(theme, board)?;
                    termios::tcflush(&self.tty, termios::QueueSelector::IFlush)?;
                } else if matches!(self.state, State::Chat) {
                    board.chat_close(self.lease()?)?;
                    self.lease = None;
                    self.editor.clear();
                    self.queue("Chat closed.\n");
                    self.menu(theme, board)?;
                    termios::tcflush(&self.tty, termios::QueueSelector::IFlush)?;
                } else {
                    self.close = true;
                }
                break;
            }
            if matches!(self.state, State::DoorPending) {
                continue;
            }
            if matches!(self.state, State::DoorMenu) && self.ansi {
                if let Some(key) = self.decoder.feed(byte, Instant::now()) {
                    match self.door_key(key, theme, board) {
                        Ok(true) => {
                            self.input.clear();
                            self.decoder = Decoder::new();
                            termios::tcflush(&self.tty, termios::QueueSelector::IFlush)?;
                            break;
                        }
                        Ok(false) => (),
                        Err(e) => {
                            self.recover(e, board, theme);
                            break;
                        }
                    }
                }
                continue;
            }
            if matches!(self.state, State::Menu) && self.ansi {
                if let Some(key) = self.decoder.feed(byte, Instant::now()) {
                    match self.action(key, board, theme) {
                        Ok(true) => {
                            self.input.clear();
                            self.decoder = Decoder::new();
                            termios::tcflush(&self.tty, termios::QueueSelector::IFlush)?;
                            break; // No menu typeahead can become editor input.
                        }
                        Ok(false) => (),
                        Err(e) => {
                            self.recover(e, board, theme);
                            break;
                        }
                    }
                }
                continue;
            }
            if matches!(self.state, State::Chat) {
                let before = self.editor.text().len();
                let event = self.editor.feed(byte, Instant::now());
                if !self.ansi && self.editor.text().len() < before && (byte == 8 || byte == 127) {
                    self.queue("\x08 \x08");
                }
                match event {
                    Some(LineEvent::Submit(text)) => {
                        self.queue("\n");
                        if let Err(e) = self.line(&text, board, jobs, theme) {
                            self.state = State::Chat;
                            self.recover(e, board, theme);
                        }
                        if !matches!(self.state, State::Chat) {
                            termios::tcflush(&self.tty, termios::QueueSelector::IFlush)?;
                        }
                        break;
                    }
                    Some(LineEvent::Discarded) => {
                        self.queue("Draft discarded; press Enter to recover.\n");
                        redraw = true;
                    }
                    Some(LineEvent::Cancel) => {
                        self.close = true;
                        break;
                    }
                    None => {
                        redraw = true;
                        if !self.ansi && (32..=126).contains(&byte) {
                            self.queue(&(byte as char).to_string());
                        }
                    }
                }
                continue;
            }
            if matches!(self.state, State::PostSubject(..)) {
                use faction_terminal::subject::{self, Edit};
                match subject::feed_with(&mut self.input, byte, self.capabilities.unicode)
                    .map_err(Error::Invalid)?
                {
                    Edit::Echo(text) => self.queue(&text),
                    Edit::Submit(text) => {
                        self.skip_lf = byte == b'\r';
                        self.queue("\n");
                        if let Err(e) = self.line(&text, board, jobs, theme) {
                            self.recover(e, board, theme);
                        }
                        termios::tcflush(&self.tty, termios::QueueSelector::IFlush)?;
                        break;
                    }
                }
                continue;
            }
            let input_unicode =
                self.capabilities.unicode && matches!(self.state, State::PostBody(..));
            let hidden = matches!(
                self.state,
                State::Password(..) | State::Confirm(..) | State::NativeFiles
            );
            if byte == b'\n' || byte == b'\r' {
                self.skip_lf = byte == b'\r';
                // Validate borrowed bytes so even invalid UTF-8 secrets stay in a
                // zeroizing allocation; FromUtf8Error must never own plaintext.
                let text = Zeroizing::new(std::str::from_utf8(&self.input)?.to_owned());
                self.input.zeroize();
                self.input.clear();
                self.queue("\n");
                if let Err(e) = self.line(&text, board, jobs, theme) {
                    self.recover(e, board, theme);
                }
                termios::tcflush(&self.tty, termios::QueueSelector::IFlush)?;
                break;
            } else if byte == 8 || byte == 127 {
                let erased = std::str::from_utf8(&self.input)
                    .ok()
                    .and_then(|s| s.chars().last())
                    .map_or(0, |c| {
                        if input_unicode {
                            faction_terminal::text::width(&faction_terminal::text::scalar(c, true))
                        } else {
                            Self::safe(&c.to_string()).len()
                        }
                    });
                self.input.pop();
                while !self.input.is_empty() && std::str::from_utf8(&self.input).is_err() {
                    self.input.pop();
                }
                if !hidden {
                    self.queue(&"\x08 \x08".repeat(erased));
                }
            } else if byte >= 32 && byte != 127 {
                let cap = match self.state {
                    State::Password(..) | State::Confirm(..) => 128,
                    State::PostSubject(..) => 160,
                    State::PostBody(..) | State::Files | State::NativeFiles => 1024,
                    State::FileUploadKey { .. } | State::FileDownloadKey(_) => 256,
                    _ => 128,
                };
                if self.input.len() >= cap {
                    self.close = true;
                    break;
                }
                self.input.push(byte);
                if !hidden {
                    if byte.is_ascii() {
                        self.queue(&(byte as char).to_string());
                    } else if let Ok(text) = std::str::from_utf8(&self.input)
                        && let Some(c) = text.chars().last()
                    {
                        self.queue(&if input_unicode {
                            faction_terminal::text::scalar(c, true)
                        } else {
                            Self::safe(&c.to_string())
                        });
                    }
                }
            } else {
                self.close = true;
                break;
            }
        }
        if redraw && self.ansi && matches!(self.state, State::Chat) {
            self.chat_prompt();
        }
        Ok(())
    }
}
