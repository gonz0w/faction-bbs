use crate::types::host;
use crate::*;

const FORM: &str = "application/x-www-form-urlencoded";
const JSON: &str = "application/json";

// Locate and bound the complete header section before parsing/allocating any
// structured fields. Input is a complete message, not a streaming reader.
fn boundary(bytes: &[u8]) -> Result<usize, Error> {
    let prefix = &bytes[..bytes.len().min(MAX_HEADER)];
    let end = prefix
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or(Error)?
        + 4;
    // httparse permits leading empty lines and bare LF by default. This private
    // profile requires a nonempty start line and CRLF for every header line.
    if matches!(bytes.first(), Some(b'\r' | b'\n')) {
        return Err(Error);
    }
    for (index, byte) in bytes[..end].iter().enumerate() {
        if (*byte == b'\n' && (index == 0 || bytes[index - 1] != b'\r'))
            || (*byte == b'\r' && bytes.get(index + 1) != Some(&b'\n'))
        {
            return Err(Error);
        }
    }
    if end > MAX_HEADER || bytes.len() - end > MAX_FORM {
        return Err(Error);
    }
    Ok(end)
}
fn framing(headers: &[httparse::Header<'_>], media: &str, request: bool) -> Result<usize, Error> {
    let mut seen = std::collections::BTreeSet::new();
    let mut length = None;
    let mut content = false;
    let mut hostname = false;
    for header in headers {
        let name = header.name.to_ascii_lowercase();
        if !seen.insert(name.clone()) {
            return Err(Error);
        }
        let value = std::str::from_utf8(header.value).map_err(|_| Error)?;
        if !value.bytes().all(|b| (0x20..=0x7e).contains(&b)) {
            return Err(Error);
        }
        match name.as_str() {
            "transfer-encoding" | "content-encoding" | "location" | "connection" | "upgrade"
            | "expect" => return Err(Error),
            "content-length" => {
                if value.is_empty()
                    || !value.bytes().all(|b| b.is_ascii_digit())
                    || (value.len() > 1 && value.starts_with('0'))
                {
                    return Err(Error);
                }
                let size = value.parse::<usize>().map_err(|_| Error)?;
                if size > MAX_FORM {
                    return Err(Error);
                }
                length = Some(size);
            }
            "content-type" => {
                if value != media {
                    return Err(Error);
                }
                content = true;
            }
            "host" => {
                if !request {
                    return Err(Error);
                }
                host(value)?;
                hostname = true;
            }
            "cache-control" => {
                if value != "no-store" {
                    return Err(Error);
                }
            }
            "pragma" if value != "no-cache" => return Err(Error),
            _ => {}
        }
    }
    if !content || (request && !hostname) {
        return Err(Error);
    }
    length.ok_or(Error)
}
fn request_body<'a>(bytes: &'a [u8], route: &str) -> Result<&'a [u8], Error> {
    let end = boundary(bytes)?;
    let mut storage = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut request = httparse::Request::new(&mut storage);
    let parsed = request.parse(&bytes[..end]).map_err(|_| Error)?;
    if parsed != httparse::Status::Complete(end)
        || request.method != Some("POST")
        || request.path != Some(route)
        || request.version != Some(1)
    {
        return Err(Error);
    }
    let length = framing(request.headers, FORM, true)?;
    if bytes.len() - end != length {
        return Err(Error);
    }
    Ok(&bytes[end..])
}
fn response_body(bytes: &[u8]) -> Result<&[u8], Error> {
    let end = boundary(bytes)?;
    let mut storage = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut response = httparse::Response::new(&mut storage);
    let parsed = response.parse(&bytes[..end]).map_err(|_| Error)?;
    if parsed != httparse::Status::Complete(end)
        || response.version != Some(1)
        || response.code != Some(200)
    {
        return Err(Error);
    }
    let length = framing(response.headers, JSON, false)?;
    if bytes.len() - end != length {
        return Err(Error);
    }
    Ok(&bytes[end..])
}
fn request(hostname: &str, route: &str, body: Vec<u8>) -> Result<Vec<u8>, Error> {
    host(hostname)?;
    let mut bytes=format!("POST {route} HTTP/1.1\r\nHost: {hostname}\r\nContent-Type: {FORM}\r\nContent-Length: {}\r\n\r\n",body.len()).into_bytes();
    bytes.extend(body);
    Ok(bytes)
}
fn response(status: &str, body: &[u8]) -> Vec<u8> {
    let mut bytes=format!("HTTP/1.1 {status}\r\nContent-Type: {JSON}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nPragma: no-cache\r\n\r\n",body.len()).into_bytes();
    bytes.extend_from_slice(body);
    bytes
}
pub fn encode_token_http(hostname: &str, input: &UnverifiedTokenRequest) -> Result<Vec<u8>, Error> {
    request(hostname, "/oauth/token", encode_token_form(input)?)
}
pub fn decode_token_http(bytes: &[u8]) -> Result<UnverifiedTokenRequest, Error> {
    decode_token_form(request_body(bytes, "/oauth/token")?)
}
pub fn encode_introspection_http(
    hostname: &str,
    input: &UnverifiedIntrospectionRequest,
) -> Result<Vec<u8>, Error> {
    request(
        hostname,
        "/oauth/introspect",
        encode_introspection_form(input)?,
    )
}
pub fn decode_introspection_http(bytes: &[u8]) -> Result<UnverifiedIntrospectionRequest, Error> {
    decode_introspection_form(request_body(bytes, "/oauth/introspect")?)
}
pub fn encode_token_response_http(input: &UnverifiedTokenResponse) -> Result<Vec<u8>, Error> {
    Ok(response("200 OK", &encode_token_json(input)?))
}
pub fn decode_token_response_http(bytes: &[u8]) -> Result<UnverifiedTokenResponse, Error> {
    decode_token_json(response_body(bytes)?)
}
pub fn encode_introspection_response_http(
    input: &UnverifiedIntrospection,
) -> Result<Vec<u8>, Error> {
    Ok(response("200 OK", &encode_introspection_json(input)?))
}
pub fn decode_introspection_response_http(bytes: &[u8]) -> Result<UnverifiedIntrospection, Error> {
    decode_introspection_json(response_body(bytes)?)
}
/// Fixed refusal bodies take no input text and reveal no member, token or reason.
pub fn encode_refusal_http(refusal: UnverifiedRefusal) -> Vec<u8> {
    match refusal {
        UnverifiedRefusal::InvalidRequest => {
            response("400 Bad Request", b"{\"error\":\"invalid_request\"}")
        }
        UnverifiedRefusal::InvalidToken => {
            response("401 Unauthorized", b"{\"error\":\"invalid_token\"}")
        }
        UnverifiedRefusal::Unavailable => response(
            "503 Service Unavailable",
            b"{\"error\":\"temporarily_unavailable\"}",
        ),
    }
}
