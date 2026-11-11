//! Outbox review and retirement use the real checkpoint and local screens.
use super::*;

const OUTBOX_RETURN: &str = "Enter to return: ";
const OUTBOX_SELECT: &str = "Operation ID to open (32 lowercase hex), or Enter to return: ";
const RETIRE_SELECT: &str = "Operation ID to retire (32 lowercase hex, or all): ";
const RETIRE_CONFIRM: &str = "Type retire to record this LOCAL decision: ";

fn menu_action(
    f: &mut Fixture,
    online: bool,
    unicode: bool,
    key: u8,
    name: &str,
    drive: impl FnOnce(&mut Driver) + Send,
) -> (io::Result<()>, String) {
    let (master, slave) = faction_client_pty::open(24, 40).unwrap();
    let saved = rustix::termios::tcgetattr(&slave).unwrap();
    let mut raw = saved.clone();
    raw.make_raw();
    rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw).unwrap();
    let capture = f.root.join(format!("{name}-{online}-{unicode}"));
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
    let result = std::thread::scope(|scope| {
        let handle = scope.spawn(move || {
            drive(&mut driver);
            driver
        });
        let mut screen = Screen {
            term: &mut term,
            activity: Instant::now(),
            unicode,
        };
        let result = if online {
            screen.online_action(&mut f.vault, &f.settings, &f.root, key, None)
        } else {
            screen.action(&mut f.vault, key)
        };
        handle.join().unwrap();
        result
    });
    drop(term);
    (result, std::fs::read_to_string(capture).unwrap())
}

#[test]
fn pending_operation_review_is_exact_and_local() {
    child("checkpoint_ui::foreground_tests::pending_tests::review_helper");
}

#[test]
#[ignore = "owned subprocess helper"]
fn review_helper() {
    let mut f = Fixture::new();
    f.vault.acknowledge([1; 16]).unwrap();
    let body = format!(
        "selected λ\u{202e}\n{}end selected",
        "continued\n".repeat(10)
    );
    f.vault
        .queue([3; 16], "review this λ", &body, f.time, f.time)
        .unwrap();
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
    f.vault
        .queue_attachment(
            [4; 16],
            "retained attachment",
            "attachment body must stay closed",
            &descriptor,
            f.time,
            f.time,
        )
        .unwrap();
    let checkpoint = f.root.join("vault/mail.fkv");
    let before = std::fs::read(&checkpoint).unwrap();
    for online in [false, true] {
        for unicode in [false, true] {
            let (result, output) = menu_action(&mut f, online, unicode, b'O', "review", |d| {
                d.wait(OUTBOX_RETURN);
                if !std::fs::read_to_string(&d.capture)
                    .unwrap()
                    .contains(OUTBOX_SELECT)
                {
                    d.send(b"\r");
                    return;
                }
                for invalid in ["bad".into(), hex(&[9; 16]), hex(&[1; 16]), hex(&[4; 16])] {
                    d.send(format!("{invalid}\r").as_bytes());
                    d.wait("Pending text unavailable.");
                    d.wait(OUTBOX_SELECT);
                }
                d.send(format!("{}\r", hex(&[3; 16])).as_bytes());
                d.wait("[LOCAL PENDING TEXT]");
                d.wait("To: bob@away.test / device 202");
                d.wait(if unicode {
                    "Subject: review this λ"
                } else {
                    "Subject: review this \\u{3bb}"
                });
                d.wait("[LOCAL BODY] Page 1/2");
                d.wait(if unicode {
                    "selected λ\\u{202e}"
                } else {
                    "selected \\u{3bb}\\u{202e}"
                });
                d.wait("[Enter] Next page  [B] Return");
                d.send(b"b");
                d.wait(OUTBOX_SELECT);
                d.send(format!("{}\r", hex(&[3; 16])).as_bytes());
                d.wait("[Enter] Next page  [B] Return");
                d.send(b"\r");
                d.wait("[LOCAL BODY] Page 2/2");
                d.wait("end selected");
                d.wait("[Enter] Return to outbox  [B] Return");
                d.send(b"\r");
                d.wait(OUTBOX_SELECT);
                d.send(b"\r");
            });
            result.unwrap();
            assert_eq!(std::fs::read(&checkpoint).unwrap(), before);
            assert!(
                output.contains(OUTBOX_SELECT),
                "O offered no local text review: {output}"
            );
            assert_eq!(output.matches("[LOCAL PENDING TEXT]").count(), 2);
            assert_eq!(output.matches("[LOCAL BODY] Page 2/2").count(), 1);
            for absent in [
                "private other body",
                "private selected body",
                "attachment body must stay closed",
                "HOME account password",
                "\u{202e}",
            ] {
                assert!(
                    !output.contains(absent),
                    "unexpected output {absent:?}: {output}"
                );
            }
        }
    }
}

#[test]
fn pending_operation_retirement_reviews_target_before_confirmation() {
    child("checkpoint_ui::foreground_tests::pending_tests::retirement_helper");
}

#[test]
#[ignore = "owned subprocess helper"]
fn retirement_helper() {
    for (online, unicode) in [(false, false), (true, true)] {
        let mut f = Fixture::new();
        let checkpoint = f.root.join("vault/mail.fkv");
        let before = std::fs::read(&checkpoint).unwrap();
        let selected = hex(&[2; 16]);
        for (name, choice, answer) in [
            ("cancel-all", "all", ""),
            ("cancel-one", selected.as_str(), ""),
            ("retire-one", selected.as_str(), "retire"),
        ] {
            let (result, output) = menu_action(&mut f, online, unicode, b'T', name, |d| {
                d.wait(RETIRE_SELECT);
                d.send(format!("{choice}\r").as_bytes());
                d.wait(RETIRE_CONFIRM);
                d.send(format!("{answer}\r").as_bytes());
                if answer == "retire" {
                    d.wait(OUTBOX_RETURN);
                    d.send(b"\r");
                }
            });
            if answer.is_empty() {
                assert_eq!(result.unwrap_err().to_string(), "confirmation cancelled");
                assert_eq!(std::fs::read(&checkpoint).unwrap(), before);
            } else {
                result.unwrap();
                assert!(output.contains("Retired locally. Delivery remains unproven"));
            }
            let (_, confirmation) = output.split_once(RETIRE_SELECT).unwrap();
            let (_, review) = confirmation.split_once("\r\n").unwrap();
            let (review, _) = review.split_once(RETIRE_CONFIRM).unwrap();
            if choice == "all" {
                assert!(
                    review.contains("Retire all pending ciphertext (2 operations)."),
                    "all count missing before confirmation: {output}"
                );
            } else {
                assert!(review.contains(&format!("Operation: {selected}\r\nTo: bob@away.test / device 202\r\nSubject: keep other pending")), "exact local target missing before confirmation: {output}");
                assert!(!review.contains(&hex(&[1; 16])));
            }
            assert!(output.contains("It proves neither delivery nor reading"));
            assert!(!output.contains("private other body"));
        }
        f.save("after-retirement");
        let reopened = f.open("after-retirement");
        let pending = reopened.inspect_outbox().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].operation, [1; 16]);
        let history = reopened.history().unwrap();
        assert_eq!(history.len(), 4);
        assert_eq!(history[2].operation(), [1; 16]);
        assert_eq!(history[2].body.as_str(), "private selected body");
        assert_eq!(history[3].operation(), [2; 16]);
        assert_eq!(history[3].body.as_str(), "private other body");
    }
}
