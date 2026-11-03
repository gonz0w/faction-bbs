//! Synthetic member-device writing controls only; never a product input route.
#[cfg(feature = "native-mail")]
mod fixture {
    use faction_client::{
        mail_checkpoint::{CheckpointVault, Direction, Wire},
        mail_data::{Data, hex, invalid},
        mail_vault::Store,
    };
    use faction_omemo::{Device, sce};
    use std::{
        io::{self, Read},
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };
    use zeroize::Zeroizing;
    const PASS: &[u8] = b"checkpoint synthetic fixture password";
    fn json(text: &str) -> String {
        let mut result = String::from("\"");
        for c in text.chars() {
            match c {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                c if c.is_control() => result.push_str(&format!("\\u{:04x}", c as u32)),
                c => result.push(c),
            }
        }
        result.push('"');
        result
    }
    fn inspect(path: &Path, now: u64) -> io::Result<()> {
        let vault = CheckpointVault::unlock(Store::at(path, false)?, PASS, false)?;
        let mut rows = Vec::new();
        for (index, row) in vault.history()?.iter().enumerate() {
            let read = row
                .received_id()
                .map(|id| vault.received_read(index, id))
                .transpose()?
                .flatten();
            rows.push(format!("{{\"direction\":{},\"address\":{},\"device\":{},\"identity\":{},\"sent_at\":{},\"subject\":{},\"body\":{},\"operation\":{},\"read\":{}}}",json(if row.direction==Direction::Received {"received"} else {"sent"}),json(&row.address),row.device,json(&hex(&row.identity)),row.sent_at,json(&row.subject),json(&row.body),json(&hex(&row.operation())),read.map(|v|v.to_string()).unwrap_or_else(||"null".into())));
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
        let cursor = vault
            .archive_checkpoint(now)
            .ok()
            .and_then(|token| token.after)
            .map(|s| json(&s))
            .unwrap_or_else(|| "null".into());
        println!(
            "{{\"history\":[{}],\"pending\":[{}],\"archive\":{{\"enabled\":{},\"cursor_present\":{},\"cursor\":{}}}}}",
            rows.join(","),
            pending.join(","),
            state.archive_enabled,
            state.archive_cursor_present,
            cursor
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
        let wall = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| invalid())?
            .as_secs();
        if args[0] == "inspect" && args.len() == 2 {
            return inspect(path, wall);
        }
        if args[0] != "seed"
            || args.len() != 3
            || ![
                "ready",
                "revoked",
                "unprepared",
                "history-only",
                "expired",
                "full",
                "pending-full",
            ]
            .contains(&args[2].as_str())
        {
            return Err(invalid());
        }
        // Two bounded header lines, then the exact remaining UTF-8 body, including final LF.
        let mut input = Zeroizing::new(String::new());
        io::stdin().take(2049).read_to_string(&mut input)?;
        if input.len() > 2048 {
            return Err(invalid());
        }
        let mut fields = input.splitn(3, '\n');
        let subject = fields.next().ok_or_else(invalid)?;
        let account = fields.next().ok_or_else(invalid)?;
        let body = fields.next().ok_or_else(invalid)?;
        if subject.is_empty() || subject.len() > 128 || body.is_empty() || body.len() > 1024 {
            return Err(invalid());
        }
        let now = if args[2] == "expired" {
            wall.saturating_sub(31 * 86400)
        } else {
            wall
        };
        let (mut a, recovery) = CheckpointVault::create(
            Store::at(path, true)?,
            Data::new("alice@home.test".into())?,
            PASS,
            101,
            now,
        )?;
        a.enable_read_tracking()?;
        let mut b = Device::fresh().map_err(|_| invalid())?;
        let ab = a.bundle()?;
        let bb = b.bundle().map_err(|_| invalid())?;
        a.enroll(account, 202, &format!("ed25519:{}", hex(&bb.identity)), now)?;
        a.enable_archive(now)?;
        b.trust(&format!("ed25519:{}", hex(&ab.identity)))
            .map_err(|_| invalid())?;
        b.start(&ab).map_err(|_| invalid())?;
        let plain =
            sce::encode(account, "alice@home.test", now, subject, body).map_err(|_| invalid())?;
        let incoming = b.seal(&plain).map_err(|_| invalid())?;
        a.receive(
            &Wire {
                namespace: "urn:xmpp:omemo:2".into(),
                from: account.into(),
                to: "alice@home.test".into(),
                device: 202,
                to_device: 101,
                identity: bb.identity,
                sent_at: now,
                key: incoming.key,
                prekey: incoming.prekey,
                payload: incoming.payload,
            },
            now,
        )?;
        match args[2].as_str() {
            "revoked" => a.revoke()?,
            "unprepared" => {
                a.fresh_device(102, now)?;
                a.enroll(account, 202, &format!("ed25519:{}", hex(&bb.identity)), now)?;
            }
            "history-only" => {
                drop(a);
                drop(CheckpointVault::unlock(
                    Store::at(path, false)?,
                    &*recovery,
                    true,
                )?);
                return Ok(());
            }
            "full" | "pending-full" => {
                let full = args[2] == "full";
                for n in 1..=if full { 63 } else { 8 } {
                    a.queue([n; 16], "synthetic capacity filler", "bounded", now, now)?;
                    if full {
                        a.acknowledge([n; 16])?;
                    }
                }
            }
            _ => {}
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
    eprintln!("Synthetic writing fixture requires native-mail");
    std::process::exit(1);
}
