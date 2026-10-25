// This profile follows XEP-0384 0.9.1 (2026-04-06), sections 5.3–5.6:
// https://xmpp.org/extensions/xep-0384.html . It deliberately excludes labels,
// multiple recipients, empty session messages, MAM/carbons and plaintext fallback.
// Call only on Elements produced by the transport's bounded pre-DOM XML guard;
// a DOM cannot expose lexical constructs discarded by its parser.
fn token(s: &str, cap: usize) -> bool {
    !s.is_empty() && s.len() <= cap && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn device(id: u32) -> bool { (1..=i32::MAX as u32).contains(&id) }
fn number(s: &str) -> io::Result<u32> {
    let n: u32 = s.parse().map_err(|_| invalid())?;
    if !device(n) || n.to_string() != s { return Err(invalid()); }
    Ok(n)
}
fn bare(jid: &str) -> io::Result<&str> {
    let who = if let Some((who, resource)) = jid.split_once('/') {
        if !token(resource, 64) { return Err(invalid()); }
        who
    } else { jid };
    if !address(who) { return Err(invalid()); }
    Ok(who)
}
fn attrs(e: &Element, required: &[&str], optional: &[&str]) -> io::Result<()> {
    if required.iter().any(|k| e.attr(k).is_none())
        || e.attrs().len() != required.len() + optional.iter().filter(|k| e.attr(k).is_some()).count()
    { return Err(invalid()); }
    Ok(())
}
fn container(e: &Element, name: &str, ns: &str, required: &[&str], optional: &[&str]) -> io::Result<()> {
    if !e.is(name,ns) || e.texts().any(|t| !t.bytes().all(|b| b" \t\r\n".contains(&b))) {
        return Err(invalid());
    }
    attrs(e,required,optional)
}
fn only<'a>(e: &'a Element, name: &str, ns: &str) -> io::Result<&'a Element> {
    let mut children = e.children();
    let child = children.next().ok_or_else(invalid)?;
    if children.next().is_some() || !child.is(name,ns) { return Err(invalid()); }
    Ok(child)
}
fn child<'a>(e: &'a Element, name: &str, ns: &str) -> io::Result<&'a Element> {
    let mut matching = e.children().filter(|c| c.is(name,ns));
    let found = matching.next().ok_or_else(invalid)?;
    if matching.next().is_some() { return Err(invalid()); }
    Ok(found)
}
fn budget(e: &Element) -> io::Result<()> {
    fn visit(e: &Element, depth: usize, nodes: &mut usize, bytes: &mut usize) -> io::Result<()> {
        *nodes += 1;
        if depth > 8 || *nodes > 128 || e.attrs().len() > 8 { return Err(invalid()); }
        *bytes = bytes.checked_add(e.name().len() + e.ns().len()).ok_or_else(invalid)?;
        for ((ns,key), value) in e.attrs().iter() {
            if value.len() > 512 || value.chars().any(char::is_control) { return Err(invalid()); }
            *bytes = bytes.checked_add(ns.to_string().len() + key.as_str().len() + value.len()).ok_or_else(invalid)?;
        }
        for text in e.texts() { *bytes = bytes.checked_add(text.len()).ok_or_else(invalid)?; }
        if *bytes > 16384 { return Err(invalid()); }
        for child in e.children() { visit(child,depth+1,nodes,bytes)?; }
        Ok(())
    }
    visit(e,1,&mut 0,&mut 0)
}
fn leaf(e: &Element, name: &str, required: &[&str], optional: &[&str], min: usize, max: usize) -> io::Result<Vec<u8>> {
    if !e.is(name,OMEMO) || e.children().next().is_some() { return Err(invalid()); }
    attrs(e,required,optional)?;
    // Bound before joining DOM text and before the base64 allocation.
    let length = e.texts().try_fold(0usize, |n,t| n.checked_add(t.len()).ok_or_else(invalid))?;
    if length > max.div_ceil(3)*4 { return Err(invalid()); }
    let text = e.text();
    let raw = STANDARD.decode(&text).map_err(|_| invalid())?;
    if !(min..=max).contains(&raw.len()) || STANDARD.encode(&raw) != text { return Err(invalid()); }
    Ok(raw)
}
fn element(name: &str, ns: &str, attributes: &[(&str,String)], children: Vec<Element>) -> Element {
    let mut builder = Element::builder(name,ns);
    for (key,value) in attributes { builder = builder.attr((*key).try_into().expect("static XML attribute"),value.clone()); }
    builder.append_all(children).build()
}
fn data(name: &str, attributes: &[(&str,String)], value: &[u8]) -> Element {
    let mut e = element(name,OMEMO,attributes,vec![]);
    e.append_text(STANDARD.encode(value)); e
}
fn iq_request(id: &str, own: &str, peer: Option<&str>, kind: &str, payload: Element) -> io::Result<Element> {
    if !token(id,128) || !address(own) || peer.is_some_and(|p| !address(p)) { return Err(invalid()); }
    let mut a = vec![("id",id.into()),("from",own.into()),("type",kind.into())];
    if let Some(peer) = peer { a.push(("to",peer.into())); }
    Ok(element("iq",CLIENT,&a,vec![payload]))
}
fn query(id: &str, own: &str, peer: &str, node: &str, item: Option<String>) -> io::Result<Element> {
    let children = item.map(|id| element("item",PUBSUB,&[("id",id)],vec![])).into_iter().collect();
    let items = element("items",PUBSUB,&[("node",node.into())],children);
    iq_request(id,own,Some(peer),"get",element("pubsub",PUBSUB,&[],vec![items]))
}
fn publish(id: &str, own: &str, node: &str, item: &str, content: Element, bundles: bool) -> io::Result<Element> {
    let item = element("item",PUBSUB,&[("id",item.into())],vec![content]);
    let publication = element("publish",PUBSUB,&[("node",node.into())],vec![item]);
    let mut options = vec![("FORM_TYPE","http://jabber.org/protocol/pubsub#publish-options"),("pubsub#access_model","open")];
    if bundles { options.push(("pubsub#max_items","max")); }
    let fields = options.into_iter().map(|(key,value)| {
        let value = Element::builder("value","jabber:x:data").append(value).build();
        let mut a = vec![("var",key.into())];
        if key == "FORM_TYPE" { a.push(("type","hidden".into())); }
        element("field","jabber:x:data",&a,vec![value])
    }).collect();
    let form = element("x","jabber:x:data",&[("type","submit".into())],fields);
    let options = element("publish-options",PUBSUB,&[],vec![form]);
    iq_request(id,own,None,"set",element("pubsub",PUBSUB,&[],vec![publication,options]))
}
fn result<'a>(e: &'a Element, id: &str, own: &str, peer: &str, node: &str, item: &str) -> io::Result<&'a Element> {
    budget(e)?;
    if !token(id,128) || !address(own) || !address(peer) { return Err(invalid()); }
    container(e,"iq",CLIENT,&["id","type","from","to"],&[])?;
    if e.attr("id") != Some(id) || e.attr("type") != Some("result")
        || bare(e.attr("from").ok_or_else(invalid)?)? != peer
        || bare(e.attr("to").ok_or_else(invalid)?)? != own { return Err(invalid()); }
    let pubsub = only(e,"pubsub",PUBSUB)?;
    container(pubsub,"pubsub",PUBSUB,&[],&[])?;
    let items = only(pubsub,"items",PUBSUB)?;
    container(items,"items",PUBSUB,&["node"],&[])?;
    if items.attr("node") != Some(node) { return Err(invalid()); }
    let entry = only(items,"item",PUBSUB)?;
    container(entry,"item",PUBSUB,&["id"],&[])?;
    if entry.attr("id") != Some(item) { return Err(invalid()); }
    Ok(entry)
}

/// Fetching a list grants no device trust; the caller must find its enrolled ID.
pub fn devices_query(id: &str, own: &str, peer: &str) -> io::Result<Element> {
    query(id,own,peer,DEVICES,None)
}
/// The caller must preserve other device IDs from its own checked list, or refuse
/// publication. This function does not silently discover or enroll devices.
pub fn devices_publish(id: &str, own: &str, devices: &[u32]) -> io::Result<Element> {
    if devices.is_empty() || devices.len() > MAX_LIST { return Err(invalid()); }
    let mut children = Vec::new();
    for (i,&id) in devices.iter().enumerate() {
        if !device(id) || devices[..i].contains(&id) { return Err(invalid()); }
        children.push(element("device",OMEMO,&[("id",id.to_string())],vec![]));
    }
    publish(id,own,DEVICES,"current",element("devices",OMEMO,&[],children),false)
}
pub fn devices_result(xml: &Element, id: &str, own: &str, peer: &str) -> io::Result<Vec<u32>> {
    let entry = result(xml,id,own,peer,DEVICES,"current")?;
    let devices = only(entry,"devices",OMEMO)?;
    container(devices,"devices",OMEMO,&[],&[])?;
    let mut out = Vec::new();
    for item in devices.children() {
        container(item,"device",OMEMO,&["id"],&[])?;
        if item.children().next().is_some() || out.len() == MAX_LIST { return Err(invalid()); }
        let id = number(item.attr("id").ok_or_else(invalid)?)?;
        if out.contains(&id) { return Err(invalid()); }
        out.push(id);
    }
    Ok(out)
}
pub fn bundle_query(id: &str, own: &str, peer: &str, device_id: u32) -> io::Result<Element> {
    if !device(device_id) { return Err(invalid()); }
    query(id,own,peer,BUNDLES,Some(device_id.to_string()))
}
/// Publishes only the supplied public material. Caller obtains it from a durable
/// checkpoint; this function cannot generate or persist private prekeys.
pub fn bundle_publish(id: &str, own: &str, device_id: u32, bundle: &Bundle) -> io::Result<Element> {
    if !device(device_id) || !device(bundle.signed_prekey_id) || !device(bundle.prekey_id) { return Err(invalid()); }
    let prekeys = element("prekeys",OMEMO,&[],vec![data("pk",&[("id",bundle.prekey_id.to_string())],&bundle.prekey)]);
    let content = element("bundle",OMEMO,&[],vec![
        data("spk",&[("id",bundle.signed_prekey_id.to_string())],&bundle.signed_prekey),
        data("spks",&[],&bundle.signature),data("ik",&[],&bundle.identity),prekeys,
    ]);
    publish(id,own,BUNDLES,&device_id.to_string(),content,true)
}
/// Checks every offered public prekey before returning the first in this bounded
/// fixed-peer profile. `start` performs native signature/session validation after
/// matching the already-enrolled full Ed25519 identity; XML never grants trust.
pub fn bundle_result(xml: &Element, id: &str, own: &str, peer: &str, device_id: u32, identity: &[u8;32]) -> io::Result<Bundle> {
    if !device(device_id) { return Err(invalid()); }
    let entry = result(xml,id,own,peer,BUNDLES,&device_id.to_string())?;
    let b = only(entry,"bundle",OMEMO)?;
    container(b,"bundle",OMEMO,&[],&[])?;
    if b.children().count() != 4 { return Err(invalid()); }
    let ik = leaf(child(b,"ik",OMEMO)?,"ik",&[],&[],32,32)?;
    if ik != identity { return Err(invalid()); }
    let spk = child(b,"spk",OMEMO)?;
    let signed_prekey = leaf(spk,"spk",&["id"],&[],32,32)?.try_into().map_err(|_| invalid())?;
    let signed_prekey_id = number(spk.attr("id").ok_or_else(invalid)?)?;
    let signature = leaf(child(b,"spks",OMEMO)?,"spks",&[],&[],64,64)?.try_into().map_err(|_| invalid())?;
    let prekeys = child(b,"prekeys",OMEMO)?;
    container(prekeys,"prekeys",OMEMO,&[],&[])?;
    let mut ids = Vec::new(); let mut first = None;
    for pk in prekeys.children() {
        if ids.len() == MAX_LIST { return Err(invalid()); }
        let key: [u8;32] = leaf(pk,"pk",&["id"],&[],32,32)?.try_into().map_err(|_| invalid())?;
        let id = number(pk.attr("id").ok_or_else(invalid)?)?;
        if ids.contains(&id) { return Err(invalid()); }
        ids.push(id);
        if first.is_none() { first = Some((id,key)); }
    }
    let (prekey_id,prekey) = first.ok_or_else(invalid)?;
    Ok(Bundle { identity:*identity,signed_prekey,signature,signed_prekey_id,prekey,prekey_id })
}

/// Encode immutable, already-persisted ciphertext. Operation IDs are retry labels,
/// not a peer receipt. Identity and sending time stay out of the clear XML.
pub fn message(wire: &Wire, operation: [u8;16]) -> io::Result<Element> {
    if wire.namespace != OMEMO || !address(&wire.from) || !address(&wire.to)
        || !device(wire.device) || !device(wire.to_device) || operation == [0;16]
        || wire.key.is_empty() || wire.key.len() > 256 || !(16..=8208).contains(&wire.payload.len())
        || wire.sent_at > 253_402_300_799 { return Err(invalid()); }
    let key = data("key",&[("rid",wire.to_device.to_string()),("kex",wire.prekey.to_string())],&wire.key);
    let keys = element("keys",OMEMO,&[("jid",wire.to.clone())],vec![key]);
    let header = element("header",OMEMO,&[("sid",wire.device.to_string())],vec![keys]);
    let encrypted = element("encrypted",OMEMO,&[],vec![header,data("payload",&[],&wire.payload)]);
    Ok(element("message",CLIENT,&[("id",crate::mail_data::hex(&operation)),("from",wire.from.clone()),("to",wire.to.clone()),("type","chat".into())],vec![encrypted]))
}
/// Decode only the one enrolled peer and recipient device. `sent_at` is a zero
/// placeholder: pass the result ONLY to `CheckpointVault::receive_network`, which
/// derives time from authenticated SCE. Do not use arrival time or an XML claim.
/// The enrolled identity is copied from the trusted caller, never from XML.
pub fn decode_message(xml: &Element, binding: Binding<'_>) -> io::Result<Wire> {
    budget(xml)?;
    if !address(binding.peer) || !address(binding.own) || !device(binding.peer_device) || !device(binding.own_device) { return Err(invalid()); }
    container(xml,"message",CLIENT,&["from","to"],&["id","type"])?;
    if xml.attr("id").is_some_and(|id| !token(id,128)) || xml.attr("type").is_some_and(|kind| kind != "chat")
        || bare(xml.attr("from").ok_or_else(invalid)?)? != binding.peer
        || bare(xml.attr("to").ok_or_else(invalid)?)? != binding.own { return Err(invalid()); }
    let encrypted = only(xml,"encrypted",OMEMO)?;
    container(encrypted,"encrypted",OMEMO,&[],&[])?;
    if encrypted.children().count() != 2 { return Err(invalid()); }
    let header = child(encrypted,"header",OMEMO)?;
    container(header,"header",OMEMO,&["sid"],&[])?;
    if number(header.attr("sid").ok_or_else(invalid)?)? != binding.peer_device { return Err(invalid()); }
    let keys = only(header,"keys",OMEMO)?;
    container(keys,"keys",OMEMO,&["jid"],&[])?;
    if keys.attr("jid") != Some(binding.own) { return Err(invalid()); }
    let key = only(keys,"key",OMEMO)?;
    let raw = leaf(key,"key",&["rid"],&["kex"],1,256)?;
    if number(key.attr("rid").ok_or_else(invalid)?)? != binding.own_device { return Err(invalid()); }
    let prekey = match key.attr("kex") { None | Some("false") => false, Some("true") => true, _ => return Err(invalid()) };
    let payload = leaf(child(encrypted,"payload",OMEMO)?,"payload",&[],&[],16,8208)?;
    Ok(Wire { namespace:OMEMO.into(),from:binding.peer.into(),to:binding.own.into(),device:binding.peer_device,to_device:binding.own_device,identity:binding.peer_identity,sent_at:0,key:raw,prekey,payload })
}
