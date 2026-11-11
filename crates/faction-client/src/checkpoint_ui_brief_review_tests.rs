//! Full selected-text review through the production Screen and real vault.
use super::*;

fn wait_either(d: &mut Driver, first: &str, second: &str) -> String {
    let limit = Instant::now() + Duration::from_secs(15);
    loop {
        let text = std::fs::read_to_string(&d.capture).unwrap();
        if text[d.offset..].contains(first) || text[d.offset..].contains(second) {
            return text;
        }
        assert!(
            Instant::now() < limit,
            "missing {first} or {second}: {text}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn brief_selected_body_full_review_and_cancel() {
    child("checkpoint_ui::foreground_tests::brief_review_tests::brief_selected_body_helper");
}

#[test]
#[ignore = "owned subprocess helper"]
fn brief_selected_body_helper() {
    use std::cell::Cell;
    for unicode in [false, true] {
        for queued in [false, true] {
            for mode in 0..4 {
                let cancel = mode != 0;
                let mut f = Fixture::new();
                let selected_body = (0..35)
                    .map(|n| format!("selected line {n:02} λ\u{202e}\n"))
                    .collect::<String>();
                f.vault
                    .queue([3; 16], "keep twin", "other twin body", f.time, f.time)
                    .unwrap();
                f.vault
                    .queue([4; 16], "keep twin", &selected_body, f.time, f.time)
                    .unwrap();
                let before = std::fs::read(f.root.join("vault/mail.fkv")).unwrap();
                let pending = f.vault.pending(f.time).unwrap();
                let cursor = f.vault.archive_checkpoint(f.time).unwrap();
                let calls = Cell::new(0);
                let cycle = |_: &mut CheckpointVault,
                             password: &str,
                             _: faction_client::mail_network::ForegroundPeer<'_>,
                             action| {
                    calls.set(calls.get() + 1);
                    assert_eq!(action, ForegroundAction::SendSelected([4; 16]));
                    assert_eq!(password, "fresh home secret");
                    ForegroundReport {
                        write: ForegroundWrite::Completed,
                        accepted: 0,
                        duplicates: 0,
                        error: None,
                    }
                };
                let (result, text, list) = view_controlled(
                    &mut f,
                    true,
                    unicode,
                    if queued { 0 } else { 5 },
                    inbox::Filter::All,
                    Some(&cycle),
                    |d| {
                        d.wait("[B] Back");
                        d.send(b"e");
                        d.wait("[B] Return");
                        if queued {
                            d.send(b"q");
                            d.wait("[LOCAL QUEUED TEXT]");
                            for position in 2..=4 {
                                d.wait("[B] Return");
                                d.send(b"j");
                                d.wait(&format!("selection {position}/4"));
                            }
                            d.wait("[B] Return");
                            d.send(b"o");
                        } else {
                            d.send(b"s");
                        }
                        let review = wait_either(d, "[Enter] Next page", "Type exchange");
                        let review = &review[review.rfind("Send selected text only.").unwrap()..];
                        assert!(
                            review.contains("selected line 00"),
                            "exact selected body absent: {review}"
                        );
                        assert!(!review.contains("other twin body"));
                        assert!(
                            !review.contains("Type exchange"),
                            "confirmation before complete body"
                        );
                        d.wait("[Enter] Next page  [B] Return");
                        if cancel {
                            d.send(match mode {
                                1 => b"b",
                                2 => b"\x03",
                                _ => b"\x04",
                            });
                            if mode > 1 {
                                return;
                            }
                        } else {
                            for page in 2..=4 {
                                d.send(b"\r");
                                d.wait(&format!("[LOCAL BODY] Page {page}/4"));
                                if page < 4 {
                                    d.wait("[Enter] Next page  [B] Return");
                                }
                            }
                            d.wait("Type exchange to connect, or Enter: ");
                            let review = std::fs::read_to_string(&d.capture).unwrap();
                            assert!(review.contains("selected line 34"));
                            assert!(!review.contains('\u{202e}'));
                            assert!(!review.contains("HOME account password"));
                            d.send(b"exchange\r");
                            d.wait("HOME account password (hidden; never sent to a BBS): ");
                            d.send(b"fresh home secret\r");
                            d.wait("Enter to return: ");
                            d.send(b"\r");
                        }
                        d.wait("[LOCAL CONVERSATION]");
                        d.wait("[B] Back");
                        d.send(b"b");
                    },
                );
                if mode > 1 {
                    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Interrupted);
                } else {
                    result.unwrap();
                }
                assert_eq!(calls.get(), usize::from(!cancel));
                assert_eq!(f.vault.pending(f.time).unwrap(), pending);
                assert_eq!(f.vault.archive_checkpoint(f.time).unwrap(), cursor);
                assert_eq!(
                    std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
                    before
                );
                assert_eq!(list.mode(), inbox::Filter::All);
                assert_eq!(list.selected().unwrap().index, if queued { 0 } else { 5 });
                assert!(!text.contains("fresh home secret"));
                if cancel {
                    assert!(
                        !text.contains("Type exchange") && !text.contains("HOME account password")
                    );
                }
            }
        }
    }
}

#[test]
fn brief_review_output_and_eof_fail_closed() {
    child("checkpoint_ui::foreground_tests::brief_review_tests::brief_review_io_helper");
}

#[test]
#[ignore = "owned subprocess helper"]
fn brief_review_io_helper() {
    use std::cell::Cell;
    for unicode in [false, true] {
        let mut f = Fixture::new();
        let before = std::fs::read(f.root.join("vault/mail.fkv")).unwrap();
        let calls = Cell::new(0);
        let cycle = |_: &mut CheckpointVault,
                     _: &str,
                     _: faction_client::mail_network::ForegroundPeer<'_>,
                     _| {
            calls.set(calls.get() + 1);
            panic!("output failure reached dispatch");
        };
        let (_master, slave) = faction_client_pty::open(24, 40).unwrap();
        let saved = rustix::termios::tcgetattr(&slave).unwrap();
        let mut term = LocalTerminal {
            input: slave.into(),
            output: std::fs::File::open("/dev/null").unwrap(),
            saved,
        };
        let mut exchange = HomeExchange::new(&f.settings);
        exchange.foreground = Some(&cycle);
        let result = Screen {
            term: &mut term,
            activity: Instant::now(),
            unicode,
        }
        .brief_exchange(&mut f.vault, &exchange, 0, Some(2));
        assert!(result.is_err());
        assert_eq!(calls.get(), 0);
        let capture = f.root.join("io-capture");
        let eof = f.root.join("eof");
        std::fs::write(&eof, []).unwrap();
        term.output = std::fs::File::create(&capture).unwrap();
        let tty = std::mem::replace(&mut term.input, std::fs::File::open(eof).unwrap());
        let result = Screen {
            term: &mut term,
            activity: Instant::now(),
            unicode,
        }
        .body(&"review line\n".repeat(20));
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
        term.input = tty;
        let capture = std::fs::read_to_string(capture).unwrap();
        assert!(capture.contains("[LOCAL BODY] Page 1/3"));
        assert!(!capture.contains("Type exchange") && !capture.contains("HOME account password"));
        assert_eq!(
            std::fs::read(f.root.join("vault/mail.fkv")).unwrap(),
            before
        );
    }
}

#[test]
fn brief_arrivals_empty_duplicates_and_unknown_read_state() {
    child("checkpoint_ui::foreground_tests::brief_review_tests::brief_arrivals_boundaries_helper");
}

#[test]
#[ignore = "owned subprocess helper"]
fn brief_arrivals_boundaries_helper() {
    use std::cell::{Cell, RefCell};
    for unicode in [false, true] {
        for attempt in 0..3 {
            let mut f = Fixture::new();
            let mut store = f.root.join("vault");
            if attempt == 2 {
                f.vault = f.open("cold");
                store = f.root.join("cold");
                f.vault.enable_archive(f.time).unwrap();
                let checkpoint = f.vault.archive_checkpoint(f.time).unwrap();
                let mut wire = f.wire("keep warm unknown");
                wire.sent_at = 0;
                f.vault
                    .receive_archive(&checkpoint, "warm-position", &wire, f.time)
                    .unwrap();
            }
            let wire = wire_body(
                &mut f,
                "hidden unknown arrival",
                "ordinary unknown-read body",
            );
            if attempt == 1 {
                f.vault.receive_network(&wire, f.time).unwrap();
            }
            let before = std::fs::read(store.join("mail.fkv")).unwrap();
            let received = f.vault.read_summary().unwrap().received;
            let wire = RefCell::new(Some(wire));
            let calls = Cell::new(0);
            let cycle = |v: &mut CheckpointVault,
                         _: &str,
                         _: faction_client::mail_network::ForegroundPeer<'_>,
                         _| {
                calls.set(calls.get() + 1);
                if attempt > 0 {
                    v.receive_network(&wire.borrow_mut().take().unwrap(), now().unwrap())
                        .unwrap();
                }
                ForegroundReport {
                    write: ForegroundWrite::NotAttempted,
                    accepted: 99,
                    duplicates: 99,
                    error: None,
                }
            };
            let (result, text, list) = view_controlled(
                &mut f,
                true,
                unicode,
                0,
                inbox::Filter::All,
                Some(&cycle),
                |d| {
                    d.wait("[B] Back");
                    d.send(b"e");
                    d.wait("[B] Return");
                    d.send(b"r");
                    d.wait("Type exchange to connect, or Enter: ");
                    d.send(b"exchange\r");
                    d.wait("HOME account password (hidden; never sent to a BBS): ");
                    d.send(b"fresh home secret\r");
                    d.wait("Enter to return: ");
                    let report = std::fs::read_to_string(&d.capture).unwrap();
                    assert_eq!(
                        report.contains("[R] Review newly received text"),
                        attempt == 2
                    );
                    if attempt == 2 {
                        d.send(b"r");
                        d.wait("[LOCAL NEWLY RECEIVED TEXT]");
                        d.wait("[B] Return");
                        let rows = std::fs::read_to_string(&d.capture).unwrap();
                        assert!(rows.contains("UNKNOWN"));
                        d.send(b"o");
                        d.wait("Body: ordinary unknown-read body");
                        d.wait("Enter to return: ");
                        assert!(
                            !std::fs::read_to_string(&d.capture)
                                .unwrap()
                                .contains("Type read to acknowledge")
                        );
                        d.send(b"\r");
                        d.wait("[LOCAL NEWLY RECEIVED TEXT]");
                        d.wait("[B] Return");
                        d.send(b"b");
                        d.wait("Enter to return: ");
                    }
                    d.send(b"\r");
                    d.wait("[LOCAL CONVERSATION]");
                    d.wait("[B] Back");
                    d.send(b"b");
                },
            );
            result.unwrap();
            assert_eq!(calls.get(), 1);
            assert_eq!(list.selected().unwrap().index, 0);
            assert!(
                !text.contains("Acknowledged on this device") && !text.contains("Reply locally")
            );
            if attempt < 2 {
                assert_eq!(std::fs::read(store.join("mail.fkv")).unwrap(), before);
            } else {
                let summary = f.vault.read_summary().unwrap();
                assert_eq!(summary.unread, None);
                assert_eq!(summary.received, received + 1);
            }
        }
    }
}

fn wire_body(f: &mut Fixture, subject: &str, body: &str) -> Wire {
    let mut wire = f.wire(subject);
    // Use a fresh, authenticated ordinary SCE body, never UI/report metadata.
    let plain = sce::encode("bob@away.test", "alice@home.test", f.time, subject, body).unwrap();
    let sealed = f.remote.seal(&plain).unwrap();
    wire.key = sealed.key;
    wire.prekey = sealed.prekey;
    wire.payload = sealed.payload;
    wire.sent_at = 0;
    wire
}

#[test]
fn brief_arrivals_explicit_review_ack_and_exact_return() {
    child("checkpoint_ui::foreground_tests::brief_review_tests::brief_arrivals_route_helper");
}

#[test]
#[ignore = "owned subprocess helper"]
fn brief_arrivals_route_helper() {
    check_brief_arrivals(false);
}

#[test]
fn brief_arrival_reply() {
    child("checkpoint_ui::foreground_tests::brief_review_tests::brief_arrival_reply_helper");
}

#[test]
#[ignore = "owned subprocess helper"]
fn brief_arrival_reply_helper() {
    check_brief_arrivals(true);
}

fn check_brief_arrivals(reply: bool) {
    use std::cell::{Cell, RefCell};
    let mut missing = Vec::new();
    for unicode in [false, true] {
        for confirm in [false, true] {
            let acknowledge = confirm && !reply;
            let mut f = Fixture::new();
            let original = f.vault.history().unwrap()[0].received_id().unwrap();
            if reply {
                f.vault.mark_read(0, original).unwrap();
            }
            let pending = f.vault.pending_deliveries(f.time).unwrap();
            let cursor = f.vault.archive_checkpoint(f.time).unwrap();
            let summary = f.vault.read_summary().unwrap();
            let matching = wire_body(&mut f, "keep new match", "matching ordinary text");
            let hidden_body = (0..15)
                .map(|n| format!("hidden line {n:02} λ\u{202e}\n"))
                .collect::<String>();
            let hidden = wire_body(&mut f, "hidden arrival λ\u{202e}", &hidden_body);
            let identity = hidden.identity;
            let expected_subject = "Re: hidden arrival λ\u{202e}";
            let expected_display = inert(expected_subject, unicode);
            let active_store = f.root.join("vault/mail.fkv");
            let after_cycle = RefCell::new(None);
            let retain = reply && std::env::var_os("FACTION359_CAPTURE_DIR").is_some();
            if retain {
                std::fs::copy(&active_store, f.root.join("before-cycle")).unwrap();
                std::fs::write(
                    f.root.join("brief-inputs"),
                    format!("{unicode} {confirm}\n{cursor:?}\n{matching:?}\n{hidden:?}\n"),
                )
                .unwrap();
            }
            let incoming = RefCell::new(Some([matching, hidden]));
            let calls = Cell::new(0);
            let cycle = |v: &mut CheckpointVault,
                         _: &str,
                         _: faction_client::mail_network::ForegroundPeer<'_>,
                         action| {
                assert_eq!(action, ForegroundAction::ReceiveOnly);
                calls.set(calls.get() + 1);
                for wire in incoming.borrow_mut().take().unwrap() {
                    v.receive_network(&wire, now().unwrap()).unwrap();
                }
                if reply {
                    after_cycle.replace(Some(std::fs::read(&active_store).unwrap()));
                }
                // Deliberately inaccurate count and a recoverable partial error.
                ForegroundReport {
                    write: ForegroundWrite::NotAttempted,
                    accepted: 0,
                    duplicates: 99,
                    error: Some(faction_client::mail_network::Error::Protocol),
                }
            };
            let (result, text, list) = view_controlled(
                &mut f,
                true,
                unicode,
                usize::from(reply),
                inbox::Filter::Unread,
                Some(&cycle),
                |d| {
                    d.wait("[B] Back");
                    d.send(b"e");
                    d.wait("[B] Return");
                    d.send(b"r");
                    d.wait("Type exchange to connect, or Enter: ");
                    d.send(b"exchange\r");
                    d.wait("HOME account password (hidden; never sent to a BBS): ");
                    d.send(b"fresh home secret\r");
                    d.wait("Enter to return: ");
                    let result = std::fs::read_to_string(&d.capture).unwrap();
                    assert!(
                        result.contains("[R] Review newly received text"),
                        "canonical arrivals missing despite inaccurate count: {result}"
                    );
                    assert!(
                        !result.contains("No new arrival observed."),
                        "count erased canonical arrival observation: {result}"
                    );
                    assert!(
                        !result.contains("[LOCAL NEWLY RECEIVED TEXT]"),
                        "arrivals auto-opened"
                    );
                    d.send(b"r");
                    d.wait("[LOCAL NEWLY RECEIVED TEXT]");
                    d.wait("[B] Return");
                    let arrivals = std::fs::read_to_string(&d.capture).unwrap();
                    let arrivals =
                        &arrivals[arrivals.rfind("[LOCAL NEWLY RECEIVED TEXT]").unwrap()..];
                    assert!(
                        arrivals.contains("keep new match") && arrivals.contains("hidden arrival")
                    );
                    assert!(
                        !arrivals.contains("keep original") && !arrivals.contains("keep selected")
                    );
                    assert!(!arrivals.contains("hidden line"), "listing copied bodies");
                    d.send(b"j");
                    d.wait("selection 2/2");
                    d.wait("[B] Return");
                    if reply {
                        if drive_arrival_reply(d, &expected_display, &identity, confirm) {
                            d.wait("selection 2/2");
                            d.wait("[B] Return");
                        }
                    } else {
                        d.send(b"o");
                        d.wait("[LOCAL BODY] Page 1/2");
                        d.wait("[Enter] Next page  [B] Return");
                        let page = std::fs::read_to_string(&d.capture).unwrap();
                        assert!(!page.contains("Type read to acknowledge"));
                        d.send(b"b"); // Cancelling pages cannot acknowledge.
                        d.wait("[LOCAL NEWLY RECEIVED TEXT]");
                        d.wait("[B] Return");
                        d.send(b"o");
                        d.wait("[Enter] Next page  [B] Return");
                        d.send(b"\r");
                        d.wait(
                            "Type read to acknowledge locally, or Enter to keep current state: ",
                        );
                        let pages = std::fs::read_to_string(&d.capture).unwrap();
                        assert!(pages.contains("hidden line 14"));
                        d.send(if acknowledge { b"read\r" } else { b"\r" });
                        d.wait("Enter to return: ");
                        let reader = std::fs::read_to_string(&d.capture).unwrap();
                        let reader = &reader[reader.rfind("[LOCAL INBOX RECORD]").unwrap()..];
                        assert!(!reader.contains("Write multiline reply"));
                        d.send(b"\r");
                        d.wait("[LOCAL NEWLY RECEIVED TEXT]");
                        d.wait("[B] Return");
                    }
                    d.send(b"b");
                    d.wait("Enter to return: ");
                    d.send(b"\r");
                    d.wait("[LOCAL CONVERSATION]");
                    d.wait("[B] Back");
                    d.send(b"b");
                },
            );
            result.unwrap();
            let writer = text.contains("[LOCAL MULTILINE WRITER]");
            let queued = reply && confirm && writer;
            if reply {
                if !writer {
                    missing.push((unicode, confirm));
                }
                let after_cycle = after_cycle.into_inner().unwrap();
                if !queued {
                    assert_eq!(std::fs::read(&active_store).unwrap(), after_cycle);
                }
                if retain {
                    std::fs::write(f.root.join("after-cycle"), after_cycle).unwrap();
                }
                assert!(!text.contains("[LOCAL INBOX RECORD]") && !text.contains("Type read"));
            }
            assert_eq!(calls.get(), 1);
            assert_eq!(text.matches("HOME account password").count(), 1);
            assert_eq!(f.vault.archive_checkpoint(f.time).unwrap(), cursor);
            assert_eq!(
                f.vault.read_summary().unwrap().unread,
                summary.unread.map(|n| n + 2 - usize::from(acknowledge))
            );
            let matching = f.vault.history().unwrap()[4].received_id().unwrap();
            let hidden = f.vault.history().unwrap()[5].received_id().unwrap();
            assert_ne!(matching, hidden);
            assert_eq!(f.vault.received_read(5, hidden).unwrap(), Some(acknowledge));
            assert_eq!(list.mode(), inbox::Filter::Unread);
            assert_eq!(list.selected().unwrap().index, usize::from(reply));
            assert_eq!(
                list.eligible().iter().map(|e| e.index).collect::<Vec<_>>(),
                if reply { vec![4, 1] } else { vec![4, 1, 0] }
            );
            let tail = &text[text.rfind("[LOCAL CONVERSATION]").unwrap()..];
            assert!(tail.contains("Unread / Search"));
            assert!(!tail.contains("hidden arrival"));
            assert!(!text.contains("Reply locally") && !text.contains("fresh home secret"));
            assert!(text.contains("Brief exchange stopped."));
            let placeholder = f.open("cold");
            drop(std::mem::replace(&mut f.vault, placeholder));
            f.vault = CheckpointVault::unlock(
                Store::at(&f.root.join("vault"), false).unwrap(),
                SECRET,
                false,
            )
            .unwrap();
            check_arrival_reply(&f, 6, expected_subject, &pending, queued);
            assert_eq!(f.vault.archive_checkpoint(f.time).unwrap(), cursor);
            assert_eq!(f.vault.received_read(0, original).unwrap(), Some(reply));
            assert_eq!(f.vault.received_read(4, matching).unwrap(), Some(false));
            assert_eq!(f.vault.received_read(5, hidden).unwrap(), Some(acknowledge));
        }
    }
    assert!(
        missing.is_empty(),
        "missing [W] Write multiline reply on the brief arrival list; W opened no writer (unicode, confirm): {missing:?}"
    );
}

#[test]
fn brief_arrivals_use_canonical_frame_identity() {
    child("checkpoint_ui::foreground_tests::brief_review_tests::brief_arrivals_identity_helper");
}

#[test]
#[ignore = "owned subprocess helper"]
fn brief_arrivals_identity_helper() {
    let mut f = Fixture::new();
    let target = foreground::Target::prepare(&f.vault, &f.settings, 0, f.time).unwrap();
    let before = foreground::Arrivals::capture(&f.vault).unwrap();
    assert!(before.new_text(&f.vault, &target).unwrap().is_empty());
    let wire = wire_body(&mut f, "hidden", "new authenticated body");
    f.vault.receive_network(&wire, f.time).unwrap();
    let frame = f.vault.history().unwrap()[4].received_id().unwrap();
    assert_eq!(
        before.new_text(&f.vault, &target).unwrap(),
        vec![(4, frame)]
    );
    f.vault.receive_network(&wire, f.time).unwrap();
    f.vault
        .queue([3; 16], "new sent", "not received", f.time, f.time)
        .unwrap();
    assert_eq!(
        before.new_text(&f.vault, &target).unwrap(),
        vec![(4, frame)]
    );
    for wrong in [
        foreground::Target {
            address: Zeroizing::new("foreign@away.test".into()),
            device: target.device,
            identity: target.identity,
        },
        foreground::Target {
            address: Zeroizing::new(target.address.to_string()),
            device: 203,
            identity: target.identity,
        },
        foreground::Target {
            address: Zeroizing::new(target.address.to_string()),
            device: target.device,
            identity: [99; 32],
        },
    ] {
        assert!(before.new_text(&f.vault, &wrong).unwrap().is_empty());
    }
    let after = foreground::Arrivals::capture(&f.vault).unwrap();
    f.vault.receive_network(&wire, f.time).unwrap();
    assert!(after.new_text(&f.vault, &target).unwrap().is_empty());
    f.vault.enable_attachments().unwrap();
    let descriptor = faction_omemo::attachment::Descriptor {
        name: Zeroizing::new("private.txt".into()),
        media_type: Zeroizing::new("text/plain".into()),
        size: 1,
        hash: Zeroizing::new([3; 32]),
        key: Zeroizing::new([4; 32]),
        iv: Zeroizing::new([5; 12]),
        locator: Zeroizing::new("https://blob.test/private".into()),
    };
    let plain = faction_omemo::attachment::encode(
        "bob@away.test",
        "alice@home.test",
        f.time,
        "attachment",
        "excluded ordinary-text caption",
        &descriptor,
    )
    .unwrap();
    let sealed = f.remote.seal(&plain).unwrap();
    let mut attachment = wire.clone();
    attachment.key = sealed.key;
    attachment.prekey = sealed.prekey;
    attachment.payload = sealed.payload;
    f.vault.receive_network(&attachment, f.time).unwrap();
    assert!(after.new_text(&f.vault, &target).unwrap().is_empty());
    f.rotate("bob@away.test", 202);
    assert!(
        before.new_text(&f.vault, &target).unwrap().is_empty(),
        "replaced identity reviewed as original peer"
    );
}
