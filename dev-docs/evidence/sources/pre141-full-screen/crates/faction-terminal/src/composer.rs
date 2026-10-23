//! A bounded local-room draft, never a private-mail store or authorization token.
use crate::{Message, MessageDetails, MessagePage, Result};
pub const BYTES: usize = 1024;
pub const LINES: usize = 32;
pub const PROMPT: &str = "Write> ";
pub const HELP: &str = "Enter adds a line. .preview .send .undo .clear .quote .cancel .help\nStart with two dots for a literal dot. Drafts are lost on disconnect.\n";
pub struct Target<'a> {
    pub room: i64,
    pub reply: Option<i64>,
    pub subject: &'a str,
}
#[derive(Default)]
pub struct Draft {
    body: String,
    lines: usize,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Changed,
    Preview,
    Send,
    Cancel,
    Quote,
    Help,
}
impl Draft {
    pub fn body(&self) -> &str {
        &self.body
    }
    pub fn lines(&self) -> usize {
        self.lines
    }
    fn append(&mut self, line: &str) -> Result<()> {
        if self.lines == LINES || self.body.len() + line.len() + usize::from(self.lines > 0) > BYTES
        {
            return Err("draft limit: 1024 bytes / 32 lines; undo or clear before adding");
        }
        if line.chars().any(char::is_control) {
            return Err("draft line contains controls");
        }
        if self.lines > 0 {
            self.body.push('\n');
        }
        self.body.push_str(line);
        self.lines += 1;
        Ok(())
    }
    pub fn command(&mut self, line: &str) -> Result<Command> {
        Ok(match line {
            ".preview" => Command::Preview,
            ".send" => {
                if self.body.trim().is_empty() {
                    return Err("empty draft; write before sending");
                }
                Command::Send
            }
            ".cancel" => Command::Cancel,
            ".quote" => Command::Quote,
            ".help" => Command::Help,
            ".clear" => {
                self.body.clear();
                self.lines = 0;
                Command::Changed
            }
            ".undo" => {
                if self.lines > 0 {
                    self.body.truncate(self.body.rfind('\n').unwrap_or(0));
                    self.lines -= 1;
                }
                Command::Changed
            }
            s if s.starts_with("..") => {
                self.append(&s[1..])?;
                Command::Changed
            }
            s if s.starts_with('.') => {
                return Err("unknown writing command; use .help or two dots");
            }
            s => {
                self.append(s)?;
                Command::Changed
            }
        })
    }
    /// All-or-nothing quote of a freshly authorized, rendered parent body.
    pub fn quote(&mut self, body: &str) -> Result<()> {
        if body.len() > BYTES {
            return Err("quote does not fit; write a shorter excerpt manually");
        }
        let lines = body.split('\n').count();
        let bytes = body.len() + 2 * lines + usize::from(self.lines > 0);
        if self.lines + lines > LINES || self.body.len() + bytes > BYTES {
            return Err("quote does not fit; write a shorter excerpt manually");
        }
        if body.chars().any(|c| c.is_control() && c != '\n') {
            return Err("quote contains controls");
        }
        // Check the complete quote before allocating any line copies.
        for line in body.split('\n') {
            self.append(&format!("> {line}"))?;
        }
        Ok(())
    }
    pub fn preview(
        &self,
        target: &Target<'_>,
        size: (u16, u16),
        offset: usize,
        unicode: bool,
    ) -> Result<MessagePage> {
        Message {
            room: target.room,
            id: 0,
            author: "you",
            reply_to: target.reply,
            body: &self.body,
        }
        .draft_page(
            size.0,
            size.1,
            offset,
            MessageDetails {
                subject: if target.subject.is_empty() {
                    "(generated/reply default on send)"
                } else {
                    target.subject
                },
                posted: "Not sent",
            },
            unicode,
        )
    }
}
