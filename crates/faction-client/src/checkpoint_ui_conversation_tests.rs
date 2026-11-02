use super::{
    conversation::{Conversation, Preview, render},
    inbox::{Entry, Filter, Inbox, Status, within_group},
};
fn entries(n: usize) -> Vec<Entry> {
    (0..n)
        .map(|index| Entry {
            index,
            status: Status::Unread,
        })
        .collect()
}
fn indices(model: &Conversation) -> Vec<usize> {
    model.visible().iter().map(|e| e.index).collect()
}
#[test]
fn conversation_empty_one_and_full_retention_clamp_without_wrapping() {
    let mut empty = Conversation::new(&Inbox::new(vec![]));
    empty.up();
    empty.down();
    empty.next();
    empty.previous();
    assert_eq!(empty.selected(), None);
    assert_eq!(empty.page(), (0, 0));
    assert!(empty.visible().is_empty());
    let mut one = Conversation::new(&Inbox::new(entries(1)));
    one.up();
    one.down();
    one.next();
    one.previous();
    assert_eq!(one.selected().unwrap().index, 0);
    let mut full = Conversation::new(&Inbox::new(entries(64)));
    assert_eq!(full.selected().unwrap().index, 63);
    assert_eq!(full.page(), (32, 32));
    assert_eq!(indices(&full), [62, 63]);
    for _ in 0..80 {
        full.up();
        full.previous();
    }
    assert_eq!(full.selected().unwrap().index, 0);
    assert_eq!(full.page(), (1, 32));
    full.down();
    assert_eq!(full.selected().unwrap().index, 1);
    full.next();
    assert_eq!(indices(&full), [2, 3]);
    full.previous();
    assert_eq!(full.selected().unwrap().index, 0);
    for _ in 0..80 {
        full.next();
        full.down();
    }
    assert_eq!(full.selected().unwrap().index, 63);
    assert_eq!(full.position(), 63);
    assert_eq!(full.len(), 64);
}
#[test]
fn conversation_intersects_exact_group_filter_and_search_preserving_original_indices() {
    let peers = [("bob", 1), ("bob", 2), ("carol", 1), ("bob", 1), ("bob", 1)];
    let mut records = entries(5);
    records[4].status = Status::Pending;
    let mut inbox = Inbox::new(within_group(records.clone(), &peers, 0));
    inbox.search(Some(vec![1, 2, 3, 4]));
    let mut model = Conversation::new(&inbox);
    assert_eq!(indices(&model), [3, 4]);
    assert_eq!(model.selected().unwrap().index, 4);
    assert!(inbox.filter(Filter::Unread));
    model.refresh(&inbox);
    assert_eq!(indices(&model), [3]);
    records[3].status = Status::Read;
    inbox.refresh_matching(
        within_group(records.clone(), &peers, 0),
        Some(vec![1, 2, 3, 4]),
    );
    model.refresh(&inbox);
    assert_eq!(model.selected(), None);
    assert!(inbox.filter(Filter::All));
    model.refresh(&inbox);
    assert_eq!(indices(&model), [3, 4]);
    inbox.search(Some(vec![]));
    model.refresh(&inbox);
    assert_eq!(model.selected(), None);
    assert!(model.visible().is_empty());
    records[0].status = Status::Unknown;
    inbox.refresh_matching(within_group(records, &peers, 0), Some(vec![]));
    assert!(!inbox.filter(Filter::Unread));
}
#[test]
fn conversation_refresh_keeps_selection_then_clamps_with_no_timestamp_authority() {
    let mut inbox = Inbox::new(entries(5));
    let mut model = Conversation::new(&inbox);
    model.up();
    model.up();
    assert_eq!(model.selected().unwrap().index, 2);
    inbox.refresh(entries(6));
    model.refresh(&inbox);
    assert_eq!(model.selected().unwrap().index, 2);
    inbox.search(Some(vec![0, 1, 3, 5]));
    model.refresh(&inbox);
    assert_eq!(model.selected().unwrap().index, 3);
    model.previous();
    assert_eq!(indices(&model), [0, 1]);
}
fn preview(body: &str, unicode: bool) -> zeroize::Zeroizing<String> {
    render(
        Preview {
            entry: Entry {
                index: 63,
                status: Status::Unread,
            },
            received: true,
            sent_at: u64::MAX,
            subject: "synthetic subject",
            body,
            attachment: true,
            selected: true,
        },
        unicode,
    )
}
#[test]
fn conversation_renderer_discloses_original_index_direction_status_utc_and_attachment() {
    let output = preview("body", false);
    assert!(output.starts_with("> 64 RECEIVED UNREAD @\r\n"));
    assert!(output.contains("UTC 18446744073709551615\r\n"));
    assert!(output.contains("Subject: synthetic subject\r\n"));
    assert!(output.contains("[O] Full record\r\n"));
    let sent = render(
        Preview {
            entry: Entry {
                index: 0,
                status: Status::Pending,
            },
            received: false,
            sent_at: 0,
            subject: "sent",
            body: "outbound",
            attachment: false,
            selected: false,
        },
        false,
    );
    assert!(sent.starts_with("  01 SENT PENDING\r\n"));
    assert!(!sent.contains('@'));
}
#[test]
fn conversation_renderer_three_rows_exact_boundary_and_explicit_continuation() {
    for unicode in [false, true] {
        for len in [0, 1, 37, 38, 110, 111, 112, 4096] {
            let output = preview(&"x".repeat(len), unicode);
            let rows = output
                .lines()
                .filter(|s| s.starts_with("| "))
                .collect::<Vec<_>>();
            assert_eq!(rows.len(), 3);
            assert_eq!(
                rows.iter().map(|s| s[2..].len()).sum::<usize>(),
                len.min(111)
            );
            assert_eq!(
                output.contains("Body continues; O opens full record."),
                len > 111
            );
            assert_eq!(output.lines().count(), 7);
            assert!(output.len() <= 1024);
        }
    }
}
#[test]
fn conversation_renderer_inert_scalars_and_cell_bounds_in_both_modes() {
    let hostile = "λ界e\u{301}\n\r\x1b]52;c;x\x07\u{85}\u{202e}\u{2066}\u{200d}".repeat(30);
    for unicode in [false, true] {
        let output = render(
            Preview {
                entry: Entry {
                    index: 0,
                    status: Status::Unknown,
                },
                received: true,
                sent_at: 8,
                subject: &hostile,
                body: &hostile,
                attachment: false,
                selected: false,
            },
            unicode,
        );
        for row in output.lines() {
            assert!(
                faction_terminal::text::width(row) <= 39,
                "row too wide: {row:?}"
            );
            assert!(row.chars().all(|c| !c.is_control()));
            assert!(!row.contains(['\u{202e}', '\u{2066}', '\u{200d}', '\u{301}']));
            if !unicode {
                assert!(row.is_ascii());
            }
        }
        assert!(output.contains("Body continues; O opens full record."));
        assert!(output.len() <= 1024);
    }
}

#[test]
fn conversation_actual_entry_reader_refresh_and_lock_boundaries() {
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "checkpoint_ui::conversation_tests::conversation_actual_interface_helper",
            "--nocapture",
        ])
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "conversation interface: {} {}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
}
#[test]
#[ignore = "owned subprocess helper for actual conversation controls"]
fn conversation_actual_interface_helper() {
    use super::{IDLE, LocalTerminal, Screen, now};
    use faction_client::{
        mail_checkpoint::{CheckpointVault, Wire},
        mail_data::{Data, hex},
        mail_vault::Store,
    };
    use faction_omemo::{Device, sce};
    use std::{
        io::{self, Write},
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };
    struct Owned(std::path::PathBuf);
    impl Drop for Owned {
        fn drop(&mut self) {
            assert!(self.0.join("OWNED").is_file());
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    let mut random = [0; 16];
    getrandom::fill(&mut random).unwrap();
    let root = Owned(
        std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("faction339-ui-OWNED-{}", hex(&random))),
    );
    std::fs::create_dir(&root.0).unwrap();
    std::fs::set_permissions(&root.0, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(root.0.join("OWNED"), b"synthetic issue339 controls\n").unwrap();
    let path = root.0.join("vault");
    let timestamp = now().unwrap();
    let (mut vault, _) = CheckpointVault::create(
        Store::at(&path, true).unwrap(),
        Data::new("alice@home.test".into()).unwrap(),
        b"synthetic conversation password",
        101,
        timestamp,
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
            timestamp,
        )
        .unwrap();
    peer.trust(&format!("ed25519:{}", hex(&local.identity)))
        .unwrap();
    peer.start(&local).unwrap();
    // Hostile sender time is shown but never used to rearrange canonical records.
    let plain = sce::encode(
        "bob@away.test",
        "alice@home.test",
        timestamp + 3600,
        "received selected",
        "private body λ界\n\u{202e}",
    )
    .unwrap();
    let incoming = peer.seal(&plain).unwrap();
    vault
        .receive(
            &Wire {
                namespace: "urn:xmpp:omemo:2".into(),
                from: "bob@away.test".into(),
                to: "alice@home.test".into(),
                device: 202,
                to_device: 101,
                identity: remote.identity,
                sent_at: timestamp + 3600,
                key: incoming.key,
                prekey: incoming.prekey,
                payload: incoming.payload,
            },
            timestamp,
        )
        .unwrap();
    vault
        .queue(
            [1; 16],
            "sent selected",
            "sent bounded body",
            timestamp,
            timestamp,
        )
        .unwrap();
    let before = std::fs::read(path.join("mail.fkv")).unwrap();
    for unicode in [false, true] {
        for online in [false, true] {
            let (master, slave) = faction_client_pty::open(24, 40).unwrap();
            let mut input = std::fs::File::from(master);
            let saved = rustix::termios::tcgetattr(&slave).unwrap();
            let mut raw = saved.clone();
            raw.make_raw();
            rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw)
                .unwrap();
            let capture = root.0.join("output");
            let mut term = LocalTerminal {
                input: slave.into(),
                output: std::fs::File::create(&capture).unwrap(),
                saved,
            };
            // Global T is inert; C/O/T enters the exact group, then O opens sent index 1.
            input.write_all(b"tcoto\rbbbb").unwrap();
            Screen {
                term: &mut term,
                activity: Instant::now(),
                unicode,
            }
            .inbox_mode(&mut vault, online)
            .unwrap();
            drop(term);
            let output = std::fs::read_to_string(&capture).unwrap();
            assert_eq!(output.matches("[LOCAL CONVERSATION]").count(), 2);
            assert_eq!(output.matches("[LOCAL INBOX RECORD]").count(), 1);
            assert!(output.contains("PENDING / local retained record 2"));
            assert!(!output.contains("Reply locally"));
            assert!(!output.contains("Type read"));
            assert!(
                output.contains("UNKNOWN"),
                "disabled tracking remains unknown"
            );
            for frame in output
                .split("---------------------------------------\r\n")
                .filter(|frame| frame.starts_with("[LOCAL CONVERSATION]"))
            {
                assert!(frame.lines().count() <= 23);
                assert!(
                    frame
                        .lines()
                        .all(|row| faction_terminal::text::width(row) <= 39)
                );
                assert!(frame.contains(&format!("UTC {}\r\n", timestamp + 3600)));
                assert!(frame.find("01 RECEIVED").unwrap() < frame.find("02 SENT").unwrap());
            }
            assert_eq!(std::fs::read(path.join("mail.fkv")).unwrap(), before);
        }
    }
    // Populate a second preview page without changing the producer. Movement
    // controls must open the canonical record selected after page transitions.
    for operation in [2, 3] {
        vault
            .queue(
                [operation; 16],
                "movement selected",
                "movement body",
                timestamp,
                timestamp,
            )
            .unwrap();
    }
    let movement_before = std::fs::read(path.join("mail.fkv")).unwrap();
    for unicode in [false, true] {
        for online in [false, true] {
            let (master, slave) = faction_client_pty::open(24, 40).unwrap();
            let mut input = std::fs::File::from(master);
            let saved = rustix::termios::tcgetattr(&slave).unwrap();
            let mut raw = saved.clone();
            raw.make_raw();
            rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw)
                .unwrap();
            let capture = root.0.join("movement-output");
            let mut term = LocalTerminal {
                input: slave.into(),
                output: std::fs::File::create(&capture).unwrap(),
                saved,
            };
            input.write_all(b"cotk\x1b[Aj\x1b[Bpno\rbbbb").unwrap();
            Screen {
                term: &mut term,
                activity: Instant::now(),
                unicode,
            }
            .inbox_mode(&mut vault, online)
            .unwrap();
            drop(term);
            let output = std::fs::read_to_string(&capture).unwrap();
            let frames = output
                .split("---------------------------------------\r\n")
                .filter(|frame| frame.starts_with("[LOCAL CONVERSATION]"))
                .collect::<Vec<_>>();
            assert_eq!(frames.len(), 8);
            for (frame, selected) in frames.iter().zip([4, 3, 2, 3, 4, 1, 3, 3]) {
                assert!(frame.contains(&format!("selection {selected}/4\r\n")));
                assert!(frame.contains(&format!("> {selected:02} ")));
                assert!(frame.lines().count() <= 23);
                assert!(
                    frame
                        .lines()
                        .all(|row| faction_terminal::text::width(row) <= 39)
                );
            }
            assert!(output.contains("PENDING / local retained record 3\r\n"));
            let restored_group = output.rsplit("[LOCAL CORRESPONDENT INBOX]").next().unwrap();
            assert!(
                restored_group.contains("> 03 PENDING"),
                "Back must restore the selected canonical record to its parent"
            );
            assert_eq!(
                std::fs::read(path.join("mail.fkv")).unwrap(),
                movement_before
            );
        }
    }
    vault.enable_read_tracking().unwrap();
    #[cfg(feature = "network-mail")]
    for unicode in [false, true] {
        for queue in [false, true] {
            let (master, slave) = faction_client_pty::open(24, 40).unwrap();
            let mut input = std::fs::File::from(master);
            let saved = rustix::termios::tcgetattr(&slave).unwrap();
            let mut raw = saved.clone();
            raw.make_raw();
            rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw)
                .unwrap();
            let capture = root.0.join("online-reply-output");
            let mut term = LocalTerminal {
                input: slave.into(),
                output: std::fs::File::create(&capture).unwrap(),
                saved,
            };
            let query = super::inbox::Search::parse(b"received selected").unwrap();
            let mut list = Inbox::new(Screen::inbox_group_entries(&vault, Some(0)).unwrap());
            list.search(Screen::inbox_matches(&vault, &query).unwrap());
            list.select_index(0);
            let canonical = std::fs::read(path.join("mail.fkv")).unwrap();
            let history_len = vault.history().unwrap().len();
            let matching_len = list.len();
            let pending = vault
                .inspect_outbox()
                .unwrap()
                .iter()
                .map(|entry| entry.operation)
                .collect::<Vec<_>>();
            let received = vault.history().unwrap()[0].received_id().unwrap();
            let driver_capture = capture.clone();
            std::thread::scope(|scope| {
                let driver = scope.spawn(move || {
                    let wait = |needle: &str| {
                        let deadline = Instant::now() + Duration::from_secs(15);
                        loop {
                            let output = std::fs::read_to_string(&driver_capture).unwrap();
                            if output.contains(needle) {
                                break;
                            }
                            assert!(
                                Instant::now() < deadline,
                                "online reply route did not reach {needle}"
                            );
                            std::thread::sleep(Duration::from_millis(10));
                        }
                    };
                    input.write_all(b"o").unwrap();
                    wait("Type read to acknowledge locally");
                    input.write_all(b"\rr").unwrap();
                    wait("Subject (Enter keeps suggestion;");
                    input.write_all(b"received selected reply\r").unwrap();
                    wait("Body (1024 UTF-8 bytes, one line): ");
                    input.write_all(b"private queued reply\r").unwrap();
                    wait("Type queue to persist encrypted ciphertext: ");
                    input
                        .write_all(if queue { b"queue\r" } else { b"\r" })
                        .unwrap();
                    let deadline = Instant::now() + Duration::from_secs(15);
                    loop {
                        let output = std::fs::read_to_string(&driver_capture).unwrap();
                        let frame = output.rsplit("[LOCAL CONVERSATION]").next().unwrap();
                        if output.matches("[LOCAL CONVERSATION]").count() == 2
                            && frame.contains("[B] Back")
                        {
                            break;
                        }
                        assert!(
                            Instant::now() < deadline,
                            "reply did not refresh conversation"
                        );
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    input.write_all(b"b").unwrap();
                    input
                });
                Screen {
                    term: &mut term,
                    activity: Instant::now(),
                    unicode,
                }
                .conversation(&mut vault, true, 0, &mut list, &query)
                .unwrap();
                let _input = driver.join().unwrap();
            });
            drop(term);
            let output = std::fs::read_to_string(&capture).unwrap();
            assert!(output.contains("UNREAD / local retained record 1"));
            assert!(output.contains("[R] Reply locally"));
            assert!(output.contains("CURRENT recipient: bob@away.test"));
            assert!(output.contains(&format!(
                "CURRENT device: 202 / ed25519:{}",
                hex(&remote.identity)
            )));
            assert_eq!(list.mode(), Filter::All);
            assert_eq!(list.selected().unwrap().index, 0);
            assert_eq!(list.len(), matching_len + usize::from(queue));
            assert_eq!(vault.received_read(0, received).unwrap(), Some(false));
            let after_pending = vault
                .inspect_outbox()
                .unwrap()
                .iter()
                .map(|entry| entry.operation)
                .collect::<Vec<_>>();
            assert_eq!(&after_pending[..pending.len()], pending.as_slice());
            assert_eq!(after_pending.len(), pending.len() + usize::from(queue));
            assert_eq!(
                vault.history().unwrap().len(),
                history_len + usize::from(queue)
            );
            if queue {
                assert!(output.contains("Queued durably; no delivery or read receipt."));
                let last = vault.history().unwrap().last().unwrap();
                assert_eq!(last.address, "bob@away.test");
                assert_eq!(last.device, 202);
                assert_eq!(last.subject.as_str(), "received selected reply");
                assert_eq!(last.body.as_str(), "private queued reply");
            } else {
                assert!(output.contains("Reply cancelled; no ciphertext queued."));
                assert_eq!(std::fs::read(path.join("mail.fkv")).unwrap(), canonical);
            }
        }
    }
    // Enter a filtered conversation and acknowledge only through its full reader.
    for online in [false, true] {
        let (master, slave) = faction_client_pty::open(24, 40).unwrap();
        let mut input = std::fs::File::from(master);
        let saved = rustix::termios::tcgetattr(&slave).unwrap();
        let mut raw = saved.clone();
        raw.make_raw();
        rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw).unwrap();
        let capture = root.0.join("ack-output");
        let mut term = LocalTerminal {
            input: slave.into(),
            output: std::fs::File::create(&capture).unwrap(),
            saved,
        };
        let query = super::inbox::Search::parse(b"received selected").unwrap();
        let mut list = Inbox::new(Screen::inbox_group_entries(&vault, Some(0)).unwrap());
        assert!(list.filter(Filter::Unread));
        list.search(Screen::inbox_matches(&vault, &query).unwrap());
        let before_summary = vault.read_summary().unwrap();
        let before_pending = vault.inspect_outbox().unwrap().len();
        if online {
            // It is already read from the offline case, so this empty view cannot
            // escape its current Unread/query scope or offer online Reply.
            input.write_all(b"onpjkb").unwrap();
            Screen {
                term: &mut term,
                activity: Instant::now(),
                unicode: true,
            }
            .conversation(&mut vault, online, 0, &mut list, &query)
            .unwrap();
            drop(term);
            let output = std::fs::read_to_string(&capture).unwrap();
            assert!(output.contains("No eligible retained records."));
            assert!(!output.contains("Reply locally"));
        } else {
            let driver_capture = capture.clone();
            std::thread::scope(|scope| {
                let driver = scope.spawn(move || {
                    input.write_all(b"o").unwrap();
                    let deadline = Instant::now() + Duration::from_secs(15);
                    loop {
                        let output = std::fs::read_to_string(&driver_capture).unwrap();
                        if output.contains("Type read to acknowledge locally") {
                            break;
                        }
                        assert!(
                            Instant::now() < deadline,
                            "full reader did not reach explicit acknowledgement"
                        );
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    input.write_all(b"read\r\rb").unwrap();
                    // Keep the master alive while Screen consumes queued input.
                    input
                });
                Screen {
                    term: &mut term,
                    activity: Instant::now(),
                    unicode: false,
                }
                .conversation(&mut vault, false, 0, &mut list, &query)
                .unwrap();
                let _input = driver.join().unwrap();
            });
            drop(term);
            let output = std::fs::read_to_string(&capture).unwrap();
            assert!(output.contains("UNREAD / local retained record 1"));
            assert!(output.contains("Acknowledged on this device only"));
            assert!(output.contains("No eligible retained records."));
            assert_eq!(before_summary.unread, Some(1));
        }
        assert_eq!(vault.read_summary().unwrap().unread, Some(0));
        assert_eq!(vault.inspect_outbox().unwrap().len(), before_pending);
        assert!(list.selected().is_none());
    }
    let canonical = std::fs::read(path.join("mail.fkv")).unwrap();
    // Cancellation, timeout, output error and EOF propagate without any write.
    for (byte, expired, fail_output) in [
        (Some(3), false, false),
        (Some(4), false, false),
        (None, true, false),
        (None, false, true),
        (None, false, false),
    ] {
        let (master, slave) = faction_client_pty::open(24, 40).unwrap();
        let observer = slave.try_clone().unwrap();
        let saved = rustix::termios::tcgetattr(&slave).unwrap();
        let mut raw = saved.clone();
        raw.make_raw();
        rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw).unwrap();
        let mut input = std::fs::File::from(master);
        if let Some(b) = byte {
            input.write_all(&[b]).unwrap();
        }
        let mut term = LocalTerminal {
            input: slave.into(),
            output: if fail_output {
                std::fs::File::open("/dev/null").unwrap()
            } else {
                std::fs::OpenOptions::new()
                    .write(true)
                    .open("/dev/null")
                    .unwrap()
            },
            saved: saved.clone(),
        };
        let old_input = if byte.is_none() && !expired && !fail_output {
            std::fs::write(root.0.join("eof"), []).unwrap();
            Some(std::mem::replace(
                &mut term.input,
                std::fs::File::open(root.0.join("eof")).unwrap(),
            ))
        } else {
            None
        };
        let mut list = Inbox::new(Screen::inbox_group_entries(&vault, Some(0)).unwrap());
        let error = Screen {
            term: &mut term,
            activity: if expired {
                Instant::now() - IDLE
            } else {
                Instant::now()
            },
            unicode: false,
        }
        .conversation(&mut vault, false, 0, &mut list, &Default::default())
        .unwrap_err();
        if !fail_output {
            assert_eq!(
                error.kind(),
                if byte.is_some() {
                    io::ErrorKind::Interrupted
                } else if expired {
                    io::ErrorKind::TimedOut
                } else {
                    io::ErrorKind::UnexpectedEof
                }
            );
        }
        if let Some(old) = old_input {
            term.input = old;
        }
        drop(term);
        let restored = rustix::termios::tcgetattr(&observer).unwrap();
        #[cfg(any(target_os = "macos", target_os = "netbsd"))]
        let pendin = rustix::termios::LocalModes::PENDIN;
        #[cfg(not(any(target_os = "macos", target_os = "netbsd")))]
        let pendin = rustix::termios::LocalModes::empty();
        assert_eq!(restored.local_modes & !pendin, saved.local_modes & !pendin);
        assert_eq!(restored.input_modes, saved.input_modes);
        assert_eq!(restored.output_modes, saved.output_modes);
        assert_eq!(restored.control_modes, saved.control_modes);
        assert_eq!(std::fs::read(path.join("mail.fkv")).unwrap(), canonical);
    }
    drop(vault);
    let owned = root.0.clone();
    drop(root);
    assert!(!owned.exists());
}
