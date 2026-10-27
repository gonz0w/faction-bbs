//! Experimental bounded attachment metadata inside SCE; no blob or delivery API.
//!
//! Decode only after native authentication, then explicitly commit or abort the
//! native candidate. Parsing itself supplies no authentication or device routing.
//! Locators and names remain inert; this module never fetches or saves files.
use crate::{Error, Result, sce};
use base64::{Engine, engine::general_purpose::STANDARD};
use roxmltree::Node;
use zeroize::Zeroizing;

const SFS: &str = "urn:xmpp:sfs:0";
const META: &str = "urn:xmpp:file:metadata:0";
const HASH: &str = "urn:xmpp:hashes:2";
const ESFS: &str = "urn:xmpp:esfs:0";
const URL: &str = "http://jabber.org/protocol/url-data";
const CIPHER: &str = "urn:xmpp:ciphers:aes-256-gcm-nopadding:0";

/// Private asserted metadata, not proof that any blob has this size or hash.
/// Caller-supplied keys/IVs are not checked for freshness. No secret Debug output.
pub struct Descriptor {
    pub name: Zeroizing<String>,
    pub media_type: Zeroizing<String>,
    pub size: u64,
    pub hash: Zeroizing<[u8; 32]>,
    pub key: Zeroizing<[u8; 32]>,
    pub iv: Zeroizing<[u8; 12]>,
    pub locator: Zeroizing<String>,
}

/// Subject/body plus exactly one experimental attachment descriptor.
pub struct Content {
    pub text: sce::Content,
    pub attachment: Descriptor,
}

impl Descriptor {
    fn validate(&self) -> Result<()> {
        let name = self.name.as_str();
        let media = self.media_type.as_str();
        let locator = self.locator.as_str();
        let mut media_parts = media.split('/');
        let token = |s: &str| {
            !s.is_empty()
                && s.bytes().all(|b| {
                    b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b)
                })
        };
        if name.is_empty()
            || name.len() > 255
            || matches!(name, "." | "..")
            || name.chars().any(|c| c.is_control() || matches!(c, '/' | '\\'))
            || !sce_name(name)
            || media.len() > 127
            || !token(media_parts.next().ok_or(Error)?)
            || !token(media_parts.next().ok_or(Error)?)
            || media_parts.next().is_some()
            || locator.len() > 512
            || !locator.bytes().all(|b| (33..=126).contains(&b))
            || locator.contains(['?', '#', '\\'])
        {
            return Err(Error);
        }
        let rest = locator.strip_prefix("https://").ok_or(Error)?;
        let authority = rest.split('/').next().ok_or(Error)?;
        if authority.is_empty() || authority.contains('@') {
            return Err(Error);
        }
        self.size.checked_add(16).ok_or(Error)?;
        Ok(())
    }
}
fn sce_name(name: &str) -> bool {
    sce::bounded(name, 255).is_ok()
}
fn leaf(out: &mut String, name: &str, text: &str) {
    out.push('<');
    out.push_str(name);
    out.push('>');
    sce::escape(out, text);
    out.push_str("</");
    out.push_str(name);
    out.push('>');
}
fn binary_leaf(out: &mut String, name: &str, bytes: &[u8]) {
    let text = Zeroizing::new(STANDARD.encode(bytes));
    leaf(out, name, &text);
}

/// Encode private XML only; use the existing native transaction before sharing.
pub fn encode(
    sender: &str,
    recipient: &str,
    sent_at: u64,
    subject: &str,
    body: &str,
    descriptor: &Descriptor,
) -> Result<Zeroizing<Vec<u8>>> {
    descriptor.validate()?;
    let mut extra = Zeroizing::new(String::new());
    extra.push_str("<file-sharing xmlns='urn:xmpp:sfs:0' disposition='attachment'><file xmlns='urn:xmpp:file:metadata:0'>");
    leaf(&mut extra, "name", &descriptor.name);
    leaf(&mut extra, "media-type", &descriptor.media_type);
    let size = Zeroizing::new(descriptor.size.to_string());
    leaf(&mut extra, "size", &size);
    extra.push_str("<hash xmlns='urn:xmpp:hashes:2' algo='sha-256'>");
    let hash = Zeroizing::new(STANDARD.encode(&*descriptor.hash));
    extra.push_str(&hash);
    extra.push_str("</hash></file><sources><encrypted xmlns='urn:xmpp:esfs:0' cipher='urn:xmpp:ciphers:aes-256-gcm-nopadding:0'>");
    binary_leaf(&mut extra, "key", &*descriptor.key);
    binary_leaf(&mut extra, "iv", &*descriptor.iv);
    extra.push_str("<sources xmlns='urn:xmpp:sfs:0'><url-data xmlns='http://jabber.org/protocol/url-data' target='");
    sce::escape(&mut extra, &descriptor.locator);
    extra.push_str("'/></sources></encrypted></sources></file-sharing>");
    sce::encode_content(sender, recipient, sent_at, subject, body, Some(&extra))
}

fn no_attributes(node: Node<'_, '_>) -> Result<()> {
    if node.attributes().len() == 0 { Ok(()) } else { Err(Error) }
}
fn sole<'a, 'input>(node: Node<'a, 'input>, ns: &str, name: &str) -> Result<Node<'a, 'input>> {
    no_attributes(node)?;
    let children = sce::elements(node)?;
    if children.len() != 1 || !sce::named(children[0], ns, name) {
        return Err(Error);
    }
    Ok(children[0])
}
fn value(node: Node<'_, '_>, limit: usize) -> Result<Zeroizing<String>> {
    let mut text = Zeroizing::new(String::new());
    for child in node.children() {
        if !child.is_text() { return Err(Error); }
        text.push_str(child.text().ok_or(Error)?);
        if text.len() > limit { return Err(Error); }
    }
    sce::bounded(&text, limit)?;
    Ok(text)
}
fn binary<const N: usize>(node: Node<'_, '_>) -> Result<Zeroizing<[u8; N]>> {
    let text = value(node, N.div_ceil(3) * 4)?;
    let bytes = Zeroizing::new(STANDARD.decode(text.as_bytes()).map_err(|_| Error)?);
    if bytes.len() != N { return Err(Error); }
    let canonical = Zeroizing::new(STANDARD.encode(&*bytes));
    if *canonical != *text { return Err(Error); }
    let mut out = Zeroizing::new([0; N]);
    out.copy_from_slice(&bytes);
    Ok(out)
}
fn descriptor(node: Node<'_, '_>) -> Result<Descriptor> {
    if sce::only_attribute(node, "disposition")? != "attachment" { return Err(Error); }
    let children = sce::elements(node)?;
    if children.len() != 2 { return Err(Error); }
    let file = children.iter().copied().find(|n| sce::named(*n, META, "file")).ok_or(Error)?;
    let sources = children.iter().copied().find(|n| sce::named(*n, SFS, "sources")).ok_or(Error)?;
    no_attributes(file)?;
    let fields = sce::elements(file)?;
    if fields.len() != 4 { return Err(Error); }
    let mut name = None;
    let mut media = None;
    let mut size = None;
    let mut hash = None;
    for field in fields {
        if sce::named(field, META, "name") && name.is_none() {
            name = Some(sce::text_leaf(field, 255)?);
        } else if sce::named(field, META, "media-type") && media.is_none() {
            media = Some(sce::text_leaf(field, 127)?);
        } else if sce::named(field, META, "size") && size.is_none() {
            let text = sce::text_leaf(field, 20)?;
            if !text.bytes().all(|b| b.is_ascii_digit()) || (text.len() > 1 && text.starts_with('0')) { return Err(Error); }
            size = Some(text.parse::<u64>().map_err(|_| Error)?);
        } else if sce::named(field, HASH, "hash") && hash.is_none() {
            if sce::only_attribute(field, "algo")? != "sha-256" { return Err(Error); }
            hash = Some(binary(field)?);
        } else { return Err(Error); }
    }
    let encrypted = sole(sources, ESFS, "encrypted")?;
    if sce::only_attribute(encrypted, "cipher")? != CIPHER { return Err(Error); }
    let fields = sce::elements(encrypted)?;
    if fields.len() != 3 { return Err(Error); }
    let mut key = None;
    let mut iv = None;
    let mut locator = None;
    for field in fields {
        if sce::named(field, ESFS, "key") && key.is_none() {
            no_attributes(field)?;
            key = Some(binary(field)?);
        } else if sce::named(field, ESFS, "iv") && iv.is_none() {
            no_attributes(field)?;
            iv = Some(binary(field)?);
        } else if sce::named(field, SFS, "sources") && locator.is_none() {
            let url = sole(field, URL, "url-data")?;
            let target = sce::only_attribute(url, "target")?;
            if !sce::elements(url)?.is_empty() { return Err(Error); }
            locator = Some(Zeroizing::new(target.to_owned()));
        } else { return Err(Error); }
    }
    let descriptor = Descriptor { name: name.ok_or(Error)?, media_type: media.ok_or(Error)?, size: size.ok_or(Error)?, hash: hash.ok_or(Error)?, key: key.ok_or(Error)?, iv: iv.ok_or(Error)?, locator: locator.ok_or(Error)? };
    descriptor.validate()?;
    Ok(descriptor)
}

/// Validate the candidate after authentication, with the trusted sending time.
pub fn decode(raw: &[u8], sender: &str, recipient: &str, enclosing_sent_at: u64) -> Result<Content> {
    let (content, stamp) = decode_network(raw, sender, recipient)?;
    if enclosing_sent_at > sce::LAST_SECOND || stamp.abs_diff(enclosing_sent_at) > 300 { return Err(Error); }
    Ok(content)
}

/// Decode already authenticated XML; never trust transport or arrival timestamps.
pub fn decode_network(raw: &[u8], sender: &str, recipient: &str) -> Result<(Content, u64)> {
    let doc = sce::parse(raw)?;
    let (content, stamp) = sce::envelope(&doc, sender, recipient, &[SFS, META, HASH, ESFS, URL])?;
    let fields = sce::elements(content)?;
    if fields.len() != 3 { return Err(Error); }
    let mut attachment = None;
    let mut text = Vec::new();
    for field in fields {
        if sce::named(field, SFS, "file-sharing") && attachment.is_none() { attachment = Some(descriptor(field)?); }
        else { text.push(field); }
    }
    Ok((Content { text: sce::text_fields(&text)?, attachment: attachment.ok_or(Error)? }, stamp))
}
