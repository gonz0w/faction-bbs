//! Shared approved-member shelf workflow. No file bytes, transfer grants or network I/O.
#![forbid(unsafe_code)]
mod catalog;
pub use catalog::{BOOKS, BYTES, Catalog, SHELVES};
use faction_core::{AreaFile, Board, Error, Result, Session, Status};
use faction_terminal::{Theme, subject::preview};
use std::{collections::VecDeque, fmt::Write};
#[derive(Clone, Copy)]
enum State {
    List,
    Shelves,
    Choose,
    Info(usize),
    Link(usize),
}
pub enum Outcome {
    Stay,
    Main,
    File(String),
    Room(i64),
}
pub struct Desk {
    files: bool,
    state: State,
    shelf: Option<usize>,
    page: usize,
    offset: usize,
    next: Option<usize>,
    previous: VecDeque<usize>,
    dimensions: Option<u16>,
    notice: String,
}
impl Default for Desk {
    fn default() -> Self {
        Self {
            files: false,
            state: State::List,
            shelf: None,
            page: 0,
            offset: 0,
            next: None,
            previous: VecDeque::new(),
            dimensions: None,
            notice: String::new(),
        }
    }
}
fn approved(b: &mut Board, s: &Session) -> Result<()> {
    if b.whoami(s)?.status != Status::Approved {
        return Err(Error::Denied);
    }
    Ok(())
}
fn resolve(b: &mut Board, s: &Session, c: &Catalog, i: usize) -> Result<AreaFile> {
    b.file_resolve(s, &c.books[i].reference)
        .map_err(|e| match e {
            Error::Denied | Error::NotFound => Error::NotFound,
            e => e,
        })
}
fn inert(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c == '\n' || c.is_ascii_graphic() || c == ' ' {
                vec![c]
            } else {
                c.escape_unicode().collect()
            }
        })
        .collect()
}
fn lines(out: &mut String, text: &str, offset: usize, width: usize) -> Option<usize> {
    let mut at = offset.min(text.len());
    for _ in 0..8 {
        if at == text.len() {
            break;
        }
        let rest = &text[at..];
        let n = rest.find('\n').unwrap_or(rest.len()).min(width);
        out.push_str(&rest[..n]);
        out.push('\n');
        at += n;
        if text.as_bytes().get(at) == Some(&b'\n') {
            at += 1
        }
    }
    (at < text.len()).then_some(at)
}
impl Desk {
    pub fn with_files(files: bool) -> Self {
        Self {
            files,
            ..Self::default()
        }
    }
    fn reset(&mut self) {
        self.offset = 0;
        self.next = None;
        self.previous.clear()
    }
    fn visible(b: &mut Board, s: &Session, c: &Catalog) -> Result<Vec<(usize, AreaFile)>> {
        let mut rows = vec![];
        for i in 0..c.books.len() {
            match resolve(b, s, c, i) {
                Ok(file) => rows.push((i, file)),
                Err(Error::NotFound) => (),
                Err(e) => return Err(e),
            }
        }
        Ok(rows)
    }
    pub fn render(
        &mut self,
        b: &mut Board,
        s: &Session,
        c: &Catalog,
        t: &Theme,
        ansi: bool,
        columns: u16,
    ) -> Result<String> {
        approved(b, s)?;
        if !c.enabled() {
            return Err(Error::Denied);
        }
        if self.dimensions.is_some_and(|old| old != columns) {
            self.reset()
        }
        self.dimensions = Some(columns);
        let width = usize::from(columns).clamp(20, 80) - 1;
        let mut out = faction_terminal::section(t, &c.name, ansi);
        if !self.notice.is_empty() {
            writeln!(out, "{}", preview(&self.notice, width)).unwrap()
        }
        match self.state {
            State::List => {
                let visible = Self::visible(b, s, c)?;
                let rows = visible
                    .iter()
                    .filter(|(i, _)| {
                        self.shelf
                            .is_none_or(|s| c.books[*i].shelf == c.shelves[s].id)
                    })
                    .collect::<Vec<_>>();
                self.page = self.page.min(rows.len().saturating_sub(1) / 4);
                if rows.is_empty() {
                    out.push_str("No eligible library entries.\n")
                } else {
                    writeln!(
                        out,
                        "{} / page {}",
                        self.shelf
                            .map(|i| preview(&c.shelves[i].title, width.saturating_sub(12)))
                            .unwrap_or_else(|| "All shelves".into()),
                        self.page + 1
                    )
                    .unwrap();
                    for (i, file) in rows.into_iter().skip(self.page * 4).take(4) {
                        writeln!(
                            out,
                            "{}",
                            preview(&format!("{}: {}", c.books[*i].id, file.filename), width)
                        )
                        .unwrap();
                        if !c.compact {
                            writeln!(
                                out,
                                " {}",
                                preview(&c.books[*i].note.replace('\n', " "), width - 1)
                            )
                            .unwrap()
                        }
                    }
                }
                out.push_str("E:shelves V:entry N/P:page Q:main\nLibrary> ");
            }
            State::Shelves => {
                out.push_str("Eligible shelves (- means all):\n");
                let visible = Self::visible(b, s, c)?;
                for shelf in &c.shelves {
                    if visible.iter().any(|(i, _)| c.books[*i].shelf == shelf.id) {
                        writeln!(
                            out,
                            "{}",
                            preview(&format!("{}: {}", shelf.id, shelf.title), width)
                        )
                        .unwrap()
                    }
                }
                out.push_str("Shelf ID (blank cancels): ");
            }
            State::Choose => out.push_str("Entry ID (blank cancels): "),
            State::Info(i) => {
                let file = resolve(b, s, c, i)?;
                let book = &c.books[i];
                let text = inert(&format!(
                    "{} / {} bytes\n{}\nCurator: {}\nRights declared: {}\nProvenance: {}\nSHA256: {}\n",
                    file.filename,
                    file.bytes,
                    file.description,
                    book.note,
                    book.rights,
                    book.provenance,
                    file.sha256
                ));
                self.next = lines(&mut out, &text, self.offset, width);
                out.push_str("N/P:notes L:file link ");
                if self.files {
                    out.push_str("F:Files ");
                }
                if let Some(room) = book.room {
                    match b.unread_cursor(s, room) {
                        Ok(_) => out.push_str("D:discussion "),
                        Err(Error::NotFound) => (),
                        Err(e) => return Err(e),
                    }
                }
                out.push_str("Q:shelves\nLibrary entry> ");
            }
            State::Link(i) => {
                let file = resolve(b, s, c, i)?;
                writeln!(out, "{}", preview(&file.filename, width)).unwrap();
                writeln!(out, "{}", c.books[i].reference).unwrap();
                out.push_str(
                    "Current local file reference.\nUse existing Files/native transfers.\nI:notes ",
                );
                if self.files {
                    out.push_str("F:Files ");
                }
                out.push_str("Q:shelves\nLibrary link> ");
            }
        }
        if out.len() > 8192 {
            return Err(Error::Invalid("library frame budget"));
        }
        Ok(out)
    }
    pub fn feed(
        &mut self,
        input: &str,
        b: &mut Board,
        s: &Session,
        c: &Catalog,
    ) -> Result<Outcome> {
        approved(b, s)?;
        if !c.enabled() {
            return Err(Error::Denied);
        }
        if input.len() > 64 || !input.bytes().all(|b| (32..=126).contains(&b)) {
            return Err(Error::Invalid("bounded ASCII library command required"));
        }
        self.notice.clear();
        let cmd = input.to_ascii_lowercase();
        match self.state {
            State::List => match cmd.as_str() {
                "q" => return Ok(Outcome::Main),
                "e" => self.state = State::Shelves,
                "v" => self.state = State::Choose,
                "n" => self.page = (self.page + 1).min(BOOKS / 4),
                "p" => self.page = self.page.saturating_sub(1),
                _ => return Err(Error::Invalid("use displayed library commands")),
            },
            State::Shelves => {
                if !cmd.is_empty() {
                    self.shelf = if cmd == "-" {
                        None
                    } else {
                        let i = c
                            .shelves
                            .iter()
                            .position(|e| e.id == cmd)
                            .ok_or(Error::NotFound)?;
                        let visible = Self::visible(b, s, c)?;
                        if !visible
                            .iter()
                            .any(|(j, _)| c.books[*j].shelf == c.shelves[i].id)
                        {
                            return Err(Error::NotFound);
                        }
                        Some(i)
                    };
                    self.page = 0
                }
                self.state = State::List;
            }
            State::Choose => {
                if cmd.is_empty() {
                    self.state = State::List
                } else {
                    let i = c
                        .books
                        .iter()
                        .position(|w| w.id == cmd)
                        .ok_or(Error::NotFound)?;
                    resolve(b, s, c, i)?;
                    self.state = State::Info(i);
                    self.reset()
                }
            }
            State::Info(i) | State::Link(i) => {
                let file = resolve(b, s, c, i)?;
                match cmd.as_str() {
                    "q" => {
                        self.state = State::List;
                        self.reset()
                    }
                    "i" => {
                        self.state = State::Info(i);
                        self.reset()
                    }
                    "l" => self.state = State::Link(i),
                    "f" if self.files => return Ok(Outcome::File(file.id)),
                    "d" if matches!(self.state, State::Info(_)) => {
                        let room = c.books[i].room.ok_or(Error::NotFound)?;
                        b.unread_cursor(s, room)?;
                        return Ok(Outcome::Room(room));
                    }
                    "n" if matches!(self.state, State::Info(_)) => {
                        if let Some(n) = self.next {
                            if self.previous.len() == 32 {
                                self.previous.pop_front();
                            }
                            self.previous.push_back(self.offset);
                            self.offset = n
                        }
                    }
                    "p" if matches!(self.state, State::Info(_)) => {
                        self.offset = self.previous.pop_back().unwrap_or(0)
                    }
                    _ => return Err(Error::Invalid("use displayed entry commands")),
                }
            }
        }
        Ok(Outcome::Stay)
    }
    pub fn recover(&mut self, e: &Error) -> bool {
        if matches!(e, Error::Invalid(_) | Error::NotFound | Error::Busy) {
            self.notice = format!("Library: {e}");
            if matches!(e, Error::NotFound) {
                self.state = State::List;
                self.reset()
            }
            true
        } else {
            false
        }
    }
}
#[cfg(test)]
mod tests;
