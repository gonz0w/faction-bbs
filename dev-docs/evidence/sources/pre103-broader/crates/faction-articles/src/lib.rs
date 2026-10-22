//! Bounded Netnews text articles, independent of storage, transport and authority.
//! This intentionally narrow profile rejects unknown headers and unsupported MIME.
#![forbid(unsafe_code)]
use std::{collections::BTreeMap, fmt};

pub const WIRE_BYTES: usize = 24 * 1024;
pub const HEADER_BYTES: usize = 4096;
pub const BODY_BYTES: usize = 16 * 1024;
pub const LINE_BYTES: usize = 998;
pub const REFERENCES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Limit,
    Syntax,
    Missing,
    Duplicate,
    Unsupported,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Limit => "article exceeds the text profile limits",
            Self::Syntax => "invalid article syntax",
            Self::Missing => "required article field missing",
            Self::Duplicate => "duplicate article field or reference",
            Self::Unsupported => "unsupported article header or MIME profile",
        })
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

/// Inputs are explicit: this codec does not mint identities, timestamps or paths.
/// From is a bare ASCII mailbox; Subject is printable ASCII. UTF-8 is supported
/// in text bodies, not raw internationalized headers or encoded-word subjects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Headers {
    pub message_id: String,
    pub from: String,
    pub date: String,
    pub subject: String,
    pub newsgroups: Vec<String>,
    pub path: String,
    pub references: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Article {
    headers: Headers,
    body: String,
}
fn atom(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b))
        && !value.starts_with('.')
        && !value.ends_with('.')
        && !value.contains("..")
}
fn domain(value: &str) -> bool {
    value.len() <= 128
        && value.split('.').all(|part| {
            !part.is_empty()
                && part.len() <= 63
                && !part.starts_with('-')
                && !part.ends_with('-')
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}
fn mailbox(value: &str) -> bool {
    value
        .split_once('@')
        .is_some_and(|(local, host)| atom(local, 64) && domain(host))
}
fn message_id(value: &str) -> bool {
    value.len() <= 192
        && value
            .strip_prefix('<')
            .and_then(|s| s.strip_suffix('>'))
            .is_some_and(mailbox)
}
fn date(value: &str) -> bool {
    // Supported output/input profile: DD Mon YYYY HH:MM:SS +HHMM (no weekday).
    let b = value.as_bytes();
    if b.len() != 26
        || b[2] != b' '
        || b[6] != b' '
        || b[11] != b' '
        || b[14] != b':'
        || b[17] != b':'
        || b[20] != b' '
        || !matches!(b[21], b'+' | b'-')
    {
        return false;
    }
    let number = |start: usize, end: usize| -> Option<u32> {
        b[start..end]
            .iter()
            .all(u8::is_ascii_digit)
            .then(|| value[start..end].parse().ok())
            .flatten()
    };
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let Some(month) = months.iter().position(|m| m.as_bytes() == &b[3..6]) else {
        return false;
    };
    let Some(year) = number(7, 11).filter(|y| *y >= 1900) else {
        return false;
    };
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    number(0, 2).is_some_and(|d| d > 0 && d <= days[month])
        && number(12, 14).is_some_and(|n| n < 24)
        && number(15, 17).is_some_and(|n| n < 60)
        && number(18, 20).is_some_and(|n| n < 60)
        && number(22, 24).is_some_and(|n| n < 24)
        && number(24, 26).is_some_and(|n| n < 60)
}
impl Headers {
    fn validate(&self) -> Result<()> {
        if !message_id(&self.message_id)
            || !mailbox(&self.from)
            || !date(&self.date)
            || self.subject.is_empty()
            || self.subject.len() > 160
            || self.subject.trim() != self.subject
            || self.subject.contains("=?")
            || !self.subject.bytes().all(|b| (32..=126).contains(&b))
            || self.newsgroups.is_empty()
            || self.newsgroups.len() > 4
            || self.path.len() > 256
            || self.path.split('!').count() > 8
            || !self.path.split('!').all(domain)
        {
            return Err(Error::Syntax);
        }
        for (i, g) in self.newsgroups.iter().enumerate() {
            if g.len() > 96
                || !g.split('.').all(|part| {
                    !part.is_empty()
                        && part.bytes().all(|b| {
                            b.is_ascii_lowercase() || b.is_ascii_digit() || b"+-_".contains(&b)
                        })
                })
            {
                return Err(Error::Syntax);
            }
            if self.newsgroups[..i].contains(g) {
                return Err(Error::Duplicate);
            }
        }
        if self.references.len() > REFERENCES {
            return Err(Error::Limit);
        }
        for (i, id) in self.references.iter().enumerate() {
            if !message_id(id) || id == &self.message_id {
                return Err(Error::Syntax);
            }
            if self.references[..i].contains(id) {
                return Err(Error::Duplicate);
            }
        }
        Ok(())
    }
}
/// Strict comparison of validated supported-profile articles. A conflicting ID
/// is quarantined/rejected by future adapters, never silently overwritten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityMatch {
    Different,
    Duplicate,
    Conflict,
}
impl Article {
    pub fn compare_identity(&self, other: &Self) -> IdentityMatch {
        if self.headers.message_id != other.headers.message_id {
            IdentityMatch::Different
        } else if self == other {
            IdentityMatch::Duplicate
        } else {
            IdentityMatch::Conflict
        }
    }

    pub fn new(headers: Headers, body: String) -> Result<Self> {
        headers.validate()?;
        if body.len() > BODY_BYTES {
            return Err(Error::Limit);
        }
        if body
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
        {
            return Err(Error::Syntax);
        }
        if body.split('\n').any(|line| line.len() > LINE_BYTES) {
            return Err(Error::Limit);
        }
        let article = Self { headers, body };
        article.to_wire()?; // Also bound the complete folded header/wire encoding.
        Ok(article)
    }
    pub fn headers(&self) -> &Headers {
        &self.headers
    }
    /// LF-normalized body. No terminal rendering, HTML execution or attachment IO.
    pub fn body(&self) -> &str {
        &self.body
    }
    pub fn to_wire(&self) -> Result<String> {
        let h = &self.headers;
        let mut wire = format!(
            "Date: {}\r\nFrom: {}\r\nMessage-ID: {}\r\nNewsgroups: {}\r\nPath: {}\r\nSubject: {}\r\n",
            h.date,
            h.from,
            h.message_id,
            h.newsgroups.join(","),
            h.path,
            h.subject
        );
        if !h.references.is_empty() {
            wire.push_str("References: ");
            // One bounded ID per physical continuation line; canonical folding.
            for (i, id) in h.references.iter().enumerate() {
                if i > 0 {
                    wire.push_str("\r\n ");
                }
                wire.push_str(id);
            }
            wire.push_str("\r\n");
        }
        wire.push_str("MIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: 8bit\r\n");
        if wire.len() > HEADER_BYTES {
            return Err(Error::Limit);
        }
        wire.push_str("\r\n");
        for c in self.body.chars() {
            if c == '\n' {
                wire.push('\r');
            }
            wire.push(c);
        }
        // One wire terminator is removed on parse; preserve body trailing LF.
        wire.push_str("\r\n");
        if wire.len() > WIRE_BYTES {
            return Err(Error::Limit);
        }
        Ok(wire)
    }
    pub fn parse(wire: &[u8]) -> Result<Self> {
        if wire.len() > WIRE_BYTES {
            return Err(Error::Limit);
        }
        let wire = std::str::from_utf8(wire).map_err(|_| Error::Syntax)?;
        let (head, body) = wire.split_once("\r\n\r\n").ok_or(Error::Missing)?;
        if head.len() + 2 > HEADER_BYTES {
            return Err(Error::Limit);
        }
        // Reject bare LF/CR everywhere before unfolding or normalizing content.
        for line in wire.split("\r\n") {
            if line.contains(['\r', '\n']) {
                return Err(Error::Syntax);
            }
            if line.len() > LINE_BYTES {
                return Err(Error::Limit);
            }
        }
        if !body.is_empty() && !body.ends_with("\r\n") {
            return Err(Error::Syntax);
        }
        let mut fields = BTreeMap::<String, String>::new();
        let mut previous = String::new();
        for line in head.split("\r\n") {
            if !line.bytes().all(|b| b == b'\t' || (32..=126).contains(&b)) {
                return Err(Error::Syntax);
            }
            if line.starts_with([' ', '\t']) {
                let value = fields.get_mut(&previous).ok_or(Error::Syntax)?;
                value.push(' ');
                value.push_str(line.trim());
                continue;
            }
            let (name, value) = line.split_once(':').ok_or(Error::Syntax)?;
            if !value.starts_with(' ') {
                return Err(Error::Syntax);
            }
            let name = name.to_ascii_lowercase();
            if !matches!(
                name.as_str(),
                "date"
                    | "from"
                    | "message-id"
                    | "newsgroups"
                    | "path"
                    | "subject"
                    | "references"
                    | "mime-version"
                    | "content-type"
                    | "content-transfer-encoding"
            ) {
                return Err(Error::Unsupported);
            }
            if fields.insert(name.clone(), value.trim().into()).is_some() {
                return Err(Error::Duplicate);
            }
            previous = name;
        }
        for (name, value) in [
            ("mime-version", "1.0"),
            ("content-type", "text/plain; charset=utf-8"),
            ("content-transfer-encoding", "8bit"),
        ] {
            if !fields
                .remove(name)
                .ok_or(Error::Missing)?
                .eq_ignore_ascii_case(value)
            {
                return Err(Error::Unsupported);
            }
        }
        if fields
            .get("references")
            .is_some_and(|s| s.trim().is_empty())
        {
            return Err(Error::Syntax);
        }
        let mut get = |name| fields.remove(name).ok_or(Error::Missing);
        let headers = Headers {
            message_id: get("message-id")?,
            from: get("from")?,
            date: get("date")?,
            subject: get("subject")?,
            newsgroups: get("newsgroups")?.split(',').map(str::to_owned).collect(),
            path: get("path")?,
            references: fields
                .remove("references")
                .unwrap_or_default()
                .split_ascii_whitespace()
                .map(str::to_owned)
                .collect(),
        };
        let body = body
            .strip_suffix("\r\n")
            .unwrap_or(body)
            .replace("\r\n", "\n");
        Self::new(headers, body)
    }
}
