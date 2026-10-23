use super::*;
impl Client {
    pub(super) fn ensure_visit(&mut self, board: &mut Board) -> faction_core::Result<()> {
        if self.lease.is_none() {
            self.lease = Some(board.chat_open(self.session()?)?);
            self.heartbeat = Instant::now();
        }
        Ok(())
    }
    pub(super) fn lounge_show(
        &mut self,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        if !board.social_session_config(self.session()?)?.enabled {
            self.queue("Social lounge disabled; returning to main menu.\n");
            return self.menu(theme, board);
        }
        let State::Lounge(mut desk) = std::mem::replace(&mut self.state, State::Closing) else {
            return Err(Error::Denied);
        };
        let frame = desk.render_with_notifications(
            board,
            self.lease()?,
            theme,
            self.ansi,
            &self.notifications,
        )?;
        self.state = State::Lounge(desk);
        self.queue(&self.capabilities.frame(&frame, self.ansi));
        self.prompt_started = Instant::now();
        Ok(())
    }
}

impl Client {
    pub(super) fn idle_notifications(
        &mut self,
        board: &mut Board,
        theme: &Theme,
        now: Instant,
    ) -> faction_core::Result<()> {
        let Some(lease) = self.lease.as_ref() else {
            return Ok(());
        };
        match self.notifications.poll(board, lease, now)? {
            Some(_) if self.ansi => self.menu(theme, board)?,
            Some(true) => {
                self.queue("\n");
                self.queue(faction_pages::notifications::PLAIN_NOTICE);
                self.queue("\nCommand (then Enter): ");
            }
            _ => (),
        }
        Ok(())
    }
}
