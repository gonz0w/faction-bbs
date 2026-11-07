use super::*;

#[test]
fn foreground_api_is_distinct_from_manual_modes() {
    let _ = foreground_cycle;
    let _ = ForegroundAction::ReceiveOnly;
}

fn peer(vault: &CheckpointVault) -> ForegroundPeer<'static> {
    let state = vault.inspection().unwrap();
    let current = state.peer.unwrap();
    assert_eq!(current.address, "alice@home.test");
    ForegroundPeer {
        address: "alice@home.test",
        device: current.device,
        identity: *current.identity,
    }
}
#[tokio::test(flavor = "current_thread")]
async fn foreground_warm_preflight_and_connection_error_mapping() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    let before = fixture.bytes("live-first");
    // An empty credential deterministically fails in connect before any network I/O.
    let actual = foreground_cycle(
        &mut bob,
        &fixture.settings,
        "",
        current,
        ForegroundAction::ReceiveOnly,
        NOW + 100,
    )
    .await;
    assert_eq!(
        actual.error,
        Some(Error::Transport(faction_xmpp::Error::Config)),
        "a supported warm store must reach fresh connection validation"
    );
    assert_eq!(actual.write, ForegroundWrite::NotAttempted);
    for wrong in [
        ForegroundPeer {
            address: "mallory@home.test",
            ..current
        },
        ForegroundPeer {
            device: 999,
            ..current
        },
        ForegroundPeer {
            identity: [99; 32],
            ..current
        },
    ] {
        let actual = foreground_cycle(
            &mut bob,
            &fixture.settings,
            "",
            wrong,
            ForegroundAction::ReceiveOnly,
            NOW + 100,
        )
        .await;
        assert_eq!(actual.error, Some(Error::State));
    }
    for time in [NOW - 1, NOW + crate::mail_checkpoint::SESSION_AGE + 1] {
        assert_eq!(
            foreground_cycle(
                &mut bob,
                &fixture.settings,
                "",
                current,
                ForegroundAction::ReceiveOnly,
                time
            )
            .await
            .error,
            Some(Error::State)
        );
    }
    assert_eq!(fixture.bytes("live-first"), before);
    bob.revoke().unwrap();
    assert_eq!(
        foreground_cycle(
            &mut bob,
            &fixture.settings,
            "",
            current,
            ForegroundAction::ReceiveOnly,
            NOW + 100
        )
        .await
        .error,
        Some(Error::State)
    );
    let (_owned, _alice, mut cold, settings) = pair();
    let cold_peer = peer(&cold);
    assert_eq!(
        foreground_cycle(
            &mut cold,
            &settings,
            "",
            cold_peer,
            ForegroundAction::ReceiveOnly,
            NOW + 100
        )
        .await
        .error,
        Some(Error::State)
    );
    cold.enable_archive(NOW + 100).unwrap();
    assert_eq!(
        foreground_cycle(
            &mut cold,
            &settings,
            "",
            cold_peer,
            ForegroundAction::ReceiveOnly,
            NOW + 100
        )
        .await
        .error,
        Some(Error::State)
    );
}
#[tokio::test(flavor = "current_thread")]
async fn foreground_foreign_home_is_account_error_before_connect() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    let mut settings = fixture.settings;
    settings.account = "mallory@away.test".into();
    assert_eq!(
        foreground_cycle(
            &mut bob,
            &settings,
            "",
            current,
            ForegroundAction::ReceiveOnly,
            NOW + 100
        )
        .await
        .error,
        Some(Error::Account)
    );
}

struct ForegroundScript {
    discovery: PresenceScript,
    reads: std::collections::VecDeque<Result<faction_xmpp::Element, faction_xmpp::Error>>,
    fail_write: bool,
}
impl ForegroundScript {
    fn new(remote: &CheckpointVault) -> Self {
        let (bundle, prekeys) = remote.publication(NOW + 100).unwrap();
        Self {
            discovery: PresenceScript {
                read_after_pending: false,
                sent: vec![],
                peer: remote.legacy().unwrap().home.clone(),
                device: remote.inspection().unwrap().local_device,
                bundle,
                prekeys,
            },
            reads: Default::default(),
            fail_write: false,
        }
    }
}
impl Channel for ForegroundScript {
    async fn send(&mut self, xml: &faction_xmpp::Element) -> Result<(), faction_xmpp::Error> {
        self.discovery.sent.push(xml.clone());
        if self.fail_write && xml.is("message", "jabber:client") {
            Err(faction_xmpp::Error::Lost)
        } else {
            Ok(())
        }
    }
    async fn next(&mut self) -> Result<faction_xmpp::Element, faction_xmpp::Error> {
        if self
            .discovery
            .sent
            .last()
            .is_some_and(|e| e.is("iq", "jabber:client"))
        {
            return self.discovery.next().await;
        }
        match self.reads.pop_front() {
            Some(read) => read,
            None => std::future::pending().await,
        }
    }
}
fn remote(fixture: &super::foreground_recovery::Recovery) -> CheckpointVault {
    CheckpointVault::unlock(
        crate::mail_vault::Store::at(&fixture.owned.0.join("alice"), false).unwrap(),
        PASS,
        false,
    )
    .unwrap()
}
fn live_frame(delivery: &crate::mail_checkpoint::Delivery) -> faction_xmpp::Element {
    let mut xml = crate::mail_wire::foreground_message(&delivery.wire, delivery.operation).unwrap();
    xml.set_attr(
        "".into(),
        "from".try_into().unwrap(),
        format!("{}/faction{}", delivery.wire.from, delivery.wire.device),
    );
    xml.append_child(
        faction_xmpp::Element::builder("stanza-id", "urn:xmpp:sid:0")
            .attr("by".try_into().unwrap(), delivery.wire.to.as_str())
            .attr("id".try_into().unwrap(), "untrusted-live-id")
            .build(),
    );
    xml
}
#[tokio::test(flavor = "current_thread")]
async fn foreground_selected_operation_is_validated_before_connect() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    assert_eq!(
        foreground_cycle(
            &mut bob,
            &fixture.settings,
            "",
            current,
            ForegroundAction::SendSelected([99; 16]),
            NOW + 100
        )
        .await
        .error,
        Some(Error::State)
    );
}
#[tokio::test(flavor = "current_thread")]
async fn foreground_receive_only_idle_is_no_observed_arrival_without_writes() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    let mut script = ForegroundScript::new(&remote(&fixture));
    let before = fixture.bytes("live-first");
    let report = super::super::foreground::exchange(
        &mut bob,
        &fixture.settings,
        &mut script,
        current,
        ForegroundAction::ReceiveOnly,
        NOW + 100,
    )
    .await;
    assert_eq!(
        report,
        ForegroundReport {
            write: ForegroundWrite::NotAttempted,
            accepted: 0,
            duplicates: 0,
            error: None
        }
    );
    assert!(script.discovery.sent.is_empty());
    assert_eq!(fixture.bytes("live-first"), before);
}
#[tokio::test(flavor = "current_thread")]
async fn foreground_selected_write_and_later_loss_keep_completed_progress() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    let wire = bob
        .queue([77; 16], "selected", "ordinary text", NOW + 100, NOW + 100)
        .unwrap();
    bob.queue([78; 16], "other", "do not send", NOW + 100, NOW + 100)
        .unwrap();
    let before = fixture.bytes("live-first");
    let cursor = bob.archive_checkpoint(NOW + 100).unwrap();
    let mut script = ForegroundScript::new(&remote(&fixture));
    script.reads.push_back(Err(faction_xmpp::Error::Lost));
    let report = super::super::foreground::exchange(
        &mut bob,
        &fixture.settings,
        &mut script,
        current,
        ForegroundAction::SendSelected([77; 16]),
        NOW + 100,
    )
    .await;
    assert_eq!(report.write, ForegroundWrite::Completed);
    assert_eq!(
        report.error,
        Some(Error::Transport(faction_xmpp::Error::Lost))
    );
    let sent = &script.discovery.sent;
    assert_eq!(sent.len(), 3);
    assert_eq!(sent[0].attr("id"), Some("peer-devices"));
    assert_eq!(sent[1].attr("id"), Some("peer-bundle"));
    assert_eq!(
        sent[2],
        crate::mail_wire::foreground_message(&wire, [77; 16]).unwrap()
    );
    assert_eq!(bob.archive_checkpoint(NOW + 100).unwrap(), cursor);
    assert_eq!(fixture.bytes("live-first"), before);
}
#[tokio::test(flavor = "current_thread")]
async fn foreground_ambiguous_selected_write_is_unconfirmed() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    bob.queue([77; 16], "selected", "ordinary text", NOW + 100, NOW + 100)
        .unwrap();
    let selected = bob
        .pending_deliveries(NOW + 100)
        .unwrap()
        .into_iter()
        .find(|item| item.operation == [77; 16])
        .expect("fresh operation must be pending");
    assert_eq!(selected.wire.to, current.address);
    assert_eq!(selected.wire.to_device, current.device);
    let record = bob
        .history()
        .unwrap()
        .iter()
        .find(|record| record.operation() == selected.operation)
        .unwrap();
    assert!(record.attachment().is_none());
    assert_eq!(record.identity, current.identity);
    let before = fixture.bytes("live-first");
    let mut script = ForegroundScript::new(&remote(&fixture));
    script.fail_write = true;
    let report = super::super::foreground::exchange(
        &mut bob,
        &fixture.settings,
        &mut script,
        current,
        ForegroundAction::SendSelected([77; 16]),
        NOW + 100,
    )
    .await;
    assert_eq!(report.write, ForegroundWrite::Unconfirmed);
    assert_eq!(
        report.error,
        Some(Error::Transport(faction_xmpp::Error::Lost))
    );
    assert_eq!(fixture.bytes("live-first"), before);
    assert_eq!(script.discovery.sent.len(), 3);
}

#[tokio::test(flavor = "current_thread")]
async fn foreground_durable_accept_and_duplicate_survive_later_loss() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    let cursor = bob.archive_checkpoint(NOW + 100).unwrap();
    let mut script = ForegroundScript::new(&remote(&fixture));
    let frame = live_frame(fixture.inventory.last().unwrap());
    script
        .reads
        .extend([Ok(frame.clone()), Ok(frame), Err(faction_xmpp::Error::Lost)]);
    let report = super::super::foreground::exchange(
        &mut bob,
        &fixture.settings,
        &mut script,
        current,
        ForegroundAction::ReceiveOnly,
        NOW + 100,
    )
    .await;
    assert_eq!(report.accepted, 1);
    assert_eq!(report.duplicates, 1);
    assert_eq!(
        report.error,
        Some(Error::Transport(faction_xmpp::Error::Lost))
    );
    assert_eq!(report.write, ForegroundWrite::NotAttempted);
    assert_eq!(bob.archive_checkpoint(NOW + 100).unwrap(), cursor);
    assert_eq!(bob.read_summary().unwrap().unread, Some(1));
    drop(bob);
    let bob = fixture.open("live-first");
    assert_eq!(bob.archive_checkpoint(NOW + 100).unwrap(), cursor);
    assert_eq!(bob.read_summary().unwrap().unread, Some(1));
}
#[tokio::test(flavor = "current_thread")]
async fn foreground_duplicate_only_report_counts_known_ciphertext() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    let frame = live_frame(fixture.inventory.last().unwrap());
    let wire = crate::mail_wire::decode_foreground_message(
        &frame,
        crate::mail_wire::Binding {
            own: &fixture.settings.account,
            own_device: 202,
            peer: current.address,
            peer_device: current.device,
            peer_identity: current.identity,
        },
    )
    .unwrap();
    bob.receive_network(&wire, NOW + 100).unwrap();
    let mut script = ForegroundScript::new(&remote(&fixture));
    script
        .reads
        .extend([Ok(frame), Err(faction_xmpp::Error::Lost)]);
    let report = super::super::foreground::exchange(
        &mut bob,
        &fixture.settings,
        &mut script,
        current,
        ForegroundAction::ReceiveOnly,
        NOW + 100,
    )
    .await;
    assert_eq!(report.duplicates, 1);
    assert_eq!(report.accepted, 0);
}
#[tokio::test(flavor = "current_thread")]
async fn foreground_receive_loss_has_typed_transport_error() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    let mut script = ForegroundScript::new(&remote(&fixture));
    script.reads.push_back(Err(faction_xmpp::Error::Lost));
    let report = super::super::foreground::exchange(
        &mut bob,
        &fixture.settings,
        &mut script,
        current,
        ForegroundAction::ReceiveOnly,
        NOW + 100,
    )
    .await;
    assert_eq!(
        report.error,
        Some(Error::Transport(faction_xmpp::Error::Lost))
    );
}
#[tokio::test(flavor = "current_thread")]
async fn foreground_wrong_resource_has_typed_protocol_error() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    let mut script = ForegroundScript::new(&remote(&fixture));
    let mut frame = live_frame(fixture.inventory.last().unwrap());
    frame.set_attr(
        "".into(),
        "to".try_into().unwrap(),
        "bob@away.test/faction999",
    );
    script.reads.push_back(Ok(frame));
    let report = super::super::foreground::exchange(
        &mut bob,
        &fixture.settings,
        &mut script,
        current,
        ForegroundAction::ReceiveOnly,
        NOW + 100,
    )
    .await;
    assert_eq!(report.error, Some(Error::Protocol));
    assert_eq!(report.accepted, 0);
}
#[tokio::test(flavor = "current_thread")]
async fn foreground_budget_stops_before_stanza_33() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    let mut script = ForegroundScript::new(&remote(&fixture));
    let presence: faction_xmpp::Element = "<presence xmlns='jabber:client'/>".parse().unwrap();
    script.reads.extend((0..32).map(|_| Ok(presence.clone())));
    script
        .reads
        .push_back(Ok(live_frame(fixture.inventory.last().unwrap())));
    let before = fixture.bytes("live-first");
    let report = super::super::foreground::exchange(
        &mut bob,
        &fixture.settings,
        &mut script,
        current,
        ForegroundAction::ReceiveOnly,
        NOW + 100,
    )
    .await;
    assert_eq!(report.error, Some(Error::Protocol));
    assert_eq!(script.reads.len(), 1);
    assert_eq!(fixture.bytes("live-first"), before);
}
#[tokio::test(flavor = "current_thread")]
async fn foreground_discovery_missing_current_device_stops_before_write() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    bob.queue([77; 16], "selected", "ordinary text", NOW + 100, NOW + 100)
        .unwrap();
    let selected = bob
        .pending_deliveries(NOW + 100)
        .unwrap()
        .into_iter()
        .find(|item| item.operation == [77; 16])
        .expect("fresh operation must be pending");
    assert_eq!(selected.wire.to, current.address);
    assert_eq!(selected.wire.to_device, current.device);
    let record = bob
        .history()
        .unwrap()
        .iter()
        .find(|record| record.operation() == selected.operation)
        .unwrap();
    assert!(record.attachment().is_none());
    assert_eq!(record.identity, current.identity);
    let before = fixture.bytes("live-first");
    let mut script = ForegroundScript::new(&remote(&fixture));
    script.discovery.device = 999;
    let report = super::super::foreground::exchange(
        &mut bob,
        &fixture.settings,
        &mut script,
        current,
        ForegroundAction::SendSelected([77; 16]),
        NOW + 100,
    )
    .await;
    assert_eq!(report.error, Some(Error::PeerUnavailable));
    assert_eq!(report.write, ForegroundWrite::NotAttempted);
    assert_eq!(script.discovery.sent.len(), 1);
    assert_eq!(fixture.bytes("live-first"), before);
    assert!(
        script
            .discovery
            .sent
            .iter()
            .all(|xml| xml.is("iq", "jabber:client"))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn foreground_discovery_changed_identity_stops_before_write() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    bob.queue([77; 16], "selected", "ordinary text", NOW + 100, NOW + 100)
        .unwrap();
    let selected = bob
        .pending_deliveries(NOW + 100)
        .unwrap()
        .into_iter()
        .find(|item| item.operation == [77; 16])
        .expect("fresh operation must be pending");
    assert_eq!(selected.wire.to, current.address);
    assert_eq!(selected.wire.to_device, current.device);
    let record = bob
        .history()
        .unwrap()
        .iter()
        .find(|record| record.operation() == selected.operation)
        .unwrap();
    assert!(record.attachment().is_none());
    assert_eq!(record.identity, current.identity);
    let before = fixture.bytes("live-first");
    let mut script = ForegroundScript::new(&remote(&fixture));
    script.discovery.bundle.identity = [99; 32];
    let report = super::super::foreground::exchange(
        &mut bob,
        &fixture.settings,
        &mut script,
        current,
        ForegroundAction::SendSelected([77; 16]),
        NOW + 100,
    )
    .await;
    assert_eq!(report.error, Some(Error::Protocol));
    assert_eq!(report.write, ForegroundWrite::NotAttempted);
    assert_eq!(script.discovery.sent.len(), 2);
    assert_eq!(fixture.bytes("live-first"), before);
    assert!(
        script
            .discovery
            .sent
            .iter()
            .all(|xml| xml.is("iq", "jabber:client"))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn foreground_production_receive_then_manual_archive_recovers_at_32_and_rejects_33() {
    use super::foreground_recovery::{Recovery, history, recover};
    for older in [32, 33] {
        let fixture = Recovery::warm(older).await;
        let mut live = fixture.open("live-first");
        let mut archive = fixture.open("archive-only");
        let current = peer(&live);
        let cursor = live.archive_checkpoint(NOW + 100).unwrap();
        let before = fixture.bytes("live-first");
        let mut script = ForegroundScript::new(&remote(&fixture));
        script.reads.extend([
            Ok(live_frame(fixture.inventory.last().unwrap())),
            Err(faction_xmpp::Error::Lost),
        ]);
        let report = super::super::foreground::exchange(
            &mut live,
            &fixture.settings,
            &mut script,
            current,
            ForegroundAction::ReceiveOnly,
            NOW + 100,
        )
        .await;
        assert!(script.discovery.sent.is_empty());
        assert_eq!(live.archive_checkpoint(NOW + 100).unwrap(), cursor);
        if older == 33 {
            assert_eq!(report.accepted, 0);
            assert_eq!(report.error, Some(Error::State));
            assert_eq!(fixture.bytes("live-first"), before);
            assert_eq!(history(&live, []), history(&archive, []));
            recover(&fixture, &mut archive, false).await;
            history(&archive, 0..=older);
            continue;
        }
        assert_eq!(report.accepted, 1);
        assert_eq!(
            report.error,
            Some(Error::Transport(faction_xmpp::Error::Lost))
        );
        history(&live, [older]);
        drop(live);
        let mut live = fixture.open("live-first");
        assert_eq!(live.archive_checkpoint(NOW + 100).unwrap(), cursor);
        history(&live, [older]);
        recover(&fixture, &mut live, true).await;
        recover(&fixture, &mut archive, false).await;
        assert_eq!(
            live.archive_checkpoint(NOW + 100).unwrap(),
            archive.archive_checkpoint(NOW + 100).unwrap()
        );
        let mut actual = history(&live, std::iter::once(older).chain(0..older));
        let mut expected = history(&archive, 0..=older);
        actual.sort_unstable();
        expected.sort_unstable();
        assert_eq!(actual, expected);
    }
}
#[tokio::test(flavor = "current_thread")]
async fn foreground_partial_accept_and_poisoned_publication_preserve_canonical_state() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    let cursor = bob.archive_checkpoint(NOW + 100).unwrap();
    let mut script = ForegroundScript::new(&remote(&fixture));
    let good = live_frame(fixture.inventory.last().unwrap());
    let mut wire = fixture.inventory[0].wire.clone();
    wire.payload[0] ^= 1;
    let bad = live_frame(&crate::mail_checkpoint::Delivery {
        operation: fixture.inventory[0].operation,
        wire,
    });
    script.reads.extend([Ok(good), Ok(bad)]);
    let report = super::super::foreground::exchange(
        &mut bob,
        &fixture.settings,
        &mut script,
        current,
        ForegroundAction::ReceiveOnly,
        NOW + 100,
    )
    .await;
    assert_eq!(report.accepted, 1);
    assert_eq!(report.error, Some(Error::State));
    assert_eq!(bob.archive_checkpoint(NOW + 100).unwrap(), cursor);
    let durable = fixture.bytes("live-first");
    bob.fail_once(crate::mail_checkpoint::Fault::BeforePublish);
    let mut script = ForegroundScript::new(&remote(&fixture));
    script
        .reads
        .push_back(Ok(live_frame(&fixture.inventory[0])));
    let report = super::super::foreground::exchange(
        &mut bob,
        &fixture.settings,
        &mut script,
        current,
        ForegroundAction::ReceiveOnly,
        NOW + 100,
    )
    .await;
    assert_eq!(report.error, Some(Error::State));
    assert_eq!(report.accepted, 0);
    assert!(bob.inspection().is_err());
    assert_eq!(fixture.bytes("live-first"), durable);
    drop(bob);
    let bob = fixture.open("live-first");
    assert_eq!(bob.archive_checkpoint(NOW + 100).unwrap(), cursor);
    assert_eq!(bob.read_summary().unwrap().unread, Some(1));
}

#[tokio::test(flavor = "current_thread")]
async fn foreground_existing_full_quarantine_and_attachment_guards_precede_connect() {
    let fixture = super::foreground_recovery::Recovery::warm(1).await;
    let mut bob = fixture.open("live-first");
    let current = peer(&bob);
    let timestamp = NOW + 100;
    bob.enable_attachments().unwrap();
    let (descriptor, _) = attachment_fixture();
    bob.queue_attachment(
        [45; 16],
        "attachment",
        "no release",
        &descriptor,
        timestamp,
        timestamp,
    )
    .unwrap();
    let before = fixture.bytes("live-first");
    let report = foreground_cycle(
        &mut bob,
        &fixture.settings,
        "",
        current,
        ForegroundAction::SendSelected([45; 16]),
        timestamp,
    )
    .await;
    assert_eq!(report.error, Some(Error::State));
    assert_eq!(report.write, ForegroundWrite::NotAttempted);
    assert_eq!(fixture.bytes("live-first"), before);
    bob.acknowledge([45; 16]).unwrap();
    let backup = fixture.owned.0.join("history-only-355");
    bob.backup(&backup).unwrap();
    let mut quarantined = CheckpointVault::unlock(
        crate::mail_vault::Store::at(&backup, false).unwrap(),
        PASS,
        false,
    )
    .unwrap();
    assert!(quarantined.inspection().unwrap().history_only);
    assert_eq!(
        foreground_cycle(
            &mut quarantined,
            &fixture.settings,
            "",
            current,
            ForegroundAction::ReceiveOnly,
            timestamp
        )
        .await
        .error,
        Some(Error::State)
    );
    for tag in 100..108 {
        bob.queue([tag; 16], "pending", "bounded", timestamp, timestamp)
            .unwrap();
    }
    assert_eq!(
        bob.pending_deliveries(timestamp).unwrap().len(),
        crate::mail_checkpoint::MAX_OUTBOX
    );
    let before = fixture.bytes("live-first");
    assert_eq!(
        foreground_cycle(
            &mut bob,
            &fixture.settings,
            "",
            current,
            ForegroundAction::ReceiveOnly,
            timestamp
        )
        .await
        .error,
        Some(Error::State)
    );
    assert_eq!(fixture.bytes("live-first"), before);
    bob.discard_pending().unwrap();
    let remaining = crate::mail_checkpoint::MAX_HISTORY - bob.history().unwrap().len();
    for index in 0..remaining {
        let tag = u8::try_from(150 + index).unwrap();
        bob.queue([tag; 16], "retained", "bounded", timestamp, timestamp)
            .unwrap();
        bob.acknowledge([tag; 16]).unwrap();
    }
    assert_eq!(
        bob.history().unwrap().len(),
        crate::mail_checkpoint::MAX_HISTORY
    );
    let before = fixture.bytes("live-first");
    assert_eq!(
        foreground_cycle(
            &mut bob,
            &fixture.settings,
            "",
            current,
            ForegroundAction::ReceiveOnly,
            timestamp
        )
        .await
        .error,
        Some(Error::State)
    );
    assert_eq!(fixture.bytes("live-first"), before);
}
