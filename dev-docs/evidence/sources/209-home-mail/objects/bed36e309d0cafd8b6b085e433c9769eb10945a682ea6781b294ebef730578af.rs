//! Strict, bounded single-peer OMEMO-2 XML profile for manual home mail.
//! Trust always comes from the caller's enrolled Ed25519 binding. These codecs
//! neither authenticate ciphertext nor advance native cryptographic state.
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

pub fn devices_query(_id: &str, _own: &str, _peer: &str) -> io::Result<Element> { Err(invalid()) }
pub fn devices_publish(_id: &str, _own: &str, _devices: &[u32]) -> io::Result<Element> { Err(invalid()) }
pub fn devices_result(_xml: &Element, _id: &str, _own: &str, _peer: &str) -> io::Result<Vec<u32>> { Err(invalid()) }
pub fn bundle_query(_id: &str, _own: &str, _peer: &str, _device: u32) -> io::Result<Element> { Err(invalid()) }
pub fn bundle_publish(_id: &str, _own: &str, _device: u32, _bundle: &Bundle) -> io::Result<Element> { Err(invalid()) }
pub fn bundle_result(_xml: &Element, _id: &str, _own: &str, _peer: &str, _device: u32, _identity: &[u8; 32]) -> io::Result<Bundle> { Err(invalid()) }
pub fn message(_wire: &Wire, _operation: [u8; 16]) -> io::Result<Element> { Err(invalid()) }
pub fn decode_message(_xml: &Element, _binding: Binding<'_>) -> io::Result<Wire> { Err(invalid()) }

#[cfg(test)]
mod tests {
    use super::*;
    const OWN: &str = "alice@home.test";
    const PEER: &str = "bob@away.test";
    fn parse(s: &str) -> Element { s.parse().unwrap() }
    fn binding() -> Binding<'static> {
        Binding { own: OWN, own_device: 101, peer: PEER, peer_device: 202, peer_identity: [7; 32] }
    }
    fn wire() -> Wire {
        Wire { namespace: OMEMO.into(), from: PEER.into(), to: OWN.into(), device: 202,
            to_device: 101, identity: [7; 32], sent_at: 1_792_944_000,
            key: vec![1, 2, 3, 4], prekey: true, payload: vec![5; 32] }
    }
    fn bundle() -> Bundle {
        Bundle { identity: [7; 32], signed_prekey: [8; 32], signature: [9; 64],
            signed_prekey_id: 3, prekey: [10; 32], prekey_id: 4 }
    }
    fn independent_message() -> String {
        format!("<message xmlns='jabber:client' from='{PEER}/reference' to='{OWN}/local' type='chat' id='independent'><encrypted xmlns='{OMEMO}'><header sid='202'><keys jid='{OWN}'><key rid='101' kex='true'>AQIDBA==</key></keys></header><payload>{}</payload></encrypted></message>", STANDARD.encode([5;32]))
    }
    fn iq(node: &str, item: &str, content: &str) -> String {
        format!("<iq xmlns='{CLIENT}' from='{PEER}' to='{OWN}/local' type='result' id='q1'><pubsub xmlns='{PUBSUB}'><items node='{node}'><item id='{item}'>{content}</item></items></pubsub></iq>")
    }
    fn independent_bundle() -> String {
        format!("<bundle xmlns='{OMEMO}'><spk id='3'>{}</spk><spks>{}</spks><ik>{}</ik><prekeys><pk id='4'>{}</pk><pk id='5'>{}</pk></prekeys></bundle>", STANDARD.encode([8;32]), STANDARD.encode([9;64]), STANDARD.encode([7;32]), STANDARD.encode([10;32]), STANDARD.encode([11;32]))
    }
    fn xml(e: &Element) -> String {
        let mut b = Vec::new(); e.write_to(&mut b).unwrap(); String::from_utf8(b).unwrap()
    }
    #[test]
    fn mail_wire_publishes_standard_nodes_and_public_material_only() {
        let q = devices_query("q1", OWN, PEER).unwrap();
        assert_eq!(q.attr("type"), Some("get")); assert_eq!(q.attr("to"), Some(PEER));
        assert!(xml(&q).contains(DEVICES));
        let d = devices_publish("q1", OWN, &[101]).unwrap();
        assert!(xml(&d).contains("current")); assert!(xml(&d).contains("pubsub#access_model"));
        let b = bundle_publish("q2", OWN, 101, &bundle()).unwrap();
        let s = xml(&b);
        assert!(s.contains(BUNDLES)); assert!(s.contains("pubsub#max_items"));
        assert!(s.contains(&STANDARD.encode([7;32]))); assert!(s.contains("<spks"));
        let query = bundle_query("q1", OWN, PEER, 202).unwrap();
        assert!(xml(&query).contains("202"));
    }
    #[test]
    fn mail_wire_reads_independent_standard_devices_and_bundle() {
        let devices = iq(DEVICES, "current", &format!("<devices xmlns='{OMEMO}'><device id='202'/><device id='303'/></devices>"));
        assert_eq!(devices_result(&parse(&devices), "q1", OWN, PEER).unwrap(), [202,303]);
        let result = iq(BUNDLES, "202", &independent_bundle());
        let got = bundle_result(&parse(&result), "q1", OWN, PEER, 202, &[7;32]).unwrap();
        assert_eq!(got.identity, [7;32]); assert_eq!(got.prekey_id, 4); assert_eq!(got.prekey, [10;32]);
        assert_eq!(got.signature, [9;64]); assert_eq!(got.signed_prekey_id, 3);
    }
    #[test]
    fn mail_wire_message_roundtrip_excludes_clear_identity_and_time() {
        let expected = wire();
        let encoded = message(&expected, [2;16]).unwrap();
        let text = xml(&encoded);
        assert!(!text.contains("sent_at")); assert!(!text.contains("identity"));
        assert!(!text.contains(&expected.sent_at.to_string()));
        let got = decode_message(&encoded, binding()).unwrap();
        assert_eq!(got.sent_at, 0); // ONLY receive_network derives authenticated SCE time.
        let mut normalized = expected; normalized.sent_at = 0;
        assert_eq!(got, normalized);
    }
    #[test]
    fn mail_wire_reads_independent_message_and_optional_false_kex() {
        let raw = independent_message();
        let got = decode_message(&parse(&raw), binding()).unwrap();
        assert_eq!(got.key, [1,2,3,4]); assert_eq!(got.identity, [7;32]); assert!(got.prekey);
        for kex in ["", " kex='false'"] {
            let got = decode_message(&parse(&raw.replace(" kex='true'", kex)), binding()).unwrap();
            assert!(!got.prekey);
        }
    }
    #[test]
    fn mail_wire_rejects_substitution_and_shape_confusion() {
        let raw = independent_message();
        for (before,after) in [
            ("bob@away.test/reference","mallory@away.test/reference"),
            ("alice@home.test/local","alice@other.test/local"),
            ("sid='202'","sid='203'"), ("rid='101'","rid='102'"),
            ("jid='alice@home.test'","jid='alice@home.test/other'"),
            ("sid='202'","sid='0202'"), ("sid='202'","sid='2147483648'"),
            ("kex='true'","kex='yes'"), ("urn:xmpp:omemo:2","urn:xmpp:omemo:1"),
            ("<header ","<header unknown='x' "), ("</header>","<extra/></header>"),
            ("</keys>","<key rid='101'>AQIDBA==</key></keys>"),
            ("</encrypted>","<header sid='202'/></encrypted>"),
            ("</message>","<body>plaintext fallback</body></message>"),
            ("<encrypted ","text<encrypted "), ("AQIDBA==","AQIDBA"),
            ("AQIDBA==","AQIDBB=="), ("AQIDBA=="," AQIDBA=="),
            ("<payload>","<payload unknown='x'>"),
            ("AQIDBA==","<extra/>AQIDBA=="),
            ("type='chat'","type='groupchat'"),
            ("id='independent'","id='bad&#x1b;'"),
        ] {
            let changed = raw.replace(before,after);
            // XML controls rejected by parser before a DOM is formed also satisfy refusal.
            if let Ok(parsed) = changed.parse::<Element>() {
                assert!(decode_message(&parsed, binding()).is_err(), "accepted mutation {before} -> {after}");
            }
        }
    }
    #[test]
    fn mail_wire_rejects_wrong_iq_owner_id_node_and_extra_children() {
        let raw = iq(DEVICES,"current", &format!("<devices xmlns='{OMEMO}'><device id='202'/></devices>"));
        for (before,after) in [
            ("id='q1'","id='q2'"), ("type='result'","type='set'"),
            ("bob@away.test","eve@away.test"), ("alice@home.test/local","alice@other.test/local"),
            (DEVICES,BUNDLES), ("id='current'","id='old'"),
            ("</devices>","<device id='202'/></devices>"),
            ("id='202'","id='0'"), ("id='202'","id='+202'"),
            ("<devices ","<devices extra='yes' "),
            ("</items>","<item id='other'/></items>"),
            ("</iq>","<extra/></iq>"),
            ("<pubsub ","garbage<pubsub "),
        ] { assert!(devices_result(&parse(&raw.replace(before,after)),"q1",OWN,PEER).is_err(), "accepted {before} -> {after}"); }
    }
    #[test]
    fn mail_wire_bundle_refuses_changed_identity_duplicate_or_unknown_keys() {
        let raw = iq(BUNDLES, "202", &independent_bundle());
        assert!(bundle_result(&parse(&raw),"q1",OWN,PEER,202,&[99;32]).is_err());
        for (before,after) in [
            ("id='202'","id='203'"), ("id='5'","id='4'"),
            ("<ik>","<ik extra='x'>"), ("</bundle>","<unknown/></bundle>"),
            ("<prekeys>","<prekeys>text"),
            ("<pk id='5'>","<pk id='0'>"), ("<spk id='3'>","<spk id='03'>"),
            ("<spks>","<spks xmlns='wrong'>"),
        ] { assert!(bundle_result(&parse(&raw.replace(before,after)),"q1",OWN,PEER,202,&[7;32]).is_err(), "accepted {before} -> {after}"); }
    }
    #[test]
    fn mail_wire_enforces_input_and_encoded_bounds() {
        let mut w = wire(); w.payload = vec![5;8209]; assert!(message(&w,[1;16]).is_err());
        w = wire(); w.key = vec![2;257]; assert!(message(&w,[1;16]).is_err());
        w = wire(); w.from = "Bob@away.test".into(); assert!(message(&w,[1;16]).is_err());
        assert!(message(&wire(),[0;16]).is_err());
        assert!(devices_query("bad\u{1b}",OWN,PEER).is_err());
        assert!(devices_publish("q1",OWN,&[101,101]).is_err());
        assert!(devices_publish("q1",OWN,&(1..=33).collect::<Vec<_>>()).is_err());
        let raw = independent_message().replace("AQIDBA==", &STANDARD.encode([1;257]));
        assert!(decode_message(&parse(&raw),binding()).is_err());
        let raw = independent_message().replace(&STANDARD.encode([5;32]), &STANDARD.encode([5;8209]));
        assert!(decode_message(&parse(&raw),binding()).is_err());
        let devices: String = (1..=33).map(|id|format!("<device id='{id}'/>")).collect();
        let raw = iq(DEVICES,"current",&format!("<devices xmlns='{OMEMO}'>{devices}</devices>"));
        assert!(devices_result(&parse(&raw),"q1",OWN,PEER).is_err());
    }
    #[test]
    fn mail_wire_full_jid_is_scoped_and_resources_are_inert() {
        let raw = independent_message();
        for resource in ["", "x/y", "x&#x9;y", "x y", "x@else.test", "../x"] {
            let altered = raw.replace("/reference", &format!("/{resource}"));
            assert!(decode_message(&parse(&altered),binding()).is_err(), "accepted resource {resource}");
        }
        let bare = raw.replace("/reference", "").replace("/local", "");
        assert!(decode_message(&parse(&bare),binding()).is_ok());
    }
}
