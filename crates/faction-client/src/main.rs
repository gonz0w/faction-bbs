//! Local terminal companion. Remote output never selects local paths or endpoints.
#![forbid(unsafe_code)]
mod mail_data;
mod mail_ui;
mod mail_vault;
mod profile;
mod sftp;
mod stream;
mod transport;
use faction_files::invalid;
use faction_terminal::{Decoder, Key};
use profile::Profile;
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    termios::{self, OptionalActions, Termios},
};
use std::{
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use transport::Remote;
struct Terminal {
    input: File,
    output: File,
    saved: Termios,
}
impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = termios::tcsetattr(&self.input, OptionalActions::Now, &self.saved);
        let _ = self.output.write_all(b"\x1b[0m\x1b[?25h\r\n");
    }
}
fn safe(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_ascii_graphic() || c == ' ' {
                vec![c]
            } else {
                c.escape_unicode().collect()
            }
        })
        .collect()
}
impl Terminal {
    fn start() -> io::Result<Self> {
        let input = File::from(rustix::io::fcntl_dupfd_cloexec(io::stdin(), 3)?);
        let output = File::from(rustix::io::fcntl_dupfd_cloexec(io::stdout(), 3)?);
        let saved = termios::tcgetattr(&input)?;
        termios::tcgetattr(&output)?;
        let mut raw = saved.clone();
        raw.make_raw();
        termios::tcsetattr(&input, OptionalActions::Now, &raw)?;
        Ok(Self {
            input,
            output,
            saved,
        })
    }
    fn print(&mut self, s: &str) -> io::Result<()> {
        self.output.write_all(s.as_bytes())
    }
    fn ready(&self, timeout_ms: i64) -> io::Result<bool> {
        let mut p = [PollFd::new(&self.input, PollFlags::IN)];
        let ts = Timespec {
            tv_sec: timeout_ms / 1000,
            tv_nsec: (timeout_ms % 1000) * 1000000,
        };
        Ok(poll(&mut p, Some(&ts))? > 0 && p[0].revents().contains(PollFlags::IN))
    }
    fn byte(&mut self) -> io::Result<u8> {
        let mut b = [0];
        self.input.read_exact(&mut b)?;
        Ok(b[0])
    }
    fn line(&mut self, label: &str, cap: usize) -> io::Result<String> {
        self.print(label)?;
        let mut bytes = Vec::new();
        loop {
            if !self.ready(100)? {
                continue;
            }
            match self.byte()? {
                3 | 4 => return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelled")),
                b'\r' | b'\n' => {
                    self.print("\r\n")?;
                    return String::from_utf8(bytes).map_err(|_| invalid());
                }
                8 | 127 => {
                    if !bytes.is_empty() {
                        bytes.pop();
                        while !bytes.is_empty() && std::str::from_utf8(&bytes).is_err() {
                            bytes.pop();
                        }
                        self.print("\x08 \x08")?;
                    }
                }
                b if (32..=126).contains(&b) && bytes.len() < cap => {
                    bytes.push(b);
                    self.output.write_all(&[b])?;
                }
                _ => (),
            }
        }
    }
    fn choice(&mut self) -> io::Result<u8> {
        const ENTRIES: [(u8, &str); 6] = [
            (b'A', "File areas"),
            (b'L', "List area / next page"),
            (b'U', "Upload local file"),
            (b'D', "Download file / link"),
            (b'I', "Inspect file link"),
            (b'B', "Back to the BBS"),
        ];
        let mut focus = 0;
        let mut decoder = Decoder::new();
        loop {
            self.print("\x1b[2J\x1b[H\x1b[1;36mFaction // Files on your device\x1b[0m\r\nArrows / Enter or hotkeys. Files and keys stay under your control.\r\n\r\n")?;
            for (i, (key, label)) in ENTRIES.iter().enumerate() {
                self.print(&format!(
                    "{} [{}] {}\r\n",
                    if i == focus { ">" } else { " " },
                    *key as char,
                    label
                ))?;
            }
            loop {
                if !self.ready(100)? {
                    decoder.expire(Instant::now());
                    continue;
                }
                if let Some(k) = decoder.feed(self.byte()?, Instant::now()) {
                    match k {
                        Key::Enter => return Ok(ENTRIES[focus].0),
                        Key::Hotkey(b)
                            if ENTRIES.iter().any(|(k, _)| *k == b.to_ascii_uppercase()) =>
                        {
                            return Ok(b.to_ascii_uppercase());
                        }
                        Key::Up => {
                            focus = (focus + ENTRIES.len() - 1) % ENTRIES.len();
                            break;
                        }
                        Key::Down => {
                            focus = (focus + 1) % ENTRIES.len();
                            break;
                        }
                        Key::Cancel => return Ok(b'B'),
                        _ => (),
                    }
                }
            }
        }
    }
}
fn reference(remote: &Remote, s: &str) -> io::Result<String> {
    if faction_files::id(s) {
        return Ok(format!("faction-file:{}/{s}", remote.namespace));
    }
    let prefix = format!("faction-file:{}/", remote.namespace);
    if let Some(id) = s.strip_prefix(&prefix)
        && faction_files::id(id)
    {
        return Ok(s.into());
    }
    Err(invalid())
}
fn transfer(t: &mut Terminal, r: &mut Remote, p: &Profile, sending: bool) -> io::Result<()> {
    let (row, path, source) = if sending {
        let local = PathBuf::from(t.line("Local file to upload: ", 512)?);
        let file = sftp::source(&local)?;
        let size = file.metadata()?.len().to_string();
        let area = t.line("Area ID: ", 10)?;
        let default = local
            .file_name()
            .and_then(|s| s.to_str())
            .filter(|s| sftp::filename(s))
            .unwrap_or("");
        let name = t.line(&format!("Remote filename [{}]: ", safe(default)), 64)?;
        let name = if name.is_empty() {
            default.to_owned()
        } else {
            name
        };
        if !sftp::filename(&name) {
            return Err(invalid());
        }
        let description = t.line("Description (max 256 bytes): ", 256)?;
        if !t
            .line(
                &format!("Upload {} bytes as {}? [y/N]: ", size, safe(&name)),
                1,
            )?
            .eq_ignore_ascii_case("y")
        {
            return Err(io::ErrorKind::Interrupted.into());
        }
        let rows = r.rpc(&["upload", &area, &name, &size, &description])?;
        if rows.len() != 1
            || rows[0].get(1).map(String::as_str) != Some("upload")
            || rows[0].get(2) != Some(&name)
            || rows[0].get(3) != Some(&size)
        {
            return Err(invalid());
        }
        (
            rows.into_iter().next().ok_or_else(invalid)?,
            local,
            Some(file),
        )
    } else {
        for (i, link) in r.recent.items.iter().enumerate() {
            t.print(&format!("#{} {}\r\n", i + 1, link))?;
        }
        let item = t.line("File ID, faction-file link or recent #number: ", 128)?;
        let item = if let Some(n) = item.strip_prefix('#') {
            r.recent
                .items
                .get(
                    n.parse::<usize>()
                        .ok()
                        .and_then(|n| n.checked_sub(1))
                        .ok_or_else(invalid)?,
                )
                .ok_or_else(invalid)?
                .clone()
        } else {
            item
        };
        let item = reference(r, &item)?;
        let metadata = r.rpc(&["info", &item])?;
        if metadata.len() != 1 || metadata[0].len() != 5 || metadata[0][0] != "info" {
            return Err(invalid());
        }
        t.print(&format!(
            "{} // {} bytes // SHA256 {}\r\n",
            safe(&metadata[0][2]),
            safe(&metadata[0][3]),
            safe(&metadata[0][4])
        ))?;
        let path = PathBuf::from(t.line("Save as (new local filename): ", 512)?);
        if !t
            .line("Download to this local path? [y/N]: ", 1)?
            .eq_ignore_ascii_case("y")
        {
            return Err(io::ErrorKind::Interrupted.into());
        }
        let rows = r.rpc(&["download", &item])?;
        if rows.len() != 1 {
            return Err(invalid());
        }
        let row = rows.into_iter().next().ok_or_else(invalid)?;
        if row.get(2) != metadata[0].get(2)
            || row.get(3) != metadata[0].get(3)
            || row.get(4) != metadata[0].get(4)
        {
            return Err(invalid());
        }
        (row, path, None)
    };
    // Validate before either streaming or cancellation indexes the grant. A
    // malformed remote response must never panic or become local authority.
    if row.len() != 6
        || row[0] != "grant"
        || row[1] != if sending { "upload" } else { "download" }
        || !faction_files::id(&row[5])
    {
        return Err(invalid());
    }
    let result = sftp::transfer(p, &row, &path, source, |done, total| {
        if t.ready(0)? {
            let b = t.byte()?;
            if b == 3 || b == 4 {
                return Err(io::ErrorKind::Interrupted.into());
            }
        }
        t.print(&format!(
            "\rTransferring: {done}/{total} bytes (Ctrl-C cancels)"
        ))
    });
    let _ = termios::tcflush(&t.input, termios::QueueSelector::IFlush);
    if result.is_err() && r.native {
        let _ = r.rpc(&["cancel", &row[5]]);
    }
    let digest = result?;
    t.print(&format!("\r\nTransfer complete. SHA256 {digest}\r\n"))?;
    if sending {
        t.print("Upload awaits sysop review; it is not published yet.\r\n")?;
    }
    Ok(())
}
fn file_menu(t: &mut Terminal, r: &mut Remote, p: &Profile) -> io::Result<()> {
    let mut after = String::new();
    let mut last_area = String::new();
    while r.native {
        let choice = t.choice()?;
        if choice == b'B' {
            r.rpc(&["back"])?;
            break;
        }
        t.print("\r\n")?;
        let result: io::Result<()> = (|| {
            match choice {
                b'A' => {
                    let cursor = t.line("After area ID [0]: ", 10)?;
                    let cursor = if cursor.is_empty() { "0" } else { &cursor };
                    for a in r.rpc(&["areas", cursor])? {
                        if a.len() != 6 || a[0] != "area" {
                            return Err(invalid());
                        }
                        t.print(&format!(
                            "{} // {} // {} // private={} // quota={}\r\n",
                            safe(&a[1]),
                            safe(&a[2]),
                            safe(&a[3]),
                            safe(&a[4]),
                            safe(&a[5])
                        ))?;
                    }
                }
                b'L' => {
                    let area = t.line(&format!("Area ID [{}]: ", safe(&last_area)), 10)?;
                    let area = if area.is_empty() {
                        last_area.clone()
                    } else {
                        area
                    };
                    let cursor = t.line(
                        &format!("After object ID [{}] (- starts over): ", safe(&after)),
                        32,
                    )?;
                    let cursor = if cursor == "-" || area != last_area {
                        String::new()
                    } else if cursor.is_empty() {
                        after.clone()
                    } else {
                        cursor
                    };
                    let rows = r.rpc(&["list", &area, &cursor])?;
                    last_area = area;
                    after.clear();
                    for f in rows {
                        if f.len() != 7 || f[0] != "file" || !faction_files::id(&f[1]) {
                            return Err(invalid());
                        }
                        t.print(&format!(
                            "{} // {} // {} bytes // by {}\r\n  {}\r\n  faction-file:{}/{}\r\n",
                            safe(&f[1]),
                            safe(&f[2]),
                            safe(&f[3]),
                            safe(&f[5]),
                            safe(&f[6]),
                            r.namespace,
                            f[1]
                        ))?;
                        r.recent
                            .feed(format!("faction-file:{}/{}", r.namespace, f[1]).as_bytes());
                        after = f[1].clone();
                    }
                }
                b'U' => transfer(t, r, p, true)?,
                b'D' => transfer(t, r, p, false)?,
                b'I' => {
                    let item = t.line("File ID or faction-file link: ", 128)?;
                    for f in r.rpc(&["info", &reference(r, &item)?])? {
                        t.print(&format!(
                            "{}\r\n",
                            f.iter().map(|s| safe(s)).collect::<Vec<_>>().join(" // ")
                        ))?;
                    }
                }
                _ => return Err(invalid()),
            }
            Ok(())
        })();
        if let Err(e) = result {
            if !r.native {
                return Err(e);
            }
            t.print(&format!(
                "\r\nOperation stopped: {}\r\n",
                safe(&e.to_string())
            ))?;
        }
        t.line("Enter to return to Files: ", 0)?;
    }
    Ok(())
}
fn run() -> io::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() == 3 && args[0] == "--mail-store" {
        return mail_ui::run(Path::new(&args[1]), &args[2]);
    }
    if args.len() != 2 || args[0] != "--profile" {
        return Err(io::Error::other(
            "Usage: faction-client --profile PRIVATE_TOML | --mail-store NEW_OR_EXISTING_PRIVATE_DIR init|open|recover",
        ));
    }
    let p = Profile::load(Path::new(&args[1]))?;
    let mut t = Terminal::start()?;
    let mut size = termios::tcgetwinsize(&t.input)?;
    if !(18..=100).contains(&size.ws_row) || !(40..=240).contains(&size.ws_col) {
        return Err(io::Error::other("use a terminal from 40x18 to 240x100"));
    }
    let mut r = Remote::start(&p, size.ws_row, size.ws_col)?;
    let mut handshake = None;
    loop {
        if r.native {
            file_menu(&mut t, &mut r, &p)?;
        }
        if !r.output.is_empty() {
            t.output.write_all(&std::mem::take(&mut r.output))?;
        }
        let mut fds = [
            PollFd::new(&r.pty, PollFlags::IN),
            PollFd::new(&t.input, PollFlags::IN),
        ];
        let ts = Timespec {
            tv_sec: 0,
            tv_nsec: 100_000_000,
        };
        poll(&mut fds, Some(&ts))?;
        let peer = fds[0].revents();
        let keyboard = fds[1].revents();
        if peer.intersects(PollFlags::IN | PollFlags::HUP | PollFlags::ERR) {
            match r.pump() {
                Ok(()) => (),
                Err(e) => {
                    // PTY EOF can arrive just before waitpid observes SSH exit.
                    let until = Instant::now() + Duration::from_secs(1);
                    let status = loop {
                        if let Some(status) = r.child.try_wait()? {
                            break Some(status);
                        }
                        if Instant::now() >= until {
                            break None;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    };
                    if status.is_some_and(|s| s.success()) {
                        break;
                    }
                    return Err(e);
                }
            }
            for f in std::mem::take(&mut r.frames) {
                if f.as_slice() == ["ready"] && handshake.is_none() {
                    let mut random = [0; 16];
                    getrandom::fill(&mut random).map_err(|_| invalid())?;
                    r.nonce = random.iter().map(|b| format!("{b:02x}")).collect();
                    r.sequence = 0;
                    r.send(format!("native-v1 {} {}\r", p.public_key, r.nonce).as_bytes())?;
                    handshake = Some(Instant::now());
                } else if f.len() == 3
                    && f[0] == "hello"
                    && f[1] == r.nonce
                    && faction_files::id(&f[2])
                    && handshake.is_some()
                {
                    r.namespace = f[2].clone();
                    r.native = true;
                    r.output.clear();
                    handshake = None;
                } else {
                    return Err(invalid());
                }
            }
        }
        if handshake.is_some_and(|s: Instant| s.elapsed() > Duration::from_secs(5)) {
            return Err(io::Error::other("native Files handshake timed out"));
        }
        if keyboard.contains(PollFlags::IN) && !r.native && handshake.is_none() {
            let mut b = [0; 64];
            let n = t.input.read(&mut b)?;
            if n == 0 {
                break;
            }
            r.send(&b[..n])?;
        }
        let next = termios::tcgetwinsize(&t.input)?;
        if (next.ws_row, next.ws_col) != (size.ws_row, size.ws_col) {
            size = next;
            termios::tcsetwinsize(&r.pty, size)?;
            let pid = rustix::process::Pid::from_raw(r.child.id() as i32).ok_or_else(invalid)?;
            rustix::process::kill_process(pid, rustix::process::Signal::WINCH)?;
        }
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Faction client: {e}");
        std::process::exit(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn client_pty_preserves_geometry_and_owns_cloexec_descriptors() {
        use rustix::io::{FdFlags, fcntl_getfd};
        let (master, slave) = faction_client_pty::open(24, 80).unwrap();
        assert_eq!(termios::tcgetwinsize(&slave).unwrap().ws_col, 80);
        assert_eq!(termios::tcgetwinsize(&slave).unwrap().ws_row, 24);
        assert!(fcntl_getfd(&master).unwrap().contains(FdFlags::CLOEXEC));
        assert!(fcntl_getfd(&slave).unwrap().contains(FdFlags::CLOEXEC));
        assert!(faction_client_pty::open(0, 80).is_err());
    }
}
