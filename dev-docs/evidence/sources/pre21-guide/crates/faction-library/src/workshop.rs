//! Shared local editorial workflow: no paths, network or private-mail authority.
use super::*;
use faction_core::{
    MagazineIssue, MagazineIssueDraft, MagazineSubmission, MagazineText, Role, request_id,
};
#[derive(Clone, Copy, Default)]
enum Stage {
    #[default]
    Home,
    Submissions,
    ChooseSource,
    Source,
    SourceForm(u8),
    Body,
    SourceReview,
    Issues,
    ChooseIssue,
    Issue,
    IssueForm(u8),
    IssueReview,
    Slot,
    SelectSource,
    EditSlot,
    Offer,
    Link,
    Confirm(Action),
}
#[derive(Clone, Copy)]
enum Action {
    Save(bool),
    WithdrawSource(bool),
    Approve(bool),
    Publish,
    WithdrawIssue(bool),
    Offer,
}
#[derive(Default)]
pub(super) struct Desk {
    stage: Stage,
    source: Option<(i64, String)>,
    issue: Option<(i64, String)>,
    nonce: String,
    text: Option<MagazineText>,
    header: Option<MagazineIssueDraft>,
    editing_entry: bool,
    slot: u8,
    object: String,
    all: bool,
    through: Option<i64>,
    last: Option<i64>,
    list_back: Vec<i64>,
    offset: usize,
    next: Option<usize>,
    back: Vec<usize>,
    columns: Option<u16>,
    finished: bool,
    notice: String,
}
impl Desk {
    fn reset(&mut self) {
        self.offset = 0;
        self.next = None;
        self.back.clear();
        self.finished = false;
    }
    fn home(&mut self) {
        self.stage = Stage::Home;
        self.source = None;
        self.issue = None;
        self.text = None;
        self.header = None;
        self.object.clear();
        self.editing_entry = false;
        self.reset();
    }
    fn source(&self, b: &mut Board, s: &Session) -> Result<MagazineSubmission> {
        let (id, rev) = self.source.as_ref().ok_or(Error::NotFound)?;
        let v = b.magazine_submission(s, *id)?;
        if v.revision != *rev {
            return Err(Error::Invalid("submission changed; reopen it"));
        }
        Ok(v)
    }
    fn issue(&self, b: &mut Board, s: &Session) -> Result<MagazineIssue> {
        let (id, token) = self.issue.as_ref().ok_or(Error::NotFound)?;
        let v = b.magazine_issue(s, *id)?;
        if v.token != *token {
            return Err(Error::Invalid("issue changed; reopen it"));
        }
        Ok(v)
    }
    fn open_issue(&mut self, b: &mut Board, s: &Session, id: i64) -> Result<()> {
        let v = b.magazine_issue(s, id)?;
        self.issue = Some((id, v.token));
        self.stage = Stage::Issue;
        self.reset();
        Ok(())
    }
    fn open_source(&mut self, b: &mut Board, s: &Session, id: i64) -> Result<()> {
        let v = b.magazine_submission(s, id)?;
        self.source = Some((id, v.revision));
        self.stage = Stage::Source;
        self.reset();
        Ok(())
    }
    fn page(&mut self, out: &mut String, text: &str, width: usize) {
        self.next = lines(out, text, self.offset, width);
        if self.next.is_none() {
            self.finished = true;
            out.push_str("[End of review]\n");
        } else {
            out.push_str("[More review text]\n");
        }
        out.push_str("N/P:review pages ");
    }
    fn back_page(&mut self, cmd: &str) -> bool {
        match cmd {
            "n" => {
                if let Some(n) = self.next {
                    if self.back.len() == 32 {
                        self.back.remove(0);
                    }
                    self.back.push(self.offset);
                    self.offset = n;
                }
                true
            }
            "p" => {
                self.offset = self.back.pop().unwrap_or(0);
                true
            }
            _ => false,
        }
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
        let sysop = b.whoami(s)?.role == Role::Sysop;
        if self.columns != Some(columns) {
            self.reset();
        }
        self.columns = Some(columns);
        let width = usize::from(columns).clamp(20, 80) - 1;
        let mut out = faction_terminal::section(t, &c.magazine_name, ansi);
        out.push_str("Magazine workshop / local board only\n");
        if !self.notice.is_empty() {
            writeln!(out, "{}", preview(&self.notice, width)).unwrap();
        }
        match self.stage {
            Stage::Home => {
                out.push_str(
                    "I:editions & contributor reviews\nS:your private drafts/submissions\n",
                );
                if sysop {
                    out.push_str("A:editor submission queue C:new draft issue\n");
                }
                out.push_str("Q:library\nWorkshop> ");
            }
            Stage::Submissions => {
                let rows = b.magazine_submissions(s, self.all, self.through.unwrap_or(i64::MAX))?;
                self.last = rows.last().map(|v| v.id);
                if rows.is_empty() {
                    out.push_str("No submissions on this page.\n");
                }
                for v in rows {
                    writeln!(
                        out,
                        "{}",
                        preview(
                            &format!("{} [{}] {} / {}", v.id, v.state, v.author, v.text.title),
                            width
                        )
                    )
                    .unwrap();
                }
                out.push_str("V:view C:new draft N/P:page R:refresh Q:workshop\nSubmissions> ");
            }
            Stage::ChooseSource => out.push_str("Submission ID (blank cancels): "),
            Stage::Source => {
                let v = self.source(b, s)?;
                writeln!(out, "Submission {} [{}] / {}", v.id, v.state, v.author).unwrap();
                let text = text_preview(&v.text);
                self.page(&mut out, &text, width);
                if v.mine && v.state != "withdrawn" {
                    out.push_str("E:edit ");
                }
                if v.mine || sysop {
                    out.push_str("X:withdraw Z:delete ");
                }
                out.push_str("Q:workshop\nSubmission> ");
            }
            Stage::SourceForm(n) => {
                if let Some((id, _)) = self.issue.as_ref().filter(|_| self.editing_entry) {
                    b.magazine_issue(s, *id)?;
                }
                out.push_str(
                    [
                        "Title (1-64; blank keeps/cancels): ",
                        "Section (1-24; blank keeps): ",
                        "Distribution rights (1-96; blank keeps): ",
                        "Provenance (1-128; blank keeps): ",
                    ][n as usize],
                );
            }
            Stage::Body => {
                writeln!(
                    out,
                    "Body: {} bytes / max 4096. Printable ASCII lines up to 120.",
                    self.text.as_ref().ok_or(Error::NotFound)?.body.len()
                )
                .unwrap();
                out.push_str(
                    "Lines append. .undo .clear .review .cancel; .. for literal dot.\nManuscript> ",
                );
            }
            Stage::SourceReview => {
                let text = text_preview(self.text.as_ref().ok_or(Error::NotFound)?);
                self.page(&mut out, &text, width);
                if self.editing_entry {
                    out.push_str("S:save editor proposal (approvals reset) ");
                } else {
                    out.push_str("S:save private draft T:submit for local review ");
                }
                out.push_str("Q:cancel\nManuscript review> ");
            }
            Stage::Issues => {
                let rows = b.magazine_issues(s, self.through.unwrap_or(i64::MAX))?;
                self.last = rows.last().map(|v| v.id);
                if rows.is_empty() {
                    out.push_str("No eligible editions/reviews on this page.\n");
                }
                for v in rows {
                    writeln!(
                        out,
                        "{}",
                        preview(
                            &format!(
                                "{}: #{} r{} [{}] {}",
                                v.id, v.number, v.edition, v.state, v.title
                            ),
                            width
                        )
                    )
                    .unwrap();
                }
                out.push_str("V:view N/P:page R:refresh Q:workshop\nEditions> ");
            }
            Stage::ChooseIssue => out.push_str("Issue ID (blank cancels): "),
            Stage::Issue => {
                let v = self.issue(b, s)?;
                writeln!(
                    out,
                    "Issue {} [{}] / approvals {}/{}",
                    v.id,
                    v.state,
                    v.entries
                        .iter()
                        .filter(|e| e.approved && (v.state != "draft" || e.current))
                        .count(),
                    v.entries.len()
                )
                .unwrap();
                self.page(&mut out, &v.text(), width);
                if v.entries.iter().any(|e| e.mine) {
                    if v.state == "draft" {
                        out.push_str("A:approve exact issue ");
                    }
                    out.push_str("U:revoke your approval ");
                }
                if sysop {
                    if v.state == "draft" {
                        out.push_str("H:header S:select slot E:edit proposal PUBLISH:publish ");
                    }
                    out.push_str("X:withdraw Z:delete ");
                    if v.state == "published" {
                        out.push_str("O:related file ");
                    }
                }
                if b.magazine_file(s, v.id)?.is_some() {
                    out.push_str("L:file link ");
                    if files {
                        out.push_str("F:Files ");
                    }
                }
                out.push_str("Q:workshop\nIssue review> ");
            }
            Stage::IssueForm(n) => {
                out.push_str(
                    [
                        "Issue number (1-9999; blank keeps/cancels): ",
                        "Edition revision (1-9999; blank keeps): ",
                        "Issue title (1-64; blank keeps): ",
                        "Edition label (1-32; blank keeps): ",
                        "Schedule label (1-64; blank keeps): ",
                    ][n as usize],
                );
            }
            Stage::IssueReview => {
                let h = self.header.as_ref().ok_or(Error::NotFound)?;
                let text = format!(
                    "{} / #{} revision {}\n{}\nEdition: {}\nSchedule: {}\nLocal approved-member audience. No public publication.\n",
                    h.name, h.number, h.edition, h.title, h.released, h.schedule
                );
                self.page(&mut out, &text, width);
                out.push_str("Save draft issue? YES or Q: ");
            }
            Stage::Slot => {
                self.issue(b, s)?;
                out.push_str("Slot 1-8 to select/replace (blank cancels): ");
            }
            Stage::SelectSource => {
                self.issue(b, s)?;
                out.push_str("Submitted manuscript ID (0 removes slot): ");
            }
            Stage::EditSlot => {
                self.issue(b, s)?;
                out.push_str("Slot 1-8 to revise proposal (blank cancels): ");
            }
            Stage::Offer => {
                self.issue(b, s)?;
                out.push_str("Published related file ID/link (- removes; blank cancels): ");
            }
            Stage::Link => {
                let v = self.issue(b, s)?;
                let object = b.magazine_file(s, v.id)?.ok_or(Error::NotFound)?;
                writeln!(out, "{}", b.file_reference(s, &object)?).unwrap();
                out.push_str("Sysop-curated related file; existing Files confirmation applies.\n");
                if files {
                    out.push_str("F:Files ");
                }
                out.push_str("Q:issue\nWorkshop link> ");
            }
            Stage::Confirm(action) => {
                match action {
                    Action::Save(submit) => {
                        if self.editing_entry {
                            self.issue(b, s)?;
                        } else if self.source.is_some() {
                            self.source(b, s)?;
                        }
                        out.push_str(if self.editing_entry{"Save changed editorial proposal? All contributor approvals reset.\n"}else if submit{"Submit to local sysop editors and selected contributors for review?\nThis does not approve publication. Rights/source remain your responsibility.\n"}else{"Save private board draft? Readable by its author and sysop.\n"});
                    }
                    Action::WithdrawSource(delete) => {
                        self.source(b, s)?;
                        out.push_str(if delete {
                            "Delete unselected submission from live desk? Backups remain.\n"
                        } else {
                            "Withdraw submission? Published issues using it are withdrawn too.\n"
                        });
                    }
                    Action::Approve(approve) => {
                        self.issue(b, s)?;
                        out.push_str(if approve{"Approve ALL your selected contributions in this exact complete issue\nfor this board's approved-member audience? No public/Gopher/NNTP consent.\n"}else{"Revoke your approval? A published issue will be withdrawn.\n"});
                    }
                    Action::Publish => {
                        self.issue(b, s)?;
                        out.push_str("Publish this exact issue locally only if every contributor approved?\n");
                    }
                    Action::WithdrawIssue(delete) => {
                        self.issue(b, s)?;
                        out.push_str(if delete{"Delete retained non-published issue and its approval records? Backups remain.\n"}else{"Withdraw this issue from ordinary readers now? Existing copies remain.\n"});
                    }
                    Action::Offer => {
                        self.issue(b, s)?;
                        out.push_str("Set/remove this sysop-curated related file? Article approvals stay unchanged.\nThis does not verify the file matches the approved edition text.\n");
                    }
                }
                out.push_str("Confirm YES or Q: ");
            }
        }
        if out.len() > 8192 {
            return Err(Error::Invalid("workshop frame budget"));
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
        if input.len() > 128 || !input.bytes().all(|v| (32..=126).contains(&v)) {
            return Err(Error::Invalid("bounded ASCII workshop input required"));
        }
        let sysop = b.whoami(s)?.role == Role::Sysop;
        let cmd = input.to_ascii_lowercase();
        self.notice.clear();
        if matches!(
            self.stage,
            Stage::Source | Stage::Issue | Stage::SourceReview | Stage::IssueReview
        ) {
            if matches!(self.stage, Stage::Source) {
                self.source(b, s)?;
            }
            if matches!(self.stage, Stage::Issue) {
                self.issue(b, s)?;
            }
            if self.back_page(&cmd) {
                return Ok(Outcome::Stay);
            }
        }
        match self.stage {
            Stage::Home => match cmd.as_str() {
                "q" => return Ok(Outcome::Main),
                "s" | "a" if cmd == "s" || sysop => {
                    self.all = cmd == "a";
                    self.stage = Stage::Submissions;
                    self.through = None;
                    self.list_back.clear();
                }
                "i" => {
                    self.stage = Stage::Issues;
                    self.through = None;
                    self.list_back.clear();
                }
                "c" if sysop => {
                    self.issue = None;
                    self.nonce = request_id()?;
                    self.header = Some(MagazineIssueDraft {
                        number: 0,
                        edition: 1,
                        name: c.magazine_name.clone(),
                        title: String::new(),
                        released: String::new(),
                        schedule: "When ready".into(),
                    });
                    self.stage = Stage::IssueForm(0);
                    self.reset();
                }
                _ => return Err(Error::Invalid("use displayed workshop commands")),
            },
            Stage::Submissions | Stage::Issues => match cmd.as_str() {
                "q" => self.home(),
                "v" => {
                    self.stage = if matches!(self.stage, Stage::Submissions) {
                        Stage::ChooseSource
                    } else {
                        Stage::ChooseIssue
                    }
                }
                "c" if matches!(self.stage, Stage::Submissions) => {
                    self.source = None;
                    self.issue = None;
                    self.editing_entry = false;
                    self.nonce = request_id()?;
                    self.text = Some(MagazineText {
                        title: String::new(),
                        section: String::new(),
                        rights: String::new(),
                        provenance: String::new(),
                        body: String::new(),
                    });
                    self.stage = Stage::SourceForm(0);
                    self.reset();
                }
                "r" => {
                    self.through = None;
                    self.list_back.clear();
                }
                "n" => {
                    if let Some(last) = self.last
                        && self.list_back.len() < 32
                    {
                        self.list_back.push(self.through.unwrap_or(i64::MAX));
                        self.through = Some(last.saturating_sub(1));
                    }
                }
                "p" => self.through = self.list_back.pop(),
                _ => return Err(Error::Invalid("use displayed list commands")),
            },
            Stage::ChooseSource => {
                if input.is_empty() {
                    self.stage = Stage::Submissions;
                } else {
                    self.open_source(b, s, positive(input)?)?;
                }
            }
            Stage::Source => {
                let v = self.source(b, s)?;
                match cmd.as_str() {
                    "q" => self.home(),
                    "e" if v.mine && v.state != "withdrawn" => {
                        self.text = Some(v.text);
                        self.editing_entry = false;
                        self.stage = Stage::SourceForm(0);
                        self.reset();
                    }
                    "x" | "z" if v.mine || sysop => {
                        self.stage = Stage::Confirm(Action::WithdrawSource(cmd == "z"))
                    }
                    _ => return Err(Error::Invalid("use displayed submission commands")),
                }
            }
            Stage::SourceForm(n) => {
                let d = self.text.as_mut().ok_or(Error::NotFound)?;
                let (field, cap) = match n {
                    0 => (&mut d.title, 64),
                    1 => (&mut d.section, 24),
                    2 => (&mut d.rights, 96),
                    _ => (&mut d.provenance, 128),
                };
                if input.is_empty() && field.is_empty() {
                    self.home();
                } else {
                    if !input.is_empty() {
                        bounded(input, cap)?;
                        *field = input.into();
                    }
                    self.stage = if n < 3 {
                        Stage::SourceForm(n + 1)
                    } else {
                        Stage::Body
                    };
                }
            }
            Stage::Body => {
                let d = self.text.as_mut().ok_or(Error::NotFound)?;
                match input {
                    ".cancel" => self.home(),
                    ".review" => {
                        d.validate()?;
                        self.stage = Stage::SourceReview;
                        self.reset();
                    }
                    ".clear" => d.body.clear(),
                    ".undo" => {
                        d.body.truncate(d.body.rfind('\n').unwrap_or(0));
                    }
                    _ => {
                        if input.starts_with('.') && !input.starts_with("..") {
                            return Err(Error::Invalid(
                                "use .review .undo .clear .cancel; .. for a literal dot",
                            ));
                        }
                        let line = if input.starts_with("..") {
                            &input[1..]
                        } else {
                            input
                        };
                        if line.len() > 120
                            || d.body.len() + line.len() + usize::from(!d.body.is_empty()) > 4096
                            || (!d.body.is_empty() && d.body.split('\n').count() >= 64)
                        {
                            return Err(Error::Invalid(
                                "manuscript 4096 bytes / 64 lines / 120 columns",
                            ));
                        }
                        if !d.body.is_empty() {
                            d.body.push('\n');
                        }
                        d.body.push_str(line);
                    }
                }
            }
            Stage::SourceReview => {
                if cmd == "q" {
                    self.home();
                } else {
                    if !self.finished {
                        return Err(Error::Invalid("read the complete review before confirming"));
                    }
                    match cmd.as_str() {
                        "s" => self.stage = Stage::Confirm(Action::Save(false)),
                        "t" if !self.editing_entry => {
                            self.stage = Stage::Confirm(Action::Save(true))
                        }
                        _ => return Err(Error::Invalid("use displayed review commands")),
                    }
                }
            }
            Stage::ChooseIssue => {
                if input.is_empty() {
                    self.stage = Stage::Issues;
                } else {
                    self.open_issue(b, s, positive(input)?)?;
                }
            }
            Stage::Issue => {
                let v = self.issue(b, s)?;
                match cmd.as_str() {
                    "q" => self.home(),
                    "a" | "u" if v.entries.iter().any(|e| e.mine) => {
                        if cmd == "a" && !self.finished {
                            return Err(Error::Invalid("read the complete issue before approving"));
                        }
                        self.stage = Stage::Confirm(Action::Approve(cmd == "a"));
                    }
                    "h" if sysop && v.state == "draft" => {
                        self.header = Some(v.draft);
                        self.stage = Stage::IssueForm(0);
                        self.reset();
                    }
                    "s" if sysop && v.state == "draft" => self.stage = Stage::Slot,
                    "e" if sysop && v.state == "draft" => self.stage = Stage::EditSlot,
                    "publish" if sysop && v.state == "draft" => {
                        if !self.finished {
                            return Err(Error::Invalid(
                                "read the complete issue before publication",
                            ));
                        }
                        self.stage = Stage::Confirm(Action::Publish);
                    }
                    "x" | "z" if sysop => {
                        self.stage = Stage::Confirm(Action::WithdrawIssue(cmd == "z"))
                    }
                    "o" if sysop && v.state == "published" => self.stage = Stage::Offer,
                    "l" => {
                        b.magazine_file(s, v.id)?.ok_or(Error::NotFound)?;
                        self.stage = Stage::Link;
                    }
                    "f" if files => {
                        return Ok(Outcome::File(
                            b.magazine_file(s, v.id)?.ok_or(Error::NotFound)?,
                        ));
                    }
                    _ => return Err(Error::Invalid("use displayed issue commands")),
                }
            }
            Stage::IssueForm(n) => {
                let h = self.header.as_mut().ok_or(Error::NotFound)?;
                if n < 2 {
                    if !input.is_empty() {
                        let v = input
                            .parse::<u16>()
                            .ok()
                            .filter(|v| (1..=9999).contains(v))
                            .ok_or(Error::Invalid("issue number/revision 1-9999"))?;
                        if n == 0 {
                            h.number = v;
                        } else {
                            h.edition = v;
                        }
                    } else if n == 0 && h.number == 0 {
                        self.home();
                        return Ok(Outcome::Stay);
                    }
                } else {
                    let (field, cap) = match n {
                        2 => (&mut h.title, 64),
                        3 => (&mut h.released, 32),
                        _ => (&mut h.schedule, 64),
                    };
                    if !input.is_empty() {
                        bounded(input, cap)?;
                        *field = input.into();
                    } else if field.is_empty() {
                        self.home();
                        return Ok(Outcome::Stay);
                    }
                }
                if n == 4 {
                    h.validate()?;
                    self.stage = Stage::IssueReview;
                    self.reset();
                } else {
                    self.stage = Stage::IssueForm(n + 1);
                }
            }
            Stage::IssueReview => {
                if cmd == "q" {
                    self.home();
                } else {
                    if cmd != "yes" || !self.finished {
                        return Err(Error::Invalid("read all pages, then YES or Q"));
                    }
                    let expected = self
                        .issue
                        .as_ref()
                        .map(|i| i.1.as_str())
                        .unwrap_or(&self.nonce);
                    let id = b.magazine_save_issue(
                        s,
                        self.issue.as_ref().map(|i| i.0),
                        expected,
                        self.header.as_ref().ok_or(Error::NotFound)?,
                    )?;
                    self.open_issue(b, s, id)?;
                }
            }
            Stage::Slot | Stage::EditSlot => {
                let v = self.issue(b, s)?;
                if input.is_empty() {
                    self.stage = Stage::Issue;
                } else {
                    self.slot = input
                        .parse::<u8>()
                        .ok()
                        .filter(|n| (1..=8).contains(n))
                        .ok_or(Error::Invalid("slot 1-8 required"))?;
                    if matches!(self.stage, Stage::Slot) {
                        self.stage = Stage::SelectSource;
                    } else {
                        let e = v
                            .entries
                            .into_iter()
                            .find(|e| e.slot == self.slot)
                            .ok_or(Error::NotFound)?;
                        self.text = Some(e.text);
                        self.editing_entry = true;
                        self.stage = Stage::SourceForm(0);
                        self.reset();
                    }
                }
            }
            Stage::SelectSource => {
                let v = self.issue(b, s)?;
                let source = input
                    .parse::<i64>()
                    .ok()
                    .filter(|n| *n >= 0)
                    .ok_or(Error::Invalid("submission ID or zero required"))?;
                b.magazine_select(s, v.id, &v.token, self.slot, source)?;
                self.open_issue(b, s, v.id)?;
            }
            Stage::Offer => {
                self.issue(b, s)?;
                if input.is_empty() {
                    self.stage = Stage::Issue;
                } else {
                    self.object = if input == "-" {
                        String::new()
                    } else if input.starts_with("faction-file:") {
                        b.file_resolve(s, input).map_err(unavailable)?.id
                    } else {
                        b.file_reference(s, input).map_err(unavailable)?;
                        input.into()
                    };
                    self.stage = Stage::Confirm(Action::Offer);
                }
            }
            Stage::Link => {
                let v = self.issue(b, s)?;
                match cmd.as_str() {
                    "q" => self.stage = Stage::Issue,
                    "f" if files => {
                        return Ok(Outcome::File(
                            b.magazine_file(s, v.id)?.ok_or(Error::NotFound)?,
                        ));
                    }
                    _ => return Err(Error::Invalid("use Q or displayed Files action")),
                }
            }
            Stage::Confirm(action) => {
                if cmd == "q" {
                    self.home();
                    return Ok(Outcome::Stay);
                }
                if cmd != "yes" {
                    return Err(Error::Invalid("type YES or Q"));
                }
                match action {
                    Action::Save(submit) => {
                        if self.editing_entry {
                            let v = self.issue(b, s)?;
                            b.magazine_edit_entry(
                                s,
                                v.id,
                                &v.token,
                                self.slot,
                                self.text.as_ref().ok_or(Error::NotFound)?,
                            )?;
                            self.editing_entry = false;
                            self.open_issue(b, s, v.id)?;
                        } else {
                            if self.source.is_some() {
                                self.source(b, s)?;
                            }
                            let expected = self
                                .source
                                .as_ref()
                                .map(|v| v.1.as_str())
                                .unwrap_or(&self.nonce);
                            let id = b.magazine_save_submission(
                                s,
                                self.source.as_ref().map(|v| v.0),
                                expected,
                                self.text.as_ref().ok_or(Error::NotFound)?,
                                submit,
                            )?;
                            self.open_source(b, s, id)?;
                        }
                    }
                    Action::WithdrawSource(delete) => {
                        let v = self.source(b, s)?;
                        b.magazine_withdraw_submission(s, v.id, &v.revision, delete)?;
                        self.home();
                    }
                    Action::Approve(approve) => {
                        let v = self.issue(b, s)?;
                        b.magazine_approve(s, v.id, &v.token, approve)?;
                        self.open_issue(b, s, v.id)?;
                    }
                    Action::Publish => {
                        let v = self.issue(b, s)?;
                        b.magazine_publish(s, v.id, &v.token)?;
                        self.open_issue(b, s, v.id)?;
                    }
                    Action::WithdrawIssue(delete) => {
                        let v = self.issue(b, s)?;
                        b.magazine_withdraw_issue(s, v.id, &v.token, delete)?;
                        self.home();
                    }
                    Action::Offer => {
                        let v = self.issue(b, s)?;
                        b.magazine_offer(s, v.id, &v.token, &self.object)?;
                        self.open_issue(b, s, v.id)?;
                    }
                }
                self.notice = "Workshop updated.".into();
            }
        }
        Ok(Outcome::Stay)
    }
    pub(super) fn recover(&mut self, e: &Error) -> bool {
        if matches!(e, Error::Invalid(_) | Error::NotFound | Error::Busy) {
            if matches!(
                e,
                Error::NotFound
                    | Error::Invalid("issue changed; reopen it")
                    | Error::Invalid("submission changed; reopen it")
            ) {
                self.home();
            }
            self.notice = format!("Workshop: {e}");
            true
        } else {
            false
        }
    }
}
fn bounded(s: &str, n: usize) -> Result<()> {
    if s.trim().is_empty() || s.len() > n || s.contains("faction-file:") {
        Err(Error::Invalid(
            "bounded nonempty field required; file links use related-file action",
        ))
    } else {
        Ok(())
    }
}
fn positive(s: &str) -> Result<i64> {
    s.parse::<i64>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or(Error::Invalid("positive ID required"))
}
fn unavailable(e: Error) -> Error {
    match e {
        Error::Denied | Error::NotFound => Error::NotFound,
        e => e,
    }
}
fn text_preview(t: &MagazineText) -> String {
    format!(
        "{}\nSection: {}\nRights: {}\nProvenance: {}\n\n{}\n",
        t.title, t.section, t.rights, t.provenance, t.body
    )
}
