//! One bounded inert gathering workflow shared by local and shared terminals.
#![forbid(unsafe_code)]
use faction_core::{
    Board, Error, GatheringDraft, GatheringPlace, Result, Role, Session, Status, request_id,
};
use faction_terminal::Theme;
use std::fmt::Write;
#[derive(Clone)]
struct Edit {
    id: Option<i64>,
    key: String,
    draft: GatheringDraft,
    step: u8,
}
#[derive(Clone)]
enum State {
    List,
    Find,
    Card(i64),
    Edit(Edit),
    Review(Edit),
    Respond(i64, String),
    Response(i64, String, u8),
    Cancel(i64, String),
    Settings,
    Name,
    Moderate,
    Remove(i64, String),
}
pub enum Outcome {
    Stay,
    Main,
    Join(GatheringPlace),
}
pub struct Desk {
    state: State,
    offset: usize,
    past: bool,
    notice: String,
    preview: usize,
    next: Option<usize>,
}
impl Default for Desk {
    fn default() -> Self {
        Self {
            state: State::List,
            offset: 0,
            past: false,
            notice: String::new(),
            preview: 0,
            next: None,
        }
    }
}
fn approved(b: &mut Board, s: &Session, admin: bool) -> Result<bool> {
    let m = b.whoami(s)?;
    if m.status != Status::Approved || (admin && m.role != Role::Sysop) {
        return Err(Error::Denied);
    }
    Ok(m.role == Role::Sysop)
}
fn clipped(text: &str, width: usize) -> &str {
    &text[..text.len().min(width)]
}
fn body(out: &mut String, text: &str, offset: usize, columns: usize) -> Option<usize> {
    let width = columns.clamp(20, 80) - 1;
    let mut at = offset.min(text.len());
    let mut rows = 0;
    while at < text.len() && rows < 4 {
        let rest = &text[at..];
        let end = rest.find('\n').unwrap_or(rest.len()).min(width);
        out.push_str(&rest[..end]);
        out.push('\n');
        at += end;
        if text.as_bytes().get(at) == Some(&b'\n') {
            at += 1
        }
        rows += 1;
    }
    (at < text.len()).then_some(at)
}
fn id(text: &str) -> Result<i64> {
    text.parse::<i64>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or(Error::Invalid("positive gathering ID required"))
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
        let admin = matches!(
            self.state,
            State::Settings | State::Name | State::Moderate | State::Remove(..)
        );
        let sysop = approved(b, s, admin)?;
        let c = b.gathering_config(s)?;
        if !admin && !c.enabled {
            return Ok(format!(
                "{}Gatherings are disabled.\nQ:main\nGatherings> ",
                faction_terminal::section(theme, &c.name, ansi)
            ));
        }
        let width = columns.clamp(20, 80) - 1;
        let mut out = faction_terminal::section(theme, &c.name, ansi);
        self.next = None;
        if !self.notice.is_empty() {
            writeln!(out, "{}", self.notice).unwrap()
        }
        match &self.state {
            State::List => {
                let rows = b.gatherings(s, self.past, self.offset)?;
                writeln!(
                    out,
                    "{} / UTC / page {}",
                    if self.past { "Recent past" } else { "Upcoming" },
                    1 + self.offset / 4
                )
                .unwrap();
                if rows.is_empty() {
                    out.push_str("No gatherings on this page.\n")
                }
                for g in rows {
                    let status = if g.hidden {
                        "HIDDEN"
                    } else if !g.organizer_approved {
                        "INACTIVE"
                    } else if g.cancelled {
                        "CANCELLED"
                    } else {
                        ""
                    };
                    if c.compact || columns < 80 {
                        writeln!(
                            out,
                            "#{} {} {}",
                            g.id,
                            clipped(&g.draft.title, width.saturating_sub(15)),
                            status
                        )
                        .unwrap();
                    } else {
                        writeln!(
                            out,
                            "#{} {} {}\n  {} UTC / {}",
                            g.id, g.draft.title, status, g.draft.starts_utc, g.organizer
                        )
                        .unwrap();
                    }
                }
                out.push_str("V:view ID N/P:page A:past/upcoming\n");
                if sysop || c.member_organizers {
                    out.push_str("C:create  ")
                }
                out.push_str("Q:main\nGatherings> ");
            }
            State::Find => out.push_str("Gathering ID (blank cancels): "),
            State::Card(event) => {
                let g = b.gathering(s, *event)?;
                writeln!(
                    out,
                    "#{} / {} / {}",
                    g.id,
                    g.organizer,
                    if g.hidden {
                        "HIDDEN"
                    } else if !g.organizer_approved {
                        "INACTIVE"
                    } else if g.cancelled {
                        "CANCELLED"
                    } else if g.ended {
                        "ENDED"
                    } else {
                        "LOCAL"
                    }
                )
                .unwrap();
                body(&mut out, &g.draft.title, 0, columns);
                writeln!(out, "{} to\n{} UTC", g.draft.starts_utc, g.ends_utc).unwrap();
                if !g.draft.place.is_empty() {
                    writeln!(out, "Place: {}", g.draft.place).unwrap()
                }
                if let Some((going, maybe)) = g.counts {
                    writeln!(out, "Going: {going} / Maybe: {maybe}").unwrap()
                } else {
                    out.push_str("Attendance counts are organizer-only.\n")
                }
                writeln!(
                    out,
                    "Your RSVP: {}",
                    match g.response {
                        1 => "going",
                        2 => "maybe",
                        _ => "none",
                    }
                )
                .unwrap();
                self.next = body(&mut out, &g.draft.body, self.preview, columns);
                out.push_str("N:more text R:RSVP J:join\n");
                if g.mine {
                    out.push_str("E:edit C:cancel  ")
                }
                out.push_str("Q:list\nGathering> ");
            }
            State::Edit(e) => {
                if let Some(id) = e.id {
                    let g = b.gathering(s, id)?;
                    if !g.mine {
                        return Err(Error::Denied);
                    }
                }
                if !sysop && !c.member_organizers {
                    return Err(Error::Invalid("member organizing is disabled"));
                }
                match e.step {
                    0 => {
                        writeln!(out, "Current: {}", e.draft.title).unwrap();
                        out.push_str("Title (1-64 ASCII, blank keeps): ");
                    }
                    1 => {
                        writeln!(out, "Current UTC: {}", e.draft.starts_utc).unwrap();
                        out.push_str("Start UTC YYYY-MM-DD HH:MM (blank keeps): ");
                    }
                    2 => {
                        writeln!(out, "Current duration: {} minutes", e.draft.minutes).unwrap();
                        out.push_str("Duration 1-1440 minutes (blank keeps): ");
                    }
                    3 => {
                        writeln!(out, "Current place: {}", e.draft.place).unwrap();
                        out.push_str("Place: room:ID, #channel, '-' none, blank keeps: ");
                    }
                    _ => {
                        body(&mut out, &e.draft.body, 0, columns);
                        out.push_str("Notes: <=8 lines/480 ASCII bytes.\n. review /clear /undo /cancel\nGathering text> ");
                    }
                }
            }
            State::Review(e) => {
                if let Some(id) = e.id {
                    b.gathering(s, id)?;
                }
                out.push_str("PREVIEW ONLY / one-off / UTC\n");
                writeln!(
                    out,
                    "{}\n{} UTC / {} minutes\nPlace: {}",
                    e.draft.title,
                    e.draft.starts_utc,
                    e.draft.minutes,
                    if e.draft.place.is_empty() {
                        "none"
                    } else {
                        &e.draft.place
                    }
                )
                .unwrap();
                self.next = body(&mut out, &e.draft.body, self.preview, columns);
                if e.id.is_some() {
                    out.push_str("Time/place changes clear RSVPs.\n")
                }
                out.push_str("S:save E:edit N:more C:cancel\nGathering review> ");
            }
            State::Respond(event, _) => {
                b.gathering(s, *event)?;
                out.push_str("RSVP: 1 going / 2 maybe / 0 clear: ");
            }
            State::Response(event, _, response) => {
                b.gathering(s, *event)?;
                writeln!(
                    out,
                    "Your response: {}",
                    match response {
                        1 => "going",
                        2 => "maybe",
                        _ => "clear",
                    }
                )
                .unwrap();
                out.push_str("Only your own response is displayed.\nNo attendee-name lists. Local plaintext\nreadable by the operator; counts may show.\nS:save C:cancel\nRSVP review> ");
            }
            State::Cancel(event, _) => {
                let g = b.gathering(s, *event)?;
                writeln!(out, "Cancel #{} / {}?", g.id, g.draft.title).unwrap();
                out.push_str("Type CANCEL (anything else returns): ");
            }
            State::Settings => {
                writeln!(out,"Enabled: {} / UTC one-off\nName: {}\nCompact: {} / member organizers: {}\nLogin reminders: {} / public counts: {}",c.enabled,c.name,c.compact,c.member_organizers,c.reminders,c.public_counts).unwrap();
                out.push_str("E:enable N:name L:layout O:organizers\nR:reminders V:counts M:moderate Q:main\nGathering settings> ");
            }
            State::Name => out.push_str("Gathering label (1-24 ASCII): "),
            State::Moderate => out.push_str("Gathering ID and hide|show|remove: "),
            State::Remove(event, _) => {
                let g = b.gathering(s, *event)?;
                writeln!(out, "Remove #{} and all responses: {}", g.id, g.draft.title).unwrap();
                out.push_str("Type DELETE (anything else cancels): ");
            }
        }
        if out.len() > 8192 {
            return Err(Error::Invalid("gathering frame exceeds budget"));
        }
        Ok(out)
    }
    pub fn feed(&mut self, input: &str, b: &mut Board, s: &Session) -> Result<Outcome> {
        if input.len() > 128 || !input.bytes().all(|b| (32..=126).contains(&b)) {
            return Err(Error::Invalid(
                "gathering input needs bounded printable ASCII",
            ));
        }
        let admin = matches!(
            self.state,
            State::Settings | State::Name | State::Moderate | State::Remove(..)
        );
        let sysop = approved(b, s, admin)?;
        let c = b.gathering_config(s)?;
        if !admin && !c.enabled {
            return Ok(Outcome::Main);
        }
        let command = input.to_ascii_lowercase();
        self.notice.clear();
        match self.state.clone() {
            State::List => match command.as_str() {
                "q" => return Ok(Outcome::Main),
                "v" => self.state = State::Find,
                "n" => self.offset = (self.offset + 4).min(60),
                "p" => self.offset = self.offset.saturating_sub(4),
                "a" => {
                    self.past = !self.past;
                    self.offset = 0
                }
                "c" if sysop || c.member_organizers => {
                    self.state = State::Edit(Edit {
                        id: None,
                        key: String::new(),
                        draft: GatheringDraft {
                            title: String::new(),
                            body: String::new(),
                            starts_utc: String::new(),
                            minutes: 120,
                            place: String::new(),
                        },
                        step: 0,
                    })
                }
                _ => self.notice = "Choose a displayed command, followed by Enter.".into(),
            },
            State::Find => {
                if input.is_empty() {
                    self.state = State::List
                } else {
                    let event = id(input)?;
                    b.gathering(s, event)?;
                    self.state = State::Card(event);
                    self.preview = 0;
                }
            }
            State::Card(event) => {
                let g = b.gathering(s, event)?;
                match command.as_str() {
                    "q" => {
                        self.state = State::List;
                        self.preview = 0
                    }
                    "n" => self.preview = self.next.unwrap_or(0),
                    "r" => self.state = State::Respond(event, g.revision),
                    "e" if g.mine && (sysop || c.member_organizers) => {
                        self.state = State::Edit(Edit {
                            id: Some(event),
                            key: g.revision,
                            draft: g.draft,
                            step: 0,
                        })
                    }
                    "c" if g.mine => self.state = State::Cancel(event, g.revision),
                    "j" => return Ok(Outcome::Join(b.gathering_destination(s, event)?)),
                    _ => self.notice = "Choose a displayed gathering command.".into(),
                }
            }
            State::Edit(mut e) => {
                if input == "/cancel" {
                    self.state = e.id.map_or(State::List, State::Card);
                    return Ok(Outcome::Stay);
                }
                match e.step {
                    0 => {
                        if !input.is_empty() {
                            e.draft.title = input.into()
                        }
                        if e.draft.title.trim().is_empty() || e.draft.title.len() > 64 {
                            return Err(Error::Invalid("title needs 1-64 ASCII bytes"));
                        }
                        e.step = 1;
                    }
                    1 => {
                        if !input.is_empty() {
                            e.draft.starts_utc = input.into()
                        }
                        e.step = 2;
                    }
                    2 => {
                        if !input.is_empty() {
                            e.draft.minutes = input
                                .parse()
                                .ok()
                                .filter(|m| (1..=1440).contains(m))
                                .ok_or(Error::Invalid("duration needs 1-1440 minutes"))?;
                        }
                        e.step = 3;
                    }
                    3 => {
                        if input == "-" {
                            e.draft.place.clear()
                        } else if !input.is_empty() {
                            e.draft.place = input.into()
                        }
                        e.step = 4;
                    }
                    _ => {
                        if input == "." {
                            self.state = State::Review(e);
                            self.preview = 0;
                            return Ok(Outcome::Stay);
                        }
                        if input == "/clear" {
                            e.draft.body.clear()
                        } else if input == "/undo" {
                            if let Some(at) = e.draft.body.rfind('\n') {
                                e.draft.body.truncate(at)
                            } else {
                                e.draft.body.clear()
                            }
                        } else {
                            let line = if input == ".." { "." } else { input };
                            let mut next = e.draft.body.clone();
                            if !next.is_empty() {
                                next.push('\n')
                            }
                            next.push_str(line);
                            faction_core::page_text(&next, 480, 8)?;
                            e.draft.body = next;
                        }
                    }
                }
                self.state = State::Edit(e);
            }
            State::Review(mut e) => match command.as_str() {
                "s" => {
                    if e.key.is_empty() {
                        e.key = request_id()?;
                    }
                    self.state = State::Review(e.clone());
                    let event = b.save_gathering(s, e.id, &e.key, &e.draft)?;
                    self.state = State::Card(event);
                    self.preview = 0;
                    self.notice = "Gathering saved locally.".into();
                }
                "e" => {
                    e.step = 0;
                    self.state = State::Edit(e)
                }
                "n" => self.preview = self.next.unwrap_or(0),
                "c" => self.state = e.id.map_or(State::List, State::Card),
                _ => (),
            },
            State::Respond(event, revision) => {
                let n = input
                    .parse::<u8>()
                    .ok()
                    .filter(|n| *n <= 2)
                    .ok_or(Error::Invalid("use 0 clear, 1 going or 2 maybe"))?;
                self.state = State::Response(event, revision, n);
            }
            State::Response(event, revision, response) => match command.as_str() {
                "s" => {
                    b.gathering_rsvp(s, event, &revision, response)?;
                    self.notice = "Your local RSVP was saved.".into();
                    self.state = State::Card(event)
                }
                "c" => self.state = State::Card(event),
                _ => (),
            },
            State::Cancel(event, revision) => {
                if input == "CANCEL" {
                    b.cancel_gathering(s, event, &revision)?;
                    self.notice = "Gathering cancelled; no automatic notifications sent.".into();
                }
                self.state = State::Card(event)
            }
            State::Settings => {
                let mut config = c;
                match command.as_str() {
                    "q" => return Ok(Outcome::Main),
                    "n" => {
                        self.state = State::Name;
                        return Ok(Outcome::Stay);
                    }
                    "m" => {
                        self.state = State::Moderate;
                        return Ok(Outcome::Stay);
                    }
                    "e" => config.enabled = !config.enabled,
                    "l" => config.compact = !config.compact,
                    "o" => config.member_organizers = !config.member_organizers,
                    "r" => config.reminders = !config.reminders,
                    "v" => config.public_counts = !config.public_counts,
                    _ => return Ok(Outcome::Stay),
                }
                b.configure_gatherings(s, &config)?;
            }
            State::Name => {
                let mut config = c;
                config.name = input.into();
                b.configure_gatherings(s, &config)?;
                self.state = State::Settings
            }
            State::Moderate => {
                let parts = input.split_whitespace().collect::<Vec<_>>();
                if parts.len() != 2 {
                    return Err(Error::Invalid("use ID hide|show|remove"));
                }
                let event = id(parts[0])?;
                let g = b.gathering(s, event)?;
                if parts[1] == "remove" {
                    self.state = State::Remove(event, g.revision)
                } else {
                    b.moderate_gathering(s, event, &g.revision, parts[1])?;
                    self.notice = "Moderation applied; author text retained.".into();
                    self.state = State::Settings
                }
            }
            State::Remove(event, revision) => {
                if input == "DELETE" {
                    b.moderate_gathering(s, event, &revision, "remove")?;
                    self.notice = "Gathering and all responses removed.".into();
                }
                self.state = State::Settings
            }
        }
        Ok(Outcome::Stay)
    }
    pub fn recover(&mut self, error: &Error) -> bool {
        if !matches!(
            error,
            Error::Invalid(_) | Error::NotFound | Error::RequestConflict | Error::Busy
        ) {
            return false;
        }
        self.notice = if matches!(error, Error::RequestConflict) {
            "Event changed; reopen before saving. Unsaved editor text kept.".into()
        } else {
            format!("Gatherings: {error}")
        };
        if matches!(error, Error::NotFound) {
            self.state = State::List;
            self.preview = 0
        } else if matches!(error, Error::RequestConflict) {
            match self.state {
                State::Respond(id, _) | State::Response(id, ..) | State::Cancel(id, _) => {
                    self.state = State::Card(id)
                }
                _ => (),
            }
        }
        true
    }
}
pub fn menu_theme(b: &mut Board, s: &Session, theme: &Theme) -> Result<Option<Theme>> {
    if b.gathering_config(s)?.enabled {
        let mut t = theme.clone();
        t.set_gatherings_enabled(true).map_err(Error::Invalid)?;
        Ok(Some(t))
    } else {
        Ok(None)
    }
}
