//! Manual archive review through the production terminal screens and archive commits.
use super::*;
use faction_client::mail_network::{ArchiveReport, Error, Mode, Report};
use std::cell::{Cell, RefCell};

#[test]
fn manual_archive_review_direct() {
    child(
        "checkpoint_ui::foreground_tests::archive_review_tests::manual_archive_review_direct_helper",
    );
}

#[test]
fn manual_archive_review_from_home_exchange() {
    child(
        "checkpoint_ui::foreground_tests::archive_review_tests::manual_archive_review_from_home_exchange_helper",
    );
}

#[test]
#[ignore = "owned subprocess helper"]
fn manual_archive_review_direct_helper() {
    check_archive_review(&["direct"]);
}

#[test]
#[ignore = "owned subprocess helper"]
fn manual_archive_review_from_home_exchange_helper() {
    check_archive_review(&["global", "group", "conversation"]);
}

fn check_archive_review(routes: &[&str]) {
    let mut missing = Vec::new();
    for &route in routes {
        for unicode in [false, true] {
            let home_cases = [(Mode::CatchUp, true, unicode), (Mode::Sync, false, false)];
            for &(mode, new_text, partial) in if route == "direct" {
                &[
                    (Mode::CatchUp, true, false),
                    (Mode::CatchUp, true, true),
                    (Mode::CatchUp, false, false),
                ][..]
            } else {
                &home_cases[..]
            } {
                if !archive_review(route, unicode, mode, new_text, partial) {
                    missing.push((route, unicode, partial));
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "missing explicit archive review (route, unicode, partial): {missing:?}"
    );
}

fn archive_review(route: &str, unicode: bool, mode: Mode, new_text: bool, partial: bool) -> bool {
    let mut f = Fixture::new();
    // Start with the saved archive-enabled store before it has any cursor.
    f.vault = f.open("empty-cursor");
    f.vault.enable_read_tracking().unwrap();
    f.vault.enable_attachments().unwrap();

    let mut duplicate = f.wire("keep duplicate");
    duplicate.sent_at = 0;
    assert!(
        !f.vault
            .receive_network(&duplicate, f.time)
            .unwrap()
            .duplicate
    );
    let mut sent = f
        .vault
        .queue([3; 16], "keep own sent", "own body", f.time, f.time)
        .unwrap();
    sent.sent_at = 0;

    let descriptor = faction_omemo::attachment::Descriptor {
        name: Zeroizing::new("not-an-arrival.txt".into()),
        media_type: Zeroizing::new("text/plain".into()),
        size: 1,
        hash: Zeroizing::new([3; 32]),
        key: Zeroizing::new([4; 32]),
        iv: Zeroizing::new([5; 12]),
        locator: Zeroizing::new("https://blob.test/private".into()),
    };
    let attachment = faction_omemo::attachment::encode(
        "bob@away.test",
        "alice@home.test",
        f.time,
        "keep attachment",
        "attachment body",
        &descriptor,
    )
    .unwrap();
    let body = (0..15)
        .map(|n| format!("archive body {n:02} λ\u{202e}\n"))
        .collect::<String>();
    let plain = sce::encode(
        "bob@away.test",
        "alice@home.test",
        f.time,
        "hidden archive text",
        &body,
    )
    .unwrap();
    let mut seal = |plain: &[u8]| {
        let encrypted = f.remote.seal(plain).unwrap();
        Wire {
            key: encrypted.key,
            prekey: encrypted.prekey,
            payload: encrypted.payload,
            ..duplicate.clone()
        }
    };
    let attachment = seal(&attachment);
    let text = seal(&plain);
    let mut mismatch = seal(&plain);
    mismatch.to_device = 999;
    let original = f.vault.history().unwrap()[0].received_id().unwrap();
    let retained = f.vault.history().unwrap().len();
    let pending = f.vault.pending(f.time).unwrap();
    let cursor = f.vault.archive_checkpoint(f.time).unwrap();
    assert!(cursor.after.is_none());

    // Keep only synthetic fixture inputs, in the existing opt-in private sink.
    if std::env::var_os("FACTION359_CAPTURE_DIR").is_some() {
        std::fs::copy(
            f.root.join("empty-cursor/mail.fkv"),
            f.root.join("before-cycle"),
        )
        .unwrap();
        std::fs::write(f.root.join("archive-inputs"), format!(
            "{route} {unicode} {mode:?} {new_text} {partial}\n{cursor:?}\n{duplicate:?}\n{sent:?}\n{attachment:?}\n{text:?}\n{mismatch:?}\n"
        )).unwrap();
    }
    let calls = Cell::new(0);
    let committed = RefCell::new(None);
    let time = f.time;
    let after_cycle = f.root.join("after-cycle");
    let active_store = f.root.join("empty-cursor/mail.fkv");
    let cycle = |v: &mut CheckpointVault, password: &str, action: Mode| {
        calls.set(calls.get() + 1);
        assert_eq!(password, "fresh archive password");
        assert_eq!(action, mode);
        if mode == Mode::Sync {
            return Ok(Report::default());
        }
        let mut receive = |id, wire: &Wire| {
            let token = v.archive_checkpoint(time).unwrap();
            v.receive_archive(&token, id, wire, time)
        };
        assert!(receive("duplicate", &duplicate).unwrap().duplicate);
        if !partial {
            assert!(receive("known-sent", &sent).unwrap().duplicate);
        }
        assert!(!receive("attachment", &attachment).unwrap().duplicate);
        if new_text {
            let receipt = receive("new-text", &text).unwrap();
            assert!(!receipt.duplicate);
            committed.replace(Some(receipt.frame));
        }
        if partial {
            assert!(receive("rejected", &mismatch).is_err());
        }
        if std::env::var_os("FACTION359_CAPTURE_DIR").is_some() {
            std::fs::copy(&active_store, &after_cycle).unwrap();
        }
        if partial {
            Err(Error::Protocol)
        } else {
            Ok(Report {
                archive: Some(ArchiveReport {
                    entries: if new_text { 4 } else { 3 },
                    complete: true,
                }),
                accepted: if new_text { 2 } else { 1 },
                duplicates: 2,
                ..Report::default()
            })
        }
    };
    let exchange = HomeExchange {
        settings: &f.settings,
        cycle: Some(&cycle),
        foreground: None,
    };
    let (master, slave) = faction_client_pty::open(24, 40).unwrap();
    let saved = rustix::termios::tcgetattr(&slave).unwrap();
    let mut raw = saved.clone();
    raw.make_raw();
    rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw).unwrap();
    let capture = f.root.join("capture");
    let mut term = LocalTerminal {
        input: slave.into(),
        output: std::fs::File::create(&capture).unwrap(),
        saved,
    };
    let mut driver = Driver {
        input: master.into(),
        capture: capture.clone(),
        offset: 0,
    };
    let group = matches!(route, "group" | "conversation").then_some(0);
    let mut list = inbox::Inbox::new(Screen::inbox_group_entries(&f.vault, group).unwrap());
    list.filter(inbox::Filter::Unread);
    let mut search = inbox::Search::parse(b"keep").unwrap();
    list.search(Screen::inbox_matches(&f.vault, &search).unwrap());
    list.select_index(0);
    let reviewed = std::thread::scope(|scope| {
        let driver = scope.spawn(move || {
            if route == "direct" {
                driver.wait("[L] Lock and exit");
                driver.send(b"a");
            } else {
                driver.wait("[B] Back");
                driver.send(b"x");
                driver.wait("[LOCAL HOME EXCHANGE]");
                driver.wait("[B] Return");
                driver.send(if mode == Mode::CatchUp { b"a" } else { b"n" });
            }
            driver.wait("HOME account password (hidden; never sent to a BBS): ");
            driver.send(b"fresh archive password\r");
            driver.wait("Enter to return: ");
            let result = std::fs::read_to_string(&driver.capture).unwrap();
            let reviewed = result.contains("[R] Review newly received text");
            assert!(!result.contains("[LOCAL NEWLY RECEIVED TEXT]"));
            assert!(!result.contains("hidden archive text"));
            if reviewed && new_text {
                driver.send(b"r");
                driver.wait("[LOCAL NEWLY RECEIVED TEXT]");
                driver.wait("[B] Return");
                let list = std::fs::read_to_string(&driver.capture).unwrap();
                let rows = &list[list.rfind("[LOCAL NEWLY RECEIVED TEXT]").unwrap()..];
                assert!(rows.contains(&format!(
                    "selection 1/1\r\n> {:02} UNREAD\r\nhidden archive text",
                    retained + 2
                )));
                assert!(!rows.contains("archive body"));
                driver.send(b"o");
                driver.wait("[LOCAL BODY] Page 1/2");
                driver.wait("[Enter] Next page  [B] Return");
                assert!(
                    !std::fs::read_to_string(&driver.capture)
                        .unwrap()
                        .contains("Type read to acknowledge")
                );
                driver.send(b"\r");
                driver.wait("[LOCAL BODY] Page 2/2");
                driver.wait("Type read to acknowledge locally, or Enter to keep current state: ");
                let pages = std::fs::read_to_string(&driver.capture).unwrap();
                assert!(pages.contains(if unicode {
                    "archive body 14 λ\\u{202e}"
                } else {
                    "archive body 14 \\u{3bb}\\u{202e}"
                }));
                assert!(!pages.contains('\u{202e}'));
                driver.send(if unicode { b"read\r" } else { b"\r" });
                driver.wait("Enter to return: ");
                driver.send(b"\r");
                driver.wait("[LOCAL NEWLY RECEIVED TEXT]");
                driver.wait("[B] Return");
                driver.send(b"b");
                driver.wait("Enter to return: ");
            }
            driver.send(if reviewed && unicode { b"b" } else { b"\r" });
            if route == "direct" {
                driver.wait("[L] Lock and exit");
                driver.send(b"l");
            } else {
                driver.wait("[B] Back");
                driver.send(b"b");
            }
            (driver, reviewed)
        });
        let mut screen = Screen {
            term: &mut term,
            activity: Instant::now(),
            unicode,
        };
        if route == "direct" {
            let key = screen.online_menu(&f.vault).unwrap();
            assert_eq!(key, b'A');
            screen
                .manual_exchange(&mut f.vault, &f.settings, key, Some(&cycle))
                .unwrap();
            assert_eq!(screen.online_menu(&f.vault).unwrap(), b'L');
        } else if route == "conversation" {
            screen
                .conversation(&mut f.vault, true, Some(&exchange), 0, &mut list, &search)
                .unwrap();
        } else {
            screen
                .inbox_view(
                    &mut f.vault,
                    true,
                    Some(&exchange),
                    group,
                    &mut list,
                    &mut search,
                )
                .unwrap();
        }
        driver.join().unwrap().1
    });
    drop(term);

    let capture = std::fs::read_to_string(capture).unwrap();
    assert_eq!(capture.matches("HOME account password").count(), 1);
    assert!(!capture.contains("fresh archive password") && !capture.contains("Reply locally"));
    assert!(capture.contains(if mode == Mode::Sync {
        "Manual cycle complete."
    } else if partial {
        "Catch-up stopped; earlier accepted entries may already be saved."
    } else {
        "Server marked this page complete; this does not prove complete server history."
    }));
    if route != "direct" {
        assert_eq!(list.mode(), inbox::Filter::Unread);
        assert_eq!(list.selected().unwrap().index, 0);
        assert_eq!(
            list.eligible()
                .iter()
                .map(|entry| entry.index)
                .collect::<Vec<_>>(),
            if mode == Mode::CatchUp {
                vec![retained, 1, 0]
            } else {
                vec![1, 0]
            }
        );
        let label = match route {
            "global" => "[LOCAL INBOX]",
            "group" => "[LOCAL CORRESPONDENT INBOX]",
            _ => "[LOCAL CONVERSATION]",
        };
        assert_eq!(capture.matches(label).count(), 2);
        let returned = &capture[capture.rfind(label).unwrap()..];
        assert!(returned.contains("> 01 ") && !returned.contains("hidden archive text"));
        assert!(returned.contains("Unread / Search") || returned.contains("Local search active."));
    }
    assert_eq!(calls.get(), 1);
    assert!(
        !reviewed || new_text,
        "non-text archive records or sync gained review"
    );
    // Reopen the canonical store before checking exact cursor, pending wires and read bits.
    let placeholder = f.open("cold");
    drop(std::mem::replace(&mut f.vault, placeholder));
    f.vault = f.open("empty-cursor");
    assert_eq!(f.vault.pending(time).unwrap(), pending);
    let mut expected_cursor = cursor;
    if mode == Mode::CatchUp {
        expected_cursor.after = Some(if new_text { "new-text" } else { "attachment" }.into());
    }
    assert_eq!(f.vault.archive_checkpoint(time).unwrap(), expected_cursor);
    assert_eq!(f.vault.received_read(0, original).unwrap(), Some(false));
    let summary = f.vault.read_summary().unwrap();
    assert_eq!(
        summary.retained,
        retained + usize::from(mode == Mode::CatchUp) + usize::from(new_text)
    );
    if let Some(frame) = committed.into_inner() {
        assert_eq!(
            f.vault.received_read(retained + 1, frame).unwrap(),
            Some(reviewed && unicode)
        );
    }
    !new_text || reviewed
}
