//! Owned synthetic software fixture; no supported CLI or provider adapter.
#[cfg(feature = "network-mail")]
mod enabled {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use faction_client::{
        mail_blob_transfer::{self as transfer, InvitedHomes, Selection, Spool},
        mail_blob_wire,
        mail_checkpoint::{CheckpointVault, Direction},
        mail_data::{Data, hex, invalid},
        mail_network::{self, Mode, Settings},
        mail_vault::Store,
        mail_wire,
    };
    use faction_files::private_blobs::{
        self as blobs, Access, Action, Binding, Principal, Request,
    };
    use faction_xmpp::Element;
    use std::{
        fs,
        io::{self, Read, Write},
        os::unix::fs::{DirBuilderExt, PermissionsExt},
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };
    use zeroize::Zeroizing;
    const OP: [u8; 16] = [61; 16];
    const PUBSUB: &str = "http://jabber.org/protocol/pubsub";
    fn mapped<T, E>(r: Result<T, E>) -> io::Result<T> {
        r.map_err(|_| invalid())
    }
    fn now() -> io::Result<u64> {
        Ok(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| invalid())?
            .as_secs())
    }
    fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
        fs::write(path, bytes)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
    }
    fn xml(e: &Element) -> io::Result<Vec<u8>> {
        let mut b = Vec::new();
        mapped(e.write_to(&mut b))?;
        Ok(b)
    }
    fn load(root: &Path, name: &str, pass: &str) -> io::Result<CheckpointVault> {
        CheckpointVault::unlock(Store::at(&root.join(name), false)?, pass.as_bytes(), false)
    }
    fn data(size: usize) -> Vec<u8> {
        (0..size).map(|i| ((i * 73 + 19) % 251) as u8).collect()
    }
    fn contains_private(bytes: &[u8], needles: &[&[u8]]) -> bool {
        needles.iter().any(|secret| {
            secret.is_empty() || bytes.windows(secret.len()).any(|window| window == *secret)
        })
    }
    // Runs on the member-device fixture, never in the handler. It loads the
    // descriptor locally and exports only a fixed pass/fail status.
    fn audit(root: &Path, vault: &CheckpointVault) -> io::Result<()> {
        let descriptor = vault
            .history()?
            .iter()
            .find_map(|r| r.attachment())
            .ok_or_else(invalid)?;
        let raw: Vec<&[u8]> = vec![
            descriptor.name.as_bytes(),
            b"private-selected-subject",
            b"private-selected-body",
            descriptor.key.as_slice(),
            descriptor.iv.as_slice(),
            descriptor.hash.as_slice(),
        ];
        let mut encoded = Zeroizing::new(
            raw.iter()
                .map(|b| STANDARD.encode(b).into_bytes())
                .collect::<Vec<Vec<u8>>>(),
        );
        encoded.push(data(64));
        let mut needles = raw;
        needles.extend(encoded.iter().map(Vec::as_slice));
        for entry in fs::read_dir(root.join("serverstore"))? {
            let path = entry?.path();
            let bytes = fs::read(path)?;
            if bytes.len() > blobs::MAX_SNAPSHOT || contains_private(&bytes, &needles) {
                return Err(invalid());
            }
        }
        let wire = fs::read(root.join("wire.private"))?;
        if wire.len() > 16 * 1024 * 1024 || contains_private(&wire, &needles) {
            return Err(invalid());
        }
        Ok(())
    }
    fn init(root: &Path, pass: &str) -> io::Result<()> {
        let stamp = now()?;
        let mut a = CheckpointVault::create(
            Store::at(&root.join("alice"), true)?,
            Data::new("alice@home.test".into())?,
            pass.as_bytes(),
            101,
            stamp,
        )?
        .0;
        let mut b = CheckpointVault::create(
            Store::at(&root.join("bob"), true)?,
            Data::new("bob@away.test".into())?,
            pass.as_bytes(),
            202,
            stamp,
        )?
        .0;
        a.enable_attachments()?;
        b.enable_attachments()?;
        let ab = a.bundle()?;
        let bb = b.bundle()?;
        a.enroll(
            "bob@away.test",
            202,
            &format!("ed25519:{}", hex(&bb.identity)),
            stamp,
        )?;
        b.enroll(
            "alice@home.test",
            101,
            &format!("ed25519:{}", hex(&ab.identity)),
            stamp,
        )?;
        a.start(&bb, stamp)?;
        a.enable_archive(stamp)?;
        // Only these public exports are made available to the server fixture.
        let public = Binding {
            sender: Principal {
                account: "alice@home.test".into(),
                device: 101,
                identity: ab.identity,
            },
            recipient: Principal {
                account: "bob@away.test".into(),
                device: 202,
                identity: bb.identity,
            },
            id: [0; 32],
            size: 16,
            created: stamp,
            expires: stamp + blobs::MAX_RETENTION,
        };
        write(
            &root.join("public-principals.bin"),
            &mapped(blobs::encode_request(&Request::Begin(public)))?,
        )?;
        for (name, v) in [("alice", &a), ("bob", &b)] {
            let (bundle, prekeys) = v.publication(stamp)?;
            let e = mail_wire::bundle_publish(
                "public-fixture",
                &v.legacy()?.home,
                v.inspection()?.local_device,
                &bundle,
                &prekeys,
            )?;
            write(&root.join(format!("{name}-bundle.xml")), &xml(&e)?)?;
        }
        fs::DirBuilder::new()
            .mode(0o700)
            .create(root.join("serverstore"))?;
        println!("{}\n{}", hex(&ab.identity), hex(&bb.identity));
        Ok(())
    }
    struct Approved {
        alice: Principal,
        bob: Principal,
        revoked: bool,
    }
    impl Access for Approved {
        fn require(
            &self,
            actor: &Principal,
            b: &Binding,
            _: Action,
            _: u64,
        ) -> Result<(), blobs::Error> {
            if !self.revoked
                && (actor == &self.alice || actor == &self.bob)
                && b.sender == self.alice
                && b.recipient == self.bob
            {
                Ok(())
            } else {
                Err(blobs::Error::Denied)
            }
        }
    }
    fn set(e: &mut Element, k: &str, v: &str) -> io::Result<()> {
        e.set_attr("".into(), k.try_into().map_err(|_| invalid())?, v);
        Ok(())
    }
    fn iq(id: &str, from: &str, to: &str) -> io::Result<Element> {
        let mut e = Element::builder("iq", "jabber:client").build();
        for (k, v) in [("id", id), ("type", "result"), ("from", from), ("to", to)] {
            set(&mut e, k, v)?;
        }
        Ok(e)
    }
    fn discovery(
        root: &Path,
        request: &Element,
        actor: &Principal,
        approved: &Approved,
    ) -> io::Result<Element> {
        if !request.is("iq", "jabber:client") {
            return Err(invalid());
        }
        let id = request.attr("id").ok_or_else(invalid)?;
        if matches!(id, "pub-bundle" | "pub-devices") {
            return iq(
                id,
                actor.account.split_once('@').ok_or_else(invalid)?.1,
                &actor.account,
            );
        }
        let peer = match request.attr("to") {
            Some(who) if who == approved.alice.account => &approved.alice,
            Some(who) if who == approved.bob.account => &approved.bob,
            _ => return Err(invalid()),
        };
        let mut reply = iq(id, &peer.account, &actor.account)?;
        let query = request
            .get_child("pubsub", PUBSUB)
            .and_then(|x| x.get_child("items", PUBSUB))
            .ok_or_else(invalid)?;
        let node = query.attr("node").ok_or_else(invalid)?;
        let mut items = Element::builder("items", PUBSUB).build();
        set(&mut items, "node", node)?;
        if node == mail_wire::DEVICES {
            let mut item = Element::builder("item", PUBSUB).build();
            set(&mut item, "id", "current")?;
            let mut devices = Element::builder("devices", mail_wire::OMEMO).build();
            let mut device = Element::builder("device", mail_wire::OMEMO).build();
            set(&mut device, "id", &peer.device.to_string())?;
            devices.append_child(device);
            item.append_child(devices);
            items.append_child(item);
        } else if node == mail_wire::BUNDLES {
            let name = if peer == &approved.alice {
                "alice"
            } else {
                "bob"
            };
            let export: Element =
                mapped(fs::read_to_string(root.join(format!("{name}-bundle.xml")))?.parse())?;
            let item = export
                .get_child("pubsub", PUBSUB)
                .and_then(|x| x.get_child("publish", PUBSUB))
                .and_then(|x| x.get_child("item", PUBSUB))
                .ok_or_else(invalid)?;
            items.append_child(item.clone());
        } else {
            return Err(invalid());
        }
        let mut pubsub = Element::builder("pubsub", PUBSUB).build();
        pubsub.append_child(items);
        reply.append_child(pubsub);
        Ok(reply)
    }
    fn handler(root: &Path) -> io::Result<()> {
        let public = match mapped(blobs::decode_request(&fs::read(
            root.join("public-principals.bin"),
        )?))? {
            Request::Begin(b) => b,
            _ => return Err(invalid()),
        };
        let mut approved = Approved {
            alice: public.sender,
            bob: public.recipient,
            revoked: false,
        };
        let mut store = mapped(blobs::Store::open(&root.join("serverstore")))?;
        let mut input = io::stdin().lock();
        let mut output = io::stdout().lock();
        loop {
            let mut length = [0; 4];
            let n = input.read(&mut length[..1])?;
            if n == 0 {
                return Ok(());
            }
            input.read_exact(&mut length[1..])?;
            let length = u32::from_be_bytes(length) as usize;
            if !(2..=16385).contains(&length) {
                return Err(invalid());
            }
            let mut bytes = vec![0; length];
            input.read_exact(&mut bytes)?;
            let actor = match bytes[0] {
                0 => approved.alice.clone(),
                1 => approved.bob.clone(),
                _ => return Err(invalid()),
            };
            let request: Element = mapped(
                std::str::from_utf8(&bytes[1..])
                    .map_err(|_| invalid())?
                    .parse(),
            )?;
            let response = if request.get_child("blob", mail_blob_wire::NS).is_some() {
                approved.revoked = root.join("server-revoked").exists();
                let service = request.attr("to").ok_or_else(invalid)?;
                if service != "home.test" {
                    return Err(invalid());
                }
                let decoded = mapped(mail_blob_wire::decode_request(
                    &request,
                    &actor.account,
                    service,
                ))?;
                mapped(mail_blob_wire::response(
                    &request,
                    service,
                    store.dispatch(&actor, &approved, decoded, now()?),
                ))?
            } else {
                discovery(root, &request, &actor, &approved)?
            };
            let response = xml(&response)?;
            if response.len() > 16384 {
                return Err(invalid());
            }
            output.write_all(&(response.len() as u32).to_be_bytes())?;
            output.write_all(&response)?;
            output.flush()?;
        }
    }
    pub fn run() -> io::Result<()> {
        let args: Vec<_> = std::env::args().skip(1).collect();
        if args.len() < 2 {
            return Err(invalid());
        }
        let action = args[0].as_str();
        let root = Path::new(&args[1]);
        if !root.is_absolute()
            || fs::read(root.join("OWNED"))? != b"Owned disposable attachment TLS proof\n"
        {
            return Err(invalid());
        }
        if action == "handler" {
            return handler(root);
        }
        let mut input = Zeroizing::new(String::new());
        io::stdin().take(8193).read_to_string(&mut input)?;
        let lines: Vec<_> = input.lines().collect();
        if input.len() > 8192 || lines.len() != 2 || lines[0].len() < 12 {
            return Err(invalid());
        }
        if action == "init" {
            return init(root, lines[0]);
        }
        let stamp = now()?;
        let name = if matches!(
            action,
            "fetch" | "fetch-uninvited" | "fetch-wrong-frame" | "receive" | "verify" | "save"
        ) {
            "bob"
        } else {
            "alice"
        };
        let mut vault = load(root, name, lines[0])?;
        if action == "audit" {
            audit(root, &vault)?;
            println!("ok");
            return Ok(());
        }
        if action == "receive" {
            let raw = fs::read_to_string(root.join("delivery.xml"))?;
            if raw.len() > 16384 {
                return Err(invalid());
            }
            let e: Element = mapped(raw.parse())?;
            let s = vault.inspection()?;
            let peer = s.peer.ok_or_else(invalid)?;
            let wire = mail_wire::decode_message(
                &e,
                mail_wire::Binding {
                    peer: peer.address,
                    peer_device: peer.device,
                    peer_identity: *peer.identity,
                    own: &vault.legacy()?.home,
                    own_device: s.local_device,
                },
            )?;
            vault.receive_network(&wire, stamp)?;
            println!("ok");
            return Ok(());
        }
        let mut spool = mapped(Spool::open(&root.join(format!("{name}-spool"))))?;
        match action {
            "prepare" => {
                let size: usize = args
                    .get(2)
                    .ok_or_else(invalid)?
                    .parse()
                    .map_err(|_| invalid())?;
                if size > 1_048_576 {
                    return Err(invalid());
                }
                let sent_at = vault
                    .history()?
                    .iter()
                    .find(|r| r.direction == Direction::Sent && r.operation() == OP)
                    .map_or(stamp, |r| r.sent_at);
                mapped(transfer::prepare(
                    &mut vault,
                    &mut spool,
                    OP,
                    Selection {
                        name: "private-selected-name.bin",
                        media_type: "application/octet-stream",
                        bytes: &data(size),
                        subject: "private-selected-subject",
                        body: "private-selected-body",
                        sent_at,
                    },
                    stamp,
                ))?;
                println!("ok");
            }
            "upload" | "fetch" | "fetch-uninvited" | "fetch-wrong-frame" | "sync" => {
                let settings = Settings::load(Path::new(args.get(2).ok_or_else(invalid)?))?;
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                if action == "sync" {
                    let report = mapped(rt.block_on(mail_network::cycle(
                        &mut vault,
                        &settings,
                        lines[1],
                        Mode::Sync,
                        stamp,
                    )))?;
                    println!("progress 0 0 {}", report.sent);
                } else {
                    let report = if action == "upload" {
                        mapped(rt.block_on(transfer::upload_cycle(
                            &mut vault, &mut spool, &settings, lines[1], OP, stamp,
                        )))?
                    } else {
                        let mut frame = vault
                            .history()?
                            .first()
                            .and_then(|r| r.received_id())
                            .ok_or_else(invalid)?;
                        if action == "fetch-wrong-frame" {
                            frame[0] ^= 1;
                        }
                        let homes: &[&str] = if action == "fetch-uninvited" {
                            &[]
                        } else {
                            &["home.test"]
                        };
                        let result = rt.block_on(transfer::fetch_cycle(
                            &vault,
                            &mut spool,
                            &settings,
                            lines[1],
                            &mapped(InvitedHomes::new(homes))?,
                            transfer::ReceivedSelection { index: 0, frame },
                            stamp,
                        ));
                        if action == "fetch-uninvited" {
                            if matches!(result, Err(transfer::Error::Route)) {
                                println!("refused route");
                            }
                            return Err(invalid());
                        }
                        if action == "fetch-wrong-frame" {
                            if matches!(result, Err(transfer::Error::Selection)) {
                                println!("refused selection");
                            }
                            return Err(invalid());
                        }
                        mapped(result)?
                    };
                    println!(
                        "progress {} {} {}",
                        report.offset,
                        u8::from(report.complete),
                        report.sent
                    );
                }
            }
            "verify" | "save" => {
                let frame = vault
                    .history()?
                    .first()
                    .and_then(|r| r.received_id())
                    .ok_or_else(invalid)?;
                let verified = mapped(transfer::verify_fetched(
                    &vault, &mut spool, 0, frame, stamp,
                ))?;
                if verified.as_bytes() != data(verified.as_bytes().len()) {
                    return Err(invalid());
                }
                if action == "save" {
                    mapped(transfer::save_fetched(
                        &vault,
                        &mut spool,
                        0,
                        frame,
                        stamp,
                        Path::new(args.get(2).ok_or_else(invalid)?),
                    ))?;
                }
                println!("ok");
            }
            _ => return Err(invalid()),
        }
        Ok(())
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn privacy_audit_detects_every_selected_secret_without_disclosing_it() {
            let needles: Vec<&[u8]> = vec![b"private-name", &[4; 32], &[8; 12], b"private-body"];
            assert!(!contains_private(b"opaque unrelated ciphertext", &needles));
            for secret in &needles {
                assert!(contains_private(secret, &needles));
            }
        }
    }
}
fn main() {
    #[cfg(feature = "network-mail")]
    if enabled::run().is_err() {
        std::process::exit(1)
    }
}
