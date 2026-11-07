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
        index: Option<usize>,
    ) -> io::Result<()> {
        let target = Target::prepare(v, exchange.settings, group, now()?)?;
        let selected = target.selected(v, index)?;
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
        self.print("At most 10 seconds; idle wait 2 seconds.\r\nAt most 32 stanzas; no automatic retry.\r\nA write is not delivery or reading.\r\nNo arrival does not prove offline.\r\n[R] Receive briefly\r\n")?;
        if selected.is_some() {
            self.print("[S] Send selected text\r\n")?;
        }
        self.print("[B] Return\r\n")?;
        let mut decoder = Decoder::new();
        let sending = loop {
            match decoder.feed(self.byte()?, Instant::now()) {
                Some(Key::Hotkey(b'B')) => return Ok(()),
                Some(Key::Hotkey(b'R')) => break false,
                Some(Key::Hotkey(b'S')) if selected.is_some() => break true,
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
                && (!sending || current.selected(v, index).ok() == Some(selected))
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
