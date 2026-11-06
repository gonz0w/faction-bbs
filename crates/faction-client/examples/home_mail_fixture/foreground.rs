//! Issue #347 only. A single foreground attempt on fresh, owned synthetic state.
//!
//! Run `home_mail_fixture foreground-347 <private-fixture-dir> <bare|full|missing>`.
//! The mode-0700 directory contains mode-0600 OWNERSHIP with the exact text
//! below, sender.toml, receiver.toml, and vaults sender/ and receiver/. Prepare
//! both vaults with archive enabled so the separate setup cycles omit presence.
//! Prepare and queue exactly one text message at the sender. The receiver must
//! have empty history and no archive position. Use a different fixture per trial.
//! Stdin lines: sender vault passphrase, receiver vault passphrase, sender home
//! password, receiver home password. The example keeps the report private in
//! foreground-347.txt and uses the receiver vault only as a disposable probe.

use faction_client::{
    mail_checkpoint::{CheckpointVault, Delivery},
    mail_data::invalid,
    mail_network::Settings,
    mail_vault::Store,
    mail_wire::{self, Binding},
};
use faction_xmpp::{Element, Error, Session};
use rustix::fs::{Mode, OFlags, open};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};

const CLIENT: &str = "jabber:client";
const MARKER: &[u8] = b"faction issue 347 disposable synthetic foreground fixture\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Route {
    Bare,
    Full,
    Missing,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum Outcome {
    Send(Error),
    Receive(Error),
    NoMessage,
    Destination,
    Codec,
    DifferentCiphertext,
    AcceptanceFailed,
    Duplicate,
    Accepted,
    ProjectedAccepted,
}

// Only the sender can emit an application stanza. The receiver has no
// application write in this driver, including presence, MAM and receipts.
pub(super) trait Channel {
    fn receiver_binding(&self) -> &str;
    async fn send(&mut self, stanza: &Element) -> Result<(), Error>;
    async fn next(&mut self) -> Result<Element, Error>;
}

struct Live<'a> {
    receiver_binding: &'a str,
    sender: Session,
    receiver: Session,
}
impl Channel for Live<'_> {
    fn receiver_binding(&self) -> &str {
        self.receiver_binding
    }

    async fn send(&mut self, stanza: &Element) -> Result<(), Error> {
        self.sender.send(stanza).await
    }

    async fn next(&mut self) -> Result<Element, Error> {
        self.receiver.next().await
    }
}

// Only the observed order of encrypted, store and stanza-id permits this
// fixture projection. The unchanged decoder still validates the first two.
fn without_archive_id(frame: &Element, own: &str) -> Option<Element> {
    const SID: &str = "urn:xmpp:sid:0";

    let mut children = frame.children();
    let encrypted = children.next()?;
    let store = children.next()?;
    let sid = children.next()?;
    let id = sid.attr("id")?;
    if children.next().is_some()
        || !encrypted.is("encrypted", mail_wire::OMEMO)
        || !store.is("store", "urn:xmpp:hints")
        || !sid.is("stanza-id", SID)
        || sid.attrs().len() != 2
        || sid.attr("by") != Some(own)
        || sid.nodes().next().is_some()
        || !(1..=256).contains(&id.len())
        || !id.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return None;
    }

    let mut projected = frame.clone();
    projected.remove_child("stanza-id", SID)?;
    Some(projected)
}

pub(super) async fn attempt(
    channel: &mut impl Channel,
    vault: &mut CheckpointVault,
    delivery: &Delivery,
    route: Route,
    now: u64,
    mut observe: impl FnMut(&Element) -> io::Result<()>,
) -> io::Result<Outcome> {
    let bare = &delivery.wire.to;
    let intended = format!("{bare}/faction{}", delivery.wire.to_device);
    let mut outgoing = mail_wire::message(&delivery.wire, delivery.operation)?;
    let destination = match route {
        Route::Bare => bare.clone(),
        Route::Full => intended.clone(),
        Route::Missing => format!("{bare}/faction0"),
    };
    if route != Route::Bare {
        outgoing.set_attr(
            "".into(),
            "to".try_into().map_err(|_| invalid())?,
            destination,
        );
    }
    let operation = outgoing.attr("id").ok_or_else(invalid)?;
    if let Err(error) = channel.send(&outgoing).await {
        return Ok(Outcome::Send(error));
    }

    for _ in 0..32 {
        let frame = match channel.next().await {
            Ok(frame) => frame,
            Err(error) => return Ok(Outcome::Receive(error)),
        };
        if !frame.is("message", CLIENT)
            || frame
                .get_child("event", "http://jabber.org/protocol/pubsub#event")
                .is_some()
        {
            continue;
        }
        // Keep the arrived shape before local checks. This is the parsed stanza
        // from the bounded transport, not a capture of the original TLS bytes.
        observe(&frame)?;
        // The outer ID correlates this fixture's observation. The decoder and
        // exact retained ciphertext are independent checks before acceptance.
        if frame.attr("id") != Some(operation) {
            continue;
        }
        let to = frame.attr("to");
        let matches_route = match route {
            Route::Bare => to == Some(bare) || to == Some(&intended),
            Route::Full => to == Some(&intended),
            // A home may fall back or rewrite the destination. A missing-resource
            // trial is observation only, even if it reaches the intended device.
            Route::Missing => false,
        };
        if channel.receiver_binding() != intended || !matches_route {
            return Ok(Outcome::Destination);
        }
        let state = vault.inspection()?;
        let Some(peer) = state.peer else {
            return Ok(Outcome::AcceptanceFailed);
        };
        let own = &vault.legacy()?.home;
        let decode = |xml| {
            mail_wire::decode_message(
                xml,
                Binding {
                    peer: peer.address,
                    peer_device: peer.device,
                    peer_identity: *peer.identity,
                    own,
                    own_device: state.local_device,
                },
            )
        };
        let (wire, projected) = match decode(&frame) {
            Ok(wire) => (wire, false),
            Err(_) if route == Route::Full => {
                let Some(projected) = without_archive_id(&frame, own) else {
                    return Ok(Outcome::Codec);
                };
                let Ok(wire) = decode(&projected) else {
                    return Ok(Outcome::Codec);
                };
                (wire, true)
            }
            Err(_) => return Ok(Outcome::Codec),
        };
        let queued = &delivery.wire;
        if wire.from != queued.from
            || wire.to != queued.to
            || wire.device != queued.device
            || wire.to_device != queued.to_device
            || wire.prekey != queued.prekey
            || wire.key != queued.key
            || wire.payload != queued.payload
        {
            return Ok(Outcome::DifferentCiphertext);
        }
        // The decoder supplies the enrolled identity and the zero timestamp
        // placeholder. Native acceptance checks identity and authenticated SCE
        // and commits only in the disposable receiver vault.
        return Ok(match vault.receive_network(&wire, now) {
            Ok(receipt) if receipt.duplicate => Outcome::Duplicate,
            Ok(_) if projected => Outcome::ProjectedAccepted,
            Ok(_) => Outcome::Accepted,
            Err(_) => Outcome::AcceptanceFailed,
        });
    }
    Ok(Outcome::NoMessage)
}

fn owned(root: &Path) -> io::Result<()> {
    let uid = rustix::process::geteuid().as_raw();
    if !root.is_absolute() || root.canonicalize()? != root {
        return Err(invalid());
    }
    let metadata = root.metadata()?;
    if !metadata.is_dir() || metadata.uid() != uid || metadata.mode() & 0o7777 != 0o700 {
        return Err(invalid());
    }
    let marker = File::from(open(
        root.join("OWNERSHIP"),
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )?);
    let metadata = marker.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != uid
        || metadata.mode() & 0o7777 != 0o600
        || metadata.nlink() != 1
    {
        return Err(invalid());
    }
    let mut value = Vec::new();
    marker
        .take(MARKER.len() as u64 + 1)
        .read_to_end(&mut value)?;
    if value != MARKER {
        return Err(invalid());
    }
    Ok(())
}

fn stanza(report: &mut File, label: &str, frame: &Element) -> io::Result<()> {
    let mut xml = Vec::new();
    frame.write_to(&mut xml).map_err(io::Error::other)?;
    let xml = std::str::from_utf8(&xml).map_err(io::Error::other)?;
    writeln!(report, "{label}={}", super::json(xml))
}

pub(super) fn record_outcome(report: &mut impl Write, outcome: Option<Outcome>) -> io::Result<()> {
    let local_write = match &outcome {
        None => "not_attempted",
        Some(Outcome::Send(_)) => "unconfirmed",
        Some(_) => "completed",
    };
    writeln!(report, "local_write={local_write}")?;
    if let Some(outcome) = outcome {
        writeln!(report, "outcome={outcome:?}")?;
    }
    Ok(())
}

pub(super) fn run(root: &Path, mode: &str, fields: &[&str], now: u64) -> io::Result<()> {
    let route = match mode {
        "bare" => Route::Bare,
        "full" => Route::Full,
        "missing" => Route::Missing,
        _ => return Err(invalid()),
    };
    let [send_pass, receive_pass, send_password, receive_password] = fields else {
        return Err(invalid());
    };
    if !(12..=128).contains(&receive_pass.len()) {
        return Err(invalid());
    }
    owned(root)?;
    let sender = CheckpointVault::unlock(
        Store::at(&root.join("sender"), false)?,
        send_pass.as_bytes(),
        false,
    )?;
    let mut receiver = CheckpointVault::unlock(
        Store::at(&root.join("receiver"), false)?,
        receive_pass.as_bytes(),
        false,
    )?;
    let send_archive = sender.archive_checkpoint(now)?;
    let receive_archive = receiver.archive_checkpoint(now)?;
    let pending = sender.pending_deliveries(now)?;
    let [delivery] = pending.as_slice() else {
        return Err(invalid());
    };
    let [record] = sender.history()? else {
        return Err(invalid());
    };
    if send_archive.after.is_some()
        || receive_archive.after.is_some()
        || !receiver.history()?.is_empty()
        || record.attachment().is_some()
    {
        return Err(invalid());
    }
    let send_settings = Settings::load(&root.join("sender.toml"))?;
    let receive_settings = Settings::load(&root.join("receiver.toml"))?;
    let send_home = send_settings
        .transport(send_archive.local_device)
        .map_err(io::Error::other)?;
    let receive_home = receive_settings
        .transport(receive_archive.local_device)
        .map_err(io::Error::other)?;
    if send_settings.account != send_archive.home
        || receive_settings.account != receive_archive.home
        || send_home.endpoint() == receive_home.endpoint()
        || !send_home.endpoint().ip().is_loopback()
        || !receive_home.endpoint().ip().is_loopback()
    {
        return Err(invalid());
    }

    let mut report = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join("foreground-347.txt"))?;
    writeln!(report, "route={route:?}")?;
    stanza(
        &mut report,
        "queued_bare_stanza",
        &mail_wire::message(&delivery.wire, delivery.operation)?,
    )?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let (send, receive) = tokio::join!(
            faction_xmpp::connect(&send_home, send_password),
            faction_xmpp::connect(&receive_home, receive_password),
        );
        // connect succeeds only after an exact server bind response. Derive this
        // binding from the receiver's local settings and vault, never the message.
        for (label, home, result) in [
            ("sender", &send_home, &send),
            ("receiver", &receive_home, &receive),
        ] {
            match result {
                Ok(_) => writeln!(report, "{label}_bound={}", home.full_jid())?,
                Err(error) => writeln!(report, "{label}_connect={error:?}")?,
            }
        }
        let (Ok(sender_session), Ok(receiver_session)) = (send, receive) else {
            return record_outcome(&mut report, None);
        };
        let mut channel = Live {
            receiver_binding: receive_home.full_jid(),
            sender: sender_session,
            receiver: receiver_session,
        };
        let outcome = attempt(&mut channel, &mut receiver, delivery, route, now, |frame| {
            stanza(&mut report, "received_stanza_before_codec", frame)
        })
        .await?;
        record_outcome(&mut report, Some(outcome))
    })?;
    let retained = sender.pending_deliveries(now)?;
    let [retained] = retained.as_slice() else {
        return Err(invalid());
    };
    if retained.operation != delivery.operation
        || retained.wire != delivery.wire
        || receiver.archive_checkpoint(now)? != receive_archive
    {
        return Err(invalid());
    }
    writeln!(
        report,
        "pending_ciphertext_unchanged=true\ncursor_unchanged=true"
    )?;
    report.sync_all()
}
