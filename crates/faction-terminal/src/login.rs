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
    pub fn identity_page_count(&self, columns: u16, rows: u16, ansi: bool) -> usize {
        if ansi && matches!((columns, rows), (80..=240, 24..=100)) {
            self.identity_ansi().map_or(1, |a| a.page_count())
        } else {
            1
        }
    }
    pub fn identity_page_prompt(
        &self,
        page: usize,
        columns: u16,
        rows: u16,
        ansi: bool,
    ) -> Option<String> {
        let count = self.identity_page_count(columns, rows, ansi);
        (page < count && page + 1 < count).then(|| {
            format!(
                "Art page {}/{}. Enter: next; S+Enter: skip: ",
                page + 1,
                count
            )
        })
    }
    /// First-page compatibility API. Adapters explicitly page a long identity.
    pub fn identity_screen(&self, columns: u16, rows: u16, ansi: bool) -> String {
        self.identity_page_screen(columns, rows, ansi, 0)
            .expect("first identity page")
    }
    /// Approved canonical prefix only. Adapters own the welcome/pause and input.
    pub fn identity_page_screen(
        &self,
        columns: u16,
        rows: u16,
        ansi: bool,
        page: usize,
    ) -> Option<String> {
        if page >= self.identity_page_count(columns, rows, ansi) {
            return None;
        }
        let ansi = ansi && matches!((columns, rows), (40..=240, 18..=100));
        let full = ansi && columns >= 80 && rows >= 24;
        let (normal, _, _) = self.menu_styles();
        let text = self
            .fitting_identity_ascii(columns, rows)
            .unwrap_or(self.board());
        if !ansi {
            return Some(text.to_owned());
        }
        let mut out = format!("{}\x1b[2J\x1b[H", normal.sgr());
        if full {
            if let Some(art) = self.identity_ansi() {
                out.push_str(&art.render_page(page)?);
            } else {
                out.push_str(text);
            }
            out.push_str("\x1b[18;1H");
        } else {
            out.push_str(text);
        }
        out.push_str(&normal.sgr());
        Some(out)
    }
    /// Skip/resize recovery never replays the first page of a long asset.
    pub fn identity_fallback_screen(&self, columns: u16, rows: u16, ansi: bool) -> String {
        let text = self
            .fitting_identity_ascii(columns, rows)
            .unwrap_or(self.board());
        if ansi && matches!((columns, rows), (40..=240, 18..=100)) {
            format!("{}\x1b[2J\x1b[H{}\r\n", self.menu_styles().0.sgr(), text)
        } else {
            text.to_owned()
        }
    }
}
