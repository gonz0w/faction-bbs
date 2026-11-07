//! Controlled cycle results exercise the real Screen consumer and canonical vault.
//! These controls do not claim successful transport or provider interoperability.
use super::*;
use faction_client::mail_checkpoint::Wire;
use faction_client::mail_network::{ArchiveReport, Error, Mode, Report, Settings};
use faction_omemo::{Device, sce};
use std::{cell::RefCell, io::Write, os::unix::fs::PermissionsExt};

#[test]
fn home_exchange_actual_consumer_preserves_scope_on_success_and_partial_failure() {
    let before = rustix::process::getrlimit(rustix::process::Resource::Core);
    // Native device allocation disables core dumps for its process. Exercise the
    // full UI fixture in a child so other tests retain their inherited limits.
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "checkpoint_ui::exchange_tests::home_exchange_actual_consumer_helper",
            "--ignored",
            "--nocapture",
        ])
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "controlled UI exchange fixture failed: {} {}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    let after = rustix::process::getrlimit(rustix::process::Resource::Core);
    assert_eq!(
        (after.current, after.maximum),
        (before.current, before.maximum),
        "controlled UI exchange must preserve the parent test process core limits"
    );
}

#[test]
#[ignore = "owned subprocess helper, exercised by the home exchange consumer control"]
fn home_exchange_actual_consumer_helper() {
    let mut random = [0; 16];
    getrandom::fill(&mut random).unwrap();
    let root = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("faction344-exchange-OWNED-{}", hex(&random)));
    std::fs::create_dir(&root).unwrap();
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(
        root.join("OWNERSHIP"),
        b"synthetic controlled UI exchange\n",
    )
    .unwrap();
    let ca = root.join("ca");
    std::fs::write(&ca, b"unused controlled-cycle CA").unwrap();
    std::fs::set_permissions(&ca, std::fs::Permissions::from_mode(0o600)).unwrap();
    let config = root.join("home.toml");
    std::fs::write(&config, format!("version=1\naccount=\"alice@home.test\"\nendpoint=\"127.0.0.1:9\"\ndomain=\"home.test\"\nca={:?}\npin=\"{}\"\n", ca.to_str().unwrap(), "00".repeat(32))).unwrap();
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o600)).unwrap();
    let settings = Settings::load(&config).unwrap();
    for unicode in [false, true] {
        for surface in ["global", "group", "conversation"] {
            for (partial, action, filter) in [
                (false, b'N', inbox::Filter::All),
                (true, b'N', inbox::Filter::All),
                (false, b'A', inbox::Filter::All),
                (true, b'A', inbox::Filter::All),
                (false, b'N', inbox::Filter::Unread),
                (true, b'N', inbox::Filter::Unread),
                (false, b'A', inbox::Filter::Unread),
                (true, b'A', inbox::Filter::Unread),
            ] {
                let path = root.join(format!("{surface}-{unicode}-{partial}-{action}-{filter:?}"));
                let time = now().unwrap();
                let (mut vault, _) = CheckpointVault::create(
                    Store::at(&path, true).unwrap(),
                    Data::new("alice@home.test".into()).unwrap(),
                    b"synthetic controlled password",
                    101,
                    time,
                )
                .unwrap();
                let mut peer = Device::fresh().unwrap();
                let local = vault.bundle().unwrap();
                let remote = peer.bundle().unwrap();
                vault
                    .enroll(
                        "bob@away.test",
                        202,
                        &format!("ed25519:{}", hex(&remote.identity)),
                        time,
                    )
                    .unwrap();
                peer.trust(&format!("ed25519:{}", hex(&local.identity)))
                    .unwrap();
                peer.start(&local).unwrap();
                let receive = |v: &mut CheckpointVault, peer: &mut Device, subject: &str| {
                    let plain = sce::encode(
                        "bob@away.test",
                        "alice@home.test",
                        time,
                        subject,
                        "literal body",
                    )
                    .unwrap();
                    let sealed = peer.seal(&plain).unwrap();
                    v.receive(
                        &Wire {
                            namespace: "urn:xmpp:omemo:2".into(),
                            from: "bob@away.test".into(),
                            to: "alice@home.test".into(),
                            device: 202,
                            to_device: 101,
                            identity: remote.identity,
                            sent_at: time,
                            key: sealed.key,
                            prekey: sealed.prekey,
                            payload: sealed.payload,
                        },
                        time,
                    )
                    .unwrap();
                };
                for _ in 0..3 {
                    receive(&mut vault, &mut peer, "keep original");
                }
                vault.enable_archive(time).unwrap();
                vault.enable_read_tracking().unwrap();
                vault
                    .queue([3; 16], "keep pending", "retained", time, time)
                    .unwrap();
                let pending = vault.inspect_outbox().unwrap().len();
                let original = vault.history().unwrap()[1].received_id().unwrap();
                let peer = RefCell::new(peer);
                let calls = RefCell::new(0);
                let cycle = |v: &mut CheckpointVault, password: &str, mode: Mode| {
                    assert_eq!(password, "fresh controlled password");
                    assert_eq!(
                        mode,
                        if action == b'A' {
                            Mode::CatchUp
                        } else {
                            Mode::Sync
                        }
                    );
                    *calls.borrow_mut() += 1;
                    if action == b'A' {
                        receive(v, &mut peer.borrow_mut(), "keep new matching");
                        receive(v, &mut peer.borrow_mut(), "nonmatching");
                    }
                    if partial {
                        Err(Error::Protocol)
                    } else {
                        Ok(if action == b'A' {
                            Report {
                                archive: Some(ArchiveReport {
                                    entries: 2,
                                    complete: false,
                                }),
                                accepted: 2,
                                ..Report::default()
                            }
                        } else {
                            Report {
                                sent: pending,
                                ..Report::default()
                            }
                        })
                    }
                };
                let exchange = HomeExchange {
                    settings: &settings,
                    cycle: Some(&cycle),
                    foreground: None,
                };
                let (master, slave) = faction_client_pty::open(24, 40).unwrap();
                let mut input = std::fs::File::from(master);
                let saved = rustix::termios::tcgetattr(&slave).unwrap();
                let mut raw = saved.clone();
                raw.make_raw();
                rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw)
                    .unwrap();
                let output = path.join("capture");
                let mut term = LocalTerminal {
                    input: slave.into(),
                    output: std::fs::File::create(&output).unwrap(),
                    saved,
                };
                let mut list = inbox::Inbox::new(
                    Screen::inbox_group_entries(
                        &vault,
                        if surface == "global" { None } else { Some(0) },
                    )
                    .unwrap(),
                );
                list.filter(filter);
                let mut search = inbox::Search::parse(b"keep").unwrap();
                list.search(Screen::inbox_matches(&vault, &search).unwrap());
                list.select_index(1);
                std::thread::scope(|scope| {
                    let driver = scope.spawn(|| {
                        let mut offset = 0;
                        let mut wait = |needle: &str| {
                            let limit = Instant::now() + Duration::from_secs(15);
                            loop {
                                let text = std::fs::read_to_string(&output).unwrap();
                                if let Some(at) = text[offset..].find(needle) {
                                    offset += at + needle.len();
                                    break;
                                }
                                assert!(
                                    Instant::now() < limit,
                                    "exchange route did not reach {needle}"
                                );
                                std::thread::sleep(Duration::from_millis(10));
                            }
                        };
                        wait("[B] Back");
                        input.write_all(b"x").unwrap();
                        wait("[B] Return");
                        input.write_all(&[action.to_ascii_lowercase()]).unwrap();
                        wait("HOME account password (hidden; never sent to a BBS): ");
                        input.write_all(b"fresh controlled password\r").unwrap();
                        wait("Enter to return: ");
                        input.write_all(b"\r").unwrap();
                        wait("[B] Back");
                        input.write_all(b"b").unwrap();
                        input
                    });
                    let mut screen = Screen {
                        term: &mut term,
                        activity: Instant::now(),
                        unicode,
                    };
                    if surface == "conversation" {
                        screen
                            .conversation(&mut vault, true, Some(&exchange), 0, &mut list, &search)
                            .unwrap();
                    } else {
                        screen
                            .inbox_view(
                                &mut vault,
                                true,
                                Some(&exchange),
                                if surface == "global" { None } else { Some(0) },
                                &mut list,
                                &mut search,
                            )
                            .unwrap();
                    }
                    driver.join().unwrap();
                });
                drop(term);
                assert_eq!(*calls.borrow(), 1);
                assert_eq!(
                    vault.read_summary().unwrap().unread,
                    Some(if action == b'A' { 5 } else { 3 })
                );
                assert_eq!(vault.received_read(1, original).unwrap(), Some(false));
                assert_eq!(vault.inspect_outbox().unwrap().len(), pending);
                assert_eq!(list.mode(), filter);
                assert_eq!(list.selected().unwrap().index, 1);
                assert_eq!(
                    list.eligible().iter().map(|e| e.index).collect::<Vec<_>>(),
                    match (action, filter) {
                        (b'A', inbox::Filter::All) => vec![4, 3, 2, 1, 0],
                        (b'A', inbox::Filter::Unread) => vec![4, 2, 1, 0],
                        (_, inbox::Filter::All) => vec![3, 2, 1, 0],
                        (_, inbox::Filter::Unread) => vec![2, 1, 0],
                    }
                );
                let text = std::fs::read_to_string(&output).unwrap();
                let label = if surface == "conversation" {
                    "[LOCAL CONVERSATION]"
                } else if surface == "group" {
                    "[LOCAL CORRESPONDENT INBOX]"
                } else {
                    "[LOCAL INBOX]"
                };
                let returned = &text[text.rfind(label).unwrap()..];
                assert!(returned.contains("> 02 "));
                assert!(
                    returned.contains("Local search active.")
                        || returned.contains(if filter == inbox::Filter::Unread {
                            "Unread / Search"
                        } else {
                            "All / Search"
                        })
                );
                assert!(!text.contains("fresh controlled password"));
                assert_eq!(text.matches("HOME account password").count(), 1);
                assert!(text.contains(if partial {
                    if action == b'A' {
                        "Catch-up stopped; earlier accepted entries may already be saved."
                    } else {
                        "Home cycle stopped; pending ciphertext retained."
                    }
                } else if action == b'A' {
                    "Archive page saved."
                } else {
                    "Manual cycle complete."
                }));
                drop(vault);
                let reopened = CheckpointVault::unlock(
                    Store::at(&path, false).unwrap(),
                    b"synthetic controlled password",
                    false,
                )
                .unwrap();
                assert_eq!(
                    reopened.history().unwrap().len(),
                    if action == b'A' { 6 } else { 4 }
                );
            }
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
