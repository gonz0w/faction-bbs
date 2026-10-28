//! Strict storage IQ wrapper. Only a trusted home adapter supplies the actor.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use faction_files::private_blobs::{self as blobs, Request, Response};
use faction_xmpp::Element;
pub const NS: &str = "urn:faction:blob:1";
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Invalid,
    Remote(blobs::Error),
}
fn token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn service(s: &str) -> bool {
    crate::mail_data::address(&format!("service@{s}")) && s.contains('.')
}
fn attr(mut e: Element, k: &str, v: &str) -> Result<Element, Error> {
    e.set_attr("".into(), k.try_into().map_err(|_| Error::Invalid)?, v);
    Ok(e)
}
fn iq(id: &str, kind: &str, from: &str, to: &str) -> Result<Element, Error> {
    if !token(id) {
        return Err(Error::Invalid);
    }
    let mut e = Element::builder("iq", "jabber:client").build();
    for (k, v) in [("id", id), ("type", kind), ("from", from), ("to", to)] {
        e = attr(e, k, v)?;
    }
    Ok(e)
}
fn root(e: &Element, kind: &str, own: &str, home: &str) -> Result<(), Error> {
    if !crate::mail_data::address(own)
        || !service(home)
        || !e.is("iq", "jabber:client")
        || e.attrs().len() != 4
        || !token(e.attr("id").ok_or(Error::Invalid)?)
        || e.attr("type") != Some(kind)
        || e.texts()
            .any(|t| !t.bytes().all(|b| b" \n\r\t".contains(&b)))
    {
        return Err(Error::Invalid);
    }
    Ok(())
}
fn sole<'a>(e: &'a Element, name: &str) -> Result<&'a Element, Error> {
    let mut children = e.children();
    let v = children.next().ok_or(Error::Invalid)?;
    if children.next().is_some() || !v.is(name, NS) {
        return Err(Error::Invalid);
    }
    Ok(v)
}
fn payload(e: &Element) -> Result<Vec<u8>, Error> {
    let v = sole(e, "blob")?;
    if !v.attrs().is_empty() || v.children().next().is_some() {
        return Err(Error::Invalid);
    }
    let s = v.text();
    if s.len() > blobs::MAX_WIRE.div_ceil(3) * 4 {
        return Err(Error::Invalid);
    }
    let b = STANDARD.decode(s.as_bytes()).map_err(|_| Error::Invalid)?;
    if b.len() > blobs::MAX_WIRE || STANDARD.encode(&b) != s {
        return Err(Error::Invalid);
    }
    Ok(b)
}
pub fn request(id: &str, own: &str, home: &str, request: &Request) -> Result<Element, Error> {
    if !crate::mail_data::address(own) || !service(home) {
        return Err(Error::Invalid);
    }
    let mut e = iq(id, "set", own, home)?;
    let b = blobs::encode_request(request).map_err(|_| Error::Invalid)?;
    e.append_child(
        Element::builder("blob", NS)
            .append(STANDARD.encode(b))
            .build(),
    );
    faction_xmpp::validate_output(&e).map_err(|_| Error::Invalid)?;
    Ok(e)
}
/// Must be called after the home authenticates `own`, independently of XML.
pub fn decode_request(e: &Element, own: &str, home: &str) -> Result<Request, Error> {
    root(e, "set", own, home)?;
    if e.attr("from") != Some(own) || e.attr("to") != Some(home) {
        return Err(Error::Invalid);
    }
    blobs::decode_request(&payload(e)?).map_err(|_| Error::Invalid)
}
/// Encode a bounded fixed error class; never serialize a path or diagnostic.
pub fn response(
    request: &Element,
    home: &str,
    result: Result<Response, blobs::Error>,
) -> Result<Element, Error> {
    let own = request.attr("from").ok_or(Error::Invalid)?;
    decode_request(request, own, home)?;
    let mut e = iq(
        request.attr("id").ok_or(Error::Invalid)?,
        if result.is_ok() { "result" } else { "error" },
        home,
        own,
    )?;
    match result {
        Ok(r) => {
            e.append_child(
                Element::builder("blob", NS)
                    .append(
                        STANDARD.encode(blobs::encode_response(&r).map_err(|_| Error::Invalid)?),
                    )
                    .build(),
            );
        }
        Err(_) => {
            e.append_child(Element::builder("failure", NS).build());
        }
    }
    faction_xmpp::validate_output(&e).map_err(|_| Error::Invalid)?;
    Ok(e)
}
pub fn decode_response(e: &Element, id: &str, own: &str, home: &str) -> Result<Response, Error> {
    let kind = e.attr("type").ok_or(Error::Invalid)?;
    if !matches!(kind, "result" | "error") {
        return Err(Error::Invalid);
    }
    root(e, kind, own, home)?;
    if e.attr("id") != Some(id) || e.attr("to") != Some(own) || e.attr("from") != Some(home) {
        return Err(Error::Invalid);
    }
    if kind == "error" {
        let v = sole(e, "failure")?;
        if !v.attrs().is_empty() || v.children().next().is_some() || !v.text().is_empty() {
            return Err(Error::Invalid);
        }
        return Err(Error::Remote(blobs::Error::Denied));
    }
    blobs::decode_response(&payload(e)?).map_err(|_| Error::Invalid)
}
