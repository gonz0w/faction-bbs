//! Local TTY adapter only. Future network adapters drive the same pure decoder.
use crate::{Result, line};
use faction_terminal::{Action, Decoder, Menu, Theme};
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    termios::{self, InputModes, LocalModes, OptionalActions, SpecialCodeIndex, Termios},
};
use std::{
    fs::File,
    io::{self, IsTerminal, Read, Write},
    time::Instant,
};
pub(super) struct Raw {
    tty: File,
    saved: Termios,
    sync: bool,
    reset_output: bool,
}
impl Raw {
    pub(super) fn open() -> Result<Self> {
        let tty = crate::tty()?;
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
        Ok(Self {
            tty,
            saved,
            sync: false,
            reset_output: true,
        })
    }
    /// Operator panels never hide the cursor and reset each ANSI frame themselves.
    /// Plain operator exit must not emit ANSI sequences into a classic terminal.
    pub(super) fn input_only() -> Result<Self> {
        let mut raw = Self::open()?;
        raw.reset_output = false;
        Ok(raw)
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
        if self.reset_output
            && let Ok(fd) = rustix::io::fcntl_dupfd_cloexec(io::stdout(), 3)
        {
            let mut output = File::from(fd);
            let reset = if self.sync {
                b"\x1b[?2026l\x1b[0m\x1b[?25h\r\n".as_slice()
            } else {
                b"\x1b[0m\x1b[?25h\r\n".as_slice()
            };
            let _ = faction_terminal::graphics::write_tty(
                &mut output,
                reset,
                Instant::now() + std::time::Duration::from_millis(20),
            );
        }
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
        && std::env::var("TERM")
            .is_ok_and(|s| !matches!(s.as_str(), "dumb" | "faction-compact") && !s.is_empty())
}
pub fn identity(
    theme: &Theme,
    ansi_allowed: bool,
    mut approved: impl FnMut() -> Result<Option<String>>,
) -> Result<()> {
    let mut output = io::stdout();
    let mut page = 0;
    loop {
        let notice = approved()?;
        let (columns, rows) = size().unwrap_or((0, 0));
        let ansi = ansi_allowed && capable();
        if let Some(screen) = theme.identity_page_screen(columns, rows, ansi, page) {
            output.write_all(screen.as_bytes())?;
        } else {
            output.write_all(
                theme
                    .identity_fallback_screen(columns, rows, ansi)
                    .as_bytes(),
            )?;
        }
        if let Some(prompt) = theme.identity_page_prompt(page, columns, rows, ansi) {
            output.flush()?;
            loop {
                termios::tcflush(&crate::tty()?, termios::QueueSelector::IFlush)?;
                let command = line(&prompt, 1)?;
                approved()?;
                if command.is_empty() {
                    page += 1;
                    break;
                }
                if command.eq_ignore_ascii_case("s") {
                    let (cols, rows) = size().unwrap_or((0, 0));
                    output.write_all(
                        theme
                            .identity_fallback_screen(cols, rows, ansi_allowed && capable())
                            .as_bytes(),
                    )?;
                    writeln!(output, "\n{}", theme.welcome())?;
                    if let Some(notice) = approved()? {
                        writeln!(output, "{notice}")?;
                    }
                    output.flush()?;
                    line("Press Enter to continue: ", 1)?;
                    approved()?;
                    return Ok(());
                }
            }
            continue;
        }
        if theme.identity_needs_pause(columns, rows, ansi) || notice.is_some() {
            write!(output, "{}\r\n", theme.welcome())?;
            if let Some(notice) = notice {
                writeln!(output, "{notice}")?;
            }
            output.flush()?;
            termios::tcflush(&crate::tty()?, termios::QueueSelector::IFlush)?;
            line("Press Enter to continue: ", 1)?;
        } else {
            writeln!(output, "\n{}", theme.welcome())?;
            output.flush()?;
        }
        approved()?;
        return Ok(());
    }
}
/// None requests the canonical line-mode fallback. Drop restores terminal state
/// and flushes pending typeahead before editor/sysop prompts get a new context.
pub(super) enum Selection {
    Action(Action),
    Plain,
    Refresh,
}
pub(super) fn select(
    board: &mut crate::Visit<'_>,
    theme: &Theme,
    sysop: bool,
    caps: faction_terminal::capabilities::Capabilities,
) -> Result<Selection> {
    if !capable() {
        return Ok(Selection::Plain);
    }
    let Some(dimensions @ (40..=240, 18..=100)) = size() else {
        return Ok(Selection::Plain);
    };
    let mut raw = Raw::open()?;
    raw.sync = caps.synchronized;
    let (mut decoder, mut menu) = (
        Decoder::new(),
        Menu::with_columns(theme, sysop, dimensions.0),
    );
    let mut output = io::stdout();
    output.write_all(b"\x1b[?25l")?;
    output.write_all(caps.frame(&menu.render(), true).as_bytes())?;
    output.flush()?;
    let stdin = io::stdin();
    loop {
        board.tick()?;
        if size() != Some(dimensions) {
            return Ok(Selection::Plain);
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
            if key == faction_terminal::Key::Hotkey(b'?') {
                return Ok(Selection::Refresh);
            }
            let previous = menu.focus();
            if let Some(action) = menu.feed(key) {
                return Ok(Selection::Action(action));
            }
            output.write_all(caps.frame(&menu.redraw(previous), true).as_bytes())?;
            output.flush()?;
        }
    }
}

/// Dedicated raw-input state, ending in settings rather than a member action.
pub(super) fn probe_sync() -> Result<bool> {
    use faction_terminal::capabilities::{Probe, SYNC_QUERY};
    let _raw = Raw::open()?;
    print!("{SYNC_QUERY}");
    io::stdout().flush()?;
    let mut probe = Probe::default();
    probe.begin(Instant::now());
    while !probe.expired(Instant::now()) {
        let readable = ready()?;
        // Polling may wake after the deadline; never accept a late response.
        if probe.expired(Instant::now()) {
            break;
        }
        if readable {
            probe
                .feed(_raw.read_byte()?)
                .map_err(|s| io::Error::new(io::ErrorKind::InvalidData, s))?;
        }
    }
    termios::tcflush(&_raw.tty, termios::QueueSelector::IFlush)?;
    probe
        .result()
        .map_err(|s| io::Error::new(io::ErrorKind::InvalidData, s).into())
}

/// Explicit preview owns the raw TTY through negotiation and bounded streaming.
pub(super) fn graphics() -> Result<Option<Graphics>> {
    use faction_terminal::graphics::{self, Probe};
    let raw = Raw::open()?;
    let mut output = File::from(rustix::io::fcntl_dupfd_cloexec(io::stdout(), 3)?);
    graphics::write_tty(
        &mut output,
        graphics::QUERY.as_bytes(),
        Instant::now() + graphics::TRANSFER_TIMEOUT,
    )?;
    let mut probe = Probe::default();
    probe.begin(Instant::now());
    while !probe.expired(Instant::now()) {
        let readable = ready()?;
        if probe.expired(Instant::now()) {
            break;
        }
        if readable {
            probe.feed(raw.read_byte()?).map_err(io::Error::other)?;
        }
    }
    termios::tcflush(&raw.tty, termios::QueueSelector::IFlush)?;
    if !probe.result().map_err(io::Error::other)? {
        return Ok(None);
    }
    let guard = Graphics;
    graphics::write_tty(
        &mut output,
        graphics::DELETE.as_bytes(),
        Instant::now() + graphics::TRANSFER_TIMEOUT,
    )?;
    graphics::write_tty(
        &mut output,
        b"\x1b[2J\x1b[H",
        Instant::now() + graphics::TRANSFER_TIMEOUT,
    )?;
    let deadline = Instant::now() + graphics::TRANSFER_TIMEOUT;
    for index in 0..graphics::CHUNKS {
        graphics::write_tty(
            &mut output,
            graphics::chunk(index)
                .ok_or_else(|| io::Error::other("graphics chunk"))?
                .as_bytes(),
            deadline,
        )?;
    }
    graphics::write_tty(
        &mut output,
        b"\r\nKitty preview (original calibration artwork).\r\n",
        deadline,
    )?;
    Ok(Some(guard))
}
pub(super) struct Graphics;
impl Drop for Graphics {
    fn drop(&mut self) {
        if let Ok(fd) = rustix::io::fcntl_dupfd_cloexec(io::stdout(), 3) {
            faction_terminal::graphics::cleanup(&mut File::from(fd));
        }
    }
}
