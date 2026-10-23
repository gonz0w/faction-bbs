use faction_core::{Board, SocialConfig, Status, request_id};
use faction_pages::social::{Desk, menu_theme, visit_command};
use faction_terminal::Theme;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("faction-social-desk-{}", request_id().unwrap()));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        Self(p)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
const PASS: &str = "synthetic-social-desk-password";
#[test]
fn board_lounge_refetches_roster_and_signals_instead_of_replaying_cached_command_output() {
    let f = Fixture::new();
    let mut b = Board::open(&f.0.join("board")).unwrap();
    b.bootstrap("sysop", PASS).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    let op = b.chat_open(&s).unwrap();
    b.register("alice", PASS).unwrap();
    b.set_status(&s, "alice", Status::Approved).unwrap();
    let a = b.authenticate("alice", PASS).unwrap();
    let lease = b.chat_open(&a).unwrap();
    b.social_configure(
        &op,
        &SocialConfig {
            enabled: true,
            name: "Original lounge".into(),
        },
    )
    .unwrap();
    b.social_availability(&op, "available", "Editor").unwrap();
    b.social_availability(&lease, "available", "Public status")
        .unwrap();
    let mut desk = Desk::default();
    desk.feed("/who", &mut b, &op).unwrap();
    let frame = desk.render(&mut b, &op, &Theme::default(), false).unwrap();
    assert!(frame.contains("alice [available]") && !frame.contains('\u{1b}'));
    b.social_block(&lease, "sysop", true).unwrap();
    let frame = desk.render(&mut b, &op, &Theme::default(), false).unwrap();
    assert!(!frame.contains("alice"));
    b.social_block(&lease, "sysop", false).unwrap();
    b.social_request(&lease, "sysop", &request_id().unwrap(), false)
        .unwrap();
    desk.feed("/pages", &mut b, &op).unwrap();
    assert!(
        desk.render(&mut b, &op, &Theme::default(), false)
            .unwrap()
            .contains("from alice: pending")
    );
    b.social_availability(&lease, "hidden", "").unwrap();
    let frame = desk.render(&mut b, &op, &Theme::default(), false).unwrap();
    assert!(!frame.contains("from alice") && frame.len() < 8192);
}
#[test]
fn shared_visit_panel_is_opt_in_and_scoped_help_does_not_change_legacy_chat_lifetime() {
    let f = Fixture::new();
    let mut b = Board::open(&f.0.join("board")).unwrap();
    b.bootstrap("sysop", PASS).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    let l = b.chat_open(&s).unwrap();
    assert!(
        menu_theme(&mut b, &s, None, &Theme::default())
            .unwrap()
            .is_none()
    );
    b.social_configure(
        &l,
        &SocialConfig {
            enabled: true,
            name: "Original lounge".into(),
        },
    )
    .unwrap();
    let theme = menu_theme(&mut b, &s, Some(&l), &Theme::default())
        .unwrap()
        .unwrap();
    assert!(theme.plain_menu(true).contains("[L] Social lounge"));
    let mut d = Desk::default();
    assert!(
        d.render(&mut b, &l, &theme, false)
            .unwrap()
            .contains("Your advertisement: hidden")
    );
    d.feed("/available Original public status", &mut b, &l)
        .unwrap();
    b.chat_leave_all(&l).unwrap();
    assert!(
        d.render(&mut b, &l, &theme, false)
            .unwrap()
            .contains("available / Original public status")
    );
    assert!(
        visit_command(&mut b, &l, "/social help")
            .unwrap()
            .unwrap()
            .contains("during this BBS visit")
    );
    assert!(
        faction_pages::social::command(&mut b, &l, "/social help")
            .unwrap()
            .unwrap()
            .contains("during this chat entry")
    );
    assert!(d.feed("q", &mut b, &l).unwrap());
    assert!(b.social_advertisement(&l).unwrap().is_some());
}
