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
}
impl<'a> Visit<'a> {
    pub(super) fn new(board: &'a mut Board) -> Self {
        Self {
            board,
            lease: None,
            heartbeat: Instant::now(),
        }
    }
    pub(super) fn ensure(&mut self, session: &Session) -> Result<()> {
        if self.lease.is_none() {
            self.lease = Some(self.board.chat_open(session)?);
            self.heartbeat = Instant::now();
        }
        Ok(())
    }
    pub(super) fn parts(&mut self) -> faction_core::Result<(&mut Board, &ChatLease)> {
        Ok((self.board, self.lease.as_ref().ok_or(Error::Denied)?))
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
