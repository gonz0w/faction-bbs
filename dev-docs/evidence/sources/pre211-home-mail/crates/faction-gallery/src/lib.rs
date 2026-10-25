//! Approved-member gallery workflow. The immutable catalog is loaded once by the host.
#![forbid(unsafe_code)]
mod catalog;
pub use catalog::{ART_FRAME_BYTES, CATALOG_BYTES, Catalog, PAGES, WORKS};
use faction_core::{Board, Error, Result, Session, Status};
use faction_terminal::{Theme, subject::preview};
use std::{collections::VecDeque, fmt::Write};
#[derive(Clone, Copy)]
enum State {
    List,
    Exhibit,
    Choose,
    Info(usize),
    Art(usize),
    Download(usize),
}
pub struct Desk {
    state: State,
    exhibit: Option<usize>,
    list_page: usize,
    art_page: usize,
    offset: usize,
    next: Option<usize>,
    previous: VecDeque<usize>,
    text: bool,
    dimensions: Option<(u16, u16, bool)>,
    notice: String,
}
impl Default for Desk {
    fn default() -> Self {
        Self {
            state: State::List,
            exhibit: None,
            list_page: 0,
            art_page: 0,
            offset: 0,
            next: None,
            previous: VecDeque::new(),
            text: false,
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
fn lines(out: &mut String, s: &str, offset: usize, width: usize, rows: usize) -> Option<usize> {
    let mut at = offset.min(s.len());
    for _ in 0..rows {
        if at == s.len() {
            break;
        }
        let rest = &s[at..];
        let len = rest.find('\n').unwrap_or(rest.len()).min(width);
        out.push_str(&rest[..len]);
        out.push('\n');
        at += len;
        if s.as_bytes().get(at) == Some(&b'\n') {
            at += 1;
        }
    }
    (at < s.len()).then_some(at)
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
impl Desk {
    fn reset(&mut self) {
        self.offset = 0;
        self.previous.clear();
        self.next = None;
        self.art_page = 0;
    }
    fn ids(&self, c: &Catalog) -> Vec<usize> {
        c.works
            .iter()
            .enumerate()
            .filter(|(_, w)| self.exhibit.is_none_or(|i| w.exhibit == c.exhibits[i].id))
            .map(|(i, _)| i)
            .collect()
    }
    pub fn render(
        &mut self,
        b: &mut Board,
        s: &Session,
        c: &Catalog,
        theme: &Theme,
        ansi: bool,
        size: (u16, u16),
    ) -> Result<String> {
        approved(b, s)?;
        if !c.enabled() {
            return Err(Error::Denied);
        }
        if self
            .dimensions
            .is_some_and(|old| old != (size.0, size.1, ansi))
        {
            self.reset();
        }
        self.dimensions = Some((size.0, size.1, ansi));
        let width = (size.0 as usize).clamp(20, 80) - 1;
        let mut out = faction_terminal::section(theme, &c.name, ansi);
        if !self.notice.is_empty() {
            writeln!(out, "{}", preview(&self.notice, width)).unwrap();
        }
        match self.state {
            State::List => {
                let ids = self.ids(c);
                let max = ids.len().saturating_sub(1) / 4;
                self.list_page = self.list_page.min(max);
                writeln!(
                    out,
                    "Exhibit: {} / page {}",
                    self.exhibit
                        .map(|i| preview(&c.exhibits[i].title, width.saturating_sub(20)))
                        .unwrap_or_else(|| "all".into()),
                    self.list_page + 1
                )
                .unwrap();
                for i in ids.into_iter().skip(self.list_page * 4).take(4) {
                    let w = &c.works[i];
                    writeln!(out, "{}", preview(&format!("{}: {}", w.id, w.title), width)).unwrap();
                    if !c.compact {
                        writeln!(
                            out,
                            " {}",
                            preview(&format!("{} / {}", w.artist, w.format), width - 1)
                        )
                        .unwrap();
                    }
                }
                out.push_str("E:exhibit V:work N/P:page Q:main\nGallery> ");
            }
            State::Exhibit => {
                out.push_str("Exhibits (- means all):\n");
                for e in &c.exhibits {
                    writeln!(out, "{}", preview(&format!("{}: {}", e.id, e.title), width)).unwrap();
                }
                out.push_str("Exhibit ID (blank cancels): ");
            }
            State::Choose => out.push_str("Work ID (blank cancels): "),
            State::Info(i) => {
                let w = &c.works[i];
                let text = inert(&format!(
                    "{}\nArtist: {}\n{}\nRights: {}\nCurator declares: {}\nProvenance: {}\nLocal approved-member display.\n",
                    w.title, w.artist, w.description, w.license, w.source, w.provenance
                ));
                self.next = lines(&mut out, &text, self.offset, width, 8);
                out.push_str("A:art N/P:notes ");
                if w.download.is_some() {
                    out.push_str("L:file link ");
                }
                out.push_str("Q:catalog\nWork> ");
            }
            State::Art(i) => {
                let w = &c.works[i];
                if ansi && size.0 >= 80 && size.1 >= 24 && !self.text {
                    let page = self.art_page.min(w.art.page_count() - 1);
                    out.clear();
                    out.push_str(&format!("{}\x1b[2J\x1b[H", theme.menu_styles().0.sgr()));
                    out.push_str(&w.art.render_page(page).unwrap());
                    out.push_str("\x1b[17;1H");
                    writeln!(
                        out,
                        "{}",
                        preview(&format!("{} / {}", w.title, w.artist), 79)
                    )
                    .unwrap();
                    writeln!(
                        out,
                        "Page {}/{} / {}",
                        page + 1,
                        w.art.page_count(),
                        preview(&w.license, 48)
                    )
                    .unwrap();
                    out.push_str("N/P:art page T:text I:notes Q:catalog\nArt> ");
                } else {
                    writeln!(
                        out,
                        "{} / text edition",
                        preview(&w.title, width.saturating_sub(15))
                    )
                    .unwrap();
                    self.next = lines(&mut out, &w.plain, self.offset, width, 8);
                    out.push_str("N/P:text page T:art I:notes Q:catalog\nArt> ");
                }
            }
            State::Download(i) => {
                let w = &c.works[i];
                let reference = w.download.as_deref().ok_or(Error::Denied)?;
                match b.file_resolve(s, reference) {
                    Ok(file) => {
                        writeln!(
                            out,
                            "{} / {} bytes",
                            preview(&file.filename, width.saturating_sub(20)),
                            file.bytes
                        )
                        .unwrap();
                        writeln!(out, "{reference}").unwrap();
                        out.push_str("Use the existing Files transfer workflow.\n");
                    }
                    Err(Error::Denied | Error::NotFound) => {
                        out.push_str("Download unavailable on this board.\n")
                    }
                    Err(e) => return Err(e),
                }
                out.push_str("I:notes Q:catalog\nGallery file> ");
            }
        }
        if out.len() > 8192 {
            return Err(Error::Invalid("gallery frame exceeds budget"));
        }
        Ok(out)
    }
    pub fn feed(&mut self, input: &str, b: &mut Board, s: &Session, c: &Catalog) -> Result<bool> {
        approved(b, s)?;
        if !c.enabled() {
            return Err(Error::Denied);
        }
        if input.len() > 64 || !input.bytes().all(|b| (32..=126).contains(&b)) {
            return Err(Error::Invalid("bounded printable gallery command required"));
        }
        let cmd = input.to_ascii_lowercase();
        self.notice.clear();
        match self.state {
            State::List => match cmd.as_str() {
                "q" => return Ok(true),
                "e" => self.state = State::Exhibit,
                "v" => self.state = State::Choose,
                "n" => {
                    self.list_page =
                        (self.list_page + 1).min(self.ids(c).len().saturating_sub(1) / 4)
                }
                "p" => self.list_page = self.list_page.saturating_sub(1),
                _ => return Err(Error::Invalid("choose a displayed gallery command")),
            },
            State::Exhibit => {
                if !cmd.is_empty() {
                    self.exhibit = if cmd == "-" {
                        None
                    } else {
                        Some(
                            c.exhibits
                                .iter()
                                .position(|e| e.id == cmd)
                                .ok_or(Error::NotFound)?,
                        )
                    };
                    self.list_page = 0;
                }
                self.state = State::List;
            }
            State::Choose => {
                if cmd.is_empty() {
                    self.state = State::List;
                } else {
                    let i = c
                        .works
                        .iter()
                        .position(|w| w.id == cmd)
                        .ok_or(Error::NotFound)?;
                    self.state = State::Info(i);
                    self.reset();
                }
            }
            State::Info(i) => match cmd.as_str() {
                "q" => {
                    self.state = State::List;
                    self.reset();
                }
                "a" => {
                    self.state = State::Art(i);
                    self.text = false;
                    self.reset();
                }
                "n" => self.next_page(),
                "p" => self.offset = self.previous.pop_back().unwrap_or(0),
                "l" if c.works[i].download.is_some() => self.state = State::Download(i),
                _ => return Err(Error::Invalid("choose a displayed work command")),
            },
            State::Art(i) => match cmd.as_str() {
                "q" => {
                    self.state = State::List;
                    self.reset();
                }
                "i" => {
                    self.state = State::Info(i);
                    self.reset();
                }
                "t" => {
                    self.text = !self.text;
                    self.reset();
                }
                "n" => {
                    if self
                        .dimensions
                        .is_some_and(|(w, h, a)| a && w >= 80 && h >= 24)
                        && !self.text
                    {
                        self.art_page = (self.art_page + 1).min(c.works[i].art.page_count() - 1);
                    } else {
                        self.next_page();
                    }
                }
                "p" => {
                    if self
                        .dimensions
                        .is_some_and(|(w, h, a)| a && w >= 80 && h >= 24)
                        && !self.text
                    {
                        self.art_page = self.art_page.saturating_sub(1);
                    } else {
                        self.offset = self.previous.pop_back().unwrap_or(0);
                    }
                }
                _ => return Err(Error::Invalid("choose a displayed art command")),
            },
            State::Download(i) => match cmd.as_str() {
                "i" => {
                    self.state = State::Info(i);
                    self.reset();
                }
                "q" => self.state = State::List,
                _ => return Err(Error::Invalid("use I or Q")),
            },
        }
        Ok(false)
    }
    fn next_page(&mut self) {
        if let Some(next) = self.next {
            if self.previous.len() == 32 {
                self.previous.pop_front();
            }
            self.previous.push_back(self.offset);
            self.offset = next;
        }
    }
    pub fn recover(&mut self, e: &Error) -> bool {
        if matches!(e, Error::Invalid(_) | Error::NotFound | Error::Busy) {
            self.notice = format!("Gallery: {e}");
            if matches!(e, Error::NotFound) {
                self.state = State::List;
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests;
