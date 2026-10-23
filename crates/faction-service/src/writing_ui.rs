//! Connection-owned, bounded room draft. No persistence or member mail keys.
use super::*;
use faction_terminal::composer::{Command, Draft, HELP, PROMPT};
impl Client {
    pub(super) fn writing_start(
        &mut self,
        board: &mut Board,
        theme: &Theme,
        room: i64,
        reply: Option<i64>,
        subject: String,
    ) -> faction_core::Result<()> {
        board.unread_cursor(self.session()?, room)?;
        self.queue(&self.capabilities.section(
            theme,
            &format!("Writing desk | room #{room}"),
            self.ansi,
        ));
        self.queue("Room post only; visible to this board. Your draft is not saved.\n");
        self.queue(HELP);
        self.prompt(
            State::Writing {
                room,
                reply,
                subject,
                draft: Draft::default(),
                preview: None,
            },
            PROMPT,
        );
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
        } = state
        else {
            return Err(Error::Denied);
        };
        board.unread_cursor(self.session()?, room)?;
        if let Some(parent) = reply {
            board.reply_subject(self.session()?, room, parent)?;
        }
        let command = if preview.is_some() {
            Ok(Command::Preview)
        } else {
            draft.command(text)
        };
        match command {
            Ok(Command::Send) => {
                let key = faction_core::request_id()?;
                self.queue(&format!("Submission ID: {key}\n"));
                return self.submit(board, room, reply, subject, draft.body().to_owned(), key);
            }
            Ok(Command::Cancel) => {
                self.queue("Draft cancelled; nothing sent.\n");
                self.pause();
                return Ok(());
            }
            Ok(Command::Help) => self.queue(HELP),
            Ok(Command::Quote) => {
                let result = (|| -> faction_core::Result<()> {
                    let parent =
                        reply.ok_or(Error::Invalid("select a reply target before quoting"))?;
                    let rows = board.posts(self.session()?, room, parent - 1)?;
                    let post = rows
                        .first()
                        .filter(|p| p.id == parent)
                        .ok_or(Error::NotFound)?;
                    draft.quote(&post.body).map_err(Error::Invalid)
                })();
                if let Err(e) = result {
                    self.queue(&format!("{e}\n"));
                }
            }
            Ok(Command::Preview) => {
                if preview.is_none() || text.eq_ignore_ascii_case("y") {
                    let (columns, rows) = self.size();
                    let page = draft
                        .preview(
                            &faction_terminal::composer::Target {
                                room,
                                reply,
                                subject: &subject,
                            },
                            (columns, rows),
                            preview.unwrap_or(0),
                            self.capabilities.unicode,
                        )
                        .map_err(Error::Invalid)?;
                    self.queue(&self.capabilities.frame(
                        &format!(
                                "{}{}",
                                self.capabilities
                                    .section(theme, "Writing desk preview", self.ansi),
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
        self.queue(&format!(
            "Draft: {} bytes / {} lines. Nothing sent.\n",
            draft.body().len(),
            draft.lines()
        ));
        self.prompt(
            State::Writing {
                room,
                reply,
                subject,
                draft,
                preview: None,
            },
            PROMPT,
        );
        Ok(())
    }
}
