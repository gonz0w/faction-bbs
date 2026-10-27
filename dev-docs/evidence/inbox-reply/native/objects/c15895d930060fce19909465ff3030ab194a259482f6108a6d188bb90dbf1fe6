//! Shared local discussion discovery; no host I/O or private-mail authority.
#![forbid(unsafe_code)]
use faction_core::{
    Board, DiscussionField as Field, DiscussionQuery as Query, Error, Result, Role, Session,
};
use faction_terminal::{Theme, subject::preview_with};
use std::{
    collections::VecDeque,
    fmt::Write,
    hash::{Hash, Hasher},
};
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Search,
    Bookmarks,
    Export,
}
#[derive(Clone, Copy)]
enum Pick {
    Open,
    Add,
    Remove,
}
struct View {
    id: i64,
    offset: usize,
    next: Option<usize>,
    stamp: Option<u64>,
    previous: VecDeque<usize>,
}
enum State {
    Browse,
    Text,
    Room,
    Author,
    Field,
    Pick(Pick),
    Review { i: i64, add: bool },
    View(View),
    Settings,
    Name,
}
pub struct Desk {
    state: State,
    mode: Mode,
    query: Option<Query>,
    draft: Query,
    start: Option<i64>,
    cursor: i64,
    previous: VecDeque<i64>,
    next: Option<i64>,
    notice: String,
}
impl Default for Desk {
    fn default() -> Self {
        Self {
            state: State::Browse,
            mode: Mode::Search,
            query: None,
            draft: Query {
                text: String::new(),
                room: None,
                author: String::new(),
                field: Field::Both,
            },
            start: None,
            cursor: 0,
            previous: VecDeque::new(),
            next: None,
            notice: String::new(),
        }
    }
}
fn id(input: &str) -> Result<i64> {
    input
        .parse::<i64>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or(Error::Invalid("positive post ID required"))
}
fn remember<T>(list: &mut VecDeque<T>, value: T) {
    if list.len() == 32 {
        list.pop_front();
    }
    list.push_back(value);
}
impl Desk {
    pub fn settings() -> Self {
        Self {
            state: State::Settings,
            ..Self::default()
        }
    }
    fn admin(&self) -> bool {
        matches!(self.state, State::Settings | State::Name)
    }
    fn refresh(&mut self, b: &mut Board, s: &Session) -> Result<()> {
        let top = b.discussion_start(s)?;
        self.start = Some(top);
        self.cursor = top;
        self.previous.clear();
        self.next = None;
        Ok(())
    }
    pub fn render(
        &mut self,
        b: &mut Board,
        s: &Session,
        theme: &Theme,
        ansi: bool,
        size: (u16, u16),
        unicode: bool,
    ) -> Result<String> {
        let c = b.discussion_config(s)?;
        if self.admin() && b.whoami(s)?.role != Role::Sysop {
            return Err(Error::Denied);
        }
        let width = (size.0 as usize).clamp(20, 80) - 1;
        let mut out = faction_terminal::section(theme, &c.name, ansi);
        if !self.admin() && !c.enabled {
            out.push_str("Find is disabled. Q:main\nFind> ");
            return Ok(out);
        }
        if !self.notice.is_empty() {
            writeln!(out, "{}", preview_with(&self.notice, width, unicode)).unwrap();
        }
        if !self.admin() && self.start.is_none() {
            self.refresh(b, s)?;
        }
        match &mut self.state {
            State::Browse => {
                self.next = None;
                if self.mode == Mode::Export {
                    out.push_str("COPYABLE REFERENCE EXPORT / this page\n");
                    let (text, next) = b.discussion_bookmark_export(s, self.cursor)?;
                    out.push_str(&text);
                    self.next = next;
                } else {
                    let page = if self.mode == Mode::Bookmarks {
                        out.push_str("Your bookmarks / newest post first\n");
                        Some(b.discussion_bookmarks(s, self.cursor)?)
                    } else if let Some(q) = &self.query {
                        writeln!(
                            out,
                            "Literal: {}",
                            preview_with(&q.text, width.saturating_sub(9), unicode)
                        )
                        .unwrap();
                        let p = b.discussion_search(s, q, self.cursor)?;
                        writeln!(out, "Newest first; inspected {} (<=100).", p.scanned).unwrap();
                        Some(p)
                    } else {
                        out.push_str("F starts a local discussion search.\n");
                        None
                    };
                    if let Some(page) = page {
                        self.next = page.next;
                        if page.hits.is_empty() {
                            out.push_str("No matching items in this scan.\n");
                        }
                        for h in page.hits {
                            writeln!(
                                out,
                                "{}",
                                preview_with(
                                    &format!("#{} / room {} / {}", h.id, h.room, h.author),
                                    width,
                                    unicode
                                )
                            )
                            .unwrap();
                            writeln!(out, "{}", preview_with(&h.subject, width, unicode)).unwrap();
                            if !c.compact {
                                writeln!(out, " {}", preview_with(&h.snippet, width - 1, unicode))
                                    .unwrap();
                            }
                        }
                    }
                }
                out.push_str("F:search B:bookmarks N/P:scan R:refresh\nJ:open A:add D:remove X:export\nQ:main\nFind> ");
            }
            State::Text => out.push_str("Literal text (1-64 UTF-8 bytes, /cancel): "),
            State::Room => out.push_str("Room ID (blank all, /cancel): "),
            State::Author => out.push_str("Exact author (blank all, /cancel): "),
            State::Field => out.push_str("Search S:subject B:body A:both: "),
            State::Pick(kind) => out.push_str(match kind {
                Pick::Open => "Open post ID (blank cancels): ",
                Pick::Add => "Bookmark post ID (blank cancels): ",
                Pick::Remove => "Remove post ID (blank cancels): ",
            }),
            State::Review { i, add } => {
                let item = b.discussion_item(s, *i)?;
                writeln!(
                    out,
                    "{} #{}: {}",
                    if *add {
                        "Save bookmark"
                    } else {
                        "Remove bookmark"
                    },
                    i,
                    preview_with(&item.hit.subject, width.saturating_sub(25), unicode)
                )
                .unwrap();
                out.push_str(
                    "Reference only; local operator-readable.\nS:save C:cancel\nBookmark review> ",
                );
            }
            State::View(v) => {
                let item = b.discussion_item(s, v.id)?;
                let mut hash = std::collections::hash_map::DefaultHasher::new();
                (&item.body, &item.hit.subject, &item.posted).hash(&mut hash);
                let stamp = hash.finish();
                if v.stamp.is_some_and(|old| old != stamp) {
                    v.offset = 0;
                    v.previous.clear();
                    out.push_str("Source changed; restarted safely.\n");
                }
                v.stamp = Some(stamp);
                let page = faction_terminal::Message {
                    room: item.hit.room,
                    id: v.id,
                    author: &item.hit.author,
                    reply_to: item.reply,
                    body: &item.body,
                }
                .page_with_capabilities(
                    size.0,
                    size.1,
                    v.offset,
                    faction_terminal::MessageDetails {
                        subject: &item.hit.subject,
                        posted: &item.posted,
                    },
                    unicode,
                )
                .map_err(Error::Invalid)?;
                out.push_str(&page.text);
                v.next = page.next;
                out.push_str(
                    "READ ONLY / room unread unchanged\nN/P:page A:bookmark Q:results\nSource> ",
                );
            }
            State::Settings => {
                writeln!(
                    out,
                    "{} / {}",
                    if c.enabled { "Enabled" } else { "Disabled" },
                    if c.compact { "compact" } else { "cards" }
                )
                .unwrap();
                out.push_str("E:toggle N:name L:layout Q:main\nFind settings> ");
            }
            State::Name => out.push_str("Find label (1-24 ASCII): "),
        }
        if out.len() > 8192 {
            return Err(Error::Invalid("discussion frame exceeds budget"));
        }
        Ok(out)
    }
    /// True exits to the host main menu. All other work stays in this bounded desk.
    pub fn feed(&mut self, input: &str, b: &mut Board, s: &Session) -> Result<bool> {
        if input.len() > 128 || input.chars().any(char::is_control) {
            return Err(Error::Invalid(
                "bounded printable discussion input required",
            ));
        }
        let c = b.discussion_config(s)?;
        if self.admin() && b.whoami(s)?.role != Role::Sysop {
            return Err(Error::Denied);
        }
        if !self.admin() && !c.enabled {
            return Ok(true);
        }
        let cmd = input.to_ascii_lowercase();
        self.notice.clear();
        if input == "/cancel"
            && matches!(
                self.state,
                State::Text | State::Room | State::Author | State::Field
            )
        {
            self.state = State::Browse;
            return Ok(false);
        }
        match &mut self.state {
            State::Browse => match cmd.as_str() {
                "q" => return Ok(true),
                "f" => self.state = State::Text,
                "b" => {
                    self.mode = Mode::Bookmarks;
                    self.refresh(b, s)?;
                }
                "x" => {
                    self.mode = Mode::Export;
                    self.refresh(b, s)?;
                }
                "r" => self.refresh(b, s)?,
                "n" => {
                    if let Some(next) = self.next {
                        remember(&mut self.previous, self.cursor);
                        self.cursor = next;
                    }
                }
                "p" => self.cursor = self.previous.pop_back().unwrap_or(self.start.unwrap_or(0)),
                "j" => self.state = State::Pick(Pick::Open),
                "a" => self.state = State::Pick(Pick::Add),
                "d" => self.state = State::Pick(Pick::Remove),
                _ => return Err(Error::Invalid("choose a displayed Find command")),
            },
            State::Text => {
                let q = Query {
                    text: input.into(),
                    ..self.draft.clone()
                };
                q.validate()?;
                self.draft.text = input.into();
                self.state = State::Room;
            }
            State::Room => {
                self.draft.room = if input.is_empty() {
                    None
                } else {
                    Some(id(input)?)
                };
                self.state = State::Author;
            }
            State::Author => {
                let mut q = self.draft.clone();
                q.author = cmd;
                q.validate()?;
                self.draft = q;
                self.state = State::Field;
            }
            State::Field => {
                self.draft.field = match cmd.as_str() {
                    "s" => Field::Subject,
                    "b" => Field::Body,
                    "a" => Field::Both,
                    _ => return Err(Error::Invalid("use S, B or A")),
                };
                self.query = Some(self.draft.clone());
                self.mode = Mode::Search;
                self.refresh(b, s)?;
                self.state = State::Browse;
            }
            State::Pick(kind) => {
                if input.is_empty() {
                    self.state = State::Browse;
                } else {
                    let i = id(input)?;
                    b.discussion_item(s, i)?;
                    self.state = match kind {
                        Pick::Open => State::View(View {
                            id: i,
                            offset: 0,
                            next: None,
                            stamp: None,
                            previous: VecDeque::new(),
                        }),
                        Pick::Add => State::Review { i, add: true },
                        Pick::Remove => State::Review { i, add: false },
                    };
                }
            }
            State::Review { i, add } => match cmd.as_str() {
                "s" => {
                    b.discussion_bookmark(s, *i, *add)?;
                    self.notice = if *add {
                        "Bookmark saved locally."
                    } else {
                        "Bookmark removed locally."
                    }
                    .into();
                    self.state = State::Browse;
                }
                "c" => self.state = State::Browse,
                _ => return Err(Error::Invalid("use S or C")),
            },
            State::View(v) => match cmd.as_str() {
                "q" => self.state = State::Browse,
                "n" => {
                    if let Some(next) = v.next {
                        remember(&mut v.previous, v.offset);
                        v.offset = next;
                    }
                }
                "p" => v.offset = v.previous.pop_back().unwrap_or(0),
                "a" => self.state = State::Review { i: v.id, add: true },
                _ => return Err(Error::Invalid("use N, P, A or Q")),
            },
            State::Settings => {
                let mut c = c;
                match cmd.as_str() {
                    "q" => return Ok(true),
                    "e" => c.enabled = !c.enabled,
                    "l" => c.compact = !c.compact,
                    "n" => {
                        self.state = State::Name;
                        return Ok(false);
                    }
                    _ => return Err(Error::Invalid("choose a displayed setting")),
                };
                b.configure_discussion(s, &c)?;
            }
            State::Name => {
                let mut c = c;
                c.name = input.into();
                b.configure_discussion(s, &c)?;
                self.state = State::Settings;
            }
        }
        Ok(false)
    }
    pub fn recover(&mut self, e: &Error) -> bool {
        if !matches!(e, Error::Busy | Error::Invalid(_) | Error::NotFound) {
            return false;
        }
        self.notice = format!("Find: {e}");
        if matches!(e, Error::NotFound) {
            self.state = State::Browse;
            self.query = None;
            self.mode = Mode::Search;
            self.start = None;
        }
        true
    }
}
pub fn menu_theme(b: &mut Board, s: &Session, theme: &Theme) -> Result<Option<Theme>> {
    if b.discussion_config(s)?.enabled {
        let mut t = theme.clone();
        t.set_find_enabled(true).map_err(Error::Invalid)?;
        Ok(Some(t))
    } else {
        Ok(None)
    }
}
