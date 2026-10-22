//! UTF-8 subject input with deterministic, inert classic-terminal echo.
//! The byte budget is independent of display cells: non-ASCII scalars echo as
//! whole ASCII escapes. No locale, TERM value or terminal brand grants authority.
use crate::Result;
pub const BYTES: usize = 160;
#[derive(Debug, PartialEq, Eq)]
pub enum Edit {
    Echo(String),
    Submit(String),
}
pub fn feed(input: &mut Vec<u8>, byte: u8) -> Result<Edit> {
    feed_with(input, byte, false)
}
pub fn feed_with(input: &mut Vec<u8>, byte: u8, unicode: bool) -> Result<Edit> {
    if byte == b'\r' || byte == b'\n' {
        let text = std::str::from_utf8(input)
            .map_err(|_| "invalid UTF-8 subject")?
            .to_owned();
        input.clear();
        return Ok(Edit::Submit(text));
    }
    if byte == 8 || byte == 127 {
        let (start, cells) = match std::str::from_utf8(input) {
            Ok(text) => text.char_indices().next_back().map_or((0, 0), |(n, c)| {
                (n, crate::text::width(&crate::text::scalar(c, unicode)))
            }),
            Err(e) if e.error_len().is_none() => (e.valid_up_to(), 0),
            Err(_) => return Err("invalid UTF-8 subject"),
        };
        input.truncate(start);
        return Ok(Edit::Echo("\x08 \x08".repeat(cells)));
    }
    if byte < 32 || input.len() >= BYTES {
        return Err("invalid or oversized subject input");
    }
    input.push(byte);
    match std::str::from_utf8(input) {
        Ok(text) => {
            let c = text.chars().next_back().ok_or("invalid subject")?;
            if c.is_control() {
                return Err("subject controls are forbidden");
            }
            Ok(Edit::Echo(crate::text::scalar(c, unicode)))
        }
        Err(e) if e.error_len().is_none() => Ok(Edit::Echo(String::new())),
        Err(_) => Err("invalid UTF-8 subject"),
    }
}
/// A bounded single-line label. Whole escapes fit or are replaced by ellipsis;
/// even bidi, combining marks, newlines and terminal controls remain inert ASCII.
pub fn preview(text: &str, columns: usize) -> String {
    preview_with(text, columns, false)
}
pub fn preview_with(text: &str, columns: usize, unicode: bool) -> String {
    let columns = columns.min(119);
    let mut out = String::new();
    let mut boundaries = Vec::new();
    let mut cells = 0;
    for c in text.chars() {
        let part = crate::text::scalar(c, unicode);
        let size = crate::text::width(&part);
        if cells + size > columns {
            while cells > columns.saturating_sub(3) {
                let (bytes, previous) = boundaries.pop().unwrap_or((0, 0));
                out.truncate(bytes);
                cells = previous;
            }
            out.push_str(&".".repeat(columns.min(3)));
            return out;
        }
        boundaries.push((out.len(), cells));
        cells += size;
        out.push_str(&part);
    }
    out
}
