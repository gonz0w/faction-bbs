//! Borrowed private body pages. Navigation retains offsets, never copied bodies.
use std::ops::Range;
use zeroize::Zeroizing;

// Two trusted pane-marker cells are part of the complete 39-cell row.
const CELLS: usize = 37;
const ROWS: usize = 10;
// At most four UTF-8 bytes per visible cell; escapes use one byte per cell.
const ROW_BYTES: usize = CELLS * 4;
pub struct Row {
    pub source: Range<usize>,
    pub text: Zeroizing<String>,
}
pub struct Page {
    pub rows: Vec<Row>,
}
#[derive(Clone)]
pub struct Body<'a> {
    source: &'a str,
    unicode: bool,
    offset: usize,
    finished: bool,
}
impl<'a> Body<'a> {
    pub fn new(source: &'a str, unicode: bool) -> Self {
        Self {
            source,
            unicode,
            offset: 0,
            finished: false,
        }
    }
    pub fn short_line(&self) -> bool {
        !self.source.contains('\n')
            && self
                .source
                .chars()
                .try_fold(0, |width, c| {
                    let part = Zeroizing::new(faction_terminal::text::scalar(c, self.unicode));
                    let next = width + faction_terminal::text::width(&part);
                    (next <= 33).then_some(next)
                })
                .is_some()
    }
    pub fn page_count(&self) -> usize {
        let mut scan = Self::new(self.source, self.unicode);
        let mut count = 0;
        while scan.next_page().is_some() {
            count += 1;
        }
        count
    }
    fn next_row(&mut self) -> Option<Row> {
        if self.finished {
            return None;
        }
        let start = self.offset;
        let mut text = Zeroizing::new(String::with_capacity(ROW_BYTES));
        let mut cells = 0;
        // An empty source or final LF has a truthful final empty logical row.
        if self.offset == self.source.len() {
            self.finished = true;
            return Some(Row {
                source: start..self.offset,
                text,
            });
        }
        while let Some(c) = self.source[self.offset..].chars().next() {
            if c == '\n' {
                self.offset += 1;
                return Some(Row {
                    source: start..self.offset,
                    text,
                });
            }
            let part = Zeroizing::new(faction_terminal::text::scalar(c, self.unicode));
            let width = faction_terminal::text::width(&part);
            if cells + width > CELLS {
                break;
            }
            assert!(text.len() + part.len() <= ROW_BYTES);
            text.push_str(&part);
            cells += width;
            self.offset += c.len_utf8();
            if self.offset == self.source.len() {
                self.finished = true;
                break;
            }
        }
        Some(Row {
            source: start..self.offset,
            text,
        })
    }
    pub fn next_page(&mut self) -> Option<Page> {
        let mut rows = Vec::with_capacity(ROWS);
        while rows.len() < ROWS {
            match self.next_row() {
                Some(row) => rows.push(row),
                None => break,
            }
        }
        (!rows.is_empty()).then_some(Page { rows })
    }
}
