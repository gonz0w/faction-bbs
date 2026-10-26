//! Local member-page interaction shared by local/worker and shared terminals.
#![forbid(unsafe_code)]
pub mod directory;
pub mod notifications;
pub mod social;
pub mod who;
use faction_core::{
    Board, Error, MemberPage, PAGE_FIELD_BYTES, PAGE_FIELDS, Result, Role, Session, page_text,
    request_id,
};
use faction_terminal::Theme;
use std::fmt::Write;
#[derive(Clone)]
enum Draft {
    Field {
        page: MemberPage,
        index: usize,
        body: String,
    },
    Plan {
        body: String,
        visible: bool,
        key: Option<String>,
    },
    Page(MemberPage),
}
enum State {
    List,
    Find,
    Card(String, usize, usize),
    Edit(MemberPage),
    Visibility(MemberPage),
    Text(Draft),
    Review(Draft),
    Delete(MemberPage),
    Settings,
    Setting(char),
    Moderate,
}
pub struct Desk {
    state: State,
    offset: usize,
    shown: Option<i64>,
    next: Option<usize>,
    notice: String,
    preview: usize,
}
impl Default for Desk {
    fn default() -> Self {
        Self {
            state: State::List,
            offset: 0,
            shown: None,
            next: None,
            notice: String::new(),
            preview: 0,
        }
    }
}
fn approved(b: &mut Board, s: &Session, sysop: bool) -> Result<String> {
    let m = b.whoami(s)?;
    if m.status != faction_core::Status::Approved || (sysop && m.role != Role::Sysop) {
        return Err(Error::Denied);
    }
    Ok(m.handle)
}
fn body(out: &mut String, text: &str, offset: usize, columns: usize) -> Option<usize> {
    let width = columns.clamp(20, 80) - 1;
    let mut at = offset.min(text.len());
    let mut rows = 0;
    while at < text.len() && rows < 8 {
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
fn selected(input: &str) -> Result<u8> {
    match input.trim().to_ascii_lowercase().as_str() {
        "all" => Ok(15),
        "none" | "" => Ok(0),
        _ => {
            let mut mask = 0;
            for token in input.split_whitespace() {
                let n: u8 = token
                    .parse()
                    .map_err(|_| Error::Invalid("choose fields 1-4, all or none"))?;
                if !(1..=4).contains(&n) {
                    return Err(Error::Invalid("choose fields 1-4, all or none"));
                }
                mask |= 1 << (n - 1);
            }
            Ok(mask)
        }
    }
}
fn choices(mask: u8) -> String {
    let values: Vec<String> = (0..4)
        .filter(|i| mask & (1 << i) != 0)
        .map(|i| (i + 1).to_string())
        .collect();
    if values.is_empty() {
        "none".into()
    } else {
        values.join(" ")
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
        let me = approved(
            b,
            s,
            matches!(
                self.state,
                State::Settings | State::Setting(_) | State::Moderate
            ),
        )?;
        let c = b.page_config(s)?;
        if !c.enabled && !matches!(self.state, State::Settings | State::Setting(_)) {
            return Err(Error::Denied);
        }
        let mut out = faction_terminal::section(theme, &c.name, ansi);
        if !self.notice.is_empty() {
            writeln!(out, "{}", self.notice).unwrap();
            self.notice.clear();
        }
        match &mut self.state {
            State::List => {
                out.push_str("Local pages shared with approved members\n");
                for name in b.page_handles(s, self.offset)? {
                    writeln!(out, "  {name}").unwrap();
                }
                out.push_str("N:next B:back F:find E:edit my page\nV:view mine S:moderate (sysop) Q:main\nFirst 100; F finds an exact handle.\nMember pages> ");
            }
            State::Find => out.push_str("Member handle: "),
            State::Card(name, slot, offset) => {
                let p = b.member_page(s, name)?;
                let plans = b.member_plans(s, name)?;
                *slot = (*slot).min(3 + plans.len());
                if c.compact {
                    writeln!(out, "{} | local page", p.handle).unwrap();
                } else {
                    writeln!(out, "+-- {} --+", p.handle).unwrap();
                    writeln!(
                        out,
                        "Page updated: {}",
                        if p.updated.is_empty() {
                            "unsaved"
                        } else {
                            &p.updated
                        }
                    )
                    .unwrap();
                }
                self.shown = None;
                if *slot < 4 {
                    writeln!(
                        out,
                        "{} ({})",
                        PAGE_FIELDS[*slot],
                        if c.fields & (1 << *slot) == 0 {
                            "disabled by sysop"
                        } else if p.visible & (1 << *slot) != 0 {
                            "shared locally"
                        } else {
                            "hidden"
                        }
                    )
                    .unwrap();
                    self.next = body(
                        &mut out,
                        if p.fields[*slot].is_empty() {
                            "(not shared / empty)"
                        } else {
                            &p.fields[*slot]
                        },
                        *offset,
                        columns,
                    );
                } else {
                    let plan = &plans[*slot - 4];
                    self.shown = Some(plan.id);
                    writeln!(
                        out,
                        ".plan #{} {}",
                        plan.id,
                        if plan.visible { "shared" } else { "hidden" }
                    )
                    .unwrap();
                    writeln!(out, "{}", plan.posted).unwrap();
                    self.next = body(&mut out, &plan.body, *offset, columns);
                }
                out.push_str("N:next field/update B:previous\nEnter:more E:edit mine X:delete plan\nQ:list | Member page> ");
            }
            State::Edit(p) => {
                writeln!(out, "Editing {} (local plaintext)", p.handle).unwrap();
                out.push_str("Local plaintext; sysop can read it.\n");
                writeln!(
                    out,
                    "Published: {} / sysop hidden: {}",
                    p.published, p.moderated
                )
                .unwrap();
                for (i, label) in PAGE_FIELDS.iter().enumerate() {
                    writeln!(
                        out,
                        "{}:{} [{}]",
                        i + 1,
                        label,
                        if c.fields & (1 << i) == 0 {
                            "disabled"
                        } else if p.visible & (1 << i) == 0 {
                            "hidden"
                        } else {
                            "shared"
                        }
                    )
                    .unwrap();
                }
                out.push_str("1-4:replace field V:visibility H:hide/show\nP:new .plan D:delete page Q:list\nMy page> ");
            }
            State::Visibility(_) => out.push_str("Shared fields (1-4, all/none): "),
            State::Text(d) => {
                let (text, max, lines) = match d {
                    Draft::Field { body, index, .. } => (
                        body.as_str(),
                        PAGE_FIELD_BYTES[*index],
                        if *index == 0 || *index == 2 { 4 } else { 2 },
                    ),
                    Draft::Plan { body, .. } => (body.as_str(), 480, 8),
                    _ => return Err(Error::Denied),
                };
                writeln!(out, "Up to {max} ASCII bytes / {lines} lines").unwrap();
                out.push_str(
                    ". reviews; .. literal .; /cancel leaves\n/undo removes last completed line\n",
                );
                self.next = body(&mut out, text, 0, columns);
                out.push_str("Page text> ");
            }
            State::Review(d) => {
                out.push_str("PREVIEW ONLY - not saved\n");
                match d {
                    Draft::Field {
                        index, body: text, ..
                    } => {
                        writeln!(out, "{}", PAGE_FIELDS[*index]).unwrap();
                        self.next = body(&mut out, text, self.preview, columns);
                    }
                    Draft::Plan {
                        body: text,
                        visible,
                        ..
                    } => {
                        writeln!(
                            out,
                            "Dated .plan: {}",
                            if *visible { "shared locally" } else { "hidden" }
                        )
                        .unwrap();
                        self.next = body(&mut out, text, self.preview, columns);
                    }
                    Draft::Page(p) => {
                        writeln!(
                            out,
                            "Publish page: {}\nShared fields: {}",
                            p.published,
                            choices(p.visible)
                        )
                        .unwrap();
                    }
                }
                out.push_str("Enter:more preview\n[S]ave [E]dit [H]ide/share plan [C]ancel: ");
            }
            State::Delete(_) => {
                out.push_str("DELETE removes page/plans; sysop hide remains. Type DELETE: ")
            }
            State::Settings => {
                writeln!(out, "Enabled {} / compact {}", c.enabled, c.compact).unwrap();
                writeln!(
                    out,
                    "Fields {} / history {} / days {}",
                    choices(c.fields),
                    c.history,
                    c.days
                )
                .unwrap();
                writeln!(out, "Sysop removal allowed: {}", c.moderator_remove).unwrap();
                out.push_str("E:enable D:disable N:name L:layout\nF:fields H:history/days R:remove policy\nQ:main | Page settings> ");
            }
            State::Setting(k) => out.push_str(match k {
                'n' => "Page label (1-24 ASCII): ",
                'f' => "Allowed fields (1-4, all/none): ",
                'h' => "Plan history (1-16) and days (7-365): ",
                _ => "Allow sysop permanent removal? YES or NO: ",
            }),
            State::Moderate => {
                out.push_str("Moderate: handle hide|show|remove (remove deletes page/plans): ")
            }
        }
        let _ = me;
        if out.len() > 8192 {
            return Err(Error::Invalid("member-page frame budget"));
        }
        Ok(out)
    }
    fn edit(&mut self, b: &mut Board, s: &Session) -> Result<()> {
        let me = approved(b, s, false)?;
        self.state = State::Edit(b.member_page(s, &me)?);
        Ok(())
    }
    pub fn feed(&mut self, input: &str, b: &mut Board, s: &Session) -> Result<bool> {
        let me = approved(
            b,
            s,
            matches!(
                self.state,
                State::Settings | State::Setting(_) | State::Moderate
            ),
        )?;
        let mut c = b.page_config(s)?;
        if !c.enabled && !matches!(self.state, State::Settings | State::Setting(_)) {
            return Err(Error::Denied);
        }
        match &mut self.state {
            State::List => match input.to_ascii_lowercase().as_str() {
                "q" => return Ok(true),
                "n" => self.offset = (self.offset + 8).min(96),
                "b" => self.offset = self.offset.saturating_sub(8),
                "f" => self.state = State::Find,
                "e" => self.edit(b, s)?,
                "v" => self.state = State::Card(me, 0, 0),
                "s" => {
                    if b.whoami(s)?.role != Role::Sysop {
                        return Err(Error::Invalid("moderation is sysop-only"));
                    }
                    self.state = State::Moderate
                }
                _ => (),
            },
            State::Find => {
                if input.is_empty() {
                    self.state = State::List
                } else {
                    b.member_page(s, input)?;
                    self.state = State::Card(input.into(), 0, 0)
                }
            }
            State::Card(name, slot, offset) => match input.to_ascii_lowercase().as_str() {
                "q" => self.state = State::List,
                "n" => {
                    *slot = slot.saturating_add(1);
                    *offset = 0
                }
                "b" => {
                    *slot = slot.saturating_sub(1);
                    *offset = 0
                }
                "" => {
                    if let Some(n) = self.next {
                        *offset = n
                    } else {
                        *slot = slot.saturating_add(1);
                        *offset = 0
                    }
                }
                "e" => self.edit(b, s)?,
                "x" => {
                    if !name.eq_ignore_ascii_case(&me) {
                        return Err(Error::Invalid("only your own plan updates can be deleted"));
                    }
                    b.remove_member_plan(s, self.shown.ok_or(Error::NotFound)?)?;
                    self.notice = "Plan update removed.".into();
                }
                _ => (),
            },
            State::Edit(p) => match input.to_ascii_lowercase().as_str() {
                "q" => self.state = State::List,
                "v" => self.state = State::Visibility(p.clone()),
                "h" => {
                    let mut p = p.clone();
                    p.published = !p.published;
                    self.state = State::Review(Draft::Page(p));
                }
                "d" => self.state = State::Delete(p.clone()),
                "p" => {
                    if p.revision.is_none() {
                        return Err(Error::Invalid("save a field/page before adding a .plan"));
                    }
                    self.state = State::Text(Draft::Plan {
                        body: String::new(),
                        visible: true,
                        key: None,
                    })
                }
                "1" | "2" | "3" | "4" => {
                    let i = input.as_bytes()[0] as usize - b'1' as usize;
                    if c.fields & (1 << i) == 0 {
                        return Err(Error::Invalid("field disabled by sysop"));
                    }
                    self.state = State::Text(Draft::Field {
                        page: p.clone(),
                        index: i,
                        body: String::new(),
                    });
                }
                _ => (),
            },
            State::Visibility(p) => {
                let mut p = p.clone();
                p.visible = selected(input)?;
                self.state = State::Review(Draft::Page(p));
            }
            State::Text(d) => {
                if input == "/cancel" {
                    self.edit(b, s)?;
                } else if input == "." {
                    self.preview = 0;
                    self.state = State::Review(d.clone())
                } else {
                    let (text, max, lines) = match d {
                        Draft::Field { index, body, .. } => (
                            body,
                            PAGE_FIELD_BYTES[*index],
                            if *index == 0 || *index == 2 { 4 } else { 2 },
                        ),
                        Draft::Plan { body, .. } => (body, 480, 8),
                        _ => return Err(Error::Denied),
                    };
                    if input == "/undo" {
                        if let Some(at) = text.rfind('\n') {
                            text.truncate(at)
                        } else {
                            text.clear()
                        }
                    } else {
                        let mut next = text.clone();
                        if !next.is_empty() {
                            next.push('\n')
                        }
                        next.push_str(if input == ".." { "." } else { input });
                        page_text(&next, max, lines)?;
                        *text = next;
                    }
                }
            }
            State::Review(d) => match input.to_ascii_lowercase().as_str() {
                "" => self.preview = self.next.unwrap_or(0),
                "c" => self.edit(b, s)?,
                "h" => {
                    if let Draft::Plan { visible, .. } = d {
                        *visible = !*visible
                    }
                }
                "e" => match d {
                    Draft::Page(p) => self.state = State::Visibility(p.clone()),
                    _ => self.state = State::Text(d.clone()),
                },
                "s" => {
                    match d {
                        Draft::Field { page, index, body } => {
                            page.fields[*index] = body.clone();
                            b.save_member_page(s, page)?;
                        }
                        Draft::Plan { body, visible, key } => {
                            if key.is_none() {
                                *key = Some(request_id()?)
                            }
                            b.add_member_plan(s, key.as_deref().unwrap(), body, *visible)?;
                        }
                        Draft::Page(p) => {
                            b.save_member_page(s, p)?;
                        }
                    }
                    self.notice = "Member page saved locally.".into();
                    self.edit(b, s)?;
                }
                _ => (),
            },
            State::Delete(p) => {
                if input == "DELETE" {
                    b.delete_member_page(s, p.revision.as_deref())?;
                    self.notice = "Page and retained plans removed.".into();
                    self.state = State::List;
                } else {
                    self.edit(b, s)?;
                }
            }
            State::Settings => match input.to_ascii_lowercase().as_str() {
                "q" => return Ok(true),
                "e" | "d" => {
                    c.enabled = input.eq_ignore_ascii_case("e");
                    b.configure_pages(s, &c)?;
                }
                "l" => {
                    c.compact = !c.compact;
                    b.configure_pages(s, &c)?;
                }
                "n" | "f" | "h" | "r" => {
                    self.state = State::Setting(input.as_bytes()[0].to_ascii_lowercase() as char)
                }
                _ => (),
            },
            State::Setting(k) => {
                if !input.is_empty() {
                    match k {
                        'n' => c.name = input.into(),
                        'f' => c.fields = selected(input)?,
                        'h' => {
                            let n = input
                                .split_whitespace()
                                .map(str::parse::<u16>)
                                .collect::<std::result::Result<Vec<_>, _>>()
                                .map_err(|_| Error::Invalid("history needs two numbers"))?;
                            if n.len() != 2 || n[0] > 16 {
                                return Err(Error::Invalid("history needs count/days"));
                            }
                            c.history = n[0] as u8;
                            c.days = n[1];
                        }
                        _ => {
                            c.moderator_remove = match input {
                                "YES" => true,
                                "NO" => false,
                                _ => return Err(Error::Invalid("type YES or NO")),
                            }
                        }
                    }
                    b.configure_pages(s, &c)?;
                }
                self.state = State::Settings;
            }
            State::Moderate => {
                let parts: Vec<_> = input.split_whitespace().collect();
                if parts.len() != 2 {
                    return Err(Error::Invalid("use handle hide|show|remove"));
                }
                if parts[1] == "remove" && !c.moderator_remove {
                    return Err(Error::Invalid("permanent removal disabled; use hide"));
                }
                b.moderate_member_page(s, parts[0], parts[1])?;
                self.notice = "Moderation applied; author content not rewritten.".into();
                self.state = State::List;
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
                "Page changed; cancel and reopen before saving.".into()
            } else {
                format!("Member pages: {e}")
            };
            if matches!(e, Error::NotFound) && matches!(self.state, State::Card(..)) {
                self.state = State::List;
            }
            true
        } else {
            false
        }
    }
}
/// Temporary menu presentation copy, not cached approval or a member registry.
pub fn menu_theme(b: &mut Board, s: &Session, theme: &Theme) -> Result<Option<Theme>> {
    if b.page_config(s)?.enabled {
        let mut t = theme.clone();
        t.set_pages_enabled(true).map_err(Error::Invalid)?;
        Ok(Some(t))
    } else {
        Ok(None)
    }
}
