// Reuse the immutable pre-batch configured-state contract; PENDIN is state only.
fn assert_configured_terminal(observer: &std::fs::File, saved: &rustix::termios::Termios) {
    let restored = rustix::termios::tcgetattr(observer).unwrap();
    #[cfg(any(target_os = "macos", target_os = "netbsd"))]
    let state = rustix::termios::LocalModes::PENDIN;
    #[cfg(not(any(target_os = "macos", target_os = "netbsd")))]
    let state = rustix::termios::LocalModes::empty();
    assert_eq!(restored.local_modes & !state, saved.local_modes & !state);
    assert_eq!(restored.input_modes, saved.input_modes);
    assert_eq!(restored.output_modes, saved.output_modes);
    assert_eq!(restored.control_modes, saved.control_modes);
    assert_eq!(restored.input_speed(), saved.input_speed());
    assert_eq!(restored.output_speed(), saved.output_speed());
    assert_eq!(
        format!("{:?}", restored.special_codes),
        format!("{:?}", saved.special_codes)
    );
}
use super::body::Body;
#[cfg(feature = "network-mail")]
use super::writer::{Effect, Writer};

#[test]
fn body_pages_preserve_all_scalars_offsets_blank_lines_and_bounds() {
    for unicode in [false, true] {
        for source in [
            "".into(),
            "short".into(),
            "x".repeat(39),
            "x".repeat(40),
            "\n".into(),
            "a\n\nb\n".into(),
            "\n".repeat(1024),
            "x".repeat(1024),
            "界λe\u{301}\r\t\x1b]52;c;x\x07\u{85}\u{202e}\u{2066}".repeat(25),
        ] {
            let mut body = Body::new(&source, unicode);
            let mut recovered = String::new();
            let mut rows = 0;
            let mut pages = 0;
            let count = body.page_count();
            let mut previous = 0;
            while let Some(page) = body.next_page() {
                pages += 1;
                assert!(!page.rows.is_empty() && page.rows.len() <= 10);
                for row in page.rows {
                    assert_eq!(row.source.start, previous);
                    assert!(source.is_char_boundary(row.source.end));
                    recovered.push_str(&source[row.source.clone()]);
                    previous = row.source.end;
                    assert!(faction_terminal::text::width(&row.text) + 2 <= 39);
                    assert!(!row.text.chars().any(char::is_control));
                    let expected = source[row.source.clone()]
                        .strip_suffix('\n')
                        .unwrap_or(&source[row.source]);
                    let inert = super::inert(expected, unicode);
                    assert_eq!(row.text.as_str(), inert.as_str());
                    rows += 1;
                }
                assert!(
                    pages <= 1025,
                    "finite progress including trailing blank row"
                );
            }
            assert_eq!(pages, count);
            assert_eq!(recovered, source);
            if source == "a\n\nb\n" {
                assert_eq!(rows, 4);
            }
            if source == "\n".repeat(1024) {
                assert_eq!(rows, 1025);
            }
            assert!(body.next_page().is_none());
        }
    }
}
#[test]
fn body_short_line_compatibility_and_exact_page_edges() {
    for unicode in [false, true] {
        assert!(Body::new("short", unicode).short_line());
        assert!(Body::new(&"x".repeat(33), unicode).short_line());
        assert!(!Body::new(&"x".repeat(34), unicode).short_line());
        assert!(!Body::new(&"x".repeat(40), unicode).short_line());
        assert!(!Body::new("a\n", unicode).short_line());
        let source = "x\n".repeat(10);
        let mut body = Body::new(&source, unicode);
        assert_eq!(body.page_count(), 2);
        assert_eq!(body.next_page().unwrap().rows.len(), 10);
        let final_page = body.next_page().unwrap();
        assert_eq!(final_page.rows.len(), 1);
        assert!(final_page.rows[0].text.is_empty());
    }
}
#[cfg(feature = "network-mail")]
#[test]
fn writer_commands_keep_exact_body_and_rejections_atomic() {
    let mut w = Writer::new();
    assert!(w.line(".done").is_err());
    assert_eq!(w.line("first").unwrap(), Effect::Added);
    assert_eq!(w.line("").unwrap(), Effect::Added);
    assert_eq!(w.line("..done").unwrap(), Effect::Added);
    assert_eq!(w.body(), "first\n\n.done");
    assert_eq!(w.line(".preview").unwrap(), Effect::Preview);
    assert_eq!(w.line(".help").unwrap(), Effect::Help);
    for bad in [
        ".unknown",
        "bad\nline",
        "bad\rline",
        "bad\tline",
        "bad\u{85}",
    ] {
        assert!(w.line(bad).is_err());
        assert_eq!(w.body(), "first\n\n.done");
    }
    assert_eq!(w.line(".undo").unwrap(), Effect::Changed);
    assert_eq!(w.body(), "first\n");
    w.line("λ界").unwrap();
    assert_eq!(w.body(), "first\n\nλ界");
    assert_eq!(w.line(".done").unwrap(), Effect::Done);
    assert_eq!(w.line(".clear").unwrap(), Effect::Changed);
    assert!(w.body().is_empty());
    w.line(" ").unwrap();
    assert!(w.line(".done").is_err());
    assert_eq!(w.line(".cancel").unwrap(), Effect::Cancel);
    assert!(w.body().is_empty());
}
#[cfg(feature = "network-mail")]
#[test]
fn writer_1024_utf8_bytes_and_32_lines_include_separators() {
    for len in [1023, 1024, 1025] {
        let mut w = Writer::new();
        assert_eq!(w.line(&"x".repeat(len)).is_ok(), len <= 1024);
        assert_eq!(w.body().len(), if len <= 1024 { len } else { 0 });
    }
    let mut w = Writer::new();
    w.line(&"λ".repeat(511)).unwrap();
    w.line("x").unwrap();
    assert_eq!(w.body().len(), 1024);
    assert!(w.line("").is_err());
    assert_eq!(w.body().len(), 1024);
    w.line(".undo").unwrap();
    assert_eq!(w.body().len(), 1022);
    w.line(".clear").unwrap();
    for _ in 0..32 {
        w.line("").unwrap();
    }
    assert_eq!(w.lines(), 32);
    assert_eq!(w.body(), "\n".repeat(31));
    assert!(w.line("x").is_err());
    assert_eq!(w.lines(), 32);
    w.line(".undo").unwrap();
    w.line("x").unwrap();
    assert_eq!(w.lines(), 32);
    assert!(w.line(".done").is_ok());
}

#[test]
fn body_actual_screen_has_whole_row_bounds_and_failure_boundaries() {
    use super::{IDLE, LocalTerminal, Screen};
    use std::{
        io::Write,
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };
    let mut random = [0; 16];
    getrandom::fill(&mut random).unwrap();
    let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
        "faction342-screen-OWNED-{}",
        faction_client::mail_data::hex(&random)
    ));
    std::fs::create_dir(&root).unwrap();
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(
        root.join("OWNED"),
        b"synthetic private body Screen controls\n",
    )
    .unwrap();
    let source = "line\n".repeat(12);
    std::fs::write(root.join("eof"), []).unwrap();
    for unicode in [false, true] {
        for case in ["back", "ctrl-c", "ctrl-d", "eof", "idle", "output", "short"] {
            let (master, slave) = faction_client_pty::open(24, 40).unwrap();
            let mut input = std::fs::File::from(master);
            let saved = rustix::termios::tcgetattr(&slave).unwrap();
            let mut raw = saved.clone();
            raw.make_raw();
            rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw)
                .unwrap();
            let capture = root.join("capture");
            std::fs::write(&capture, b"").unwrap();
            let output = if case == "output" {
                std::fs::File::open(&capture).unwrap()
            } else {
                std::fs::File::create(&capture).unwrap()
            };
            let mut term = LocalTerminal {
                input: slave.into(),
                output,
                saved: saved.clone(),
            };
            let tty = if case == "eof" {
                Some(std::mem::replace(
                    &mut term.input,
                    std::fs::File::open(root.join("eof")).unwrap(),
                ))
            } else {
                None
            };
            input
                .write_all(match case {
                    "ctrl-c" => b"\x03",
                    "ctrl-d" => b"\x04",
                    _ => b"b",
                })
                .unwrap();
            let mut screen = Screen {
                term: &mut term,
                activity: if case == "idle" {
                    Instant::now() - IDLE - Duration::from_secs(1)
                } else {
                    Instant::now()
                },
                unicode,
            };
            let result = screen.body(if case == "short" {
                "short body"
            } else {
                &source
            });
            match case {
                "back" => assert!(!result.unwrap()),
                "short" => assert!(result.unwrap()),
                "ctrl-c" | "ctrl-d" => {
                    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::Interrupted)
                }
                "eof" => assert_eq!(
                    result.unwrap_err().kind(),
                    std::io::ErrorKind::UnexpectedEof
                ),
                "idle" => assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::TimedOut),
                "output" => assert!(result.is_err()),
                _ => unreachable!(),
            }
            if let Some(tty) = tty {
                term.input = tty;
            }
            drop(term);
            assert_configured_terminal(&input, &saved);
            let text = std::fs::read_to_string(&capture).unwrap();
            assert!(
                !text.contains("Type read")
                    && !text.contains("Reply locally")
                    && !text.contains("Type queue")
            );
            assert!(
                text.lines()
                    .filter(|line| line.starts_with("| ") || line.starts_with("Body: "))
                    .all(|line| faction_terminal::text::width(line) <= 39)
            );
            if case == "short" {
                assert_eq!(text, "Body: short body\r\n\r\n");
            }
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn writing_actual_record_and_menu_routes_preserve_fresh_decisions() {
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "checkpoint_ui::writing_tests::writing_actual_record_and_menu_helper",
            "--ignored",
            "--nocapture",
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
#[test]
#[ignore = "owned subprocess helper exercised by writing_actual_record_and_menu_routes_preserve_fresh_decisions"]
fn writing_actual_record_and_menu_helper() {
    use super::{
        LocalTerminal, Screen,
        inbox::{Entry, Status},
        now,
    };
    use faction_client::{
        mail_checkpoint::{CheckpointVault, Wire},
        mail_data::{Data, hex},
        mail_vault::Store,
    };
    use faction_omemo::{Device, sce};
    use std::{
        io::Write,
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };
    let mut random = [0; 16];
    getrandom::fill(&mut random).unwrap();
    let root = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("faction341-route-OWNED-{}", hex(&random)));
    std::fs::create_dir(&root).unwrap();
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(
        root.join("OWNED"),
        b"synthetic actual private-writing Screen route\n",
    )
    .unwrap();
    let path = root.join("vault");
    let time = now().unwrap();
    let (mut vault, _) = CheckpointVault::create(
        Store::at(&path, true).unwrap(),
        Data::new("alice@home.test".into()).unwrap(),
        b"synthetic writing test password",
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
    for subject in ["selected source", "unrelated source"] {
        let plain = sce::encode(
            "bob@away.test",
            "alice@home.test",
            time,
            subject,
            &"body line\n".repeat(12),
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
                    sent_at: time,
                    key: incoming.key,
                    prekey: incoming.prekey,
                    payload: incoming.payload,
                },
                time,
            )
            .unwrap();
    }
    vault.enable_read_tracking().unwrap();
    let source = vault.history().unwrap()[0].received_id().unwrap();
    let unrelated = vault.history().unwrap()[1].received_id().unwrap();
    let before = std::fs::read(path.join("mail.fkv")).unwrap();
    // Queued "read" after the page-next key must be discarded before fresh read input.
    for unicode in [false, true] {
        let (master, slave) = faction_client_pty::open(24, 40).unwrap();
        let mut input = std::fs::File::from(master);
        let saved = rustix::termios::tcgetattr(&slave).unwrap();
        let observer = std::fs::File::from(slave.try_clone().unwrap());
        let mut raw = saved.clone();
        raw.make_raw();
        rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw).unwrap();
        let capture = root.join("record-output");
        let mut term = LocalTerminal {
            input: slave.into(),
            output: std::fs::File::create(&capture).unwrap(),
            saved: saved.clone(),
        };
        std::thread::scope(|scope| {
            let driver = scope.spawn(|| {
                let wait = |needle: &str| {
                    let limit = Instant::now() + Duration::from_secs(15);
                    loop {
                        let text = std::fs::read_to_string(&capture).unwrap();
                        if text.contains(needle) {
                            break;
                        }
                        assert!(
                            Instant::now() < limit,
                            "record route did not reach {needle}"
                        );
                        std::thread::sleep(Duration::from_millis(10));
                    }
                };
                wait("[Enter] Next page");
                let text = std::fs::read_to_string(&capture).unwrap();
                assert!(!text.contains("Type read") && !text.contains("Reply locally"));
                input.write_all(b"\rread\r").unwrap();
                wait("Type read to acknowledge locally");
                input.write_all(b"\r").unwrap();
                #[cfg(feature = "network-mail")]
                {
                    wait("[R] Reply locally");
                    input.write_all(b"b").unwrap();
                }
                #[cfg(not(feature = "network-mail"))]
                {
                    wait("Enter to return: ");
                    input.write_all(b"\r").unwrap();
                }
                input
            });
            Screen {
                term: &mut term,
                activity: Instant::now(),
                unicode,
            }
            .inbox_record_mode(
                &mut vault,
                Entry {
                    index: 0,
                    status: Status::Unread,
                },
                cfg!(feature = "network-mail"),
            )
            .unwrap();
            let _master = driver.join().unwrap();
        });
        drop(term);
        assert_eq!(std::fs::read(path.join("mail.fkv")).unwrap(), before);
        assert_eq!(vault.received_read(0, source).unwrap(), Some(false));
        assert_eq!(vault.received_read(1, unrelated).unwrap(), Some(false));
        let output = std::fs::read_to_string(&capture).unwrap();
        assert!(!output.contains("Acknowledged on this device"));
        assert!(
            output
                .lines()
                .filter(|l| l.starts_with("| "))
                .all(|l| faction_terminal::text::width(l) <= 39)
        );
        assert_configured_terminal(&observer, &saved);
    }
    #[cfg(feature = "network-mail")]
    for unicode in [false, true] {
        for reply in [false, true] {
            for queue in [false, true] {
                let original = std::fs::read(path.join("mail.fkv")).unwrap();
                let history = vault.history().unwrap().len();
                let pending = vault.inspect_outbox().unwrap().len();
                let (master, slave) = faction_client_pty::open(24, 40).unwrap();
                let mut input = std::fs::File::from(master);
                let saved = rustix::termios::tcgetattr(&slave).unwrap();
                let observer = std::fs::File::from(slave.try_clone().unwrap());
                let mut raw = saved.clone();
                raw.make_raw();
                rustix::termios::tcsetattr(&slave, rustix::termios::OptionalActions::Now, &raw)
                    .unwrap();
                let capture = root.join("writer-output");
                let mut term = LocalTerminal {
                    input: slave.into(),
                    output: std::fs::File::create(&capture).unwrap(),
                    saved: saved.clone(),
                };
                std::thread::scope(|scope| {
                    let driver = scope.spawn(|| {
                        let wait = |needle: &str| {
                            let limit = Instant::now() + Duration::from_secs(15);
                            loop {
                                let text = std::fs::read_to_string(&capture).unwrap();
                                if text.contains(needle) {
                                    break;
                                }
                                assert!(
                                    Instant::now() < limit,
                                    "writer route did not reach {needle}"
                                );
                                std::thread::sleep(Duration::from_millis(10));
                            }
                        };
                        if reply {
                            wait("[Enter] Next page");
                            input.write_all(b"\r").unwrap();
                            wait("Type read to acknowledge locally");
                            input.write_all(b"\r").unwrap();
                            wait("[R] Reply locally");
                            input.write_all(b"w").unwrap();
                        } else {
                            input.write_all(b"w").unwrap();
                        }
                        wait(if reply {
                            "Subject (Enter keeps suggestion;"
                        } else {
                            "Subject (128 UTF-8 bytes): "
                        });
                        let text = std::fs::read_to_string(&capture).unwrap();
                        assert!(text.contains("CURRENT recipient: bob@away.test"));
                        assert!(text.contains(&format!(
                            "CURRENT device: 202 / ed25519:{}",
                            hex(&remote.identity)
                        )));
                        input.write_all(b"fresh multiline\r").unwrap();
                        wait("write> ");
                        input
                            .write_all(b"first\r\r..done\rlast\r.preview\r.done\rqueue\r")
                            .unwrap();
                        wait("Type queue to persist encrypted ciphertext: ");
                        input
                            .write_all(if queue { b"queue\r" } else { b"\r" })
                            .unwrap();
                        input
                    });
                    let mut screen = Screen {
                        term: &mut term,
                        activity: Instant::now(),
                        unicode,
                    };
                    if reply {
                        screen
                            .inbox_record_mode(
                                &mut vault,
                                Entry {
                                    index: 0,
                                    status: Status::Unread,
                                },
                                true,
                            )
                            .unwrap();
                    } else {
                        assert_eq!(screen.online_menu(&vault).unwrap(), b'W');
                        screen.write_message(&mut vault, None).unwrap();
                    }
                    let _master = driver.join().unwrap();
                });
                drop(term);
                assert_configured_terminal(&observer, &saved);
                assert_eq!(vault.history().unwrap().len(), history + usize::from(queue));
                assert_eq!(
                    vault.inspect_outbox().unwrap().len(),
                    pending + usize::from(queue)
                );
                assert_eq!(vault.received_read(0, source).unwrap(), Some(false));
                assert_eq!(vault.received_read(1, unrelated).unwrap(), Some(false));
                if queue {
                    let row = vault.history().unwrap().last().unwrap();
                    assert_eq!(row.subject.as_str(), "fresh multiline");
                    assert_eq!(row.body.as_str(), "first\n\n.done\nlast");
                    assert_eq!(row.address, "bob@away.test");
                    assert_eq!(row.device, 202);
                } else {
                    assert_eq!(std::fs::read(path.join("mail.fkv")).unwrap(), original);
                }
            }
        }
    }
    drop(vault);
    std::fs::remove_dir_all(root).unwrap();
}
