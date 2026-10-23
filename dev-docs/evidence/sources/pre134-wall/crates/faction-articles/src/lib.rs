//! Bounded Netnews text articles, independent of storage, transport and authority.
//! This intentionally narrow profile rejects unknown headers and unsupported MIME.
#![forbid(unsafe_code)]
mod encoding;
use std::{collections::BTreeMap, fmt};

/// Version 2 uses standard MIME body semantics (no invented trailing newline).
pub const PROFILE_VERSION: u8 = 2;

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
/// From is a bare ASCII mailbox. Subject/body are decoded UTF-8 text; the codec
/// encodes non-ASCII subjects with bounded RFC 2047 UTF-8 encoded words.
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
fn normalize_date(value: &str) -> Result<String> {
    let (weekday, value) = if let Some((day, tail)) = value.split_once(", ") {
        (Some(day), tail)
    } else {
        (None, value)
    };
    let fields: Vec<_> = value.split_ascii_whitespace().collect();
    if fields.len() != 5 {
        return Err(Error::Syntax);
    }
    let time = if fields[3].len() == 5 {
        format!("{}:00", fields[3])
    } else {
        fields[3].into()
    };
    let day = if fields[0].len() == 1 {
        format!("0{}", fields[0])
    } else {
        fields[0].into()
    };
    let normalized = format!("{day} {} {} {time} {}", fields[1], fields[2], fields[4]);
    if !date(&normalized) {
        return Err(Error::Syntax);
    }
    if let Some(weekday) = weekday {
        let month = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ]
        .iter()
        .position(|m| *m == fields[1])
        .ok_or(Error::Syntax)?
            + 1;
        let mut year = fields[2].parse::<i32>().map_err(|_| Error::Syntax)?;
        let d = day.parse::<i32>().map_err(|_| Error::Syntax)?;
        if month < 3 {
            year -= 1;
        }
        let n = (year + year / 4 - year / 100
            + year / 400
            + [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4][month - 1]
            + d)
            % 7;
        if ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][n as usize] != weekday {
            return Err(Error::Syntax);
        }
    }
    Ok(normalized)
}
impl Headers {
    fn validate(&self) -> Result<()> {
        if !message_id(&self.message_id)
            || !mailbox(&self.from)
            || !date(&self.date)
            || self.subject.is_empty()
            || self.subject.len() > 160
            || self.subject.trim() != self.subject
            || self.subject.chars().any(char::is_control)
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

    pub fn new(mut headers: Headers, body: String) -> Result<Self> {
        headers.date = normalize_date(&headers.date)?;
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
            encoding::subject_wire(&h.subject)
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
        wire.push_str("MIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: quoted-printable\r\n");
        if wire.len() > HEADER_BYTES {
            return Err(Error::Limit);
        }
        wire.push_str("\r\n");
        encoding::qp_append(&mut wire, &self.body);
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
        let has_mime = ["content-type", "content-transfer-encoding"]
            .iter()
            .any(|name| fields.contains_key(*name));
        match fields.remove("mime-version") {
            Some(value) if value == "1.0" => (),
            Some(_) => return Err(Error::Unsupported),
            None if has_mime => return Err(Error::Missing),
            None => (),
        }
        let ascii = encoding::content_type(
            fields
                .remove("content-type")
                .as_deref()
                .unwrap_or("text/plain; charset=us-ascii"),
        )?;
        let transfer = fields
            .remove("content-transfer-encoding")
            .unwrap_or_else(|| "7bit".into());
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
            subject: encoding::subject_parse(&get("subject")?)?,
            newsgroups: get("newsgroups")?.split(',').map(str::to_owned).collect(),
            path: get("path")?,
            references: fields
                .remove("references")
                .unwrap_or_default()
                .split_ascii_whitespace()
                .map(str::to_owned)
                .collect(),
        };
        let body = encoding::decode_body(body, &transfer, ascii)?;
        Self::new(headers, body)
    }
}
