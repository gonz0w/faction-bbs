use faction_core::{Board, Error, Session, request_id};
use faction_find::Desk;
use faction_terminal::Theme;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-poll-desk-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> (Self, Board, Session) {
        let p = std::env::temp_dir().join(format!("faction-find-desk-{}", request_id().unwrap()));
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
    d.render(b, s, &Theme::default(), false, (40, 18), false)
        .unwrap()
}
fn feed(d: &mut Desk, v: &str, b: &mut Board, s: &Session) {
    assert!(!d.feed(v, b, s).unwrap());
}
#[test]
fn preview_cancel_busy_retry_and_export_leave_the_read_ledger_untouched() {
    let (f, mut b, s) = Fixture::new();
    let mut settings = Desk::settings();
    feed(&mut settings, "e", &mut b, &s);
    b.post(&s, 1, &request_id().unwrap(), "Useful ANSI guide", None)
        .unwrap();
    let mut d = Desk::default();
    render(&mut d, &mut b, &s);
    for v in ["f", "Useful", "", "", "b"] {
        feed(&mut d, v, &mut b, &s);
    }
    assert!(render(&mut d, &mut b, &s).contains("Useful ANSI guide"));
    for v in ["a", "1", "c"] {
        feed(&mut d, v, &mut b, &s);
    }
    assert!(b.discussion_bookmarks(&s, 99).unwrap().hits.is_empty());
    for v in ["a", "1"] {
        feed(&mut d, v, &mut b, &s);
    }
    let lock = rusqlite::Connection::open(f.0.join("board/faction.sqlite3")).unwrap();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    let e = d.feed("s", &mut b, &s).unwrap_err();
    assert!(matches!(e, Error::Busy));
    assert!(d.recover(&e));
    assert!(render(&mut d, &mut b, &s).contains("Bookmark review>"));
    lock.execute_batch("ROLLBACK").unwrap();
    feed(&mut d, "s", &mut b, &s);
    feed(&mut d, "x", &mut b, &s);
    assert!(render(&mut d, &mut b, &s).contains("FACTION-BOOKMARKS/1"));
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
}
#[test]
fn exact_source_view_plain_paging_disable_and_deleted_room_recovery() {
    let (f, mut b, s) = Fixture::new();
    let mut c = b.discussion_config(&s).unwrap();
    c.enabled = true;
    b.configure_discussion(&s, &c).unwrap();
    let room = b.create_room(&s, "archive", "Archive").unwrap();
    let body = (0..100).map(|n| format!("word{n:03} ")).collect::<String>() + "END-OF-SOURCE";
    b.post(&s, room, &request_id().unwrap(), &body, None)
        .unwrap();
    let mut d = Desk::default();
    render(&mut d, &mut b, &s);
    for v in ["f", "word", &room.to_string(), "", "b"] {
        feed(&mut d, v, &mut b, &s);
    }
    render(&mut d, &mut b, &s);
    for v in ["j", "1"] {
        feed(&mut d, v, &mut b, &s);
    }
    let mut complete = false;
    for _ in 0..30 {
        let frame = render(&mut d, &mut b, &s);
        assert!(!frame.contains('\x1b'));
        if frame.contains("END-OF-SOURCE") {
            complete = true;
            break;
        }
        feed(&mut d, "n", &mut b, &s);
    }
    assert!(complete);
    assert_eq!(b.unread_cursor(&s, room).unwrap(), 0);
    feed(&mut d, "q", &mut b, &s);
    let db = rusqlite::Connection::open(f.0.join("board/faction.sqlite3")).unwrap();
    db.execute_batch("PRAGMA foreign_keys=ON;DELETE FROM post_metadata;DELETE FROM posts;DELETE FROM rooms WHERE id=2").unwrap();
    let e = d
        .render(&mut b, &s, &Theme::default(), false, (40, 18), false)
        .unwrap_err();
    assert!(d.recover(&e));
    assert!(render(&mut d, &mut b, &s).contains("F starts"));
    c.enabled = false;
    b.configure_discussion(&s, &c).unwrap();
    assert!(render(&mut d, &mut b, &s).contains("Find is disabled"));
    assert!(d.feed("q", &mut b, &s).unwrap());
    assert!(!d.recover(&Error::Denied));
}
