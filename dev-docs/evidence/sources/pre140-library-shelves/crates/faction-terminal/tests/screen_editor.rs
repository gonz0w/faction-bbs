use faction_terminal::{
    Theme,
    composer::{Command, Draft, Screen, ScreenEvent, Target},
    text,
};
use std::time::{Duration, Instant};
fn feed(s: &mut Screen, d: &mut Draft, bytes: &[u8], unicode: bool) {
    for &b in bytes {
        s.feed(d, b, Instant::now(), (40, 18), unicode).unwrap();
    }
}
#[test]
fn caret_navigation_inserts_without_splitting_utf8_and_deletes_whole_scalars() {
    let mut d = Draft::from_body("one 界").unwrap();
    let mut s = Screen::at_end(&d);
    feed(&mut s, &mut d, b"\x1b[D", true);
    feed(&mut s, &mut d, b"bright ", true);
    assert_eq!(d.body(), "one bright 界");
    feed(&mut s, &mut d, b"\x04", true);
    assert_eq!(d.body(), "one bright ");
    feed(&mut s, &mut d, "界".as_bytes(), true);
    feed(&mut s, &mut d, b"\x7f", true);
    assert_eq!(d.body(), "one bright ");
}
#[test]
fn paragraph_split_join_start_end_and_line_deletion_work() {
    let mut d = Draft::from_body("first\nsecond").unwrap();
    let mut s = Screen::at_end(&d);
    feed(&mut s, &mut d, b"\x01\x7f", false);
    assert_eq!(d.body(), "firstsecond");
    feed(&mut s, &mut d, b"\r", false);
    assert_eq!(d.body(), "first\nsecond");
    feed(&mut s, &mut d, b"\x05\x15", false);
    assert_eq!(d.body(), "first\n");
    feed(&mut s, &mut d, b"new\x01\x0b", false);
    assert_eq!(d.body(), "first\n");
}
#[test]
fn visible_rows_and_unicode_cells_keep_caret_on_a_bounded_frame() {
    let body = (0..24)
        .map(|i| format!("row{i:02} 界"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut d = Draft::from_body(&body).unwrap();
    let mut s = Screen::at_end(&d);
    for unicode in [false, true] {
        let frame = s.render(
            &d,
            &Target {
                room: 1,
                reply: None,
                subject: "Échos 界",
            },
            (40, 18),
            unicode,
            &Theme::default(),
            (false, "not saved"),
        );
        assert!(frame.len() < 8192);
        assert!(!frame.contains("row00"));
        assert!(frame.contains("row23"));
        for line in frame.split('\n').skip(3).take(14) {
            assert!(text::width(line) <= 39);
        }
    }
    feed(&mut s, &mut d, b"\x1b[A!", true);
    assert!(d.body().contains("row22 界!"));
}
#[test]
fn quotas_fail_atomically_and_control_strings_cannot_send_or_save() {
    let mut d = Draft::from_body(&"x".repeat(1024)).unwrap();
    let mut s = Screen::at_end(&d);
    assert!(
        s.feed(&mut d, b'y', Instant::now(), (80, 24), false)
            .is_err()
    );
    assert_eq!(d.body().len(), 1024);
    let mut d = Draft::default();
    let mut s = Screen::at_end(&d);
    for &b in b"\x1b]52;clipboard\x14\x13\x07" {
        assert!(!matches!(
            s.feed(&mut d, b, Instant::now(), (80, 24), false).unwrap(),
            ScreenEvent::Command(Command::Send | Command::Save)
        ));
    }
    assert!(d.body().is_empty());
    assert_eq!(
        s.feed(&mut d, 20, Instant::now(), (80, 24), false),
        Ok(ScreenEvent::Command(Command::Send))
    );
    s.feed(&mut d, 27, Instant::now(), (80, 24), false).unwrap();
    assert!(
        s.feed(
            &mut d,
            b'[',
            Instant::now() + Duration::from_secs(1),
            (80, 24),
            false
        )
        .is_err()
    );
    assert_eq!(
        s.feed(&mut d, 3, Instant::now(), (80, 24), false),
        Ok(ScreenEvent::Command(Command::Cancel))
    );
}
#[test]
fn custom_writing_title_is_bounded_inert_and_loaded_from_a_sysop_theme() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let p = std::env::temp_dir().join(format!("faction-writing-theme-{}", std::process::id()));
    fs::create_dir(&p).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(
        p.join("theme.toml"),
        "version=1\nid=\"night-desk\"\nwriting_title=\"Night Desk\"\n",
    )
    .unwrap();
    let t = Theme::load(&p).unwrap();
    assert_eq!(t.writing_title(), "Night Desk");
    fs::write(
        p.join("theme.toml"),
        "version=1\nid=\"night-desk\"\nwriting_title=\"bad\\u001b\"\n",
    )
    .unwrap();
    assert!(Theme::load(&p).is_err());
    fs::remove_dir_all(p).unwrap();
}

#[test]
fn partial_arrow_remains_a_key_when_geometry_changes_between_bytes() {
    let mut draft = Draft::from_body("keep this").unwrap();
    let mut screen = Screen::at_end(&draft);
    screen
        .feed(&mut draft, 27, Instant::now(), (80, 24), false)
        .unwrap();
    screen
        .feed(&mut draft, b'[', Instant::now(), (40, 18), false)
        .unwrap();
    screen
        .feed(&mut draft, b'D', Instant::now(), (40, 18), false)
        .unwrap();
    assert_eq!(draft.body(), "keep this");
    screen
        .feed(&mut draft, b'!', Instant::now(), (40, 18), false)
        .unwrap();
    assert_eq!(draft.body(), "keep thi!s");
}
