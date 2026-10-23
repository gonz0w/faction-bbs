//! Bounded local wall workflow shared by worker and shared-owner terminals.
//! No I/O, network, shell, secrets, background tasks or terminal negotiation.
#![forbid(unsafe_code)]
use faction_core::{Board, Error, Result, Role, Session, WallConfig, request_id, wall_text};
use faction_terminal::Theme;
use std::fmt::Write;
#[derive(Default)]
enum State {
    #[default]
    Browse,
    Text {
        old: Option<(i64, String)>,
        edition: i64,
        body: String,
    },
    Review {
        old: Option<(i64, String)>,
        edition: i64,
        body: String,
        key: Option<String>,
    },
    Archives,
    Settings,
    Setting(char),
}
#[derive(Default)]
pub struct Desk {
    state: State,
    edition: Option<i64>,
    offset: usize,
    shown: Option<(i64, String)>,
    notice: String,
}
fn approved(b: &mut Board, s: &Session, sysop: bool) -> Result<()> {
    let m = b.whoami(s)?;
    if m.status != faction_core::Status::Approved || (sysop && m.role != Role::Sysop) {
        return Err(Error::Denied);
    }
    Ok(())
}
fn body(out: &mut String, text: &str, columns: usize) {
    let width = columns.clamp(20, 80) - 1;
    for line in text.split('\n') {
        if line.is_empty() {
            out.push('\n');
        }
        for bytes in line.as_bytes().chunks(width) {
            out.push_str(std::str::from_utf8(bytes).expect("validated ASCII wall"));
            out.push('\n');
        }
    }
}
impl Desk {
    pub fn settings() -> Self {
        Self {
            state: State::Settings,
            ..Self::default()
        }
    }
    pub fn render(
        &mut self,
        b: &mut Board,
        s: &Session,
        theme: &Theme,
        ansi: bool,
        columns: usize,
    ) -> Result<String> {
        approved(
            b,
            s,
            matches!(self.state, State::Settings | State::Setting(_)),
        )?;
        let c = b.wall_config(s)?;
        if !c.enabled && !matches!(self.state, State::Settings | State::Setting(_)) {
            return Err(Error::Denied);
        }
        let mut out = faction_terminal::section(theme, &c.name, ansi);
        if !self.notice.is_empty() {
            writeln!(out, "{}", self.notice).unwrap();
            self.notice.clear();
        }
        match &self.state {
            State::Browse => {
                let p = b.wall_page(s, self.edition, self.offset)?;
                self.offset = p.offset;
                writeln!(
                    out,
                    "{} #{}  {}/{}",
                    if p.current { "Current" } else { "Archive" },
                    p.edition,
                    if p.count == 0 { 0 } else { p.offset + 1 },
                    p.count
                )
                .unwrap();
                self.shown = p.note.as_ref().map(|n| (n.id, n.body.clone()));
                if let Some(n) = p.note {
                    writeln!(
                        out,
                        "#{} by {}{}\n{}",
                        n.id,
                        n.author,
                        if n.edited { " (edited)" } else { "" },
                        n.posted
                    )
                    .unwrap();
                    body(&mut out, &n.body, columns);
                } else {
                    out.push_str("No notes yet. Leave a signal.\n");
                }
                out.push_str(
                    "N:next B:back P:post A:archives Q:main\nX:delete E:edit C:current\nWall> ",
                );
            }
            State::Text { body: draft, .. } => {
                writeln!(out, "Local ASCII note: {} bytes / 4 lines", c.note_bytes).unwrap();
                out.push_str(". reviews; .. literal .; /cancel leaves\n/undo removes the last completed line\n");
                body(&mut out, draft, columns);
                out.push_str("Wall text> ");
            }
            State::Review { body: draft, .. } => {
                out.push_str("NOT PUBLISHED - review your note\n");
                body(&mut out, draft, columns);
                out.push_str("[S]end [E]dit [C]ancel: ");
            }
            State::Archives => {
                out.push_str("Retained weekly editions (UTC Monday):\n");
                for (ed, date, count) in b.wall_archives(s)? {
                    writeln!(out, "{ed}: {date} ({count})").unwrap();
                }
                out.push_str("Edition ID, or Enter for current: ");
            }
            State::Settings => {
                writeln!(out, "Enabled: {}  Weekly: {}", c.enabled, c.weekly).unwrap();
                writeln!(
                    out,
                    "Notes {} / bytes {} / days {}",
                    c.max_notes, c.note_bytes, c.retention_days
                )
                .unwrap();
                writeln!(
                    out,
                    "Cooldown {}s / archives {}",
                    c.cooldown_seconds, c.archive_editions
                )
                .unwrap();
                out.push_str("[E]nable [D]isable [N]ame [L]imits\n[W]eekly toggle [A]rchive count [Q]uit\nWall settings> ");
            }
            State::Setting(k) => out.push_str(match k {
                'n' => "Wall name (1-24 ASCII): ",
                'l' => "Limits: notes bytes days cooldown: ",
                'a' => "Retain 0-12 weekly editions: ",
                _ => "Weekly change CLEARS notes/archives. Type YES: ",
            }),
        }
        if out.len() > 8192 {
            return Err(Error::Invalid("wall frame budget"));
        }
        Ok(out)
    }
    /// Only explicit Send creates a request ID. Busy retry retains it and body.
    pub fn feed(&mut self, input: &str, b: &mut Board, s: &Session) -> Result<bool> {
        approved(
            b,
            s,
            matches!(self.state, State::Settings | State::Setting(_)),
        )?;
        let mut c = b.wall_config(s)?;
        if !c.enabled && !matches!(self.state, State::Settings | State::Setting(_)) {
            return Err(Error::Denied);
        }
        match &mut self.state {
            State::Browse => match input.to_ascii_lowercase().as_str() {
                "q" => return Ok(true),
                "n" | "" => self.offset = self.offset.saturating_add(1),
                "b" => self.offset = self.offset.saturating_sub(1),
                "c" => {
                    self.edition = None;
                    self.offset = 0;
                }
                "a" => self.state = State::Archives,
                "p" => {
                    let p = b.wall_page(s, self.edition, self.offset)?;
                    if !p.current {
                        return Err(Error::Invalid("archive is read-only; choose C for current"));
                    }
                    self.state = State::Text {
                        old: None,
                        edition: p.edition,
                        body: String::new(),
                    };
                }
                "x" => {
                    let (id, _) = self.shown.as_ref().ok_or(Error::NotFound)?;
                    b.wall_delete(s, *id)?;
                    if self.edition.is_some() {
                        self.edition = None;
                        self.offset = 0;
                    }
                    self.notice = "Note deleted from the local wall.".into();
                }
                "e" => {
                    let p = b.wall_page(s, self.edition, self.offset)?;
                    let (id, old) = self.shown.clone().ok_or(Error::NotFound)?;
                    if !p.current {
                        return Err(Error::Denied);
                    }
                    self.state = State::Text {
                        old: Some((id, old.clone())),
                        edition: p.edition,
                        body: old,
                    };
                }
                _ => self.notice = "Choose N B P A X E C or Q, then Enter.".into(),
            },
            State::Text {
                old,
                edition,
                body: draft,
            } => {
                if input == "/cancel" {
                    self.state = State::Browse;
                } else if input == "/undo" {
                    if let Some(i) = draft.rfind('\n') {
                        draft.truncate(i)
                    } else {
                        draft.clear()
                    }
                } else if input == "." {
                    wall_text(draft, c.note_bytes as usize)?;
                    self.state = State::Review {
                        old: old.clone(),
                        edition: *edition,
                        body: draft.clone(),
                        key: None,
                    };
                } else {
                    let line = if input == ".." { "." } else { input };
                    let mut next = draft.clone();
                    if !next.is_empty() {
                        next.push('\n')
                    }
                    next.push_str(line);
                    wall_text(&next, c.note_bytes as usize)?;
                    *draft = next;
                    if draft.split('\n').count() == 4 {
                        self.state = State::Review {
                            old: old.clone(),
                            edition: *edition,
                            body: draft.clone(),
                            key: None,
                        };
                    }
                }
            }
            State::Review {
                old,
                edition,
                body,
                key,
            } => match input.to_ascii_lowercase().as_str() {
                "s" => {
                    if let Some((id, expected)) = old {
                        b.wall_edit(s, *id, expected, body)?;
                    } else {
                        if key.is_none() {
                            *key = Some(request_id()?)
                        }
                        b.wall_post(s, *edition, key.as_deref().unwrap(), body)?;
                    }
                    self.notice = "Wall note published locally.".into();
                    self.edition = None;
                    self.offset = usize::MAX;
                    self.state = State::Browse;
                }
                "e" => {
                    self.state = State::Text {
                        old: old.clone(),
                        edition: *edition,
                        body: body.clone(),
                    }
                }
                "c" => self.state = State::Browse,
                _ => (),
            },
            State::Archives => {
                self.edition = if input.is_empty() {
                    None
                } else {
                    Some(
                        input
                            .parse()
                            .map_err(|_| Error::Invalid("edition must be a number"))?,
                    )
                };
                self.offset = 0;
                self.state = State::Browse;
            }
            State::Settings => match input.to_ascii_lowercase().as_str() {
                "q" => return Ok(true),
                "e" | "d" => {
                    c.enabled = input.eq_ignore_ascii_case("e");
                    b.configure_wall(s, &c)?;
                }
                "n" | "l" | "a" | "w" => {
                    self.state = State::Setting(input.as_bytes()[0].to_ascii_lowercase() as char)
                }
                _ => (),
            },
            State::Setting(k) => {
                if !input.is_empty() {
                    match k {
                        'n' => c.name = input.into(),
                        'l' => {
                            let n = input
                                .split_whitespace()
                                .map(str::parse::<u16>)
                                .collect::<std::result::Result<Vec<_>, _>>()
                                .map_err(|_| Error::Invalid("four numeric wall limits required"))?;
                            if n.len() != 4 {
                                return Err(Error::Invalid("four numeric wall limits required"));
                            }
                            c.max_notes = n[0];
                            c.note_bytes = n[1];
                            c.retention_days = n[2];
                            c.cooldown_seconds = n[3];
                        }
                        'a' => {
                            c.archive_editions = input
                                .parse()
                                .map_err(|_| Error::Invalid("invalid archive count"))?
                        }
                        _ => {
                            if input != "YES" {
                                self.state = State::Settings;
                                return Ok(false);
                            }
                            c.weekly = !c.weekly;
                            c.archive_editions = 0;
                        }
                    }
                    b.configure_wall(s, &c)?;
                }
                self.state = State::Settings;
            }
        }
        Ok(false)
    }
    pub fn recover(&mut self, error: &Error) -> bool {
        if matches!(
            error,
            Error::Invalid(_) | Error::NotFound | Error::RequestConflict | Error::Busy
        ) {
            self.notice = format!("Wall: {error}");
            // A vanished archive must not trap the session in an invalid view.
            if matches!(error, Error::NotFound) && matches!(self.state, State::Browse) {
                self.edition = None;
                self.offset = 0;
            }
            true
        } else {
            false
        }
    }
}
/// A temporary presentation copy only when enabled; no retained theme/member cache.
pub fn menu_theme(b: &mut Board, s: &Session, theme: &Theme) -> Result<Option<Theme>> {
    let WallConfig { enabled, .. } = b.wall_config(s)?;
    if enabled {
        let mut t = theme.clone();
        t.set_wall_enabled(true).map_err(Error::Invalid)?;
        Ok(Some(t))
    } else {
        Ok(None)
    }
}
