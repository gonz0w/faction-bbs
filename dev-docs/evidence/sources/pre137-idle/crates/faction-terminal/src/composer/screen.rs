//! Cell-aware, bounded full-screen room editing. Terminal input grants no action authority.
use super::*;
use crate::{Decoder, Key, Theme, subject, text};
use std::time::Instant;
#[derive(Default)]
pub struct Screen {
    caret: usize,
    decoder: Decoder,
    scalar: Vec<u8>,
    top: usize,
}
#[derive(Debug, PartialEq, Eq)]
pub enum ScreenEvent {
    Changed,
    Command(Command),
    None,
}
struct Row {
    start: usize,
    end: usize,
    text: String,
}
impl Screen {
    pub fn at_end(draft: &Draft) -> Self {
        Self {
            caret: draft.body.len(),
            ..Self::default()
        }
    }
    pub fn reset_input(&mut self) {
        self.decoder = Decoder::new();
        self.scalar.clear();
    }
    fn rows(draft: &Draft, width: usize, unicode: bool) -> Vec<Row> {
        let mut rows = Vec::new();
        let mut start = 0;
        let mut part = String::new();
        let mut cells = 0;
        for (i, c) in draft.body.char_indices() {
            if c == '\n' {
                rows.push(Row {
                    start,
                    end: i,
                    text: part,
                });
                start = i + 1;
                part = String::new();
                cells = 0;
                continue;
            }
            let shown = text::scalar(c, unicode);
            let n = text::width(&shown);
            if cells + n > width {
                rows.push(Row {
                    start,
                    end: i,
                    text: part,
                });
                start = i;
                part = String::new();
                cells = 0;
            }
            cells += n;
            part.push_str(&shown);
        }
        rows.push(Row {
            start,
            end: draft.body.len(),
            text: part,
        });
        rows
    }
    fn location(&self, rows: &[Row], draft: &Draft, unicode: bool) -> (usize, usize) {
        let row = rows
            .iter()
            .rposition(|r| r.start <= self.caret)
            .unwrap_or(0);
        let col = text::width(&text::render(
            &draft.body[rows[row].start..self.caret],
            unicode,
        ));
        (row, col)
    }
    fn insert(&mut self, draft: &mut Draft, text: &str) -> Result<ScreenEvent> {
        draft.replace(self.caret, self.caret, text)?;
        self.caret += text.len();
        Ok(ScreenEvent::Changed)
    }
    pub fn feed(
        &mut self,
        draft: &mut Draft,
        byte: u8,
        now: Instant,
        size: (u16, u16),
        unicode: bool,
    ) -> Result<ScreenEvent> {
        if byte == 3 {
            self.reset_input();
            return Ok(ScreenEvent::Command(Command::Cancel));
        }
        if !self.scalar.is_empty() {
            self.scalar.push(byte);
            match std::str::from_utf8(&self.scalar) {
                Ok(s) => {
                    if s.chars().any(char::is_control) {
                        return Err("editor control input");
                    }
                    let s = s.to_owned();
                    self.scalar.clear();
                    return self.insert(draft, &s);
                }
                Err(e) if e.error_len().is_none() && self.scalar.len() < 4 => {
                    return Ok(ScreenEvent::None);
                }
                Err(_) => return Err("invalid UTF-8 editor input"),
            }
        }
        let ready = self.decoder.text_ready();
        if ready {
            if (0xc2..=0xf4).contains(&byte) {
                self.scalar.push(byte);
                return Ok(ScreenEvent::None);
            }
            if byte >= 128 {
                return Err("invalid UTF-8 editor input");
            }
            let command = match byte {
                16 => Some(Command::Preview),
                19 => Some(Command::Save),
                20 => Some(Command::Send),
                7 => Some(Command::Quote),
                17 => Some(Command::Line),
                3 => Some(Command::Cancel),
                _ => None,
            };
            if let Some(c) = command {
                return Ok(ScreenEvent::Command(c));
            }
            match byte {
                1 => {
                    self.caret = draft.body[..self.caret].rfind('\n').map_or(0, |i| i + 1);
                    return Ok(ScreenEvent::Changed);
                }
                5 => {
                    self.caret = draft.body[self.caret..]
                        .find('\n')
                        .map_or(draft.body.len(), |i| self.caret + i);
                    return Ok(ScreenEvent::Changed);
                }
                8 | 127 => {
                    if let Some((i, _)) = draft.body[..self.caret].char_indices().next_back() {
                        draft.replace(i, self.caret, "")?;
                        self.caret = i;
                    }
                    return Ok(ScreenEvent::Changed);
                }
                4 => {
                    if let Some(c) = draft.body[self.caret..].chars().next() {
                        draft.replace(self.caret, self.caret + c.len_utf8(), "")?;
                    }
                    return Ok(ScreenEvent::Changed);
                }
                11 => {
                    let end = draft.body[self.caret..]
                        .find('\n')
                        .map_or(draft.body.len(), |i| self.caret + i);
                    draft.replace(self.caret, end, "")?;
                    return Ok(ScreenEvent::Changed);
                }
                21 => {
                    let start = draft.body[..self.caret].rfind('\n').map_or(0, |i| i + 1);
                    let end = draft.body[self.caret..]
                        .find('\n')
                        .map_or(draft.body.len(), |i| self.caret + i + 1);
                    draft.replace(start, end, "")?;
                    self.caret = start;
                    return Ok(ScreenEvent::Changed);
                }
                _ => (),
            }
        }
        let key = self.decoder.feed(byte, now);
        if self.decoder.quarantined() {
            return Err("unsupported editor escape sequence");
        }
        match key {
            Some(Key::Left) => {
                self.caret = draft.body[..self.caret]
                    .char_indices()
                    .next_back()
                    .map_or(0, |(i, _)| i);
            }
            Some(Key::Right) => {
                if let Some(c) = draft.body[self.caret..].chars().next() {
                    self.caret += c.len_utf8();
                }
            }
            Some(Key::Up | Key::Down) => {
                let rows = Self::rows(
                    draft,
                    usize::from(size.0).saturating_sub(1).clamp(39, 119),
                    unicode,
                );
                let (row, col) = self.location(&rows, draft, unicode);
                let next = if key == Some(Key::Up) {
                    row.saturating_sub(1)
                } else {
                    (row + 1).min(rows.len() - 1)
                };
                self.caret = rows[next].start;
                let mut cells = 0;
                for (i, c) in draft.body[rows[next].start..rows[next].end].char_indices() {
                    let n = text::width(&text::scalar(c, unicode));
                    if cells + n > col {
                        break;
                    }
                    cells += n;
                    self.caret = rows[next].start + i + c.len_utf8();
                }
            }
            Some(Key::Enter) => return self.insert(draft, "\n"),
            _ if ready && (32..=126).contains(&byte) => {
                return self.insert(draft, &(byte as char).to_string());
            }
            _ => return Ok(ScreenEvent::None),
        }
        Ok(ScreenEvent::Changed)
    }
    pub fn render(
        &mut self,
        draft: &Draft,
        target: &Target<'_>,
        size: (u16, u16),
        unicode: bool,
        theme: &Theme,
        status: (bool, &str),
    ) -> String {
        let (saved, notice) = status;
        let width = usize::from(size.0).saturating_sub(1).clamp(39, 119);
        let height = usize::from(size.1).clamp(18, 100).saturating_sub(9).min(32);
        self.caret = self.caret.min(draft.body.len());
        let rows = Self::rows(draft, width, unicode);
        let (row, col) = self.location(&rows, draft, unicode);
        if row < self.top {
            self.top = row;
        } else if row >= self.top + height {
            self.top = row + 1 - height;
        }
        let title = format!("{} | room #{}", theme.writing_title(), target.room);
        let title = if title.len() > width {
            format!("Room #{}", target.room)
        } else {
            title
        };
        let mut out = crate::section(theme, &title, true);
        out.push_str(&subject::preview_with(
            &format!(
                "Subject: {}",
                if target.subject.is_empty() {
                    "(default on send)"
                } else {
                    target.subject
                }
            ),
            width,
            unicode,
        ));
        out.push('\n');
        out.push_str(&subject::preview_with(
            &format!(
                "{}B | {} | {}",
                draft.body.len(),
                if saved {
                    "checkpoint exists"
                } else {
                    "not saved"
                },
                notice
            ),
            width,
            false,
        ));
        out.push('\n');
        for i in self.top..self.top + height {
            if let Some(r) = rows.get(i) {
                out.push_str(&r.text);
            }
            out.push('\n');
        }
        out.push_str("^P Preview  ^S Save  ^T Send\n^G Quote  ^Q Line  ^C Cancel\nArrows edit; ^A/^E start/end\n");
        out.push_str(&format!("\x1b[{};{}H", 6 + row - self.top, col + 1));
        out
    }
}
