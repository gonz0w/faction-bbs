//! Deliberate member-device terminal entry. Never called by a remote stream,
//! profile, environment variable, or transport control frame. The explicit home-mail
//! action in the trusted local picker may call run_online between board visits.

#[cfg(feature = "network-mail")]
use faction_client::mail_checkpoint::ReadSummary;
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

#[path = "checkpoint_ui_inbox.rs"]
mod inbox;

#[cfg(feature = "network-mail")]
#[path = "checkpoint_ui_reply.rs"]
mod reply;

#[cfg(feature = "network-mail")]
#[path = "checkpoint_ui_attachments.rs"]
mod attachments;

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
    fn search_line(&mut self) -> io::Result<Option<inbox::Search>> {
        self.print("Search (64 bytes; blank clears): ")?;
        let mut bytes = Zeroizing::new(Vec::with_capacity(inbox::SEARCH_BYTES));
        #[cfg(test)]
        assert!(
            bytes.capacity() >= inbox::SEARCH_BYTES,
            "search scratch must not reallocate private prefixes"
        );
        let mut oversized = false;
        loop {
            match self.byte()? {
                b'\r' | b'\n' => {
                    self.print("\r\n")?;
                    return Ok(if oversized {
                        None
                    } else {
                        inbox::Search::parse(&bytes).ok()
                    });
                }
                8 | 127 => {
                    let remove = std::str::from_utf8(&bytes)
                        .ok()
                        .and_then(|text| text.chars().next_back())
                        .map_or(1, char::len_utf8);
                    let len = bytes.len().saturating_sub(remove);
                    bytes.truncate(len);
                }
                byte => {
                    if bytes.len() < inbox::SEARCH_BYTES {
                        bytes.push(byte);
                    } else {
                        // Drain the rejected line without growing private scratch.
                        oversized = true;
                    }
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
    fn local_read_status(&mut self, v: &CheckpointVault) -> io::Result<()> {
        let summary = v.read_summary()?;
        if let Some(unread) = summary.unread {
            self.print(&format!(
                "Local unread: {unread} / received {}; retained {}; pending {}\r\n",
                summary.received, summary.retained, summary.pending
            ))?;
        } else {
            self.print("Local unread: disabled; explicit E upgrade required\r\n")?;
            self.print(&format!(
                "Local retained: {}; received {}; pending {}\r\n",
                summary.retained, summary.received, summary.pending
            ))?;
        }
        self.print("Local retained snapshot only; no background refresh.\r\n")
    }
    fn menu(&mut self, v: &CheckpointVault) -> io::Result<u8> {
        let state = v.inspection()?;
        self.clear()?;
        self.print("[LOCAL / OFFLINE] Native mail checkpoints\r\nNo network inbox or send. No delivery/read/exactly-once claim.\r\n")?;
        self.show("Home (claimed, not authenticated): ", &v.legacy()?.home)?;
        self.local_read_status(v)?;
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
        self.print("[I] Local inbox\r\n")?;
        self.print("[E] Enable local read tracking\r\n")?;
        self.print("[P] Public device / peer   [V] Verify peer identity\r\n[H] Authenticated history [O] Durable outbox\r\n[T] Retire ciphertext     [R] Revoke peer locally\r\n[F] Fresh device         [B] History-only encrypted backup\r\n[S] Reset local secrets  [D] Preserved legacy drafts / Curve pins\r\n[L] Lock and exit\r\n")?;
        let mut decoder = Decoder::new();
        loop {
            if let Some(Key::Hotkey(c)) = decoder.feed(self.byte()?, Instant::now()) {
                let c = c.to_ascii_uppercase();
                if b"IPVHOTRFBSDLE".contains(&c) && c != b' ' {
                    return Ok(c);
                }
            }
        }
    }
    fn inbox_entries(v: &CheckpointVault) -> io::Result<Vec<inbox::Entry>> {
        let pending = v.inspect_outbox()?;
        v.history()?
            .iter()
            .enumerate()
            .map(|(index, record)| {
                let status = if let Some(identity) = record.received_id() {
                    match v.received_read(index, identity)? {
                        Some(false) => inbox::Status::Unread,
                        Some(true) => inbox::Status::Read,
                        None => inbox::Status::Unknown,
                    }
                } else if pending.iter().any(|p| p.operation == record.operation()) {
                    inbox::Status::Pending
                } else {
                    inbox::Status::Retired
                };
                Ok(inbox::Entry { index, status })
            })
            .collect()
    }
    fn inbox_group_entries(
        v: &CheckpointVault,
        group: Option<usize>,
    ) -> io::Result<Vec<inbox::Entry>> {
        let entries = Self::inbox_entries(v)?;
        if let Some(group) = group {
            let peers = v
                .history()?
                .iter()
                .map(|record| (record.address.as_str(), record.device))
                .collect::<Vec<_>>();
            Ok(inbox::within_group(entries, &peers, group))
        } else {
            Ok(entries)
        }
    }
    fn inbox_matches(
        v: &CheckpointVault,
        search: &inbox::Search,
    ) -> io::Result<Option<Vec<usize>>> {
        if !search.active() {
            return Ok(None);
        }
        Ok(Some(
            v.history()?
                .iter()
                .enumerate()
                .filter_map(|(index, record)| {
                    search
                        .matches(&record.address, &record.subject, &record.body)
                        .then_some(index)
                })
                .collect(),
        ))
    }
    fn correspondents(&mut self, v: &mut CheckpointVault, online: bool) -> io::Result<()> {
        let peers = v
            .history()?
            .iter()
            .map(|record| (record.address.as_str(), record.device))
            .collect::<Vec<_>>();
        let mut groups = inbox::Correspondents::new(&peers);
        drop(peers);
        let mut decoder = Decoder::new();
        loop {
            self.print("\r\n---------------------------------------\r\n[LOCAL CORRESPONDENTS]\r\nRetained device groups; newest first.\r\nNo contact or key-continuity claim.\r\n")?;
            let (page, pages) = groups.page();
            self.print(&format!(
                "Page {page}/{pages}; selection {}/{}\r\n",
                if groups.len() == 0 {
                    0
                } else {
                    groups.position() + 1
                },
                groups.len()
            ))?;
            if groups.visible().is_empty() {
                self.print("No retained correspondents.\r\n")?;
            }
            for (offset, &index) in groups.visible().iter().enumerate() {
                let record = &v.history()?[index];
                let number = (page - 1) * inbox::PAGE_ROWS + offset + 1;
                self.print(&inbox::correspondent_row(
                    number,
                    record.device,
                    &record.address,
                    groups.selected() == Some(index),
                    self.unicode,
                ))?;
                self.print("\r\n")?;
            }
            self.print("[Up/Down J/K] Select  [N/P] Page\r\n[Enter/O] Open  [B] Back\r\n")?;
            loop {
                let Some(key) = decoder.feed(self.byte()?, Instant::now()) else {
                    continue;
                };
                match key {
                    Key::Up | Key::Hotkey(b'K') => groups.up(),
                    Key::Down | Key::Hotkey(b'J') => groups.down(),
                    Key::Hotkey(b'N') => groups.next(),
                    Key::Hotkey(b'P') => groups.previous(),
                    Key::Hotkey(b'B') => return Ok(()),
                    Key::Enter | Key::Hotkey(b'O') => {
                        if let Some(index) = groups.selected() {
                            self.inbox_list(v, online, Some(index))?;
                            let peers = v
                                .history()?
                                .iter()
                                .map(|record| (record.address.as_str(), record.device))
                                .collect::<Vec<_>>();
                            groups.refresh(&peers);
                            decoder = Decoder::new();
                        }
                    }
                    _ => continue,
                }
                break;
            }
        }
    }
    fn inbox(&mut self, v: &mut CheckpointVault) -> io::Result<()> {
        self.inbox_mode(v, false)
    }
    fn inbox_mode(&mut self, v: &mut CheckpointVault, online: bool) -> io::Result<()> {
        self.inbox_list(v, online, None)
    }
    fn inbox_list(
        &mut self,
        v: &mut CheckpointVault,
        online: bool,
        group: Option<usize>,
    ) -> io::Result<()> {
        let mut list = inbox::Inbox::new(Self::inbox_group_entries(v, group)?);
        let mut search = inbox::Search::default();
        let mut decoder = Decoder::new();
        let mut unknown = false;
        loop {
            // A compact fixed separator and cell-bounded rows fit classic 40 columns.
            self.print("\r\n---------------------------------------\r\n")?;
            if let Some(index) = group {
                self.print("[LOCAL CORRESPONDENT INBOX]\r\nPeer: ")?;
                let record = &v.history()?[index];
                self.print(&inbox::compact(&record.address, self.unicode, 33))?;
                self.print(&format!("\r\nDevice: {}\r\n", record.device))?;
            } else {
                self.print("[LOCAL INBOX]\r\n")?;
            }
            self.print(match list.mode() {
                inbox::Filter::All => "All retained; newest first\r\n",
                inbox::Filter::Unread => "Unread received; newest first\r\n",
            })?;
            if search.active() {
                self.print("Local search active.\r\n")?;
            }
            self.print("Device snapshot; no home refresh.\r\n")?;
            let (page, pages) = list.page();
            self.print(&format!(
                "Page {page}/{pages}; selection {}/{}\r\n",
                if list.len() == 0 {
                    0
                } else {
                    list.position() + 1
                },
                list.len()
            ))?;
            if unknown {
                self.print("Unread unknown; tracking disabled.\r\n")?;
            }
            if list.visible().is_empty() {
                self.print(if search.active() {
                    "No matching retained records.\r\n"
                } else if list.mode() == inbox::Filter::Unread {
                    "No unread received records.\r\n"
                } else {
                    "No retained records.\r\n"
                })?;
            }
            for entry in list.visible() {
                let selected = list.selected() == Some(*entry);
                let prefix = format!(
                    "{} {:02} {:7}{} ",
                    if selected { '>' } else { ' ' },
                    entry.index + 1,
                    entry.status.label(),
                    if v.history()?[entry.index].attachment().is_some() {
                        "@"
                    } else {
                        ""
                    }
                );
                self.print(&prefix)?;
                self.print(&inbox::compact(
                    &v.history()?[entry.index].subject,
                    self.unicode,
                    39 - prefix.len(),
                ))?;
                self.print("\r\n")?;
            }
            self.print(
                "[Up/Down J/K] Select  [N/P] Page\r\n[Enter/O] Open  [U] Unread  [A] All\r\n",
            )?;
            if group.is_none() {
                self.print("[C] Correspondents\r\n")?;
            }
            self.print("[S] Search  [B] Back\r\n")?;
            loop {
                let Some(key) = decoder.feed(self.byte()?, Instant::now()) else {
                    continue;
                };
                match key {
                    Key::Up | Key::Hotkey(b'K') => list.up(),
                    Key::Down | Key::Hotkey(b'J') => list.down(),
                    Key::Hotkey(b'N') => list.next(),
                    Key::Hotkey(b'P') => list.previous(),
                    Key::Hotkey(b'U') => {
                        unknown = v.read_summary()?.unread.is_none();
                        if !unknown {
                            list.filter(inbox::Filter::Unread);
                        }
                    }
                    Key::Hotkey(b'A') => {
                        unknown = false;
                        list.filter(inbox::Filter::All);
                    }
                    Key::Hotkey(b'S') => {
                        if let Some(query) = self.search_line()? {
                            let matching = Self::inbox_matches(v, &query)?;
                            search = query;
                            list.search(matching);
                        } else {
                            self.print("Search rejected; previous query kept.\r\n")?;
                        }
                        decoder = Decoder::new();
                    }
                    Key::Hotkey(b'B') => return Ok(()),
                    Key::Hotkey(b'C') if group.is_none() => {
                        self.correspondents(v, online)?;
                        list.refresh_matching(
                            Self::inbox_entries(v)?,
                            Self::inbox_matches(v, &search)?,
                        );
                        decoder = Decoder::new();
                    }
                    Key::Enter | Key::Hotkey(b'O') => {
                        if let Some(entry) = list.selected() {
                            self.inbox_record_mode(v, entry, online)?;
                            list.refresh_matching(
                                Self::inbox_group_entries(v, group)?,
                                Self::inbox_matches(v, &search)?,
                            );
                            decoder = Decoder::new();
                        }
                    }
                    _ => continue,
                }
                break;
            }
        }
    }
    #[cfg(test)]
    fn inbox_record(&mut self, v: &mut CheckpointVault, entry: inbox::Entry) -> io::Result<()> {
        self.inbox_record_mode(v, entry, false)
    }
    fn inbox_record_mode(
        &mut self,
        v: &mut CheckpointVault,
        entry: inbox::Entry,
        online: bool,
    ) -> io::Result<()> {
        let index = entry.index;
        let record = &v.history()?[index];
        self.print("\r\n---------------------------------------\r\n[LOCAL INBOX RECORD]\r\n")?;
        self.print(&format!(
            "{} / local retained record {}\r\n",
            entry.status.label(),
            index + 1
        ))?;
        self.show("Peer: ", &record.address)?;
        self.print(&format!(
            "Device: {} / UTC {}\r\n",
            record.device, record.sent_at
        ))?;
        self.show("Subject: ", &record.subject)?;
        self.show("Body: ", &record.body)?;
        #[cfg(feature = "network-mail")]
        if let Some(descriptor) = record.attachment() {
            self.attachment_details(descriptor)?;
            self.print("Use M at the online menu for actions.\r\n")?;
        }
        let received = record.received_id();
        if let Some(identity) = received
            && v.received_read(index, identity)?.is_some()
        {
            // Fresh input only after all selected body output succeeded. Bind
            // the decision to its original history index and received identity.
            rustix::termios::tcflush(&self.term.input, rustix::termios::QueueSelector::IFlush)?;
            let answer = self.line(
                "Type read to acknowledge locally, or Enter to keep current state: ",
                16,
                false,
            )?;
            if answer.as_str() == "read" {
                v.mark_read(index, identity)?;
                self.print("Acknowledged on this device only; no read receipt sent.\r\n")?;
            }
        }
        #[cfg(feature = "network-mail")]
        if online && let Some(identity) = received {
            self.print("[R] Reply locally  [B/Enter] Back to inbox\r\n")?;
            let mut decoder = Decoder::new();
            loop {
                match decoder.feed(self.byte()?, Instant::now()) {
                    Some(Key::Hotkey(b'R')) => return self.reply(v, index, identity),
                    Some(Key::Enter | Key::Hotkey(b'B')) => return Ok(()),
                    _ => {}
                }
            }
        }
        #[cfg(not(feature = "network-mail"))]
        let _ = online;
        self.pause()
    }
    fn action(&mut self, v: &mut CheckpointVault, key: u8) -> io::Result<()> {
        if key == b'I' {
            return self.inbox(v);
        }
        self.clear()?;
        match key {
            b'E' => {
                if v.read_summary()?.unread.is_some() {
                    self.print("Encrypted local read tracking already enabled; existing decisions kept.\r\n")?;
                } else {
                    self.print("Explicit encrypted local read metadata upgrade. Older clients cannot open FACTCP04.\r\nRetained received messages start unread; sent messages never count.\r\nNo home read receipt, network cycle, key change or plaintext sidecar.\r\n")?;
                    self.confirm(
                        "Type enable to add encrypted local read tracking: ",
                        "enable",
                    )?;
                    v.enable_read_tracking()?;
                    self.print("Encrypted local read tracking enabled. Existing received history starts unread.\r\n")?;
                }
            }
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
                let count = v.history()?.len();
                let pending = v
                    .inspect_outbox()?
                    .into_iter()
                    .map(|p| p.operation)
                    .collect::<Vec<_>>();
                self.print(&format!("Authenticated history: {count}/64\r\n"))?;
                for index in 0..count {
                    let r = &v.history()?[index];
                    self.print(match r.direction {
                        Direction::Received => "RECEIVED / authenticated local history\r\n",
                        Direction::Sent if pending.contains(&r.operation()) => {
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
                    if let Some(identity) = r.received_id()
                        && let Some(acknowledged) = v.received_read(index, identity)?
                    {
                        self.print(if acknowledged {
                            "READ / acknowledged on this device\r\n"
                        } else {
                            "UNREAD / retained on this device\r\n"
                        })?;
                        // Only a fresh local decision after successful body output
                        // may mark this exact received record. Drop pasted typeahead.
                        rustix::termios::tcflush(
                            &self.term.input,
                            rustix::termios::QueueSelector::IFlush,
                        )?;
                        let answer = self.line(
                            "Type read to acknowledge locally, or Enter to keep current state: ",
                            16,
                            false,
                        )?;
                        if answer.as_str() == "read" {
                            v.mark_read(index, identity)?;
                            self.print(
                                "Acknowledged on this device only; no read receipt sent.\r\n",
                            )?;
                        }
                    }
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
#[cfg(feature = "network-mail")]
impl Screen<'_> {
    fn compose_line(&mut self, label: &str, cap: usize) -> io::Result<Zeroizing<String>> {
        self.print(label)?;
        let mut input = Zeroizing::new(Vec::new());
        loop {
            let byte = self.byte()?;
            match faction_terminal::subject::feed_limited(&mut input, byte, self.unicode, cap)
                .map_err(|_| invalid())?
            {
                faction_terminal::subject::Edit::Echo(text) => self.print(&Zeroizing::new(text))?,
                faction_terminal::subject::Edit::Submit(text) => {
                    self.print("\r\n")?;
                    return Ok(Zeroizing::new(text));
                }
            }
        }
    }
    fn reply_unavailable(&mut self, v: &CheckpointVault) -> io::Result<()> {
        // A poisoned handle must reach the existing drop/reopen lock path.
        v.inspection()?;
        self.print("Reply unavailable; check selected received record, current prepared peer, expiry and retained-store limits.\r\n")
    }
    fn reply_target(&mut self, target: &reply::Target) -> io::Result<()> {
        self.show("CURRENT recipient: ", &target.address)?;
        self.print(&format!(
            "CURRENT device: {} / ed25519:{}\r\n",
            target.device,
            hex(&target.identity)
        ))
    }
    fn reply(
        &mut self,
        v: &mut CheckpointVault,
        index: usize,
        received: [u8; 32],
    ) -> io::Result<()> {
        let target = match reply::Target::prepare(v, index, received, now()?) {
            Ok(target) => target,
            Err(_) => return self.reply_unavailable(v),
        };
        self.reply_target(&target)?;
        self.print(
            "Current independently verified identity; history does not prove key continuity.\r\n",
        )?;
        let (suggestion, shortened) = reply::suggested_subject(&v.history()?[index].subject);
        self.show("Suggested subject: ", &suggestion)?;
        if shortened {
            self.print("Suggested subject shortened to the 128 UTF-8 byte limit.\r\n")?;
        }
        let replacement = self.compose_line(
            "Subject (Enter keeps suggestion; fresh text replaces, 128 UTF-8 bytes): ",
            128,
        )?;
        let subject = if replacement.is_empty() {
            suggestion
        } else {
            replacement
        };
        let body = self.compose_line("Body (1024 UTF-8 bytes, one line): ", 1024)?;
        self.print("Review local reply intent:\r\n")?;
        self.reply_target(&target)?;
        self.show("Subject: ", &subject)?;
        self.show("Body: ", &body)?;
        // Discard typeahead only AFTER all reviewed fields have reached output.
        rustix::termios::tcflush(&self.term.input, rustix::termios::QueueSelector::IFlush)?;
        if self
            .line("Type queue to persist encrypted ciphertext: ", 32, false)?
            .as_str()
            != "queue"
        {
            return self.print("Reply cancelled; no ciphertext queued.\r\n");
        }
        let timestamp = now()?;
        if target.recheck(v, timestamp).is_err() {
            return self.reply_unavailable(v);
        }
        let mut operation = [0; 16];
        getrandom::fill(&mut operation).map_err(|_| invalid())?;
        match v.queue(operation, &subject, &body, timestamp, timestamp) {
            Ok(_) => self.print("Queued durably; no delivery or read receipt.\r\n"),
            Err(_) => self.reply_unavailable(v),
        }
    }
    fn online_menu(&mut self, v: &CheckpointVault) -> io::Result<u8> {
        self.clear()?;
        self.print("[LOCAL / MANUAL ONLINE] Encrypted home mail\r\n")?;
        self.show("Home: ", &v.legacy()?.home)?;
        self.local_read_status(v)?;
        self.print(&format!(
            "History: {}   Pending ciphertext: {}\r\n",
            v.history()?.len(),
            v.inspect_outbox()?.len()
        ))?;
        self.print(if v.inspection()?.archive_enabled {
            "Archive checkpoints: enabled\r\n"
        } else {
            "Archive checkpoints: disabled; explicit upgrade required\r\n"
        })?;
        self.print("[I] Local inbox\r\n[M] Private attachments\r\n")?;
        self.print("[E] Enable local read tracking\r\n")?;
        self.print("[G] Enable archive checkpoints\r\n[A] Catch up one archive page\r\n")?;
        self.print("[U] Publish public keys   [K] Prepare verified peer\r\n")?;
        self.print(if v.inspection()?.archive_enabled {
            "[C] Compose and queue     [N] Send pending\r\n"
        } else {
            "[C] Compose and queue     [N] Connect / sync\r\n"
        })?;
        self.print("[P] Public identity       [V] Verify peer\r\n[H] History               [O] Pending operation IDs\r\n[T] Retire ciphertext     [R] Revoke peer locally\r\n[F] Fresh device          [B] History-only backup\r\n[S] Reset local secrets   [D] Legacy local drafts\r\n[L] Lock and exit\r\n")?;
        let mut decoder = Decoder::new();
        loop {
            if let Some(Key::Hotkey(key)) = decoder.feed(self.byte()?, Instant::now()) {
                let key = key.to_ascii_uppercase();
                if b"IUKCNPVHOTRFBSDLGAEM".contains(&key) {
                    return Ok(key);
                }
            }
        }
    }
    async fn network_cancel(&mut self) -> io::Result<()> {
        loop {
            tokio::time::sleep(Duration::from_millis(50)).await;
            if self.term.ready(0)? && matches!(self.term.byte()?, 3 | 4) {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "local cancellation",
                ));
            }
        }
    }
    fn online_action(
        &mut self,
        v: &mut CheckpointVault,
        settings: &faction_client::mail_network::Settings,
        root: &Path,
        key: u8,
        provider_profile: Option<&Path>,
    ) -> io::Result<()> {
        use faction_client::mail_network::{self, Mode};
        if key == b'M' {
            return self.attachment_desk(v, settings, root, provider_profile);
        }
        if key == b'I' {
            return self.inbox_mode(v, true);
        }
        if !b"UKCNOGA".contains(&key) {
            return self.action(v, key);
        }
        self.clear()?;
        if key == b'G' {
            if v.inspection()?.archive_enabled {
                self.print("Archive checkpoints already enabled; existing cursor kept.\r\n")?;
            } else {
                self.print("Explicit checkpoint schema upgrade for one bounded archive page.\r\nExisting keys, trust, history, recovery and pending ciphertext remain.\r\nArchive mode receives retained mail with A; N sends pending ciphertext without announcing availability.\r\nOlder clients cannot open the upgraded schema. Keep a history-only backup first.\r\n")?;
                self.confirm(
                    "Type upgrade to enable encrypted archive checkpoints: ",
                    "upgrade",
                )?;
                match v.enable_archive(now()?) {
                    Ok(()) => self.print("Archive checkpoints enabled; existing messages, keys and pending ciphertext kept.\r\n")?,
                    Err(_) => {
                        v.inspection()?;
                        self.print("Archive upgrade stopped; check verified peer, recovery, expiry and retained-store limits.\r\n")?;
                    }
                }
            }
            return self.pause();
        }
        if key == b'A' {
            if !v.inspection()?.archive_enabled {
                self.print("Enable archive checkpoints with G before catch-up.\r\n")?;
                return self.pause();
            }
            if v.archive_checkpoint(now()?).is_err() {
                self.print("Catch-up unavailable; check verified peer, recovery, expiry and full history/outbox.\r\nNo archive cursor or pending ciphertext was changed.\r\n")?;
                return self.pause();
            }
            self.print("One explicit archive page: at most four entries. Server order and completeness are not guaranteed.\r\n")?;
        }
        if key == b'O' {
            self.print("Durable pending ciphertext; delivery remains unproven.\r\nA socket write never retires these IDs. Use Retire only as a deliberate local decision.\r\n")?;
            for item in v.inspect_outbox()? {
                self.print(&format!(
                    "Operation: {} / {} / UTC {}\r\n",
                    hex(&item.operation),
                    item.address,
                    item.sent_at
                ))?;
            }
            return self.pause();
        }
        if key == b'C' {
            let state = v.inspection()?;
            if !state.started {
                self.print("Prepare the independently verified peer before composing.\r\n")?;
                return self.pause();
            }
            let peer = state.peer.ok_or_else(invalid)?.address.to_owned();
            let subject = self.compose_line("Subject (128 UTF-8 bytes): ", 128)?;
            let body = self.compose_line("Body (1024 UTF-8 bytes, one line): ", 1024)?;
            self.show("To: ", &peer)?;
            self.show("Subject: ", &subject)?;
            self.show("Body: ", &body)?;
            self.confirm("Type queue to persist encrypted ciphertext: ", "queue")?;
            let mut operation = [0; 16];
            getrandom::fill(&mut operation).map_err(|_| invalid())?;
            match v.queue(operation, &subject, &body, now()?, now()?) {
                Ok(_) => self.print("Queued durably; no delivery or read receipt.\r\n")?,
                Err(_) => {
                    v.inspection()?;
                    self.print("Queue stopped; check peer, expiry and retained-store limits.\r\n")?;
                }
            }
            return self.pause();
        }
        let mode = match key {
            b'U' => Mode::Publish,
            b'K' => Mode::Prepare,
            b'A' => Mode::CatchUp,
            _ => Mode::Sync,
        };
        if mode == Mode::Prepare {
            self.print("Use Prepare only on the chosen FIRST SENDER. The receiving member publishes and connects first.\r\nUses only your already verified peer identity. It cannot replace an existing session.\r\n")?;
            self.confirm(
                "Type prepare to start the verified peer session: ",
                "prepare",
            )?;
        }
        let password = self.line(
            "HOME account password (hidden; never sent to a BBS): ",
            128,
            true,
        )?;
        self.print("Connecting directly to your home. Ctrl-C locks and cancels.\r\n")?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let timestamp = now()?;
        let result = runtime.block_on(async {
            tokio::select! {
                result=mail_network::cycle(v,settings,&password,mode,timestamp)=>Ok(result),
                cancelled=self.network_cancel()=>{cancelled?;Err(invalid())}
            }
        })?;
        drop(password);
        self.activity = Instant::now();
        match result {
            Ok(report) => {
                if let Some(page) = report.archive {
                    self.print(&format!("Archive page saved. Entries: {}; accepted: {}; duplicates or known sent copies: {}.\r\n", page.entries, report.accepted, report.duplicates))?;
                    self.print(if page.complete {
                        "Server marked this page complete; this does not prove complete server history.\r\n"
                    } else {
                        "More may be available. Use A for another deliberate page.\r\n"
                    })?;
                    self.print("Pending ciphertext is retained; archived sent copies are not delivery receipts.\r\n")?;
                } else {
                    self.print(&format!("Manual cycle complete. Sent writes: {}; accepted: {}; duplicates: {}.\r\nPending ciphertext is retained. A write is not a delivery/read receipt.\r\n",report.sent,report.accepted,report.duplicates))?;
                }
            }
            Err(error) => {
                if mode == Mode::CatchUp && v.inspection().is_err() {
                    self.print("Local checkpoint unavailable. Lock and reopen the canonical store before further work.\r\n")?;
                    return Err(invalid());
                }
                v.inspection()?;
                self.print(if matches!(error, mail_network::Error::ArchiveCursorUnavailable) {
                    "Home no longer recognizes the saved archive position.\r\nRetained history is intact. Automatic reset or skip is unavailable.\r\nAsk your home operator about archive retention; the saved cursor is kept.\r\n"
                } else if mode == Mode::CatchUp {
                    "Catch-up stopped; earlier accepted entries may already be saved.\r\nInspect History and retry explicitly. After ambiguous local I/O, lock and reopen the canonical store.\r\n"
                } else {
                    "Home cycle stopped; pending ciphertext retained.\r\nCheck local configuration, account or peer; history remains available.\r\n"
                })?;
            }
        }
        self.pause()
    }
}
/// Explicit trusted-local entry only. Remote streams/picker metadata never call it.
#[cfg(feature = "network-mail")]
pub fn run_online(path: &Path, config: &Path, unicode: bool) -> io::Result<()> {
    run_online_with_status(path, config, unicode).map(|_| ())
}
#[cfg(feature = "network-mail")]
pub fn run_online_with_provider(
    path: &Path,
    config: &Path,
    unicode: bool,
    provider_profile: Option<&Path>,
) -> io::Result<()> {
    run_online_with_status_and_provider(path, config, unicode, provider_profile).map(|_| ())
}
/// Return only a healthy local retained-state snapshot after graceful locking.
/// Cancellation, unlock/configuration failures and poisoned handles return no
/// successful snapshot. This never establishes home completeness or freshness.
#[cfg(feature = "network-mail")]
pub fn run_online_with_status(
    path: &Path,
    config: &Path,
    unicode: bool,
) -> io::Result<Option<ReadSummary>> {
    run_online_with_status_and_provider(path, config, unicode, None)
}
#[cfg(feature = "network-mail")]
pub fn run_online_with_status_and_provider(
    path: &Path,
    config: &Path,
    unicode: bool,
    provider_profile: Option<&Path>,
) -> io::Result<Option<ReadSummary>> {
    if !path.is_absolute() || !config.is_absolute() {
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
        let settings = faction_client::mail_network::Settings::load(config)?;
        let mut s = Screen {
            term: &mut terminal,
            activity: Instant::now(),
            unicode,
        };
        s.print("[LOCAL / MANUAL ONLINE] Encrypted home mail\r\nManual connections; keys and decryption stay on this device.\r\nManual archive pages; server completeness is not guaranteed.\r\n")?;
        let store = Store::at(path, false)?;
        let password = s.line("LOCAL checkpoint passphrase (hidden): ", 128, true)?;
        let mut vault = CheckpointVault::unlock(store, password.as_bytes(), false)?;
        drop(password);
        if vault.legacy()?.home != settings.account {
            return Err(invalid());
        }
        loop {
            let key = s.online_menu(&vault)?;
            if key == b'L' {
                break;
            }
            s.online_action(&mut vault, &settings, path, key, provider_profile)?;
        }
        Ok(Some(vault.read_summary()?))
    })();
    let _ = terminal.print(&"\r\n".repeat(32));
    if result.is_err() {
        let _=terminal.print("Home mail locked: local operation stopped. Reopen the canonical store before retrying.\r\n");
    }
    result
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
    fn inbox_output_failure_and_input_eof_preserve_unread_and_restore_tty() {
        // Backend creation permanently disables core dumps. Keep its process
        // limits out of the parent suite, including the existing early guard.
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "checkpoint_ui::tests::inbox_output_failure_and_input_eof_helper",
                "--nocapture",
            ])
            .output()
            .unwrap();
        assert!(
            child.status.success(),
            "isolated inbox failure control failed: {}",
            String::from_utf8_lossy(&child.stdout)
        );
    }
    #[test]
    #[ignore = "owned subprocess helper, exercised by the inbox failure control"]
    fn inbox_output_failure_and_input_eof_helper() {
        use faction_client::mail_checkpoint::Wire;
        use faction_omemo::{Device, sce};
        use std::os::unix::fs::PermissionsExt;
        struct Owned(std::path::PathBuf);
        impl Drop for Owned {
            fn drop(&mut self) {
                assert!(self.0.join("OWNERSHIP").is_file());
                std::fs::remove_dir_all(&self.0).unwrap();
            }
        }
        let mut id = [0; 16];
        getrandom::fill(&mut id).unwrap();
        let root = Owned(
            std::env::temp_dir()
                .canonicalize()
                .unwrap()
                .join(format!("faction219-ui-OWNED-{}", hex(&id))),
        );
        std::fs::create_dir(&root.0).unwrap();
        std::fs::set_permissions(&root.0, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::write(
            root.0.join("OWNERSHIP"),
            b"synthetic inbox failure control\n",
        )
        .unwrap();
        let path = root.0.join("vault");
        let timestamp = now().unwrap();
        let (mut vault, _) = CheckpointVault::create(
            Store::at(&path, true).unwrap(),
            Data::new("alice@home.test".into()).unwrap(),
            b"synthetic local failure test password",
            101,
            timestamp,
        )
        .unwrap();
        let mut peer = Device::fresh().unwrap();
        let local = vault.bundle().unwrap();
        let remote = peer.bundle().unwrap();
        vault
            .enroll(
                "bob@away.test",
                202,
                &format!("ed25519:{}", hex(&remote.identity)),
                timestamp,
            )
            .unwrap();
        peer.trust(&format!("ed25519:{}", hex(&local.identity)))
            .unwrap();
        peer.start(&local).unwrap();
        let plain = sce::encode(
            "bob@away.test",
            "alice@home.test",
            timestamp,
            "synthetic selected subject",
            "synthetic selected body",
        )
        .unwrap();
        let incoming = peer.seal(&plain).unwrap();
        vault
            .receive(
                &Wire {
                    namespace: "urn:xmpp:omemo:2".into(),
                    from: "bob@away.test".into(),
                    to: "alice@home.test".into(),
                    device: 202,
                    to_device: 101,
                    identity: remote.identity,
                    sent_at: timestamp,
                    key: incoming.key,
                    prekey: incoming.prekey,
                    payload: incoming.payload,
                },
                timestamp,
            )
            .unwrap();
        vault.enable_read_tracking().unwrap();
        #[cfg(feature = "network-mail")]
        let vault_identity = vault.history().unwrap()[0].received_id().unwrap();
        let before = std::fs::read(path.join("mail.fkv")).unwrap();
        std::fs::write(root.0.join("eof"), []).unwrap();
        #[cfg(feature = "network-mail")]
        let attempts = [false, true].as_slice();
        #[cfg(not(feature = "network-mail"))]
        let attempts = [false].as_slice();
        for &reply_attempt in attempts {
            #[cfg(not(feature = "network-mail"))]
            let _ = reply_attempt;
            for fail_output in [true, false] {
                let (_master, slave) = faction_client_pty::open(24, 40).unwrap();
                let observer = std::fs::File::from(slave.try_clone().unwrap());
                let saved = rustix::termios::tcgetattr(&slave).unwrap();
                let mut raw = saved.clone();
                raw.make_raw();
                rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw)
                    .unwrap();
                let mut term = LocalTerminal {
                    input: slave.try_clone().unwrap().into(),
                    // An actual read-only output descriptor fails writes without
                    // depending on platform-specific /dev/full or signal timing.
                    output: if fail_output {
                        std::fs::File::open("/dev/null").unwrap()
                    } else {
                        slave.into()
                    },
                    saved: saved.clone(),
                };
                let input = if fail_output {
                    None
                } else {
                    Some(std::mem::replace(
                        &mut term.input,
                        std::fs::File::open(root.0.join("eof")).unwrap(),
                    ))
                };
                let error = {
                    let mut screen = Screen {
                        term: &mut term,
                        activity: Instant::now(),
                        unicode: false,
                    };
                    #[cfg(feature = "network-mail")]
                    if reply_attempt {
                        screen.reply(&mut vault, 0, vault_identity).unwrap_err()
                    } else if fail_output {
                        screen
                            .inbox_record(
                                &mut vault,
                                inbox::Entry {
                                    index: 0,
                                    status: inbox::Status::Unread,
                                },
                            )
                            .unwrap_err()
                    } else {
                        screen.inbox(&mut vault).unwrap_err()
                    }
                    #[cfg(not(feature = "network-mail"))]
                    if fail_output {
                        screen
                            .inbox_record(
                                &mut vault,
                                inbox::Entry {
                                    index: 0,
                                    status: inbox::Status::Unread,
                                },
                            )
                            .unwrap_err()
                    } else {
                        screen.inbox(&mut vault).unwrap_err()
                    }
                };
                if !fail_output {
                    assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
                }
                // Put the owned tty input back so Drop can restore the actual tty.
                if let Some(input) = input {
                    term.input = input;
                }
                drop(term);
                let restored = rustix::termios::tcgetattr(&observer).unwrap();
                // Darwin and NetBSD termios headers describe PENDIN as retyping
                // or re-echoing pending input. Both kernels set this state bit when
                // restoring identical configured modes; mask only that observed bit.
                #[cfg(any(target_os = "macos", target_os = "netbsd"))]
                let queue_state = rustix::termios::LocalModes::PENDIN;
                #[cfg(not(any(target_os = "macos", target_os = "netbsd")))]
                let queue_state = rustix::termios::LocalModes::empty();
                assert_eq!(
                    restored.local_modes & !queue_state,
                    saved.local_modes & !queue_state
                );
                assert_eq!(restored.input_modes, saved.input_modes);
                assert_eq!(restored.output_modes, saved.output_modes);
                assert_eq!(restored.control_modes, saved.control_modes);
                assert_eq!(restored.input_speed(), saved.input_speed());
                assert_eq!(restored.output_speed(), saved.output_speed());
                // Rustix exposes the complete control array through Debug, with no
                // public iterator. Compare every control value, including padding.
                assert_eq!(
                    format!("{:?}", restored.special_codes),
                    format!("{:?}", saved.special_codes)
                );
                assert_eq!(vault.read_summary().unwrap().unread, Some(1));
                assert_eq!(std::fs::read(path.join("mail.fkv")).unwrap(), before);
            }
        }
        drop(vault);
    }
    #[test]
    fn correspondents_opens_from_actual_inbox_and_backs_to_global() {
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "checkpoint_ui::tests::correspondents_actual_interface_helper",
                "--nocapture",
            ])
            .output()
            .unwrap();
        assert!(
            child.status.success(),
            "actual correspondent interface failed: {} {}",
            String::from_utf8_lossy(&child.stdout),
            String::from_utf8_lossy(&child.stderr)
        );
    }
    #[test]
    #[ignore = "owned subprocess helper, exercised by the correspondent interface control"]
    fn correspondents_actual_interface_helper() {
        use std::{io::Write, os::unix::fs::PermissionsExt};
        struct Owned(std::path::PathBuf);
        impl Drop for Owned {
            fn drop(&mut self) {
                assert!(self.0.join("OWNERSHIP").is_file());
                std::fs::remove_dir_all(&self.0).unwrap();
            }
        }
        let mut id = [0; 16];
        getrandom::fill(&mut id).unwrap();
        let root = Owned(
            std::env::temp_dir()
                .canonicalize()
                .unwrap()
                .join(format!("faction235-interface-OWNED-{}", hex(&id))),
        );
        std::fs::create_dir(&root.0).unwrap();
        std::fs::set_permissions(&root.0, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::write(
            root.0.join("OWNERSHIP"),
            b"synthetic correspondent interface control\n",
        )
        .unwrap();
        let path = root.0.join("vault");
        let (mut vault, _) = CheckpointVault::create(
            Store::at(&path, true).unwrap(),
            Data::new("alice@home.test".into()).unwrap(),
            b"synthetic local interface password",
            101,
            now().unwrap(),
        )
        .unwrap();
        let before = std::fs::read(path.join("mail.fkv")).unwrap();
        for unicode in [false, true] {
            for online in [false, true] {
                let (master, slave) = faction_client_pty::open(24, 40).unwrap();
                let mut input = std::fs::File::from(master);
                let saved = rustix::termios::tcgetattr(&slave).unwrap();
                let mut raw = saved.clone();
                raw.make_raw();
                rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw)
                    .unwrap();
                let capture = root.0.join("output");
                let mut term = LocalTerminal {
                    input: slave.into(),
                    output: std::fs::File::create(&capture).unwrap(),
                    saved,
                };
                input.write_all(b"cbb").unwrap();
                Screen {
                    term: &mut term,
                    activity: Instant::now(),
                    unicode,
                }
                .inbox_mode(&mut vault, online)
                .unwrap();
                drop(term);
                let output = std::fs::read_to_string(capture).unwrap();
                assert!(
                    output.contains("[LOCAL CORRESPONDENTS]"),
                    "C must open the correspondent picker"
                );
                assert!(output.contains("No retained correspondents."));
                assert_eq!(
                    output.matches("[LOCAL INBOX]").count(),
                    2,
                    "picker Back returns to original global inbox"
                );
                assert!(!output.contains("Reply locally"));
                assert_eq!(std::fs::read(path.join("mail.fkv")).unwrap(), before);
            }
        }
    }
    #[test]
    fn search_opens_from_actual_inbox_and_is_ephemeral() {
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "checkpoint_ui::tests::search_actual_interface_helper",
                "--nocapture",
            ])
            .output()
            .unwrap();
        assert!(
            child.status.success(),
            "actual search interface failed: {} {}",
            String::from_utf8_lossy(&child.stdout),
            String::from_utf8_lossy(&child.stderr)
        );
    }
    #[test]
    #[ignore = "owned subprocess helper, exercised by the search interface control"]
    fn search_actual_interface_helper() {
        use std::{io::Write, os::unix::fs::PermissionsExt};
        struct Owned(std::path::PathBuf);
        impl Drop for Owned {
            fn drop(&mut self) {
                assert!(self.0.join("OWNERSHIP").is_file());
                std::fs::remove_dir_all(&self.0).unwrap();
            }
        }
        let mut id = [0; 16];
        getrandom::fill(&mut id).unwrap();
        let root = Owned(
            std::env::temp_dir()
                .canonicalize()
                .unwrap()
                .join(format!("faction242-interface-OWNED-{}", hex(&id))),
        );
        std::fs::create_dir(&root.0).unwrap();
        std::fs::set_permissions(&root.0, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::write(
            root.0.join("OWNERSHIP"),
            b"synthetic search interface control\n",
        )
        .unwrap();
        let path = root.0.join("vault");
        let (mut vault, _) = CheckpointVault::create(
            Store::at(&path, true).unwrap(),
            Data::new("alice@home.test".into()).unwrap(),
            b"synthetic local interface password",
            101,
            now().unwrap(),
        )
        .unwrap();
        let before = std::fs::read(path.join("mail.fkv")).unwrap();
        for unicode in [false, true] {
            for online in [false, true] {
                let (master, slave) = faction_client_pty::open(24, 40).unwrap();
                let mut input = std::fs::File::from(master);
                let saved = rustix::termios::tcgetattr(&slave).unwrap();
                let mut raw = saved.clone();
                raw.make_raw();
                rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw)
                    .unwrap();
                let capture = root.0.join("output");
                let mut term = LocalTerminal {
                    input: slave.into(),
                    output: std::fs::File::create(&capture).unwrap(),
                    saved,
                };
                input.write_all(b"sneedle\rs\rsneedle\rs\xff\rb").unwrap();
                Screen {
                    term: &mut term,
                    activity: Instant::now(),
                    unicode,
                }
                .inbox_mode(&mut vault, online)
                .unwrap();
                drop(term);
                let output = std::fs::read_to_string(capture).unwrap();
                assert!(
                    output.contains("Search (64 bytes; blank clears): "),
                    "S must open local search"
                );
                assert_eq!(output.matches("Local search active.").count(), 3);
                assert_eq!(output.matches("No matching retained records.").count(), 3);
                assert!(output.contains("Search rejected; previous query kept."));
                assert!(
                    !output.contains("needle"),
                    "query stays out of terminal output"
                );
                assert!(
                    output
                        .lines()
                        .all(|line| faction_terminal::text::width(line) <= 39),
                    "every search list and hint line fits classic display"
                );
                // Each invocation is a new owning view; its first draw is unfiltered.
                let first = output
                    .split("Search (64 bytes; blank clears): ")
                    .next()
                    .unwrap();
                assert!(first.contains("No retained records."));
                assert!(!first.contains("Local search active."));
                assert!(!output.contains("Reply locally"));
                assert_eq!(std::fs::read(path.join("mail.fkv")).unwrap(), before);
            }
        }
    }
    #[test]
    fn search_prompt_edits_unicode_bounds_input_and_keeps_lock_errors() {
        use std::io::Write;
        let boundary = "x".repeat(64);
        let cases = [
            (b"\r".to_vec(), Some("")),
            ("λ界\x7f\r".as_bytes().to_vec(), Some("λ")),
            (
                [vec![b'x'; 64], b"\r".to_vec()].concat(),
                Some(boundary.as_str()),
            ),
            ([vec![b'x'; 65], b"\x7f\r".to_vec()].concat(), None),
            (b"\xff\r".to_vec(), None),
            (b"\x1b\r".to_vec(), None),
            ("\u{202e}\r".as_bytes().to_vec(), None),
        ];
        for unicode in [false, true] {
            for (input, expected) in &cases {
                let (master, slave) = faction_client_pty::open(24, 40).unwrap();
                let saved = rustix::termios::tcgetattr(&slave).unwrap();
                let mut raw = saved.clone();
                raw.make_raw();
                rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw)
                    .unwrap();
                let mut term = LocalTerminal {
                    input: slave.into(),
                    output: std::fs::OpenOptions::new()
                        .write(true)
                        .open("/dev/null")
                        .unwrap(),
                    saved,
                };
                let mut master = std::fs::File::from(master);
                master.write_all(input).unwrap();
                let result = Screen {
                    term: &mut term,
                    activity: Instant::now(),
                    unicode,
                }
                .search_line()
                .unwrap();
                assert_eq!(result.is_some(), expected.is_some());
                if let (Some(query), Some(text)) = (result, expected) {
                    assert_eq!(query.active(), !text.is_empty());
                    assert!(query.matches(text, "", ""));
                    if *text == "λ" {
                        assert!(!query.matches("界", "", ""));
                    }
                }
            }
        }
        for (input, expired, output_failure, expected) in [
            (Some(3), false, false, io::ErrorKind::Interrupted),
            (Some(4), false, false, io::ErrorKind::Interrupted),
            (None, true, false, io::ErrorKind::TimedOut),
            (None, false, true, io::ErrorKind::Other),
            (None, false, false, io::ErrorKind::UnexpectedEof),
        ] {
            let (master, slave) = faction_client_pty::open(24, 40).unwrap();
            let saved = rustix::termios::tcgetattr(&slave).unwrap();
            let mut raw = saved.clone();
            raw.make_raw();
            rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw)
                .unwrap();
            let mut term = LocalTerminal {
                input: slave.into(),
                output: std::fs::OpenOptions::new()
                    .write(!output_failure)
                    .read(output_failure)
                    .open("/dev/null")
                    .unwrap(),
                saved,
            };
            let tty = if input.is_none() && !expired && !output_failure {
                let mut eof =
                    std::fs::File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
                        .unwrap();
                std::io::Seek::seek(&mut eof, std::io::SeekFrom::End(0)).unwrap();
                Some(std::mem::replace(&mut term.input, eof))
            } else {
                None
            };
            let mut master = std::fs::File::from(master);
            if let Some(byte) = input {
                master.write_all(&[byte]).unwrap();
            }
            let error = Screen {
                term: &mut term,
                activity: if expired {
                    Instant::now() - IDLE
                } else {
                    Instant::now()
                },
                unicode: false,
            }
            .search_line()
            .err()
            .expect("input/output failure must leave the prompt");
            if !output_failure {
                assert_eq!(error.kind(), expected);
            }
            if let Some(tty) = tty {
                term.input = tty;
            }
        }
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
    #[cfg(feature = "network-mail")]
    #[test]
    fn online_entry_disables_core_limits_before_any_input() {
        const MARKER: &str = "FACTION209_CORE_LIMIT_TEST_CHILD";
        if std::env::var_os(MARKER).is_some() {
            use rustix::process::{Resource, Rlimit, getrlimit, setrlimit};
            let inherited = getrlimit(Resource::Core);
            assert!(
                inherited.maximum.is_none_or(|n| n >= 8192),
                "test requires an inherited nonzero hard core limit"
            );
            setrlimit(
                Resource::Core,
                Rlimit {
                    current: Some(4096),
                    maximum: Some(8192),
                },
            )
            .unwrap();
            // No terminal/config exists. The guard must precede even terminal
            // initialization, and thus every hidden prompt or file operation.
            assert!(
                run_online(
                    Path::new("/faction209-absent-store"),
                    Path::new("/faction209-absent-config"),
                    false
                )
                .is_err()
            );
            let after = getrlimit(Resource::Core);
            assert_eq!(after.current, Some(0));
            assert_eq!(after.maximum, Some(0));
        } else {
            let child = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "checkpoint_ui::tests::online_entry_disables_core_limits_before_any_input",
                    "--nocapture",
                ])
                .env(MARKER, "1")
                .output()
                .unwrap();
            assert!(
                child.status.success(),
                "isolated early-core-guard regression failed: {}",
                String::from_utf8_lossy(&child.stdout)
            );
        }
    }
}
