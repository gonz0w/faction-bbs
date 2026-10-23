//! Presentation preferences and observations never grant member/host authority.
use std::time::{Duration, Instant};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Color {
    #[default]
    Ansi16,
    Indexed256,
    Rgb,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Capabilities {
    pub unicode: bool,
    pub color: Color,
    pub blocks: bool,
    pub braille: bool,
    pub synchronized: bool,
}
pub const PROMPT: &str =
    "Terminal: [C]lassic [U]TF8 [1/2/3]color [B]locks [A]braille [S]ync [P]robe [D]one: ";
impl Capabilities {
    pub fn choose(&mut self, command: &str) -> bool {
        match command.to_ascii_lowercase().as_str() {
            "c" => *self = Self::default(),
            "u" => self.unicode = !self.unicode,
            "1" => self.color = Color::Ansi16,
            "2" => self.color = Color::Indexed256,
            "3" => self.color = Color::Rgb,
            "b" => self.blocks = !self.blocks,
            "a" => self.braille = !self.braille,
            "s" => self.synchronized = !self.synchronized,
            _ => return false,
        };
        true
    }
    pub fn screen(self, columns: u16, rows: u16, ansi: bool) -> String {
        let mut out = format!(
            "Terminal preferences (session only)\nGeometry: {columns}x{rows}; ANSI: {ansi}\nUTF-8: {}; cells: Unicode-width 0.2.2, ambiguous=1\nColor: {:?}; blocks: {}; braille: {}; sync: {}\nMouse/extended keys/bitmap: disabled\nC restores classic text. Choose UTF8 only if your font agrees.\n",
            self.unicode, self.color, self.blocks, self.braille, self.synchronized
        );
        if self.unicode && self.blocks {
            out.push_str("Blocks: ▁▂▃▄▅▆▇█\n");
        }
        if self.unicode && self.braille {
            out.push_str("Braille: ⡀⡄⡆⡇⣇⣧⣷⣿\n");
        }
        if ansi {
            match self.color {
                Color::Ansi16 => out.push_str("\x1b[35mFACTION frequency\x1b[0m\n"),
                Color::Indexed256 => out.push_str("\x1b[38;5;135mFACTION frequency\x1b[0m\n"),
                Color::Rgb => out.push_str("\x1b[38;2;190;80;255mFACTION frequency\x1b[0m\n"),
            }
        }
        out.push_str(PROMPT);
        out
    }
    pub fn section(self, theme: &crate::Theme, title: &str, ansi: bool) -> String {
        let out = crate::section(theme, title, ansi);
        if !ansi || self.color == Color::Ansi16 {
            return out;
        }
        let style = theme.menu_styles().0;
        let sgr = match self.color {
            Color::Indexed256 => format!("\x1b[0;38;5;{};48;5;{}m", style.fg, style.bg),
            Color::Rgb => {
                const VGA: [u32; 16] = [
                    0x000000, 0xaa0000, 0x00aa00, 0xaa5500, 0x0000aa, 0xaa00aa, 0x00aaaa, 0xaaaaaa,
                    0x555555, 0xff5555, 0x55ff55, 0xffff55, 0x5555ff, 0xff55ff, 0x55ffff, 0xffffff,
                ];
                let (f, b) = (VGA[usize::from(style.fg)], VGA[usize::from(style.bg)]);
                format!(
                    "\x1b[0;38;2;{};{};{};48;2;{};{};{}m",
                    f >> 16,
                    (f >> 8) & 255,
                    f & 255,
                    b >> 16,
                    (b >> 8) & 255,
                    b & 255
                )
            }
            Color::Ansi16 => unreachable!(),
        };
        out.replacen(
            &style.sgr(),
            &format!("{sgr}{}", if style.bold { "\x1b[1m" } else { "" }),
            1,
        )
    }
    pub fn frame(self, text: &str, ansi: bool) -> String {
        if ansi && self.synchronized {
            format!("\x1b[?2026h{text}\x1b[?2026l")
        } else {
            text.into()
        }
    }
}
pub const SYNC_QUERY: &str = "\x1b[?2026$p";
pub const PROBE_TIME: Duration = Duration::from_millis(300);
/// One exact DECRQM reply, 32 bytes, 300 ms after query output drains.
/// Extra bytes, malformed or incomplete replies fail; none are returned as keys.
#[derive(Default)]
pub struct Probe {
    bytes: Vec<u8>,
    started: Option<Instant>,
}
impl Probe {
    pub fn begin(&mut self, now: Instant) {
        if self.started.is_none() {
            self.started = Some(now)
        }
    }
    pub fn expired(&self, now: Instant) -> bool {
        self.started
            .is_some_and(|t| now.saturating_duration_since(t) >= PROBE_TIME)
    }
    pub fn feed(&mut self, b: u8) -> crate::Result<()> {
        if self.bytes.len() >= 32 {
            return Err("terminal probe overflow");
        }
        self.bytes.push(b);
        let prefix = b"\x1b[?2026;";
        let i = self.bytes.len() - 1;
        if (i < prefix.len() && b != prefix[i])
            || (i == prefix.len() && !(b'0'..=b'4').contains(&b))
            || (i == prefix.len() + 1 && b != b'$')
            || (i == prefix.len() + 2 && b != b'y')
            || i > prefix.len() + 2
        {
            return Err("unexpected terminal probe input");
        }
        Ok(())
    }
    pub fn result(&self) -> crate::Result<bool> {
        if self.bytes.is_empty() {
            return Ok(false);
        }
        if self.bytes.len() != 11 {
            return Err("incomplete terminal probe");
        }
        Ok(matches!(self.bytes[8], b'1' | b'2'))
    }
}
