//! Synthetic test driver only. Never an alternate product unlock/input path.
use faction_client::{
    mail_checkpoint::{CheckpointVault, Direction},
    mail_data::{Data, hex, invalid},
    mail_network::{self, Mode, Settings},
    mail_vault::Store,
};
use std::{
    io::{self, Read},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use zeroize::Zeroizing;
fn json(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
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
fn run() -> io::Result<()> {
    let args = std::env::args_os()
        .skip(1)
        .map(|s| s.into_string().map_err(|_| invalid()))
        .collect::<io::Result<Vec<_>>>()?;
    if args.len() < 2 {
        return Err(invalid());
    }
    let action = args[0].as_str();
    let path = Path::new(&args[1]);
    let mut input = Zeroizing::new(String::new());
    io::stdin().take(8193).read_to_string(&mut input)?;
    if input.len() > 8192 || !input.ends_with('\n') {
        return Err(invalid());
    }
    let fields: Vec<_> = input.lines().collect();
    let pass = fields.first().ok_or_else(invalid)?;
    if pass.len() < 12 || pass.len() > 128 {
        return Err(invalid());
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| invalid())?
        .as_secs();
    if action == "init" && args.len() == 4 && fields.len() == 1 {
        let device = args[3].parse().map_err(|_| invalid())?;
        let (v, _recovery) = CheckpointVault::create(
            Store::at(path, true)?,
            Data::new(args[2].clone())?,
            pass.as_bytes(),
            device,
            now,
        )?;
        println!("{}", hex(v.inspection()?.identity));
        return Ok(());
    }
    let mut vault = CheckpointVault::unlock(Store::at(path, false)?, pass.as_bytes(), false)?;
    match action {
        "enroll" if args.len() == 5 && fields.len() == 1 => {
            vault.enroll(
                &args[2],
                args[3].parse().map_err(|_| invalid())?,
                &format!("ed25519:{}", args[4]),
                now,
            )?;
        }
        "publish" | "prepare" | "sync" if args.len() == 3 && fields.len() == 2 => {
            let settings = Settings::load(Path::new(&args[2]))?;
            let mode = match action {
                "publish" => Mode::Publish,
                "prepare" => Mode::Prepare,
                _ => Mode::Sync,
            };
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            if let Err(error) = rt.block_on(mail_network::cycle(
                &mut vault, &settings, fields[1], mode, now,
            )) {
                eprintln!("{error:?}");
                return Err(invalid());
            }
        }
        "queue" if args.len() == 2 && fields.len() == 3 => {
            let mut id = [0; 16];
            getrandom::fill(&mut id).map_err(|_| invalid())?;
            vault.queue(id, fields[1], fields[2], now, now)?;
        }
        "inspect" if args.len() == 2 && fields.len() == 1 => {
            let mut history = Vec::new();
            for row in vault.history()? {
                let direction = if row.direction == Direction::Sent {
                    "sent"
                } else {
                    "received"
                };
                history.push(format!(
                    "{{\"direction\":{},\"subject\":{},\"body\":{},\"sent_at\":{},\"address\":{}}}",
                    json(direction),
                    json(&row.subject),
                    json(&row.body),
                    row.sent_at,
                    json(&row.address)
                ));
            }
            let pending = vault
                .inspect_outbox()?
                .into_iter()
                .map(|r| {
                    format!(
                        "{{\"operation\":{},\"address\":{}}}",
                        json(&hex(&r.operation)),
                        json(r.address)
                    )
                })
                .collect::<Vec<_>>();
            println!(
                "{{\"history\":[{}],\"pending\":[{}]}}",
                history.join(","),
                pending.join(",")
            );
            return Ok(());
        }
        _ => return Err(invalid()),
    }
    println!("ok");
    Ok(())
}
fn main() {
    if run().is_err() {
        std::process::exit(1)
    }
}
