//! Current approved-room snapshot per frame/action; retain only the ID cursor.
use super::*;
use faction_terminal::rooms::{Choice, Room, View};
impl Client {
    pub(super) fn rooms_show(
        &mut self,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        let rows = board.rooms(self.session()?)?;
        self.rooms_render(&rows, theme)
    }
    fn rooms_render(
        &mut self,
        rows: &[faction_core::Room],
        theme: &Theme,
    ) -> faction_core::Result<()> {
        let (columns, _) = self.size();
        let items: Vec<_> = rows
            .iter()
            .map(|r| Room {
                id: r.id,
                slug: &r.slug,
                title: &r.title,
                unread: r.unread,
            })
            .collect();
        let State::RoomBrowse(view) = &mut self.state else {
            return Err(Error::Denied);
        };
        let frame = view.render(&items, columns, self.ansi, self.capabilities.unicode, theme);
        if frame.len() > 8192 {
            return Err(Error::Invalid("room frame budget"));
        }
        self.queue(&self.capabilities.frame(&frame, self.ansi));
        self.prompt_started = Instant::now();
        Ok(())
    }
    pub(super) fn rooms_key(
        &mut self,
        key: Key,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<bool> {
        let rows = board.rooms(self.session()?)?;
        let items: Vec<_> = rows
            .iter()
            .map(|r| Room {
                id: r.id,
                slug: &r.slug,
                title: &r.title,
                unread: r.unread,
            })
            .collect();
        let State::RoomBrowse(view) = &mut self.state else {
            return Err(Error::Denied);
        };
        let was_card = view.is_card();
        match view.feed(key, &items) {
            Choice::Stay => {
                let transition = was_card != view.is_card();
                self.rooms_render(&rows, theme)?;
                Ok(transition)
            }
            Choice::Main => {
                self.menu(theme, board)?;
                Ok(true)
            }
            Choice::Read { room, new_only } => {
                let after = if new_only {
                    board.unread_cursor(self.session()?, room)?
                } else {
                    0
                };
                self.posts(board, room, after, 0, theme)?;
                Ok(true)
            }
            Choice::Post(room) => {
                board.unread_cursor(self.session()?, room)?;
                self.queue(&self.capabilities.section(
                    theme,
                    &format!("Post in room #{room}"),
                    self.ansi,
                ));
                self.post_open(board, room)?;
                Ok(true)
            }
        }
    }
    pub(super) fn rooms_open(
        &mut self,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.state = State::RoomBrowse(View::default());
        self.rooms_show(board, theme)
    }
}
