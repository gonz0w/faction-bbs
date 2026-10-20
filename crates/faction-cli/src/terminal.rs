//! Local TTY adapter only. Future network adapters drive the same pure decoder.
use crate::{Result, line};
use faction_terminal::{Action, Decoder, Menu, Theme};
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    termios::{self, InputModes, LocalModes, OptionalActions, SpecialCodeIndex, Termios},
};
use std::{
    fs::{File, OpenOptions},
    io::{self, IsTerminal, Read, Write},
    time::Instant,
};
pub(super) struct Raw {
    tty: File,
    saved: Termios,
}
impl Raw {
    pub(super) fn open() -> Result<Self> {
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
        mode.input_modes.remove(
            InputModes::ICRNL
                | InputModes::INLCR
                | InputModes::IGNCR
                | InputModes::IXON
                | InputModes::IXOFF,
        );
        mode.special_codes[SpecialCodeIndex::VMIN] = 1;
        mode.special_codes[SpecialCodeIndex::VTIME] = 0;
        termios::tcsetattr(&tty, OptionalActions::Flush, &mode)?;
        Ok(Self { tty, saved })
    }
    pub(super) fn read_byte(&self) -> io::Result<u8> {
        let mut byte = [0];
        (&self.tty).read_exact(&mut byte)?;
        Ok(byte[0])
    }
}
pub(super) fn ready() -> Result<bool> {
    let stdin = io::stdin();
    let mut fds = [PollFd::new(&stdin, PollFlags::IN)];
    poll(
        &mut fds,
        Some(&Timespec {
            tv_sec: 0,
            tv_nsec: 100_000_000,
        }),
    )?;
    if fds[0]
        .revents()
        .intersects(PollFlags::HUP | PollFlags::ERR | PollFlags::NVAL)
    {
        return Err(io::Error::from(io::ErrorKind::UnexpectedEof).into());
    }
    Ok(fds[0].revents().contains(PollFlags::IN))
}
impl Drop for Raw {
    fn drop(&mut self) {
        let _ = termios::tcsetattr(&self.tty, OptionalActions::Flush, &self.saved);
        let _ = io::stdout().write_all(b"\x1b[0m\x1b[?25h\r\n");
        let _ = io::stdout().flush();
    }
}
pub(super) fn size() -> Option<(u16, u16)> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return None;
    }
    let w = termios::tcgetwinsize(io::stdin()).ok()?;
    Some((w.ws_col, w.ws_row))
}
pub(super) fn capable() -> bool {
    matches!(size(), Some((40..=240, 18..=100)))
        && std::env::var("TERM").is_ok_and(|s| s != "dumb" && !s.is_empty())
}
pub fn identity(theme: &Theme, ansi: bool) -> Result<()> {
    let mut output = io::stdout();
    if ansi && capable() && matches!(size(), Some((80..=240, 24..=100))) {
        output.write_all(b"\x1b[0;37;40m\x1b[2J\x1b[H")?;
        if let Some(art) = theme.identity_ansi() {
            output.write_all(art.render().as_bytes())?;
        } else {
            output.write_all(theme.identity_ascii().unwrap_or(theme.board()).as_bytes())?;
        }
        write!(output, "\x1b[18;1H{}\r\n", theme.welcome())?;
        output.flush()?;
        line("Press Enter to continue: ", 1)?;
    } else {
        writeln!(
            output,
            "\n{}\n{}",
            theme.identity_ascii().unwrap_or(theme.board()),
            theme.welcome()
        )?;
        output.flush()?;
    }
    Ok(())
}
/// None requests the canonical line-mode fallback. Drop restores terminal state
/// and flushes pending typeahead before editor/sysop prompts get a new context.
pub fn select(theme: &Theme, sysop: bool) -> Result<Option<Action>> {
    if !capable() {
        return Ok(None);
    }
    let dimensions = size();
    let raw = Raw::open()?;
    let (mut decoder, mut menu) = (Decoder::new(), Menu::new(theme, sysop));
    let mut output = io::stdout();
    output.write_all(b"\x1b[?25l")?;
    output.write_all(menu.render().as_bytes())?;
    output.flush()?;
    let stdin = io::stdin();
    loop {
        if size() != dimensions {
            return Ok(None);
        }
        // Poll the actual stdin TTY, not /dev/tty (Darwin cannot poll that alias).
        let mut fds = [PollFd::new(&stdin, PollFlags::IN)];
        poll(
            &mut fds,
            Some(&Timespec {
                tv_sec: 0,
                tv_nsec: 100_000_000,
            }),
        )?;
        decoder.expire(Instant::now());
        if fds[0]
            .revents()
            .intersects(PollFlags::HUP | PollFlags::ERR | PollFlags::NVAL)
        {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof).into());
        }
        if !fds[0].revents().contains(PollFlags::IN) {
            continue;
        }
        let mut byte = [0];
        (&raw.tty).read_exact(&mut byte)?;
        if let Some(key) = decoder.feed(byte[0], Instant::now()) {
            let previous = menu.focus();
            if let Some(action) = menu.feed(key) {
                return Ok(Some(action));
            }
            output.write_all(menu.redraw(previous).as_bytes())?;
            output.flush()?;
        }
    }
}
