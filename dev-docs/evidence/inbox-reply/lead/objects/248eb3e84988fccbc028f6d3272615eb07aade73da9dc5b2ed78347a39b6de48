use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};

async fn cycle(lifetime: Duration) -> (Cycle<DuplexStream>, DuplexStream) {
    let (mut peer, io) = tokio::io::duplex(65536);
    peer.write_all(b"<stream:stream xmlns='jabber:client' xmlns:stream='http://etherx.jabber.org/streams' from='home.test' version='1.0'><stream:features/>").await.unwrap();
    let (io, state) = GuardedIo::new(io);
    let pending = initiate_stream(io, CLIENT_NS, header("home.test"), timeouts())
        .await
        .unwrap();
    let (_, stream) = pending.recv_features::<Element>().await.unwrap();
    let mut outgoing_header = [0; 4096];
    assert!(peer.read(&mut outgoing_header).await.unwrap() > 0);
    (
        Cycle {
            stream: Some(stream),
            state,
            deadline: Instant::now() + lifetime,
        },
        peer,
    )
}

#[tokio::test(flavor = "current_thread")]
async fn successive_reads_and_writes_share_original_deadline() {
    let (mut cycle, mut peer) = cycle(Duration::from_millis(30)).await;
    let stanza = Element::builder("message", CLIENT_NS).build();
    cycle.send(&stanza).await.unwrap();
    let mut sent = [0; 256];
    assert!(peer.read(&mut sent).await.unwrap() > 0);
    tokio::time::sleep(Duration::from_millis(35)).await;
    assert_eq!(cycle.next().await, Err(Error::Timeout));
    assert_eq!(
        peer.read(&mut sent).await.unwrap(),
        0,
        "deadline drops socket"
    );
    assert_eq!(cycle.send(&stanza).await, Err(Error::Lost));
}

#[tokio::test(flavor = "current_thread")]
async fn active_read_deadline_drops_transport() {
    let (mut cycle, mut peer) = cycle(Duration::from_millis(10)).await;
    assert_eq!(cycle.next().await, Err(Error::Timeout));
    assert_eq!(peer.read(&mut [0; 1]).await.unwrap(), 0);
}

#[tokio::test(flavor = "current_thread")]
async fn dropped_pending_read_cancels_and_cannot_reuse_partial_stream() {
    let (mut cycle, mut peer) = cycle(Duration::from_secs(1)).await;
    assert!(
        tokio::time::timeout(Duration::from_millis(5), cycle.next())
            .await
            .is_err()
    );
    assert_eq!(peer.read(&mut [0; 1]).await.unwrap(), 0);
    assert_eq!(cycle.next().await, Err(Error::Lost));
}

#[tokio::test(flavor = "current_thread")]
async fn rejected_output_writes_zero_bytes_and_closes_transport() {
    let (mut cycle, mut peer) = cycle(Duration::from_secs(1)).await;
    let stanza = Element::builder("message", CLIENT_NS)
        .append("&".repeat(MAX_OUTPUT_BYTES))
        .build();
    assert_eq!(cycle.send(&stanza).await, Err(Error::OutputLimit));
    assert_eq!(peer.read(&mut [0; 1]).await.unwrap(), 0);
}

#[tokio::test(flavor = "current_thread")]
async fn receive_guard_rejection_is_typed_and_latched() {
    let (mut cycle, mut peer) = cycle(Duration::from_secs(1)).await;
    peer.write_all(format!("<message>{}</message>", "x".repeat(17000)).as_bytes())
        .await
        .unwrap();
    assert_eq!(cycle.next().await, Err(Error::XmlLimit));
    assert_eq!(cycle.next().await, Err(Error::Lost));
}

#[test]
fn binding_requires_exact_identity_and_no_extra_payload() {
    let wanted = "alice@home.test/device";
    let good = "<iq xmlns='jabber:client' type='result' id='bind-209'><bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'><jid>alice@home.test/device</jid></bind></iq>";
    assert_eq!(validate_binding(&good.parse().unwrap(), wanted), Ok(()));
    for bad in [
        good.replace("bind-209", "other"),
        good.replace("alice@home.test/device", "mallory@home.test/device"),
        good.replace("/device", "/else"),
        good.replace("</bind>", "<jid>alice@home.test/device</jid></bind>"),
        good.replace("</iq>", "<extra/></iq>"),
        good.replace("</jid>", "<extra/></jid>"),
    ] {
        assert_eq!(
            validate_binding(&bad.parse().unwrap(), wanted),
            Err(Error::Bind)
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn actual_xso_encoding_cannot_exceed_cap_via_empty_children() {
    let (mut cycle, mut peer) = cycle(Duration::from_secs(1)).await;
    let mut stanza = Element::builder("message", CLIENT_NS).build();
    for _ in 0..80 {
        stanza.append_child(Element::builder("x", CLIENT_NS).build());
    }
    let mut shorthand = Vec::new();
    stanza.write_to(&mut shorthand).unwrap();
    stanza.append_text("x".repeat(MAX_OUTPUT_BYTES - shorthand.len()));
    assert_eq!(cycle.send(&stanza).await, Err(Error::OutputLimit));
    assert_eq!(peer.read(&mut [0; 1]).await.unwrap(), 0);
}
