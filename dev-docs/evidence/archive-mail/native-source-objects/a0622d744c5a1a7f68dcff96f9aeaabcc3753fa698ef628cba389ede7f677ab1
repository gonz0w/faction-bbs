use faction_core::{Board, request_id};
use faction_pages::Desk;
use faction_terminal::{Action, Key, Menu, Theme};
use std::{fs, os::unix::fs::PermissionsExt};
#[test]
fn preview_visibility_hidden_plan_history_and_explicit_delete() {
    let root = std::env::temp_dir().join(format!("faction-pages-ui-{}", request_id().unwrap()));
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let mut b = Board::open(&root.join("board")).unwrap();
    b.bootstrap("sysop", "synthetic-pages-password").unwrap();
    let s = b.authenticate("sysop", "synthetic-pages-password").unwrap();
    let t = Theme::default();
    let mut d = Desk::settings();
    d.feed("e", &mut b, &s).unwrap();
    d.feed("n", &mut b, &s).unwrap();
    d.feed("Basement people", &mut b, &s).unwrap();
    assert!(d.feed("q", &mut b, &s).unwrap());
    let mut d = Desk::default();
    d.feed("e", &mut b, &s).unwrap();
    d.feed("1", &mut b, &s).unwrap();
    d.feed("Hello from the terminal.", &mut b, &s).unwrap();
    d.feed(".", &mut b, &s).unwrap();
    let frame = d.render(&mut b, &s, &t, false, 40).unwrap();
    assert!(frame.contains("PREVIEW ONLY"));
    assert!(b.member_page(&s, "sysop").unwrap().revision.is_none());
    assert!(!frame.contains('\x1b'));
    d.feed("s", &mut b, &s).unwrap();
    assert_eq!(
        b.member_page(&s, "sysop").unwrap().fields[0],
        "Hello from the terminal."
    );
    d.feed("v", &mut b, &s).unwrap();
    d.feed("none", &mut b, &s).unwrap();
    d.feed("s", &mut b, &s).unwrap();
    assert_eq!(b.member_page(&s, "sysop").unwrap().visible, 0);
    d.feed("p", &mut b, &s).unwrap();
    for i in 0..8 {
        d.feed(&format!("line {i} {}", "X".repeat(45)), &mut b, &s)
            .unwrap();
    }
    d.feed(".", &mut b, &s).unwrap();
    let first = d.render(&mut b, &s, &t, false, 40).unwrap();
    assert!(first.contains("line 0"));
    assert!(!first.contains("line 7"));
    d.feed("", &mut b, &s).unwrap();
    let second = d.render(&mut b, &s, &t, false, 40).unwrap();
    assert!(second.contains("line 7"));
    d.feed("h", &mut b, &s).unwrap();
    d.feed("s", &mut b, &s).unwrap();
    assert!(!b.member_plans(&s, "sysop").unwrap()[0].visible);
    d.feed("d", &mut b, &s).unwrap();
    d.feed("no", &mut b, &s).unwrap();
    assert!(b.member_page(&s, "sysop").unwrap().revision.is_some());
    d.feed("d", &mut b, &s).unwrap();
    d.feed("DELETE", &mut b, &s).unwrap();
    assert!(b.member_page(&s, "sysop").unwrap().revision.is_none());
    assert!(b.member_plans(&s, "sysop").unwrap().is_empty());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn optional_member_action_preserves_classic_menu_with_all_optional_services() {
    let mut t = Theme::default();
    assert_eq!(t.hotkey(b'M', true), None);
    t.set_files_enabled(true).unwrap();
    t.set_directory_enabled(true).unwrap();
    t.set_wall_enabled(true).unwrap();
    t.set_pages_enabled(true).unwrap();
    assert_eq!(t.hotkey(b'M', true), Some(Action::MemberPages));
    let mut m = Menu::with_columns(&t, true, 40);
    for _ in 0..40 {
        let before = m.focus();
        m.feed(Key::Down);
        let frame = m.render();
        assert!(frame.contains("Ctrl-C exits"));
        assert!(!frame.contains("\x1b[19;"));
        assert!(!m.redraw(before).is_empty());
    }
}
