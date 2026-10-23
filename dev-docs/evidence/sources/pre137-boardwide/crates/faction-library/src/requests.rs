//! Shared line-oriented request desk, including classic/no-escape rendering.
use super::*;
use faction_core::{FileRequest, FileRequestDraft, FileRequestOffer, Role, request_id};
#[derive(Clone, Copy, Default)]
enum Stage {
    #[default]
    List,
    Choose,
    Detail,
    Title,
    Description,
    Create,
    Object,
    Rights,
    Provenance,
    Fulfill,
    Close,
    Delete,
    Link,
}
#[derive(Default)]
pub(super) struct Desk {
    stage: Stage,
    selected: Option<(i64, String)>,
    title: String,
    description: String,
    object: String,
    rights: String,
    provenance: String,
    nonce: String,
    through: Option<i64>,
    previous: Vec<i64>,
    last: Option<i64>,
    notice: String,
    dimensions: Option<u16>,
    offset: usize,
    next: Option<usize>,
    back: Vec<usize>,
}
fn unavailable(e: Error) -> Error {
    match e {
        Error::Denied | Error::NotFound => Error::NotFound,
        e => e,
    }
}
impl Desk {
    fn source(&self, b: &mut Board, s: &Session) -> Result<FileRequest> {
        let (id, rev) = self.selected.as_ref().ok_or(Error::NotFound)?;
        let row = b.file_request(s, *id)?;
        if row.revision != *rev {
            return Err(Error::Invalid("request changed; reopen it"));
        }
        Ok(row)
    }
    fn draft(&self) -> FileRequestDraft {
        FileRequestDraft {
            title: self.title.clone(),
            description: self.description.clone(),
        }
    }
    fn offer(&self) -> FileRequestOffer {
        FileRequestOffer {
            object: self.object.clone(),
            rights: self.rights.clone(),
            provenance: self.provenance.clone(),
        }
    }
    fn list(&mut self) {
        self.stage = Stage::List;
        self.offset = 0;
        self.next = None;
        self.back.clear();
        self.selected = None;
        self.title.clear();
        self.description.clear();
        self.object.clear();
        self.rights.clear();
        self.provenance.clear();
        self.nonce.clear();
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
        if self.dimensions != Some(columns) {
            self.offset = 0;
            self.next = None;
            self.back.clear();
        }
        self.dimensions = Some(columns);
        let width = usize::from(columns).clamp(20, 80) - 1;
        let mut out = faction_terminal::section(t, &c.requests_name, ansi);
        if !self.notice.is_empty() {
            writeln!(out, "{}", preview(&self.notice, width)).unwrap();
        }
        match self.stage {
            Stage::List => {
                let rows = b.file_requests(s, self.through.unwrap_or(i64::MAX))?;
                self.last = rows.last().map(|r| r.id);
                if rows.is_empty() {
                    out.push_str("No requests on this page.\n");
                }
                for r in rows {
                    writeln!(
                        out,
                        "{}",
                        preview(&format!("#{} [{}] {}", r.id, r.state, r.draft.title), width)
                    )
                    .unwrap();
                    if !c.compact {
                        writeln!(out, " by {}", r.creator).unwrap();
                    }
                }
                out.push_str("C:create V:view N/P:page R:refresh Q:library\nRequests> ");
            }
            Stage::Choose => out.push_str("Request ID (blank cancels): "),
            Stage::Title => out.push_str("Request title (1-64 ASCII; blank cancels): "),
            Stage::Description => out.push_str("What are you looking for? (1-128 ASCII): "),
            Stage::Create => {
                self.draft().validate()?;
                self.next = lines(
                    &mut out,
                    &format!("{}\n{}", self.title, self.description),
                    self.offset,
                    width,
                );
                out.push_str("N/P:review pages\n");
                out.push_str("Local board request, readable by sysops.\nRequest only permitted works. No automatic retrieval.\nPost request? YES or Q to cancel: ");
            }
            stage => {
                let r = self.source(b, s)?;
                writeln!(
                    out,
                    "#{} [{}] {}\nBy {}",
                    r.id,
                    r.state,
                    preview(&r.draft.title, width),
                    r.creator
                )
                .unwrap();
                let can_manage = r.mine || b.whoami(s)?.role == Role::Sysop;
                match stage {
                    Stage::Detail => {
                        let mut text = format!("{}\n{}\n", r.draft.title, r.draft.description);
                        if let Some(o) = &r.offer {
                            text.push_str(&format!(
                                "Rights: {}\nSource: {}\n",
                                o.rights, o.provenance
                            ));
                        } else if r.state != "open" {
                            text.push_str("No currently eligible fulfillment.\n");
                        }
                        self.next = lines(&mut out, &text, self.offset, width);
                        out.push_str("N/P:details ");
                        if r.offer.is_some() {
                            out.push_str("L:link ");
                            if files {
                                out.push_str("F:Files ");
                            }
                        }
                        if r.state == "open" {
                            out.push_str("U:fulfill ");
                        }
                        if can_manage {
                            out.push_str("X:close Z:delete ");
                        }
                        out.push_str("Q:list\nRequest> ");
                    }
                    Stage::Link => {
                        let o = r.offer.ok_or(Error::NotFound)?;
                        writeln!(out, "{}", b.file_reference(s, &o.object)?).unwrap();
                        out.push_str("Use existing Files/native transfers. Q:back\nRequest link> ");
                    }
                    Stage::Object => {
                        out.push_str("Published local file ID or reference (blank cancels): ")
                    }
                    Stage::Rights => out.push_str("Declared distribution rights (1-96 ASCII): "),
                    Stage::Provenance => out.push_str("Source/provenance (1-128 ASCII): "),
                    Stage::Fulfill => {
                        let o = self.offer();
                        o.validate()?;
                        let reference = b.file_reference(s, &o.object)?;
                        writeln!(out, "{reference}").unwrap();
                        self.next = lines(
                            &mut out,
                            &format!("Rights: {}\nSource: {}", o.rights, o.provenance),
                            self.offset,
                            width,
                        );
                        out.push_str("N/P:review pages\n");
                        out.push_str("Existing published file only; grants remain unchanged.\nFulfill request? YES or Q to cancel: ");
                    }
                    Stage::Close | Stage::Delete => {
                        if !can_manage {
                            return Err(Error::Denied);
                        }
                        out.push_str(if matches!(stage, Stage::Delete) {
                            "Delete request permanently from live desk? YES or Q: "
                        } else {
                            "Close request? YES or Q: "
                        });
                    }
                    _ => unreachable!(),
                }
            }
        }
        if out.len() > 8192 {
            return Err(Error::Invalid("request frame budget"));
        }
        Ok(out)
    }
    pub(super) fn feed(
        &mut self,
        input: &str,
        b: &mut Board,
        s: &Session,
        files: bool,
    ) -> Result<Outcome> {
        if input.len() > 128 || !input.bytes().all(|b| (32..=126).contains(&b)) {
            return Err(Error::Invalid("bounded ASCII request input required"));
        }
        self.notice.clear();
        let cmd = input.to_ascii_lowercase();
        if cmd == "q"
            && !matches!(
                self.stage,
                Stage::Title
                    | Stage::Description
                    | Stage::Rights
                    | Stage::Provenance
                    | Stage::Object
            )
        {
            if matches!(self.stage, Stage::List) {
                return Ok(Outcome::Main);
            }
            if matches!(self.stage, Stage::Link) {
                self.stage = Stage::Detail;
            } else {
                self.list();
            }
            return Ok(Outcome::Stay);
        }
        if matches!(self.stage, Stage::Create | Stage::Fulfill) {
            if matches!(self.stage, Stage::Fulfill) {
                self.source(b, s)?;
            }
            match cmd.as_str() {
                "n" => {
                    if let Some(n) = self.next
                        && self.back.len() < 32
                    {
                        self.back.push(self.offset);
                        self.offset = n;
                    }
                    return Ok(Outcome::Stay);
                }
                "p" => {
                    self.offset = self.back.pop().unwrap_or(0);
                    return Ok(Outcome::Stay);
                }
                "yes" if self.next.is_some() => {
                    return Err(Error::Invalid("read remaining review pages before YES"));
                }
                _ => (),
            }
        }
        match self.stage {
            Stage::List => match cmd.as_str() {
                "c" => {
                    self.list();
                    self.nonce = request_id()?;
                    self.stage = Stage::Title;
                }
                "v" => self.stage = Stage::Choose,
                "r" => {
                    self.through = None;
                    self.previous.clear();
                }
                "n" => {
                    if let Some(last) = self.last
                        && self.previous.len() < 64
                    {
                        self.previous.push(self.through.unwrap_or(i64::MAX));
                        self.through = Some(last.saturating_sub(1));
                    }
                }
                "p" => self.through = self.previous.pop(),
                _ => return Err(Error::Invalid("use displayed request commands")),
            },
            Stage::Choose => {
                if input.is_empty() {
                    self.list();
                } else {
                    let id = input
                        .parse::<i64>()
                        .map_err(|_| Error::Invalid("positive request ID required"))?;
                    let r = b.file_request(s, id)?;
                    self.offset = 0;
                    self.next = None;
                    self.back.clear();
                    self.selected = Some((id, r.revision));
                    self.stage = Stage::Detail;
                }
            }
            Stage::Title => {
                if input.is_empty() {
                    self.list();
                } else {
                    self.title = input.into();
                    FileRequestDraft {
                        title: self.title.clone(),
                        description: "check".into(),
                    }
                    .validate()?;
                    self.stage = Stage::Description;
                }
            }
            Stage::Description => {
                if input.is_empty() {
                    self.list();
                } else {
                    self.description = input.into();
                    self.draft().validate()?;
                    self.offset = 0;
                    self.next = None;
                    self.back.clear();
                    self.stage = Stage::Create;
                }
            }
            Stage::Create => {
                if cmd != "yes" {
                    return Err(Error::Invalid("type YES or Q"));
                }
                let id = b.create_file_request(s, &self.nonce, &self.draft())?;
                let r = b.file_request(s, id)?;
                self.selected = Some((id, r.revision));
                self.stage = Stage::Detail;
                self.notice = "Request posted.".into();
            }
            Stage::Detail => {
                let r = self.source(b, s)?;
                match cmd.as_str() {
                    "u" if r.state == "open" => {
                        self.object.clear();
                        self.rights.clear();
                        self.provenance.clear();
                        self.stage = Stage::Object;
                    }
                    "n" => {
                        if let Some(n) = self.next
                            && self.back.len() < 32
                        {
                            self.back.push(self.offset);
                            self.offset = n;
                        }
                    }
                    "p" => self.offset = self.back.pop().unwrap_or(0),
                    "l" if r.offer.is_some() => self.stage = Stage::Link,
                    "f" if files => {
                        let o = r.offer.ok_or(Error::NotFound)?;
                        b.file_reference(s, &o.object)?;
                        return Ok(Outcome::File(o.object));
                    }
                    "x" | "z" if r.mine || b.whoami(s)?.role == Role::Sysop => {
                        self.stage = if cmd == "x" {
                            Stage::Close
                        } else {
                            Stage::Delete
                        }
                    }
                    _ => return Err(Error::Invalid("use displayed request commands")),
                }
            }
            Stage::Object => {
                self.source(b, s)?;
                if input.is_empty() {
                    self.list();
                } else {
                    self.object = if input.starts_with("faction-file:") {
                        b.file_resolve(s, input).map_err(unavailable)?.id
                    } else {
                        b.file_reference(s, input).map_err(unavailable)?;
                        input.into()
                    };
                    self.stage = Stage::Rights;
                }
            }
            Stage::Rights => {
                self.source(b, s)?;
                if input.is_empty() {
                    self.list();
                } else {
                    self.rights = input.into();
                    FileRequestOffer {
                        object: self.object.clone(),
                        rights: self.rights.clone(),
                        provenance: "check".into(),
                    }
                    .validate()?;
                    self.stage = Stage::Provenance;
                }
            }
            Stage::Provenance => {
                self.source(b, s)?;
                if input.is_empty() {
                    self.list();
                } else {
                    self.provenance = input.into();
                    self.offer().validate()?;
                    self.offset = 0;
                    self.next = None;
                    self.back.clear();
                    self.stage = Stage::Fulfill;
                }
            }
            Stage::Fulfill | Stage::Close | Stage::Delete => {
                let r = self.source(b, s)?;
                if cmd != "yes" {
                    return Err(Error::Invalid("type YES or Q"));
                }
                if matches!(self.stage, Stage::Fulfill) {
                    b.fulfill_file_request(s, r.id, &r.revision, &self.offer())?;
                } else {
                    b.finish_file_request(
                        s,
                        r.id,
                        &r.revision,
                        matches!(self.stage, Stage::Delete),
                    )?;
                }
                self.list();
                self.notice = "Request updated.".into();
            }
            Stage::Link => return Err(Error::Invalid("use Q to return")),
        }
        Ok(Outcome::Stay)
    }
    pub(super) fn recover(&mut self, e: &Error) -> bool {
        if matches!(e, Error::Invalid(_) | Error::NotFound | Error::Busy) {
            if matches!(
                e,
                Error::NotFound | Error::Invalid("request changed; reopen it")
            ) {
                self.list();
            }
            self.notice = format!("Request: {e}");
            true
        } else {
            false
        }
    }
}
