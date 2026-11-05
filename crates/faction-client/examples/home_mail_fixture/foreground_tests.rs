use super::foreground::{Channel, Outcome, Route, attempt};
use faction_client::{
    mail_checkpoint::{CheckpointVault, Delivery, Direction},
    mail_data::{Data, hex},
    mail_vault::Store,
    mail_wire::{self, OMEMO},
};
use faction_xmpp::{Element, Error};
use std::{
    collections::VecDeque,
    fs::{self, DirBuilder},
    os::unix::fs::DirBuilderExt,
    path::PathBuf,
};

const NOW: u64 = 1_792_886_400;
const ALICE: &str = "alice@sender.test";
const BOB: &str = "bob@receiver.test";
const BOB_FULL: &str = "bob@receiver.test/faction202";
const BOB_OTHER: &str = "bob@receiver.test/faction203";
const SUBJECT: &str = "Synthetic subject 347";
const BODY: &str = "Synthetic body 347";
const PASS: &[u8] = b"synthetic local test password";

struct Owned(PathBuf);
impl Owned {
    fn new() -> Self {
        let mut nonce = [0; 16];
        getrandom::fill(&mut nonce).unwrap();
        let path = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("faction347-controlled-{}", hex(&nonce)));
        DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }

    fn create(&self, name: &str, account: &str, device: u32) -> CheckpointVault {
        CheckpointVault::create(
            Store::at(&self.0.join(name), true).unwrap(),
            Data::new(account.into()).unwrap(),
            PASS,
            device,
            NOW,
        )
        .unwrap()
        .0
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

struct Pair {
    sender: CheckpointVault,
    receiver: CheckpointVault,
    delivery: Delivery,
    // Release both vault locks before removing the owned directory.
    owned: Owned,
}
impl Pair {
    fn new(unapproved: bool) -> Self {
        let owned = Owned::new();
        let mut sender = owned.create("sender", ALICE, 101);
        let mut receiver = owned.create("receiver", BOB, 202);
        enroll(&mut sender, &receiver, BOB, 202);
        if unapproved {
            let impostor = owned.create("unapproved", ALICE, 101);
            enroll(&mut receiver, &impostor, ALICE, 101);
        } else {
            enroll(&mut receiver, &sender, ALICE, 101);
        }
        receiver.enable_archive(NOW).unwrap();
        receiver.enable_read_tracking().unwrap();
        sender.start(&receiver.bundle().unwrap(), NOW).unwrap();
        sender.queue([7; 16], SUBJECT, BODY, NOW, NOW).unwrap();
        drop(sender);
        let sender = CheckpointVault::unlock(
            Store::at(&owned.0.join("sender"), false).unwrap(),
            PASS,
            false,
        )
        .unwrap();
        let delivery = sender.pending_deliveries(NOW).unwrap().pop().unwrap();
        Self {
            owned,
            sender,
            receiver,
            delivery,
        }
    }

    fn assert_pending(&self, before: &[u8]) {
        let pending = self.sender.pending_deliveries(NOW).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].operation, self.delivery.operation);
        assert_eq!(pending[0].wire, self.delivery.wire);
        assert_eq!(self.delivery.operation, [7; 16]);
        assert_eq!(
            fs::read(self.owned.0.join("sender/mail.fkv")).unwrap(),
            before
        );
    }
}

fn enroll(local: &mut CheckpointVault, remote: &CheckpointVault, account: &str, device: u32) {
    let fingerprint = format!("ed25519:{}", hex(remote.inspection().unwrap().identity));
    local.enroll(account, device, &fingerprint, NOW).unwrap();
}

type Relay = fn(Element) -> Result<Element, Error>;

struct Controlled {
    bound: &'static str,
    relay: Relay,
    sent: Vec<Element>,
    arrivals: VecDeque<Element>,
    send_error: Option<Error>,
    received: usize,
}
impl Controlled {
    fn new(bound: &'static str, relay: Relay) -> Self {
        Self {
            bound,
            relay,
            sent: vec![],
            arrivals: VecDeque::new(),
            send_error: None,
            received: 0,
        }
    }
}
impl Channel for Controlled {
    fn receiver_binding(&self) -> &str {
        self.bound
    }

    async fn send(&mut self, stanza: &Element) -> Result<(), Error> {
        self.sent.push(stanza.clone());
        self.send_error.map_or(Ok(()), Err)
    }

    async fn next(&mut self) -> Result<Element, Error> {
        self.received += 1;
        assert_eq!(self.sent.len(), 1);
        if let Some(frame) = self.arrivals.pop_front() {
            return Ok(frame);
        }
        (self.relay)(self.sent[0].clone())
    }
}

#[tokio::test]
async fn foreground_bare_and_full_change_only_outer_route() {
    for (route, destination) in [(Route::Bare, BOB), (Route::Full, BOB_FULL)] {
        let mut pair = Pair::new(false);
        let before = fs::read(pair.owned.0.join("sender/mail.fkv")).unwrap();
        let cursor = pair.receiver.archive_checkpoint(NOW).unwrap();
        let mut channel = Controlled::new(BOB_FULL, Ok);
        let mut observed = Vec::new();
        let outcome = attempt(
            &mut channel,
            &mut pair.receiver,
            &pair.delivery,
            route,
            NOW,
            |frame| {
                observed.push(frame.clone());
                Ok(())
            },
        )
        .await
        .unwrap();
        assert_eq!(outcome, Outcome::Accepted);
        assert_eq!(channel.receiver_binding(), BOB_FULL);
        assert_eq!(channel.received, 1);
        assert_eq!(channel.sent.len(), 1); // Neither side sent presence, MAM or a receipt.
        assert_eq!(observed, channel.sent);
        let mut routed = channel.sent.pop().unwrap();
        assert!(routed.is("message", "jabber:client"));
        assert_eq!(routed.attr("to"), Some(destination));
        assert_eq!(routed.attr("id"), Some("07070707070707070707070707070707"));
        let keys = routed
            .get_child("encrypted", OMEMO)
            .unwrap()
            .get_child("header", OMEMO)
            .unwrap()
            .get_child("keys", OMEMO)
            .unwrap();
        assert_eq!(keys.attr("jid"), Some(BOB));
        assert_eq!(
            keys.get_child("key", OMEMO).unwrap().attr("rid"),
            Some("202")
        );
        routed.set_attr("".into(), "to".try_into().unwrap(), BOB);
        assert_eq!(
            routed,
            mail_wire::message(&pair.delivery.wire, pair.delivery.operation).unwrap()
        );

        let records = pair.receiver.history().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].direction, Direction::Received);
        assert_eq!(records[0].subject.as_str(), SUBJECT);
        assert_eq!(records[0].body.as_str(), BODY);
        assert_eq!(records[0].sent_at, NOW);
        assert_eq!(pair.receiver.read_summary().unwrap().unread, Some(1));
        assert_eq!(pair.receiver.archive_checkpoint(NOW).unwrap(), cursor);
        pair.assert_pending(&before);
    }
}

fn outer_to(mut frame: Element, to: &str) -> Element {
    frame.set_attr("".into(), "to".try_into().unwrap(), to);
    frame
}

fn wrong_recipient(mut frame: Element) -> Result<Element, Error> {
    frame
        .get_child_mut("encrypted", OMEMO)
        .unwrap()
        .get_child_mut("header", OMEMO)
        .unwrap()
        .get_child_mut("keys", OMEMO)
        .unwrap()
        .get_child_mut("key", OMEMO)
        .unwrap()
        .set_attr("".into(), "rid".try_into().unwrap(), "203");
    Ok(frame)
}

fn tampered_payload(mut frame: Element) -> Result<Element, Error> {
    let text = frame
        .get_child_mut("encrypted", OMEMO)
        .unwrap()
        .get_child_mut("payload", OMEMO)
        .unwrap()
        .texts_mut()
        .next()
        .unwrap();
    text.replace_range(..1, if text.starts_with('A') { "B" } else { "A" });
    Ok(frame)
}

fn own_stanza_id(mut frame: Element) -> Result<Element, Error> {
    frame.append_child(
        "<stanza-id xmlns='urn:xmpp:sid:0' by='bob@receiver.test' id='fixture-position'/>"
            .parse()
            .unwrap(),
    );
    Ok(frame)
}

#[tokio::test]
async fn foreground_rejections_leave_history_cursor_and_pending_unchanged() {
    let mut pair = Pair::new(false);
    let sender_before = fs::read(pair.owned.0.join("sender/mail.fkv")).unwrap();
    let receiver_path = pair.owned.0.join("receiver/mail.fkv");
    let receiver_before = fs::read(&receiver_path).unwrap();
    let cursor = pair.receiver.archive_checkpoint(NOW).unwrap();
    let cases: [(Route, &str, Relay, Outcome); 8] = [
        (
            Route::Full,
            BOB_FULL,
            |e| Ok(outer_to(e, BOB_OTHER)),
            Outcome::Destination,
        ),
        (Route::Full, BOB_OTHER, Ok, Outcome::Destination),
        (
            Route::Full,
            BOB_FULL,
            |e| Ok(outer_to(e, "other@receiver.test/faction202")),
            Outcome::Destination,
        ),
        (
            Route::Missing,
            BOB_FULL,
            |e| Ok(outer_to(e, BOB_FULL)),
            Outcome::Destination,
        ),
        (Route::Full, BOB_FULL, wrong_recipient, Outcome::Codec),
        (
            Route::Full,
            BOB_FULL,
            tampered_payload,
            Outcome::DifferentCiphertext,
        ),
        (Route::Full, BOB_FULL, own_stanza_id, Outcome::Codec),
        (
            Route::Full,
            BOB_FULL,
            |_| Err(Error::Lost),
            Outcome::Receive(Error::Lost),
        ),
    ];
    for (route, bound, relay, expected) in cases {
        let mut channel = Controlled::new(bound, relay);
        let mut observed = 0;
        let outcome = attempt(
            &mut channel,
            &mut pair.receiver,
            &pair.delivery,
            route,
            NOW,
            |_| {
                observed += 1;
                Ok(())
            },
        )
        .await
        .unwrap();
        assert_eq!(outcome, expected);
        assert_eq!(channel.sent.len(), 1);
        if route == Route::Missing {
            assert_eq!(
                channel.sent[0].attr("to"),
                Some("bob@receiver.test/faction0")
            );
        }
        assert_eq!(
            observed,
            usize::from(expected != Outcome::Receive(Error::Lost))
        );
        assert!(pair.receiver.history().unwrap().is_empty());
        assert_eq!(pair.receiver.archive_checkpoint(NOW).unwrap(), cursor);
        assert_eq!(fs::read(&receiver_path).unwrap(), receiver_before);
        pair.assert_pending(&sender_before);
    }

    let mut pair = Pair::new(true);
    let sender_before = fs::read(pair.owned.0.join("sender/mail.fkv")).unwrap();
    let receiver_path = pair.owned.0.join("receiver/mail.fkv");
    let receiver_before = fs::read(&receiver_path).unwrap();
    let cursor = pair.receiver.archive_checkpoint(NOW).unwrap();
    let mut channel = Controlled::new(BOB_FULL, Ok);
    assert_eq!(
        attempt(
            &mut channel,
            &mut pair.receiver,
            &pair.delivery,
            Route::Full,
            NOW,
            |_| Ok(())
        )
        .await
        .unwrap(),
        Outcome::AcceptanceFailed
    );

    channel = Controlled::new(BOB_FULL, Ok);
    channel.send_error = Some(Error::Lost);
    assert_eq!(
        attempt(
            &mut channel,
            &mut pair.receiver,
            &pair.delivery,
            Route::Full,
            NOW,
            |_| {
                panic!("a failed write cannot yield an arrival");
            }
        )
        .await
        .unwrap(),
        Outcome::Send(Error::Lost)
    );
    assert_eq!(channel.received, 0);
    assert!(pair.receiver.history().unwrap().is_empty());
    assert_eq!(pair.receiver.read_summary().unwrap().unread, Some(0));
    assert_eq!(pair.receiver.archive_checkpoint(NOW).unwrap(), cursor);
    assert_eq!(fs::read(receiver_path).unwrap(), receiver_before);
    pair.assert_pending(&sender_before);
}

fn alternate(pair: &mut Pair) -> Element {
    let operation = [8; 16];
    let wire = pair
        .sender
        .queue(operation, "Alternate subject", "Alternate body", NOW, NOW)
        .unwrap();
    outer_to(mail_wire::message(&wire, operation).unwrap(), BOB_FULL)
}

#[tokio::test]
async fn foreground_correlation_ignores_an_unrelated_valid_arrival_before_the_target() {
    let mut pair = Pair::new(false);
    let alternate = alternate(&mut pair);
    let mut channel = Controlled::new(BOB_FULL, Ok);
    channel.arrivals.push_back(alternate.clone());
    let mut observed = Vec::new();
    let outcome = attempt(
        &mut channel,
        &mut pair.receiver,
        &pair.delivery,
        Route::Full,
        NOW,
        |frame| {
            observed.push(frame.clone());
            Ok(())
        },
    )
    .await
    .unwrap();
    assert_eq!(outcome, Outcome::Accepted);
    let history = pair.receiver.history().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].subject.as_str(), SUBJECT);
    assert_eq!(observed, [alternate, channel.sent[0].clone()]);
    assert_eq!(channel.received, 2);
}

#[tokio::test]
async fn foreground_correlation_rejects_different_valid_ciphertext_with_the_target_id() {
    let mut pair = Pair::new(false);
    let mut alternate = alternate(&mut pair);
    alternate.set_attr(
        "".into(),
        "id".try_into().unwrap(),
        hex(&pair.delivery.operation),
    );
    let before = fs::read(pair.owned.0.join("receiver/mail.fkv")).unwrap();
    let mut channel = Controlled::new(BOB_FULL, Ok);
    channel.arrivals.push_back(alternate.clone());
    let mut observed = Vec::new();
    let outcome = attempt(
        &mut channel,
        &mut pair.receiver,
        &pair.delivery,
        Route::Full,
        NOW,
        |frame| {
            observed.push(frame.clone());
            Ok(())
        },
    )
    .await
    .unwrap();
    assert!(!matches!(outcome, Outcome::Accepted | Outcome::Duplicate));
    assert_eq!(observed, [alternate]);
    assert!(pair.receiver.history().unwrap().is_empty());
    assert_eq!(
        fs::read(pair.owned.0.join("receiver/mail.fkv")).unwrap(),
        before
    );
}

#[test]
fn foreground_report_distinguishes_no_attempt_completed_and_unconfirmed_write() {
    use super::foreground::record_outcome;
    for (outcome, expected) in [
        (None, "local_write=not_attempted\n"),
        (
            Some(Outcome::Send(Error::Lost)),
            "local_write=unconfirmed\noutcome=Send(Lost)\n",
        ),
        (
            Some(Outcome::Receive(Error::Lost)),
            "local_write=completed\noutcome=Receive(Lost)\n",
        ),
        (
            Some(Outcome::AcceptanceFailed),
            "local_write=completed\noutcome=AcceptanceFailed\n",
        ),
    ] {
        let mut report = Vec::new();
        record_outcome(&mut report, outcome).unwrap();
        assert_eq!(report, expected.as_bytes());
    }
}
