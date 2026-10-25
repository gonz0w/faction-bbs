use faction_xmpp_probe::guard::{Guard, MAX_STANZA_BYTES};
fn main() {
 let mut guard=Guard::new();
 for b in b"<s> " {guard.feed(*b).unwrap();}
 let stanza=format!("<m>{}</m>","a".repeat(MAX_STANZA_BYTES+1-7));
 assert_eq!(stanza.len(), MAX_STANZA_BYTES+1);
 assert!(stanza.bytes().any(|b|guard.feed(b).is_err()), "raw stanza exceeding 16KiB after root whitespace must be rejected");
}
