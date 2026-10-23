//! Approved-member local directory; never resolves or connects to a listed host.
use super::*;
impl Client {
    pub(super) fn directory_show(
        &mut self,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        let State::Directory(view) = &self.state else {
            return Err(Error::Denied);
        };
        let directory = self.directory.as_ref().ok_or(Error::Denied)?;
        let (columns, rows) = self.size();
        let frame = directory.render(view, columns, rows, self.ansi, theme);
        if frame.len() > 8192 {
            return Err(Error::Invalid("directory frame budget"));
        }
        self.queue(&self.capabilities.frame(&frame, self.ansi));
        if !self.ansi {
            self.queue("Directory command: ");
        }
        self.prompt_started = Instant::now();
        Ok(())
    }
    pub(super) fn directory_key(
        &mut self,
        key: Key,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<bool> {
        self.approved(board)?;
        let (columns, rows) = self.size();
        let directory = self.directory.as_ref().ok_or(Error::Denied)?;
        let State::Directory(view) = &mut self.state else {
            return Err(Error::Denied);
        };
        let was_card = view.is_card();
        if view.feed(key, directory, columns, rows) {
            self.menu(theme, board)?;
            return Ok(true);
        }
        let transition = was_card != view.is_card();
        self.directory_show(board, theme)?;
        Ok(transition)
    }
}
