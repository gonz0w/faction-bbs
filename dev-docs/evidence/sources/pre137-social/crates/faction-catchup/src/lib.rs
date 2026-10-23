//! Explicit personal catch-up workflow, independent of host I/O and mail.
#![forbid(unsafe_code)]
use faction_core::{
    Board, CatchupModule as M, CatchupTarget, CatchupWindow, Error, Result, Role, Session, Status,
};
use faction_terminal::Theme;
use std::{collections::VecDeque, fmt::Write};
enum State {
    Browse,
    Jump(M),
    Subscriptions,
    Checkpoint,
    Settings,
    Name,
    Order,
    NoticeEdit(String),
    NoticeReview(String),
}
pub enum Outcome {
    Stay,
    Main,
    Open(CatchupTarget),
}
pub struct Desk {
    state: State,
    module: Option<M>,
    window: Option<CatchupWindow>,
    after: i64,
    history: VecDeque<i64>,
    next: Option<i64>,
    notice: String,
    files_available: bool,
    width: usize,
}
impl Default for Desk {
    fn default() -> Self {
        Self::new(false)
    }
}
fn cut(s: &str, width: usize) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_graphic() || c == ' ' {
                c
            } else {
                '?'
            }
        })
        .take(width)
        .collect()
}
fn text(out: &mut String, s: &str, offset: usize, width: usize, rows: usize) -> Option<i64> {
    let mut at = offset.min(s.len());
    for _ in 0..rows {
        if at == s.len() {
            break;
        }
        let tail = &s[at..];
        let len = tail.find('\n').unwrap_or(tail.len()).min(width);
        out.push_str(&tail[..len]);
        out.push('\n');
        at += len;
        if s.as_bytes().get(at) == Some(&b'\n') {
            at += 1;
        }
    }
    (at < s.len()).then_some(at as i64)
}
fn approved(b: &mut Board, s: &Session, admin: bool) -> Result<()> {
    let m = b.whoami(s)?;
    if m.status != Status::Approved || (admin && m.role != Role::Sysop) {
        return Err(Error::Denied);
    }
    Ok(())
}
impl Desk {
    pub fn new(files_available: bool) -> Self {
        Self {
            state: State::Browse,
            module: None,
            window: None,
            after: 0,
            history: VecDeque::new(),
            next: None,
            notice: String::new(),
            files_available,
            width: 79,
        }
    }
    pub fn settings() -> Self {
        Self {
            state: State::Settings,
            ..Self::default()
        }
    }
    fn admin(&self) -> bool {
        matches!(
            self.state,
            State::Settings
                | State::Name
                | State::Order
                | State::NoticeEdit(_)
                | State::NoticeReview(_)
        )
    }
    fn reset(&mut self) {
        self.after = 0;
        self.history.clear();
        self.next = None;
    }
    fn window<'a>(&'a mut self, b: &mut Board, s: &Session) -> Result<&'a CatchupWindow> {
        if self.window.is_none() {
            self.window = Some(b.catchup_window(s)?);
        }
        Ok(self.window.as_ref().unwrap())
    }
    pub fn render(
        &mut self,
        b: &mut Board,
        s: &Session,
        theme: &Theme,
        ansi: bool,
        columns: usize,
    ) -> Result<String> {
        approved(b, s, self.admin())?;
        let c = b.catchup_config(s)?;
        let width = columns.clamp(20, 80) - 1;
        self.width = width;
        let mut out = faction_terminal::section(theme, &c.name, ansi);
        if !self.admin() && !c.enabled {
            out.push_str("Catch-up is disabled. Q:main\nCatch-up> ");
            return Ok(out);
        }
        if !self.notice.is_empty() {
            writeln!(out, "{}", cut(&self.notice, width)).unwrap();
        }
        if !self.admin() && self.module.is_none_or(|m| !c.modules.contains(&m)) {
            self.module = c.modules.first().copied();
            self.reset();
        }
        if !self.admin() {
            self.window(b, s)?;
        }
        match &self.state {
            State::Browse => {
                let w = self.window.as_ref().unwrap();
                writeln!(out, "Last saved visit: {}", w.last_saved.as_deref().unwrap_or("not set")).unwrap();
                if let Some(module) = self.module {
                    writeln!(out, "Section: {}", module.key()).unwrap();
                    self.next = None;
                    if module == M::Notice {
                        if c.notice.is_empty() {
                            out.push_str("No current sysop notice.\n");
                        } else {
                            self.next = text(&mut out, &c.notice, self.after as usize, width, 4);
                        }
                    } else if module == M::Files && !self.files_available {
                        out.push_str("Files unavailable in this session.\n");
                    } else {
                        let page = b.catchup_page(s, module, self.after, w)?;
                        self.next = page.next;
                        if module == M::Replies {
                            out.push_str("Direct replies in next 100 unread/room.\n");
                        }
                        if page.items.is_empty() {
                            out.push_str("No eligible items on this page.\n");
                        }
                        for item in page.items {
                            if module == M::Files {
                                writeln!(out, "File {}", item.key).unwrap();
                                writeln!(out, "{}", cut(&item.title, width)).unwrap();
                                writeln!(out, "{}", cut(&item.summary, width)).unwrap();
                            } else {
                                let label = format!("#{} ", item.key);
                                if c.compact {
                                    let summary = item.summary.replace(" direct replies in next 100 unread", " replies");
                                    let tail = format!(" / {summary}");
                                    let room = width.saturating_sub(label.len() + tail.len());
                                    writeln!(out, "{}", cut(&format!("{label}{}{tail}", cut(&item.title, room)), width)).unwrap();
                                } else {
                                    writeln!(out, "{}{}", label, cut(&item.title, width.saturating_sub(label.len()))).unwrap();
                                    writeln!(out, " {}", cut(&item.summary, width - 1)).unwrap();
                                }
                            }
                        }
                    }
                    writeln!(out, "Sections: {}", c.modules.iter().map(|m| m.key()).collect::<Vec<_>>().join(",")).unwrap();
                } else {
                    out.push_str("All sections disabled.\n");
                }
                out.push_str("M:next / M name N/P:page R:refresh\n");
                if self.module.is_some_and(|m| m != M::Notice && (m != M::Files || self.files_available)) {
                    out.push_str("J:source ID ");
                }
                out.push_str("S:subscriptions\nC:save visit Q:main\nCatch-up> ");
            }
            State::Jump(module) => {
                if !c.modules.contains(module) {
                    return Err(Error::Invalid("catch-up section is disabled"));
                }
                out.push_str(if *module == M::Files { "File object ID (blank cancels): " } else { "Source ID (blank cancels): " });
            }
            State::Subscriptions => {
                let p = b.catchup_preferences(s)?;
                writeln!(out, "Rooms: {}", if p.favorites_only { "favorites only" } else { "all" }).unwrap();
                let ids = p.favorites.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(" ");
                text(&mut out, &ids, 0, width, 4);
                out.push_str("+ID -ID all fav none Q:back\nSubscriptions> ");
            }
            State::Checkpoint => out.push_str("Save this visit's file cutoff and UTC time.\nRoom unread stays unchanged.\nNo automatic login tracking.\nS:save C:cancel\nVisit review> "),
            State::Settings => {
                writeln!(out, "{} / {}", if c.enabled { "Enabled" } else { "Disabled" }, if c.compact { "compact" } else { "cards" }).unwrap();
                writeln!(out, "Sections: {}", c.modules.iter().map(|m| m.key()).collect::<Vec<_>>().join(",")).unwrap();
                out.push_str("E:toggle N:name L:layout O:order/sections\nA:announcement Q:main\nCatch-up settings> ");
            }
            State::Name => out.push_str("Catch-up label (1-24 ASCII): "),
            State::Order => out.push_str("Order: notice,replies,rooms,files,gatherings\nComma-separated keys, '-' disables all: "),
            State::NoticeEdit(body) => {
                text(&mut out, body, 0, width, 4);
                out.push_str("<=4 lines/240 ASCII bytes.\n. preview /clear /undo /cancel\nAnnouncement text> ");
            }
            State::NoticeReview(body) => {
                out.push_str("PREVIEW / local approved-member notice\n");
                text(&mut out, body, 0, width, 8);
                out.push_str("S:save E:edit C:cancel\nAnnouncement review> ");
            }
        }
        if out.len() > 8192 {
            return Err(Error::Invalid("catch-up frame exceeds budget"));
        }
        Ok(out)
    }
    pub fn feed(&mut self, input: &str, b: &mut Board, s: &Session) -> Result<Outcome> {
        if input.len() > 128 || !input.bytes().all(|b| (32..=126).contains(&b)) {
            return Err(Error::Invalid(
                "catch-up input needs bounded printable ASCII",
            ));
        }
        approved(b, s, self.admin())?;
        let c = b.catchup_config(s)?;
        if !self.admin() && !c.enabled {
            return Ok(Outcome::Main);
        }
        let cmd = input.to_ascii_lowercase();
        self.notice.clear();
        match &mut self.state {
            State::Browse => {
                if cmd == "q" {
                    return Ok(Outcome::Main);
                }
                if cmd == "m" || cmd.starts_with("m ") {
                    let module = if cmd == "m" {
                        if c.modules.is_empty() {
                            None
                        } else {
                            let at = self
                                .module
                                .and_then(|m| c.modules.iter().position(|v| *v == m))
                                .unwrap_or(0);
                            Some(c.modules[(at + 1) % c.modules.len()])
                        }
                    } else {
                        let m = M::parse(cmd[2..].trim())?;
                        if !c.modules.contains(&m) {
                            return Err(Error::Invalid("catch-up section is disabled"));
                        }
                        Some(m)
                    };
                    self.module = module;
                    self.reset();
                } else {
                    match cmd.as_str() {
                        "n" => {
                            if let Some(next) = self.next {
                                if self.history.len() == 32 {
                                    self.history.pop_front();
                                }
                                self.history.push_back(self.after);
                                self.after = next;
                            }
                        }
                        "p" => self.after = self.history.pop_back().unwrap_or(0),
                        "r" => {
                            self.window = None;
                            self.reset();
                        }
                        "s" => self.state = State::Subscriptions,
                        "c" => {
                            self.window(b, s)?;
                            self.state = State::Checkpoint;
                        }
                        "j" => {
                            let m = self.module.ok_or(Error::Invalid("no section enabled"))?;
                            if m == M::Notice || (m == M::Files && !self.files_available) {
                                return Err(Error::Invalid("no source jump here"));
                            }
                            self.state = State::Jump(m);
                        }
                        _ => return Err(Error::Invalid("choose a displayed catch-up command")),
                    }
                }
            }
            State::Jump(module) => {
                if input.is_empty() {
                    self.state = State::Browse;
                } else {
                    let w = self
                        .window
                        .as_ref()
                        .ok_or(Error::Invalid("refresh this visit"))?;
                    let target = b.catchup_target(s, *module, input, w)?;
                    return Ok(Outcome::Open(target));
                }
            }
            State::Subscriptions => {
                if cmd == "q" {
                    self.state = State::Browse;
                } else {
                    let w = self
                        .window
                        .as_ref()
                        .ok_or(Error::Invalid("refresh this visit"))?;
                    b.catchup_subscribe(s, &w.revision, &cmd)?;
                    self.window = None;
                    self.window(b, s)?;
                    self.reset();
                    self.notice = "Subscriptions saved locally.".into();
                }
            }
            State::Checkpoint => match cmd.as_str() {
                "s" => {
                    let w = self
                        .window
                        .as_ref()
                        .ok_or(Error::Invalid("refresh this visit"))?;
                    b.catchup_checkpoint(s, w)?;
                    self.window = None;
                    self.reset();
                    self.state = State::Browse;
                    self.notice = "Visit saved; room unread unchanged.".into();
                }
                "c" => self.state = State::Browse,
                _ => return Err(Error::Invalid("use S or C")),
            },
            State::Settings => {
                let mut c = c;
                match cmd.as_str() {
                    "q" => return Ok(Outcome::Main),
                    "e" => c.enabled = !c.enabled,
                    "l" => c.compact = !c.compact,
                    "n" => {
                        self.state = State::Name;
                        return Ok(Outcome::Stay);
                    }
                    "o" => {
                        self.state = State::Order;
                        return Ok(Outcome::Stay);
                    }
                    "a" => {
                        self.state = State::NoticeEdit(c.notice);
                        return Ok(Outcome::Stay);
                    }
                    _ => return Err(Error::Invalid("choose a displayed setting")),
                }
                b.configure_catchup(s, &c)?;
            }
            State::Name => {
                let mut c = c;
                c.name = input.into();
                b.configure_catchup(s, &c)?;
                self.state = State::Settings;
            }
            State::Order => {
                let mut c = c;
                c.modules = if input == "-" {
                    vec![]
                } else {
                    input
                        .split(',')
                        .map(|m| M::parse(m.trim()))
                        .collect::<Result<Vec<_>>>()?
                };
                b.configure_catchup(s, &c)?;
                self.state = State::Settings;
            }
            State::NoticeEdit(body) => match input {
                "/cancel" => self.state = State::Settings,
                "." => self.state = State::NoticeReview(body.clone()),
                "/clear" => body.clear(),
                "/undo" => {
                    if let Some(at) = body.rfind('\n') {
                        body.truncate(at);
                    } else {
                        body.clear();
                    }
                }
                _ => {
                    let mut draft = body.clone();
                    if !draft.is_empty() {
                        draft.push('\n');
                    }
                    draft.push_str(if input == ".." { "." } else { input });
                    let mut check = c;
                    check.notice = draft.clone();
                    check.validate()?;
                    *body = draft;
                }
            },
            State::NoticeReview(body) => match cmd.as_str() {
                "s" => {
                    let mut c = c;
                    c.notice = body.clone();
                    b.configure_catchup(s, &c)?;
                    self.state = State::Settings;
                    self.notice = "Announcement saved locally.".into();
                }
                "e" => self.state = State::NoticeEdit(body.clone()),
                "c" => self.state = State::Settings,
                _ => return Err(Error::Invalid("use S, E or C")),
            },
        }
        Ok(Outcome::Stay)
    }
    pub fn recover(&mut self, e: &Error) -> bool {
        if !matches!(
            e,
            Error::Invalid(_) | Error::NotFound | Error::RequestConflict | Error::Busy
        ) {
            return false;
        }
        self.notice = format!("Catch-up: {e}");
        if matches!(e, Error::RequestConflict) {
            self.notice = "Visit changed elsewhere; refreshed safely.".into();
            self.window = None;
            self.reset();
            self.state = State::Browse;
        } else if matches!(e, Error::NotFound) || matches!(self.state, State::Jump(_)) {
            self.state = State::Browse;
        }
        true
    }
}
pub fn menu_theme(b: &mut Board, s: &Session, theme: &Theme) -> Result<Option<Theme>> {
    if b.catchup_config(s)?.enabled {
        let mut t = theme.clone();
        t.set_catchup_enabled(true).map_err(Error::Invalid)?;
        Ok(Some(t))
    } else {
        Ok(None)
    }
}
