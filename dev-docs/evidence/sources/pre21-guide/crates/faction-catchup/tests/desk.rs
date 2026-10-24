use faction_catchup::{Desk, Outcome};
use faction_core::{Board, Error, Session, request_id};
use faction_terminal::Theme;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-poll-desk-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> (Self, Board, Session) {
        let p =
            std::env::temp_dir().join(format!("faction-catchup-desk-{}", request_id().unwrap()));
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
    assert!(matches!(d.feed(v, b, s).unwrap(), Outcome::Stay));
}
#[test]
fn explicit_checkpoint_cancel_busy_retry_and_concurrent_revision_leave_unread_alone() {
    let (f, mut b, s) = Fixture::new();
    let mut settings = Desk::settings();
    feed(&mut settings, "e", &mut b, &s);
    let mut d = Desk::default();
    assert!(render(&mut d, &mut b, &s).contains("not set"));
    let before = b.catchup_window(&s).unwrap();
    feed(&mut d, "c", &mut b, &s);
    feed(&mut d, "c", &mut b, &s);
    assert_eq!(b.catchup_window(&s).unwrap(), before);
    feed(&mut d, "c", &mut b, &s);
    let lock = rusqlite::Connection::open(f.0.join("board/faction.sqlite3")).unwrap();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    let e = d.feed("s", &mut b, &s).err().unwrap();
    assert!(matches!(e, Error::Busy));
    assert!(d.recover(&e));
    assert!(render(&mut d, &mut b, &s).contains("Visit review>"));
    lock.execute_batch("ROLLBACK").unwrap();
    feed(&mut d, "s", &mut b, &s);
    assert!(b.catchup_window(&s).unwrap().last_saved.is_some());
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
    render(&mut d, &mut b, &s);
    let other = b.catchup_window(&s).unwrap();
    b.catchup_subscribe(&s, &other.revision, "+1").unwrap();
    feed(&mut d, "c", &mut b, &s);
    let e = d.feed("s", &mut b, &s).err().unwrap();
    assert!(matches!(e, Error::RequestConflict));
    assert!(d.recover(&e));
    assert!(render(&mut d, &mut b, &s).contains("Visit changed elsewhere"));
    assert_eq!(b.catchup_preferences(&s).unwrap().favorites, vec![1]);
}
#[test]
fn plain_optional_modules_safe_ids_and_disabled_held_content() {
    let (_f, mut b, s) = Fixture::new();
    let mut c = b.catchup_config(&s).unwrap();
    c.enabled = true;
    c.notice = "Meet in the lobby".into();
    b.configure_catchup(&s, &c).unwrap();
    let mut d = Desk::default();
    let frame = render(&mut d, &mut b, &s);
    assert!(frame.contains("Meet in the lobby"));
    assert!(!frame.contains('\x1b'));
    feed(&mut d, "m files", &mut b, &s);
    assert!(render(&mut d, &mut b, &s).contains("Files unavailable"));
    assert!(d.feed("j", &mut b, &s).is_err());
    feed(&mut d, "m rooms", &mut b, &s);
    render(&mut d, &mut b, &s);
    feed(&mut d, "j", &mut b, &s);
    assert!(matches!(
        d.feed("1", &mut b, &s).unwrap(),
        Outcome::Open(faction_core::CatchupTarget::Room(1))
    ));
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
    c.enabled = false;
    b.configure_catchup(&s, &c).unwrap();
    let frame = render(&mut d, &mut b, &s);
    assert!(!frame.contains("Meet in the lobby"));
    assert!(matches!(d.feed("q", &mut b, &s).unwrap(), Outcome::Main));
    assert!(!d.recover(&Error::Denied));
}
