use faction_xmpp_probe::guard::{Guard, GuardedIo, MAX_BYTES, MAX_DEPTH, MAX_EVENTS};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[test]
fn bounded_incremental_xml_accepts_fragmented_stream() {
    let mut guard = Guard::new();
    for b in b"<s:stream xmlns:s='http://etherx.jabber.org/streams'><message><encrypted xmlns='urn:xmpp:omemo:2'>YWJj</encrypted></message>" {
        guard.feed(*b).unwrap();
    }
    assert_eq!(guard.depth(), 1);
    assert!(guard.events() > 0);
}

#[test]
fn rejects_depth_events_bytes_malformed_and_dtd() {
    for xml in [format!("<s>{}", "<x>".repeat(MAX_DEPTH)), format!("<s>{}", "<x/>".repeat(MAX_EVENTS)), format!("<s>{}", " ".repeat(MAX_BYTES)), "<s><a></b>".into(), "<!DOCTYPE x><s/>".into()] {
        let mut guard = Guard::new();
        assert!(xml.bytes().any(|b| guard.feed(b).is_err()));
        assert!(guard.feed(b' ').is_err(), "rejection must stay latched");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn coalesced_restart_preserves_unread_bytes() {
    let first = b"<s><success/>";
    let second = b"<s><message/></s>";
    let (mut peer, local) = tokio::io::duplex(1024);
    peer.write_all(&[first.as_slice(), second.as_slice()].concat()).await.unwrap();
    let (mut io, state) = GuardedIo::new(local);
    let mut buf = [0; 512];
    let mut got = Vec::new();
    while got.len() < first.len() {
        let n = io.read(&mut buf).await.unwrap();
        assert_eq!(n, 1, "SASL success must be an exact read barrier");
        got.extend_from_slice(&buf[..n]);
    }
    state.borrow_mut().restart().unwrap();
    while got.len() < first.len() + second.len() {
        let n = io.read(&mut buf).await.unwrap();
        got.extend_from_slice(&buf[..n]);
    }
    assert_eq!(got, [first.as_slice(), second.as_slice()].concat());
    assert_eq!(state.borrow().bytes(), got.len());
    assert!(state.borrow_mut().restart().is_err(), "only one authenticated restart");
}

#[tokio::test(flavor = "current_thread")]
async fn guard_rejects_before_dom_consumer_observes_bad_byte() {
    let (mut peer, local) = tokio::io::duplex(1024);
    let xml = format!("<s>{}", "<x>".repeat(MAX_DEPTH));
    peer.write_all(xml.as_bytes()).await.unwrap();
    let (mut io, _) = GuardedIo::new(local);
    let mut received = Vec::new();
    assert!(io.read_to_end(&mut received).await.is_err());
    assert!(received.len() < xml.len());
}
