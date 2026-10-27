use faction_core::{Board, request_id};
use faction_terminal::{Action, Key, Menu, Theme};
use faction_wall::Desk;
use std::{fs, os::unix::fs::PermissionsExt};
#[test]
fn explicit_review_classic_plain_inert_ascii_and_stable_moderation() {
    let root = std::env::temp_dir().join(format!("faction-wall-desk-{}", request_id().unwrap()));
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let mut b = Board::open(&root.join("board")).unwrap();
    b.bootstrap("sysop", "synthetic-wall-password").unwrap();
    let s = b.authenticate("sysop", "synthetic-wall-password").unwrap();
    let theme = Theme::default();
    let mut desk = Desk::settings();
    let mut frame = desk.render(&mut b, &s, &theme, false, 40).unwrap();
    assert!(frame.contains("Enabled: false"));
    desk.feed("e", &mut b, &s).unwrap();
    desk.feed("n", &mut b, &s).unwrap();
    desk.feed("Basement signals", &mut b, &s).unwrap();
    desk.feed("l", &mut b, &s).unwrap();
    desk.feed("16 240 7 1", &mut b, &s).unwrap();
    assert!(desk.feed("q", &mut b, &s).unwrap());
    let mut desk = Desk::default();
    desk.render(&mut b, &s, &theme, false, 40).unwrap();
    desk.feed("p", &mut b, &s).unwrap();
    desk.feed(" /\\", &mut b, &s).unwrap();
    desk.feed("/  \\", &mut b, &s).unwrap();
    desk.feed(".", &mut b, &s).unwrap();
    frame = desk.render(&mut b, &s, &theme, false, 40).unwrap();
    assert!(frame.contains("NOT PUBLISHED"));
    assert!(!frame.contains('\x1b'));
    assert_eq!(b.wall_page(&s, None, 0).unwrap().count, 0);
    desk.feed("s", &mut b, &s).unwrap();
    desk.render(&mut b, &s, &theme, false, 40).unwrap();
    let selected = b.wall_page(&s, None, 0).unwrap().note.unwrap().id;
    // An invalid input cannot modify already published data.
    desk.feed("p", &mut b, &s).unwrap();
    assert!(desk.feed("\x1b[2J", &mut b, &s).is_err());
    assert!(desk.feed(&"X".repeat(73), &mut b, &s).is_err());
    desk.feed("/cancel", &mut b, &s).unwrap();
    desk.render(&mut b, &s, &theme, false, 40).unwrap();
    b.wall_delete(&s, selected).unwrap();
    assert!(desk.feed("x", &mut b, &s).is_err());
    assert_eq!(b.wall_page(&s, None, 0).unwrap().count, 0);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn optional_wall_custom_theme_and_compact_all_services_menu() {
    let mut theme = Theme::default();
    assert_eq!(theme.hotkey(b'W', true), None);
    theme.set_files_enabled(true).unwrap();
    theme.set_directory_enabled(true).unwrap();
    theme.set_wall_enabled(true).unwrap();
    assert_eq!(theme.hotkey(b'W', true), Some(Action::Wall));
    let mut menu = Menu::with_columns(&theme, true, 40);
    for _ in 0..30 {
        let old = menu.focus();
        menu.feed(Key::Down);
        let frame = menu.render();
        for part in frame.split("\x1b[").skip(1) {
            if let Some(end) = part.find('H') {
                let pos = &part[..end];
                if let Some((row, _)) = pos.split_once(';')
                    && let Ok(row) = row.parse::<usize>()
                {
                    assert!(row <= 18, "{row}");
                }
            }
        }
        assert!(!menu.redraw(old).is_empty());
    }
    // Custom action labels and keys remain declarations, never enabling authority.
    let root = std::env::temp_dir().join(format!("faction-wall-theme-{}", request_id().unwrap()));
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let mut pack = String::from("version=1\nid='wall-test'\n");
    for (action, key, label) in [
        ("rooms", "R", "Rooms"),
        ("new", "N", "New"),
        ("history", "H", "History"),
        ("post", "P", "Post"),
        ("sysop", "S", "Sysop"),
        ("quit", "Q", "Quit"),
        ("wall", "Z", "Graffiti"),
    ] {
        pack += &format!("\n[[menu]]\naction='{action}'\nkey='{key}'\nlabel='{label}'\n");
    }
    fs::write(root.join("theme.toml"), pack).unwrap();
    let mut custom = Theme::load(&root).unwrap();
    assert_eq!(custom.hotkey(b'Z', true), None);
    custom.set_wall_enabled(true).unwrap();
    assert_eq!(custom.hotkey(b'Z', true), Some(Action::Wall));
    assert!(custom.plain_menu(true).contains("[Z] Graffiti"));
    fs::remove_dir_all(root).unwrap();
}
