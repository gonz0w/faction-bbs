//! Shared bounded local voting workflow. No I/O, host or network authority.
#![forbid(unsafe_code)]
use faction_core::{
    Board, Error, PollDraft, PollResults, PollState, Result, Role, Session, Status, request_id,
};
use faction_terminal::Theme;
use std::fmt::Write;
#[derive(Clone)]
struct Edit {
    id: Option<i64>,
    key: String,
    draft: PollDraft,
    step: u8,
}
#[derive(Clone)]
enum State {
    List,
    Find,
    Card(i64),
    Notes(i64, usize),
    Edit(Edit),
    Review(Edit),
    ReviewNotes(Edit, usize),
    Vote(i64, String),
    VoteReview(i64, String, u8),
    Finish(i64, String, PollState),
    Settings,
    Name,
    Moderate,
    Remove(i64, String),
}
pub struct Desk {
    state: State,
    offset: usize,
    finished: bool,
    page: usize,
    notice: String,
    width: usize,
}
impl Default for Desk {
    fn default() -> Self {
        Self {
            state: State::List,
            offset: 0,
            finished: false,
            page: 0,
            notice: String::new(),
            width: 79,
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
fn id(v: &str) -> Result<i64> {
    v.parse::<i64>()
        .ok()
        .filter(|v| *v > 0)
        .ok_or(Error::Invalid("positive poll ID required"))
}
fn clipped(v: &str, n: usize) -> &str {
    &v[..v.len().min(n)]
}
fn lines(out: &mut String, text: &str, width: usize, offset: usize, limit: usize) -> Option<usize> {
    let mut at = offset.min(text.len());
    for _ in 0..limit {
        if at == text.len() {
            break;
        }
        let rest = &text[at..];
        let len = rest.find('\n').unwrap_or(rest.len()).min(width);
        out.push_str(&rest[..len]);
        out.push('\n');
        at += len;
        if text.as_bytes().get(at) == Some(&b'\n') {
            at += 1;
        }
    }
    (at < text.len()).then_some(at)
}
fn mode(p: PollResults) -> &'static str {
    match p {
        PollResults::Always => "always",
        PollResults::AfterVote => "after your vote",
        PollResults::AfterClose => "after close",
    }
}
fn state(p: PollState) -> &'static str {
    match p {
        PollState::Open => "OPEN",
        PollState::Closed => "CLOSED",
        PollState::Cancelled => "CANCELLED",
    }
}
fn last_page(d: &PollDraft, width: usize) -> usize {
    d.choices.len().saturating_sub(1) / if width < 79 { 1 } else { 2 }
}
fn choice_page(out: &mut String, d: &PollDraft, counts: Option<&[u16]>, page: usize, width: usize) {
    let take = if width < 79 { 1 } else { 2 };
    let start = (page * take).min((d.choices.len() - 1) / take * take);
    let total = counts
        .map(|c| c.iter().map(|n| usize::from(*n)).sum::<usize>())
        .unwrap_or(0);
    for (i, label) in d.choices.iter().enumerate().skip(start).take(take) {
        write!(out, "{}: ", i + 1).unwrap();
        lines(out, label, width - 3, 0, 2);
        if let Some(c) = counts {
            let n = usize::from(c[i]);
            writeln!(
                out,
                "  {:12} {n}/{total}",
                "#".repeat((n * 12).checked_div(total).unwrap_or(0))
            )
            .unwrap();
        }
    }
    writeln!(
        out,
        "Choices {}-{}/{}",
        start + 1,
        (start + take).min(d.choices.len()),
        d.choices.len()
    )
    .unwrap();
}
impl Desk {
    pub fn settings() -> Self {
        Self {
            state: State::Settings,
            ..Self::default()
        }
    }
    fn admin(&self) -> bool {
        matches!(
            self.state,
            State::Settings | State::Name | State::Moderate | State::Remove(..)
        )
    }
    pub fn render(
        &mut self,
        b: &mut Board,
        s: &Session,
        theme: &Theme,
        ansi: bool,
        columns: usize,
    ) -> Result<String> {
        let admin = self.admin();
        let sysop = approved(b, s, admin)?;
        let c = b.poll_config(s)?;
        let width = columns.clamp(20, 80) - 1;
        self.width = width;
        if !admin
            && !sysop
            && !c.member_creators
            && matches!(
                self.state,
                State::Edit(_) | State::Review(_) | State::ReviewNotes(..)
            )
        {
            self.state = State::List;
            self.notice = "Member creation disabled; unsaved edit discarded.".into();
        }
        let mut out = faction_terminal::section(theme, &c.name, ansi);
        if !admin && !c.enabled {
            out.push_str("Polls are disabled. Q:main\nPolls> ");
            return Ok(out);
        }
        if !self.notice.is_empty() {
            writeln!(out, "{}", self.notice).unwrap();
        }
        match &self.state {
            State::List => {
                let rows = b.polls(s, self.finished, self.offset)?;
                writeln!(
                    out,
                    "{} / page {}",
                    if self.finished {
                        "Finished polls"
                    } else {
                        "Open polls"
                    },
                    self.offset / 4 + 1
                )
                .unwrap();
                if rows.is_empty() {
                    out.push_str("No polls on this page.\n");
                }
                for p in rows {
                    writeln!(
                        out,
                        "#{} {}{}",
                        p.id,
                        clipped(&p.draft.question, width.saturating_sub(13)),
                        if p.hidden {
                            " HIDDEN"
                        } else if !p.creator_approved {
                            " INACTIVE"
                        } else {
                            ""
                        }
                    )
                    .unwrap();
                    if !c.compact && columns >= 80 {
                        writeln!(out, "  {} / {}", p.creator, state(p.state)).unwrap();
                    }
                }
                out.push_str("V:view ID N/P:page A:open/finished\n");
                if sysop || c.member_creators {
                    out.push_str("C:create ");
                }
                out.push_str("Q:main\nPolls> ");
            }
            State::Find => out.push_str("Poll ID (blank cancels): "),
            State::Card(id) => {
                let p = b.poll(s, *id)?;
                writeln!(
                    out,
                    "#{} / {} / {}{}",
                    p.id,
                    p.creator,
                    state(p.state),
                    if p.hidden {
                        " / HIDDEN"
                    } else if !p.creator_approved {
                        " / INACTIVE"
                    } else {
                        ""
                    }
                )
                .unwrap();
                lines(&mut out, &p.draft.question, width, 0, 5);
                writeln!(out, "Results: {}", mode(p.draft.results)).unwrap();
                writeln!(
                    out,
                    "Your ballot: {}",
                    if p.own_vote == 0 {
                        "none".into()
                    } else {
                        p.own_vote.to_string()
                    }
                )
                .unwrap();
                self.page = self.page.min(last_page(&p.draft, width));
                choice_page(&mut out, &p.draft, p.counts.as_deref(), self.page, width);
                if p.counts.is_none() {
                    out.push_str("Results are not available.\n");
                }
                out.push_str("N/P:choices D:notes\n");
                if p.state == PollState::Open && !p.hidden && p.creator_approved {
                    out.push_str("V:vote/withdraw ");
                }
                if p.mine && p.state == PollState::Open {
                    if !p.ever_voted {
                        out.push_str("E:edit ");
                    }
                    out.push_str("F:close C:cancel ");
                }
                out.push_str("Q:list\nPoll> ");
            }
            State::Notes(id, offset) => {
                let p = b.poll(s, *id)?;
                out.push_str("Poll notes\n");
                lines(&mut out, &p.draft.description, width, *offset, 4);
                out.push_str("N:next Q:detail\nPoll notes> ");
            }
            State::Edit(e) => {
                if let Some(id) = e.id {
                    b.poll(s, id)?;
                }
                if !sysop && !c.member_creators {
                    return Err(Error::Invalid("member poll creation is disabled"));
                }
                match e.step {
                    0 => {
                        writeln!(
                            out,
                            "Current: {}",
                            clipped(&e.draft.question, width.saturating_sub(9))
                        )
                        .unwrap();
                        out.push_str("Question (1-96 ASCII, blank keeps): ");
                    }
                    1 => {
                        writeln!(
                            out,
                            "{} choices; /clear /undo /cancel",
                            e.draft.choices.len()
                        )
                        .unwrap();
                        choice_preview(&mut out, &e.draft.choices, width);
                        out.push_str("Choice (1-48 ASCII), '.' finishes: ");
                    }
                    2 => {
                        writeln!(out, "Current results: {}", mode(e.draft.results)).unwrap();
                        out.push_str("Results A:always V:after vote C:after close (blank keeps): ");
                    }
                    _ => {
                        lines(&mut out, &e.draft.description, width, 0, 4);
                        out.push_str("Notes: <=4 lines/240 ASCII bytes.\n. preview /clear /undo /cancel\nPoll text> ");
                    }
                }
            }
            State::Review(e) => {
                if let Some(id) = e.id {
                    b.poll(s, id)?;
                }
                out.push_str("PREVIEW ONLY / single choice\n");
                lines(&mut out, &e.draft.question, width, 0, 5);
                writeln!(out, "Results: {}", mode(e.draft.results)).unwrap();
                self.page = self.page.min(last_page(&e.draft, width));
                choice_page(&mut out, &e.draft, None, self.page, width);
                out.push_str("S:save E:edit D:notes N/P:choices C:cancel\nPoll review> ");
            }
            State::ReviewNotes(e, offset) => {
                if let Some(id) = e.id {
                    b.poll(s, id)?;
                }
                out.push_str("PREVIEW NOTES / unsaved\n");
                lines(&mut out, &e.draft.description, width, *offset, 4);
                out.push_str("N:next Q:review\nPoll note preview> ");
            }
            State::Vote(id, _) => {
                b.poll(s, *id)?;
                out.push_str("Choice number (0 withdraws): ");
            }
            State::VoteReview(id, revision, choice) => {
                let p = b.poll(s, *id)?;
                if p.revision != *revision {
                    return Err(Error::RequestConflict);
                }
                if *choice == 0 {
                    out.push_str("Withdraw your ballot.\n");
                } else {
                    writeln!(
                        out,
                        "Vote {}: {}",
                        choice,
                        p.draft.choices[usize::from(*choice) - 1]
                    )
                    .unwrap();
                }
                out.push_str("Local plaintext ballot, readable by the\noperator/backups. No secret-ballot guarantee.\nS:save C:cancel\nBallot review> ");
            }
            State::Finish(id, _, action) => {
                b.poll(s, *id)?;
                writeln!(out, "{} is final; ballots cannot change.", state(*action)).unwrap();
                out.push_str(if *action == PollState::Closed {
                    "Type CLOSE (anything else returns): "
                } else {
                    "Type CANCEL (anything else returns): "
                });
            }
            State::Settings => {
                writeln!(
                    out,
                    "{} / {} / creators: {}",
                    if c.enabled { "Enabled" } else { "Disabled" },
                    if c.compact { "compact" } else { "cards" },
                    if c.member_creators {
                        "approved members"
                    } else {
                        "sysops"
                    }
                )
                .unwrap();
                out.push_str(
                    "E:toggle N:name L:layout O:creators\nM:moderate Q:main\nPoll settings> ",
                );
            }
            State::Name => out.push_str("Poll label (1-24 ASCII): "),
            State::Moderate => out.push_str("Poll ID and hide|show|remove: "),
            State::Remove(id, _) => {
                b.poll(s, *id)?;
                out.push_str("Type DELETE (anything else cancels): ");
            }
        }
        if out.len() > 8192 {
            return Err(Error::Invalid("poll frame exceeds budget"));
        }
        Ok(out)
    }
    pub fn feed(&mut self, input: &str, b: &mut Board, s: &Session) -> Result<bool> {
        if input.len() > 128 || !input.bytes().all(|b| (32..=126).contains(&b)) {
            return Err(Error::Invalid("poll input needs bounded printable ASCII"));
        }
        let sysop = approved(b, s, self.admin())?;
        let c = b.poll_config(s)?;
        if !self.admin() && !c.enabled {
            return Ok(true);
        }
        let cmd = input.to_ascii_lowercase();
        self.notice.clear();
        match self.state.clone() {
            State::List => match cmd.as_str() {
                "q" => return Ok(true),
                "v" => self.state = State::Find,
                "n" => self.offset = (self.offset + 4).min(28),
                "p" => self.offset = self.offset.saturating_sub(4),
                "a" => {
                    self.finished = !self.finished;
                    self.offset = 0;
                }
                "c" if sysop || c.member_creators => {
                    self.state = State::Edit(Edit {
                        id: None,
                        key: String::new(),
                        draft: PollDraft {
                            question: String::new(),
                            description: String::new(),
                            choices: vec![],
                            results: PollResults::AfterVote,
                        },
                        step: 0,
                    });
                }
                _ => return Err(Error::Invalid("choose a displayed poll command")),
            },
            State::Find => {
                if input.is_empty() {
                    self.state = State::List;
                } else {
                    let id = id(input)?;
                    b.poll(s, id)?;
                    self.state = State::Card(id);
                    self.page = 0;
                }
            }
            State::Card(id) => {
                let p = b.poll(s, id)?;
                match cmd.as_str() {
                    "q" => self.state = State::List,
                    "n" => self.page = (self.page + 1).min(last_page(&p.draft, self.width)),
                    "p" => self.page = self.page.saturating_sub(1),
                    "d" => self.state = State::Notes(id, 0),
                    "v" if p.state == PollState::Open && !p.hidden && p.creator_approved => {
                        self.state = State::Vote(id, p.revision)
                    }
                    "e" if p.mine && !p.ever_voted && p.state == PollState::Open => {
                        self.state = State::Edit(Edit {
                            id: Some(id),
                            key: p.revision,
                            draft: p.draft,
                            step: 0,
                        })
                    }
                    "f" | "c" if p.mine && p.state == PollState::Open => {
                        self.state = State::Finish(
                            id,
                            p.revision,
                            if cmd == "f" {
                                PollState::Closed
                            } else {
                                PollState::Cancelled
                            },
                        )
                    }
                    _ => return Err(Error::Invalid("choose a displayed poll command")),
                }
            }
            State::Notes(id, offset) => {
                let p = b.poll(s, id)?;
                match cmd.as_str() {
                    "q" => self.state = State::Card(id),
                    "n" => {
                        let mut ignored = String::new();
                        let next = lines(&mut ignored, &p.draft.description, self.width, offset, 4)
                            .unwrap_or(0);
                        self.state = State::Notes(id, next);
                    }
                    _ => return Err(Error::Invalid("use N or Q")),
                }
            }
            State::Edit(mut e) => {
                if cmd == "/cancel" {
                    self.state = e.id.map_or(State::List, State::Card);
                    return Ok(false);
                }
                if let Some(id) = e.id {
                    b.poll(s, id)?;
                }
                match e.step {
                    0 => {
                        if !input.is_empty() {
                            if input.len() > 96 {
                                return Err(Error::Invalid("question needs <=96 ASCII bytes"));
                            }
                            e.draft.question = input.into();
                        }
                        if e.draft.question.trim().is_empty() {
                            return Err(Error::Invalid("question required"));
                        }
                        e.step = 1;
                    }
                    1 => match input {
                        "." => {
                            if e.draft.choices.len() < 2 {
                                return Err(Error::Invalid("at least two choices required"));
                            }
                            e.step = 2;
                        }
                        "/clear" => e.draft.choices.clear(),
                        "/undo" => {
                            e.draft.choices.pop();
                        }
                        _ => {
                            if input.trim().is_empty()
                                || input.len() > 48
                                || e.draft.choices.len() >= 8
                            {
                                return Err(Error::Invalid("2-8 choices, each 1-48 ASCII bytes"));
                            }
                            e.draft.choices.push(input.into());
                        }
                    },
                    2 => {
                        e.draft.results = match cmd.as_str() {
                            "a" => PollResults::Always,
                            "v" => PollResults::AfterVote,
                            "c" => PollResults::AfterClose,
                            "" => e.draft.results,
                            _ => return Err(Error::Invalid("choose A, V or C")),
                        };
                        e.step = 3;
                    }
                    _ => match input {
                        "." => {
                            e.draft.validate()?;
                            self.state = State::Review(e);
                            self.page = 0;
                            return Ok(false);
                        }
                        "/clear" => e.draft.description.clear(),
                        "/undo" => {
                            if let Some(at) = e.draft.description.rfind('\n') {
                                e.draft.description.truncate(at);
                            } else {
                                e.draft.description.clear();
                            }
                        }
                        _ => {
                            let line = if input == ".." { "." } else { input };
                            let mut d = e.draft.clone();
                            if !d.description.is_empty() {
                                d.description.push('\n');
                            }
                            d.description.push_str(line);
                            d.validate()?;
                            e.draft = d;
                        }
                    },
                }
                self.state = State::Edit(e);
            }
            State::Review(mut e) => match cmd.as_str() {
                "s" => {
                    if e.key.is_empty() {
                        e.key = request_id()?;
                    }
                    self.state = State::Review(e.clone());
                    let id = b.save_poll(s, e.id, &e.key, &e.draft)?;
                    self.state = State::Card(id);
                    self.page = 0;
                    self.notice = "Poll saved locally.".into();
                }
                "d" => self.state = State::ReviewNotes(e, 0),
                "e" => {
                    e.step = 0;
                    self.state = State::Edit(e);
                }
                "n" => self.page = (self.page + 1).min(last_page(&e.draft, self.width)),
                "p" => self.page = self.page.saturating_sub(1),
                "c" => self.state = e.id.map_or(State::List, State::Card),
                _ => return Err(Error::Invalid("use S, E, N/P or C")),
            },
            State::ReviewNotes(e, offset) => match cmd.as_str() {
                "q" => self.state = State::Review(e),
                "n" => {
                    let mut ignored = String::new();
                    let next = lines(&mut ignored, &e.draft.description, self.width, offset, 4)
                        .unwrap_or(0);
                    self.state = State::ReviewNotes(e, next);
                }
                _ => return Err(Error::Invalid("use N or Q")),
            },
            State::Vote(id, revision) => {
                let p = b.poll(s, id)?;
                let choice = input
                    .parse::<u8>()
                    .ok()
                    .filter(|n| usize::from(*n) <= p.draft.choices.len())
                    .ok_or(Error::Invalid("choose a displayed choice or zero"))?;
                self.state = State::VoteReview(id, revision, choice);
            }
            State::VoteReview(id, revision, choice) => match cmd.as_str() {
                "s" => {
                    b.vote_poll(s, id, &revision, choice)?;
                    self.state = State::Card(id);
                    self.notice = "Ballot saved locally.".into();
                }
                "c" => self.state = State::Card(id),
                _ => return Err(Error::Invalid("use S or C")),
            },
            State::Finish(id, revision, action) => {
                if input
                    == if action == PollState::Closed {
                        "CLOSE"
                    } else {
                        "CANCEL"
                    }
                {
                    b.set_poll_state(s, id, &revision, action)?;
                    self.notice = format!("Poll {}.", state(action));
                }
                self.state = State::Card(id);
            }
            State::Settings => {
                let mut c = c;
                match cmd.as_str() {
                    "q" => return Ok(true),
                    "e" => c.enabled = !c.enabled,
                    "l" => c.compact = !c.compact,
                    "o" => c.member_creators = !c.member_creators,
                    "n" => {
                        self.state = State::Name;
                        return Ok(false);
                    }
                    "m" => {
                        self.state = State::Moderate;
                        return Ok(false);
                    }
                    _ => return Err(Error::Invalid("choose a displayed setting")),
                }
                b.configure_polls(s, &c)?;
            }
            State::Name => {
                let mut c = c;
                c.name = input.into();
                b.configure_polls(s, &c)?;
                self.state = State::Settings;
            }
            State::Moderate => {
                let parts = input.split_whitespace().collect::<Vec<_>>();
                if parts.len() != 2 {
                    return Err(Error::Invalid("use ID hide|show|remove"));
                }
                let id = id(parts[0])?;
                let p = b.poll(s, id)?;
                if parts[1] == "remove" {
                    self.state = State::Remove(id, p.revision);
                } else {
                    b.moderate_poll(s, id, &p.revision, parts[1])?;
                    self.notice = "Moderation applied; author text retained.".into();
                    self.state = State::Settings;
                }
            }
            State::Remove(id, revision) => {
                if input == "DELETE" {
                    b.moderate_poll(s, id, &revision, "remove")?;
                    self.notice = "Poll and ballots removed.".into();
                }
                self.state = State::Settings;
            }
        }
        Ok(false)
    }
    pub fn recover(&mut self, e: &Error) -> bool {
        if !matches!(
            e,
            Error::Invalid(_) | Error::NotFound | Error::RequestConflict | Error::Busy
        ) {
            return false;
        }
        self.notice = if matches!(e, Error::RequestConflict) {
            "Poll changed; reopen before saving. Unsaved editor kept.".into()
        } else {
            format!("Polls: {e}")
        };
        if matches!(e, Error::NotFound) {
            self.state = State::List;
        } else if matches!(e, Error::RequestConflict) {
            match self.state {
                State::Vote(id, _) | State::VoteReview(id, ..) | State::Finish(id, ..) => {
                    self.state = State::Card(id)
                }
                _ => (),
            }
        }
        true
    }
}
fn choice_preview(out: &mut String, choices: &[String], width: usize) {
    for (i, s) in choices.iter().enumerate().rev().take(3).rev() {
        writeln!(out, "{}: {}", i + 1, clipped(s, width - 3)).unwrap();
    }
}
pub fn menu_theme(b: &mut Board, s: &Session, theme: &Theme) -> Result<Option<Theme>> {
    if b.poll_config(s)?.enabled {
        let mut t = theme.clone();
        t.set_polls_enabled(true).map_err(Error::Invalid)?;
        Ok(Some(t))
    } else {
        Ok(None)
    }
}
