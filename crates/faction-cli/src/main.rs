//! Local development console; never install this owner-capable command as an SSH forced command.
#![forbid(unsafe_code)]
use faction_core::{Board, Error, POST_BYTES, Role, Session, Status, request_id};
use rustix::termios::{self, LocalModes, OptionalActions, SpecialCodeIndex, Termios};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn invalid(reason: &'static str) -> Box<dyn std::error::Error> {
    Error::Invalid(reason).into()
}
fn line(prompt: &str, max: usize) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    bounded_line(&mut io::stdin().lock(), max)
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
fn secret(prompt: &str) -> Result<Zeroizing<String>> {
    let tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
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
    let result = (|| -> Result<Zeroizing<String>> {
        loop {
            (&tty).read_exact(&mut *byte)?;
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
fn new_secret() -> Result<Zeroizing<String>> {
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
fn read_room(board: &mut Board, s: &Session, new_only: bool) -> Result<()> {
    let room = number("Room ID: ")?;
    let mut after = if new_only {
        board.unread_cursor(s, room)?
    } else {
        0
    };
    loop {
        let rows = board.posts(s, room, after)?;
        if rows.is_empty() {
            println!("No more messages.");
            break;
        }
        for p in &rows {
            println!(
                "\n#{} by {}{}\n{}",
                p.id,
                safe(&p.author),
                p.reply_to
                    .map(|n| format!(" (reply to #{n})"))
                    .unwrap_or_default(),
                safe(&p.body)
            );
        }
        io::stdout().flush()?;
        after = rows.last().expect("nonempty page").id;
        board.mark_read(s, room, after)?;
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
fn menu(board: &mut Board, s: &Session) -> Result<()> {
    let member = board.whoami(s)?;
    if member.status != Status::Approved {
        println!(
            "Account {:?}; sysop approval required. No community access.",
            member.status
        );
        return Ok(());
    }
    println!(
        "\nFaction BBS - local development console\nWelcome, {}.\nPlain-text surface; ANSI identity/themes and SSH are still to come.",
        safe(&member.handle)
    );
    loop {
        let member = board.whoami(s)?;
        if member.status != Status::Approved {
            return Err(Error::Denied.into());
        }
        println!(
            "\n[R]ooms  [N]ew messages  [H]istory  [P]ost  {}[Q]uit",
            if member.role == Role::Sysop {
                "[S]ysop  "
            } else {
                ""
            }
        );
        let command = line("Command (then Enter): ", 8)?.to_ascii_lowercase();
        if command == "q" {
            println!("Logged out.");
            return Ok(());
        }
        let result = match command.as_str() {
            "r" => show_rooms(board, s),
            "n" => read_room(board, s, true),
            "h" => read_room(board, s, false),
            "p" => post(board, s),
            "s" => sysop(board, s),
            _ => {
                println!("Unknown command.");
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
    }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() || args == ["--help"] {
        println!(
            "Faction local development console\nUsage: faction --state-dir PATH init|register|login|recover\nPrivate local state only. Never use as an SSH forced command.\nNo password arguments, environment credentials, listener or network service."
        );
        return Ok(());
    }
    if args.len() != 3 || args[0] != "--state-dir" {
        return Err(invalid("use --help for syntax"));
    }
    let path = PathBuf::from(&args[1]);
    let command = args[2].to_str().ok_or_else(|| invalid("invalid command"))?;
    if !["init", "register", "login", "recover"].contains(&command) {
        return Err(invalid("unknown command"));
    }
    let mut board = Board::open(&path)?;
    let name = line("Handle: ", 24)?;
    match command {
        "init" => {
            let value = new_secret()?;
            board.bootstrap(&name, &value)?;
            println!("Sysop initialized. Lobby created.");
        }
        "register" => {
            let value = new_secret()?;
            board.register(&name, &value)?;
            println!("Registered pending sysop approval.");
        }
        "login" => {
            let value = secret("Password (hidden): ")?;
            let s = board.authenticate(&name, &value)?;
            drop(value);
            menu(&mut board, &s)?;
        }
        "recover" => {
            if line("Local filesystem-owner recovery. Type RECOVER: ", 8)? != "RECOVER" {
                return Err(invalid("recovery cancelled"));
            }
            let value = new_secret()?;
            board.recover_local_sysop(&name, &value)?;
            println!("Sysop password reset; prior sessions expired.");
        }
        _ => unreachable!(),
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Faction: {e}");
        std::process::exit(1)
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
