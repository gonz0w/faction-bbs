//! Synchronous local/worker visit: one existing Board connection and optional lease.
use crate::{Result, TimedInput};
use faction_core::{Board, ChatLease, Error, Session};
use rustix::event::{PollFd, PollFlags, Timespec, poll};
use std::{
    io::{self, Read},
    ops::{Deref, DerefMut},
    time::{Duration, Instant},
};

pub(super) struct Visit<'a> {
    board: &'a mut Board,
    lease: Option<ChatLease>,
    heartbeat: Instant,
    activity: faction_core::VisitActivity,
    published_activity: Option<faction_core::VisitActivity>,
    notifications: faction_pages::notifications::Notifications,
}
impl<'a> Visit<'a> {
    pub(super) fn new(board: &'a mut Board) -> Self {
        Self {
            board,
            lease: None,
            heartbeat: Instant::now(),
            activity: faction_core::VisitActivity::Menu,
            published_activity: None,
            notifications: Default::default(),
        }
    }
    pub(super) fn ensure(&mut self, session: &Session) -> Result<()> {
        if self.lease.is_none() {
            self.lease = Some(self.board.chat_open(session)?);
            self.heartbeat = Instant::now();
        }
        self.publish_activity()?;
        Ok(())
    }
    pub(super) fn set_activity(
        &mut self,
        activity: faction_core::VisitActivity,
    ) -> faction_core::Result<()> {
        self.activity = activity;
        self.publish_activity()
    }
    fn publish_activity(&mut self) -> faction_core::Result<()> {
        if self.published_activity != Some(self.activity)
            && let Some(lease) = &self.lease
        {
            self.board.visit_activity(lease, self.activity)?;
            self.published_activity = Some(self.activity);
        }
        Ok(())
    }
    pub(super) fn parts(
        &mut self,
    ) -> faction_core::Result<(
        &mut Board,
        &ChatLease,
        &mut faction_pages::notifications::Notifications,
    )> {
        Ok((
            self.board,
            self.lease.as_ref().ok_or(Error::Denied)?,
            &mut self.notifications,
        ))
    }
    pub(super) fn poll_notifications(
        &mut self,
        now: Instant,
    ) -> faction_core::Result<Option<bool>> {
        let Some(lease) = self.lease.as_ref() else {
            return Ok(None);
        };
        self.notifications.poll(self.board, lease, now)
    }
    pub(super) fn idle_theme(
        &mut self,
        s: &Session,
        theme: &faction_terminal::Theme,
    ) -> faction_core::Result<faction_terminal::Theme> {
        let mut base = theme.clone();
        base.set_lounge_enabled(false).map_err(Error::Invalid)?;
        base.set_who_enabled(false).map_err(Error::Invalid)?;
        base.set_lounge_notice(false);
        Ok(self.menu_theme(s, &base)?.unwrap_or(base))
    }
    pub(super) fn menu_line(&mut self, prompt: &str, max: usize) -> Result<String> {
        if !self.notifications.enabled() {
            return self.line(prompt, max);
        }
        use faction_terminal::subject::{Edit, feed_limited};
        use std::io::Write;
        let raw = crate::terminal::Raw::input_only()?;
        print!("{prompt}");
        io::stdout().flush()?;
        let mut input = Vec::with_capacity(max);
        let deadline = Instant::now() + Duration::from_secs(120);
        let mut last_input = Instant::now();
        loop {
            self.tick()?;
            if Instant::now() >= deadline {
                return Err(io::Error::from(io::ErrorKind::TimedOut).into());
            }
            if crate::terminal::ready()? {
                let byte = raw.read_byte()?;
                last_input = Instant::now();
                if byte == 3 || byte == 4 {
                    return Err(io::Error::from(io::ErrorKind::Interrupted).into());
                }
                match feed_limited(&mut input, byte, false, max).map_err(crate::invalid)? {
                    Edit::Echo(text) => {
                        print!("{text}");
                        io::stdout().flush()?;
                    }
                    Edit::Submit(text) => {
                        println!();
                        return Ok(text);
                    }
                }
            } else if input.is_empty()
                && last_input.elapsed() >= faction_pages::notifications::INTERVAL
            {
                match self.poll_notifications(Instant::now()) {
                    Ok(Some(true)) => {
                        print!("\n{}\n{prompt}", faction_pages::notifications::PLAIN_NOTICE);
                        io::stdout().flush()?;
                    }
                    Err(Error::Busy) => (),
                    Err(e) => return Err(e.into()),
                    _ => (),
                }
            }
        }
    }
    pub(super) fn menu_theme(
        &mut self,
        session: &Session,
        theme: &faction_terminal::Theme,
    ) -> faction_core::Result<Option<faction_terminal::Theme>> {
        faction_pages::social::menu_theme(self.board, session, self.lease.as_ref(), theme)
    }
    pub(super) fn tick(&mut self) -> faction_core::Result<()> {
        if let Some(lease) = &self.lease
            && self.heartbeat.elapsed() >= Duration::from_secs(10)
        {
            match self.board.chat_heartbeat(lease) {
                Ok(()) => self.heartbeat = Instant::now(),
                Err(Error::Busy) => (),
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
    pub(super) fn close(&mut self) -> faction_core::Result<()> {
        if let Some(lease) = &self.lease {
            self.board.chat_close(lease)?;
            self.lease = None;
        }
        Ok(())
    }
    pub(super) fn line(&mut self, prompt: &str, max: usize) -> Result<String> {
        use std::io::Write;
        print!("{prompt}");
        io::stdout().flush()?;
        crate::bounded_line(
            &mut Input {
                visit: self,
                source: TimedInput {
                    file: crate::tty()?,
                    deadline: Instant::now() + Duration::from_secs(120),
                },
            },
            max,
        )
    }
}
impl Deref for Visit<'_> {
    type Target = Board;
    fn deref(&self) -> &Board {
        self.board
    }
}
impl DerefMut for Visit<'_> {
    fn deref_mut(&mut self) -> &mut Board {
        self.board
    }
}
impl Drop for Visit<'_> {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

pub(super) struct Input<'a, 'b> {
    pub(super) visit: &'a mut Visit<'b>,
    pub(super) source: TimedInput,
}
impl Read for Input<'_, '_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        loop {
            self.visit.tick().map_err(io::Error::other)?;
            let remaining = self
                .source
                .deadline
                .checked_duration_since(Instant::now())
                .ok_or_else(|| io::Error::from(io::ErrorKind::TimedOut))?;
            let wait = remaining.min(Duration::from_millis(100));
            let mut fds = [PollFd::new(&self.source.file, PollFlags::IN)];
            poll(
                &mut fds,
                Some(&Timespec {
                    tv_sec: wait.as_secs() as _,
                    tv_nsec: i64::from(wait.subsec_nanos()),
                }),
            )?;
            if fds[0]
                .revents()
                .intersects(PollFlags::HUP | PollFlags::ERR | PollFlags::NVAL)
            {
                return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
            }
            if fds[0].revents().contains(PollFlags::IN) {
                return self.source.file.read(bytes);
            }
        }
    }
}
