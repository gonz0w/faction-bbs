use crate::{Decoder, Key};
use std::time::Instant;
pub const LINE_BYTES: usize = 768;
#[derive(Debug, PartialEq, Eq)]
pub enum LineEvent {
    Submit(String),
    Cancel,
    Discarded,
}
/// Bounded ASCII tail editor. Escape sequences never become command text.
pub struct LineEditor {
    decoder: Decoder,
    text: String,
    overflow: bool,
    reported: bool,
}
impl Default for LineEditor {
    fn default() -> Self {
        Self {
            decoder: Decoder::new(),
            text: String::with_capacity(LINE_BYTES),
            overflow: false,
            reported: false,
        }
    }
}
impl LineEditor {
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn clear(&mut self) {
        self.text.clear();
    }
    pub fn expire(&mut self, now: Instant) -> Option<LineEvent> {
        self.decoder.expire(now);
        self.quarantine()
    }
    fn quarantine(&mut self) -> Option<LineEvent> {
        if self.decoder.quarantined() {
            self.text.clear();
            if !self.reported {
                self.reported = true;
                return Some(LineEvent::Discarded);
            }
        } else {
            self.reported = false
        }
        None
    }
    pub fn feed(&mut self, byte: u8, now: Instant) -> Option<LineEvent> {
        let ready = self.decoder.text_ready();
        let key = self.decoder.feed(byte, now);
        if key == Some(Key::Cancel) {
            return Some(LineEvent::Cancel);
        }
        if self.overflow {
            if byte == b'\r' || byte == b'\n' {
                self.overflow = false;
                self.text.clear();
                return Some(LineEvent::Discarded);
            }
            return None;
        }
        if let Some(event) = self.quarantine() {
            return Some(event);
        }
        if key == Some(Key::Enter) {
            return Some(LineEvent::Submit(std::mem::replace(
                &mut self.text,
                String::with_capacity(LINE_BYTES),
            )));
        }
        if ready && (byte == 8 || byte == 127) {
            self.text.pop();
        } else if ready && (32..=126).contains(&byte) {
            if self.text.len() == LINE_BYTES {
                self.overflow = true;
                self.text.clear();
            } else {
                self.text.push(byte as char);
            }
        }
        None
    }
    /// Safe one-row viewport; the complete logical draft remains in memory.
    pub fn viewport(&self, columns: usize) -> &str {
        let start = self.text.len().saturating_sub(columns);
        &self.text[start..]
    }
}
