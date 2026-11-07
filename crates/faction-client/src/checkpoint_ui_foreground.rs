//! Reviewed member-device foreground authority; no persisted UI state.
use super::*;
use faction_client::mail_network::{ForegroundReport, ForegroundWrite, Settings};

#[cfg(test)]
pub(super) type ControlledForeground<'a> = dyn Fn(
        &mut CheckpointVault,
        &str,
        faction_client::mail_network::ForegroundPeer<'_>,
        faction_client::mail_network::ForegroundAction,
    ) -> ForegroundReport
    + 'a;

pub(super) struct Target {
    pub address: Zeroizing<String>,
    pub device: u32,
    pub identity: [u8; 32],
}
impl Target {
    pub fn pending(&self, v: &CheckpointVault) -> io::Result<Vec<usize>> {
        let records = v.history()?.iter().collect::<Vec<_>>();
        Ok(self.pending_records(&records, &v.inspect_outbox()?))
    }
    pub(super) fn pending_records(
        &self,
        records: &[&faction_client::mail_checkpoint::Record],
        pending: &[faction_client::mail_checkpoint::OutboxInspection<'_>],
    ) -> Vec<usize> {
        records
            .iter()
            .enumerate()
            .filter_map(|(index, record)| {
                (record.direction == Direction::Sent
                    && record.attachment().is_none()
                    && record.address == self.address.as_str()
                    && record.device == self.device
                    && record.identity == self.identity
                    && records
                        .iter()
                        .filter(|other| other.operation() == record.operation())
                        .count()
                        == 1
                    && pending
                        .iter()
                        .filter(|item| item.operation == record.operation())
                        .count()
                        == 1
                    && pending.iter().any(|item| {
                        item.operation == record.operation()
                            && item.address == self.address.as_str()
                            && item.device == self.device
                    }))
                .then_some(index)
            })
            .collect()
    }
    pub fn prepare(
        v: &CheckpointVault,
        settings: &Settings,
        group: usize,
        time: u64,
    ) -> io::Result<Self> {
        let archive = v.archive_checkpoint(time)?;
        let state = v.inspection()?;
        let peer = state.peer.ok_or_else(invalid)?;
        let records = v.history()?;
        let record = records.get(group).ok_or_else(invalid)?;
        if archive.home != settings.account
            || archive.after.is_none()
            || !state.started
            || record.address != peer.address
            || record.device != peer.device
            || record.identity != *peer.identity
        {
            return Err(invalid());
        }
        Ok(Self {
            address: Zeroizing::new(peer.address.into()),
            device: peer.device,
            identity: *peer.identity,
        })
    }
    pub fn selected(
        &self,
        v: &CheckpointVault,
        index: Option<usize>,
    ) -> io::Result<Option<[u8; 16]>> {
        let records = v.history()?;
        let Some(record) = index.and_then(|index| records.get(index)) else {
            return Ok(None);
        };
        if record.direction != Direction::Sent
            || record.attachment().is_some()
            || record.address != self.address.as_str()
            || record.device != self.device
            || record.identity != self.identity
        {
            return Ok(None);
        }
        Ok(v.inspect_outbox()?
            .iter()
            .any(|item| {
                item.operation == record.operation()
                    && item.address == self.address.as_str()
                    && item.device == self.device
            })
            .then_some(record.operation()))
    }
}

pub(super) fn report(report: &ForegroundReport) -> String {
    let mut text = String::from(match report.write {
        ForegroundWrite::NotAttempted => "No selected text write attempted.\r\n",
        ForegroundWrite::Unconfirmed => "Selected text write is unconfirmed.\r\n",
        ForegroundWrite::Completed => "Selected text local write completed.\r\n",
    });
    text.push_str(&format!(
        "Accepted on this device: {}.\r\nDuplicates: {}.\r\n",
        report.accepted, report.duplicates
    ));
    if report.accepted == 0 {
        text.push_str("No new arrival observed.\r\nThis does not prove offline,\r\nnondelivery or a complete home inbox.\r\n");
    }
    if report.error.is_some() {
        text.push_str("Brief exchange stopped.\r\nEarlier accepted mail remains saved.\r\n");
    }
    text.push_str("Pending ciphertext is retained.\r\nA write is not delivery or reading.\r\n");
    text
}

impl Screen<'_> {
    fn queued_text(&mut self, v: &CheckpointVault, target: &Target) -> io::Result<Option<usize>> {
        let entries = target
            .pending(v)?
            .into_iter()
            .map(|index| inbox::Entry {
                index,
                status: inbox::Status::Pending,
            })
            .collect();
        let mut list = inbox::Inbox::new(entries);
        if let Some(index) = list.eligible().iter().map(|entry| entry.index).min() {
            list.select_index(index);
        }
        let mut model = conversation::Conversation::new(&list);
        let mut decoder = Decoder::new();
        rustix::termios::tcflush(&self.term.input, rustix::termios::QueueSelector::IFlush)?;
        loop {
            self.print("\r\n[LOCAL QUEUED TEXT]\r\n")?;
            let (page, pages) = model.page();
            self.print(&format!(
                "Page {page}/{pages}; selection {}/{}\r\n",
                if model.len() == 0 {
                    0
                } else {
                    model.position() + 1
                },
                model.len()
            ))?;
            if model.len() == 0 {
                self.print("No eligible queued ordinary text.\r\n")?;
            }
            for entry in model.visible() {
                let record = &v.history()?[entry.index];
                self.print(&format!(
                    "{} {:02} ",
                    if model.selected() == Some(*entry) {
                        '>'
                    } else {
                        ' '
                    },
                    entry.index + 1
                ))?;
                self.print(&inbox::compact(&record.subject, self.unicode, 34))?;
                self.print("\r\n")?;
            }
            self.print("[Up/Down J/K] Select  [N/P] Page\r\n[Enter/O] Review selected text\r\n[B] Return\r\n")?;
            loop {
                match decoder.feed(self.byte()?, Instant::now()) {
                    Some(Key::Up | Key::Hotkey(b'K')) => model.up(),
                    Some(Key::Down | Key::Hotkey(b'J')) => model.down(),
                    Some(Key::Hotkey(b'N')) => model.next(),
                    Some(Key::Hotkey(b'P')) => model.previous(),
                    Some(Key::Hotkey(b'B')) => return Ok(None),
                    Some(Key::Enter | Key::Hotkey(b'O')) => {
                        if let Some(entry) = model.selected() {
                            if target.pending(v)?.contains(&entry.index) {
                                return Ok(Some(entry.index));
                            }
                            self.print("Selection changed; no text selected.\r\n")?;
                            return Ok(None);
                        }
                    }
                    _ => continue,
                }
                break;
            }
        }
    }
    fn brief_actions(&mut self, selected: bool) -> io::Result<()> {
        self.print("At most 10 seconds; idle wait 2 seconds.\r\nAt most 32 stanzas; no automatic retry.\r\nA write is not delivery or reading.\r\nNo arrival does not prove offline.\r\n[R] Receive briefly\r\n")?;
        if selected {
            self.print("[S] Send selected text\r\n")?;
        }
        self.print("[Q] Choose queued text\r\n[B] Return\r\n")
    }
    /// Full inert review with the existing borrowed, bounded body pager.
    fn brief_review(&mut self, text: &str) -> io::Result<bool> {
        let mut pages = body::Body::new(text, self.unicode);
        let count = pages.page_count();
        let mut number = 0;
        while let Some(page) = pages.next_page() {
            number += 1;
            self.print(&format!("[LOCAL BRIEF REVIEW] Page {number}/{count}\r\n"))?;
            for row in page.rows {
                self.print("| ")?;
                self.print(&row.text)?;
                self.print("\r\n")?;
            }
            if number < count {
                self.print("[Enter] Next page  [B] Return\r\n")?;
                let mut decoder = Decoder::new();
                loop {
                    match decoder.feed(self.byte()?, Instant::now()) {
                        Some(Key::Enter) => break,
                        Some(Key::Hotkey(b'B')) => return Ok(false),
                        _ => {}
                    }
                }
            }
        }
        Ok(true)
    }
    pub(super) fn brief_exchange(
        &mut self,
        v: &mut CheckpointVault,
        exchange: &HomeExchange<'_>,
        group: usize,
        mut index: Option<usize>,
    ) -> io::Result<()> {
        let target = Target::prepare(v, exchange.settings, group, now()?)?;
        let mut selected = target.selected(v, index)?;
        let mut picked = false;
        self.clear()?;
        rustix::termios::tcflush(&self.term.input, rustix::termios::QueueSelector::IFlush)?;
        self.print("[LOCAL BRIEF EXCHANGE]\r\n")?;
        let review = Zeroizing::new(format!(
            "Home: {}\nPeer: {}\nPeer device: {}\nCurrent identity:\n{}",
            exchange.settings.account,
            target.address.as_str(),
            target.device,
            hex(&target.identity)
        ));
        if !self.brief_review(&review)? {
            return Ok(());
        }
        self.brief_actions(selected.is_some())?;
        let mut decoder = Decoder::new();
        let sending = loop {
            match decoder.feed(self.byte()?, Instant::now()) {
                Some(Key::Hotkey(b'B')) => return Ok(()),
                Some(Key::Hotkey(b'R')) => break false,
                Some(Key::Hotkey(b'S')) if selected.is_some() => break true,
                Some(Key::Hotkey(b'Q')) => {
                    if let Some(chosen) = self.queued_text(v, &target)? {
                        index = Some(chosen);
                        selected = target.selected(v, index)?;
                        if selected.is_some() {
                            picked = true;
                            break true;
                        }
                        self.print("Selection changed; no text selected.\r\n")?;
                    }
                    self.brief_actions(selected.is_some())?;
                    decoder = Decoder::new();
                }
                _ => {}
            }
        };
        if sending {
            let record = &v.history()?[index.ok_or_else(invalid)?];
            let review = Zeroizing::new(format!(
                "Send selected text only.\nRetained record: {}\nOperation:\n{}\nSubject: {}",
                index.unwrap() + 1,
                hex(&selected.unwrap()),
                record.subject.as_str()
            ));
            if !self.brief_review(&review)? {
                return Ok(());
            }
        } else {
            self.print("Receive briefly; no text release.\r\n")?;
        }
        rustix::termios::tcflush(&self.term.input, rustix::termios::QueueSelector::IFlush)?;
        if self
            .line("Type exchange to connect, or Enter: ", 32, false)?
            .as_str()
            != "exchange"
        {
            return Ok(());
        }
        let password = self.line(
            "HOME account password (hidden; never sent to a BBS): ",
            128,
            true,
        )?;
        let current = Target::prepare(v, exchange.settings, group, now()?);
        if !current.is_ok_and(|current| {
            current.address == target.address
                && current.device == target.device
                && current.identity == target.identity
                && (!sending
                    || (current.selected(v, index).ok() == Some(selected)
                        && (!picked
                            || index.is_some_and(|index| {
                                current
                                    .pending(v)
                                    .is_ok_and(|pending| pending.contains(&index))
                            }))))
        }) {
            v.inspection()?;
            self.print("Selection changed; nothing connected.\r\n")?;
            return self.pause();
        }
        self.print("Connecting directly to your home.\r\nCtrl-C/D locks and cancels.\r\n")?;
        let peer = faction_client::mail_network::ForegroundPeer {
            address: &target.address,
            device: target.device,
            identity: target.identity,
        };
        let action = if sending {
            faction_client::mail_network::ForegroundAction::SendSelected(
                selected.ok_or_else(invalid)?,
            )
        } else {
            faction_client::mail_network::ForegroundAction::ReceiveOnly
        };
        #[cfg(test)]
        let controlled = exchange
            .foreground
            .map(|cycle| cycle(v, &password, peer, action));
        #[cfg(not(test))]
        let controlled: Option<ForegroundReport> = None;
        let result = if let Some(result) = controlled {
            result
        } else {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            let timestamp = now()?;
            runtime.block_on(async {
                tokio::select! {
                    report = faction_client::mail_network::foreground_cycle(v, exchange.settings, &password, peer, action, timestamp) => Ok(report),
                    cancelled = self.network_cancel() => { cancelled?; Err(invalid()) }
                }
            })?
        };
        drop(password);
        self.activity = Instant::now();
        if v.inspection().is_err() {
            self.print(
                "Local checkpoint unavailable.\r\nLock and reopen the canonical store.\r\n",
            )?;
            return Err(invalid());
        }
        self.print(&report(&result))?;
        self.pause()
    }
}
