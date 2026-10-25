use super::*;
impl Client {
    pub(super) fn library_show(
        &mut self,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        let State::Library(mut desk) = std::mem::replace(&mut self.state, State::Closing) else {
            return Err(Error::Denied);
        };
        let catalog = self.library.as_ref().ok_or(Error::Denied)?;
        let frame = match desk.render(
            board,
            self.session()?,
            catalog,
            theme,
            self.ansi,
            self.size().0,
        ) {
            Ok(f) => f,
            Err(e) => {
                if !desk.recover(&e) {
                    return Err(e);
                }
                desk.render(
                    board,
                    self.session()?,
                    catalog,
                    theme,
                    self.ansi,
                    self.size().0,
                )?
            }
        };
        self.state = State::Library(desk);
        self.queue(&self.capabilities.frame(&frame, self.ansi));
        self.prompt_started = Instant::now();
        Ok(())
    }
}
