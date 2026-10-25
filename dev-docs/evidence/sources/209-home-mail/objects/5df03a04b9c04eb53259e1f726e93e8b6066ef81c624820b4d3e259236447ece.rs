//! Strict, bounded single-peer OMEMO-2 XML profile for manual home mail.
//! Trust always comes from the caller's enrolled Ed25519 binding. These codecs
//! neither authenticate ciphertext nor advance native cryptographic state.
//! Device lists contain at most 32 IDs; bundles contain 25..=32 public prekeys.
//! Publication uses a bounded subset of the already-durable native store's 100
//! prekeys, excluding the passive session's consumed key. No new key generation,
//! automatic session reset, multi-device trust or full XEP conformance is claimed.
use crate::mail_checkpoint::Wire;
use crate::mail_data::{address, invalid};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use faction_omemo::Bundle;
use faction_xmpp::Element;
use std::io;

pub const OMEMO: &str = "urn:xmpp:omemo:2";
pub const DEVICES: &str = "urn:xmpp:omemo:2:devices";
pub const BUNDLES: &str = "urn:xmpp:omemo:2:bundles";
const CLIENT: &str = "jabber:client";
const PUBSUB: &str = "http://jabber.org/protocol/pubsub";
const MAX_LIST: usize = 32;

/// Obtained from the local checkpoint, never from a received stanza or bundle.
pub struct Binding<'a> {
    pub peer: &'a str,
    pub peer_device: u32,
    pub peer_identity: [u8; 32],
    pub own: &'a str,
    pub own_device: u32,
}

// This profile follows XEP-0384 0.9.1 (2026-04-06), sections 5.3–5.6:
// https://xmpp.org/extensions/xep-0384.html . It deliberately excludes labels,
// multiple recipients, empty session messages, MAM/carbons and plaintext fallback.
// Call only on Elements produced by the transport's bounded pre-DOM XML guard;
// a DOM cannot expose lexical constructs discarded by its parser.
fn token(s: &str, cap: usize) -> bool {
    !s.is_empty()
        && s.len() <= cap
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn device(id: u32) -> bool {
    (1..=i32::MAX as u32).contains(&id)
}
fn number(s: &str) -> io::Result<u32> {
    let n: u32 = s.parse().map_err(|_| invalid())?;
    if !device(n) || n.to_string() != s {
        return Err(invalid());
    }
    Ok(n)
}
fn bare(jid: &str) -> io::Result<&str> {
    let who = if let Some((who, resource)) = jid.split_once('/') {
        if !token(resource, 64) {
            return Err(invalid());
        }
        who
    } else {
        jid
    };
    if !address(who) {
        return Err(invalid());
    }
    Ok(who)
}
fn attrs(e: &Element, required: &[&str], optional: &[&str]) -> io::Result<()> {
    if required.iter().any(|k| e.attr(*k).is_none())
        || e.attrs().len()
            != required.len() + optional.iter().filter(|k| e.attr(**k).is_some()).count()
    {
        return Err(invalid());
    }
    Ok(())
}
fn container(
    e: &Element,
    name: &str,
    ns: &str,
    required: &[&str],
    optional: &[&str],
) -> io::Result<()> {
    if !e.is(name, ns)
        || e.texts()
            .any(|t| !t.bytes().all(|b| b" \t\r\n".contains(&b)))
    {
        return Err(invalid());
    }
    attrs(e, required, optional)
}
fn only<'a>(e: &'a Element, name: &str, ns: &str) -> io::Result<&'a Element> {
    let mut children = e.children();
    let child = children.next().ok_or_else(invalid)?;
    if children.next().is_some() || !child.is(name, ns) {
        return Err(invalid());
    }
    Ok(child)
}
fn child<'a>(e: &'a Element, name: &str, ns: &str) -> io::Result<&'a Element> {
    let mut matching = e.children().filter(|c| c.is(name, ns));
    let found = matching.next().ok_or_else(invalid)?;
    if matching.next().is_some() {
        return Err(invalid());
    }
    Ok(found)
}
fn budget(e: &Element) -> io::Result<()> {
    fn visit(e: &Element, depth: usize, nodes: &mut usize, bytes: &mut usize) -> io::Result<()> {
        *nodes += 1;
        if depth > 8 || *nodes > 128 || e.attrs().len() > 8 {
            return Err(invalid());
        }
        *bytes = bytes
            .checked_add(e.name().len() + e.ns().len())
            .ok_or_else(invalid)?;
        for ((ns, key), value) in e.attrs().iter() {
            if value.len() > 512 || value.chars().any(char::is_control) {
                return Err(invalid());
            }
            *bytes = bytes
                .checked_add(ns.to_string().len() + key.as_str().len() + value.len())
                .ok_or_else(invalid)?;
        }
        for text in e.texts() {
            *bytes = bytes.checked_add(text.len()).ok_or_else(invalid)?;
        }
        if *bytes > 16384 {
            return Err(invalid());
        }
        for child in e.children() {
            visit(child, depth + 1, nodes, bytes)?;
        }
        Ok(())
    }
    visit(e, 1, &mut 0, &mut 0)
}
fn leaf(
    e: &Element,
    name: &str,
    required: &[&str],
    optional: &[&str],
    min: usize,
    max: usize,
) -> io::Result<Vec<u8>> {
    if !e.is(name, OMEMO) || e.children().next().is_some() {
        return Err(invalid());
    }
    attrs(e, required, optional)?;
    // Bound before joining DOM text and before the base64 allocation.
    let length = e
        .texts()
        .try_fold(0usize, |n, t| n.checked_add(t.len()).ok_or_else(invalid))?;
    if length > max.div_ceil(3) * 4 {
        return Err(invalid());
    }
    let text = e.text();
    let raw = STANDARD.decode(&text).map_err(|_| invalid())?;
    if !(min..=max).contains(&raw.len()) || STANDARD.encode(&raw) != text {
        return Err(invalid());
    }
    Ok(raw)
}
fn element(name: &str, ns: &str, attributes: &[(&str, String)], children: Vec<Element>) -> Element {
    let mut builder = Element::builder(name, ns);
    for (key, value) in attributes {
        builder = builder.attr(
            (*key).try_into().expect("static XML attribute"),
            value.clone(),
        );
    }
    builder.append_all(children).build()
}
fn data(name: &str, attributes: &[(&str, String)], value: &[u8]) -> Element {
    let mut e = element(name, OMEMO, attributes, vec![]);
    e.append_text(STANDARD.encode(value));
    e
}
fn iq_request(
    id: &str,
    own: &str,
    peer: Option<&str>,
    kind: &str,
    payload: Element,
) -> io::Result<Element> {
    if !token(id, 128) || !address(own) || peer.is_some_and(|p| !address(p)) {
        return Err(invalid());
    }
    let mut a = vec![
        ("id", id.into()),
        ("from", own.into()),
        ("type", kind.into()),
    ];
    if let Some(peer) = peer {
        a.push(("to", peer.into()));
    }
    Ok(element("iq", CLIENT, &a, vec![payload]))
}
fn query(id: &str, own: &str, peer: &str, node: &str, item: Option<String>) -> io::Result<Element> {
    let children = item
        .map(|id| element("item", PUBSUB, &[("id", id)], vec![]))
        .into_iter()
        .collect();
    let items = element("items", PUBSUB, &[("node", node.into())], children);
    iq_request(
        id,
        own,
        Some(peer),
        "get",
        element("pubsub", PUBSUB, &[], vec![items]),
    )
}
fn publish(
    id: &str,
    own: &str,
    node: &str,
    item: &str,
    content: Element,
    bundles: bool,
) -> io::Result<Element> {
    let item = element("item", PUBSUB, &[("id", item.into())], vec![content]);
    let publication = element("publish", PUBSUB, &[("node", node.into())], vec![item]);
    let mut options = vec![
        (
            "FORM_TYPE",
            "http://jabber.org/protocol/pubsub#publish-options",
        ),
        ("pubsub#access_model", "open"),
    ];
    if bundles {
        options.push(("pubsub#max_items", "max"));
    }
    let fields = options
        .into_iter()
        .map(|(key, value)| {
            let value = Element::builder("value", "jabber:x:data")
                .append(value)
                .build();
            let mut a = vec![("var", key.into())];
            if key == "FORM_TYPE" {
                a.push(("type", "hidden".into()));
            }
            element("field", "jabber:x:data", &a, vec![value])
        })
        .collect();
    let form = element("x", "jabber:x:data", &[("type", "submit".into())], fields);
    let options = element("publish-options", PUBSUB, &[], vec![form]);
    iq_request(
        id,
        own,
        None,
        "set",
        element("pubsub", PUBSUB, &[], vec![publication, options]),
    )
}
fn result<'a>(
    e: &'a Element,
    id: &str,
    own: &str,
    peer: &str,
    node: &str,
    item: &str,
) -> io::Result<&'a Element> {
    budget(e)?;
    if !token(id, 128) || !address(own) || !address(peer) {
        return Err(invalid());
    }
    container(e, "iq", CLIENT, &["id", "type", "from", "to"], &[])?;
    if e.attr("id") != Some(id)
        || e.attr("type") != Some("result")
        || bare(e.attr("from").ok_or_else(invalid)?)? != peer
        || bare(e.attr("to").ok_or_else(invalid)?)? != own
    {
        return Err(invalid());
    }
    let pubsub = only(e, "pubsub", PUBSUB)?;
    container(pubsub, "pubsub", PUBSUB, &[], &[])?;
    let items = only(pubsub, "items", PUBSUB)?;
    container(items, "items", PUBSUB, &["node"], &[])?;
    if items.attr("node") != Some(node) {
        return Err(invalid());
    }
    let entry = only(items, "item", PUBSUB)?;
    container(entry, "item", PUBSUB, &["id"], &[])?;
    if entry.attr("id") != Some(item) {
        return Err(invalid());
    }
    Ok(entry)
}

/// Fetching a list grants no device trust; the caller must find its enrolled ID.
pub fn devices_query(id: &str, own: &str, peer: &str) -> io::Result<Element> {
    query(id, own, peer, DEVICES, None)
}
/// The caller must preserve other device IDs from its own checked list, or refuse
/// publication. This function does not silently discover or enroll devices.
pub fn devices_publish(id: &str, own: &str, devices: &[u32]) -> io::Result<Element> {
    if devices.is_empty() || devices.len() > MAX_LIST {
        return Err(invalid());
    }
    let mut children = Vec::new();
    for (i, &id) in devices.iter().enumerate() {
        if !device(id) || devices[..i].contains(&id) {
            return Err(invalid());
        }
        children.push(element("device", OMEMO, &[("id", id.to_string())], vec![]));
    }
    publish(
        id,
        own,
        DEVICES,
        "current",
        element("devices", OMEMO, &[], children),
        false,
    )
}
pub fn devices_result(xml: &Element, id: &str, own: &str, peer: &str) -> io::Result<Vec<u32>> {
    let entry = result(xml, id, own, peer, DEVICES, "current")?;
    let devices = only(entry, "devices", OMEMO)?;
    container(devices, "devices", OMEMO, &[], &[])?;
    let mut out = Vec::new();
    for item in devices.children() {
        container(item, "device", OMEMO, &["id"], &[])?;
        if item.children().next().is_some() || out.len() == MAX_LIST {
            return Err(invalid());
        }
        let id = number(item.attr("id").ok_or_else(invalid)?)?;
        if out.contains(&id) {
            return Err(invalid());
        }
        out.push(id);
    }
    Ok(out)
}
pub fn bundle_query(id: &str, own: &str, peer: &str, device_id: u32) -> io::Result<Element> {
    if !device(device_id) {
        return Err(invalid());
    }
    query(id, own, peer, BUNDLES, Some(device_id.to_string()))
}
/// Publishes only the supplied public material. Caller obtains it from a durable
/// checkpoint; this function cannot generate or persist private prekeys. Only
/// the bundle's identity, signed prekey and signature are used; the supplied
/// public-prekey list supersedes its single legacy `prekey` field.
pub fn bundle_publish(
    id: &str,
    own: &str,
    device_id: u32,
    bundle: &Bundle,
    prekeys: &[(u32, [u8; 32])],
) -> io::Result<Element> {
    if !device(device_id)
        || !device(bundle.signed_prekey_id)
        || !(25..=MAX_LIST).contains(&prekeys.len())
    {
        return Err(invalid());
    }
    let mut children = Vec::new();
    for (i, (id, key)) in prekeys.iter().enumerate() {
        if !device(*id) || prekeys[..i].iter().any(|p| p.0 == *id) {
            return Err(invalid());
        }
        children.push(data("pk", &[("id", id.to_string())], key));
    }
    let prekeys = element("prekeys", OMEMO, &[], children);
    let content = element(
        "bundle",
        OMEMO,
        &[],
        vec![
            data(
                "spk",
                &[("id", bundle.signed_prekey_id.to_string())],
                &bundle.signed_prekey,
            ),
            data("spks", &[], &bundle.signature),
            data("ik", &[], &bundle.identity),
            prekeys,
        ],
    );
    publish(id, own, BUNDLES, &device_id.to_string(), content, true)
}
/// Checks every offered public prekey before uniformly selecting one in this
/// bounded fixed-peer profile. `start` performs native signature/session validation after
/// matching the already-enrolled full Ed25519 identity; XML never grants trust.
pub fn bundle_result(
    xml: &Element,
    id: &str,
    own: &str,
    peer: &str,
    device_id: u32,
    identity: &[u8; 32],
) -> io::Result<Bundle> {
    if !device(device_id) {
        return Err(invalid());
    }
    let entry = result(xml, id, own, peer, BUNDLES, &device_id.to_string())?;
    let b = only(entry, "bundle", OMEMO)?;
    container(b, "bundle", OMEMO, &[], &[])?;
    if b.children().count() != 4 {
        return Err(invalid());
    }
    let ik = leaf(child(b, "ik", OMEMO)?, "ik", &[], &[], 32, 32)?;
    if ik != identity {
        return Err(invalid());
    }
    let spk = child(b, "spk", OMEMO)?;
    let signed_prekey = leaf(spk, "spk", &["id"], &[], 32, 32)?
        .try_into()
        .map_err(|_| invalid())?;
    let signed_prekey_id = number(spk.attr("id").ok_or_else(invalid)?)?;
    let signature = leaf(child(b, "spks", OMEMO)?, "spks", &[], &[], 64, 64)?
        .try_into()
        .map_err(|_| invalid())?;
    let prekeys = child(b, "prekeys", OMEMO)?;
    container(prekeys, "prekeys", OMEMO, &[], &[])?;
    let mut ids = Vec::new();
    let mut keys = Vec::new();
    for pk in prekeys.children() {
        if ids.len() == MAX_LIST {
            return Err(invalid());
        }
        let key: [u8; 32] = leaf(pk, "pk", &["id"], &[], 32, 32)?
            .try_into()
            .map_err(|_| invalid())?;
        let id = number(pk.attr("id").ok_or_else(invalid)?)?;
        if ids.contains(&id) {
            return Err(invalid());
        }
        ids.push(id);
        keys.push((id, key));
    }
    if !(25..=MAX_LIST).contains(&keys.len()) {
        return Err(invalid());
    }
    let mut selected = None;
    let count = keys.len() as u32;
    let ceiling = u32::MAX - u32::MAX % count;
    // Rejection sampling avoids modulo bias; an entropy source failure cannot
    // cause an unbounded synchronization cycle.
    for _ in 0..16 {
        let mut raw = [0; 4];
        getrandom::fill(&mut raw).map_err(|_| invalid())?;
        let sample = u32::from_ne_bytes(raw);
        if sample < ceiling {
            selected = Some((sample % count) as usize);
            break;
        }
    }
    let (prekey_id, prekey) = keys[selected.ok_or_else(invalid)?];
    Ok(Bundle {
        identity: *identity,
        signed_prekey,
        signature,
        signed_prekey_id,
        prekey,
        prekey_id,
    })
}

/// Encode immutable, already-persisted ciphertext. Operation IDs are retry labels,
/// not a peer receipt. Identity and sending time stay out of the clear XML.
pub fn message(wire: &Wire, operation: [u8; 16]) -> io::Result<Element> {
    if wire.namespace != OMEMO
        || !address(&wire.from)
        || !address(&wire.to)
        || !device(wire.device)
        || !device(wire.to_device)
        || operation == [0; 16]
        || wire.key.is_empty()
        || wire.key.len() > 256
        || !(16..=8208).contains(&wire.payload.len())
        || wire.sent_at > 253_402_300_799
    {
        return Err(invalid());
    }
    let key = data(
        "key",
        &[
            ("rid", wire.to_device.to_string()),
            ("kex", wire.prekey.to_string()),
        ],
        &wire.key,
    );
    let keys = element("keys", OMEMO, &[("jid", wire.to.clone())], vec![key]);
    let header = element(
        "header",
        OMEMO,
        &[("sid", wire.device.to_string())],
        vec![keys],
    );
    let encrypted = element(
        "encrypted",
        OMEMO,
        &[],
        vec![header, data("payload", &[], &wire.payload)],
    );
    Ok(element(
        "message",
        CLIENT,
        &[
            ("id", crate::mail_data::hex(&operation)),
            ("from", wire.from.clone()),
            ("to", wire.to.clone()),
            ("type", "chat".into()),
        ],
        vec![encrypted],
    ))
}
/// Decode only the one enrolled peer and recipient device. `sent_at` is a zero
/// placeholder: pass the result ONLY to `CheckpointVault::receive_network`, which
/// derives time from authenticated SCE. Do not use arrival time or an XML claim.
/// The enrolled identity is copied from the trusted caller, never from XML.
pub fn decode_message(xml: &Element, binding: Binding<'_>) -> io::Result<Wire> {
    budget(xml)?;
    if !address(binding.peer)
        || !address(binding.own)
        || !device(binding.peer_device)
        || !device(binding.own_device)
    {
        return Err(invalid());
    }
    container(xml, "message", CLIENT, &["from", "to"], &["id", "type"])?;
    if xml.attr("id").is_some_and(|id| !token(id, 128))
        || xml.attr("type").is_some_and(|kind| kind != "chat")
        || bare(xml.attr("from").ok_or_else(invalid)?)? != binding.peer
        || bare(xml.attr("to").ok_or_else(invalid)?)? != binding.own
    {
        return Err(invalid());
    }
    let encrypted = child(xml, "encrypted", OMEMO)?;
    match xml.children().count() {
        1 => {}
        2 => {
            let hint = child(xml, "store", "urn:xmpp:hints")?;
            container(hint, "store", "urn:xmpp:hints", &[], &[])?;
            if hint.children().next().is_some() {
                return Err(invalid());
            }
        }
        _ => return Err(invalid()),
    }
    container(encrypted, "encrypted", OMEMO, &[], &[])?;
    if encrypted.children().count() != 2 {
        return Err(invalid());
    }
    let header = child(encrypted, "header", OMEMO)?;
    container(header, "header", OMEMO, &["sid"], &[])?;
    if number(header.attr("sid").ok_or_else(invalid)?)? != binding.peer_device {
        return Err(invalid());
    }
    let keys = only(header, "keys", OMEMO)?;
    container(keys, "keys", OMEMO, &["jid"], &[])?;
    if keys.attr("jid") != Some(binding.own) {
        return Err(invalid());
    }
    let key = only(keys, "key", OMEMO)?;
    let raw = leaf(key, "key", &["rid"], &["kex"], 1, 256)?;
    if number(key.attr("rid").ok_or_else(invalid)?)? != binding.own_device {
        return Err(invalid());
    }
    let prekey = match key.attr("kex") {
        None | Some("false") => false,
        Some("true") => true,
        _ => return Err(invalid()),
    };
    let payload = leaf(
        child(encrypted, "payload", OMEMO)?,
        "payload",
        &[],
        &[],
        16,
        8208,
    )?;
    Ok(Wire {
        namespace: OMEMO.into(),
        from: binding.peer.into(),
        to: binding.own.into(),
        device: binding.peer_device,
        to_device: binding.own_device,
        identity: binding.peer_identity,
        sent_at: 0,
        key: raw,
        prekey,
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const OWN: &str = "alice@home.test";
    const PEER: &str = "bob@away.test";
    fn parse(s: &str) -> Element {
        s.parse().unwrap()
    }
    fn binding() -> Binding<'static> {
        Binding {
            own: OWN,
            own_device: 101,
            peer: PEER,
            peer_device: 202,
            peer_identity: [7; 32],
        }
    }
    fn wire() -> Wire {
        Wire {
            namespace: OMEMO.into(),
            from: PEER.into(),
            to: OWN.into(),
            device: 202,
            to_device: 101,
            identity: [7; 32],
            sent_at: 1_792_944_000,
            key: vec![1, 2, 3, 4],
            prekey: true,
            payload: vec![5; 32],
        }
    }
    fn bundle() -> Bundle {
        Bundle {
            identity: [7; 32],
            signed_prekey: [8; 32],
            signature: [9; 64],
            signed_prekey_id: 3,
            prekey: [10; 32],
            prekey_id: 4,
        }
    }
    fn public_keys() -> Vec<(u32, [u8; 32])> {
        (4..=28).map(|id| (id, [id as u8; 32])).collect()
    }
    #[test]
    fn mail_wire_accepts_inert_standard_root_iq_xml_language() {
        let raw = format!("<iq xmlns='jabber:client' xml:lang='en' from='{PEER}' to='{OWN}/local' type='result' id='q1'><pubsub xmlns='http://jabber.org/protocol/pubsub'><items node='urn:xmpp:omemo:2:devices'><item id='current'><devices xmlns='urn:xmpp:omemo:2'><device id='202'/></devices></item></items></pubsub></iq>");
        for language in ["en", "en-US", "", &"a".repeat(64)] {
            let input = raw.replace("xml:lang='en'", &format!("xml:lang='{language}'"));
            assert_eq!(devices_result(&parse(&input), "q1", OWN, PEER).unwrap(), [202]);
        }
        let bundle = iq(BUNDLES, "202", &independent_bundle())
            .replacen("<iq ", "<iq xml:lang='en' ", 1);
        assert_eq!(bundle_result(&parse(&bundle), "q1", OWN, PEER, 202, &[7; 32]).unwrap().identity, [7; 32]);
    }
    #[test]
    fn mail_wire_accepts_inert_standard_root_message_xml_language() {
        let raw = independent_message();
        let expected = decode_message(&parse(&raw), binding()).unwrap();
        for language in ["en", "en-US", "", &"a".repeat(64)] {
            let input = raw.replacen("<message ", &format!("<message xml:lang='{language}' "), 1);
            // Language metadata cannot change identity, addressing, ciphertext,
            // prekey state or the authenticated-time placeholder.
            assert_eq!(decode_message(&parse(&input), binding()).unwrap(), expected);
        }
    }
    #[test]
    fn mail_wire_rejects_language_namespace_shape_and_bounds() {
        let iq = iq(DEVICES, "current", &format!("<devices xmlns='{OMEMO}'><device id='202'/></devices>"));
        let message = independent_message();
        for attribute in [
            "lang='en'".to_owned(),
            "xmlns:other='urn:wrong' other:lang='en'".to_owned(),
            "xml:space='preserve'".to_owned(),
            "xml:lang='en' unknown='x'".to_owned(),
            format!("xml:lang='{}'", "a".repeat(65)),
            "xml:lang='en_US'".to_owned(),
            "xml:lang='é'".to_owned(),
            "xml:lang='en US'".to_owned(),
            "xml:lang='en&#x9;US'".to_owned(),
        ] {
            let input = iq.replacen("<iq ", &format!("<iq {attribute} "), 1);
            assert!(devices_result(&parse(&input), "q1", OWN, PEER).is_err(), "accepted {attribute}");
            let input = message.replacen("<message ", &format!("<message {attribute} "), 1);
            assert!(decode_message(&parse(&input), binding()).is_err(), "accepted {attribute}");
        }
        let nested = iq.replace("<devices ", "<devices xml:lang='en' ");
        assert!(devices_result(&parse(&nested), "q1", OWN, PEER).is_err());
        let nested = message.replace("<header ", "<header xml:lang='en' ");
        assert!(decode_message(&parse(&nested), binding()).is_err());
    }
    #[test]
    fn mail_wire_requires_standard_minimum_and_bounded_public_prekeys() {
        let keys = public_keys();
        let e = bundle_publish("q1", OWN, 101, &bundle(), &keys).unwrap();
        let serialized = xml(&e);
        assert_eq!(serialized.matches("<pk ").count(), 25);
        assert!(bundle_publish("q1", OWN, 101, &bundle(), &keys[..24]).is_err());
        let mut duplicate = keys.clone();
        duplicate[24].0 = duplicate[0].0;
        assert!(bundle_publish("q1", OWN, 101, &bundle(), &duplicate).is_err());
        let too_many: Vec<_> = (1..=33).map(|id| (id, [1; 32])).collect();
        assert!(bundle_publish("q1", OWN, 101, &bundle(), &too_many).is_err());
    }
    fn independent_message() -> String {
        format!(
            "<message xmlns='jabber:client' from='{PEER}/reference' to='{OWN}/local' type='chat' id='independent'><encrypted xmlns='{OMEMO}'><header sid='202'><keys jid='{OWN}'><key rid='101' kex='true'>AQIDBA==</key></keys></header><payload>{}</payload></encrypted></message>",
            STANDARD.encode([5; 32])
        )
    }
    fn iq(node: &str, item: &str, content: &str) -> String {
        format!(
            "<iq xmlns='{CLIENT}' from='{PEER}' to='{OWN}/local' type='result' id='q1'><pubsub xmlns='{PUBSUB}'><items node='{node}'><item id='{item}'>{content}</item></items></pubsub></iq>"
        )
    }
    fn independent_bundle() -> String {
        independent_bundle_count(25)
    }
    fn independent_bundle_count(count: u32) -> String {
        let prekeys: String = (4..4 + count)
            .map(|id| format!("<pk id='{id}'>{}</pk>", STANDARD.encode([id as u8 + 6; 32])))
            .collect();
        format!(
            "<bundle xmlns='{OMEMO}'><spk id='3'>{}</spk><spks>{}</spks><ik>{}</ik><prekeys>{prekeys}</prekeys></bundle>",
            STANDARD.encode([8; 32]),
            STANDARD.encode([9; 64]),
            STANDARD.encode([7; 32])
        )
    }
    fn xml(e: &Element) -> String {
        let mut b = Vec::new();
        e.write_to(&mut b).unwrap();
        String::from_utf8(b).unwrap()
    }
    #[test]
    fn mail_wire_publishes_standard_nodes_and_public_material_only() {
        let q = devices_query("q1", OWN, PEER).unwrap();
        assert_eq!(q.attr("type"), Some("get"));
        assert_eq!(q.attr("to"), Some(PEER));
        assert!(xml(&q).contains(DEVICES));
        let d = devices_publish("q1", OWN, &[101]).unwrap();
        assert!(xml(&d).contains("current"));
        assert!(xml(&d).contains("pubsub#access_model"));
        let b = bundle_publish("q2", OWN, 101, &bundle(), &public_keys()).unwrap();
        let s = xml(&b);
        assert!(s.contains(BUNDLES));
        assert!(s.contains("pubsub#max_items"));
        assert!(s.contains(&STANDARD.encode([7; 32])));
        assert!(s.contains("<spks"));
        let query = bundle_query("q1", OWN, PEER, 202).unwrap();
        assert!(xml(&query).contains("202"));
    }
    #[test]
    fn mail_wire_reads_independent_standard_devices_and_bundle() {
        let devices = iq(
            DEVICES,
            "current",
            &format!("<devices xmlns='{OMEMO}'><device id='202'/><device id='303'/></devices>"),
        );
        assert_eq!(
            devices_result(&parse(&devices), "q1", OWN, PEER).unwrap(),
            [202, 303]
        );
        let result = iq(BUNDLES, "202", &independent_bundle());
        let got = bundle_result(&parse(&result), "q1", OWN, PEER, 202, &[7; 32]).unwrap();
        assert_eq!(got.identity, [7; 32]);
        // XEP-0384 selection is random; every independently supplied key must
        // retain its exact id/value association.
        assert!((4..29).contains(&got.prekey_id));
        assert_eq!(got.prekey, [got.prekey_id as u8 + 6; 32]);
        assert_eq!(got.signature, [9; 64]);
        assert_eq!(got.signed_prekey_id, 3);
    }
    #[test]
    fn mail_wire_message_roundtrip_excludes_clear_identity_and_time() {
        let expected = wire();
        let encoded = message(&expected, [2; 16]).unwrap();
        let text = xml(&encoded);
        assert!(!text.contains("sent_at"));
        assert!(!text.contains("identity"));
        assert!(!text.contains(&expected.sent_at.to_string()));
        let got = decode_message(&encoded, binding()).unwrap();
        assert_eq!(got.sent_at, 0); // ONLY receive_network derives authenticated SCE time.
        let mut normalized = expected;
        normalized.sent_at = 0;
        assert_eq!(got, normalized);
    }
    #[test]
    fn mail_wire_reads_independent_message_and_optional_false_kex() {
        let raw = independent_message();
        let got = decode_message(&parse(&raw), binding()).unwrap();
        assert_eq!(got.key, [1, 2, 3, 4]);
        assert_eq!(got.identity, [7; 32]);
        assert!(got.prekey);
        for kex in ["", " kex='false'"] {
            let got = decode_message(&parse(&raw.replace(" kex='true'", kex)), binding()).unwrap();
            assert!(!got.prekey);
        }
    }
    #[test]
    fn mail_wire_rejects_underfilled_discovery_bundle() {
        // XEP-0384 0.9.1 section 4.2 requires at least 25 public prekeys.
        // The earlier two-key syntax fixture is not a publishable profile.
        let raw = iq(BUNDLES, "202", &independent_bundle_count(2));
        assert!(bundle_result(&parse(&raw), "q1", OWN, PEER, 202, &[7; 32]).is_err());
    }
    #[test]
    fn mail_wire_accepts_exact_standard_store_hint_only() {
        let raw = independent_message()
            .replace("</message>", "<store xmlns='urn:xmpp:hints'/></message>");
        assert!(decode_message(&parse(&raw), binding()).is_ok());
        for (a, b) in [
            ("<store ", "<store extra='x' "),
            ("/></message>", "><body>plaintext</body></store></message>"),
            (
                "/></message>",
                "/><store xmlns='urn:xmpp:hints'/></message>",
            ),
        ] {
            assert!(decode_message(&parse(&raw.replace(a, b)), binding()).is_err());
        }
    }
    #[test]
    fn mail_wire_rejects_substitution_and_shape_confusion() {
        let raw = independent_message();
        for (before, after) in [
            ("bob@away.test/reference", "mallory@away.test/reference"),
            ("alice@home.test/local", "alice@other.test/local"),
            ("sid='202'", "sid='203'"),
            ("rid='101'", "rid='102'"),
            ("jid='alice@home.test'", "jid='alice@home.test/other'"),
            ("sid='202'", "sid='0202'"),
            ("sid='202'", "sid='2147483648'"),
            ("kex='true'", "kex='yes'"),
            ("urn:xmpp:omemo:2", "urn:xmpp:omemo:1"),
            ("<header ", "<header unknown='x' "),
            ("</header>", "<extra/></header>"),
            ("</keys>", "<key rid='101'>AQIDBA==</key></keys>"),
            ("</encrypted>", "<header sid='202'/></encrypted>"),
            ("</message>", "<body>plaintext fallback</body></message>"),
            ("<encrypted ", "text<encrypted "),
            ("AQIDBA==", "AQIDBA"),
            ("AQIDBA==", "AQIDBB=="),
            ("AQIDBA==", " AQIDBA=="),
            ("<payload>", "<payload unknown='x'>"),
            ("AQIDBA==", "<extra/>AQIDBA=="),
            ("type='chat'", "type='groupchat'"),
            ("id='independent'", "id='bad&#x1b;'"),
        ] {
            let changed = raw.replace(before, after);
            // XML controls rejected by parser before a DOM is formed also satisfy refusal.
            if let Ok(parsed) = changed.parse::<Element>() {
                assert!(
                    decode_message(&parsed, binding()).is_err(),
                    "accepted mutation {before} -> {after}"
                );
            }
        }
    }
    #[test]
    fn mail_wire_rejects_wrong_iq_owner_id_node_and_extra_children() {
        let raw = iq(
            DEVICES,
            "current",
            &format!("<devices xmlns='{OMEMO}'><device id='202'/></devices>"),
        );
        for (before, after) in [
            ("id='q1'", "id='q2'"),
            ("type='result'", "type='set'"),
            ("bob@away.test", "eve@away.test"),
            ("alice@home.test/local", "alice@other.test/local"),
            (DEVICES, BUNDLES),
            ("id='current'", "id='old'"),
            ("</devices>", "<device id='202'/></devices>"),
            ("id='202'", "id='0'"),
            ("id='202'", "id='+202'"),
            ("<devices ", "<devices extra='yes' "),
            ("</items>", "<item id='other'/></items>"),
            ("</iq>", "<extra/></iq>"),
            ("<pubsub ", "garbage<pubsub "),
        ] {
            assert!(
                devices_result(&parse(&raw.replace(before, after)), "q1", OWN, PEER).is_err(),
                "accepted {before} -> {after}"
            );
        }
    }
    #[test]
    fn mail_wire_bundle_refuses_changed_identity_duplicate_or_unknown_keys() {
        let raw = iq(BUNDLES, "202", &independent_bundle());
        assert!(bundle_result(&parse(&raw), "q1", OWN, PEER, 202, &[99; 32]).is_err());
        for (before, after) in [
            ("id='202'", "id='203'"),
            ("id='5'", "id='4'"),
            ("<ik>", "<ik extra='x'>"),
            ("</bundle>", "<unknown/></bundle>"),
            ("<prekeys>", "<prekeys>text"),
            ("<pk id='5'>", "<pk id='0'>"),
            ("<spk id='3'>", "<spk id='03'>"),
            ("<spks>", "<spks xmlns='wrong'>"),
        ] {
            assert!(
                bundle_result(
                    &parse(&raw.replace(before, after)),
                    "q1",
                    OWN,
                    PEER,
                    202,
                    &[7; 32]
                )
                .is_err(),
                "accepted {before} -> {after}"
            );
        }
    }
    #[test]
    fn mail_wire_enforces_input_and_encoded_bounds() {
        let mut w = wire();
        w.payload = vec![5; 8209];
        assert!(message(&w, [1; 16]).is_err());
        w = wire();
        w.key = vec![2; 257];
        assert!(message(&w, [1; 16]).is_err());
        w = wire();
        w.from = "Bob@away.test".into();
        assert!(message(&w, [1; 16]).is_err());
        assert!(message(&wire(), [0; 16]).is_err());
        assert!(devices_query("bad\u{1b}", OWN, PEER).is_err());
        assert!(devices_publish("q1", OWN, &[101, 101]).is_err());
        assert!(devices_publish("q1", OWN, &(1..=33).collect::<Vec<_>>()).is_err());
        let raw = independent_message().replace("AQIDBA==", &STANDARD.encode([1; 257]));
        assert!(decode_message(&parse(&raw), binding()).is_err());
        let raw =
            independent_message().replace(&STANDARD.encode([5; 32]), &STANDARD.encode([5; 8209]));
        assert!(decode_message(&parse(&raw), binding()).is_err());
        let devices: String = (1..=33).map(|id| format!("<device id='{id}'/>")).collect();
        let raw = iq(
            DEVICES,
            "current",
            &format!("<devices xmlns='{OMEMO}'>{devices}</devices>"),
        );
        assert!(devices_result(&parse(&raw), "q1", OWN, PEER).is_err());
    }
    #[test]
    fn mail_wire_full_jid_is_scoped_and_resources_are_inert() {
        let raw = independent_message();
        for resource in ["", "x/y", "x&#x9;y", "x y", "x@else.test", "../x"] {
            let altered = raw.replace("/reference", &format!("/{resource}"));
            assert!(
                decode_message(&parse(&altered), binding()).is_err(),
                "accepted resource {resource}"
            );
        }
        let bare = raw.replace("/reference", "").replace("/local", "");
        assert!(decode_message(&parse(&bare), binding()).is_ok());
    }
}
