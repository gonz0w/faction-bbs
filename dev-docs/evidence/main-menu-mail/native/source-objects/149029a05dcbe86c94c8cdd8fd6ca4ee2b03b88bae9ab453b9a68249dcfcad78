//! Local directory submission workflow shared by every terminal adapter.
use faction_core::{
    Board, DirectoryDraft, DirectoryKey, DirectorySubmission, Error, Result, Role, Session, Status,
    request_id,
};
use faction_terminal::{
    Theme,
    directory::{Directory, Overlay},
};
use std::fmt::Write;

pub fn owner_keys(directory: &Directory) -> Vec<DirectoryKey> {
    directory
        .curator_keys()
        .into_iter()
        .map(|(slug, host, port, login)| DirectoryKey {
            slug,
            host,
            port,
            login,
        })
        .collect()
}
/// The core supplies current approved-author publication rows; the terminal
/// validates their union without acquiring storage or connection authority.
pub fn catalog(board: &mut Board, session: &Session, owner: &Directory) -> Result<Directory> {
    let mut cards = Vec::new();
    for row in board.directory_published(session)? {
        let d = &row.draft;
        d.validate()?;
        let mut source = format!(
            "version=1\n[[boards]]\nslug={:?}\nname={:?}\nsummary={:?}\nhost={:?}\nport={}\nlogin={:?}\nreviewed={:?}\nvalid_until={:?}\n",
            d.slug,
            d.name,
            d.summary,
            d.host,
            d.port,
            d.login,
            row.reviewed
                .as_deref()
                .ok_or(Error::Invalid("missing directory review date"))?,
            row.valid_until
                .as_deref()
                .ok_or(Error::Invalid("missing directory expiry date"))?
        );
        if let Some(hint) = &d.fingerprint {
            writeln!(source, "fingerprint={hint:?}").unwrap();
        }
        cards.push(Overlay {
            id: row.id,
            revision: row.revision,
            catalog: Directory::parse(&source).map_err(Error::Invalid)?,
            expired: row.state == "expired",
        });
    }
    owner.compose(cards).map_err(Error::Invalid)
}
const FIELDS: [&str; 7] = [
    "Slug",
    "Name",
    "Summary",
    "SSH host",
    "SSH port",
    "SSH login",
    "Public fingerprint (blank: none)",
];
enum State {
    List,
    Input {
        fields: [String; 7],
        index: usize,
        nonce: String,
    },
    Preview {
        draft: DirectoryDraft,
        nonce: String,
    },
    Card(DirectorySubmission),
    ReviewDate(DirectorySubmission),
    ExpiryDate(DirectorySubmission, String),
    Confirm {
        row: DirectorySubmission,
        action: &'static str,
        reviewed: Option<String>,
        until: Option<String>,
    },
}
pub struct Desk {
    state: State,
    review: bool,
    after: i64,
    shown: Vec<DirectorySubmission>,
    page: usize,
    notice: String,
}
impl Default for Desk {
    fn default() -> Self {
        Self {
            state: State::List,
            review: false,
            after: 0,
            shown: Vec::new(),
            page: 0,
            notice: String::new(),
        }
    }
}
fn actor(b: &mut Board, s: &Session, review: bool) -> Result<Role> {
    let m = b.whoami(s)?;
    if m.status != Status::Approved || (review && m.role != Role::Sysop) {
        return Err(Error::Denied);
    }
    // The startup-only feature gate applies even before an unsaved field exists.
    b.directory_submissions(s, 0)?;
    Ok(m.role)
}
fn draft_lines(d: &DirectoryDraft) -> Vec<String> {
    vec![
        format!("Slug: {}", d.slug),
        format!("Name: {}", d.name),
        format!("Summary: {}", d.summary),
        format!("Host: {}", d.host),
        format!("Port: {} | Login: {}", d.port, d.login),
        format!(
            "Public hint: {}",
            d.fingerprint.as_deref().unwrap_or("none")
        ),
    ]
}
fn wrap(lines: Vec<String>, width: usize) -> Vec<String> {
    lines
        .into_iter()
        .flat_map(|line| {
            line.split('\n')
                .flat_map(|part| {
                    if part.is_empty() {
                        vec![String::new()]
                    } else {
                        part.as_bytes()
                            .chunks(width)
                            .map(|v| String::from_utf8(v.to_vec()).expect("ASCII workflow text"))
                            .collect()
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

impl Desk {
    pub fn render(
        &mut self,
        b: &mut Board,
        s: &Session,
        theme: &Theme,
        ansi: bool,
        dimensions: (usize, usize),
    ) -> Result<String> {
        let (columns, rows) = dimensions;
        let role = actor(b, s, self.review)?;
        let width = columns.clamp(40, 120) - 1;
        let height = rows.clamp(18, 100);
        let mut body = Vec::new();
        let controls: &str;
        let mut prompt = "Submission command: ".to_owned();
        match &self.state {
            State::List => {
                body.push(
                    if self.review {
                        "Sysop review / all retained rows"
                    } else {
                        "Your local submissions / no auto-publish"
                    }
                    .into(),
                );
                let rows = b.directory_submissions(s, self.after)?;
                // Preserve the core page cursor even when sysop's own list filters
                // other authors out of the four bounded rows.
                self.shown = rows;
                for (i, r) in self
                    .shown
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| self.review || r.mine)
                {
                    body.push(format!(
                        "{}: #{} {} [{}]",
                        i + 1,
                        r.id,
                        r.draft.name,
                        r.state
                    ));
                }
                if !self.shown.iter().any(|r| self.review || r.mine) {
                    body.push("No submissions on this page.".into());
                }
                controls = if role == Role::Sysop {
                    "1-4:open N:new >:next B:first\nM:review L:mine"
                } else {
                    "1-4:open N:new >:next B:first"
                };
                body.push("Owner pack changes require pack/restart.".into());
            }
            State::Input { index, .. } => {
                body.push("New unsaved listing / C:cancel".into());
                body.push(format!("Field {} / 7", index + 1));
                controls = "No save or publication until confirmed.";
                prompt = format!("{}: ", FIELDS[*index]);
            }
            State::Preview { draft, .. } => {
                body.push("Draft preview / not saved".into());
                body.extend(draft_lines(draft));
                controls = "S:save private draft C:cancel N/P:more";
            }
            State::Card(r) => {
                body.push(format!("Submission #{} [{}]", r.id, r.state));
                body.push(format!("Author: {} | revision {}", r.author, r.revision));
                body.extend(draft_lines(&r.draft));
                if let Some(v) = &r.reviewed {
                    body.push(format!("Reviewed: {v}"));
                }
                if let Some(v) = &r.valid_until {
                    body.push(format!("Valid until: {v}"));
                }
                controls = if self.review {
                    if r.state == "pending" {
                        "A:approve R:reject L:list N/P:more"
                    } else if r.state == "published" || r.state == "expired" {
                        "X:remove E:expire L:list N/P:more"
                    } else {
                        "Private draft; author must submit. L:list"
                    }
                } else if r.state == "draft" {
                    "S:submit W:withdraw L:list N/P:more"
                } else {
                    "W:withdraw L:list N/P:more"
                };
            }
            State::ReviewDate(r) => {
                body.push(format!("Approve exact revision of #{}", r.id));
                controls = "C:cancel / dates use YYYY-MM-DD";
                prompt = "Reviewed date: ".into();
            }
            State::ExpiryDate(_, date) => {
                body.push(format!("Reviewed date: {date}"));
                controls = "C:cancel / dates use YYYY-MM-DD";
                prompt = "Valid until date: ".into();
            }
            State::Confirm {
                row,
                action,
                reviewed,
                until,
            } => {
                body.push(format!(
                    "{} submission #{}",
                    action.to_ascii_uppercase(),
                    row.id
                ));
                body.push(format!("Exact revision: {}", row.revision));
                if let Some(v) = reviewed {
                    body.push(format!("Reviewed: {v}"));
                }
                if let Some(v) = until {
                    body.push(format!("Valid until: {v}"));
                }
                controls = "Type the uppercase action to confirm.\nAnything else cancels.";
            }
        }
        let footer = wrap(
            vec![
                controls.into(),
                "Q:back / no connection made".into(),
                prompt,
            ],
            width,
        );
        let mut head = vec!["Directory submissions".to_owned()];
        if !self.notice.is_empty() {
            head.extend(wrap(vec![std::mem::take(&mut self.notice)], width));
        }
        let body = wrap(body, width);
        let count = height.saturating_sub(head.len() + footer.len()).max(1);
        let start =
            (self.page.saturating_mul(count)).min(body.len().saturating_sub(1) / count * count);
        head.extend(body.into_iter().skip(start).take(count));
        head.extend(footer);
        let mut out = String::new();
        if ansi {
            let (normal, _, _) = theme.menu_styles();
            out.push_str("\x1b[2J\x1b[H");
            out.push_str(&normal.sgr());
        }
        for (i, line) in head.iter().enumerate() {
            out.push_str(line);
            if i + 1 < head.len() {
                out.push('\n');
            }
        }
        if ansi {
            out.push_str("\x1b[0m");
        }
        if out.len() > 8192 {
            return Err(Error::Invalid("directory submission frame bound"));
        }
        Ok(out)
    }
    pub fn feed(&mut self, input: &str, b: &mut Board, s: &Session) -> Result<bool> {
        let role = actor(b, s, self.review)?;
        if input.len() > 128 || !input.bytes().all(|v| v.is_ascii_graphic() || v == b' ') {
            return Err(Error::Invalid("bounded printable ASCII required"));
        }
        if input.eq_ignore_ascii_case("q") {
            return Ok(true);
        }
        self.notice.clear();
        match &mut self.state {
            State::List => match input.to_ascii_lowercase().as_str() {
                "n" | "c" => {
                    self.page = 0;
                    self.review = false;
                    self.state = State::Input {
                        fields: Default::default(),
                        index: 0,
                        nonce: request_id()?,
                    };
                }
                ">" => {
                    if let Some(r) = self.shown.last() {
                        self.after = r.id;
                    }
                }
                "b" => self.after = 0,
                "m" => {
                    if role != Role::Sysop {
                        return Err(Error::Denied);
                    }
                    self.review = true;
                    self.after = 0;
                }
                "l" => {
                    self.review = false;
                    self.after = 0;
                }
                "1" | "2" | "3" | "4" => {
                    let i = input.parse::<usize>().unwrap() - 1;
                    if let Some(row) = self.shown.get(i) {
                        if !self.review && !row.mine {
                            return Err(Error::Denied);
                        }
                        self.state = State::Card(row.clone());
                        self.page = 0;
                    }
                }
                _ => (),
            },
            State::Input {
                fields,
                index,
                nonce,
            } => {
                if input.eq_ignore_ascii_case("c") {
                    self.state = State::List;
                } else {
                    fields[*index] = input.to_owned();
                    if *index < 6 {
                        *index += 1;
                    } else {
                        let draft = DirectoryDraft {
                            slug: fields[0].clone(),
                            name: fields[1].clone(),
                            summary: fields[2].clone(),
                            host: fields[3].clone(),
                            port: fields[4]
                                .parse()
                                .map_err(|_| Error::Invalid("SSH port must be 1-65535"))?,
                            login: fields[5].clone(),
                            fingerprint: (!fields[6].is_empty()).then(|| fields[6].clone()),
                        }
                        .canonicalized()?;
                        self.state = State::Preview {
                            draft,
                            nonce: nonce.clone(),
                        };
                        self.page = 0;
                    }
                }
            }
            State::Preview { draft, nonce } => match input.to_ascii_lowercase().as_str() {
                "s" => {
                    let id = b.directory_save(s, nonce, draft)?;
                    self.state = State::Card(b.directory_submission(s, id)?);
                    self.page = 0;
                    self.notice = "Private draft saved; S submits for review.".into();
                }
                "c" => {
                    self.state = State::List;
                    self.page = 0;
                    self.notice = "Unsaved listing cancelled.".into();
                }
                "n" => self.page = self.page.saturating_add(1),
                "p" => self.page = self.page.saturating_sub(1),
                _ => (),
            },
            State::Card(row) => match input.to_ascii_lowercase().as_str() {
                "l" => {
                    self.state = State::List;
                    self.page = 0;
                }
                "n" => self.page = self.page.saturating_add(1),
                "p" => self.page = self.page.saturating_sub(1),
                "s" if !self.review && row.mine && row.state == "draft" => {
                    b.directory_submit(s, row.id, &row.revision)?;
                    self.state = State::Card(b.directory_submission(s, row.id)?);
                    self.notice = "Submitted pending sysop review.".into();
                    self.page = 0;
                }
                "w" if !self.review && row.mine => {
                    b.directory_withdraw(s, row.id, &row.revision)?;
                    self.state = State::List;
                    self.notice = "Submission withdrawn and removed.".into();
                    self.page = 0;
                }
                "a" if self.review && row.state == "pending" => {
                    self.state = State::ReviewDate(row.clone());
                    self.page = 0;
                }
                "r" if self.review && row.state == "pending" => {
                    self.state = State::Confirm {
                        row: row.clone(),
                        action: "reject",
                        reviewed: None,
                        until: None,
                    }
                }
                "x" if self.review && (row.state == "published" || row.state == "expired") => {
                    self.state = State::Confirm {
                        row: row.clone(),
                        action: "remove",
                        reviewed: None,
                        until: None,
                    }
                }
                "e" if self.review && row.state == "published" => {
                    self.state = State::Confirm {
                        row: row.clone(),
                        action: "expire",
                        reviewed: None,
                        until: None,
                    }
                }
                _ => (),
            },
            State::ReviewDate(row) => {
                if input.eq_ignore_ascii_case("c") {
                    self.state = State::Card(row.clone());
                } else {
                    faction_terminal::seasons::Date::parse(input).map_err(Error::Invalid)?;
                    self.state = State::ExpiryDate(row.clone(), input.into());
                }
            }
            State::ExpiryDate(row, date) => {
                if input.eq_ignore_ascii_case("c") {
                    self.state = State::Card(row.clone());
                } else {
                    let end =
                        faction_terminal::seasons::Date::parse(input).map_err(Error::Invalid)?;
                    let start =
                        faction_terminal::seasons::Date::parse(date).map_err(Error::Invalid)?;
                    if end.fields() < start.fields() {
                        return Err(Error::Invalid("expiry precedes review"));
                    }
                    self.state = State::Confirm {
                        row: row.clone(),
                        action: "approve",
                        reviewed: Some(date.clone()),
                        until: Some(input.into()),
                    };
                }
            }
            State::Confirm {
                row,
                action,
                reviewed,
                until,
            } => {
                if input == action.to_ascii_uppercase() {
                    b.directory_review(
                        s,
                        row.id,
                        &row.revision,
                        action,
                        reviewed.as_deref(),
                        until.as_deref(),
                    )?;
                    self.notice = format!("Directory {action} applied.");
                    self.state = State::List;
                } else {
                    self.state = State::Card(row.clone());
                }
                self.page = 0;
            }
        }
        Ok(false)
    }
    pub fn recover(&mut self, e: &Error) -> bool {
        if matches!(
            e,
            Error::Invalid(_) | Error::NotFound | Error::RequestConflict | Error::Busy
        ) {
            self.notice = if matches!(e, Error::RequestConflict) {
                "Submission changed; reopen before acting.".into()
            } else {
                format!("Directory: {e}")
            };
            if matches!(e, Error::NotFound | Error::RequestConflict) {
                self.state = State::List;
                self.page = 0;
            }
            true
        } else {
            false
        }
    }
}
