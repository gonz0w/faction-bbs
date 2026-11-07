//! Foreground member-device draft only. No room draft, storage or network authority.
use zeroize::{Zeroize, Zeroizing};
pub const BYTES: usize = 1024;
pub const LINES: usize = 32;
#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    Added,
    Changed,
    Preview,
    Done,
    Cancel,
    Help,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Command,
    Invalid,
    Bounds,
    Empty,
}
pub struct Writer {
    bytes: Zeroizing<Vec<u8>>,
    starts: [usize; LINES],
    lines: usize,
}
impl Writer {
    pub fn new() -> Self {
        Self {
            bytes: Zeroizing::new(Vec::with_capacity(BYTES)),
            starts: [0; LINES],
            lines: 0,
        }
    }
    pub fn body(&self) -> &str {
        std::str::from_utf8(&self.bytes).expect("validated writer UTF-8")
    }
    pub fn lines(&self) -> usize {
        self.lines
    }
    fn remove_to(&mut self, len: usize) {
        // Wipe removed initialized storage BEFORE truncate/clear makes it inaccessible.
        self.bytes[len..].zeroize();
        self.bytes.truncate(len);
    }
    pub fn line(&mut self, line: &str) -> Result<Effect, Error> {
        if line.starts_with('.') && !line.starts_with("..") {
            return match line {
                ".done" if self.body().trim().is_empty() => Err(Error::Empty),
                ".done" => Ok(Effect::Done),
                ".preview" => Ok(Effect::Preview),
                ".help" => Ok(Effect::Help),
                ".clear" | ".cancel" => {
                    self.remove_to(0);
                    self.lines = 0;
                    Ok(if line == ".cancel" {
                        Effect::Cancel
                    } else {
                        Effect::Changed
                    })
                }
                ".undo" => {
                    if self.lines > 0 {
                        self.lines -= 1;
                        self.remove_to(self.starts[self.lines]);
                    }
                    Ok(Effect::Changed)
                }
                _ => Err(Error::Command),
            };
        }
        let line = if line.starts_with("..") {
            &line[1..]
        } else {
            line
        };
        if line.chars().any(char::is_control) {
            return Err(Error::Invalid);
        }
        let separator = usize::from(self.lines > 0);
        if self.lines == LINES || self.bytes.len() + separator + line.len() > BYTES {
            return Err(Error::Bounds);
        }
        self.starts[self.lines] = self.bytes.len();
        if separator != 0 {
            self.bytes.push(b'\n');
        }
        self.bytes.extend_from_slice(line.as_bytes());
        self.lines += 1;
        Ok(Effect::Added)
    }
}
