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
    for xml in [
        format!("<s>{}", "<x>".repeat(MAX_DEPTH)),
        format!("<s>{}", "<x/>".repeat(MAX_EVENTS)),
        format!("<s>{}", " ".repeat(MAX_BYTES)),
        "<s><a></b>".into(),
        "<!DOCTYPE x><s/>".into(),
    ] {
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
    peer.write_all(&[first.as_slice(), second.as_slice()].concat())
        .await
        .unwrap();
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
    assert!(
        state.borrow_mut().restart().is_err(),
        "only one authenticated restart"
    );
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

#[test]
fn lifetime_byte_budget_survives_stanza_boundaries_and_restart() {
    let mut guard = Guard::new();
    for b in b"<s><success/>" {
        guard.feed(*b).unwrap();
    }
    guard.restart().unwrap();
    for b in b"<s>" {
        guard.feed(*b).unwrap();
    }
    let xml = format!("{}", format!("{}<x/>", " ".repeat(7000)).repeat(10));
    assert!(xml.bytes().any(|b| guard.feed(b).is_err()));
    assert_eq!(guard.bytes(), MAX_BYTES + 1);
}

#[test]
fn individual_stanza_budget_rejects_before_lifetime_budget() {
    let mut guard = Guard::new();
    let xml = format!("<s><message>{}", "a".repeat(20000));
    assert!(xml.bytes().any(|b| guard.feed(b).is_err()));
    assert!(guard.bytes() < MAX_BYTES);
    assert!(guard.bytes() <= faction_xmpp_probe::guard::MAX_STANZA_BYTES + 4);
}

#[tokio::test(flavor = "current_thread")]
async fn fragmented_restart_and_repeated_fill_do_not_double_count() {
    use tokio::io::AsyncBufReadExt;
    let (mut peer, local) = tokio::io::duplex(32);
    let (mut io, state) = GuardedIo::new(local);
    for phase in [b"<s><success/>".as_slice(), b"<s><x/></s>".as_slice()] {
        for byte in phase {
            peer.write_all(&[*byte]).await.unwrap();
            assert_eq!(io.fill_buf().await.unwrap(), &[*byte]);
            let count = state.borrow().bytes();
            assert_eq!(io.fill_buf().await.unwrap(), &[*byte]);
            assert_eq!(state.borrow().bytes(), count);
            io.consume(1);
        }
        if state.borrow().depth() == 1 {
            state.borrow_mut().restart().unwrap();
        }
    }
    assert_eq!(
        state.borrow().bytes(),
        b"<s><success/>".len() + b"<s><x/></s>".len()
    );
}
