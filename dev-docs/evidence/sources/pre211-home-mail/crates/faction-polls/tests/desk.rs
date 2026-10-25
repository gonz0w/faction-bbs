use faction_core::{Board, Error, Session, request_id};
use faction_polls::Desk;
use faction_terminal::Theme;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-poll-desk-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> (Self, Board, Session) {
        let p = std::env::temp_dir().join(format!("faction-poll-desk-{}", request_id().unwrap()));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        let mut b = Board::open(&p.join("board")).unwrap();
        b.bootstrap("sysop", PASS).unwrap();
        let s = b.authenticate("sysop", PASS).unwrap();
        (Self(p), b, s)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn render(d: &mut Desk, b: &mut Board, s: &Session) -> String {
    d.render(b, s, &Theme::default(), false, 40).unwrap()
}
fn feed(d: &mut Desk, v: &str, b: &mut Board, s: &Session) {
    assert!(!d.feed(v, b, s).unwrap());
}
fn form(d: &mut Desk, b: &mut Board, s: &Session) {
    for v in [
        "c",
        "Which night?",
        "Friday",
        "Saturday",
        "Sunday",
        ".",
        "v",
        "Local notes",
        ".",
    ] {
        feed(d, v, b, s);
    }
}
#[test]
fn preview_cancel_and_busy_save_keep_explicit_intent_and_plain_inert_text() {
    let (f, mut b, s) = Fixture::new();
    let mut settings = Desk::settings();
    feed(&mut settings, "e", &mut b, &s);
    let mut d = Desk::default();
    form(&mut d, &mut b, &s);
    assert!(render(&mut d, &mut b, &s).contains("PREVIEW ONLY"));
    assert!(b.polls(&s, false, 0).unwrap().is_empty());
    feed(&mut d, "d", &mut b, &s);
    assert!(render(&mut d, &mut b, &s).contains("Local notes"));
    feed(&mut d, "q", &mut b, &s);
    let lock = rusqlite_connection(&f);
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    let error = d.feed("s", &mut b, &s).unwrap_err();
    assert!(matches!(error, Error::Busy));
    assert!(d.recover(&error));
    assert!(render(&mut d, &mut b, &s).contains("PREVIEW ONLY"));
    lock.execute_batch("ROLLBACK").unwrap();
    feed(&mut d, "s", &mut b, &s);
    assert_eq!(b.polls(&s, false, 0).unwrap().len(), 1);
    assert!(!render(&mut d, &mut b, &s).contains('\x1b'));
    d.render(&mut b, &s, &Theme::default(), false, 80).unwrap();
    for _ in 0..2 {
        feed(&mut d, "n", &mut b, &s);
        assert!(
            d.render(&mut b, &s, &Theme::default(), false, 80)
                .unwrap()
                .contains("3: Sunday")
        );
    }
    feed(&mut d, "p", &mut b, &s);
    assert!(
        d.render(&mut b, &s, &Theme::default(), false, 80)
            .unwrap()
            .contains("1: Friday")
    );
    assert!(d.feed("\x1b[2J", &mut b, &s).is_err());
}
#[test]
fn stale_choice_review_does_not_index_changed_choices_or_save_a_ballot() {
    let (_f, mut b, s) = Fixture::new();
    let mut c = b.poll_config(&s).unwrap();
    c.enabled = true;
    b.configure_polls(&s, &c).unwrap();
    let mut d = Desk::default();
    form(&mut d, &mut b, &s);
    feed(&mut d, "s", &mut b, &s);
    feed(&mut d, "v", &mut b, &s);
    feed(&mut d, "3", &mut b, &s);
    let p = b.poll(&s, 1).unwrap();
    let mut changed = p.draft;
    changed.choices.pop();
    b.save_poll(&s, Some(1), &p.revision, &changed).unwrap();
    let error = d
        .render(&mut b, &s, &Theme::default(), false, 40)
        .unwrap_err();
    assert!(matches!(error, Error::RequestConflict));
    assert!(d.recover(&error));
    assert!(render(&mut d, &mut b, &s).contains("Poll changed"));
    assert_eq!(b.poll(&s, 1).unwrap().own_vote, 0);
    feed(&mut d, "q", &mut b, &s);
    feed(&mut d, "c", &mut b, &s);
    feed(&mut d, "Held draft", &mut b, &s);
    c.enabled = false;
    b.configure_polls(&s, &c).unwrap();
    assert!(!render(&mut d, &mut b, &s).contains("Held draft"));
    assert!(d.feed("q", &mut b, &s).unwrap());
    assert!(!d.recover(&Error::Denied));
}
fn rusqlite_connection(f: &Fixture) -> rusqlite::Connection {
    rusqlite::Connection::open(f.0.join("board/faction.sqlite3")).unwrap()
}
