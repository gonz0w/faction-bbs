//! Shared terminal session presentation. Owner command parsing lives only in faction.
#![forbid(unsafe_code)]
mod chat;
mod directory;
pub mod operator;
mod terminal;
use faction_core::{Board, Error, POST_BYTES, Role, Session, Status, request_id};
use faction_terminal::{Action, Theme};
use rustix::termios::{self, InputModes, LocalModes, OptionalActions, SpecialCodeIndex, Termios};
use std::time::{Duration, Instant};
use std::{
    fs::File,
    io::{self, Read, Write},
};
use zeroize::Zeroizing;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn invalid(reason: &'static str) -> Box<dyn std::error::Error> {
    Error::Invalid(reason).into()
}
pub fn line(prompt: &str, max: usize) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    bounded_line(
        &mut TimedInput {
            file: tty()?,
            deadline: Instant::now() + Duration::from_secs(120),
        },
        max,
    )
}
fn subject_line(unicode: bool) -> Result<String> {
    use faction_terminal::subject::{self, Edit};
    let tty = tty()?;
    let saved = termios::tcgetattr(&tty)?;
    let mut mode = saved.clone();
    mode.local_modes.remove(
        LocalModes::ECHO
            | LocalModes::ECHONL
            | LocalModes::ICANON
            | LocalModes::ISIG
            | LocalModes::IEXTEN,
    );
    mode.input_modes.remove(
        InputModes::ISTRIP
            | InputModes::ICRNL
            | InputModes::INLCR
            | InputModes::IGNCR
            | InputModes::IXON
            | InputModes::IXOFF,
    );
    mode.special_codes[SpecialCodeIndex::VMIN] = 1;
    mode.special_codes[SpecialCodeIndex::VTIME] = 0;
    termios::tcsetattr(&tty, OptionalActions::Flush, &mode)?;
    let _restore = Hidden { tty: &tty, saved };
    print!("Subject (blank for generated/reply default): ");
    io::stdout().flush()?;
    let mut source = TimedInput {
        file: tty.try_clone()?,
        deadline: Instant::now() + Duration::from_secs(120),
    };
    let mut input = Vec::with_capacity(subject::BYTES);
    loop {
        let mut byte = [0];
        source.read_exact(&mut byte)?;
        match subject::feed_with(&mut input, byte[0], unicode)
            .map_err(|reason| io::Error::new(io::ErrorKind::InvalidData, reason))?
        {
            Edit::Echo(text) => {
                print!("{text}");
                io::stdout().flush()?;
            }
            Edit::Submit(text) => {
                println!();
                return Ok(text);
            }
        }
    }
}
/// Duplicate the already-open terminal. Broker workers deliberately have no
/// controlling /dev/tty; stdin is the descriptor admitted by the broker.
fn tty() -> Result<File> {
    if !std::io::IsTerminal::is_terminal(&io::stdin()) {
        return Err(invalid("a terminal is required"));
    }
    Ok(File::from(rustix::io::fcntl_dupfd_cloexec(io::stdin(), 3)?))
}
struct TimedInput {
    file: File,
    deadline: Instant,
}
impl Read for TimedInput {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        use rustix::event::{PollFd, PollFlags, Timespec, poll};
        let remaining = self
            .deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| io::Error::from(io::ErrorKind::TimedOut))?;
        let mut fds = [PollFd::new(&self.file, PollFlags::IN)];
        let timeout = Timespec {
            tv_sec: remaining.as_secs() as _,
            tv_nsec: i64::from(remaining.subsec_nanos()) as _,
        };
        if poll(&mut fds, Some(&timeout))? == 0 || !fds[0].revents().contains(PollFlags::IN) {
            return Err(io::Error::from(io::ErrorKind::TimedOut));
        }
        self.file.read(bytes)
    }
}
fn bounded_line(input: &mut impl Read, max: usize) -> Result<String> {
    let mut bytes = Vec::new();
    let mut byte = [0];
    loop {
        if input.read(&mut byte)? == 0 {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof).into());
        }
        if byte[0] == b'\n' {
            break;
        }
        if bytes.len() == max {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "input too long; console closed",
            )
            .into());
        }
        bytes.push(byte[0]);
    }
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    Ok(String::from_utf8(bytes)?)
}
struct Hidden<'a> {
    tty: &'a File,
    saved: Termios,
}
impl Drop for Hidden<'_> {
    fn drop(&mut self) {
        let _ = termios::tcsetattr(self.tty, OptionalActions::Flush, &self.saved);
    }
}
pub fn secret(prompt: &str) -> Result<Zeroizing<String>> {
    let tty = tty()?;
    let saved = termios::tcgetattr(&tty)?;
    let mut mode = saved.clone();
    mode.local_modes.remove(
        LocalModes::ECHO
            | LocalModes::ECHONL
            | LocalModes::ICANON
            | LocalModes::ISIG
            | LocalModes::IEXTEN,
    );
    mode.special_codes[SpecialCodeIndex::VMIN] = 1;
    mode.special_codes[SpecialCodeIndex::VTIME] = 0;
    termios::tcsetattr(&tty, OptionalActions::Flush, &mode)?;
    let guard = Hidden { tty: &tty, saved };
    // Announce readiness only after echo has been disabled.
    (&tty).write_all(prompt.as_bytes())?;
    (&tty).flush()?;
    let mut bytes = Zeroizing::new(Vec::with_capacity(128));
    let mut byte = Zeroizing::new([0]);
    let mut input = TimedInput {
        file: tty.try_clone()?,
        deadline: Instant::now() + Duration::from_secs(120),
    };
    let result = (|| -> Result<Zeroizing<String>> {
        loop {
            input.read_exact(&mut *byte)?;
            match byte[0] {
                b'\n' | b'\r' => break,
                3 | 4 => return Err(io::Error::from(io::ErrorKind::Interrupted).into()),
                8 | 127 => {
                    bytes.pop();
                    while !bytes.is_empty() && std::str::from_utf8(&bytes).is_err() {
                        bytes.pop();
                    }
                }
                0..=31 => return Err(invalid("password contains a control character")),
                b => {
                    if bytes.len() == 128 {
                        return Err(invalid("password exceeds 128 bytes"));
                    }
                    bytes.push(b);
                }
            }
        }
        Ok(Zeroizing::new(std::str::from_utf8(&bytes)?.to_owned()))
    })();
    drop(guard);
    (&tty).write_all(b"\n")?;
    result
}
pub fn new_secret() -> Result<Zeroizing<String>> {
    let value = secret("New password (12+ characters, hidden): ")?;
    let again = secret("Confirm password (hidden): ")?;
    if *value != *again {
        return Err(invalid("passwords do not match"));
    }
    Ok(value)
}
/// Escape every non-ASCII/control byte sequence supplied by members or sysops.
fn safe(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for c in value.chars() {
        if c.is_ascii_graphic() || c == ' ' || c == '\n' {
            output.push(c);
        } else {
            output.extend(c.escape_unicode());
        }
    }
    output
}
fn number(prompt: &str) -> Result<i64> {
    let n: i64 = line(prompt, 20)?.parse()?;
    if n < 1 {
        return Err(invalid("positive ID required"));
    }
    Ok(n)
}
fn show_rooms(board: &mut Board, s: &Session) -> Result<()> {
    for r in board.rooms(s)? {
        println!(
            "{}  {} - {} ({} unread)",
            r.id,
            safe(&r.slug),
            safe(&r.title),
            r.unread
        );
    }
    println!("Room listing is limited to the first 100 rooms in this increment.");
    Ok(())
}
fn read_room(
    board: &mut Board,
    s: &Session,
    new_only: bool,
    theme: &Theme,
    ansi: bool,
    caps: faction_terminal::capabilities::Capabilities,
) -> Result<()> {
    let room = number("Room ID: ")?;
    let mut after = if new_only {
        board.unread_cursor(s, room)?
    } else {
        0
    };
    let mut offset = 0;
    loop {
        let rows = board.posts(s, room, after)?;
        let Some(post) = rows.first() else {
            println!("No more messages.");
            break;
        };
        let metadata = board.post_metadata(s, post.id)?;
        let (columns, height) = terminal::size().unwrap_or((80, 24));
        let page = faction_terminal::Message {
            room,
            id: post.id,
            author: &post.author,
            reply_to: post.reply_to,
            body: &post.body,
        }
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
            caps.unicode,
        )
        .map_err(invalid)?;
        print!(
            "{}",
            caps.frame(
                &format!(
                    "{}{}",
                    caps.section(theme, "Read messages", ansi && terminal::capable()),
                    page.text
                ),
                ansi && terminal::capable()
            )
        );
        io::stdout().flush()?;
        if let Some(next) = page.next {
            offset = next;
        } else {
            after = post.id;
            offset = 0;
            board.mark_read(s, room, after)?;
        }
        if !line("More? [y/N]: ", 8)?.eq_ignore_ascii_case("y") {
            break;
        }
    }
    Ok(())
}
fn post(board: &mut Board, s: &Session, unicode: bool) -> Result<()> {
    let room = number("Room ID: ")?;
    let parent = line("Reply to post ID (blank for new message): ", 20)?;
    let reply = if parent.is_empty() {
        None
    } else {
        Some(parent.parse()?)
    };
    if let Some(parent) = reply {
        let preview = format!("Reply subject: {}", board.reply_subject(s, room, parent)?);
        println!(
            "{}",
            faction_terminal::subject::preview_with(
                &preview,
                usize::from(terminal::size().unwrap_or((80, 24)).0).saturating_sub(1),
                unicode
            )
        );
    }
    let subject = subject_line(unicode)?;
    let body = line("Message (one line, up to 1024 bytes): ", POST_BYTES)?;
    let key = request_id()?;
    println!("Submission ID: {key}");
    loop {
        match board.post_article(
            s,
            room,
            &key,
            faction_core::PostDraft {
                subject: &subject,
                body: &body,
                reply_to: reply,
            },
        ) {
            Ok(p) => {
                println!("Posted #{}.", p.id);
                return Ok(());
            }
            Err(Error::Busy) => {
                if !line("Database busy. Retry this same submission? [y/N]: ", 8)?
                    .eq_ignore_ascii_case("y")
                {
                    return Ok(());
                }
            }
            Err(e) => return Err(e.into()),
        }
    }
}
fn terminal_settings(
    last: &mut Option<std::time::Instant>,
    caps: &mut faction_terminal::capabilities::Capabilities,
    ansi: bool,
    board: &mut Board,
    session: &Session,
) -> Result<()> {
    let mut image: Option<terminal::Graphics> = None;
    loop {
        if board.whoami(session)?.status != Status::Approved {
            return Err(Error::Denied.into());
        }
        let (columns, rows) = terminal::size().unwrap_or((0, 0));
        let screen = if image.is_some() {
            format!(
                "Graphics preview. C clears; D returns.\n{}",
                faction_terminal::capabilities::PROMPT
            )
        } else {
            caps.screen(columns, rows, ansi && terminal::capable())
        };
        print!("{}", caps.frame(&screen, ansi && terminal::capable()));
        let choice = line("", 1)?;
        if choice.eq_ignore_ascii_case("d") {
            return Ok(());
        }
        if choice.eq_ignore_ascii_case("g") {
            if ansi && terminal::capable() {
                let now = std::time::Instant::now();
                if last
                    .is_none_or(|t| now.duration_since(t) >= faction_terminal::graphics::INTERVAL)
                {
                    drop(image.take());
                    image = terminal::graphics()?;
                    *last = Some(std::time::Instant::now());
                    if image.is_none() {
                        println!("Kitty unavailable; classic fallback.");
                    }
                } else {
                    println!("Graphics limited to one frame per second.");
                }
            } else {
                println!("ANSI disabled; probe refused.");
            }
        } else if choice.eq_ignore_ascii_case("p") {
            if ansi && terminal::capable() {
                caps.synchronized = terminal::probe_sync()?;
            } else {
                println!("ANSI disabled; probe refused.");
            }
        } else {
            if choice.eq_ignore_ascii_case("c") {
                drop(image.take());
            }
            caps.choose(&choice);
        }
    }
}
fn sysop(board: &mut Board, s: &Session) -> Result<()> {
    let command =
        line("Sysop: [L]ist [A]pprove [V]revoke [C]reate room: ", 8)?.to_ascii_lowercase();
    match command.as_str() {
        "l" => {
            for m in board.members(s)? {
                println!("{} {:?} {:?}", safe(&m.handle), m.status, m.role)
            }
            println!("First 100 accounts; approval by exact handle works beyond this list.");
        }
        "a" | "v" => {
            let name = line("Member handle: ", 24)?;
            board.set_status(
                s,
                &name,
                if command == "a" {
                    Status::Approved
                } else {
                    Status::Revoked
                },
            )?;
            println!("Status updated; previous sessions expired.");
        }
        "c" => {
            let slug = line("Room slug: ", 32)?;
            let title = line("Room title: ", 64)?;
            let id = board.create_room(s, &slug, &title)?;
            println!("Created room #{id}.");
        }
        _ => println!("Unknown sysop command."),
    }
    Ok(())
}
/// Choose only after current local approval; keep this borrowed theme throughout
/// the complete local/worker menu, editor, room and chat session.
pub fn scheduled_menu(
    board: &mut Board,
    s: &Session,
    themes: &faction_terminal::seasons::Themes,
    ansi: bool,
) -> Result<()> {
    let index = if board.whoami(s)?.status == Status::Approved {
        themes.select_now()
    } else {
        0
    };
    menu(board, s, themes.get(index), ansi)
}
pub fn scheduled_menu_with_directory(
    board: &mut Board,
    s: &Session,
    themes: &faction_terminal::seasons::Themes,
    directory: &faction_terminal::directory::Directory,
    ansi: bool,
) -> Result<()> {
    let index = if board.whoami(s)?.status == Status::Approved {
        themes.select_now()
    } else {
        0
    };
    menu_with_directory(board, s, themes.get(index), directory, ansi)
}
pub fn menu(board: &mut Board, s: &Session, theme: &Theme, ansi: bool) -> Result<()> {
    menu_with_directory(
        board,
        s,
        theme,
        &faction_terminal::directory::Directory::default(),
        ansi,
    )
}
fn menu_with_directory(
    board: &mut Board,
    s: &Session,
    theme: &Theme,
    directory: &faction_terminal::directory::Directory,
    ansi: bool,
) -> Result<()> {
    let member = board.whoami(s)?;
    if member.status != Status::Approved {
        println!(
            "Account {:?}; sysop approval required. No community access.",
            member.status
        );
        return Ok(());
    }
    let mut caps = faction_terminal::capabilities::Capabilities::default();
    let mut last_graphics = None;
    terminal::identity(theme, ansi, || {
        if board.whoami(s)?.status != Status::Approved {
            return Err(Error::Denied.into());
        }
        Ok(())
    })?;
    println!("Welcome, {}. Faction BBS session.", safe(&member.handle));
    loop {
        let member = board.whoami(s)?;
        if member.status != Status::Approved {
            return Err(Error::Denied.into());
        }
        let selected = if ansi {
            terminal::select(theme, member.role == Role::Sysop, caps)?
        } else {
            None
        };
        let used_ansi = selected.is_some();
        let action = if let Some(action) = selected {
            Some(action)
        } else {
            print!("{}", theme.plain_menu(member.role == Role::Sysop));
            let command = line("Command (then Enter): ", 8)?;
            if command.len() == 1 {
                theme.hotkey(command.as_bytes()[0], member.role == Role::Sysop)
            } else {
                None
            }
        };
        if let Some(action) = action {
            let title = match action {
                Action::Rooms => "Rooms",
                Action::New | Action::History => "Read messages",
                Action::Post => "Post a message",
                Action::Sysop => "Sysop controls",
                Action::Chat => "Local chat",
                Action::Doors => "Doors",
                Action::Files => "File areas",
                Action::Quit => "Disconnect",
                Action::Terminal => "Terminal settings",
                Action::Contacts => "Other BBSes",
            };
            print!("{}", caps.section(theme, title, used_ansi));
        }
        let result = match action {
            Some(Action::Quit) => {
                println!("{}\nLogged out.", theme.logout());
                return Ok(());
            }
            Some(Action::Contacts) => directory::run(board, s, theme, directory, used_ansi),
            Some(Action::Rooms) => show_rooms(board, s),
            Some(Action::New) => read_room(board, s, true, theme, used_ansi, caps),
            Some(Action::History) => read_room(board, s, false, theme, used_ansi, caps),
            Some(Action::Post) => post(board, s, caps.unicode),
            Some(Action::Terminal) => {
                terminal_settings(&mut last_graphics, &mut caps, ansi, board, s)
            }
            Some(Action::Sysop) => sysop(board, s),
            Some(Action::Chat) => chat::run(board, s, ansi),
            _ => {
                println!("Unknown or unavailable command.");
                Ok(())
            }
        };
        if let Err(e) = result {
            if matches!(
                e.downcast_ref::<Error>(),
                Some(
                    Error::Invalid(_)
                        | Error::NotFound
                        | Error::RequestConflict
                        | Error::Busy
                        | Error::Storage(_)
                )
            ) {
                println!("Action failed: {e}");
            } else {
                return Err(e);
            }
        }
        if used_ansi && !matches!(action, Some(Action::Contacts)) {
            line("Press Enter to return to menu: ", 1)?;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn member_output_is_inert() {
        assert_eq!(
            safe("hi\x1b[2J\u{202e}é\n"),
            "hi\\u{1b}[2J\\u{202e}\\u{e9}\n"
        );
    }
    #[test]
    fn line_limit_does_not_accept_truncated_command() {
        assert!(bounded_line(&mut &b"approveextra\n"[..], 7).is_err());
        assert_eq!(bounded_line(&mut &b"hello\n"[..], 5).unwrap(), "hello");
        assert!(bounded_line(&mut &b"missing newline"[..], 32).is_err());
    }
}
