use super::*;
impl Client {
    pub(super) fn catchup_show(
        &mut self,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        // Move the small optional workflow out while borrowing opaque authority.
        let State::Catchup(mut desk) = std::mem::replace(&mut self.state, State::Closing) else {
            return Err(Error::Denied);
        };
        let (columns, _) = self.size();
        let frame = match desk.render(board, self.session()?, theme, self.ansi, columns as usize) {
            Ok(f) => f,
            Err(e) => {
                if !desk.recover(&e) {
                    return Err(e);
                }
                desk.render(board, self.session()?, theme, self.ansi, columns as usize)?
            }
        };
        self.state = State::Catchup(desk);
        self.queue(&self.capabilities.frame(&frame, self.ansi));
        self.prompt_started = Instant::now();
        Ok(())
    }
}
