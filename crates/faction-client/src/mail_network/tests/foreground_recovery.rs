use super::*;
use crate::mail_checkpoint::{Direction, Fault};

const AT: u64 = NOW + 100;
const ARCHIVE: &str = "archive-only";
const LIVE: &str = "live-first";

fn subject(index: usize) -> String {
    format!("recovery message {index}")
}

fn body(index: usize) -> String {
    format!("authenticated recovery body {index}")
}

// Check ordered received inventory and read decisions against fixture input,
// separately from what the scripted archive reports in fin.
pub(super) fn history(
    vault: &CheckpointVault,
    recovered: impl IntoIterator<Item = usize>,
) -> Vec<[u8; 32]> {
    let records = vault.history().unwrap();
    let expected = [
        (
            "prekey bootstrap".into(),
            "authenticated prekey".into(),
            NOW,
            true,
        ),
        (
            "read archive seed".into(),
            "authenticated archive seed".into(),
            NOW + 2,
            true,
        ),
    ]
    .into_iter()
    .chain(
        recovered
            .into_iter()
            .map(|index| (subject(index), body(index), NOW + 3 + index as u64, false)),
    )
    .collect::<Vec<_>>();
    assert_eq!(records.len(), expected.len() + 1); // One synthetic sent reply.
    assert_eq!(records[1].direction, Direction::Sent);
    assert_eq!(records[1].subject.as_str(), "ratchet reply");
    let mut received = records
        .iter()
        .enumerate()
        .filter(|(_, record)| record.direction == Direction::Received);
    let ids = expected
        .iter()
        .map(|(subject, body, time, read)| {
            let (index, record) = received.next().unwrap();
            let id = record.received_id().unwrap();
            assert_eq!(record.address, "alice@home.test");
            assert_eq!(record.subject.as_str(), subject);
            assert_eq!(record.body.as_str(), body);
            assert_eq!(record.sent_at, *time);
            assert_eq!(vault.received_read(index, id).unwrap(), Some(*read));
            id
        })
        .collect::<Vec<_>>();
    assert!(received.next().is_none());
    let summary = vault.read_summary().unwrap();
    assert_eq!(summary.unread, Some(expected.len() - 2));
    assert_eq!(summary.received, expected.len());
    assert_eq!(summary.pending, 0);
    ids
}

pub(super) async fn recover(fixture: &Recovery, vault: &mut CheckpointVault, duplicate: bool) {
    let count = fixture.inventory.len();
    for start in (0..count).step_by(4) {
        let end = (start + 4).min(count);
        let last = end == count;
        let report = fixture
            .page(vault, start..end, last, MamEnd::Good)
            .await
            .unwrap();
        let duplicates = usize::from(last && duplicate);
        assert_eq!(report.accepted, end - start - duplicates);
        assert_eq!(report.duplicates, duplicates);
        assert_eq!(report.sent, 0);
        assert_eq!(
            report.archive,
            Some(ArchiveReport {
                entries: end - start,
                complete: last
            })
        );
        assert_eq!(
            vault.archive_checkpoint(AT).unwrap().after,
            Some(format!("item{}", end + 1))
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn foreground_archive_recovers_older_and_replays_live_at_native_skip_limit() {
    for older in [1, 32] {
        let fixture = Recovery::warm(older).await;
        let before = fixture.bytes("bob");
        assert_eq!(fixture.bytes(ARCHIVE), before);
        assert_eq!(fixture.bytes(LIVE), before);
        let mut archive = fixture.open(ARCHIVE);
        let mut live = fixture.open(LIVE);
        let seed_cursor = archive.archive_checkpoint(AT).unwrap();
        assert_eq!(seed_cursor.after.as_deref(), Some("item1"));
        assert_eq!(live.archive_checkpoint(AT).unwrap(), seed_cursor);
        assert_eq!(history(&live, []), history(&archive, []));

        let receipt = fixture.receive_newest(&mut live).unwrap();
        assert!(!receipt.duplicate);
        assert_eq!(live.archive_checkpoint(AT).unwrap(), seed_cursor);
        assert_eq!(history(&live, [older]).last(), Some(&receipt.frame));
        drop(live);
        let mut live = fixture.open(LIVE);
        assert_eq!(live.archive_checkpoint(AT).unwrap(), seed_cursor);
        assert_eq!(history(&live, [older]).last(), Some(&receipt.frame));

        recover(&fixture, &mut archive, false).await;
        recover(&fixture, &mut live, true).await;
        let cursor = archive.archive_checkpoint(AT).unwrap();
        assert_eq!(live.archive_checkpoint(AT).unwrap(), cursor);
        drop(live);
        drop(archive);
        let live = fixture.open(LIVE);
        let archive = fixture.open(ARCHIVE);
        assert_eq!(live.archive_checkpoint(AT).unwrap(), cursor);
        assert_eq!(archive.archive_checkpoint(AT).unwrap(), cursor);
        let mut live_ids = history(&live, std::iter::once(older).chain(0..older));
        let mut archive_ids = history(&archive, 0..=older);
        live_ids.sort_unstable();
        archive_ids.sort_unstable();
        assert_eq!(live_ids, archive_ids);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn foreground_archive_rejects_33_unseen_and_archive_only_recovers_inventory() {
    let fixture = Recovery::warm(33).await;
    let canonical = fixture.bytes("bob");
    assert_eq!(fixture.bytes(ARCHIVE), canonical);
    assert_eq!(fixture.bytes(LIVE), canonical);
    let mut archive = fixture.open(ARCHIVE);
    let mut live = fixture.open(LIVE);
    let cursor = live.archive_checkpoint(AT).unwrap();
    assert!(fixture.receive_newest(&mut live).is_err());
    assert_eq!(fixture.bytes(LIVE), canonical);
    assert_eq!(live.archive_checkpoint(AT).unwrap(), cursor);
    assert_eq!(history(&live, []), history(&archive, []));
    drop(live);
    let live = fixture.open(LIVE);
    assert_eq!(live.archive_checkpoint(AT).unwrap(), cursor);
    history(&live, []);

    recover(&fixture, &mut archive, false).await;
    drop(archive);
    let archive = fixture.open(ARCHIVE);
    assert_eq!(
        archive.archive_checkpoint(AT).unwrap().after.as_deref(),
        Some("item35")
    );
    history(&archive, 0..=33);
    assert_eq!(fixture.bytes(LIVE), canonical);
}

#[tokio::test(flavor = "current_thread")]
async fn foreground_archive_post_live_faults_preserve_the_last_valid_checkpoint() {
    let fixture = Recovery::warm(1).await;
    let mut live = fixture.open(LIVE);
    let cursor = live.archive_checkpoint(AT).unwrap();
    fixture.receive_newest(&mut live).unwrap();
    drop(live);
    let before = fixture.bytes(LIVE);
    let mut live = fixture.open(LIVE);
    let ids = history(&live, [1]);
    for (end, expected) in [
        (MamEnd::WrongId, Error::Protocol),
        (MamEnd::Lost, Error::Transport(faction_xmpp::Error::Lost)),
        (MamEnd::Idle, Error::Transport(faction_xmpp::Error::Timeout)),
    ] {
        assert_eq!(
            fixture.page(&mut live, 0..2, true, end).await.unwrap_err(),
            expected
        );
        assert_eq!(fixture.bytes(LIVE), before);
        assert_eq!(live.archive_checkpoint(AT).unwrap(), cursor);
        assert_eq!(history(&live, [1]), ids);
    }

    // Known fixture inventory still contains both entries. An empty partial
    // response supplies no cursor evidence and cannot satisfy the inventory.
    let empty = fixture
        .page(&mut live, 0..0, false, MamEnd::Good)
        .await
        .unwrap();
    assert_eq!(
        empty.archive,
        Some(ArchiveReport {
            entries: 0,
            complete: false
        })
    );
    assert_eq!(fixture.bytes(LIVE), before);
    assert_eq!(live.archive_checkpoint(AT).unwrap(), cursor);

    live.fail_once(Fault::BeforePublish);
    assert_eq!(
        fixture
            .page(&mut live, 0..2, true, MamEnd::Good)
            .await
            .unwrap_err(),
        Error::State
    );
    assert!(live.inspection().is_err());
    assert_eq!(fixture.bytes(LIVE), before);
    drop(live);
    let mut live = fixture.open(LIVE);
    assert_eq!(live.archive_checkpoint(AT).unwrap(), cursor);
    assert_eq!(history(&live, [1]), ids);
    recover(&fixture, &mut live, true).await;
    history(&live, [1, 0]);
}

pub(super) struct Recovery {
    pub(super) settings: Settings,
    pub(super) inventory: Vec<crate::mail_checkpoint::Delivery>,
    pub(super) owned: Owned,
}

impl Recovery {
    pub(super) async fn warm(older: usize) -> Self {
        use crate::mail_vault::Store;
        use std::os::unix::fs::DirBuilderExt;

        let (owned, mut alice, mut bob, settings) = pair();
        let bootstrap = retain(
            &mut alice,
            1,
            "prekey bootstrap",
            "authenticated prekey",
            NOW,
        );
        assert!(bootstrap.wire.prekey);
        let first = bob.receive(&bootstrap.wire, AT).unwrap();
        let reply = retain(&mut bob, 2, "ratchet reply", "authenticated reply", NOW + 1);
        assert!(!alice.receive(&reply.wire, AT).unwrap().duplicate);

        // Seed the receiving chain after the reply. There are no further replies
        // or starts before the subsequent ordinary messages on that chain.
        bob.enable_archive(AT).unwrap();
        bob.enable_read_tracking().unwrap();
        let seed = retain(
            &mut alice,
            3,
            "read archive seed",
            "authenticated archive seed",
            NOW + 2,
        );
        assert!(!seed.wire.prekey);
        let report = archive_page(&mut bob, &settings, 0, &[seed], true, MamEnd::Good)
            .await
            .unwrap();
        assert_eq!(report.accepted, 1);
        assert_eq!(report.duplicates, 0);
        assert_eq!(
            bob.archive_checkpoint(AT).unwrap().after.as_deref(),
            Some("item1")
        );
        bob.mark_read(0, first.frame).unwrap();
        let seed_id = bob.history().unwrap()[2].received_id().unwrap();
        bob.mark_read(2, seed_id).unwrap();

        let inventory = (0..=older)
            .map(|index| {
                let tag = u8::try_from(index + 4).unwrap();
                let delivery = retain(
                    &mut alice,
                    tag,
                    &subject(index),
                    &body(index),
                    NOW + 3 + index as u64,
                );
                assert!(!delivery.wire.prekey);
                delivery
            })
            .collect();
        drop(alice);
        drop(bob);

        // The ordinary backup path quarantines ratchets. Copy the exact
        // synthetic canonical bytes only after releasing the original lock.
        for branch in [ARCHIVE, LIVE] {
            let path = owned.0.join(branch);
            fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
            fs::copy(owned.0.join("bob/mail.fkv"), path.join("mail.fkv")).unwrap();
            drop(Store::at(&path, false).unwrap());
        }
        Self {
            settings,
            inventory,
            owned,
        }
    }

    pub(super) fn open(&self, branch: &str) -> CheckpointVault {
        let path = self.owned.0.join(branch);
        CheckpointVault::unlock(
            crate::mail_vault::Store::at(&path, false).unwrap(),
            PASS,
            false,
        )
        .unwrap()
    }

    pub(super) fn bytes(&self, branch: &str) -> Vec<u8> {
        fs::read(self.owned.0.join(branch).join("mail.fkv")).unwrap()
    }

    fn receive_newest(
        &self,
        vault: &mut CheckpointVault,
    ) -> io::Result<crate::mail_checkpoint::Receipt> {
        let newest = self.inventory.last().unwrap();
        let mut stanza = crate::mail_wire::message(&newest.wire, newest.operation).unwrap();
        stanza.set_attr(
            "".into(),
            "to".try_into().unwrap(),
            "bob@away.test/faction202",
        );
        assert_eq!(
            stanza.attr("id"),
            Some(crate::mail_data::hex(&newest.operation).as_str())
        );
        let state = vault.inspection().unwrap();
        let peer = state.peer.unwrap();
        let wire = crate::mail_wire::decode_message(
            &stanza,
            crate::mail_wire::Binding {
                peer: peer.address,
                peer_device: peer.device,
                peer_identity: *peer.identity,
                own: &self.settings.account,
                own_device: state.local_device,
            },
        )
        .unwrap();
        let mut expected = newest.wire.clone();
        expected.sent_at = 0;
        assert_eq!(wire, expected);

        // Start after #349's metadata projection, on this disposable copy.
        // The normal archive-enabled live receive gate is not changed.
        vault.receive_network(&wire, AT)
    }

    async fn page(
        &self,
        vault: &mut CheckpointVault,
        range: std::ops::Range<usize>,
        complete: bool,
        end: MamEnd,
    ) -> Result<Report, Error> {
        archive_page(
            vault,
            &self.settings,
            range.start + 1,
            &self.inventory[range],
            complete,
            end,
        )
        .await
    }
}

fn retain(
    sender: &mut CheckpointVault,
    tag: u8,
    subject: &str,
    body: &str,
    time: u64,
) -> crate::mail_checkpoint::Delivery {
    let operation = [tag; 16];
    let wire = sender.queue(operation, subject, body, time, AT).unwrap();
    let mut pending = sender.pending_deliveries(AT).unwrap();
    assert_eq!(pending.len(), 1);
    let retained = pending.pop().unwrap();
    assert_eq!(retained.operation, operation);
    assert_eq!(retained.wire, wire);
    // Only this synthetic sender explicitly retires each operation after copying
    // its immutable ciphertext. Archive capacity and delivery are unrelated.
    sender.acknowledge(operation).unwrap();
    retained
}

async fn archive_page(
    vault: &mut CheckpointVault,
    settings: &Settings,
    offset: usize,
    items: &[crate::mail_checkpoint::Delivery],
    complete: bool,
    end: MamEnd,
) -> Result<Report, Error> {
    use crate::mail_archive::{MAM, RSM};

    assert!(items.len() <= 4);
    let mut script = MamScript::new(items.iter().map(|item| item.wire.clone()).collect());
    script.operations = items.iter().map(|item| item.operation).collect();
    script.offset = offset;
    script.complete = complete;
    script.end = end;
    let ready = preflight(vault, settings, Mode::CatchUp, AT).unwrap();
    let report = exchange(vault, settings, &mut script, Mode::CatchUp, ready).await;
    assert_eq!(script.index, items.len());
    let [iq] = script.sent.as_slice() else {
        panic!("catch-up must send one scoped query");
    };
    assert_eq!(iq.attr("to"), Some(settings.account.as_str()));
    let query = iq.get_child("query", MAM).unwrap();
    let scope = query
        .get_child("x", "jabber:x:data")
        .unwrap()
        .children()
        .find(|field| field.attr("var") == Some("with"))
        .unwrap();
    assert_eq!(
        scope.get_child("value", "jabber:x:data").unwrap().text(),
        "alice@home.test"
    );
    let set = query.get_child("set", RSM).unwrap();
    assert_eq!(set.get_child("max", RSM).unwrap().text(), "4");
    assert_eq!(
        set.get_child("after", RSM).map(faction_xmpp::Element::text),
        (offset != 0).then(|| format!("item{offset}"))
    );
    report
}
