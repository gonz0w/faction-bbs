//! Approved-member local directory; never resolves or connects to a listed host.
use super::*;
impl Client {
    pub(super) fn directory_show(
        &mut self,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        let owner = self.directory.as_ref().ok_or(Error::Denied)?;
        let directory = faction_pages::directory::catalog(board, self.session()?, owner)?;
        let (columns, rows) = self.size();
        let State::Directory(view) = &mut self.state else {
            return Err(Error::Denied);
        };
        view.synchronize(&directory);
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
        if key == Key::Hotkey(b'S') {
            self.state = State::DirectoryDesk(Box::default());
            self.directory_desk_show(board, theme)?;
            return Ok(true);
        }
        let (columns, rows) = self.size();
        let owner = self.directory.as_ref().ok_or(Error::Denied)?;
        let directory = faction_pages::directory::catalog(board, self.session()?, owner)?;
        let State::Directory(view) = &mut self.state else {
            return Err(Error::Denied);
        };
        let was_card = view.is_card();
        if view.feed(key, &directory, columns, rows) {
            self.menu(theme, board)?;
            return Ok(true);
        }
        let transition = was_card != view.is_card();
        self.directory_show(board, theme)?;
        Ok(transition)
    }
    pub(super) fn directory_desk_show(
        &mut self,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        let State::DirectoryDesk(mut desk) = std::mem::replace(&mut self.state, State::Closing)
        else {
            return Err(Error::Denied);
        };
        let (columns, rows) = self.size();
        let frame = match desk.render(
            board,
            self.session()?,
            theme,
            self.ansi,
            (columns as usize, rows as usize),
        ) {
            Ok(frame) => frame,
            Err(e) => {
                if !desk.recover(&e) {
                    return Err(e);
                }
                desk.render(
                    board,
                    self.session()?,
                    theme,
                    self.ansi,
                    (columns as usize, rows as usize),
                )?
            }
        };
        self.state = State::DirectoryDesk(desk);
        self.queue(&self.capabilities.frame(&frame, self.ansi));
        self.prompt_started = Instant::now();
        Ok(())
    }
}
