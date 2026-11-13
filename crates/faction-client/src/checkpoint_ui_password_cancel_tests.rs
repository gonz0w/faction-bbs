//! Blank hidden input through the real local screens, with controlled dispatch.
use super::*;
use std::cell::Cell;

const PROMPT: &str = "HOME account password (hidden; never sent to a BBS): ";
const HINT: &str = "Enter on an empty password cancels.";
const CANCELLED: &str = "Home exchange cancelled.";
const MENU: &str = "[LOCAL / MANUAL ONLINE] Encrypted home mail";
const INBOX: &str = "[LOCAL INBOX]";
const CONVERSATION: &str = "[LOCAL CONVERSATION]";

// Let the old controlled dispatch finish too so RED records actual dispatches.
fn wait_for_return(driver: &mut Driver, view: &str) {
    let limit = Instant::now() + Duration::from_secs(15);
    let pause = "Enter to return: ";
    loop {
        let text = std::fs::read_to_string(&driver.capture).unwrap();
        let tail = &text[driver.offset..];
        if let Some(at) = tail.find(view) {
            driver.offset += at + view.len();
            return;
        }
        if let Some(at) = tail.find(pause) {
            driver.offset += at + pause.len();
            driver.send(b"\r");
        }
        assert!(Instant::now() < limit, "missing return to {view}: {text}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn assert_cancelled(text: &str) {
    assert!(text.contains(&format!("{HINT}\r\n{PROMPT}\r\n{CANCELLED}\r\n")));
    assert!(!text.contains("Connecting directly to your home."));
    assert!(!text.contains("Enter to return: "));
}

fn fixture() -> Fixture {
    let mut f = Fixture::new();
    let different = f.wire("different subject");
    f.vault.receive(&different, f.time).unwrap();
    f
}

#[test]
fn home_password_cancel_manual_pty() {
    child("checkpoint_ui::foreground_tests::password_cancel_tests::manual_helper");
}

#[test]
#[ignore = "owned subprocess helper"]
fn manual_helper() {
    let mut f = fixture();
    let canonical = f.root.join("vault/mail.fkv");
    let before = std::fs::read(&canonical).unwrap();
    let mut observed = Vec::new();
    for unicode in [false, true] {
        for inbox_route in [false, true] {
            for action in [b'a', b'n'] {
                let calls = Cell::new(0);
                let cycle = |_: &mut CheckpointVault, _: &str, _| {
                    calls.set(calls.get() + 1);
                    Ok(faction_client::mail_network::Report::default())
                };
                let mut exchange = HomeExchange::new(&f.settings);
                exchange.cycle = Some(&cycle);
                let mut list =
                    inbox::Inbox::new(Screen::inbox_group_entries(&f.vault, None).unwrap());
                list.filter(inbox::Filter::Unread);
                let mut search = inbox::Search::parse(b"keep").unwrap();
                list.search(Screen::inbox_matches(&f.vault, &search).unwrap());
                list.select_index(0);

                let (master, slave) = faction_client_pty::open(24, 40).unwrap();
                let saved = rustix::termios::tcgetattr(&slave).unwrap();
                let mut raw = saved.clone();
                raw.make_raw();
                rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw)
                    .unwrap();
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
                std::thread::scope(|scope| {
                    let input = scope.spawn(move || {
                        if inbox_route {
                            driver.wait("[B] Back");
                            driver.send(b"x");
                            driver.wait("[B] Return");
                        } else {
                            driver.wait("[L] Lock and exit");
                        }
                        driver.send(&[action]);
                        driver.wait(PROMPT);
                        driver.send(b"\r");
                        if inbox_route {
                            wait_for_return(&mut driver, INBOX);
                            driver.wait("[B] Back");
                            driver.send(b"b");
                        } else {
                            wait_for_return(&mut driver, MENU);
                            driver.wait("[L] Lock and exit");
                            driver.send(b"l");
                        }
                        driver
                    });
                    let mut screen = Screen {
                        term: &mut term,
                        activity: Instant::now(),
                        unicode,
                    };
                    if inbox_route {
                        screen
                            .inbox_view(
                                &mut f.vault,
                                true,
                                Some(&exchange),
                                None,
                                &mut list,
                                &mut search,
                            )
                            .unwrap();
                    } else {
                        let key = screen.online_menu(&f.vault).unwrap();
                        assert_eq!(key, action.to_ascii_uppercase());
                        screen
                            .online_action(
                                &mut f.vault,
                                &f.settings,
                                &f.root,
                                key,
                                None,
                                Some(&cycle),
                            )
                            .unwrap();
                        assert_eq!(screen.online_menu(&f.vault).unwrap(), b'L');
                    }
                    input.join().unwrap();
                });
                drop(term);

                assert_eq!(std::fs::read(&canonical).unwrap(), before);
                let text = std::fs::read_to_string(capture).unwrap();
                if inbox_route {
                    assert_eq!(list.mode(), inbox::Filter::Unread);
                    assert_eq!(list.selected().unwrap().index, 0);
                    assert_eq!(
                        list.eligible().iter().map(|e| e.index).collect::<Vec<_>>(),
                        [1, 0]
                    );
                    let returned = &text[text.rfind(INBOX).unwrap()..];
                    assert!(returned.contains("Local search active."));
                    assert!(returned.contains("> 01 UNREAD"));
                }
                observed.push(((unicode, inbox_route, action as char, calls.get()), text));
            }
        }
    }
    let dispatched: Vec<_> = observed
        .iter()
        .filter_map(|(route, _)| (route.3 != 0).then_some(*route))
        .collect();
    assert!(
        dispatched.is_empty(),
        "blank password dispatched (unicode, inbox, action, calls): {dispatched:?}"
    );
    for (_, text) in observed {
        assert_cancelled(&text);
    }
}

#[test]
fn home_password_cancel_brief_pty() {
    child("checkpoint_ui::foreground_tests::password_cancel_tests::brief_helper");
}

#[test]
#[ignore = "owned subprocess helper"]
fn brief_helper() {
    let mut f = fixture();
    let canonical = f.root.join("vault/mail.fkv");
    let before = std::fs::read(&canonical).unwrap();
    let pending = f.vault.pending(f.time).unwrap();
    let cursor = f.vault.archive_checkpoint(f.time).unwrap();
    let mut observed = Vec::new();
    for unicode in [false, true] {
        for send in [false, true] {
            let calls = Cell::new(0);
            let cycle = |_: &mut CheckpointVault,
                         _: &str,
                         _: faction_client::mail_network::ForegroundPeer<'_>,
                         _| {
                calls.set(calls.get() + 1);
                ForegroundReport {
                    write: ForegroundWrite::NotAttempted,
                    accepted: 0,
                    duplicates: 0,
                    error: None,
                }
            };
            let (selected, filter, eligible, scope) = if send {
                (2, inbox::Filter::All, &([3, 2, 1, 0])[..], "All / Search")
            } else {
                (0, inbox::Filter::Unread, &([1, 0])[..], "Unread / Search")
            };
            let (result, text, list) =
                view_controlled(&mut f, true, unicode, selected, filter, Some(&cycle), |d| {
                    d.wait("[B] Back");
                    d.send(b"e");
                    d.wait("[B] Return");
                    d.send(if send { b"s" } else { b"r" });
                    if send {
                        d.wait("Send selected text only.");
                        d.wait("private selected body");
                    } else {
                        d.wait("Receive briefly; no text release.");
                    }
                    d.wait("Type exchange to connect, or Enter: ");
                    d.send(b"exchange\r");
                    d.wait(PROMPT);
                    d.send(b"\r");
                    wait_for_return(d, CONVERSATION);
                    d.wait("[B] Back");
                    d.send(b"b");
                });
            result.unwrap();
            assert_eq!(std::fs::read(&canonical).unwrap(), before);
            assert_eq!(f.vault.pending(f.time).unwrap(), pending);
            assert_eq!(f.vault.archive_checkpoint(f.time).unwrap(), cursor);
            assert_eq!(list.mode(), filter);
            assert_eq!(list.selected().unwrap().index, selected);
            assert_eq!(
                list.eligible().iter().map(|e| e.index).collect::<Vec<_>>(),
                eligible
            );
            assert!(text[text.rfind(CONVERSATION).unwrap()..].contains(scope));
            observed.push(((unicode, send, calls.get()), text));
        }
    }
    let dispatched: Vec<_> = observed
        .iter()
        .filter_map(|(route, _)| (route.2 != 0).then_some(*route))
        .collect();
    assert!(
        dispatched.is_empty(),
        "blank password dispatched (unicode, send, calls): {dispatched:?}"
    );
    for (_, text) in observed {
        assert_cancelled(&text);
    }
}
