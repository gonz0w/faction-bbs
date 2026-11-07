//! One explicitly selected foreground attempt, separate from manual mail modes.
use super::*;

/// The peer context reviewed in the trusted local conversation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForegroundPeer<'a> {
    pub address: &'a str,
    pub device: u32,
    pub identity: [u8; 32],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForegroundAction {
    ReceiveOnly,
    SendSelected([u8; 16]),
}
/// Local transport-write observation; never peer delivery or acknowledgement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForegroundWrite {
    NotAttempted,
    Unconfirmed,
    Completed,
}
/// Durable progress is retained even when a later step fails.
#[derive(Debug, Eq, PartialEq)]
pub struct ForegroundReport {
    pub write: ForegroundWrite,
    pub accepted: usize,
    pub duplicates: usize,
    pub error: Option<Error>,
}
struct Checked {
    archive: crate::mail_checkpoint::ArchiveCheckpoint,
    selected: Option<crate::mail_checkpoint::Delivery>,
    clock: CycleClock,
}
fn check(
    vault: &CheckpointVault,
    settings: &Settings,
    peer: ForegroundPeer<'_>,
    action: ForegroundAction,
    now: u64,
) -> Result<Checked, Error> {
    let clock = CycleClock::new(now);
    if backend(vault.legacy())?.home != settings.account {
        return Err(Error::Account);
    }
    let archive = backend(vault.archive_checkpoint(now))?;
    let state = backend(vault.inspection())?;
    let current = state.peer.ok_or(Error::State)?;
    if !state.started
        || archive.after.is_none()
        || current.address != peer.address
        || current.device != peer.device
        || *current.identity != peer.identity
    {
        return Err(Error::State);
    }
    let selected = match action {
        ForegroundAction::ReceiveOnly => None,
        ForegroundAction::SendSelected(operation) => {
            let delivery = backend(vault.pending_deliveries(now))?
                .into_iter()
                .find(|item| item.operation == operation)
                .ok_or(Error::State)?;
            let record = backend(vault.history())?
                .iter()
                .find(|record| {
                    record.direction == crate::mail_checkpoint::Direction::Sent
                        && record.operation() == operation
                })
                .ok_or(Error::State)?;
            if record.attachment().is_some()
                || record.address != peer.address
                || record.device != peer.device
                || record.identity != peer.identity
                || delivery.wire.from != settings.account
                || delivery.wire.device != state.local_device
                || delivery.wire.to != peer.address
                || delivery.wire.to_device != peer.device
                || delivery.wire.identity != *state.identity
            {
                return Err(Error::State);
            }
            Some(delivery)
        }
    };
    Ok(Checked {
        archive,
        selected,
        clock,
    })
}
fn recheck(
    vault: &CheckpointVault,
    settings: &Settings,
    peer: ForegroundPeer<'_>,
    action: ForegroundAction,
    checked: &Checked,
) -> Result<(), Error> {
    let current = check(vault, settings, peer, action, checked.clock.now())?;
    let same_selected = match (&checked.selected, &current.selected) {
        (None, None) => true,
        (Some(before), Some(after)) => {
            before.operation == after.operation && before.wire == after.wire
        }
        _ => false,
    };
    if current.archive != checked.archive || !same_selected {
        return Err(Error::State);
    }
    Ok(())
}
fn empty_report() -> ForegroundReport {
    ForegroundReport {
        write: ForegroundWrite::NotAttempted,
        accepted: 0,
        duplicates: 0,
        error: None,
    }
}
/// One fresh authentication and connection. Dropping the future closes it.
/// The caller supplies fresh zeroizing credential storage for this attempt.
pub async fn foreground_cycle(
    vault: &mut CheckpointVault,
    settings: &Settings,
    password: &str,
    peer: ForegroundPeer<'_>,
    action: ForegroundAction,
    now: u64,
) -> ForegroundReport {
    let mut report = empty_report();
    let result: Result<(), Error> = async {
        let checked = check(vault, settings, peer, action, now)?;
        let home = settings
            .transport(checked.archive.local_device)
            .map_err(Error::Transport)?;
        let mut session = faction_xmpp::connect(&home, password)
            .await
            .map_err(Error::Transport)?;
        exchange_ready(
            vault,
            settings,
            &mut session,
            peer,
            action,
            checked,
            &mut report,
        )
        .await
    }
    .await;
    report.error = result.err();
    report
}
#[cfg(test)]
pub(super) async fn exchange<C: Channel>(
    vault: &mut CheckpointVault,
    settings: &Settings,
    session: &mut C,
    peer: ForegroundPeer<'_>,
    action: ForegroundAction,
    now: u64,
) -> ForegroundReport {
    let mut report = empty_report();
    let result: Result<(), Error> = async {
        let checked = check(vault, settings, peer, action, now)?;
        exchange_ready(vault, settings, session, peer, action, checked, &mut report).await
    }
    .await;
    report.error = result.err();
    report
}
fn accept(
    vault: &mut CheckpointVault,
    settings: &Settings,
    peer: ForegroundPeer<'_>,
    action: ForegroundAction,
    checked: &Checked,
    xml: &faction_xmpp::Element,
    report: &mut ForegroundReport,
) -> Result<(), Error> {
    recheck(vault, settings, peer, action, checked)?;
    if !xml.is("message", "jabber:client")
        || xml
            .get_child("event", "http://jabber.org/protocol/pubsub#event")
            .is_some()
    {
        return Ok(());
    }
    let wire = protocol(crate::mail_wire::decode_foreground_message(
        xml,
        crate::mail_wire::Binding {
            own: &settings.account,
            own_device: checked.archive.local_device,
            peer: peer.address,
            peer_device: peer.device,
            peer_identity: peer.identity,
        },
    ))?;
    let receipt = backend(vault.receive_network(&wire, checked.clock.now()))?;
    if receipt.duplicate {
        report.duplicates += 1;
    } else {
        report.accepted += 1;
    }
    Ok(())
}
async fn next<C: Channel>(
    session: &mut C,
    remaining: &mut usize,
) -> Result<Option<faction_xmpp::Element>, Error> {
    if *remaining == 0 {
        return Err(Error::Protocol);
    }
    match tokio::time::timeout(std::time::Duration::from_secs(2), session.next()).await {
        Err(_) => Ok(None),
        Ok(result) => {
            let xml = result.map_err(Error::Transport)?;
            *remaining -= 1;
            Ok(Some(xml))
        }
    }
}
async fn exchange_ready<C: Channel>(
    vault: &mut CheckpointVault,
    settings: &Settings,
    session: &mut C,
    peer: ForegroundPeer<'_>,
    action: ForegroundAction,
    checked: Checked,
    report: &mut ForegroundReport,
) -> Result<(), Error> {
    let mut remaining = 32;
    recheck(vault, settings, peer, action, &checked)?;
    if let Some(delivery) = &checked.selected {
        for id in ["peer-devices", "peer-bundle"] {
            let query = protocol(if id == "peer-devices" {
                crate::mail_wire::devices_query(id, &settings.account, peer.address)
            } else {
                crate::mail_wire::bundle_query(id, &settings.account, peer.address, peer.device)
            })?;
            session.send(&query).await.map_err(Error::Transport)?;
            let response = loop {
                let xml = next(session, &mut remaining)
                    .await?
                    .ok_or(Error::Transport(faction_xmpp::Error::Timeout))?;
                if xml.is("iq", "jabber:client") && xml.attr("id") == Some(id) {
                    break xml;
                }
                accept(vault, settings, peer, action, &checked, &xml, report)?;
            };
            if id == "peer-devices" {
                let devices = crate::mail_wire::devices_result(
                    &response,
                    id,
                    &settings.account,
                    peer.address,
                )
                .map_err(|_| Error::PeerUnavailable)?;
                if !devices.contains(&peer.device) {
                    return Err(Error::PeerUnavailable);
                }
            } else {
                protocol(crate::mail_wire::bundle_result(
                    &response,
                    id,
                    &settings.account,
                    peer.address,
                    peer.device,
                    &peer.identity,
                ))?;
            }
            recheck(vault, settings, peer, action, &checked)?;
        }
        let xml = protocol(crate::mail_wire::foreground_message(
            &delivery.wire,
            delivery.operation,
        ))?;
        report.write = ForegroundWrite::Unconfirmed;
        session.send(&xml).await.map_err(Error::Transport)?;
        report.write = ForegroundWrite::Completed;
    }
    loop {
        let Some(xml) = next(session, &mut remaining).await? else {
            return Ok(());
        };
        accept(vault, settings, peer, action, &checked, &xml, report)?;
    }
}
