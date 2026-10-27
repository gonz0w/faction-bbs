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

impl Client {
    pub(super) fn activity(&self) -> Option<faction_core::VisitActivity> {
        use faction_core::VisitActivity as A;
        Some(match self.state {
            State::Menu | State::MailOffer | State::MailParked(_) => A::Menu,
            State::Chat => A::Chat,
            State::Lounge(_) => A::Lounge,
            State::Who(_) => A::Who,
            State::RoomBrowse(_)
            | State::RoomsMore(_)
            | State::RoomId(_)
            | State::ReadMore(..)
            | State::PostRoom
            | State::DraftChoice(..)
            | State::PostReply(..)
            | State::PostSubject(..)
            | State::PostBody(..)
            | State::Writing { .. }
            | State::PostRetry { .. } => A::Rooms,
            State::DoorPending | State::DoorMenu | State::DoorInput | State::Classic => A::Doors,
            State::Files
            | State::NativeFiles
            | State::FileUploadKey { .. }
            | State::FileDownloadKey(_) => A::Files,
            State::Terminal
            | State::GraphicsProbe(_)
            | State::Graphics(..)
            | State::Probe(_)
            | State::Sysop
            | State::MembersMore(_)
            | State::MemberStatus(_)
            | State::RoomSlug
            | State::RoomTitle(_) => A::Settings,
            State::Guide(_) => A::Menu,
            State::Directory(_) | State::DirectoryDesk(_) => A::Directory,
            State::Gallery(_) => A::Gallery,
            State::Library(_) => A::Library,
            State::Wall(_)
            | State::Pages(_)
            | State::Polls(_)
            | State::Catchup(_)
            | State::Find(_)
            | State::Gatherings(_) => A::Community,
            _ => return self.published_activity,
        })
    }
    pub(super) fn who_show(
        &mut self,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        if !board.social_session_config(self.session()?)?.enabled {
            self.queue("Social presence disabled; returning to main menu.\n");
            return self.menu(theme, board);
        }
        board.visit_activity(self.lease()?, faction_core::VisitActivity::Who)?;
        self.published_activity = Some(faction_core::VisitActivity::Who);
        let State::Who(mut desk) = std::mem::replace(&mut self.state, State::Closing) else {
            return Err(Error::Denied);
        };
        let frame = desk.render(board, self.lease()?, theme, self.ansi)?;
        self.state = State::Who(desk);
        self.queue(&self.capabilities.frame(&frame, self.ansi));
        self.prompt_started = Instant::now();
        Ok(())
    }
}
