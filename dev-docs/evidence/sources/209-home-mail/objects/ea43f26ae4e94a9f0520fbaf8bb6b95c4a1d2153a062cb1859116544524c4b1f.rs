#![cfg(feature = "native")]
use faction_omemo::sce;
const SENT: u64 = 1_792_886_400;
#[test]
fn network_sce_uses_authenticated_sender_time_without_arrival_metadata() {
    let bytes = sce::encode("alice@home.test", "bob@away.test", SENT, "subject λ", "body <&>").unwrap();
    let (content, timestamp) = sce::decode_network(&bytes, "alice@home.test", "bob@away.test").unwrap();
    assert_eq!(timestamp, SENT);
    assert_eq!(content.subject.as_str(), "subject λ");
    assert_eq!(content.body.as_str(), "body <&>");
    assert!(sce::decode(&bytes, "alice@home.test", "bob@away.test", SENT + 301).is_err());
}
#[test]
fn network_sce_keeps_exact_addresses_and_strict_envelope() {
    let bytes = sce::encode("alice@home.test", "bob@away.test", SENT, "subject", "body").unwrap();
    assert!(sce::decode_network(&bytes, "mallory@home.test", "bob@away.test").is_err());
    let mut bad = bytes.to_vec(); bad.extend_from_slice(b"<time/>");
    assert!(sce::decode_network(&bad, "alice@home.test", "bob@away.test").is_err());
}
