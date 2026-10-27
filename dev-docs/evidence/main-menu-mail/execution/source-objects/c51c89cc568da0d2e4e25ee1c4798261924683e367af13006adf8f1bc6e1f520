//! Per-visit presentation consent and bounded current-signal coalescing.
use faction_core::{Board, ChatLease, Error, Result};
use std::time::{Duration, Instant};

pub const INTERVAL: Duration = Duration::from_secs(2);
pub const PLAIN_NOTICE: &str = "Page waiting: open Social lounge";

pub struct Notifications {
    enabled: bool,
    next_poll: Instant,
    seen: Vec<(i64, u8)>,
}
impl Default for Notifications {
    fn default() -> Self {
        Self {
            enabled: false,
            next_poll: Instant::now(),
            seen: Vec::new(),
        }
    }
}
impl Notifications {
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn due(&self, now: Instant) -> bool {
        self.enabled && now >= self.next_poll
    }
    pub fn command(
        &mut self,
        board: &mut Board,
        lease: &ChatLease,
        input: &str,
    ) -> Result<Option<String>> {
        let (verb, args) = input.split_once(' ').unwrap_or((input, ""));
        if !verb.eq_ignore_ascii_case("/notify") {
            return Ok(None);
        }
        if input.len() > 128 || !input.bytes().all(|b| (32..=126).contains(&b)) {
            return Err(Error::Invalid(
                "bounded ASCII notification command required",
            ));
        }
        board.social_config(lease)?; // current approval/epoch/lease, even for off
        match args.trim() {
            "" => (),
            "on" => {
                if !self.enabled {
                    self.enabled = true;
                    self.seen.clear();
                    self.next_poll = Instant::now() + INTERVAL;
                }
            }
            "off" => {
                self.enabled = false;
                self.seen.clear();
            }
            _ => return Err(Error::Invalid("/notify on|off (this visit)")),
        }
        Ok(Some(format!(
            "Idle page notifications: {} (this visit). /notify on|off.\nOnly an idle main menu; no bells, popups over drafts or forced switches.\n",
            if self.enabled { "on" } else { "off" }
        )))
    }
    /// Called only by an idle main menu. None is unchanged/not due/off; Some
    /// reports a changed current eligible set. No queued events or identity text.
    pub fn poll(
        &mut self,
        board: &mut Board,
        lease: &ChatLease,
        now: Instant,
    ) -> Result<Option<bool>> {
        if !self.due(now) {
            return Ok(None);
        }
        self.next_poll = now + INTERVAL; // also bounds retries under SQLite contention
        let mut keys = Vec::with_capacity(8);
        if board.social_config(lease)?.enabled {
            for page in board.social_pages(lease)? {
                let state = match (page.outgoing, page.state.as_str()) {
                    (false, "pending") => 0,
                    (true, "accepted") => 1,
                    (true, "declined") => 2,
                    _ => continue,
                };
                keys.push((page.id, state));
            }
        }
        if self.seen == keys {
            return Ok(None);
        }
        let added = keys.iter().any(|key| !self.seen.contains(key));
        let cleared = !self.seen.is_empty() && keys.is_empty();
        self.seen = keys;
        Ok(if added {
            Some(true)
        } else if cleared {
            Some(false)
        } else {
            None
        })
    }
}
