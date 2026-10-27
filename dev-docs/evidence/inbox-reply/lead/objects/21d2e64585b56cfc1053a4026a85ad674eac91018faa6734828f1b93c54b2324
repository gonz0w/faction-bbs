//! Device-local preferences. No routes, network I/O, disk history or remote setters.
use crate::Terminal;
use std::io;
pub const MAX_FILE: u64 = 16 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Travel {
    pub compact: bool,
    pub file_budget: u64,
}
impl Default for Travel {
    fn default() -> Self {
        Self {
            compact: false,
            file_budget: MAX_FILE,
        }
    }
}
fn decimal(s: &str) -> io::Result<u64> {
    if s.is_empty() || s.len() > 8 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(io::Error::other("use decimal bytes from 0 to 16777216"));
    }
    s.parse::<u64>()
        .ok()
        .filter(|n| *n <= MAX_FILE)
        .ok_or_else(|| io::Error::other("use decimal bytes from 0 to 16777216"))
}
impl Travel {
    pub fn budget(&mut self, s: &str) -> io::Result<()> {
        self.file_budget = decimal(s)?;
        Ok(())
    }
    pub fn check_file(&self, s: &str) -> io::Result<u64> {
        let bytes = decimal(s)?;
        if self.file_budget == 0 || bytes > self.file_budget {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "File deferred: {bytes} payload bytes; local ceiling {}. Change travel settings before a later visit.",
                    self.file_budget
                ),
            ));
        }
        Ok(bytes)
    }
    pub fn summary(&self) -> String {
        format!(
            "{}; per-file ceiling {} bytes",
            if self.compact {
                "Compact text"
            } else {
                "Ordinary terminal"
            },
            self.file_budget
        )
    }
    pub fn panel(&mut self, t: &mut Terminal) -> io::Result<()> {
        let mut draft = *self;
        loop {
            t.print("\x1b[2J\x1b[HLOCAL TRAVEL / Before connecting\r\n")?;
            t.print(if draft.compact {
                "Mode: compact; ANSI/bitmap art off.\r\n"
            } else {
                "Mode: ordinary terminal.\r\n"
            })?;
            t.print(&format!(
                "Per-file ceiling: {} bytes\r\n",
                draft.file_budget
            ))?;
            t.print("0 defers every file transfer.\r\nPayload only; SSH/retries cost extra.\r\nApplies to all visits this run.\r\nNo route or account changes.\r\nOffline exchange is not available.\r\n\r\nO:ordinary C:compact B:byte ceiling\r\nS:apply Q:cancel\r\n")?;
            match t.line("Travel> ", 1)?.to_ascii_lowercase().as_str() {
                "o" => draft.compact = false,
                "c" => draft.compact = true,
                "b" => {
                    let value = t.line("Payload bytes [0-16777216]: ", 16)?;
                    if draft.budget(&value).is_err() {
                        t.print("Invalid ceiling; settings unchanged.\r\n")?;
                        t.line("Enter to continue: ", 0)?;
                    }
                }
                "s" => {
                    *self = draft;
                    return Ok(());
                }
                "q" => return Ok(()),
                _ => (),
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn byte_limits_are_explicit_strict_and_atomic() {
        let mut t = Travel::default();
        assert_eq!(t.file_budget, MAX_FILE);
        assert!(!t.compact);
        for s in [
            "",
            "-1",
            "+1",
            "1.5",
            "1e6",
            " 12",
            "12\n",
            "16777217",
            "99999999999999999999",
            "\x1b[31m",
        ] {
            assert!(t.budget(s).is_err());
            assert_eq!(t.file_budget, MAX_FILE);
        }
        t.budget("1024").unwrap();
        assert_eq!(t.check_file("1024").unwrap(), 1024);
        assert!(t.check_file("1025").is_err());
        t.budget("0").unwrap();
        assert!(t.check_file("0").is_err());
        assert!(t.check_file("1").is_err());
    }
    #[test]
    fn payload_validation_never_treats_estimates_or_invalid_metadata_as_authority() {
        let t = Travel::default();
        assert_eq!(t.check_file("0").unwrap(), 0);
        assert_eq!(t.check_file("16777216").unwrap(), MAX_FILE);
        for s in ["unknown", "18446744073709551616", "16777217", "-0", "0x10"] {
            assert!(t.check_file(s).is_err());
        }
        let mut draft = t;
        draft.compact = true;
        draft.budget("123").unwrap();
        assert!(!t.compact);
        assert_eq!(t.file_budget, MAX_FILE);
        assert!(draft.summary().contains("Compact text"));
    }
}
