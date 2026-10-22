//! Trusted local entry point. No SSH process, remote stream or key-unlock RPC.
use crate::mail_data::invalid;
use crate::{
    Terminal,
    mail_data::{Data, hex, unhex},
    mail_vault::{Store, Vault},
    safe,
};
use faction_terminal::{Decoder, Key};
use std::{
    io,
    path::Path,
    time::{Duration, Instant},
};
use zeroize::Zeroizing;
const IDLE: Duration = Duration::from_secs(300);
struct Screen<'a> {
    term: &'a mut Terminal,
    activity: Instant,
}
impl Screen<'_> {
    fn print(&mut self, s: &str) -> io::Result<()> {
        self.term.print(s)
    }
    fn byte(&mut self) -> io::Result<u8> {
        loop {
            if self.activity.elapsed() >= IDLE {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "local mail locked after inactivity",
                ));
            }
            if self.term.ready(100)? {
                let b = self.term.byte()?;
                self.activity = Instant::now();
                if b == 3 || b == 4 {
                    return Err(io::Error::new(
                        io::ErrorKind::Interrupted,
                        "local mail cancelled",
                    ));
                }
                return Ok(b);
            }
        }
    }
    fn line(&mut self, label: &str, cap: usize, secret: bool) -> io::Result<Zeroizing<String>> {
        // Discard typeahead before secrets: never reuse a pasted BBS command as
        // a local unlock secret. Secrets have no echo, mask, argv or env path.
        if secret {
            rustix::termios::tcflush(&self.term.input, rustix::termios::QueueSelector::IFlush)?;
        }
        self.print(label)?;
        let mut value = Zeroizing::new(String::new());
        loop {
            match self.byte()? {
                b'\r' | b'\n' => {
                    self.print("\r\n")?;
                    return Ok(value);
                }
                8 | 127 => {
                    if value.pop().is_some() && !secret {
                        self.print("\x08 \x08")?;
                    }
                }
                b if (32..=126).contains(&b) => {
                    if value.len() == cap {
                        return Err(invalid());
                    }
                    value.push(b as char);
                    if !secret {
                        self.print(&(b as char).to_string())?;
                    }
                }
                _ => (),
            }
        }
    }
    fn password(&mut self) -> io::Result<Zeroizing<String>> {
        let a = self.line(
            "New LOCAL passphrase (12-128 characters, hidden): ",
            128,
            true,
        )?;
        let b = self.line("Repeat LOCAL passphrase (hidden): ", 128, true)?;
        if a.len() < 12 || *a != *b {
            return Err(io::Error::other("passphrases differ or are too short"));
        }
        Ok(a)
    }
    fn recovery(&mut self, key: &[u8]) -> io::Result<()> {
        let shown = Zeroizing::new(hex(key));
        self.print("\x1b[2J\x1b[H[LOCAL] Recovery secret - keep it OFFLINE with an encrypted backup.\r\nAnyone holding this secret and a backup can open that backup.\r\n\r\n")?;
        self.print(&shown)?;
        self.print(
            "\r\n\r\nNo sysop/password reset can recover it. Old backups keep old secrets.\r\n",
        )?;
        let _ = self.line(
            "Save it privately, then Enter to clear this screen: ",
            0,
            false,
        )?;
        self.print("\x1b[2J\x1b[H")
    }
    fn choice(&mut self, home: &str, drafts: usize, pins: usize) -> io::Result<u8> {
        const ITEMS: [(u8, &str); 7] = [
            (b'D', "Draft a message (LOCAL ONLY)"),
            (b'V', "View saved drafts"),
            (b'X', "Delete a draft"),
            (b'T', "Verify / pin a device"),
            (b'R', "Remove a device pin"),
            (b'B', "Encrypted backup"),
            (b'L', "Lock and exit"),
        ];
        let mut focus = 0;
        let mut decoder = Decoder::new();
        loop {
            self.print(&format!("\x1b[2J\x1b[H\x1b[1;36mFaction // Mail on YOUR device\x1b[0m\r\n[LOCAL / OFFLINE] Home: {} (claimed, not authenticated)\r\nEncrypted local drafts: {drafts}/32 | Device pins: {pins}/64\r\nInbox / send: unavailable until the native OMEMO adapter is verified.\r\nNo BBS connection is opened here. Arrows / Enter or hotkeys.\r\n\r\n",safe(home)))?;
            for (i, (key, label)) in ITEMS.iter().enumerate() {
                self.print(&format!(
                    "{} [{}] {}\r\n",
                    if i == focus { ">" } else { " " },
                    *key as char,
                    label
                ))?;
            }
            loop {
                let b = self.byte()?;
                if let Some(key) = decoder.feed(b, Instant::now()) {
                    match key {
                        Key::Up => {
                            focus = (focus + ITEMS.len() - 1) % ITEMS.len();
                            break;
                        }
                        Key::Down => {
                            focus = (focus + 1) % ITEMS.len();
                            break;
                        }
                        Key::Enter => return Ok(ITEMS[focus].0),
                        Key::Hotkey(c) => {
                            let c = c.to_ascii_uppercase();
                            if ITEMS.iter().any(|i| i.0 == c) {
                                return Ok(c);
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    fn pause(&mut self) -> io::Result<()> {
        self.line("Enter to return: ", 0, false).map(|_| ())
    }
    fn action(&mut self, v: &mut Vault, key: u8) -> io::Result<()> {
        self.print("\x1b[2J\x1b[H[LOCAL / OFFLINE] Faction mail\r\n\r\n")?;
        match key {
            b'D' => {
                let to = self.line("To (full user@home): ", 254, false)?;
                let subject = self.line("Subject: ", 128, false)?;
                let body = self.line("Body (one line, up to 4096 bytes): ", 4096, false)?;
                v.data.draft(to.to_string(), subject, body)?;
                v.save()?;
                self.print("Saved encrypted on this device. NOT sent or queued for delivery.\r\n")?;
            }
            b'V' => {
                for (i, d) in v.data.drafts.iter().enumerate() {
                    self.print(&format!("{} -> {}\r\n", i + 1, safe(&d.to)))?;
                    self.print(&d.subject)?;
                    self.print("\r\n")?;
                    self.print(&d.body)?;
                    self.print("\r\n\r\n")?;
                    // Display one bounded entry at a time, so 32 drafts do not
                    // silently scroll private content into terminal history.
                    self.pause()?;
                    self.print("\x1b[2J\x1b[H")?;
                }
            }
            b'X' => {
                let i = self
                    .line("Draft number to delete: ", 2, false)?
                    .parse::<usize>()
                    .map_err(|_| invalid())?;
                if i == 0 || i > v.data.drafts.len() {
                    return Err(invalid());
                }
                v.data.drafts.remove(i - 1);
                v.save()?;
                self.print("Deleted from the current vault. Backups may retain it.\r\n")?;
            }
            b'T' => {
                self.print("Compare the full OMEMO-2 Curve25519-format fingerprint with the\r\nmember using an independent trusted channel. A server list is NOT proof.\r\nPins are stored locally; no network enrollment or trust export occurs.\r\n")?;
                let who = self.line("Member (full user@home): ", 254, false)?;
                let id = self
                    .line("Device ID (1-2147483647): ", 10, false)?
                    .parse::<u32>()
                    .map_err(|_| invalid())?;
                let fp = self.line("Verified fingerprint (64 lowercase hex): ", 64, false)?;
                let fingerprint = unhex(&fp)?;
                self.print(&format!(
                    "Pin {} / device {} / {}\r\n",
                    safe(&who),
                    id,
                    fp.as_str()
                ))?;
                if self
                    .line("After independent comparison, type verified: ", 8, false)?
                    .as_str()
                    != "verified"
                {
                    return Err(invalid());
                }
                v.data.pin(who.to_string(), id, fingerprint)?;
                v.save()?;
            }
            b'R' => {
                for (i, p) in v.data.pins.iter().enumerate() {
                    self.print(&format!(
                        "{} {} / {} / {}\r\n",
                        i + 1,
                        safe(&p.address),
                        p.device,
                        hex(&p.fingerprint)
                    ))?;
                }
                let i = self
                    .line("Pin number to remove locally: ", 2, false)?
                    .parse::<usize>()
                    .map_err(|_| invalid())?;
                if i == 0 || i > v.data.pins.len() {
                    return Err(invalid());
                }
                v.data.pins.remove(i - 1);
                v.save()?;
                self.print(
                    "Removed locally. This does not revoke a remote device or old copies.\r\n",
                )?;
            }
            b'B' => {
                let path = self.line(
                    "NEW absolute backup directory (parent must exist): ",
                    1024,
                    false,
                )?;
                v.backup(Path::new(path.as_str()))?;
                self.print("Encrypted backup created. Test opening it before relying on it.\r\n")?;
            }
            _ => return Err(invalid()),
        }
        self.pause()
    }
}
pub fn run(path: &Path, action: &str) -> io::Result<()> {
    if !matches!(action, "init" | "open" | "recover") {
        return Err(invalid());
    }
    rustix::process::setrlimit(
        rustix::process::Resource::Core,
        rustix::process::Rlimit {
            current: Some(0),
            maximum: Some(0),
        },
    )?;
    let mut t = Terminal::start()?;
    let result = (|| {
        let mut s = Screen {
            term: &mut t,
            activity: Instant::now(),
        };
        s.print("\x1b[2J\x1b[H[LOCAL / OFFLINE] Faction encrypted mail vault\r\nSecrets stay on this device. Network mail is not implemented yet.\r\n")?;
        let mut v = match action {
            "init" => {
                let home = s.line("Your full home identity (local claim only): ", 254, false)?;
                let data = Data::new(home.to_string())?;
                let password = s.password()?;
                let (v, recovery) =
                    Vault::create(Store::at(path, true)?, data, password.as_bytes())?;
                drop(password);
                s.recovery(&*recovery)?;
                v
            }
            "recover" => {
                let store = Store::at(path, false)?;
                let typed = s.line("Offline recovery secret (64 hex, hidden): ", 64, true)?;
                let recovery = Zeroizing::new(unhex(&typed)?);
                drop(typed);
                let mut v = Vault::unlock(store, &*recovery, true)?;
                drop(recovery);
                let password = s.password()?;
                let next = v.reset(password.as_bytes())?;
                drop(password);
                s.recovery(&*next)?;
                v
            }
            _ => {
                let store = Store::at(path, false)?;
                let password =
                    s.line("LOCAL passphrase (hidden; not a BBS password): ", 128, true)?;
                let v = Vault::unlock(store, password.as_bytes(), false)?;
                drop(password);
                v
            }
        };
        loop {
            let key = s.choice(&v.data.home, v.data.drafts.len(), v.data.pins.len())?;
            if key == b'L' {
                break;
            }
            // Any write/validation error exits and relocks: never continue with
            // unsaved in-memory trust/draft changes after a failed commit.
            s.action(&mut v, key)?;
        }
        Ok(())
    })();
    let _ = t.print("\x1b[2J\x1b[H"); // best effort, not a terminal scrollback erasure claim
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inactivity_expires_without_input_or_a_remote_connection() {
        let (_master, slave) = faction_client_pty::open(24, 80).unwrap();
        let saved = rustix::termios::tcgetattr(&slave).unwrap();
        let mut t = Terminal {
            input: slave.try_clone().unwrap().into(),
            output: slave.into(),
            saved,
        };
        let mut screen = Screen {
            term: &mut t,
            activity: Instant::now() - IDLE,
        };
        assert_eq!(screen.byte().unwrap_err().kind(), io::ErrorKind::TimedOut);
    }
}
