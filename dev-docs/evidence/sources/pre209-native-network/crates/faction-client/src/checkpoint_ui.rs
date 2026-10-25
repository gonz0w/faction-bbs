//! Deliberate member-device terminal entry. Never called by a remote stream,
//! profile, environment variable, picker, or transport control frame.

use faction_client::{
    mail_checkpoint::{CheckpointVault, Direction, MAX_HISTORY, MAX_OUTBOX},
    mail_data::{Data, hex, invalid, unhex},
    mail_vault::{Store, Vault},
};
use faction_terminal::{Decoder, Key};
use std::{
    io,
    path::Path,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use zeroize::Zeroizing;
const IDLE: Duration = Duration::from_secs(300);

// Default output is classic ASCII. Unicode is an explicit local capability
// override, never inferred from TERM/remote replies. Escape controls, bidi marks,
// invisible format characters and line separators even with the override.
fn inert(s: &str, unicode: bool) -> Zeroizing<String> {
    let mut out = Zeroizing::new(String::new());
    for c in s.chars() {
        if c == '\n' {
            out.extend(c.escape_unicode());
        } else {
            out.push_str(&Zeroizing::new(faction_terminal::text::scalar(c, unicode)));
        }
    }
    out
}
fn now() -> io::Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| invalid())
}
// A plain terminal guard keeps the existing v1/SSH terminal behavior unchanged.
struct LocalTerminal {
    input: std::fs::File,
    output: std::fs::File,
    saved: rustix::termios::Termios,
}
impl LocalTerminal {
    fn start() -> io::Result<Self> {
        let input = std::fs::File::from(rustix::io::fcntl_dupfd_cloexec(io::stdin(), 3)?);
        let output = std::fs::File::from(rustix::io::fcntl_dupfd_cloexec(io::stdout(), 3)?);
        let saved = rustix::termios::tcgetattr(&input)?;
        rustix::termios::tcgetattr(&output)?;
        let mut raw = saved.clone();
        raw.make_raw();
        rustix::termios::tcsetattr(&input, rustix::termios::OptionalActions::Now, &raw)?;
        Ok(Self {
            input,
            output,
            saved,
        })
    }
    fn print(&mut self, text: &str) -> io::Result<()> {
        std::io::Write::write_all(&mut self.output, text.as_bytes())
    }
    fn ready(&self, ms: i64) -> io::Result<bool> {
        use rustix::event::{PollFd, PollFlags, Timespec, poll};
        let mut fds = [PollFd::new(&self.input, PollFlags::IN)];
        Ok(poll(
            &mut fds,
            Some(&Timespec {
                tv_sec: ms / 1000,
                tv_nsec: (ms % 1000) * 1_000_000,
            }),
        )? > 0
            && fds[0].revents().contains(PollFlags::IN))
    }
    fn byte(&mut self) -> io::Result<u8> {
        let mut b = [0];
        std::io::Read::read_exact(&mut self.input, &mut b)?;
        Ok(b[0])
    }
}
impl Drop for LocalTerminal {
    fn drop(&mut self) {
        let _ = rustix::termios::tcflush(&self.input, rustix::termios::QueueSelector::IFlush);
        let _ = rustix::termios::tcsetattr(
            &self.input,
            rustix::termios::OptionalActions::Now,
            &self.saved,
        );
        let _ = self.print("\r\n");
    }
}
struct Screen<'a> {
    term: &'a mut LocalTerminal,
    activity: Instant,
    unicode: bool,
}
impl Screen<'_> {
    fn print(&mut self, s: &str) -> io::Result<()> {
        self.term.print(s)
    }
    fn show(&mut self, label: &str, s: &str) -> io::Result<()> {
        self.print(label)?;
        self.print(&inert(s, self.unicode))?;
        self.print("\r\n")
    }
    fn clear(&mut self) -> io::Result<()> {
        // Fixed local clear only; no private data becomes a terminal instruction.
        // Plain terminals receive a separator; neither erases scrollback.
        self.print("\r\n----------------------------------------------------------------\r\n")
    }
    fn byte(&mut self) -> io::Result<u8> {
        loop {
            if self.activity.elapsed() >= IDLE {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "inactivity lock"));
            }
            if self.term.ready(100)? {
                let b = self.term.byte()?;
                self.activity = Instant::now();
                if matches!(b, 3 | 4) {
                    return Err(io::Error::new(
                        io::ErrorKind::Interrupted,
                        "local cancellation",
                    ));
                }
                return Ok(b);
            }
        }
    }
    fn line(&mut self, label: &str, cap: usize, secret: bool) -> io::Result<Zeroizing<String>> {
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
                        return Err(io::Error::other("input exceeds local limit"));
                    }
                    value.push(b as char);
                    if !secret {
                        self.print(&(b as char).to_string())?;
                    }
                }
                _ => {
                    return Err(io::Error::other(
                        "only printable ASCII is accepted for local input",
                    ));
                }
            }
        }
    }
    fn confirm(&mut self, label: &str, expected: &str) -> io::Result<()> {
        rustix::termios::tcflush(&self.term.input, rustix::termios::QueueSelector::IFlush)?;
        if self.line(label, expected.len().max(32), false)?.as_str() != expected {
            return Err(io::Error::other("confirmation cancelled"));
        }
        Ok(())
    }
    fn password(&mut self) -> io::Result<Zeroizing<String>> {
        let a = self.line(
            "New LOCAL passphrase (12-128 ASCII characters, hidden): ",
            128,
            true,
        )?;
        let b = self.line("Repeat LOCAL passphrase (hidden): ", 128, true)?;
        if a.len() < 12 || *a != *b {
            return Err(io::Error::other("passphrases differ or are too short"));
        }
        Ok(a)
    }
    fn device(&mut self, label: &str) -> io::Result<u32> {
        let id = self
            .line(label, 10, false)?
            .parse::<u32>()
            .map_err(|_| invalid())?;
        if !(1..=i32::MAX as u32).contains(&id) {
            return Err(invalid());
        }
        Ok(id)
    }
    fn recovery(&mut self, key: &[u8]) -> io::Result<()> {
        self.clear()?;
        self.print("NEW LOCAL recovery secret. Save OFFLINE with an encrypted backup.\r\nAnyone holding this secret and a copy can open that copy.\r\n\r\n")?;
        self.print(&Zeroizing::new(hex(key)))?;
        self.print(
            "\r\n\r\nOld copies keep their old secrets. No sysop/password-reset recovery.\r\n",
        )?;
        self.print("This hides the visible screen only. Terminal scrollback is NOT erased.\r\n")?;
        rustix::termios::tcflush(&self.term.input, rustix::termios::QueueSelector::IFlush)?;
        self.line("Save it privately, then Enter to clear: ", 0, false)?;
        // Best-effort plain viewport clear. Terminal scrollback is not erased.
        self.print(&"\r\n".repeat(32))
    }
    fn pause(&mut self) -> io::Result<()> {
        self.line("Enter to return: ", 0, false).map(|_| ())
    }
    fn menu(&mut self, v: &CheckpointVault) -> io::Result<u8> {
        let state = v.inspection()?;
        self.clear()?;
        self.print("[LOCAL / OFFLINE] Native mail checkpoints\r\nNo network inbox or send. No delivery/read/exactly-once claim.\r\n")?;
        self.show("Home (claimed, not authenticated): ", &v.legacy()?.home)?;
        let current = now()?;
        if state.history_only {
            self.print("HISTORY ONLY: backup/recovery cannot resume old ratchets.\r\n")?;
        }
        if current < state.created || current > state.expires {
            self.print("EXPIRED / clock outside session lifetime: no protocol retry.\r\n")?;
        }
        if state.peer.as_ref().is_some_and(|p| p.revoked) {
            self.print("REVOKED: peer blocked locally; no protocol retry.\r\n")?;
        }
        if v.history()?.len() == MAX_HISTORY {
            self.print("HISTORY FULL: new messages blocked; no silent eviction.\r\n")?;
        }
        if v.inspect_outbox()?.len() == MAX_OUTBOX {
            self.print(
                "OUTBOX FULL: new pending messages blocked; explicit retirement required.\r\n",
            )?;
        }
        if state.history_only
            || current < state.created
            || current > state.expires
            || state.peer.as_ref().is_some_and(|p| p.revoked)
        {
            self.print("Retire pending ciphertext, then create a fresh device and verify the peer again.\r\n")?;
        }
        self.print("[P] Public device / peer   [V] Verify peer identity\r\n[H] Authenticated history [O] Durable outbox\r\n[T] Retire ciphertext     [R] Revoke peer locally\r\n[F] Fresh device         [B] History-only encrypted backup\r\n[S] Reset local secrets  [D] Preserved legacy drafts / Curve pins\r\n[L] Lock and exit\r\n")?;
        let mut decoder = Decoder::new();
        loop {
            if let Some(Key::Hotkey(c)) = decoder.feed(self.byte()?, Instant::now()) {
                let c = c.to_ascii_uppercase();
                if b"PVHOTRFBSDL".contains(&c) && c != b' ' {
                    return Ok(c);
                }
            }
        }
    }
    fn action(&mut self, v: &mut CheckpointVault, key: u8) -> io::Result<()> {
        self.clear()?;
        match key {
            b'P' => {
                let state = v.inspection()?;
                self.print(&format!(
                    "Local public device: {}\r\ned25519:{}\r\n",
                    state.local_device,
                    hex(state.identity)
                ))?;
                self.print(&format!(
                    "Session UTC seconds: {}..{}; protocol started: {}\r\n",
                    state.created, state.expires, state.started
                ))?;
                if let Some(p) = state.peer {
                    self.print(&format!(
                        "Peer: {} / {} / ed25519:{}{}\r\n",
                        p.address,
                        p.device,
                        hex(p.identity),
                        if p.revoked { " / REVOKED" } else { "" }
                    ))?;
                } else {
                    self.print(
                        "No native peer enrolled. Legacy Curve pins confer no native trust.\r\n",
                    )?;
                }
            }
            b'V' => {
                let state = v.inspection()?;
                if state.peer.is_some() {
                    self.print("Peer already enrolled; no changed-device acceptance. Revoke/retire and use a fresh device.\r\n")?;
                } else if state.history_only || now()? < state.created || now()? > state.expires {
                    self.print("Fresh device required before peer verification.\r\n")?;
                } else {
                    self.print("Compare the FULL Ed25519 identity with the member over an independent\r\ntrusted channel. A home/server device list is not proof. No network enrollment.\r\n")?;
                    let who = self.line("Member (full user@home): ", 254, false)?;
                    let id = self.device("Peer device ID (1-2147483647): ")?;
                    let typed = self.line(
                        "Full independently verified identity (ed25519: + 64 lowercase hex): ",
                        72,
                        false,
                    )?;
                    self.show("Member: ", &who)?;
                    self.print(&format!("Device: {id}\r\n"))?;
                    self.show("Identity: ", &typed)?;
                    self.confirm("Type verified after independent comparison: ", "verified")?;
                    v.enroll(&who, id, &typed, now()?)?;
                    self.print("Peer enrolled locally. No session or network send started.\r\n")?;
                }
            }
            b'H' => {
                let history = v.history()?;
                let pending = v.inspect_outbox()?;
                self.print(&format!("Authenticated history: {}/64\r\n", history.len()))?;
                for r in history {
                    self.print(match r.direction {
                        Direction::Received => "RECEIVED / authenticated local history\r\n",
                        Direction::Sent if pending.iter().any(|p| p.operation == r.operation()) => {
                            "PENDING / local ciphertext; delivery unproven\r\n"
                        }
                        Direction::Sent => "RETIRED / local decision; delivery unproven\r\n",
                    })?;
                    self.print(&format!(
                        "Peer: {} / device {} / UTC {}\r\n",
                        r.address, r.device, r.sent_at
                    ))?;
                    if r.direction == Direction::Sent {
                        self.print(&format!("Operation: {}\r\n", hex(&r.operation())))?;
                    }
                    self.show("Subject: ", &r.subject)?;
                    self.show("Body: ", &r.body)?;
                    self.line("Enter for next record: ", 0, false)?;
                    self.clear()?;
                }
            }
            b'O' => {
                let pending = v.inspect_outbox()?;
                self.print(&format!("Durable outbox: {}/8\r\nOFFLINE: no delivery attempted by this UI. Remote delivery is unknown.\r\n", pending.len()))?;
                for p in pending {
                    self.print(&format!(
                        "Operation: {}\r\nTo: {} / device {} / UTC {}\r\n",
                        hex(&p.operation),
                        p.address,
                        p.device,
                        p.sent_at
                    ))?;
                }
                self.print("Expiry/revocation/recovery may block retry; retirement is a LOCAL decision.\r\n")?;
            }
            b'T' => {
                self.print("Retirement removes pending ciphertext. History and IDs survive.\r\nIt proves neither delivery nor reading; old copies may remain.\r\n")?;
                let typed = self.line(
                    "Operation ID to retire (32 lowercase hex, or all): ",
                    32,
                    false,
                )?;
                let id = if typed.as_str() == "all" {
                    None
                } else {
                    if typed.len() != 32 {
                        return Err(invalid());
                    }
                    let full = unhex(&format!("{}{}", typed.as_str(), "0".repeat(32)))?;
                    Some(<[u8; 16]>::try_from(&full[..16]).map_err(|_| invalid())?)
                };
                if let Some(id) = id
                    && !v.inspect_outbox()?.iter().any(|p| p.operation == id)
                {
                    return Err(io::Error::other("operation is not pending"));
                }
                self.confirm("Type retire to record this LOCAL decision: ", "retire")?;
                if let Some(id) = id {
                    v.acknowledge(id)?;
                } else {
                    v.discard_pending()?;
                }
                self.print(
                    "Retired locally. Delivery remains unproven; old copies may remain.\r\n",
                )?;
            }
            b'R' => {
                self.confirm("Type revoke to block this peer locally: ", "revoke")?;
                v.revoke()?;
                self.print("Peer REVOKED locally. No remote device or old copy was revoked.\r\n")?;
            }
            b'F' => {
                if !v.inspect_outbox()?.is_empty() {
                    self.print(
                        "Retire pending ciphertext explicitly before changing devices.\r\n",
                    )?;
                } else {
                    let id = self.device("New local device ID (different from current): ")?;
                    self.print("Old history, operation IDs and legacy drafts/pins remain. This creates a new\r\nnative identity. Old ratchets will not resume; peer verification is required again.\r\n")?;
                    self.confirm(
                        "Type fresh device to retire the old protocol identity: ",
                        "fresh device",
                    )?;
                    v.fresh_device(id, now()?)?;
                    self.print("Fresh local device created. Independently verify and enroll the peer again.\r\n")?;
                }
            }
            b'B' => {
                let path =
                    self.line("NEW absolute history-only backup directory: ", 1024, false)?;
                v.backup(Path::new(path.as_str()))?;
                self.print("Encrypted HISTORY-ONLY backup created. Test opening it before relying on it.\r\nOld copies keep old secrets; backups cannot resume old ratchets.\r\n")?;
            }
            b'S' => {
                let password = self.password()?;
                let recovery = v.reset(password.as_bytes())?;
                drop(password);
                self.recovery(&*recovery)?;
                self.print("Current secrets rotated. Old copies retain old secrets. Recovery quarantine is unchanged.\r\n")?;
            }
            b'D' => {
                let d = v.legacy()?;
                self.print(&format!("Legacy Curve25519 pins: {} (NOT Ed25519 enrollment)\r\nPreserved legacy drafts: {} (NOT sent or queued)\r\n",d.pins.len(),d.drafts.len()))?;
                for p in &d.pins {
                    self.print(&format!(
                        "Legacy {} / {} / {}\r\n",
                        p.address,
                        p.device,
                        hex(&p.fingerprint)
                    ))?;
                }
                for draft in &d.drafts {
                    self.show("To: ", &draft.to)?;
                    self.show("Subject: ", &draft.subject)?;
                    self.show("Body: ", &draft.body)?;
                    self.line("Enter for next draft: ", 0, false)?;
                    self.clear()?;
                }
            }
            _ => return Err(invalid()),
        }
        self.pause()
    }
}
pub fn run(path: &Path, action: &str, unicode: bool) -> io::Result<()> {
    if !path.is_absolute() || !matches!(action, "init" | "upgrade" | "open" | "recover") {
        return Err(invalid());
    }
    rustix::process::setrlimit(
        rustix::process::Resource::Core,
        rustix::process::Rlimit {
            current: Some(0),
            maximum: Some(0),
        },
    )?;
    let mut terminal = LocalTerminal::start()?;
    let result = (|| {
        let mut s = Screen {
            term: &mut terminal,
            activity: Instant::now(),
            unicode,
        };
        s.print("[LOCAL / OFFLINE] Native mail checkpoints\r\nMember-device experimental store. No SSH, provider, inbox or send connection.\r\n")?;
        let mut v = match action {
            "init" => {
                let home = s.line("Your full home identity (local claim only): ", 254, false)?;
                let data = Data::new(home.to_string())?;
                let device = s.device("Local device ID (1-2147483647): ")?;
                let password = s.password()?;
                let (v, key) = CheckpointVault::create(
                    Store::at(path, true)?,
                    data,
                    password.as_bytes(),
                    device,
                    now()?,
                )?;
                drop(password);
                s.recovery(&*key)?;
                v
            }
            "upgrade" => {
                s.print("Explicit v1-to-v2 upgrade replaces the current file and rotates secrets.\r\nPreserves legacy drafts/Curve pins without accepting them as native Ed25519 trust.\r\nKeep a tested v1 backup first. Old copies retain old secrets. V1 commands cannot open v2.\r\n")?;
                s.confirm("Type upgrade to replace v1 with v2: ", "upgrade")?;
                let store = Store::at(path, false)?;
                let old = s.line("Existing v1 LOCAL passphrase (hidden): ", 128, true)?;
                let legacy = Vault::unlock(store, old.as_bytes(), false)?;
                drop(old);
                let id = s.device("Local device ID (1-2147483647): ")?;
                let password = s.password()?;
                let (v, key) = CheckpointVault::upgrade(legacy, password.as_bytes(), id, now()?)?;
                drop(password);
                s.recovery(&*key)?;
                v
            }
            "recover" => {
                let store = Store::at(path, false)?;
                s.print("Recovery durably opens HISTORY ONLY before changing secrets.\r\nThis cannot be undone by password reopen/reset. Old ratchets cannot resume.\r\n")?;
                let typed = s.line("Offline recovery secret (64 hex, hidden): ", 64, true)?;
                let recovery = Zeroizing::new(unhex(&typed)?);
                drop(typed);
                let mut v = CheckpointVault::unlock(store, &*recovery, true)?;
                drop(recovery);
                let password = s.password()?;
                let next = v.reset(password.as_bytes())?;
                drop(password);
                s.recovery(&*next)?;
                v
            }
            _ => {
                let store = Store::at(path, false)?;
                let password = s.line("LOCAL checkpoint passphrase (hidden): ", 128, true)?;
                let v = CheckpointVault::unlock(store, password.as_bytes(), false)?;
                drop(password);
                v
            }
        };
        loop {
            let key = s.menu(&v)?;
            if key == b'L' {
                break;
            }
            s.action(&mut v, key)?;
        }
        Ok(())
    })();
    // Private state and unlock buffers have left scope before error output.
    let _ = terminal.print(&"\r\n".repeat(32));
    if result.is_err() {
        let _=terminal.print("Checkpoint locked: local operation stopped.\r\nWrite errors may poison the handle. Drop and reopen the canonical store; never retry that handle.\r\nAfter an interrupted reset/upgrade the NEW password may already apply.\r\nHistory-only recovery/backup needs explicit retirement and fresh-device verification.\r\n");
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_text_is_inert_in_classic_and_unicode_modes() {
        let text = "λ\n\x1b]52;c;hostile\x07\u{202e}safe";
        assert_eq!(
            inert(text, false).as_str(),
            "\\u{3bb}\\u{a}\\u{1b}]52;c;hostile\\u{7}\\u{202e}safe"
        );
        assert_eq!(
            inert(text, true).as_str(),
            "λ\\u{a}\\u{1b}]52;c;hostile\\u{7}\\u{202e}safe"
        );
    }
    #[test]
    fn inactivity_locks_without_remote_connection() {
        let (_master, slave) = faction_client_pty::open(24, 80).unwrap();
        let saved = rustix::termios::tcgetattr(&slave).unwrap();
        let mut term = LocalTerminal {
            input: slave.try_clone().unwrap().into(),
            output: slave.into(),
            saved,
        };
        let mut s = Screen {
            term: &mut term,
            activity: Instant::now() - IDLE,
            unicode: false,
        };
        assert_eq!(s.byte().unwrap_err().kind(), io::ErrorKind::TimedOut);
    }
}
