use std::time::{Duration, Instant};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Next,
    Enter,
    Hotkey(u8),
    Cancel,
}
#[derive(Default)]
enum State {
    #[default]
    Ground,
    Escape,
    Csi,
    Ss3,
    String,
    Quarantine,
}
/// Unknown sequences fail closed until a line boundary. Control strings drain
/// through ST/BEL (not newline); Ctrl-C/D always permits leaving the local menu.
#[derive(Default)]
pub struct Decoder {
    state: State,
    sequence: Vec<u8>,
    started: Option<Instant>,
    skip_lf: bool,
    string_escape: bool,
    string_bel: bool,
    utf8_remaining: u8,
}
impl Decoder {
    pub(crate) fn text_ready(&self) -> bool {
        matches!(self.state, State::Ground) && self.utf8_remaining == 0
    }
    pub(crate) fn quarantined(&self) -> bool {
        matches!(self.state, State::Quarantine)
    }
    pub fn new() -> Self {
        Self {
            sequence: Vec::with_capacity(16),
            ..Self::default()
        }
    }
    pub fn expire(&mut self, now: Instant) {
        if self
            .started
            .is_some_and(|at| now.saturating_duration_since(at) >= Duration::from_millis(300))
        {
            if !matches!(self.state, State::String) {
                self.state = State::Quarantine;
            }
            self.sequence.clear();
            self.started = None;
        }
    }
    pub fn feed(&mut self, byte: u8, now: Instant) -> Option<Key> {
        self.expire(now);
        if byte == 3 || byte == 4 {
            return Some(Key::Cancel);
        }
        if self.utf8_remaining > 0 {
            self.utf8_remaining -= 1;
            if !(0x80..=0xbf).contains(&byte) {
                self.utf8_remaining = 0;
                self.state = State::Quarantine;
            }
            return None;
        }
        if matches!(self.state, State::Ground) && (0xc2..=0xf4).contains(&byte) {
            self.utf8_remaining = if byte < 0xe0 {
                1
            } else if byte < 0xf0 {
                2
            } else {
                3
            };
            return None;
        }
        if self.skip_lf {
            self.skip_lf = false;
            if byte == b'\n' {
                return None;
            }
        }
        match self.state {
            State::Ground => match byte {
                27 => {
                    self.state = State::Escape;
                    self.started = Some(now);
                    None
                }
                0x9b => {
                    self.state = State::Csi;
                    self.started = Some(now);
                    None
                }
                0x90 | 0x9d | 0x9e | 0x9f => {
                    self.state = State::String;
                    self.string_bel = byte == 0x9d;
                    None
                }
                b'\r' | b'\n' => {
                    self.skip_lf = byte == b'\r';
                    Some(Key::Enter)
                }
                b'\t' => Some(Key::Next),
                b if b.is_ascii_alphabetic() => Some(Key::Hotkey(b.to_ascii_uppercase())),
                _ => None,
            },
            State::Escape => {
                self.string_bel = byte == b']';
                self.state = match byte {
                    b'[' => State::Csi,
                    b'O' => State::Ss3,
                    b']' | b'P' | b'_' | b'^' => State::String,
                    _ => State::Quarantine,
                };
                None
            }
            State::Csi | State::Ss3 => {
                if byte.is_ascii_digit() || byte == b';' {
                    if self.sequence.len() == 16 {
                        self.state = State::Quarantine;
                        self.sequence.clear();
                    } else {
                        self.sequence.push(byte)
                    }
                    return None;
                }
                let simple = self.sequence.is_empty() || self.sequence == b"1";
                self.sequence.clear();
                self.started = None;
                let key = if simple {
                    match byte {
                        b'A' => Some(Key::Up),
                        b'B' => Some(Key::Down),
                        b'C' => Some(Key::Right),
                        b'D' => Some(Key::Left),
                        _ => None,
                    }
                } else {
                    None
                };
                self.state = if key.is_some() {
                    State::Ground
                } else {
                    State::Quarantine
                };
                key
            }
            State::String => {
                if (byte == 7 && self.string_bel)
                    || byte == 0x9c
                    || (self.string_escape && byte == b'\\')
                {
                    self.state = State::Ground;
                    self.started = None;
                    self.string_escape = false;
                } else {
                    self.string_escape = byte == 27;
                }
                None
            }
            State::Quarantine => {
                if byte == b'\r' || byte == b'\n' {
                    self.state = State::Ground;
                    self.started = None;
                    self.skip_lf = byte == b'\r';
                }
                None
            }
        }
    }
}
