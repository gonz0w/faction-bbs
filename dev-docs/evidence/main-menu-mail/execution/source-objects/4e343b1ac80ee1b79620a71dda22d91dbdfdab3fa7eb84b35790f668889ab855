//! Owner-reviewed local editions. Consent/provenance are declarations, not signatures.
use super::*;
use catalog::{check_reference, slug, tables, text};
use toml::{Table, Value};
pub(super) struct Issue {
    pub id: String,
    pub title: String,
    pub number: u16,
    pub revision: u16,
    pub state: String,
    pub editor: String,
    pub released: String,
    pub schedule: String,
    pub download: Option<String>,
    pub room: Option<i64>,
    pub articles: Vec<Article>,
}
pub(super) struct Article {
    pub id: String,
    pub title: String,
    pub section: String,
    pub author: String,
    pub rights: String,
    pub provenance: String,
    pub consent: String,
    pub body: String,
}
fn integer(t: &mut Table, k: &str) -> catalog::Result<u16> {
    match t.remove(k) {
        Some(Value::Integer(n)) if (1..=9999).contains(&n) => Ok(n as u16),
        _ => Err("issue number/revision needs 1-9999"),
    }
}
pub(super) fn parse(t: &mut Table) -> catalog::Result<Vec<Issue>> {
    if !t.contains_key("magazine") {
        return Ok(vec![]);
    }
    let mut issues: Vec<Issue> = vec![];
    let mut total = 0;
    for mut row in tables(t, "magazine", 8)? {
        let id = text(&mut row, "id", 32, false)?;
        slug(&id)?;
        let title = text(&mut row, "title", 64, false)?;
        let number = integer(&mut row, "number")?;
        let revision = integer(&mut row, "revision")?;
        if issues.iter().any(|i| i.id == id || i.number == number) {
            return Err("duplicate issue ID/number");
        }
        let state = text(&mut row, "state", 16, false)?;
        if !["draft", "published", "withdrawn"].contains(&state.as_str()) {
            return Err("invalid issue state");
        }
        let editor = text(&mut row, "editor", 64, false)?;
        let released = text(&mut row, "released", 32, false)?;
        let schedule = text(&mut row, "schedule", 64, false)?;
        let download = if row.contains_key("download") {
            let s = text(&mut row, "download", 78, false)?;
            check_reference(&s)?;
            Some(s)
        } else {
            None
        };
        let room = match row.remove("room") {
            None => None,
            Some(Value::Integer(n)) if n > 0 => Some(n),
            _ => return Err("positive discussion room required"),
        };
        let mut articles: Vec<Article> = vec![];
        for mut a in tables(&mut row, "article", 8)? {
            total += 1;
            if total > 32 {
                return Err("32 articles per pack");
            }
            let id = text(&mut a, "id", 32, false)?;
            slug(&id)?;
            if articles.iter().any(|v| v.id == id) {
                return Err("duplicate article ID");
            }
            let title = text(&mut a, "title", 64, false)?;
            let section = text(&mut a, "section", 24, false)?;
            let author = text(&mut a, "author", 64, false)?;
            let rights = text(&mut a, "rights", 96, false)?;
            let provenance = text(&mut a, "provenance", 160, false)?;
            let consent = text(&mut a, "consent", 16, false)?;
            if !["pending", "board-local"].contains(&consent.as_str())
                || (state == "published" && consent != "board-local")
            {
                return Err("published articles need declared board-local consent");
            }
            let body = text(&mut a, "body", 8192, true)?;
            if body.split('\n').count() > 128 || body.split('\n').any(|l| l.len() > 120) {
                return Err("article line budget");
            }
            if [&title, &section, &author, &rights, &provenance, &body]
                .iter()
                .any(|s| s.contains("faction-file:"))
            {
                return Err("file references belong in the issue download field");
            }
            if !a.is_empty() {
                return Err("unknown article field");
            }
            articles.push(Article {
                id,
                title,
                section,
                author,
                rights,
                provenance,
                consent,
                body,
            });
        }
        if !row.is_empty() || articles.is_empty() {
            return Err("issue needs articles and known fields");
        }
        issues.push(Issue {
            id,
            title,
            number,
            revision,
            state,
            editor,
            released,
            schedule,
            download,
            room,
            articles,
        });
    }
    Ok(issues)
}
impl Catalog {
    pub fn published_issues(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.state == "published")
            .count()
    }
    /// Offline owner review, including unshared drafts. Never called by member UI.
    pub fn magazine_review(&self) -> String {
        self.issues
            .iter()
            .map(|i| {
                format!(
                    "{} #{} rev {} [{}] / {} articles\n",
                    i.id,
                    i.number,
                    i.revision,
                    i.state,
                    i.articles.len()
                )
            })
            .collect()
    }
    /// Bounded plain owner preview/export. New file publication still uses Files.
    pub fn magazine_text(&self, id: &str, preview: bool) -> catalog::Result<String> {
        let i = self
            .issues
            .iter()
            .find(|i| i.id == id)
            .ok_or("issue unavailable")?;
        if !preview && i.state != "published" {
            return Err("only published editions can be exported");
        }
        let mut out = format!(
            "{} / #{} / revision {} [{}]\n{}\nEditor: {}\nEdition: {}\nSchedule: {}\nLOCAL EDITION. No automatic public redistribution authority.\n",
            self.magazine_name,
            i.number,
            i.revision,
            i.state,
            i.title,
            i.editor,
            i.released,
            i.schedule
        );
        for a in &i.articles {
            out.push_str(&format!(
                "\n== {} / {} ==\nBy {}\nRights: {}\nProvenance: {}\nDeclared consent: {}\n\n{}\n",
                a.section, a.title, a.author, a.rights, a.provenance, a.consent, a.body
            ));
        }
        if out.len() > 65536 {
            return Err("edition text budget");
        }
        Ok(out)
    }
}
#[derive(Clone, Copy, Default)]
enum State {
    #[default]
    Issues,
    ChooseIssue,
    Contents(usize),
    ChooseArticle(usize),
    Read(usize, usize),
    Credits(usize, usize),
    Link(usize),
}
#[derive(Default)]
pub(super) struct Desk {
    state: State,
    page: usize,
    offset: usize,
    next: Option<usize>,
    previous: Vec<usize>,
    columns: Option<u16>,
    notice: String,
}
fn issue(c: &Catalog, i: usize) -> Result<&Issue> {
    c.issues
        .get(i)
        .filter(|i| i.state == "published")
        .ok_or(Error::NotFound)
}
fn file(b: &mut Board, s: &Session, i: &Issue) -> Result<Option<AreaFile>> {
    match &i.download {
        None => Ok(None),
        Some(reference) => match b.file_resolve(s, reference) {
            Ok(f) => Ok(Some(f)),
            Err(Error::Denied | Error::NotFound) => Ok(None),
            Err(e) => Err(e),
        },
    }
}
impl Desk {
    fn reset(&mut self) {
        self.offset = 0;
        self.next = None;
        self.previous.clear();
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render(
        &mut self,
        b: &mut Board,
        s: &Session,
        c: &Catalog,
        t: &Theme,
        ansi: bool,
        columns: u16,
        files: bool,
    ) -> Result<String> {
        if self.columns != Some(columns) {
            self.reset();
        }
        self.columns = Some(columns);
        let width = usize::from(columns).clamp(20, 80) - 1;
        let mut out = faction_terminal::section(t, &c.magazine_name, ansi);
        if !self.notice.is_empty() {
            writeln!(out, "{}", preview(&self.notice, width)).unwrap();
        }
        match self.state {
            State::Issues => {
                let issues = c
                    .issues
                    .iter()
                    .filter(|i| i.state == "published")
                    .collect::<Vec<_>>();
                self.page = self.page.min(issues.len().saturating_sub(1) / 4);
                if issues.is_empty() {
                    out.push_str("No local editions available.\n");
                }
                for i in issues.iter().skip(self.page * 4).take(4) {
                    writeln!(
                        out,
                        "{}",
                        preview(&format!("{}: #{} {}", i.id, i.number, i.title), width)
                    )
                    .unwrap();
                    if !c.compact {
                        writeln!(
                            out,
                            " revision {} / {}",
                            i.revision,
                            preview(&i.released, width.saturating_sub(18))
                        )
                        .unwrap();
                    }
                }
                out.push_str("V:issue N/P:page Q:library\nMagazine> ");
            }
            State::ChooseIssue => out.push_str("Issue ID (blank cancels): "),
            State::Contents(n) => {
                let i = issue(c, n)?;
                writeln!(
                    out,
                    "#{} rev {} / {}",
                    i.number,
                    i.revision,
                    preview(&i.title, width.saturating_sub(20))
                )
                .unwrap();
                self.page = self.page.min(i.articles.len().saturating_sub(1) / 4);
                for a in i.articles.iter().skip(self.page * 4).take(4) {
                    writeln!(out, "{}", preview(&format!("{}: {}", a.id, a.title), width)).unwrap();
                    if !c.compact {
                        writeln!(
                            out,
                            " {} / {}",
                            preview(&a.section, 12),
                            preview(&a.author, width.saturating_sub(17))
                        )
                        .unwrap();
                    }
                }
                out.push_str("R:article N/P:contents ");
                if file(b, s, i)?.is_some() {
                    out.push_str("L:edition link ");
                    if files {
                        out.push_str("F:Files ");
                    }
                }
                if let Some(room) = i.room {
                    match b.unread_cursor(s, room) {
                        Ok(_) => out.push_str("D:discussion "),
                        Err(Error::NotFound) => (),
                        Err(e) => return Err(e),
                    }
                }
                out.push_str("I:issue notes Q:editions\nContents> ");
            }
            State::ChooseArticle(n) => {
                issue(c, n)?;
                out.push_str("Article ID (blank cancels): ");
            }
            State::Read(n, a) | State::Credits(n, a) => {
                let i = issue(c, n)?;
                writeln!(out, "#{} revision {}", i.number, i.revision).unwrap();
                let text = if a == usize::MAX {
                    format!(
                        "{}\nEditor: {}\nEdition: {}\nSchedule: {}\nLocal release; not public distribution permission.\n",
                        i.title, i.editor, i.released, i.schedule
                    )
                } else {
                    let a = i.articles.get(a).ok_or(Error::NotFound)?;
                    if matches!(self.state, State::Credits(..)) {
                        format!(
                            "{}\nAuthor: {}\nSection: {}\nRights: {}\nProvenance: {}\nDeclared consent: {}\n",
                            a.title, a.author, a.section, a.rights, a.provenance, a.consent
                        )
                    } else {
                        format!("{}\nBy {}\n\n{}", a.title, a.author, a.body)
                    }
                };
                self.next = lines(&mut out, &text, self.offset, width);
                out.push_str("N/P:page ");
                if a != usize::MAX {
                    out.push_str("C:credits R:read ");
                }
                out.push_str("T:contents Q:editions\nMagazine page> ");
            }
            State::Link(n) => {
                let i = issue(c, n)?;
                let f = file(b, s, i)?.ok_or(Error::NotFound)?;
                writeln!(
                    out,
                    "{}\n{}",
                    preview(&f.filename, width),
                    i.download.as_ref().ok_or(Error::NotFound)?
                )
                .unwrap();
                out.push_str("Current file permissions and transfer confirmation apply.\n");
                if files {
                    out.push_str("F:Files ");
                }
                out.push_str("T:contents Q:editions\nEdition link> ");
            }
        }
        if out.len() > 8192 {
            return Err(Error::Invalid("magazine frame budget"));
        }
        Ok(out)
    }
    pub(super) fn feed(
        &mut self,
        input: &str,
        b: &mut Board,
        s: &Session,
        c: &Catalog,
        files: bool,
    ) -> Result<Outcome> {
        if input.len() > 64 || !input.bytes().all(|b| (32..=126).contains(&b)) {
            return Err(Error::Invalid("bounded ASCII magazine command required"));
        }
        self.notice.clear();
        let cmd = input.to_ascii_lowercase();
        match self.state {
            State::Issues => match cmd.as_str() {
                "q" => return Ok(Outcome::Main),
                "v" => self.state = State::ChooseIssue,
                "n" => self.page = (self.page + 1).min(1),
                "p" => self.page = self.page.saturating_sub(1),
                _ => return Err(Error::Invalid("use displayed magazine commands")),
            },
            State::ChooseIssue => {
                if cmd.is_empty() {
                    self.state = State::Issues;
                } else {
                    let n = c
                        .issues
                        .iter()
                        .position(|i| i.id == cmd && i.state == "published")
                        .ok_or(Error::NotFound)?;
                    self.state = State::Contents(n);
                    self.page = 0;
                    self.reset();
                }
            }
            State::ChooseArticle(n) => {
                let i = issue(c, n)?;
                if cmd.is_empty() {
                    self.state = State::Contents(n);
                } else {
                    let a = i
                        .articles
                        .iter()
                        .position(|a| a.id == cmd)
                        .ok_or(Error::NotFound)?;
                    self.state = State::Read(n, a);
                    self.reset();
                }
            }
            State::Contents(n) | State::Link(n) => {
                let i = issue(c, n)?;
                match cmd.as_str() {
                    "q" => {
                        self.state = State::Issues;
                        self.page = 0;
                    }
                    "t" => {
                        self.state = State::Contents(n);
                        self.page = 0;
                    }
                    "r" if matches!(self.state, State::Contents(_)) => {
                        self.state = State::ChooseArticle(n)
                    }
                    "i" if matches!(self.state, State::Contents(_)) => {
                        self.state = State::Credits(n, usize::MAX);
                        self.reset();
                    }
                    "n" if matches!(self.state, State::Contents(_)) => {
                        self.page = (self.page + 1).min(1)
                    }
                    "p" if matches!(self.state, State::Contents(_)) => {
                        self.page = self.page.saturating_sub(1)
                    }
                    "l" => {
                        file(b, s, i)?.ok_or(Error::NotFound)?;
                        self.state = State::Link(n);
                    }
                    "f" if files => {
                        let f = file(b, s, i)?.ok_or(Error::NotFound)?;
                        return Ok(Outcome::File(f.id));
                    }
                    "d" if matches!(self.state, State::Contents(_)) => {
                        let room = i.room.ok_or(Error::NotFound)?;
                        b.unread_cursor(s, room)?;
                        return Ok(Outcome::Room(room));
                    }
                    _ => return Err(Error::Invalid("use displayed contents commands")),
                }
            }
            State::Read(n, a) | State::Credits(n, a) => {
                issue(c, n)?;
                match cmd.as_str() {
                    "q" => {
                        self.state = State::Issues;
                        self.page = 0;
                        self.reset();
                    }
                    "t" => {
                        self.state = State::Contents(n);
                        self.page = 0;
                        self.reset();
                    }
                    "c" if a != usize::MAX => {
                        self.state = State::Credits(n, a);
                        self.reset();
                    }
                    "r" if a != usize::MAX => {
                        self.state = State::Read(n, a);
                        self.reset();
                    }
                    "n" => {
                        if let Some(next) = self.next {
                            if self.previous.len() == 32 {
                                self.previous.remove(0);
                            }
                            self.previous.push(self.offset);
                            self.offset = next;
                        }
                    }
                    "p" => self.offset = self.previous.pop().unwrap_or(0),
                    _ => return Err(Error::Invalid("use displayed reading commands")),
                }
            }
        }
        Ok(Outcome::Stay)
    }
    pub(super) fn recover(&mut self, e: &Error) -> bool {
        if matches!(e, Error::Invalid(_) | Error::NotFound | Error::Busy) {
            self.notice = format!("Magazine: {e}");
            if matches!(e, Error::NotFound) {
                self.state = State::Issues;
                self.page = 0;
                self.reset();
            }
            true
        } else {
            false
        }
    }
}
