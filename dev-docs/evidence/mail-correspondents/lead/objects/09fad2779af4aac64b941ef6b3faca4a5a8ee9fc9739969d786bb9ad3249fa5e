//! Disposable synthetic correspondent fixtures only. Never a product unlock/input path.
//! The known password protects synthetic lab data; private inspection stays out of Git.
#[cfg(feature = "native-mail")]
mod fixture {
    use faction_client::{
        mail_checkpoint::{CheckpointVault, Direction, Wire},
        mail_data::{Data, hex, invalid},
        mail_vault::Store,
    };
    use faction_omemo::{Device, Message, sce};
    use std::{
        io::{self, Read},
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };
    use zeroize::Zeroizing;

    const PASS: &[u8] = b"checkpoint synthetic fixture password";

    fn json(value: &str) -> String {
        let mut out = String::from("\"");
        for c in value.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out.push('"');
        out
    }

    fn inspect(path: &Path, now: u64) -> io::Result<()> {
        let vault = CheckpointVault::unlock(Store::at(path, false)?, PASS, false)?;
        let mut history = Vec::new();
        for (index, row) in vault.history()?.iter().enumerate() {
            let received = row.received_id();
            let read = received
                .map(|id| vault.received_read(index, id))
                .transpose()?
                .flatten();
            history.push(format!(
                "{{\"direction\":{},\"address\":{},\"device\":{},\"identity\":{},\"sent_at\":{},\"subject\":{},\"body\":{},\"operation\":{},\"received_id\":{},\"read\":{}}}",
                json(if row.direction == Direction::Received { "received" } else { "sent" }),
                json(&row.address), row.device, json(&hex(&row.identity)), row.sent_at,
                json(&row.subject), json(&row.body), json(&hex(&row.operation())),
                received.map(|id| json(&hex(&id))).unwrap_or_else(|| "null".into()),
                read.map(|value| value.to_string()).unwrap_or_else(|| "null".into())
            ));
        }
        let pending = vault
            .inspect_outbox()?
            .iter()
            .map(|row| {
                format!(
                    "{{\"operation\":{},\"address\":{},\"device\":{},\"sent_at\":{}}}",
                    json(&hex(&row.operation)),
                    json(row.address),
                    row.device,
                    row.sent_at
                )
            })
            .collect::<Vec<_>>();
        let state = vault.inspection()?;
        let token = vault.archive_checkpoint(now).ok();
        let archive = format!(
            "{{\"enabled\":{},\"cursor_present\":{},\"cursor\":{}}}",
            state.archive_enabled,
            state.archive_cursor_present,
            token
                .and_then(|token| token.after)
                .map(|value| json(&value))
                .unwrap_or_else(|| "null".into())
        );
        println!(
            "{{\"history\":[{}],\"pending\":[{}],\"archive\":{}}}",
            history.join(","),
            pending.join(","),
            archive
        );
        Ok(())
    }

    pub fn run() -> io::Result<()> {
        let args = std::env::args().skip(1).collect::<Vec<_>>();
        if !(2..=3).contains(&args.len()) {
            return Err(invalid());
        }
        let path = Path::new(&args[1]);
        if !path.is_absolute() {
            return Err(invalid());
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| invalid())?
            .as_secs();
        if args[0] == "inspect" && args.len() == 2 {
            return inspect(path, now);
        }
        if args[0] != "seed" || args.len() != 3 {
            return Err(invalid());
        }
        let mode = args[2].as_str();
        if ![
            "ready",
            "unknown",
            "empty",
            "single",
            "full",
            "revoked",
            "history-only",
        ]
        .contains(&mode)
        {
            return Err(invalid());
        }
        // Actual generated account labels and message markers arrive only on stdin.
        let mut input = Zeroizing::new(String::new());
        io::stdin().take(2049).read_to_string(&mut input)?;
        let lines = input.lines().collect::<Vec<_>>();
        if input.len() > 2048 || !input.ends_with('\n') || lines.len() != 4 {
            return Err(invalid());
        }
        let (mut a, recovery) = CheckpointVault::create(
            Store::at(path, true)?,
            Data::new("alice@home.test".into())?,
            PASS,
            101,
            now,
        )?;
        if mode != "unknown" {
            a.enable_read_tracking()?;
        }
        if mode == "empty" {
            return Ok(());
        }
        let count = if mode == "full" {
            64
        } else if mode == "single" {
            1
        } else {
            4
        };
        for phase in 0..count {
            let local = 101 + phase as u32;
            if phase > 0 {
                a.fresh_device(local, now)?;
            }
            let account = if mode == "full" {
                format!("p{phase:02}.{}", lines[2])
            } else {
                lines[if phase == 2 { 3 } else { 2 }].to_string()
            };
            let device = if mode == "full" {
                202 + phase as u32
            } else if phase == 1 {
                203
            } else {
                202
            };
            let mut b = Device::fresh().map_err(|_| invalid())?;
            let ab = a.bundle()?;
            let bb = b.bundle().map_err(|_| invalid())?;
            a.enroll(
                &account,
                device,
                &format!("ed25519:{}", hex(&bb.identity)),
                now,
            )?;
            a.enable_archive(now)?;
            b.trust(&format!("ed25519:{}", hex(&ab.identity)))
                .map_err(|_| invalid())?;
            b.start(&ab).map_err(|_| invalid())?;
            let subject = format!(
                "{} recv {phase:02} λ界\t\u{202e} {}",
                lines[0],
                "x".repeat(40)
            );
            let plain = sce::encode(&account, "alice@home.test", now, &subject, lines[1])
                .map_err(|_| invalid())?;
            let incoming = b.seal(&plain).map_err(|_| invalid())?;
            let token = a.archive_checkpoint(now)?;
            a.receive_archive(
                &token,
                &format!("synthetic-correspondents-position-{phase}"),
                &Wire {
                    namespace: "urn:xmpp:omemo:2".into(),
                    from: account.clone(),
                    to: "alice@home.test".into(),
                    device,
                    to_device: local,
                    identity: bb.identity,
                    sent_at: 0,
                    key: incoming.key,
                    prekey: incoming.prekey,
                    payload: incoming.payload,
                },
                now,
            )?;
            if phase == 1 && mode != "unknown" {
                let index = a.history()?.len() - 1;
                let id = a.history()?[index].received_id().ok_or_else(invalid)?;
                a.mark_read(index, id)?;
            }
            if mode != "full" {
                let operation = [phase as u8 + 1; 16];
                let wire = a.queue(
                    operation,
                    &format!("{} sent {phase:02}", lines[0]),
                    lines[1],
                    now,
                    now,
                )?;
                let plain = b
                    .open_pending(&Message {
                        key: wire.key,
                        prekey: wire.prekey,
                        payload: wire.payload,
                    })
                    .map_err(|_| invalid())?;
                drop(plain);
                b.commit().map_err(|_| invalid())?;
                if phase + 1 != count {
                    a.acknowledge(operation)?;
                }
            }
        }
        if mode == "revoked" {
            a.revoke()?;
        }
        if mode == "history-only" {
            drop(a);
            let _ = CheckpointVault::unlock(Store::at(path, false)?, &*recovery, true)?;
        }
        Ok(())
    }
}

#[cfg(feature = "native-mail")]
fn main() {
    if fixture::run().is_err() {
        std::process::exit(1);
    }
}

#[cfg(not(feature = "native-mail"))]
fn main() {
    eprintln!("Synthetic correspondent fixture requires native-mail feature");
    std::process::exit(1);
}
