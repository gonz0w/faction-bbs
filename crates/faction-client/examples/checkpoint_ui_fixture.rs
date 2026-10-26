//! Lab-only synthetic fixture for probe_checkpoint_ui.py. No provider/listener.
//! This known passphrase protects disposable synthetic data only; never real mail.
#[cfg(feature = "native-mail")]
#[allow(dead_code)]
#[path = "../src/stream.rs"]
mod stream;

#[cfg(feature = "native-mail")]
fn hostile_stream() -> std::io::Result<()> {
    use rustix::termios::{self, OptionalActions, QueueSelector};
    use std::io::{Read, Write};
    struct Restore(termios::Termios);
    impl Drop for Restore {
        fn drop(&mut self) {
            let _ = termios::tcflush(std::io::stdin(), QueueSelector::IFlush);
            let _ = termios::tcsetattr(std::io::stdin(), OptionalActions::Now, &self.0);
        }
    }
    let saved = termios::tcgetattr(std::io::stdin())?;
    let mut raw = saved.clone();
    raw.make_raw();
    termios::tcsetattr(std::io::stdin(), OptionalActions::Now, &raw)?;
    let _restore = Restore(saved);
    std::io::stdout().write_all(b"REMOTE FILTER PTY READY\r\n")?;
    let mut filter = stream::Filter::default();
    let mut filtered = Vec::new();
    for _ in 0..8192 {
        let mut byte = [0];
        std::io::stdin().read_exact(&mut byte)?;
        if byte[0] == b'\n' {
            std::io::stdout().write_all(&filtered)?;
            std::io::stdout()
                .write_all(b"\r\nREMOTE FILTER TEXT ONLY; no local store API called\r\n")?;
            return Ok(());
        }
        for event in filter.feed(&byte)? {
            match event {
                stream::Event::Text(text) => filtered.extend(text),
                stream::Event::Frame(_) => {
                    return Err(std::io::Error::other(
                        "unsolicited file frame is not checkpoint authority",
                    ));
                }
            }
        }
    }
    Err(std::io::Error::other(
        "bounded remote fixture input exceeded",
    ))
}
#[cfg(feature = "native-mail")]
fn main() -> std::io::Result<()> {
    use faction_client::{
        mail_checkpoint::{CheckpointVault, SESSION_AGE, Wire},
        mail_data::{Data, hex},
        mail_vault::{Store, Vault},
    };
    use faction_omemo::{Device, Message, sce};
    use std::{
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };
    use zeroize::Zeroizing;
    const PASS: &[u8] = b"checkpoint synthetic fixture password";
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    assert_eq!(
        args.len(),
        2,
        "fixture requires owned absolute directory and public scenario"
    );
    let path = Path::new(&args[0]);
    assert!(path.is_absolute());
    let mode = args[1].as_str();
    if mode == "inspect-unread" {
        let v = CheckpointVault::unlock(Store::at(path, false)?, PASS, false)?;
        let summary = v.read_summary()?;
        let state = v.inspection()?;
        println!(
            "{{\"enabled\":{},\"unread\":{},\"received\":{},\"retained\":{},\"pending\":{},\"history_only\":{},\"archive_enabled\":{},\"archive_cursor_present\":{}}}",
            summary.unread.is_some(),
            summary
                .unread
                .map(|n| n.to_string())
                .unwrap_or_else(|| "null".into()),
            summary.received,
            summary.retained,
            summary.pending,
            summary.history_only,
            state.archive_enabled,
            state.archive_cursor_present
        );
        return Ok(());
    }
    if mode == "hostile-stream" {
        return hostile_stream();
    }
    let mut data = Data::new("alice@home.test".into())?;
    data.draft(
        "bob@away.test".into(),
        Zeroizing::new("legacy draft".into()),
        Zeroizing::new("synthetic legacy body".into()),
    )?;
    data.pin("bob@away.test".into(), 202, [0xab; 32])?;
    if mode == "legacy" {
        let _ = Vault::create(Store::at(path, true)?, data, PASS)?;
        return Ok(());
    }
    let current = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let created = if mode == "expired" || mode == "unread-expired" {
        current - SESSION_AGE - 10
    } else {
        current
    };
    let (mut a, recovery) =
        CheckpointVault::create(Store::at(path, true)?, data, PASS, 101, created)?;
    let mut b = Device::fresh().unwrap();
    let ab = a.bundle()?;
    let bb = b.bundle().unwrap();
    a.enroll(
        "bob@away.test",
        202,
        &format!("ed25519:{}", hex(&bb.identity)),
        created,
    )?;
    b.trust(&format!("ed25519:{}", hex(&ab.identity))).unwrap();
    a.start(&bb, created)?;
    let outgoing = a.queue(
        [1; 16],
        "synthetic subject λ",
        "inert \nline \u{202e}safe",
        created,
        created,
    )?;
    if mode == "archive-pending" || mode.starts_with("unread") {
        a.enable_archive(created)?;
        let token = a.archive_checkpoint(created)?;
        let mut echo = outgoing.clone();
        echo.sent_at = 0;
        a.receive_archive(&token, "synthetic-private-archive-id", &echo, created)?;
    }
    let decoded = b
        .open_pending(&Message {
            key: outgoing.key,
            prekey: outgoing.prekey,
            payload: outgoing.payload,
        })
        .unwrap();
    drop(decoded);
    b.commit().unwrap();
    let plain = sce::encode(
        "bob@away.test",
        "alice@home.test",
        created,
        if mode == "unread-unicode" {
            "synthetic incoming λ"
        } else {
            "synthetic incoming"
        },
        "synthetic received body",
    )
    .unwrap();
    let incoming = b.seal(&plain).unwrap();
    a.receive(
        &Wire {
            namespace: "urn:xmpp:omemo:2".into(),
            from: "bob@away.test".into(),
            to: "alice@home.test".into(),
            device: 202,
            to_device: 101,
            identity: bb.identity,
            sent_at: created,
            key: incoming.key,
            prekey: incoming.prekey,
            payload: incoming.payload,
        },
        created,
    )?;
    if mode == "full" {
        a.acknowledge([1; 16])?;
        for n in 2..=63 {
            a.queue(
                [n; 16],
                "synthetic bounded history",
                "synthetic full-store fixture",
                created,
                created,
            )?;
            a.acknowledge([n; 16])?;
        }
    }
    if mode == "revoked" || mode == "unread-revoked" {
        a.revoke()?;
    }
    if mode.starts_with("unread") {
        a.enable_read_tracking()?;
    }
    if mode == "unread-history-only" {
        drop(a);
        let _ = CheckpointVault::unlock(Store::at(path, false)?, &*recovery, true)?;
    }
    Ok(())
}
#[cfg(not(feature = "native-mail"))]
fn main() {
    eprintln!("Lab fixture requires native-mail feature");
    std::process::exit(1);
}
