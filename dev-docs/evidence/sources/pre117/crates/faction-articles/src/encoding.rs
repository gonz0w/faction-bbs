//! Bounded MIME text encodings. No resource fetching or multipart processing.
use super::{BODY_BYTES, Error, Result};
use base64::{Engine, engine::general_purpose::STANDARD};

pub(crate) fn qp_append(out: &mut String, body: &str) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut column = 0;
    for byte in body.bytes() {
        if byte == b'\n' {
            out.push_str("\r\n");
            column = 0;
            continue;
        }
        let literal = (33..=126).contains(&byte) && byte != b'=';
        let length = if literal { 1 } else { 3 };
        if column + length > 75 {
            out.push_str("=\r\n");
            column = 0;
        }
        if literal {
            out.push(byte as char);
        } else {
            out.push('=');
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 15) as usize] as char);
        }
        column += length;
    }
    // Soft termination preserves a final byte without inventing decoded LF.
    if column > 0 {
        out.push_str("=\r\n");
    }
}

fn hex(b: u8) -> Result<u8> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(Error::Syntax),
    }
}
fn unquote(bytes: &[u8], header: bool) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'=' if !header && bytes.get(i + 1..i + 3) == Some(b"\r\n") => {
                i += 3;
                continue;
            }
            b'=' => {
                let pair = bytes.get(i + 1..i + 3).ok_or(Error::Syntax)?;
                out.push(hex(pair[0])? * 16 + hex(pair[1])?);
                i += 3;
            }
            b'_' if header => {
                out.push(b' ');
                i += 1;
            }
            byte if byte.is_ascii()
                && (byte >= 32 || (!header && matches!(byte, b'\r' | b'\n' | b'\t'))) =>
            {
                out.push(byte);
                i += 1;
            }
            _ => return Err(Error::Syntax),
        }
        if out.len() > BODY_BYTES * 2 {
            return Err(Error::Limit);
        }
    }
    Ok(out)
}
pub(crate) fn decode_body(body: &str, transfer: &str, ascii: bool) -> Result<String> {
    let bytes = match transfer.to_ascii_lowercase().as_str() {
        "7bit" => {
            if !body.is_ascii() {
                return Err(Error::Syntax);
            }
            body.as_bytes().to_vec()
        }
        "8bit" => body.as_bytes().to_vec(),
        "quoted-printable" => {
            for line in body.split("\r\n") {
                if line.len() > 76 || line.ends_with([' ', '\t']) {
                    return Err(Error::Syntax);
                }
            }
            unquote(body.as_bytes(), false)?
        }
        "base64" => {
            if !body
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"+/= \t\r\n".contains(&b))
            {
                return Err(Error::Syntax);
            }
            let compact: String = body.chars().filter(|c| !c.is_ascii_whitespace()).collect();
            STANDARD.decode(compact).map_err(|_| Error::Syntax)?
        }
        _ => return Err(Error::Unsupported),
    };
    if bytes.len() > BODY_BYTES * 2 {
        return Err(Error::Limit);
    }
    if ascii && !bytes.is_ascii() {
        return Err(Error::Syntax);
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| Error::Syntax)?
        .replace("\r\n", "\n");
    if text.len() > BODY_BYTES {
        return Err(Error::Limit);
    }
    Ok(text)
}
pub(crate) fn content_type(value: &str) -> Result<bool> {
    let mut parts = value.split(';').map(str::trim);
    if !parts
        .next()
        .is_some_and(|s| s.eq_ignore_ascii_case("text/plain"))
    {
        return Err(Error::Unsupported);
    }
    let Some(parameter) = parts.next() else {
        return Ok(true);
    };
    if parts.next().is_some() {
        return Err(Error::Unsupported);
    }
    let (name, value) = parameter.split_once('=').ok_or(Error::Syntax)?;
    if !name.trim().eq_ignore_ascii_case("charset") {
        return Err(Error::Unsupported);
    }
    let value = value.trim();
    let value = if value.starts_with('"') {
        value
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .ok_or(Error::Syntax)?
    } else {
        value
    };
    if value.eq_ignore_ascii_case("utf-8") {
        Ok(false)
    } else if value.eq_ignore_ascii_case("us-ascii") {
        Ok(true)
    } else {
        Err(Error::Unsupported)
    }
}
pub(crate) fn subject_wire(value: &str) -> String {
    if value.is_ascii() && !value.contains("=?") {
        return value.into();
    }
    let mut words = Vec::new();
    let mut chunk = String::new();
    for c in value.chars() {
        if chunk.len() + c.len_utf8() > 42 {
            words.push(format!("=?UTF-8?B?{}?=", STANDARD.encode(chunk.as_bytes())));
            chunk.clear();
        }
        chunk.push(c);
    }
    if !chunk.is_empty() {
        words.push(format!("=?UTF-8?B?{}?=", STANDARD.encode(chunk.as_bytes())));
    }
    words.join("\r\n ")
}
pub(crate) fn subject_parse(value: &str) -> Result<String> {
    let mut out = String::new();
    let mut i = 0;
    while i < value.len() {
        if value[i..].starts_with("=?") {
            if i > 0 && !value.as_bytes()[i - 1].is_ascii_whitespace() {
                return Err(Error::Syntax);
            }
            let end = value[i + 2..].find("?=").ok_or(Error::Syntax)? + i + 4;
            if end - i > 75 || (end < value.len() && !value.as_bytes()[end].is_ascii_whitespace()) {
                return Err(Error::Syntax);
            }
            let mut parts = value[i + 2..end - 2].split('?');
            let charset = parts.next().ok_or(Error::Syntax)?;
            let kind = parts.next().ok_or(Error::Syntax)?;
            let text = parts.next().ok_or(Error::Syntax)?;
            if parts.next().is_some() || text.is_empty() {
                return Err(Error::Syntax);
            }
            let ascii = charset.eq_ignore_ascii_case("us-ascii");
            if !ascii && !charset.eq_ignore_ascii_case("utf-8") {
                return Err(Error::Unsupported);
            }
            let decoded = if kind.eq_ignore_ascii_case("b") {
                STANDARD.decode(text).map_err(|_| Error::Syntax)?
            } else if kind.eq_ignore_ascii_case("q") {
                unquote(text.as_bytes(), true)?
            } else {
                return Err(Error::Unsupported);
            };
            if ascii && !decoded.is_ascii() {
                return Err(Error::Syntax);
            }
            out.push_str(std::str::from_utf8(&decoded).map_err(|_| Error::Syntax)?);
            i = end;
            let after = value[i..].len() - value[i..].trim_start_matches([' ', '\t']).len() + i;
            if value[after..].starts_with("=?") {
                i = after;
            }
        } else {
            out.push(value.as_bytes()[i] as char);
            i += 1;
        }
        if out.len() > 160 {
            return Err(Error::Limit);
        }
    }
    Ok(out)
}
