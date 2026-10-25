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
    let xml = format!("{}<x/>", " ".repeat(7000)).repeat(10);
    assert!(xml.bytes().any(|b| guard.feed(b).is_err()));
    assert_eq!(guard.bytes(), MAX_BYTES);
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

#[tokio::test(flavor = "current_thread")]
async fn parser_lookahead_unconsumed_byte_is_replayed_after_trusted_restart() {
    use tokio::io::AsyncBufReadExt;
    let (mut peer, local) = tokio::io::duplex(64);
    let (mut io, state) = GuardedIo::new(local);
    peer.write_all(b"<s><success/><s><x/></s>").await.unwrap();
    let mut first = vec![0; b"<s><success/>".len()];
    io.read_exact(&mut first).await.unwrap();
    // rxml's AsyncReader fills before draining its zero-byte EndElement event.
    // That peeks the next root's '<' without calling consume.
    assert_eq!(io.fill_buf().await.unwrap(), b"<");
    let physical_reads = state.borrow().bytes();
    state.borrow_mut().restart().unwrap();
    assert_eq!(io.fill_buf().await.unwrap(), b"<");
    assert_eq!(state.borrow().bytes(), physical_reads);
    let mut second = vec![0; b"<s><x/></s>".len()];
    io.read_exact(&mut second).await.unwrap();
    assert_eq!(second, b"<s><x/></s>");
    assert_eq!(state.borrow().depth(), 0);
}

#[test]
fn physical_byte_and_event_counters_never_exceed_fixed_caps() {
    let mut guard = Guard::new();
    let xml = format!("<s>{}", format!("{}<x/>", " ".repeat(7000)).repeat(10));
    assert!(xml.bytes().any(|b| guard.feed(b).is_err()));
    assert!(guard.bytes() <= MAX_BYTES);
    let mut guard = Guard::new();
    assert!(format!("<s>{}", "<x/>".repeat(MAX_EVENTS))
        .bytes()
        .any(|b| guard.feed(b).is_err()));
    assert!(guard.events() <= MAX_EVENTS);
}

#[test]
fn root_whitespace_never_erases_the_next_stanzas_opening_byte() {
    use faction_xmpp_probe::guard::{Rejection, MAX_STANZA_BYTES};
    for whitespace in [" ", "   ", "\n\t"] {
        let mut guard = Guard::new();
        for byte in format!("<s>{whitespace}").bytes() {
            guard.feed(byte).unwrap();
        }
        let oversized = format!("<m>{}</m>", "a".repeat(MAX_STANZA_BYTES + 1 - 7));
        assert_eq!(oversized.len(), MAX_STANZA_BYTES + 1);
        assert!(oversized.bytes().any(|byte| guard.feed(byte).is_err()),
            "root whitespace must not allow a stanza larger than 16384 raw bytes");
        assert_eq!(guard.rejection(), Some(Rejection::Limit));

        // Inter-stanza whitespace consumes the same conservative bound. A
        // complete stanza that fits the remaining budget must still succeed.
        let mut guard = Guard::new();
        for byte in format!("<s>{whitespace}").bytes() {
            guard.feed(byte).unwrap();
        }
        let fitting = format!("<m>{}</m>", "a".repeat(MAX_STANZA_BYTES - whitespace.len() - 7));
        assert_eq!(fitting.len() + whitespace.len(), MAX_STANZA_BYTES);
        for byte in fitting.bytes() {
            guard.feed(byte).unwrap();
        }
        assert_eq!(guard.depth(), 1);
        assert_eq!(guard.rejection(), None);
    }
}
