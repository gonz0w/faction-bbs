use crate::Theme;
impl Theme {
    fn fitting_identity_ascii(&self, columns: u16, rows: u16) -> Option<&str> {
        self.identity_ascii().filter(|s| {
            s.lines().count() <= usize::from(rows.saturating_sub(5))
                && s.lines().all(|l| l.len() < usize::from(columns))
        })
    }
    /// Keep an actual compact identity card visible before the menu clears it.
    /// Plain line mode and compact board-name-only fallback retain their flow.
    pub fn identity_needs_pause(&self, columns: u16, rows: u16, ansi: bool) -> bool {
        ansi && matches!((columns, rows), (40..=240, 18..=100))
            && ((columns >= 80 && rows >= 24)
                || self.fitting_identity_ascii(columns, rows).is_some())
    }
    /// Trusted identity prefix, after approval. Art stays in the declared 80x16
    /// importer; smaller/plain terminals get a fitting ASCII asset or board name.
    /// The caller owns welcome/handle/pause and terminal input state.
    pub fn identity_screen(&self, columns: u16, rows: u16, ansi: bool) -> String {
        let ansi = ansi && matches!((columns, rows), (40..=240, 18..=100));
        let full = ansi && columns >= 80 && rows >= 24;
        let (normal, _, _) = self.menu_styles();
        let ascii = self.fitting_identity_ascii(columns, rows);
        let text = ascii.unwrap_or(self.board());
        if !ansi {
            return text.to_owned();
        }
        let mut out = format!("{}\x1b[2J\x1b[H", normal.sgr());
        if full {
            if let Some(art) = self.identity_ansi() {
                out.push_str(&art.render());
            } else {
                out.push_str(text);
            }
            out.push_str("\x1b[18;1H");
        } else {
            out.push_str(text);
        }
        out.push_str(&normal.sgr());
        out
    }
}
