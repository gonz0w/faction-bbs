//! Bounded scalar rendering. Escape zero-width, controls and formatting scalars.
//! Ambiguous characters use one cell; members can recover with classic mode.
use unicode_width::UnicodeWidthChar;
fn literal(c: char, unicode: bool) -> bool {
    c.is_ascii_graphic()
        || c == ' '
        || (unicode
            && !c.is_control()
            && matches!(c.width(), Some(1 | 2))
            && !matches!(c,'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}'|'\u{200e}'|'\u{200f}'|'\u{61c}'|'\u{1f3fb}'..='\u{1f3ff}'|'\u{1f1e6}'..='\u{1f1ff}'))
}
pub fn scalar(c: char, unicode: bool) -> String {
    if c == '\n' {
        "\\n".into()
    } else if literal(c, unicode) {
        c.to_string()
    } else {
        c.escape_unicode().collect()
    }
}
pub fn cells(c: char) -> usize {
    c.width().unwrap_or(0)
}
pub fn width(s: &str) -> usize {
    s.chars().map(|c| c.width().unwrap_or(0)).sum()
}
pub fn render(s: &str, unicode: bool) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c == '\n' || literal(c, unicode) {
            out.push(c)
        } else {
            out.extend(c.escape_unicode())
        }
    }
    out
}
