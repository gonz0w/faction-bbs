//! Device-local preview metadata and bounded zeroized rendering. No state authority.
use super::inbox::{self, Entry, Inbox};
use zeroize::Zeroizing;

const PAGE: usize = 2;
pub struct Conversation {
    entries: Vec<Entry>,
    position: usize,
}
impl Conversation {
    pub fn new(inbox: &Inbox) -> Self {
        let mut model = Self {
            entries: Vec::new(),
            position: 0,
        };
        model.refresh(inbox);
        if let Some(selected) = inbox.selected() {
            model.position = model
                .entries
                .iter()
                .position(|e| e.index == selected.index)
                .unwrap_or(0);
        }
        model
    }
    pub fn refresh(&mut self, inbox: &Inbox) {
        let selected = self.selected();
        self.entries = inbox.eligible().to_vec();
        // Only canonical append indices order the view. Sender UTC has no role.
        self.entries.sort_unstable_by_key(|e| e.index);
        self.position = selected
            .and_then(|old| self.entries.iter().position(|e| e.index == old.index))
            .unwrap_or(self.position)
            .min(self.entries.len().saturating_sub(1));
    }
    pub fn selected(&self) -> Option<Entry> {
        self.entries.get(self.position).copied()
    }
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn visible(&self) -> &[Entry] {
        let start = self.position / PAGE * PAGE;
        &self.entries[start..self.entries.len().min(start + PAGE)]
    }
    pub fn page(&self) -> (usize, usize) {
        if self.entries.is_empty() {
            (0, 0)
        } else {
            (self.position / PAGE + 1, self.entries.len().div_ceil(PAGE))
        }
    }
    pub fn up(&mut self) {
        self.position = self.position.saturating_sub(1);
    }
    pub fn down(&mut self) {
        self.position = (self.position + 1).min(self.entries.len().saturating_sub(1));
    }
    pub fn next(&mut self) {
        let next = (self.position / PAGE + 1) * PAGE;
        if next < self.entries.len() {
            self.position = next;
        }
    }
    pub fn previous(&mut self) {
        if self.position >= PAGE {
            self.position = (self.position / PAGE - 1) * PAGE;
        }
    }
}
/// Text remains borrowed from the unlocked vault; model metadata never holds it.
pub struct Preview<'a> {
    pub entry: Entry,
    pub received: bool,
    pub sent_at: u64,
    pub subject: &'a str,
    pub body: &'a str,
    pub attachment: bool,
    pub selected: bool,
}
pub fn render(preview: Preview<'_>, unicode: bool) -> Zeroizing<String> {
    let mut out = Zeroizing::new(String::with_capacity(1024));
    out.push_str(&format!(
        "{} {:02} {} {}{}\r\nUTC {}\r\nSubject: ",
        if preview.selected { '>' } else { ' ' },
        preview.entry.index + 1,
        if preview.received { "RECEIVED" } else { "SENT" },
        preview.entry.status.label(),
        if preview.attachment { " @" } else { "" },
        preview.sent_at
    ));
    out.push_str(&inbox::compact(preview.subject, unicode, 30));
    out.push_str("\r\n");
    let mut chars = preview.body.chars().peekable();
    for _ in 0..3 {
        out.push_str("| ");
        let mut cells = 0;
        while let Some(&c) = chars.peek() {
            // Existing scalar rules escape controls, newlines, bidi and combining
            // marks. One scalar is kept whole across bounded preview rows.
            let scalar = Zeroizing::new(if c == '\n' {
                c.escape_unicode().collect()
            } else {
                faction_terminal::text::scalar(c, unicode)
            });
            let width = faction_terminal::text::width(&scalar);
            if cells + width > 37 {
                break;
            }
            out.push_str(&scalar);
            cells += width;
            chars.next();
        }
        out.push_str("\r\n");
    }
    out.push_str(if chars.peek().is_some() {
        "Body continues; O opens full record.\r\n"
    } else {
        "[O] Full record\r\n"
    });
    out
}
