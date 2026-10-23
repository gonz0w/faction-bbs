//! Bounded room navigation. IDs are presentation hints, never authorization.
use crate::{Key, Theme, subject::preview_with};
pub const LIMIT: usize = 100;
const PAGE: usize = 4;
pub struct Room<'a> {
    pub id: i64,
    pub slug: &'a str,
    pub title: &'a str,
    pub unread: i64,
}
#[derive(Default, Clone, Copy, Debug)]
pub struct View {
    selected: Option<i64>,
    detail: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Choice {
    Stay,
    Main,
    Read { room: i64, new_only: bool },
    Post(i64),
}
impl View {
    fn sync(&mut self, rooms: &[Room<'_>]) -> usize {
        if let Some(i) = rooms
            .iter()
            .take(LIMIT)
            .position(|r| Some(r.id) == self.selected)
        {
            return i;
        }
        self.selected = rooms.first().map(|r| r.id);
        self.detail = false;
        0
    }
    pub fn is_card(&self) -> bool {
        self.detail
    }
    pub fn feed(&mut self, key: Key, rooms: &[Room<'_>]) -> Choice {
        if key == Key::Hotkey(b'Q') {
            return Choice::Main;
        }
        let i = self.sync(rooms);
        let count = rooms.len().min(LIMIT);
        if count == 0 {
            return Choice::Stay;
        }
        if key == Key::Hotkey(b'U') {
            if let Some(next) = (1..=count)
                .map(|n| (i + n) % count)
                .find(|n| rooms[*n].unread > 0)
            {
                self.selected = Some(rooms[next].id);
                self.detail = false;
            }
            return Choice::Stay;
        }
        if self.detail {
            return match key {
                Key::Hotkey(b'L') => {
                    self.detail = false;
                    Choice::Stay
                }
                Key::Enter | Key::Hotkey(b'N') => Choice::Read {
                    room: rooms[i].id,
                    new_only: true,
                },
                Key::Hotkey(b'H') => Choice::Read {
                    room: rooms[i].id,
                    new_only: false,
                },
                Key::Hotkey(b'P') => Choice::Post(rooms[i].id),
                _ => Choice::Stay,
            };
        }
        let next = match key {
            Key::Up => (i + count - 1) % count,
            Key::Down | Key::Next => (i + 1) % count,
            Key::Right | Key::Hotkey(b'N') => {
                if i / PAGE + 1 < count.div_ceil(PAGE) {
                    (i / PAGE + 1) * PAGE
                } else {
                    0
                }
            }
            Key::Left | Key::Hotkey(b'P') => {
                if i >= PAGE {
                    (i / PAGE - 1) * PAGE
                } else {
                    (count - 1) / PAGE * PAGE
                }
            }
            Key::Enter => {
                self.detail = true;
                i
            }
            Key::Hotkey(k @ b'A'..=b'D') => {
                let n = i / PAGE * PAGE + usize::from(k - b'A');
                if n < count {
                    self.detail = true;
                    n
                } else {
                    i
                }
            }
            _ => i,
        };
        self.selected = Some(rooms[next].id);
        Choice::Stay
    }
    /// Single bounded frame. Fetch fresh permission-filtered rows at the caller.
    pub fn render(
        &mut self,
        rooms: &[Room<'_>],
        columns: u16,
        ansi: bool,
        unicode: bool,
        theme: &Theme,
    ) -> String {
        let i = self.sync(rooms);
        let width = usize::from(columns.clamp(40, 120)) - 1;
        let count = rooms.len().min(LIMIT);
        let mut lines = vec![
            ("ROOMS / local conversations".to_owned(), true),
            ("Choose a room; keep the discussion going.".into(), false),
            (String::new(), false),
        ];
        if count == 0 {
            lines.push(("No rooms available.".into(), false));
            lines.push(("Q: main menu".into(), false));
        } else if self.detail {
            let r = &rooms[i];
            lines.extend([
                (format!("ROOM #{}", r.id), true),
                (format!("Slug: {}", r.slug), false),
                (format!("Title: {}", r.title), false),
                (format!("{} unread messages", r.unread), false),
                (String::new(), false),
                ("N/Enter:new H:history P:post".into(), false),
                ("L:list U:next unread Q:main".into(), false),
                ("Finished actions return to main menu.".into(), false),
            ]);
        } else {
            let start = i / PAGE * PAGE;
            lines.push((
                format!("Page {} / {}", start / PAGE + 1, count.div_ceil(PAGE)),
                false,
            ));
            for (n, r) in rooms
                .iter()
                .enumerate()
                .skip(start)
                .take(PAGE)
                .take(count - start)
            {
                lines.push((
                    format!(
                        "{} [{}] #{} ({} unread)",
                        if n == i { ">" } else { " " },
                        char::from(b'A' + (n - start) as u8),
                        r.id,
                        r.unread
                    ),
                    n == i,
                ));
                lines.push((format!("  {} - {}", r.slug, r.title), false));
            }
            lines.extend([
                ("Arrows/Tab:select A-D/Enter:open".into(), false),
                ("N/P:page U:next unread Q:main".into(), false),
                ("First 100 rooms; counts refresh on keys.".into(), false),
            ]);
        }
        let (normal, focus, _) = theme.menu_styles();
        let mut out = if ansi {
            "\x1b[2J\x1b[H".to_owned()
        } else {
            String::new()
        };
        for (n, (line, selected)) in lines.iter().enumerate() {
            let text = preview_with(line, width, unicode);
            if ansi {
                out.push_str(&format!(
                    "\x1b[{};1H{}{}\x1b[K",
                    n + 1,
                    if *selected { focus.sgr() } else { normal.sgr() },
                    text
                ));
            } else {
                out.push_str(&text);
                out.push('\n');
            }
        }
        if ansi {
            out.push_str("\x1b[0m");
        }
        debug_assert!(out.len() <= 8192);
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn rows() -> Vec<Room<'static>> {
        (1..=100)
            .map(|id| Room {
                id,
                slug: "room",
                title: "Café 界\u{1b}[31m",
                unread: i64::from(id == 8),
            })
            .collect()
    }
    #[test]
    fn pages_named_cards_and_actions_bind_the_selected_room() {
        let rooms = rows();
        let mut v = View::default();
        v.render(&rooms, 80, false, false, &Theme::default());
        v.feed(Key::Right, &rooms);
        assert!(
            v.render(&rooms, 80, false, false, &Theme::default())
                .contains("Page 2 / 25")
        );
        v.feed(Key::Hotkey(b'D'), &rooms);
        assert!(v.is_card());
        assert_eq!(
            v.feed(Key::Hotkey(b'H'), &rooms),
            Choice::Read {
                room: 8,
                new_only: false
            }
        );
        assert_eq!(v.feed(Key::Hotkey(b'P'), &rooms), Choice::Post(8));
        assert_eq!(
            v.feed(Key::Enter, &rooms),
            Choice::Read {
                room: 8,
                new_only: true
            }
        );
        assert_eq!(v.feed(Key::Hotkey(b'Q'), &rooms), Choice::Main);
    }
    #[test]
    fn compact_classic_and_unicode_are_inert_and_bounded() {
        let rooms = rows();
        let mut v = View::default();
        for unicode in [false, true] {
            let text = v.render(&rooms, 40, false, unicode, &Theme::default());
            assert!(!text.contains('\x1b') && text.lines().count() <= 18);
            assert!(text.lines().all(|s| crate::text::width(s) < 40));
            assert!(text.contains("\\u{1b}"));
            assert_eq!(text.contains("Café 界"), unicode);
        }
        assert!(v.render(&rooms, 120, true, true, &Theme::default()).len() < 8192);
    }
    #[test]
    fn selection_tracks_id_not_offset_and_next_unread_does_not_mark_read() {
        let mut rooms = rows();
        let mut v = View::default();
        v.feed(Key::Hotkey(b'U'), &rooms);
        v.feed(Key::Enter, &rooms);
        assert_eq!(
            v.feed(Key::Hotkey(b'N'), &rooms),
            Choice::Read {
                room: 8,
                new_only: true
            }
        );
        rooms.remove(0);
        assert_eq!(v.feed(Key::Hotkey(b'P'), &rooms), Choice::Post(8));
        rooms.retain(|r| r.id != 8);
        assert_eq!(v.feed(Key::Hotkey(b'P'), &rooms), Choice::Stay);
        assert!(!v.is_card());
        assert_eq!(v.feed(Key::Enter, &[]), Choice::Stay);
        assert!(
            v.render(&[], 40, false, false, &Theme::default())
                .contains("No rooms")
        );
    }
}
