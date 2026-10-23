//! Connection-bound room editing/checkpoints. No member mail keys or private-mail draft store.
use super::*;
use faction_terminal::composer::{Command, Draft, HELP, PROMPT, Screen, ScreenEvent, Target};
impl Client {
    pub(super) fn post_open(&mut self, board: &mut Board, room: i64) -> faction_core::Result<()> {
        board.unread_cursor(self.session()?, room)?;
        if let Some(saved) = board.room_draft(self.session()?, room)? {
            self.prompt(
                State::DraftChoice(room, saved),
                "Saved room draft: [R]esume [D]elete and start new [Q]uit: ",
            );
        } else {
            self.prompt(
                State::PostReply(room),
                "Reply to post ID (blank for new message): ",
            );
        }
        Ok(())
    }
    pub(super) fn writing_start(
        &mut self,
        board: &mut Board,
        theme: &Theme,
        room: i64,
        reply: Option<i64>,
        subject: String,
    ) -> faction_core::Result<()> {
        self.writing_begin(board, theme, room, reply, subject, None)
    }
    pub(super) fn writing_begin(
        &mut self,
        board: &mut Board,
        theme: &Theme,
        room: i64,
        reply: Option<i64>,
        subject: String,
        saved: Option<faction_core::SavedRoomDraft>,
    ) -> faction_core::Result<()> {
        board.unread_cursor(self.session()?, room)?;
        if let Some(parent) = reply {
            board.reply_subject(self.session()?, room, parent)?;
        }
        self.queue(&self.capabilities.section(
            theme,
            &format!("{} | room #{room}", theme.writing_title()),
            self.ansi,
        ));
        self.queue("Room post only. Save is plaintext on this board, readable by its sysop.\n");
        self.queue(HELP);
        let draft = Draft::from_body(saved.as_ref().map_or("", |d| d.body.as_str()))
            .map_err(Error::Invalid)?;
        self.state = State::Writing {
            room,
            reply,
            subject,
            draft,
            preview: None,
            key: saved.map(|d| d.request_id),
            screen: None,
            confirm: false,
        };
        self.writing_show(board, theme, "write or .screen")
    }
    pub(super) fn writing_show(
        &mut self,
        board: &mut Board,
        theme: &Theme,
        notice: &str,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        let size = self.size();
        let frame = if let State::Writing {
            room,
            reply,
            subject,
            draft,
            key,
            screen,
            ..
        } = &mut self.state
        {
            if let Some(view) = screen {
                view.render(
                    draft,
                    &Target {
                        room: *room,
                        reply: *reply,
                        subject,
                    },
                    size,
                    self.capabilities.unicode,
                    theme,
                    (key.is_some(), notice),
                )
            } else {
                format!(
                    "Draft: {} bytes / {} lines. Nothing sent.\n{PROMPT}",
                    draft.body().len(),
                    draft.lines()
                )
            }
        } else {
            return Err(Error::Denied);
        };
        if frame.len() > 8192 {
            return Err(Error::Invalid("writing frame budget"));
        }
        self.queue(&self.capabilities.frame(&frame, self.ansi));
        self.prompt_started = Instant::now();
        Ok(())
    }
    pub(super) fn writing_input(
        &mut self,
        bytes: &[u8],
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        let size = self.size();
        let mut state = std::mem::replace(&mut self.state, State::Closing);
        let mut command = None;
        let mut changed = false;
        let mut fallback = false;
        if let State::Writing {
            draft,
            screen: Some(view),
            ..
        } = &mut state
        {
            for &byte in bytes {
                match view.feed(draft, byte, Instant::now(), size, self.capabilities.unicode) {
                    Ok(ScreenEvent::Command(c)) => {
                        command = Some(c);
                        break;
                    }
                    Ok(ScreenEvent::Changed) => changed = true,
                    Ok(ScreenEvent::None) => (),
                    Err(_) => {
                        fallback = true;
                        break;
                    }
                }
            }
        } else {
            return Err(Error::Denied);
        }
        self.state = state;
        self.prompt_started = Instant::now();
        if fallback {
            if let State::Writing {
                screen, confirm, ..
            } = &mut self.state
            {
                *screen = None;
                *confirm = true;
            }
            self.queue("Editor input discarded; draft kept. Press Enter to resume line editing: ");
            let _ = termios::tcflush(&self.tty, termios::QueueSelector::IFlush);
            return Ok(());
        }
        if let Some(c) = command {
            let text = match c {
                Command::Save => ".save",
                Command::Preview => ".preview",
                Command::Send => ".send",
                Command::Quote => ".quote",
                Command::Cancel => ".cancel",
                _ => ".line",
            };
            let state = std::mem::replace(&mut self.state, State::Closing);
            self.writing_line(state, text, board, theme)?;
            let _ = termios::tcflush(&self.tty, termios::QueueSelector::IFlush);
        } else if changed {
            self.writing_show(board, theme, "Unsaved edits need Save")?;
        }
        Ok(())
    }
    pub(super) fn writing_line(
        &mut self,
        state: State,
        text: &str,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        let State::Writing {
            room,
            reply,
            subject,
            mut draft,
            preview,
            mut key,
            mut screen,
            confirm,
        } = state
        else {
            return Err(Error::Denied);
        };
        board.unread_cursor(self.session()?, room)?;
        if let Some(parent) = reply {
            board.reply_subject(self.session()?, room, parent)?;
        }
        let command = if confirm {
            if screen.is_some() && text == "SEND" {
                Ok(Command::Send)
            } else {
                Ok(Command::Changed)
            }
        } else if preview.is_some() {
            Ok(Command::Preview)
        } else {
            draft.command(text)
        };
        let mut notice = "Unsaved edits need Save";
        match command {
            Ok(Command::Send) => {
                if screen.is_some() && !confirm {
                    self.prompt(
                        State::Writing {
                            room,
                            reply,
                            subject,
                            draft,
                            preview: None,
                            key,
                            screen,
                            confirm: true,
                        },
                        "Type SEND to post, or Enter to keep editing: ",
                    );
                    return Ok(());
                }
                let key = match key {
                    Some(k) => k,
                    None => faction_core::request_id()?,
                };
                self.queue(&format!("Submission ID: {key}\n"));
                return self.submit(board, room, reply, subject, draft.body().to_owned(), key);
            }
            Ok(Command::Save) => match board.save_room_draft(
                self.session()?,
                room,
                key.as_deref(),
                faction_core::PostDraft {
                    subject: &subject,
                    body: draft.body(),
                    reply_to: reply,
                },
            ) {
                Ok(saved) => {
                    key = Some(saved.request_id);
                    self.queue(
                        "Checkpoint saved on this board for 30 days; edits need Save again.\n",
                    );
                    notice = "checkpoint saved";
                }
                Err(e) => self.queue(&format!("Save failed; previous checkpoint retained: {e}\n")),
            },
            Ok(Command::Cancel) => {
                if let Some(k) = key.as_deref() {
                    board.delete_room_draft(self.session()?, room, k)?;
                }
                self.queue("Draft cancelled; nothing sent.\n");
                self.pause();
                return Ok(());
            }
            Ok(Command::Leave) => {
                self.queue("Writing closed; last explicit checkpoint kept.\n");
                self.pause();
                return Ok(());
            }
            Ok(Command::Screen) => {
                if self.ansi {
                    screen = Some(Screen::at_end(&draft));
                } else {
                    self.queue("Full-screen unavailable; classic line editing remains active.\n");
                }
            }
            Ok(Command::Line) => {
                screen = None;
                self.queue(
                    &self
                        .capabilities
                        .section(theme, theme.writing_title(), self.ansi),
                );
                self.queue("Classic line editing; draft kept.\n");
            }
            Ok(Command::Help) => self.queue(HELP),
            Ok(Command::Quote) => {
                let result = (|| -> faction_core::Result<()> {
                    let parent =
                        reply.ok_or(Error::Invalid("select a reply target before quoting"))?;
                    let rows = board.posts(self.session()?, room, parent - 1)?;
                    let p = rows
                        .first()
                        .filter(|p| p.id == parent)
                        .ok_or(Error::NotFound)?;
                    draft.quote(&p.body).map_err(Error::Invalid)
                })();
                if let Err(e) = result {
                    self.queue(&format!("{e}\n"));
                }
            }
            Ok(Command::Preview) => {
                if preview.is_none() || text.eq_ignore_ascii_case("y") {
                    let page = draft
                        .preview(
                            &Target {
                                room,
                                reply,
                                subject: &subject,
                            },
                            self.size(),
                            preview.unwrap_or(0),
                            self.capabilities.unicode,
                        )
                        .map_err(Error::Invalid)?;
                    self.queue(&self.capabilities.frame(
                        &format!(
                            "{}{}",
                            self.capabilities.section(
                                theme,
                                &format!("{} preview", theme.writing_title()),
                                self.ansi
                            ),
                            page.text
                        ),
                        self.ansi,
                    ));
                    if let Some(next) = page.next {
                        self.prompt(
                            State::Writing {
                                room,
                                reply,
                                subject,
                                draft,
                                preview: Some(next),
                                key,
                                screen,
                                confirm: false,
                            },
                            "Preview more? [y/N]: ",
                        );
                        return Ok(());
                    }
                }
            }
            Ok(Command::Changed) => (),
            Err(e) => self.queue(&format!("{e}\n")),
        }
        self.state = State::Writing {
            room,
            reply,
            subject,
            draft,
            preview: None,
            key,
            screen,
            confirm: false,
        };
        self.writing_show(board, theme, notice)
    }
}
