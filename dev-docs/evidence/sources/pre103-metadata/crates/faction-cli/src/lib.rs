//! Shared terminal session presentation. Owner command parsing lives only in faction.
#![forbid(unsafe_code)]
mod chat;
mod terminal;
use faction_core::{Board, Error, POST_BYTES, Role, Session, Status, request_id};
use faction_terminal::{Action, Theme};
use rustix::termios::{self, LocalModes, OptionalActions, SpecialCodeIndex, Termios};
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
        let (columns, height) = terminal::size().unwrap_or((80, 24));
        let page = faction_terminal::Message {
            room,
            id: post.id,
            author: &post.author,
            reply_to: post.reply_to,
            body: &post.body,
        }
        .page(columns, height, offset)
        .map_err(invalid)?;
        print!(
            "{}{}",
            faction_terminal::section(theme, "Read messages", ansi && terminal::capable()),
            page.text
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
fn post(board: &mut Board, s: &Session) -> Result<()> {
    let room = number("Room ID: ")?;
    let parent = line("Reply to post ID (blank for new message): ", 20)?;
    let reply = if parent.is_empty() {
        None
    } else {
        Some(parent.parse()?)
    };
    let body = line("Message (one line, up to 1024 bytes): ", POST_BYTES)?;
    let key = request_id()?;
    println!("Submission ID: {key}");
    loop {
        match board.post(s, room, &key, &body, reply) {
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
pub fn menu(board: &mut Board, s: &Session, theme: &Theme, ansi: bool) -> Result<()> {
    let member = board.whoami(s)?;
    if member.status != Status::Approved {
        println!(
            "Account {:?}; sysop approval required. No community access.",
            member.status
        );
        return Ok(());
    }
    terminal::identity(theme, ansi)?;
    println!("Welcome, {}. Faction BBS session.", safe(&member.handle));
    loop {
        let member = board.whoami(s)?;
        if member.status != Status::Approved {
            return Err(Error::Denied.into());
        }
        let selected = if ansi {
            terminal::select(theme, member.role == Role::Sysop)?
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
            };
            print!("{}", faction_terminal::section(theme, title, used_ansi));
        }
        let result = match action {
            Some(Action::Quit) => {
                println!("{}\nLogged out.", theme.logout());
                return Ok(());
            }
            Some(Action::Rooms) => show_rooms(board, s),
            Some(Action::New) => read_room(board, s, true, theme, used_ansi),
            Some(Action::History) => read_room(board, s, false, theme, used_ansi),
            Some(Action::Post) => post(board, s),
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
        if used_ansi {
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
