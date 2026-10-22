//! Inert, bounded message pages shared by local and service presentations.
use crate::{Result, Theme};

pub const MESSAGE_BYTES: usize = 8192;

/// This is presentation data, never an authorization token or a wire article.
pub struct Message<'a> {
    pub room: i64,
    pub id: i64,
    pub author: &'a str,
    pub reply_to: Option<i64>,
    pub body: &'a str,
}
pub struct MessageDetails<'a> {
    pub subject: &'a str,
    pub posted: &'a str,
}
pub struct MessagePage {
    pub text: String,
    /// Byte offset in the rendered body. Keep the same source and rendering mode.
    pub next: Option<usize>,
}
fn inert(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if c.is_ascii_graphic() || c == ' ' || c == '\n' {
            out.push(c);
        } else {
            out.extend(c.escape_unicode());
        }
    }
    out
}
/// Trusted section change only; plain sessions emit no escape sequences.
pub fn section(theme: &Theme, title: &str, ansi: bool) -> String {
    let heading = format!("{}\n{}\n\n", inert(theme.board()), inert(title));
    if ansi {
        format!(
            "\x1b[?25h{}\x1b[2J\x1b[H{heading}",
            theme.menu_styles().0.sgr()
        )
    } else {
        format!("\n{heading}")
    }
}
fn line(text: &str, width: usize) -> (&str, usize) {
    let mut end = 0;
    let mut cells = 0;
    for (i, c) in text.char_indices() {
        if c == '\n' || cells + crate::text::cells(c) > width {
            break;
        }
        cells += crate::text::cells(c);
        end = i + c.len_utf8();
    }
    // Keep a complete public file reference in the byte stream for native
    // recent-link discovery and plain SSH copy/paste. A narrow terminal may
    // soft-wrap this one 78-byte token; account for its physical rows below.
    if let Some(start) = text.find("faction-file:")
        && start < end
    {
        let finish = start + 78;
        if let Some(token) = text.get(start..finish)
            && token.as_bytes()[45] == b'/'
            && token[13..45]
                .bytes()
                .chain(token[46..].bytes())
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            && finish > end
        {
            end = if start == 0 { finish } else { start };
        }
    }
    if end == text.len() {
        return (text, end);
    }
    if text.as_bytes()[end] == b'\n' {
        return (&text[..end], end + 1);
    }
    let wrap = text[..end]
        .rfind(' ')
        .filter(|n| *n >= width / 2)
        .map_or(end, |n| n + 1);
    (&text[..wrap], wrap)
}
fn heading(out: &mut String, text: &str, width: usize) -> usize {
    let mut remaining = text;
    let mut lines = 0;
    while !remaining.is_empty() {
        let (part, consumed) = line(remaining, width);
        out.push_str(part.trim_end());
        out.push('\n');
        remaining = &remaining[consumed..];
        lines += 1;
    }
    lines
}
impl Message<'_> {
    pub fn page(&self, columns: u16, rows: u16, offset: usize) -> Result<MessagePage> {
        self.page_inner(columns, rows, offset, None, false)
    }
    pub fn page_with_metadata(
        &self,
        columns: u16,
        rows: u16,
        offset: usize,
        details: MessageDetails<'_>,
    ) -> Result<MessagePage> {
        if details.subject.len() > 160 || details.posted.len() > 64 {
            return Err("message metadata limit");
        }
        self.page_inner(columns, rows, offset, Some(details), false)
    }
    pub fn page_with_capabilities(
        &self,
        columns: u16,
        rows: u16,
        offset: usize,
        details: MessageDetails<'_>,
        unicode: bool,
    ) -> Result<MessagePage> {
        if details.subject.len() > 160 || details.posted.len() > 64 {
            return Err("message metadata limit");
        }
        self.page_inner(columns, rows, offset, Some(details), unicode)
    }
    fn page_inner(
        &self,
        columns: u16,
        rows: u16,
        offset: usize,
        details: Option<MessageDetails<'_>>,
        unicode: bool,
    ) -> Result<MessagePage> {
        if self.body.len() > MESSAGE_BYTES || self.author.len() > 64 {
            return Err("message presentation limit");
        }
        let width = usize::from(columns).saturating_sub(1).clamp(39, 119);
        let height = usize::from(rows).clamp(18, 100);
        let body = crate::text::render(self.body, unicode);
        if offset > body.len() || !body.is_char_boundary(offset) {
            return Err("message page offset");
        }
        let mut text = String::new();
        let mut used = heading(
            &mut text,
            &format!("Room #{} | chronological messages", self.room),
            width,
        );
        text.push_str(&"-".repeat(width));
        text.push('\n');
        used += 1;
        used += heading(
            &mut text,
            &format!("#{} by {}", self.id, inert(self.author)),
            width,
        );
        if let Some(parent) = self.reply_to {
            used += heading(&mut text, &format!("Reply to #{parent}"), width);
        }
        if let Some(details) = details {
            for (name, value) in [("Subject", details.subject), ("Posted", details.posted)] {
                let label =
                    crate::subject::preview_with(&format!("{name}: {value}"), width, unicode);
                text.push_str(&label);
                text.push('\n');
                used += 1;
            }
        }
        if offset > 0 {
            text.push_str("(message continued)\n");
            used += 1;
        }
        text.push('\n');
        used += 1;
        // Reserve the final separator, prompt and terminal's current input line.
        let budget = height.saturating_sub(used + 6).clamp(1, 32);
        let mut position = offset;
        let mut remaining = budget;
        while remaining > 0 && position < body.len() {
            let (part, consumed) = line(&body[position..], width);
            let cost = crate::text::width(part).div_ceil(width).max(1);
            if cost > remaining {
                break;
            }
            text.push_str(part.trim_end());
            text.push('\n');
            position += consumed;
            remaining -= cost;
        }
        if position == offset && position < body.len() {
            return Err("terminal too small for message reference");
        }
        text.push_str(&"-".repeat(width));
        text.push('\n');
        Ok(MessagePage {
            text,
            next: (position < body.len()).then_some(position),
        })
    }
}
