//! Bounded XEP-0420 SCE subset for local private-mail checkpoints.
//!
//! Accepts only whole-second UTC XEP-0082 timestamps (`YYYY-MM-DDTHH:MM:SSZ`),
//! years 1970..=9999. Offsets, fractions and leap seconds require a separate review.
//! Timestamp agreement is with the enclosing sending time, so delayed mail works.
use crate::{Error, MAX_PLAINTEXT, Result};
use roxmltree::{Document, Node, ParsingOptions};
use zeroize::Zeroizing;

const SCE: &str = "urn:xmpp:sce:1";
const CLIENT: &str = "jabber:client";
const LAST_SECOND: u64 = 253_402_300_799;
const NODE_LIMIT: u32 = 128;

/// Private text. Callers must also treat the parser input as private.
/// The XML library may allocate temporary normalized strings without zeroization.
pub struct Content {
    pub subject: Zeroizing<String>,
    pub body: Zeroizing<String>,
}

fn xml_text(text: &str) -> bool {
    text.chars().all(|c| matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}'))
}
fn bounded(text: &str, limit: usize) -> Result<()> {
    if text.is_empty() || text.len() > limit || !xml_text(text) {
        Err(Error)
    } else {
        Ok(())
    }
}
fn whitespace(text: &str) -> bool {
    text.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n'))
}
fn escape(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\'' => out.push_str("&apos;"),
            '"' => out.push_str("&quot;"),
            '\r' => out.push_str("&#13;"),
            '\n' => out.push_str("&#10;"),
            '\t' => out.push_str("&#9;"),
            _ => out.push(c),
        }
    }
}
fn leap(year: u64) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}
fn month_days(year: u64, month: u64) -> u64 {
    match month {
        2 => {
            if leap(year) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
fn parse_stamp(stamp: &str) -> Result<u64> {
    let b = stamp.as_bytes();
    if b.len() != 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
    {
        return Err(Error);
    }
    fn digits(bytes: &[u8]) -> Result<u64> {
        bytes.iter().try_fold(0, |n, b| {
            if b.is_ascii_digit() {
                Ok(n * 10 + u64::from(b - b'0'))
            } else {
                Err(Error)
            }
        })
    }
    let year = digits(&b[..4])?;
    let month = digits(&b[5..7])?;
    let day = digits(&b[8..10])?;
    let hour = digits(&b[11..13])?;
    let minute = digits(&b[14..16])?;
    let second = digits(&b[17..19])?;
    if !(1970..=9999).contains(&year)
        || !(1..=12).contains(&month)
        || !(1..=month_days(year, month)).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err(Error);
    }
    let days = (1970..year)
        .map(|y| if leap(y) { 366 } else { 365 })
        .sum::<u64>()
        + (1..month).map(|m| month_days(year, m)).sum::<u64>()
        + day
        - 1;
    Ok(days * 86400 + hour * 3600 + minute * 60 + second)
}
fn format_stamp(epoch: u64) -> Result<String> {
    if epoch > LAST_SECOND {
        return Err(Error);
    }
    let mut days = epoch / 86400;
    let mut year = 1970;
    let mut month = 1;
    loop {
        let n = if leap(year) { 366 } else { 365 };
        if days < n {
            break;
        }
        days -= n;
        year += 1;
    }
    while days >= month_days(year, month) {
        days -= month_days(year, month);
        month += 1;
    }
    Ok(format!(
        "{year:04}-{month:02}-{:02}T{:02}:{:02}:{:02}Z",
        days + 1,
        epoch % 86400 / 3600,
        epoch % 3600 / 60,
        epoch % 60
    ))
}

/// Encode nonempty UTF-8 subject (<=128 bytes) and body (<=1024 bytes),
/// escaped as inert XML text, exact address bindings and random-content padding.
pub fn encode(
    sender: &str,
    recipient: &str,
    sent_at: u64,
    subject: &str,
    body: &str,
) -> Result<Zeroizing<Vec<u8>>> {
    bounded(sender, MAX_PLAINTEXT)?;
    bounded(recipient, MAX_PLAINTEXT)?;
    bounded(subject, 128)?;
    bounded(body, 1024)?;
    let stamp = format_stamp(sent_at)?;
    let mut out = Zeroizing::new(String::with_capacity(MAX_PLAINTEXT));
    out.push_str("<envelope xmlns='urn:xmpp:sce:1'><content><subject xmlns='jabber:client'>");
    escape(&mut out, subject);
    out.push_str("</subject><body xmlns='jabber:client'>");
    escape(&mut out, body);
    out.push_str("</body></content><from jid='");
    escape(&mut out, sender);
    out.push_str("'/><to jid='");
    escape(&mut out, recipient);
    out.push_str("'/><time stamp='");
    out.push_str(&stamp);
    out.push_str("'/><rpad>");
    // Hex is an inert alphabet with random content, not predictable filler.
    let mut random = Zeroizing::new([0u8; 128]);
    getrandom::fill(&mut *random).map_err(|_| Error)?;
    const HEX: &[u8] = b"0123456789abcdef";
    for b in random.iter() {
        out.push(HEX[usize::from(b >> 4)] as char);
        out.push(HEX[usize::from(b & 15)] as char);
    }
    out.push_str("</rpad></envelope>");
    if out.len() > MAX_PLAINTEXT {
        return Err(Error);
    }
    // Move the owned String allocation into a zeroizing byte allocation.
    Ok(Zeroizing::new(std::mem::take(&mut *out).into_bytes()))
}
fn named(node: Node<'_, '_>, ns: &str, name: &str) -> bool {
    node.is_element() && node.tag_name().namespace() == Some(ns) && node.tag_name().name() == name
}
fn elements<'a, 'input>(node: Node<'a, 'input>) -> Result<Vec<Node<'a, 'input>>> {
    let mut children = Vec::new();
    for child in node.children() {
        if child.is_element() {
            children.push(child);
        } else if !child.is_text() || !whitespace(child.text().ok_or(Error)?) {
            return Err(Error);
        }
    }
    Ok(children)
}
fn only_attribute<'a>(node: Node<'a, '_>, key: &str) -> Result<&'a str> {
    let mut attrs = node.attributes();
    let attr = attrs.next().ok_or(Error)?;
    if attrs.next().is_some() || attr.namespace().is_some() || attr.name() != key {
        return Err(Error);
    }
    Ok(attr.value())
}
fn text_leaf(node: Node<'_, '_>, limit: usize) -> Result<Zeroizing<String>> {
    if node.attributes().len() != 0 {
        return Err(Error);
    }
    let mut text = Zeroizing::new(String::new());
    for child in node.children() {
        if !child.is_text() {
            return Err(Error);
        }
        text.push_str(child.text().ok_or(Error)?);
        if text.len() > limit {
            return Err(Error);
        }
    }
    bounded(&text, limit)?;
    Ok(text)
}

/// Authenticate the exact SCE subset after native decryption and before commit.
/// Caller-supplied addresses and enclosing sending time are trusted outer metadata.
pub fn decode(
    raw: &[u8],
    sender: &str,
    recipient: &str,
    enclosing_sent_at: u64,
) -> Result<Content> {
    if raw.len() > MAX_PLAINTEXT || enclosing_sent_at > LAST_SECOND {
        return Err(Error);
    }
    bounded(sender, MAX_PLAINTEXT)?;
    bounded(recipient, MAX_PLAINTEXT)?;
    let text = std::str::from_utf8(raw).map_err(|_| Error)?;
    // Reject declarations, CDATA, comments, DTDs and PIs before parsing. Only
    // built-in/numeric XML escapes are useful; external entities have no resolver.
    if text.contains("<!") || text.contains("<?") {
        return Err(Error);
    }
    let doc = Document::parse_with_options(
        text,
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: NODE_LIMIT,
            entity_resolver: None,
        },
    )
    .map_err(|_| Error)?;
    let roots = elements(doc.root())?;
    if roots.len() != 1 {
        return Err(Error);
    }
    let root = roots[0];
    if !named(root, SCE, "envelope") || root.attributes().len() != 0 {
        return Err(Error);
    }
    for node in doc.descendants().filter(|n| n.is_element()) {
        if node.namespaces().any(|ns| {
            !matches!(
                ns.uri(),
                SCE | CLIENT | "http://www.w3.org/XML/1998/namespace"
            )
        }) {
            return Err(Error);
        }
    }
    let children = elements(root)?;
    if children.len() != 5 {
        return Err(Error);
    }
    let mut content = None;
    let mut from = None;
    let mut to = None;
    let mut time = None;
    let mut pad = None;
    for child in children {
        let slot = if named(child, SCE, "content") {
            &mut content
        } else if named(child, SCE, "from") {
            &mut from
        } else if named(child, SCE, "to") {
            &mut to
        } else if named(child, SCE, "time") {
            &mut time
        } else if named(child, SCE, "rpad") {
            &mut pad
        } else {
            return Err(Error);
        };
        if slot.replace(child).is_some() {
            return Err(Error);
        }
    }
    for (node, expected) in [(from.ok_or(Error)?, sender), (to.ok_or(Error)?, recipient)] {
        if only_attribute(node, "jid")? != expected || !elements(node)?.is_empty() {
            return Err(Error);
        }
    }
    let time = time.ok_or(Error)?;
    if !elements(time)?.is_empty()
        || parse_stamp(only_attribute(time, "stamp")?)?.abs_diff(enclosing_sent_at) > 300
    {
        return Err(Error);
    }
    let _padding = text_leaf(pad.ok_or(Error)?, MAX_PLAINTEXT)?;
    let content = content.ok_or(Error)?;
    if content.attributes().len() != 0 {
        return Err(Error);
    }
    let fields = elements(content)?;
    if fields.len() != 2 {
        return Err(Error);
    }
    let mut subject = None;
    let mut body = None;
    for field in fields {
        if named(field, CLIENT, "subject") && subject.is_none() {
            subject = Some(text_leaf(field, 128)?);
        } else if named(field, CLIENT, "body") && body.is_none() {
            body = Some(text_leaf(field, 1024)?);
        } else {
            return Err(Error);
        }
    }
    Ok(Content {
        subject: subject.ok_or(Error)?,
        body: body.ok_or(Error)?,
    })
}
