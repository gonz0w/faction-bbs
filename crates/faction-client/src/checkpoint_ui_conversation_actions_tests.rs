//! Conversation actions reuse the qualified real-vault and PTY fixtures.
use super::*;
include!("test_fixture_retention.rs");

#[test]
fn conversation_fixture_retains_executed_outputs_before_cleanup() {
    let mut random = [0; 16];
    getrandom::fill(&mut random).unwrap();
    let configured = std::env::var_os("FACTION359_CAPTURE_DIR");
    let parent = configured
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let sink = parent.join(format!("retention-contract-OWNED-{}", hex(&random)));
    std::fs::create_dir(&sink).unwrap();
    std::fs::set_permissions(&sink, std::fs::Permissions::from_mode(0o700)).unwrap();
    let before = rustix::process::getrlimit(rustix::process::Resource::Core);
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "checkpoint_ui::foreground_tests::actions::conversation_fixture_retention_helper",
            "--ignored",
            "--nocapture",
        ])
        .env("FACTION359_CAPTURE_DIR", &sink)
        .output()
        .unwrap();
    let after = rustix::process::getrlimit(rustix::process::Resource::Core);
    if configured.is_none() {
        std::fs::remove_dir_all(&sink).unwrap();
    }
    assert_eq!(
        (before.current, before.maximum),
        (after.current, after.maximum)
    );
    assert!(
        result.status.success(),
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
#[test]
#[ignore = "owned subprocess helper"]
fn conversation_fixture_retention_helper() {
    let f = Fixture::new();
    assert_eq!(f.vault.pending(f.time).unwrap().len(), 2);
    let root = f.root.clone();
    let expected = std::fs::read(root.join("vault/mail.fkv")).unwrap();
    let sink = std::path::PathBuf::from(std::env::var_os("FACTION359_CAPTURE_DIR").unwrap());
    drop(f);
    assert!(!root.exists());
    assert_eq!(
        std::fs::read(
            sink.join("finished")
                .join(root.file_name().unwrap())
                .join("vault/mail.fkv")
        )
        .unwrap(),
        expected
    );
}

fn retain(f: &Fixture) {
    if let Some(sink) = std::env::var_os("FACTION359_CAPTURE_DIR") {
        capture_owned_test_fixture(&f.root, &std::path::PathBuf::from(sink));
    }
}
pub(super) fn retain_finished(f: &Fixture) {
    if let Some(sink) = std::env::var_os("FACTION359_CAPTURE_DIR") {
        use std::os::unix::fs::DirBuilderExt;
        let sink = std::path::PathBuf::from(sink).join("finished");
        if let Err(error) = std::fs::DirBuilder::new().mode(0o700).create(&sink) {
            assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        }
        capture_owned_test_fixture(&f.root, &sink);
    }
}

#[test]
fn conversation_write_entry() {
    child("checkpoint_ui::foreground_tests::actions::conversation_write_entry_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn conversation_write_entry_helper() {
    for unicode in [false, true] {
        let mut f = Fixture::new();
        assert_eq!(f.vault.pending(f.time).unwrap().len(), 2);
        assert!(conversation::compose_target(&f.vault, &f.settings, 0, f.time).is_ok());
        retain(&f);
        let before = std::fs::read(f.root.join("vault/mail.fkv")).unwrap();
        let (result, text, list) =
            view_controlled(&mut f, true, unicode, 0, inbox::Filter::Unread, None, |d| {
                d.wait("[B] Back");
                d.send(b"b");
            });
        result.unwrap();
        assert!(text.contains("[W] Write fresh text"));
        assert_eq!(list.mode(), inbox::Filter::Unread);
        assert_eq!(list.selected().unwrap().index, 0);
        assert_eq!(
            std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
            before
        );
        let (result, text, _) = view(&mut f, false, unicode, 0, |d| {
            d.wait("[B] Back");
            d.send(b"b");
        });
        result.unwrap();
        assert!(!text.contains("[W] Write fresh text"));
    }
}

#[test]
fn conversation_write_route() {
    child("checkpoint_ui::foreground_tests::actions::conversation_write_route_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn conversation_write_route_helper() {
    let mut f = Fixture::new();
    assert!(conversation::compose_target(&f.vault, &f.settings, 0, f.time).is_ok());
    assert_eq!(f.vault.pending(f.time).unwrap().len(), 2);
    retain(&f);
    let before = std::fs::read(f.root.join("vault/mail.fkv")).unwrap();
    let (result, text, list) =
        view_controlled(&mut f, true, false, 0, inbox::Filter::Unread, None, |d| {
            d.wait("[B] Back");
            d.send(b"w");
            d.wait("Subject (128 UTF-8 bytes): ");
            d.send(b"fresh subject\r");
            d.wait("write> ");
            d.send(b".cancel\r");
            d.wait("Writing cancelled; no ciphertext queued.");
            d.wait("[LOCAL CONVERSATION]");
            d.wait("[B] Back");
            d.send(b"b");
        });
    result.unwrap();
    assert_eq!(list.selected().unwrap().index, 0);
    assert_eq!(list.mode(), inbox::Filter::Unread);
    assert_eq!(
        std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
        before
    );
    assert!(!text.contains("Suggested subject:") && !text.contains("HOME account password"));
}

#[test]
fn conversation_write_queue_return() {
    child("checkpoint_ui::foreground_tests::actions::conversation_write_queue_return_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn conversation_write_queue_return_helper() {
    for unicode in [false, true] {
        for filter in [inbox::Filter::All, inbox::Filter::Unread] {
            let mut f = Fixture::new();
            assert!(conversation::compose_target(&f.vault, &f.settings, 0, f.time).is_ok());
            assert_eq!(f.vault.pending(f.time).unwrap().len(), 2);
            retain(&f);
            let cursor = f.vault.archive_checkpoint(f.time).unwrap();
            let read = f.vault.read_summary().unwrap();
            let (result, text, list) =
                view_controlled(&mut f, true, unicode, 0, filter, None, |d| {
                    d.wait("[B] Back");
                    d.send(b"w");
                    d.wait("Subject (128 UTF-8 bytes): ");
                    d.send("keep fresh λ\r".as_bytes());
                    d.wait("write> ");
                    d.send(b"new first line\r");
                    d.wait("write> ");
                    d.send(b"new second line\r");
                    d.wait("write> ");
                    d.send(b".done\rqueue\r");
                    d.wait("Type queue to persist encrypted ciphertext: ");
                    std::thread::sleep(Duration::from_millis(75));
                    assert!(
                        !std::fs::read_to_string(&d.capture)
                            .unwrap()
                            .contains("Queued durably")
                    );
                    d.send(b"queue\r");
                    d.wait("Queued durably; no delivery or read receipt.");
                    d.wait("[LOCAL CONVERSATION]");
                    d.wait("[B] Back");
                    d.send(b"b");
                });
            result.unwrap();
            let row = &f.vault.history().unwrap()[4];
            assert_eq!(row.subject.as_str(), "keep fresh λ");
            assert_eq!(row.body.as_str(), "new first line\nnew second line");
            assert_eq!(row.address, "bob@away.test");
            assert_eq!(row.device, 202);
            assert_eq!(row.identity, f.remote.bundle().unwrap().identity);
            assert_eq!(f.vault.pending(f.time).unwrap().len(), 3);
            assert_eq!(f.vault.archive_checkpoint(f.time).unwrap(), cursor);
            assert_eq!(f.vault.read_summary().unwrap().unread, read.unread);
            assert_eq!(list.mode(), filter);
            assert_eq!(list.selected().unwrap().index, 0);
            assert!(
                !text.contains("HOME account password") && !text.contains("Suggested subject:")
            );
            assert_eq!(
                list.eligible().len(),
                if filter == inbox::Filter::All { 5 } else { 2 }
            );
            assert!(text[text.rfind("[LOCAL CONVERSATION]").unwrap()..].contains("/ Search"));
            let saved = f.root.join("reopen");
            f.save("reopen");
            let reopened =
                CheckpointVault::unlock(Store::at(&saved, false).unwrap(), SECRET, false).unwrap();
            assert_eq!(
                reopened.history().unwrap()[4].body.as_str(),
                "new first line\nnew second line"
            );
        }
    }
}

#[test]
fn conversation_pending_associations() {
    child("checkpoint_ui::foreground_tests::actions::conversation_pending_associations_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn conversation_pending_associations_helper() {
    let mut f = Fixture::new();
    let target = foreground::Target::prepare(&f.vault, &f.settings, 0, f.time).unwrap();
    assert_eq!(f.vault.pending(f.time).unwrap().len(), 2);
    retain(&f);
    assert_eq!(target.pending(&f.vault).unwrap(), vec![2, 3]);
    f.vault.acknowledge([1; 16]).unwrap();
    assert_eq!(target.pending(&f.vault).unwrap(), vec![3]);
    f.vault.enable_attachments().unwrap();
    let attachment = faction_omemo::attachment::Descriptor {
        name: Zeroizing::new("private.txt".into()),
        media_type: Zeroizing::new("text/plain".into()),
        size: 1,
        hash: Zeroizing::new([3; 32]),
        key: Zeroizing::new([4; 32]),
        iv: Zeroizing::new([5; 12]),
        locator: Zeroizing::new("https://blob.test/private".into()),
    };
    f.vault
        .queue_attachment([3; 16], "attachment", "body", &attachment, f.time, f.time)
        .unwrap();
    assert_eq!(target.pending(&f.vault).unwrap(), vec![3]);
    f.vault.discard_pending().unwrap();
    assert!(target.pending(&f.vault).unwrap().is_empty());
    f.rotate("bob@away.test", 202);
    assert!(target.pending(&f.vault).unwrap().is_empty());
    let current = foreground::Target::prepare(&f.vault, &f.settings, 5, f.time).unwrap();
    for n in 1..=8 {
        f.vault
            .queue([n + 10; 16], "bounded", "body", f.time, f.time)
            .unwrap();
        assert_eq!(current.pending(&f.vault).unwrap().len(), n as usize);
    }
    // Enumeration is bounded at eight; E admission still refuses a full outbox.
    assert!(foreground::Target::prepare(&f.vault, &f.settings, 5, f.time).is_err());
    assert!(reply::Target::compose(&f.vault, f.time).is_err());
}

#[test]
fn conversation_pending_rejects_ambiguous_associations() {
    child("checkpoint_ui::foreground_tests::actions::conversation_pending_ambiguous_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn conversation_pending_ambiguous_helper() {
    use faction_client::mail_checkpoint::OutboxInspection;
    let f = Fixture::new();
    let target = foreground::Target::prepare(&f.vault, &f.settings, 0, f.time).unwrap();
    assert_eq!(f.vault.pending(f.time).unwrap().len(), 2);
    retain(&f);
    let records = f.vault.history().unwrap();
    let pending = f.vault.inspect_outbox().unwrap();
    assert_eq!(
        target.pending_records(&[&records[2], &records[3]], &pending),
        vec![0, 1]
    );
    let duplicated = [&records[2], &records[2], &records[3]];
    let duplicate_pending = [
        OutboxInspection {
            operation: [1; 16],
            address: "bob@away.test",
            device: 202,
            sent_at: f.time,
        },
        OutboxInspection {
            operation: [1; 16],
            address: "bob@away.test",
            device: 202,
            sent_at: f.time,
        },
    ];
    let mut failures = Vec::new();
    if target.pending_records(&duplicated, &pending) != vec![2] {
        failures.push("duplicate canonical association");
    }
    if !target
        .pending_records(&[&records[2]], &duplicate_pending)
        .is_empty()
    {
        failures.push("duplicate pending association");
    }
    for (address, device) in [("other@away.test", 202), ("bob@away.test", 203)] {
        let mismatch = [OutboxInspection {
            operation: [1; 16],
            address,
            device,
            sent_at: f.time,
        }];
        if !target.pending_records(&[&records[2]], &mismatch).is_empty() {
            failures.push("foreign pending association");
        }
    }
    let wrong_key = foreground::Target {
        address: Zeroizing::new("bob@away.test".into()),
        device: 202,
        identity: [9; 32],
    };
    assert!(
        wrong_key
            .pending_records(&[&records[2]], &pending)
            .is_empty()
    );
    assert!(target.pending_records(&[&records[0]], &pending).is_empty());
    assert!(target.pending_records(&[&records[2]], &[]).is_empty());
    assert!(
        failures.is_empty(),
        "invalid queue association accepted: {failures:?}"
    );
}

#[test]
fn conversation_queued_picker_entry() {
    child("checkpoint_ui::foreground_tests::actions::conversation_queued_picker_entry_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn conversation_queued_picker_entry_helper() {
    let mut f = Fixture::new();
    assert!(foreground::Target::prepare(&f.vault, &f.settings, 0, f.time).is_ok());
    assert_eq!(f.vault.pending(f.time).unwrap().len(), 2);
    retain(&f);
    let before = std::fs::read(f.root.join("vault/mail.fkv")).unwrap();
    let (result, text, list) =
        view_controlled(&mut f, true, false, 0, inbox::Filter::Unread, None, |d| {
            d.wait("[B] Back");
            d.send(b"e");
            d.wait("[B] Return");
            d.send(b"b");
            d.wait("[LOCAL CONVERSATION]");
            d.wait("[B] Back");
            d.send(b"b");
        });
    result.unwrap();
    assert!(text.contains("[Q] Choose queued text"));
    assert!(!text.contains("[S] Send selected text"));
    assert_eq!(list.mode(), inbox::Filter::Unread);
    assert_eq!(list.selected().unwrap().index, 0);
    assert_eq!(
        std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
        before
    );
}

#[test]
fn conversation_queued_picker_navigation_return() {
    child("checkpoint_ui::foreground_tests::actions::conversation_queued_picker_navigation_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn conversation_queued_picker_navigation_helper() {
    for unicode in [false, true] {
        for count in [2, 0, 1, 7] {
            let mut f = Fixture::new();
            if count == 0 {
                f.vault.discard_pending().unwrap();
            }
            if count == 1 {
                f.vault.acknowledge([1; 16]).unwrap();
            }
            for n in 3..=count {
                f.vault
                    .queue(
                        [n; 16],
                        &format!("λ\u{202e} {}", "long".repeat(25)),
                        "hidden",
                        f.time,
                        f.time,
                    )
                    .unwrap();
            }
            assert!(foreground::Target::prepare(&f.vault, &f.settings, 0, f.time).is_ok());
            assert_eq!(f.vault.pending(f.time).unwrap().len(), count as usize);
            retain(&f);
            let before = std::fs::read(f.root.join("vault/mail.fkv")).unwrap();
            let (result, text, list) = view_controlled(
                &mut f,
                true,
                unicode,
                0,
                inbox::Filter::Unread,
                None,
                move |d| {
                    d.wait("[B] Back");
                    d.send(b"e");
                    d.wait("[B] Return");
                    d.send(b"q");
                    d.wait("[LOCAL QUEUED TEXT]");
                    d.wait("[B] Return");
                    if count > 0 {
                        let rendered = std::fs::read_to_string(&d.capture).unwrap();
                        let picker = &rendered[rendered.rfind("[LOCAL QUEUED TEXT]").unwrap()..];
                        assert!(
                            picker.contains("selection 1/"),
                            "picker must start at oldest eligible canonical record: {picker}"
                        );
                    }
                    if count > 1 {
                        d.send(b"j");
                        d.wait("selection 2/");
                        d.wait("[B] Return");
                        if count > 2 {
                            d.send(b"n");
                            d.wait("Page 2/");
                            d.wait("[B] Return");
                            d.send(b"p");
                            d.wait("Page 1/");
                            d.wait("[B] Return");
                        }
                    }
                    d.send(b"b");
                    d.wait("[Q] Choose queued text");
                    d.wait("[B] Return");
                    d.send(b"b");
                    d.wait("[LOCAL CONVERSATION]");
                    d.wait("[B] Back");
                    d.send(b"b");
                },
            );
            result.unwrap();
            assert_eq!(list.mode(), inbox::Filter::Unread);
            assert_eq!(list.selected().unwrap().index, 0);
            assert_eq!(
                std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
                before
            );
            assert!(!text.contains("HOME account password") && !text.contains("Type exchange"));
            assert!(
                text[text.rfind("[LOCAL CONVERSATION]").unwrap()..].contains("Unread / Search")
            );
            if count == 0 {
                assert!(text.contains("No eligible queued ordinary text."));
            }
            if count == 2 {
                assert!(
                    text.contains("> 03 keep selected") && text.contains("> 04 keep other pending")
                );
            }
            for part in text.split("[LOCAL QUEUED TEXT]").skip(1) {
                let picker = part.split("[Q] Choose queued text").next().unwrap();
                for row in picker
                    .lines()
                    .filter(|row| row.starts_with("> ") || row.starts_with("  "))
                {
                    assert!(faction_terminal::text::width(row) <= 39);
                    assert!(!row.contains('\u{202e}'));
                }
            }
        }
    }
}

#[test]
fn conversation_queued_selection_dispatch_return() {
    child("checkpoint_ui::foreground_tests::actions::conversation_queued_dispatch_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn conversation_queued_dispatch_helper() {
    use std::cell::Cell;
    for unicode in [false, true] {
        for failure in [false, true] {
            let mut f = Fixture::new();
            assert_eq!(f.vault.pending(f.time).unwrap().len(), 2);
            assert!(foreground::Target::prepare(&f.vault, &f.settings, 0, f.time).is_ok());
            retain(&f);
            let pending = f.vault.pending(f.time).unwrap();
            let cursor = f.vault.archive_checkpoint(f.time).unwrap();
            let before = std::fs::read(f.root.join("vault/mail.fkv")).unwrap();
            let identity = f.remote.bundle().unwrap().identity;
            let calls = Cell::new(0);
            let cycle = |v: &mut CheckpointVault,
                         password: &str,
                         peer: faction_client::mail_network::ForegroundPeer<'_>,
                         action: ForegroundAction| {
                assert_eq!(password, "fresh synthetic home password");
                assert_eq!(
                    (peer.address, peer.device, peer.identity),
                    ("bob@away.test", 202, identity)
                );
                assert_eq!(action, ForegroundAction::SendSelected([2; 16]));
                assert_eq!(v.pending(now().unwrap()).unwrap(), pending);
                calls.set(calls.get() + 1);
                ForegroundReport {
                    write: if failure {
                        ForegroundWrite::Unconfirmed
                    } else {
                        ForegroundWrite::Completed
                    },
                    accepted: 0,
                    duplicates: 0,
                    error: failure.then_some(faction_client::mail_network::Error::Protocol),
                }
            };
            let (result, text, list) = view_controlled(
                &mut f,
                true,
                unicode,
                0,
                inbox::Filter::Unread,
                Some(&cycle),
                |d| {
                    for connect in [false, true] {
                        d.wait("[B] Back");
                        d.send(b"e");
                        d.wait("[B] Return");
                        d.send(b"q");
                        d.wait("[LOCAL QUEUED TEXT]");
                        d.wait("[B] Return");
                        d.send(b"j");
                        d.wait("selection 2/2");
                        d.wait("[B] Return");
                        d.send(b"oexchange\rqueued password\r");
                        d.wait("Type exchange to connect, or Enter: ");
                        let review = std::fs::read_to_string(&d.capture).unwrap();
                        assert!(
                            review.contains("Retained record: 4")
                                && review.contains(&hex(&[2; 16]))
                                && review.contains("Subject: keep other pending")
                        );
                        assert!(!review.contains("HOME account password"));
                        if connect {
                            d.send(b"exchange\r");
                            d.wait("HOME account password (hidden; never sent to a BBS): ");
                            d.send(b"fresh synthetic home password\r");
                            d.wait("Pending ciphertext is retained.");
                            d.wait("Enter to return: ");
                            d.send(b"\r");
                        } else {
                            std::thread::sleep(Duration::from_millis(75));
                            assert!(
                                !std::fs::read_to_string(&d.capture)
                                    .unwrap()
                                    .contains("Connecting directly to your home.")
                            );
                            d.send(b"\r");
                        }
                        d.wait("[LOCAL CONVERSATION]");
                    }
                    d.wait("[B] Back");
                    d.send(b"b");
                },
            );
            result.unwrap();
            assert_eq!(calls.get(), 1);
            assert_eq!(f.vault.pending(f.time).unwrap(), pending);
            assert_eq!(f.vault.archive_checkpoint(f.time).unwrap(), cursor);
            assert_eq!(
                std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
                before
            );
            assert_eq!(list.mode(), inbox::Filter::Unread);
            assert_eq!(list.selected().unwrap().index, 0);
            assert_eq!(
                list.eligible()
                    .iter()
                    .map(|entry| entry.index)
                    .collect::<Vec<_>>(),
                vec![1, 0]
            );
            assert!(
                text[text.rfind("[LOCAL CONVERSATION]").unwrap()..].contains("Unread / Search")
            );
            assert_eq!(text.matches("HOME account password").count(), 1);
            assert!(
                !text.contains("queued password")
                    && !text.contains("fresh synthetic home password")
            );
        }
    }
}

#[test]
fn conversation_inherited_authority_and_eighth_queue_limits() {
    child("checkpoint_ui::foreground_tests::actions::conversation_inherited_limits_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn conversation_inherited_limits_helper() {
    for unicode in [false, true] {
        let mut f = Fixture::new();
        for n in 3..=7 {
            f.vault
                .queue([n; 16], "hidden", "body", f.time, f.time)
                .unwrap();
        }
        assert_eq!(f.vault.pending(f.time).unwrap().len(), 7);
        assert!(conversation::compose_target(&f.vault, &f.settings, 0, f.time).is_ok());
        retain(&f);
        let (result, text, list) =
            view_controlled(&mut f, true, unicode, 0, inbox::Filter::Unread, None, |d| {
                d.wait("[B] Back");
                d.send(b"w");
                d.wait("Subject (128 UTF-8 bytes): ");
                d.send(b"eighth hidden\r");
                d.wait("write> ");
                d.send(b"eighth fresh body\r");
                d.wait("write> ");
                d.send(b".done\r");
                d.wait("Type queue to persist encrypted ciphertext: ");
                d.send(b"queue\r");
                d.wait("Queued durably; no delivery or read receipt.");
                d.wait("[LOCAL CONVERSATION]");
                d.wait("[B] Back");
                d.send(b"b");
            });
        result.unwrap();
        assert_eq!(f.vault.inspect_outbox().unwrap().len(), 8);
        assert_eq!(f.vault.history().unwrap().len(), 10);
        assert_eq!(
            f.vault.history().unwrap()[9].body.as_str(),
            "eighth fresh body"
        );
        assert!(conversation::compose_target(&f.vault, &f.settings, 0, f.time).is_err());
        assert!(foreground::Target::prepare(&f.vault, &f.settings, 0, f.time).is_err());
        let tail = &text[text.rfind("[LOCAL CONVERSATION]").unwrap()..];
        assert!(!tail.contains("[W]") && !tail.contains("[E]"));
        assert_eq!(list.mode(), inbox::Filter::Unread);
        assert_eq!(list.selected().unwrap().index, 0);
    }
    let mut f = Fixture::new();
    let target = conversation::compose_target(&f.vault, &f.settings, 0, f.time).unwrap();
    assert!(conversation::compose_target(&f.vault, &f.settings, 0, f.time - 1).is_err());
    assert!(
        conversation::compose_target(
            &f.vault,
            &f.settings,
            0,
            f.vault.inspection().unwrap().expires + 1
        )
        .is_err()
    );
    f.vault.backup(&f.root.join("history-only")).unwrap();
    let archived = f.open("history-only");
    assert!(conversation::compose_target(&archived, &f.settings, 0, f.time).is_err());
    drop(archived);
    f.vault.revoke().unwrap();
    assert!(target.recheck(&f.vault, f.time).is_err());
    assert!(conversation::compose_target(&f.vault, &f.settings, 0, f.time).is_err());
    drop(f);
    let mut f = Fixture::new();
    f.vault.discard_pending().unwrap();
    f.vault.fresh_device(102, f.time).unwrap();
    let peer = f.remote.bundle().unwrap();
    f.vault
        .enroll(
            "bob@away.test",
            202,
            &format!("ed25519:{}", hex(&peer.identity)),
            f.time,
        )
        .unwrap();
    assert!(!f.vault.inspection().unwrap().started);
    assert!(conversation::compose_target(&f.vault, &f.settings, 0, f.time).is_err());
    drop(f);
    let mut f = Fixture::new();
    f.vault.discard_pending().unwrap();
    while f.vault.history().unwrap().len() < MAX_HISTORY {
        let operation = [f.vault.history().unwrap().len() as u8 + 10; 16];
        f.vault
            .queue(operation, "retained limit", "body", f.time, f.time)
            .unwrap();
        f.vault.acknowledge(operation).unwrap();
    }
    assert!(conversation::compose_target(&f.vault, &f.settings, 0, f.time).is_err());
    assert!(foreground::Target::prepare(&f.vault, &f.settings, 0, f.time).is_err());
    drop(f);
    let mut f = Fixture::new();
    let canonical = f.root.join("vault/mail.fkv");
    let saved = f.root.join("vault/saved");
    std::fs::rename(&canonical, &saved).unwrap();
    assert!(
        f.vault
            .queue([99; 16], "fault", "body", f.time, f.time)
            .is_err()
    );
    std::fs::rename(&saved, &canonical).unwrap();
    assert!(f.vault.inspection().is_err());
    assert!(conversation::compose_target(&f.vault, &f.settings, 0, f.time).is_err());
}

#[test]
fn conversation_compose_target_contract() {
    child("checkpoint_ui::foreground_tests::actions::conversation_compose_target_helper");
}
#[test]
#[ignore = "owned subprocess helper"]
fn conversation_compose_target_helper() {
    let mut invalidly_accepted = Vec::new();
    for (label, address, device) in [
        ("account", "other@away.test", 202),
        ("device", "bob@away.test", 203),
        ("replacement", "bob@away.test", 202),
    ] {
        let mut f = Fixture::new();
        f.rotate(address, device);
        assert!(reply::Target::compose(&f.vault, f.time).is_ok());
        assert!(conversation::compose_target(&f.vault, &f.settings, 4, f.time).is_ok());
        retain(&f);
        if conversation::compose_target(&f.vault, &f.settings, 0, f.time).is_ok() {
            invalidly_accepted.push(label);
        }
    }
    let mut foreign = Fixture::new();
    foreign.settings.account = "other@home.test".into();
    retain(&foreign);
    if conversation::compose_target(&foreign.vault, &foreign.settings, 0, foreign.time).is_ok() {
        invalidly_accepted.push("home");
    }
    assert!(
        invalidly_accepted.is_empty(),
        "conversation target accepted {invalidly_accepted:?}"
    );
    let mut f = Fixture::new();
    assert_eq!(f.vault.pending(f.time).unwrap().len(), 2);
    assert!(f.vault.inspection().unwrap().started);
    retain(&f);
    let target = conversation::compose_target(&f.vault, &f.settings, 0, f.time).unwrap();
    assert_eq!(target.address.as_str(), "bob@away.test");
    assert_eq!(target.device, 202);
    assert_eq!(target.identity, f.remote.bundle().unwrap().identity);
    // W has compose's limits and needs no archive/cursor authority.
    for name in ["cold", "empty-cursor"] {
        let v = f.open(name);
        assert!(v.inspection().unwrap().started);
        assert!(conversation::compose_target(&v, &f.settings, 0, f.time).is_ok());
    }
    let canonical = std::fs::read(f.root.join("vault/mail.fkv")).unwrap();
    assert!(conversation::compose_target(&f.vault, &f.settings, 99, f.time).is_err());
    f.settings.account = "other@home.test".into();
    assert!(conversation::compose_target(&f.vault, &f.settings, 0, f.time).is_err());
    assert_eq!(
        std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
        canonical
    );
    drop(f);
    for (address, device) in [
        ("other@away.test", 202),
        ("bob@away.test", 203),
        ("bob@away.test", 202),
    ] {
        let mut f = Fixture::new();
        f.rotate(address, device);
        assert!(reply::Target::compose(&f.vault, f.time).is_ok());
        assert!(conversation::compose_target(&f.vault, &f.settings, 4, f.time).is_ok());
        assert!(
            conversation::compose_target(&f.vault, &f.settings, 0, f.time).is_err(),
            "old conversation acquired replacement authority"
        );
        retain(&f);
    }
}
