//! Trusted-local attachment desk. Never called with a remote stream.
use super::*;
use faction_client::{
    attachment_save::SaveError,
    mail_blob_transfer::{self as transfer, InvitedHomes, ReceivedSelection, Spool},
    mail_network::Settings,
};
use faction_omemo::attachment::Descriptor;

const MAX_CALLS: usize = 65;
const TRANSFER_TIME: Duration = Duration::from_secs(60);
#[derive(Debug, Eq, PartialEq)]
enum Step {
    Continue,
    Complete,
    Stop,
}
/// Facts from successful SDK replies only. An error may have made additional
/// durable progress; it never silently retries and never resets this budget.
struct Foreground {
    calls: usize,
    offset: u64,
    total: u64,
}
impl Foreground {
    fn new(total: u64) -> Self {
        Self {
            calls: 0,
            offset: 0,
            total,
        }
    }
    fn accept(&mut self, offset: u64, complete: bool) -> Step {
        self.calls += 1;
        if self.calls > MAX_CALLS
            || offset > self.total
            || offset < self.offset
            || (complete && offset != self.total)
            || (!complete && offset <= self.offset)
        {
            return Step::Stop;
        }
        self.offset = offset;
        if complete {
            Step::Complete
        } else if self.calls == MAX_CALLS {
            Step::Stop
        } else {
            Step::Continue
        }
    }
}
async fn foreground_step(
    remaining: Duration,
    sdk: impl std::future::Future<Output = Result<transfer::Report, transfer::Error>>,
    cancellation: impl std::future::Future<Output = io::Result<()>>,
) -> io::Result<Result<transfer::Report, transfer::Error>> {
    if remaining.is_zero() {
        return Ok(Err(transfer::Error::Transport(
            faction_xmpp::Error::Timeout,
        )));
    }
    tokio::select! {
        result = tokio::time::timeout(remaining, sdk) => Ok(result.unwrap_or(Err(transfer::Error::Transport(faction_xmpp::Error::Timeout)))),
        cancelled = cancellation => { cancelled?; Err(invalid()) }
    }
}
fn save_failure(error: &transfer::Error) -> &'static str {
    match error {
        transfer::Error::Save(SaveError::PublicationUncertain) => {
            "Publication may have completed.\r\nInspect the destination before retrying.\r\nDo not remove it to simulate rollback.\r\n"
        }
        transfer::Error::Save(SaveError::BeforePublication) => {
            "Save failed before publication.\r\nCheck destination and private parent.\r\nExisting files are never overwritten.\r\n"
        }
        _ => "Verification/save unavailable.\r\nNo verified file was published.\r\n",
    }
}
fn operation_id(value: &str) -> io::Result<[u8; 16]> {
    if value.len() != 32
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(invalid());
    }
    let bytes = unhex(&format!("{value}{}", "0".repeat(32)))?;
    let mut id = [0; 16];
    id.copy_from_slice(&bytes[..16]);
    if id == [0; 16] {
        return Err(invalid());
    }
    Ok(id)
}
fn managed(d: &Descriptor) -> bool {
    d.validate().is_ok() && d.locator.starts_with("urn:faction:blob:1:")
}
struct PeerSnapshot {
    address: Zeroizing<String>,
    device: u32,
    identity: [u8; 32],
}
impl PeerSnapshot {
    fn prepare(v: &CheckpointVault, capacity: bool, timestamp: u64) -> io::Result<Self> {
        let s = v.inspection()?;
        let p = s.peer.ok_or_else(invalid)?;
        if !s.attachments_enabled
            || !s.started
            || s.history_only
            || p.revoked
            || timestamp < s.created
            || timestamp > s.expires
            || (capacity
                && (v.history()?.len() >= MAX_HISTORY || v.inspect_outbox()?.len() >= MAX_OUTBOX))
        {
            return Err(invalid());
        }
        Ok(Self {
            address: Zeroizing::new(p.address.into()),
            device: p.device,
            identity: *p.identity,
        })
    }
    fn recheck(&self, v: &CheckpointVault, capacity: bool, timestamp: u64) -> io::Result<()> {
        let current = Self::prepare(v, capacity, timestamp)?;
        if self.address != current.address
            || self.device != current.device
            || self.identity != current.identity
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn matches(&self, row: &faction_client::mail_checkpoint::Record) -> bool {
        row.address.as_str() == self.address.as_str()
            && row.device == self.device
            && row.identity == self.identity
    }
}
fn received(
    v: &CheckpointVault,
    selected: ReceivedSelection,
) -> io::Result<&faction_client::mail_checkpoint::Record> {
    let row = v.history()?.get(selected.index).ok_or_else(invalid)?;
    if row.direction != Direction::Received
        || row.received_id() != Some(selected.frame)
        || !row.attachment().is_some_and(managed)
    {
        return Err(invalid());
    }
    Ok(row)
}
#[derive(Clone, Copy)]
enum TransferSelection {
    Upload([u8; 16]),
    Fetch(ReceivedSelection),
}
fn completion_message(selected: TransferSelection) -> &'static str {
    match selected {
        TransferSelection::Upload(_) => {
            "Foreground transfer complete.\r\nPending ciphertext retained; a write\r\ndoes not prove delivery or reading.\r\n"
        }
        TransferSelection::Fetch(_) => {
            "Foreground transfer complete.\r\nFetched ciphertext verified locally.\r\nNo read receipt or automatic save.\r\n"
        }
    }
}
impl Screen<'_> {
    /// Wrap complete inert values to classic cells without retaining another
    /// unzeroized private copy or interpreting metadata as terminal instructions.
    fn attachment_field(&mut self, label: &str, value: &str) -> io::Result<()> {
        self.print(label)?;
        self.print("\r\n")?;
        let mut cells = 0;
        for scalar in value.chars() {
            let mut scratch = Zeroizing::new([0; 4]);
            let text = inert(scalar.encode_utf8(&mut *scratch), self.unicode);
            let width = faction_terminal::text::width(&text);
            if cells + width > 39 {
                self.print("\r\n")?;
                cells = 0;
            }
            self.print(&text)?;
            cells += width;
        }
        self.print("\r\n")
    }
    pub(super) fn attachment_details(&mut self, descriptor: &Descriptor) -> io::Result<()> {
        self.attachment_field("Private attachment name:", &descriptor.name)?;
        self.attachment_field("Media type:", &descriptor.media_type)?;
        self.print(&format!("Plaintext bytes: {}\r\n", descriptor.size))?;
        self.print(if managed(descriptor) {
            "Managed ciphertext attachment.\r\n"
        } else {
            "Unsupported locator; no fetch or save.\r\n"
        })
    }
    fn attachment_peer(&mut self, peer: &PeerSnapshot) -> io::Result<()> {
        self.attachment_field("CURRENT verified recipient:", &peer.address)?;
        self.print(&format!("Device: {}\r\n", peer.device))?;
        self.attachment_field("CURRENT Ed25519 identity:", &hex(&peer.identity))
    }
    fn attachment_unavailable(&mut self, v: &CheckpointVault) -> io::Result<()> {
        v.inspection()?;
        self.print("Attachment action unavailable.\r\nCheck selection, current peer, expiry,\r\nretained limits and private storage.\r\n")
    }
    pub(super) fn attachment_desk(
        &mut self,
        v: &mut CheckpointVault,
        settings: &Settings,
        root: &Path,
    ) -> io::Result<()> {
        let mut decoder = Decoder::new();
        loop {
            self.print(
                "\r\n---------------------------------------\r\n[LOCAL PRIVATE ATTACHMENTS]\r\n",
            )?;
            self.print(if v.inspection()?.attachments_enabled {
                "Attachment checkpoints: enabled\r\n"
            } else {
                "Attachment checkpoints: disabled\r\n"
            })?;
            self.print("Manual actions; no background transfer.\r\n[E] Enable attachments\r\n[C] Compose file  [U] Upload pending\r\n[R] Received attachment  [B] Back\r\n")?;
            let key = loop {
                if let Some(Key::Hotkey(key)) = decoder.feed(self.byte()?, Instant::now())
                    && b"ECURB".contains(&key)
                {
                    break key;
                }
            };
            match key {
                b'B' => return Ok(()),
                b'E' => self.attachment_enable(v)?,
                b'C' => self.attachment_compose(v, root)?,
                b'U' => self.attachment_upload(v, settings, root)?,
                b'R' => self.attachment_received(v, settings, root)?,
                _ => unreachable!(),
            }
            decoder = Decoder::new();
        }
    }
    fn attachment_enable(&mut self, v: &mut CheckpointVault) -> io::Result<()> {
        if v.inspection()?.attachments_enabled {
            return self.print("Attachments already enabled.\r\n");
        }
        self.print("Explicit FACTCP05 checkpoint upgrade.\r\nOlder clients cannot open this format.\r\nTrust, history, read/archive decisions\r\nand pending mail remain. No connection.\r\nBlobs have separate retention/backups.\r\nLimits: one 1 MiB file per message;\r\neight spool objects; seven-day expiry.\r\nIncomplete preparation can leave opaque\r\norphans; no inventory or automatic sweep.\r\nKeep a history-only backup first.\r\n")?;
        rustix::termios::tcflush(&self.term.input, rustix::termios::QueueSelector::IFlush)?;
        if self
            .line("Type enable to adopt FACTCP05: ", 32, false)?
            .as_str()
            != "enable"
        {
            return self.print("Attachment upgrade cancelled.\r\n");
        }
        v.enable_attachments()?;
        self.print("Attachment checkpoints enabled.\r\n")
    }
    fn attachment_compose(&mut self, v: &mut CheckpointVault, root: &Path) -> io::Result<()> {
        let peer = match PeerSnapshot::prepare(v, true, now()?) {
            Ok(p) => p,
            Err(_) => return self.attachment_unavailable(v),
        };
        self.attachment_peer(&peer)?;
        let source = self.compose_line("Absolute LOCAL source path (blank cancels): ", 1024)?;
        if source.is_empty() {
            return self.print("Attachment compose cancelled.\r\n");
        }
        let name = self.compose_line("Private display name (255 UTF-8 bytes): ", 255)?;
        let media = self.line("Media type (127 ASCII bytes): ", 127, false)?;
        let subject = self.compose_line("Subject (128 UTF-8 bytes): ", 128)?;
        let body = self.compose_line("Body (1024 UTF-8 bytes, one line): ", 1024)?;
        // Current authority and capacity are checked before loading any plaintext.
        if peer.recheck(v, true, now()?).is_err() {
            return self.attachment_unavailable(v);
        }
        let bytes = match faction_client::attachment_source::load(Path::new(source.as_str())) {
            Ok(bytes) => bytes,
            Err(_) => return self.print("Local source refused; nothing queued.\r\n"),
        };
        // Validate private display metadata before any spool creation or crypto.
        let preview = Descriptor {
            name,
            media_type: media,
            size: bytes.len() as u64,
            hash: Zeroizing::new([0; 32]),
            key: Zeroizing::new([0; 32]),
            iv: Zeroizing::new([0; 12]),
            locator: Zeroizing::new(format!("urn:faction:blob:1:{}", "0".repeat(64))),
        };
        if preview.validate().is_err() {
            return self.print("Attachment metadata refused.\r\nNothing queued.\r\n");
        }
        self.print("Review LOCAL attachment intent:\r\n")?;
        self.attachment_peer(&peer)?;
        self.attachment_field("Selected local source:", &source)?;
        self.attachment_details(&preview)?;
        self.attachment_field("Subject:", &subject)?;
        self.attachment_field("Body:", &body)?;
        rustix::termios::tcflush(&self.term.input, rustix::termios::QueueSelector::IFlush)?;
        if self
            .line("Type queue to encrypt and persist attachment: ", 32, false)?
            .as_str()
            != "queue"
        {
            return self.print("Attachment cancelled; nothing queued.\r\n");
        }
        let timestamp = now()?;
        if peer.recheck(v, true, timestamp).is_err() {
            return self.attachment_unavailable(v);
        }
        let mut operation = [0; 16];
        getrandom::fill(&mut operation).map_err(|_| invalid())?;
        let mut spool = match Spool::open(&root.join("attachments")) {
            Ok(s) => s,
            Err(_) => return self.attachment_unavailable(v),
        };
        let selected = transfer::Selection {
            name: &preview.name,
            media_type: &preview.media_type,
            bytes: &bytes,
            subject: &subject,
            body: &body,
            sent_at: timestamp,
        };
        match transfer::prepare(v, &mut spool, operation, selected, timestamp) {
            Ok(_) => {
                self.print(
                    "Attachment queued durably.\r\nNo upload or delivery attempted.\r\nOperation: ",
                )?;
                self.print(&hex(&operation))?;
                self.print("\r\n")
            }
            Err(_) => {
                v.inspection()?;
                self.print("Attachment queue stopped.\r\nInspect retained mail before retrying.\r\nA bounded opaque spool orphan may remain.\r\n")
            }
        }
    }
    fn attachment_upload(
        &mut self,
        v: &mut CheckpointVault,
        settings: &Settings,
        root: &Path,
    ) -> io::Result<()> {
        let peer = match PeerSnapshot::prepare(v, false, now()?) {
            Ok(p) => p,
            Err(_) => return self.attachment_unavailable(v),
        };
        let pending = v
            .inspect_outbox()?
            .into_iter()
            .map(|p| p.operation)
            .collect::<Vec<_>>();
        let mut count = 0;
        for row in v.history()? {
            if row.direction == Direction::Sent
                && row.attachment().is_some_and(managed)
                && pending.contains(&row.operation())
            {
                count += 1;
                self.print("Pending operation:\r\n")?;
                self.print(&hex(&row.operation()))?;
                self.print("\r\n")?;
            }
        }
        if count == 0 {
            return self.print("No pending managed attachment.\r\n");
        }
        let value = self.line("Full operation ID (blank cancels): ", 32, false)?;
        if value.is_empty() {
            return Ok(());
        }
        let operation = match operation_id(&value) {
            Ok(id) => id,
            Err(_) => return self.attachment_unavailable(v),
        };
        let Some(row) = v.history()?.iter().find(|r| {
            r.direction == Direction::Sent
                && r.operation() == operation
                && pending.contains(&operation)
                && r.attachment().is_some_and(managed)
        }) else {
            return self.attachment_unavailable(v);
        };
        if !peer.matches(row) {
            return self.attachment_unavailable(v);
        }
        let descriptor = row.attachment().ok_or_else(invalid)?;
        let total = descriptor.size + 16;
        self.attachment_peer(&peer)?;
        self.attachment_details(descriptor)?;
        self.attachment_transfer(
            v,
            settings,
            root,
            TransferSelection::Upload(operation),
            total,
            &peer,
            None,
        )
    }
    fn attachment_received(
        &mut self,
        v: &mut CheckpointVault,
        settings: &Settings,
        root: &Path,
    ) -> io::Result<()> {
        let mut count = 0;
        for (index, row) in v.history()?.iter().enumerate() {
            if row.direction == Direction::Received
                && let Some(d) = row.attachment()
            {
                count += 1;
                self.print(&format!(
                    "Received record {} / {} bytes\r\n",
                    index + 1,
                    d.size
                ))?;
                self.print(&inbox::compact(&d.name, self.unicode, 39))?;
                self.print("\r\n")?;
            }
        }
        if count == 0 {
            return self.print("No received attachment.\r\n");
        }
        let value = self.line("Original record number (blank cancels): ", 2, false)?;
        if value.is_empty() {
            return Ok(());
        }
        let index = match value.parse::<usize>().ok().and_then(|n| n.checked_sub(1)) {
            Some(n) => n,
            None => return self.attachment_unavailable(v),
        };
        let Some(row) = v.history()?.get(index) else {
            return self.attachment_unavailable(v);
        };
        let Some(frame) = row.received_id() else {
            return self.attachment_unavailable(v);
        };
        let Some(d) = row.attachment() else {
            return self.attachment_unavailable(v);
        };
        self.attachment_field("Authenticated sender:", &row.address)?;
        self.print(&format!(
            "Device: {} / record {}\r\n",
            row.device,
            index + 1
        ))?;
        self.attachment_details(d)?;
        if !managed(d) {
            return Ok(());
        }
        let selected = ReceivedSelection { index, frame };
        let mut decoder = Decoder::new();
        loop {
            self.print(
                "[F] Fetch/retry  [V] Verify locally\r\n[S] Save verified file  [B] Back\r\n",
            )?;
            let key = loop {
                if let Some(Key::Hotkey(key)) = decoder.feed(self.byte()?, Instant::now())
                    && b"FVSB".contains(&key)
                {
                    break key;
                }
            };
            match key {
                b'B' => return Ok(()),
                b'F' => {
                    let peer = match PeerSnapshot::prepare(v, false, now()?) {
                        Ok(p) => p,
                        Err(_) => {
                            self.attachment_unavailable(v)?;
                            continue;
                        }
                    };
                    let row = received(v, selected)?;
                    if !peer.matches(row) {
                        self.attachment_unavailable(v)?;
                        continue;
                    }
                    let total = row.attachment().ok_or_else(invalid)?.size + 16;
                    let sender_home = row.address.split_once('@').ok_or_else(invalid)?.1;
                    let own_home = settings.account.split_once('@').ok_or_else(invalid)?.1;
                    let invited = if sender_home != own_home {
                        self.attachment_field("Invited sender-home route:", sender_home)?;
                        self.print("Traffic uses YOUR configured pinned home.\r\nYour password is never sent to this route.\r\n")?;
                        rustix::termios::tcflush(
                            &self.term.input,
                            rustix::termios::QueueSelector::IFlush,
                        )?;
                        if self
                            .line("Type invite for this fetch only: ", 32, false)?
                            .as_str()
                            != "invite"
                        {
                            self.print("Fetch cancelled; no connection.\r\n")?;
                            continue;
                        }
                        InvitedHomes::new(&[sender_home]).map_err(|_| invalid())?
                    } else {
                        InvitedHomes::new(&[]).map_err(|_| invalid())?
                    };
                    self.attachment_transfer(
                        v,
                        settings,
                        root,
                        TransferSelection::Fetch(selected),
                        total,
                        &peer,
                        Some(&invited),
                    )?;
                }
                b'V' | b'S' => self.attachment_local(v, root, selected, key == b'S')?,
                _ => unreachable!(),
            }
            decoder = Decoder::new();
        }
    }
    fn attachment_local(
        &mut self,
        v: &CheckpointVault,
        root: &Path,
        selected: ReceivedSelection,
        save: bool,
    ) -> io::Result<()> {
        if received(v, selected).is_err() {
            return self.attachment_unavailable(v);
        }
        // Verification/save are explicit local actions; navigation never opens it.
        let mut spool = match Spool::open(&root.join("attachments")) {
            Ok(s) => s,
            Err(_) => return self.attachment_unavailable(v),
        };
        if !save {
            return match transfer::verify_fetched(
                v,
                &mut spool,
                selected.index,
                selected.frame,
                now()?,
            ) {
                Ok(blob) => self.print(&format!(
                    "Verified locally: {} plaintext bytes.\r\nNo save or open performed.\r\n",
                    blob.as_bytes().len()
                )),
                Err(_) => {
                    v.inspection()?;
                    self.print("Local verification unavailable.\r\nCheck fetched bytes, retention and\r\nlocal identity; no file was published.\r\n")
                }
            };
        }
        self.print("Select a NEW full absolute destination.\r\nExisting final parent must be owned 0700.\r\nOutput is 0600; no overwrite or open.\r\n")?;
        let path = self.compose_line("Absolute save path (blank cancels): ", 1024)?;
        if path.is_empty() {
            return self.print("Save cancelled; no file published.\r\n");
        }
        self.attachment_field("Review save destination:", &path)?;
        rustix::termios::tcflush(&self.term.input, rustix::termios::QueueSelector::IFlush)?;
        if self
            .line("Type save to verify and publish: ", 32, false)?
            .as_str()
            != "save"
        {
            return self.print("Save cancelled; no file published.\r\n");
        }
        match transfer::save_fetched(
            v,
            &mut spool,
            selected.index,
            selected.frame,
            now()?,
            Path::new(path.as_str()),
        ) {
            Ok(()) => self.print(
                "Verified file saved privately.\r\nNo open, extraction or read receipt.\r\n",
            ),
            Err(error) => {
                v.inspection()?;
                self.print(save_failure(&error))
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn attachment_transfer(
        &mut self,
        v: &mut CheckpointVault,
        settings: &Settings,
        root: &Path,
        selected: TransferSelection,
        total: u64,
        peer: &PeerSnapshot,
        invited: Option<&InvitedHomes>,
    ) -> io::Result<()> {
        peer.recheck(v, false, now()?)?;
        if settings.account != v.legacy()?.home {
            return self.attachment_unavailable(v);
        }
        rustix::termios::tcflush(&self.term.input, rustix::termios::QueueSelector::IFlush)?;
        if self
            .line("Type transfer to connect to YOUR home: ", 32, false)?
            .as_str()
            != "transfer"
        {
            return self.print("Transfer cancelled; no connection.\r\n");
        }
        self.print("Foreground: at most 65 SDK calls / 60s.\r\nStops on first error; retry is explicit.\r\nCtrl-C/D cancels and locks.\r\n")?;
        let password = self.line("OWN HOME account password (hidden): ", 128, true)?;
        // Secret input cannot substitute for the reviewed selection/authority.
        peer.recheck(v, false, now()?)?;
        match selected {
            TransferSelection::Fetch(s) => {
                if !peer.matches(received(v, s)?) {
                    return self.attachment_unavailable(v);
                }
            }
            TransferSelection::Upload(id) => {
                let row = v
                    .history()?
                    .iter()
                    .find(|r| {
                        r.direction == Direction::Sent
                            && r.operation() == id
                            && r.attachment().is_some_and(managed)
                    })
                    .ok_or_else(invalid)?;
                if !peer.matches(row) || !v.inspect_outbox()?.iter().any(|p| p.operation == id) {
                    return self.attachment_unavailable(v);
                }
            }
        }
        let mut spool = match Spool::open(&root.join("attachments")) {
            Ok(s) => s,
            Err(_) => return self.attachment_unavailable(v),
        };
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let mut progress = Foreground::new(total);
        let started = Instant::now();
        loop {
            let Some(remaining) = TRANSFER_TIME.checked_sub(started.elapsed()) else {
                self.print("Foreground time budget reached.\r\nRetry only after inspecting local state.\r\n")?;
                break;
            };
            let result = runtime.block_on(foreground_step(
                remaining,
                async {
                    match selected {
                        TransferSelection::Upload(operation) => {
                            transfer::upload_cycle(
                                v,
                                &mut spool,
                                settings,
                                &password,
                                operation,
                                now().map_err(|_| transfer::Error::State)?,
                            )
                            .await
                        }
                        TransferSelection::Fetch(selection) => {
                            transfer::fetch_cycle(
                                v,
                                &mut spool,
                                settings,
                                &password,
                                invited.ok_or(transfer::Error::Route)?,
                                selection,
                                now().map_err(|_| transfer::Error::State)?,
                            )
                            .await
                        }
                    }
                },
                self.network_cancel(),
            ))?;
            match result {
                Ok(report) => {
                    let step = progress.accept(report.offset, report.complete);
                    self.print(&format!(
                        "Confirmed ciphertext bytes: {}\r\nSDK call: {} / 65; writes: {}\r\n",
                        report.offset, progress.calls, report.sent
                    ))?;
                    if step == Step::Complete {
                        self.print(completion_message(selected))?;
                        break;
                    }
                    if step == Step::Stop {
                        self.print("Stopped: progress or call bound.\r\nInspect state before an explicit retry.\r\n")?;
                        break;
                    }
                }
                _ => {
                    v.inspection()?;
                    self.print("Transfer stopped; no automatic retry.\r\nFurther bytes may already be durable.\r\nInspect state before an explicit retry.\r\n")?;
                    break;
                }
            }
        }
        drop(password);
        self.activity = Instant::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn foreground_requires_strict_progress_and_caps_calls() {
        let mut run = Foreground::new(1_048_592);
        assert_eq!(run.accept(16_384, false), Step::Continue);
        assert_eq!(run.accept(16_384, false), Step::Stop);
        let mut run = Foreground::new(1_048_592);
        for n in 1..=64 {
            assert_eq!(run.accept(n * 16_384, false), Step::Continue);
        }
        assert_eq!(run.accept(1_048_592, true), Step::Complete);
        assert_eq!(run.accept(1_048_592, true), Step::Stop);
    }
    #[test]
    fn foreground_rejects_wrong_bounds_and_regressions() {
        for (offset, complete) in [(0, false), (17, false), (15, true)] {
            assert_eq!(Foreground::new(16).accept(offset, complete), Step::Stop);
        }
        assert_eq!(Foreground::new(16).accept(16, true), Step::Complete);
        let mut run = Foreground::new(32);
        assert_eq!(run.accept(16, false), Step::Continue);
        assert_eq!(run.accept(15, false), Step::Stop);
    }
    #[test]
    fn exact_operation_input_rejects_partial_or_zero_identity() {
        assert!(operation_id("123").is_err());
        assert!(operation_id(&"00".repeat(16)).is_err());
        assert!(operation_id(&"AA".repeat(16)).is_err());
        assert_eq!(operation_id(&"01".repeat(16)).unwrap(), [1; 16]);
    }

    // Native startup permanently suppresses the hard core limit. Keep these
    // real-vault contracts in isolated children so the separate early-entry guard
    // can still prove that production suppresses a previously nonzero limit.
    fn isolated(name: &str) -> bool {
        const MARKER: &str = "FACTION263_ATTACHMENT_UI_TEST_CHILD";
        if std::env::var(MARKER).ok().as_deref() == Some(name) {
            return false;
        }
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", name, "--nocapture"])
            .env(MARKER, name)
            .output()
            .unwrap();
        assert!(
            child.status.success(),
            "isolated attachment UI contract failed: {} {}",
            String::from_utf8_lossy(&child.stdout),
            String::from_utf8_lossy(&child.stderr)
        );
        true
    }
    const TIME: u64 = 1_792_886_400;
    const PASS: &[u8] = b"synthetic attachment desk password";
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            use std::os::unix::fs::DirBuilderExt;
            let mut id = [0; 16];
            getrandom::fill(&mut id).unwrap();
            let root = std::env::temp_dir()
                .canonicalize()
                .unwrap()
                .join(format!("faction263-ui-test-OWNED-{}", hex(&id)));
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&root)
                .unwrap();
            std::fs::write(root.join("OWNED"), b"Owned synthetic attachment desk test").unwrap();
            Self(root)
        }
        fn vault(&self, name: &str, home: &str, device: u32) -> CheckpointVault {
            CheckpointVault::create(
                Store::at(&self.0.join(name), true).unwrap(),
                Data::new(home.into()).unwrap(),
                PASS,
                device,
                TIME,
            )
            .unwrap()
            .0
        }
        fn pair(&self) -> (CheckpointVault, CheckpointVault) {
            let mut a = self.vault("alice", "alice@home.test", 101);
            let mut b = self.vault("bob", "bob@away.test", 202);
            a.enable_attachments().unwrap();
            b.enable_attachments().unwrap();
            let ab = a.bundle().unwrap();
            let bb = b.bundle().unwrap();
            a.enroll(
                "bob@away.test",
                202,
                &format!("ed25519:{}", hex(&bb.identity)),
                TIME,
            )
            .unwrap();
            b.enroll(
                "alice@home.test",
                101,
                &format!("ed25519:{}", hex(&ab.identity)),
                TIME,
            )
            .unwrap();
            a.start(&bb, TIME).unwrap();
            let wire = a.queue([1; 16], "text", "body", TIME, TIME).unwrap();
            b.receive(&wire, TIME).unwrap();
            a.acknowledge([1; 16]).unwrap();
            (a, b)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            assert_eq!(
                std::fs::read(self.0.join("OWNED")).unwrap(),
                b"Owned synthetic attachment desk test"
            );
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn descriptor(locator: &str) -> Descriptor {
        Descriptor {
            name: Zeroizing::new("private.bin".into()),
            media_type: Zeroizing::new("application/octet-stream".into()),
            size: 5,
            hash: Zeroizing::new([1; 32]),
            key: Zeroizing::new([2; 32]),
            iv: Zeroizing::new([3; 12]),
            locator: Zeroizing::new(locator.into()),
        }
    }
    #[test]
    fn peer_preflight_refuses_unprepared_expired_changed_capacity_and_quarantine() {
        if isolated(
            "checkpoint_ui::attachments::tests::peer_preflight_refuses_unprepared_expired_changed_capacity_and_quarantine",
        ) {
            return;
        }
        let f = Fixture::new();
        let unprepared = f.vault("unprepared", "test@home.test", 303);
        assert!(PeerSnapshot::prepare(&unprepared, true, TIME).is_err());
        let (mut a, _b) = f.pair();
        let snapshot = PeerSnapshot::prepare(&a, true, TIME).unwrap();
        assert!(snapshot.recheck(&a, true, TIME).is_ok());
        assert!(snapshot.recheck(&a, true, TIME - 1).is_err());
        assert!(
            snapshot
                .recheck(&a, true, a.inspection().unwrap().expires + 1)
                .is_err()
        );
        let mut changed = PeerSnapshot::prepare(&a, true, TIME).unwrap();
        changed.identity[0] ^= 1;
        assert!(changed.recheck(&a, true, TIME).is_err());
        changed = PeerSnapshot::prepare(&a, true, TIME).unwrap();
        changed.device += 1;
        assert!(changed.recheck(&a, true, TIME).is_err());
        changed = PeerSnapshot::prepare(&a, true, TIME).unwrap();
        changed.address = Zeroizing::new("other@away.test".into());
        assert!(changed.recheck(&a, true, TIME).is_err());
        for n in 2..=9 {
            a.queue([n; 16], "pending", "body", TIME, TIME).unwrap();
        }
        assert!(PeerSnapshot::prepare(&a, true, TIME).is_err());
        assert!(PeerSnapshot::prepare(&a, false, TIME).is_ok());
        a.discard_pending().unwrap();
        for n in 10..=64 {
            a.queue([n; 16], "retained", "body", TIME, TIME).unwrap();
            a.acknowledge([n; 16]).unwrap();
        }
        assert_eq!(a.history().unwrap().len(), 64);
        assert!(PeerSnapshot::prepare(&a, true, TIME).is_err());
        let backup = f.0.join("backup");
        a.backup(&backup).unwrap();
        let historical =
            CheckpointVault::unlock(Store::at(&backup, false).unwrap(), PASS, false).unwrap();
        assert!(PeerSnapshot::prepare(&historical, false, TIME).is_err());
        a.revoke().unwrap();
        assert!(snapshot.recheck(&a, false, TIME).is_err());
        assert!(!f.0.join("alice/attachments").exists());
    }
    #[test]
    fn received_selection_needs_exact_original_frame_and_managed_attachment() {
        if isolated(
            "checkpoint_ui::attachments::tests::received_selection_needs_exact_original_frame_and_managed_attachment",
        ) {
            return;
        }
        let f = Fixture::new();
        let (mut a, mut b) = f.pair();
        let managed_locator = format!("urn:faction:blob:1:{}", "ab".repeat(32));
        let d = descriptor(&managed_locator);
        let wire = b
            .queue_attachment([2; 16], "attachment", "body", &d, TIME, TIME)
            .unwrap();
        a.receive(&wire, TIME).unwrap();
        let selected = ReceivedSelection {
            index: 1,
            frame: a.history().unwrap()[1].received_id().unwrap(),
        };
        assert!(received(&a, selected).is_ok());
        assert!(
            received(
                &a,
                ReceivedSelection {
                    index: 0,
                    ..selected
                }
            )
            .is_err()
        );
        assert!(
            received(
                &a,
                ReceivedSelection {
                    index: 64,
                    ..selected
                }
            )
            .is_err()
        );
        assert!(
            received(
                &a,
                ReceivedSelection {
                    frame: [0; 32],
                    ..selected
                }
            )
            .is_err()
        );
        let wire = b.queue([3; 16], "text", "body", TIME, TIME).unwrap();
        a.receive(&wire, TIME).unwrap();
        assert!(
            received(
                &a,
                ReceivedSelection {
                    index: 2,
                    frame: a.history().unwrap()[2].received_id().unwrap()
                }
            )
            .is_err()
        );
        let unsupported = descriptor("https://home.test/cipher.bin");
        let wire = b
            .queue_attachment([4; 16], "unsupported", "body", &unsupported, TIME, TIME)
            .unwrap();
        a.receive(&wire, TIME).unwrap();
        assert!(
            received(
                &a,
                ReceivedSelection {
                    index: 3,
                    frame: a.history().unwrap()[3].received_id().unwrap()
                }
            )
            .is_err()
        );
        for locator in [
            "urn:faction:blob:1:ab",
            &format!("urn:faction:blob:1:{}", "AB".repeat(32)),
        ] {
            assert!(!managed(&descriptor(locator)));
        }
        assert!(
            PeerSnapshot::prepare(&a, false, TIME)
                .unwrap()
                .matches(received(&a, selected).unwrap())
        );
        assert!(a.read_summary().unwrap().unread.is_none());
        assert!(!f.0.join("alice/attachments").exists());
    }
    #[test]
    fn save_phase_messages_keep_uncertain_publication_actionable() {
        let before = save_failure(&transfer::Error::Save(SaveError::BeforePublication));
        let uncertain = save_failure(&transfer::Error::Save(SaveError::PublicationUncertain));
        assert!(before.contains("before publication"));
        assert!(uncertain.contains("Inspect the destination before retrying"));
        assert!(!uncertain.contains("No verified file was published"));
        assert_ne!(before, uncertain);
    }
    #[test]
    fn foreground_deadline_and_cancellation_drop_one_pending_sdk_future() {
        use std::{cell::Cell, rc::Rc};
        struct Dropped(Rc<Cell<usize>>);
        impl Drop for Dropped {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let drops = Rc::new(Cell::new(0));
        let result = runtime.block_on(foreground_step(
            Duration::from_millis(5),
            {
                let guard = Dropped(drops.clone());
                async move {
                    let _guard = guard;
                    std::future::pending::<Result<transfer::Report, transfer::Error>>().await
                }
            },
            std::future::pending::<io::Result<()>>(),
        ));
        assert!(matches!(
            result,
            Ok(Err(transfer::Error::Transport(
                faction_xmpp::Error::Timeout
            )))
        ));
        assert_eq!(drops.get(), 1);
        let result = runtime.block_on(foreground_step(
            Duration::from_secs(1),
            {
                let guard = Dropped(drops.clone());
                async move {
                    let _guard = guard;
                    std::future::pending::<Result<transfer::Report, transfer::Error>>().await
                }
            },
            async {
                Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "local cancellation",
                ))
            },
        ));
        assert_eq!(result.err().unwrap().kind(), io::ErrorKind::Interrupted);
        assert_eq!(drops.get(), 2);
    }

    #[test]
    fn exhausted_foreground_budget_never_polls_another_sdk_call() {
        use std::cell::Cell;
        let polls = Cell::new(0);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = runtime.block_on(foreground_step(
            Duration::ZERO,
            async {
                polls.set(polls.get() + 1);
                Ok(transfer::Report {
                    offset: 16,
                    complete: true,
                    requests: 1,
                    sent: 0,
                })
            },
            std::future::pending::<io::Result<()>>(),
        ));
        assert!(matches!(
            result,
            Ok(Err(transfer::Error::Transport(
                faction_xmpp::Error::Timeout
            )))
        ));
        assert_eq!(
            polls.get(),
            0,
            "expired UI budget must not start another SDK call"
        );
    }

    #[test]
    fn completion_separates_pending_upload_from_received_fetch() {
        let upload = completion_message(TransferSelection::Upload([1; 16]));
        let fetch = completion_message(TransferSelection::Fetch(ReceivedSelection {
            index: 0,
            frame: [2; 32],
        }));
        assert!(upload.contains("Pending ciphertext retained"));
        assert!(fetch.contains("Fetched ciphertext verified locally"));
        assert!(!fetch.contains("pending"));
        assert!(fetch.contains("No read receipt or automatic save"));
    }
}
