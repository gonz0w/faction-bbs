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

#[test]
fn notification_consent_is_default_off_visit_only_bounded_and_not_availability() {
    use faction_pages::notifications::{INTERVAL, Notifications};
    use std::time::Instant;
    let f = Fixture::new();
    let mut b = Board::open(&f.0.join("board")).unwrap();
    b.bootstrap("sysop", PASS).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    let l = b.chat_open(&s).unwrap();
    b.social_configure(
        &l,
        &SocialConfig {
            enabled: true,
            name: "Owner lounge".into(),
        },
    )
    .unwrap();
    let mut n = Notifications::default();
    assert!(!n.enabled());
    assert_eq!(n.poll(&mut b, &l, Instant::now() + INTERVAL).unwrap(), None);
    for command in [
        "/notify bad",
        "/notify on extra",
        "/notify on\n",
        "/notify on\u{1b}",
    ] {
        assert!(n.command(&mut b, &l, command).is_err());
    }
    let mut desk = Desk::default();
    desk.feed_with_notifications("/notify on", &mut b, &l, &mut n)
        .unwrap();
    assert!(n.enabled());
    let out = desk
        .render_with_notifications(&mut b, &l, &Theme::default(), false, &n)
        .unwrap();
    assert!(
        out.contains("Idle page notifications: on") && out.contains("Your advertisement: hidden")
    );
    assert_eq!(n.poll(&mut b, &l, Instant::now() + INTERVAL).unwrap(), None);
    n.command(&mut b, &l, "/notify off").unwrap();
    assert!(!n.enabled());
    assert!(!Notifications::default().enabled());
}
#[test]
fn notification_poll_coalesces_current_pending_pages_and_never_resurrects_muted_signals() {
    use faction_pages::notifications::{INTERVAL, Notifications};
    use std::time::Instant;
    let f = Fixture::new();
    let mut b = Board::open(&f.0.join("board")).unwrap();
    b.bootstrap("sysop", PASS).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    let op = b.chat_open(&s).unwrap();
    b.register("alice", PASS).unwrap();
    b.set_status(&s, "alice", Status::Approved).unwrap();
    let a = b.authenticate("alice", PASS).unwrap();
    let l = b.chat_open(&a).unwrap();
    b.social_configure(
        &op,
        &SocialConfig {
            enabled: true,
            name: "Owner lounge".into(),
        },
    )
    .unwrap();
    b.social_availability(&op, "available", "").unwrap();
    b.social_availability(&l, "available", "").unwrap();
    let mut n = Notifications::default();
    n.command(&mut b, &l, "/notify on").unwrap();
    b.social_request(&op, "alice", &request_id().unwrap(), false)
        .unwrap();
    let now = Instant::now() + INTERVAL;
    assert_eq!(n.poll(&mut b, &l, now).unwrap(), Some(true));
    assert_eq!(n.poll(&mut b, &l, now).unwrap(), None);
    assert_eq!(n.poll(&mut b, &l, now + INTERVAL).unwrap(), None);
    n.command(&mut b, &l, "/notify on").unwrap();
    assert_eq!(n.poll(&mut b, &l, now + INTERVAL * 2).unwrap(), None);
    b.social_preferences(&l, true, None).unwrap();
    assert_eq!(n.poll(&mut b, &l, now + INTERVAL * 3).unwrap(), Some(false));
    b.social_preferences(&l, false, None).unwrap();
    assert_eq!(n.poll(&mut b, &l, now + INTERVAL * 4).unwrap(), None);
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(n.command(&mut b, &l, "/notify off").is_err());
    assert!(n.poll(&mut b, &l, now + INTERVAL * 5).is_err());
}
#[test]
fn notification_poll_announces_only_explicit_outgoing_answers_and_rechecks_disable() {
    use faction_pages::notifications::{INTERVAL, Notifications};
    use std::time::Instant;
    let f = Fixture::new();
    let mut b = Board::open(&f.0.join("board")).unwrap();
    b.bootstrap("sysop", PASS).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    let op = b.chat_open(&s).unwrap();
    b.register("alice", PASS).unwrap();
    b.set_status(&s, "alice", Status::Approved).unwrap();
    let a = b.authenticate("alice", PASS).unwrap();
    let l = b.chat_open(&a).unwrap();
    b.social_configure(
        &op,
        &SocialConfig {
            enabled: true,
            name: "Owner lounge".into(),
        },
    )
    .unwrap();
    b.social_availability(&op, "available", "").unwrap();
    b.social_availability(&l, "available", "").unwrap();
    let mut n = Notifications::default();
    n.command(&mut b, &l, "/notify on").unwrap();
    b.social_request(&l, "sysop", &request_id().unwrap(), true)
        .unwrap();
    let now = Instant::now() + INTERVAL;
    assert_eq!(n.poll(&mut b, &l, now).unwrap(), None);
    let id = b.social_pages(&op).unwrap()[0].id;
    b.social_answer(&op, id, true).unwrap();
    assert_eq!(n.poll(&mut b, &l, now + INTERVAL).unwrap(), Some(true));
    assert_eq!(n.poll(&mut b, &l, now + INTERVAL * 2).unwrap(), None);
    b.social_configure(
        &op,
        &SocialConfig {
            enabled: false,
            name: "Owner lounge".into(),
        },
    )
    .unwrap();
    assert_eq!(n.poll(&mut b, &l, now + INTERVAL * 3).unwrap(), Some(false));
    assert_eq!(n.poll(&mut b, &l, now + INTERVAL * 4).unwrap(), None);
}
