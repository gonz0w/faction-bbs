use super::*;
impl Client {
    pub(super) fn find_show(
        &mut self,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        // Move the small optional workflow out while borrowing opaque authority.
        let State::Find(mut desk) = std::mem::replace(&mut self.state, State::Closing) else {
            return Err(Error::Denied);
        };
        let (columns, rows) = self.size();
        let frame = match desk.render(
            board,
            self.session()?,
            theme,
            self.ansi,
            (columns, rows),
            self.capabilities.unicode,
        ) {
            Ok(f) => f,
            Err(e) => {
                if !desk.recover(&e) {
                    return Err(e);
                }
                desk.render(
                    board,
                    self.session()?,
                    theme,
                    self.ansi,
                    (columns, rows),
                    self.capabilities.unicode,
                )?
            }
        };
        self.state = State::Find(desk);
        self.queue(&self.capabilities.frame(&frame, self.ansi));
        self.prompt_started = Instant::now();
        Ok(())
    }
}
