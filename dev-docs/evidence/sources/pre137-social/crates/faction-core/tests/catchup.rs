use faction_core::{
    Board, CatchupModule as M, CatchupTarget, Error, Session, Status, export_local, request_id,
    restore_local, verify_export,
};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-poll-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("faction-catchup-{}", request_id().unwrap()));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        Self(p)
    }
    fn path(&self) -> PathBuf {
        self.0.join("board")
    }
    fn db(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(self.path().join("faction.sqlite3")).unwrap()
    }
    fn board(&self) -> (Board, Session) {
        let mut b = Board::open(&self.path()).unwrap();
        b.bootstrap("sysop", PASS).unwrap();
        let s = b.authenticate("sysop", PASS).unwrap();
        (b, s)
    }
    fn enable(&self, b: &mut Board, s: &Session, members: bool) {
        let mut c = b.catchup_config(s).unwrap();
        c.enabled = true;
        let _ = members;
        b.configure_catchup(s, &c).unwrap();
    }
    fn member(&self, b: &mut Board, s: &Session, name: &str) -> Session {
        b.register(name, PASS).unwrap();
        b.set_status(s, name, Status::Approved).unwrap();
        b.authenticate(name, PASS).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn post(b: &mut Board, s: &Session, body: &str, reply: Option<i64>) -> i64 {
    b.post(s, 1, &request_id().unwrap(), body, reply)
        .unwrap()
        .id
}
fn file(f: &Fixture, area: i64, id: &str) {
    f.db()
        .execute(
            "INSERT INTO file_objects VALUES(?1,?2,1,?3,'Synthetic file',1,?4,'published')",
            rusqlite::params![id, area, format!("{}.txt", &id[..4]), "a".repeat(64)],
        )
        .unwrap();
}
#[test]
fn optional_policy_current_membership_and_no_automatic_activity_tracking() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    assert!(b.catchup_window(&s).is_err());
    f.enable(&mut b, &s, false);
    let w = b.catchup_window(&s).unwrap();
    assert!(w.last_saved.is_none());
    assert!(w.revision.is_empty());
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM catchup_members", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    let a = f.member(&mut b, &s, "alice");
    let c = b.catchup_config(&s).unwrap();
    assert!(b.configure_catchup(&a, &c).is_err());
    b.register("pending", PASS).unwrap();
    let pending = b.authenticate("pending", PASS).unwrap();
    assert!(b.catchup_window(&pending).is_err());
    let other = Fixture::new();
    let (mut ob, _) = other.board();
    assert!(ob.catchup_preferences(&s).is_err());
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(b.catchup_checkpoint(&a, &w).is_err());
}
#[test]
fn preview_counts_replies_and_checkpoint_never_acknowledge_posts() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let a = f.member(&mut b, &s, "alice");
    let parent = post(&mut b, &s, "A local question", None);
    post(&mut b, &a, "Direct reply", Some(parent));
    post(&mut b, &s, "Self reply", Some(parent));
    let w = b.catchup_window(&s).unwrap();
    let page = b.catchup_page(&s, M::Replies, 0, &w).unwrap();
    assert_eq!(page.items[0].summary, "1 direct replies in next 100 unread");
    assert_eq!(
        b.catchup_page(&s, M::Rooms, 0, &w).unwrap().items[0].summary,
        "3 unread"
    );
    assert_eq!(
        b.catchup_target(&s, M::Rooms, "1", &w).unwrap(),
        CatchupTarget::Room(1)
    );
    b.catchup_checkpoint(&s, &w).unwrap();
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM unread", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn long_absences_are_capped_without_a_second_post_ledger() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    for _ in 0..105 {
        post(&mut b, &s, "Synthetic unread", None);
    }
    let w = b.catchup_window(&s).unwrap();
    assert_eq!(
        b.catchup_page(&s, M::Rooms, 0, &w).unwrap().items[0].summary,
        "100+ unread"
    );
    for n in 0..8 {
        b.create_room(&s, &format!("room{n}"), "More rooms")
            .unwrap();
    }
    let p = b.catchup_page(&s, M::Rooms, 0, &w).unwrap();
    assert_eq!(p.items.len(), 4);
    let p2 = b.catchup_page(&s, M::Rooms, p.next.unwrap(), &w).unwrap();
    assert_eq!(p2.items[0].id, 5);
    assert!(b.catchup_page(&s, M::Rooms, -1, &w).is_err());
}
#[test]
fn favorites_are_member_owned_bounded_and_revision_checked() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let a = f.member(&mut b, &s, "alice");
    for n in 0..16 {
        b.create_room(&s, &format!("room{n}"), "Favorite candidate")
            .unwrap();
    }
    let first = b.catchup_window(&s).unwrap();
    b.catchup_subscribe(&s, &first.revision, "+2").unwrap();
    assert!(matches!(
        b.catchup_subscribe(&s, &first.revision, "+3"),
        Err(Error::RequestConflict)
    ));
    for id in 3..=17 {
        let w = b.catchup_window(&s).unwrap();
        b.catchup_subscribe(&s, &w.revision, &format!("+{id}"))
            .unwrap();
    }
    let w = b.catchup_window(&s).unwrap();
    assert!(b.catchup_subscribe(&s, &w.revision, "+1").is_err());
    assert_eq!(b.catchup_preferences(&a).unwrap().favorites.len(), 0);
    assert_eq!(b.catchup_page(&s, M::Rooms, 0, &w).unwrap().items[0].id, 2);
    b.catchup_subscribe(&s, &w.revision, "none").unwrap();
    let w = b.catchup_window(&s).unwrap();
    assert!(
        b.catchup_page(&s, M::Rooms, 0, &w)
            .unwrap()
            .items
            .is_empty()
    );
    b.catchup_subscribe(&s, &w.revision, "all").unwrap();
    assert_eq!(b.catchup_page(&s, M::Rooms, 0, &w).unwrap().items[0].id, 1);
}
#[test]
fn file_window_is_publication_order_and_cannot_skip_later_publications() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let area = b
        .file_create_area(&s, "library", "Library", false, 10000)
        .unwrap();
    file(&f, area, &"1".repeat(32));
    f.db().execute("INSERT INTO file_objects VALUES(?1,?2,1,'later.txt','Pending before visit',1,?3,'pending')",rusqlite::params!["2".repeat(32),area,"a".repeat(64)]).unwrap();
    let w = b.catchup_window(&s).unwrap();
    b.file_review(&s, &"2".repeat(32), true).unwrap();
    assert_eq!(b.catchup_page(&s, M::Files, 0, &w).unwrap().items.len(), 1);
    b.catchup_checkpoint(&s, &w).unwrap();
    let next = b.catchup_window(&s).unwrap();
    let p = b.catchup_page(&s, M::Files, 0, &next).unwrap();
    assert_eq!(p.items.len(), 1);
    assert_eq!(p.items[0].id, 2);
    assert_eq!(
        b.catchup_target(&s, M::Files, &"2".repeat(32), &next)
            .unwrap(),
        CatchupTarget::File("2".repeat(32))
    );
    assert!(matches!(
        b.catchup_checkpoint(&s, &w),
        Err(Error::RequestConflict)
    ));
    // Deleting the greatest publication must not lower the AUTOINCREMENT high water.
    f.db()
        .execute("DELETE FROM file_objects WHERE id=?1", ["2".repeat(32)])
        .unwrap();
    assert_eq!(b.catchup_window(&s).unwrap().through, 2);
    let mut invalid = next;
    invalid.through = 999;
    assert!(b.catchup_checkpoint(&s, &invalid).is_err());
}
#[test]
fn file_access_and_stale_targets_are_rechecked_without_private_metadata() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let a = f.member(&mut b, &s, "alice");
    let area = b
        .file_create_area(&s, "private", "Private library", true, 10000)
        .unwrap();
    file(&f, area, &"a".repeat(32));
    let w = b.catchup_window(&a).unwrap();
    assert!(
        b.catchup_page(&a, M::Files, 0, &w)
            .unwrap()
            .items
            .is_empty()
    );
    b.file_allow(&s, area, "alice", true).unwrap();
    assert_eq!(b.catchup_page(&a, M::Files, 0, &w).unwrap().items.len(), 1);
    b.file_allow(&s, area, "alice", false).unwrap();
    assert!(b.catchup_target(&a, M::Files, &"a".repeat(32), &w).is_err());
    assert!(
        b.catchup_page(&a, M::Files, 0, &w)
            .unwrap()
            .items
            .is_empty()
    );
    b.file_review(&s, &"a".repeat(32), false).unwrap();
    assert!(b.catchup_target(&s, M::Files, &"a".repeat(32), &w).is_err());
}
#[test]
fn modules_notice_and_gathering_visibility_are_current_and_local() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let a = f.member(&mut b, &s, "alice");
    let w = b.catchup_window(&s).unwrap();
    assert!(
        b.catchup_page(&s, M::Gatherings, 0, &w)
            .unwrap()
            .items
            .is_empty()
    );
    f.db()
        .execute("UPDATE gathering_settings SET enabled=1", [])
        .unwrap();
    f.db().execute("INSERT INTO gatherings(organizer,request_id,revision,title,body,starts_at,ends_at,place,cancelled,hidden,changed_at) VALUES(1,?1,?2,'Visible event','Local description',unixepoch()+3600,unixepoch()+7200,'room:1',0,0,unixepoch())",rusqlite::params!["1".repeat(32),"2".repeat(32)]).unwrap();
    assert_eq!(
        b.catchup_page(&a, M::Gatherings, 0, &w)
            .unwrap()
            .items
            .len(),
        1
    );
    assert_eq!(
        b.catchup_target(&a, M::Gatherings, "1", &w).unwrap(),
        CatchupTarget::Gathering(1)
    );
    f.db()
        .execute("UPDATE gatherings SET hidden=1", [])
        .unwrap();
    assert!(
        b.catchup_page(&a, M::Gatherings, 0, &w)
            .unwrap()
            .items
            .is_empty()
    );
    assert!(b.catchup_target(&a, M::Gatherings, "1", &w).is_err());
    let mut c = b.catchup_config(&s).unwrap();
    c.notice = "Local sysop announcement".into();
    c.modules = vec![M::Rooms];
    b.configure_catchup(&s, &c).unwrap();
    assert!(b.catchup_config(&a).unwrap().notice.is_empty());
    assert!(b.catchup_page(&s, M::Files, 0, &w).is_err());
    c.modules = vec![M::Files, M::Files];
    assert!(b.configure_catchup(&s, &c).is_err());
    assert!(M::parse("mail").is_err());
}
#[test]
fn busy_reads_stay_available_and_explicit_checkpoint_retries() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let w = b.catchup_window(&s).unwrap();
    let lock = f.db();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    assert!(b.catchup_page(&s, M::Rooms, 0, &w).is_ok());
    assert!(matches!(b.catchup_checkpoint(&s, &w), Err(Error::Busy)));
    lock.execute_batch("ROLLBACK").unwrap();
    b.catchup_checkpoint(&s, &w).unwrap();
    assert!(b.catchup_window(&s).unwrap().last_saved.is_some());
}
#[test]
fn snapshot_publication_baseline_and_exact_schema_twelve_recovery() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let area = b
        .file_create_area(&s, "library", "Library", false, 10000)
        .unwrap();
    file(&f, area, &"1".repeat(32));
    let w = b.catchup_window(&s).unwrap();
    b.catchup_checkpoint(&s, &w).unwrap();
    let w = b.catchup_window(&s).unwrap();
    b.catchup_subscribe(&s, &w.revision, "+1").unwrap();
    drop(b);
    let out = f.0.join("export");
    export_local(&f.path(), &out).unwrap();
    verify_export(&out).unwrap();
    let restored = f.0.join("restored");
    restore_local(&out, &restored).unwrap();
    let mut b = Board::open(&restored).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert_eq!(b.catchup_window(&s).unwrap().after, 1);
    assert_eq!(b.catchup_preferences(&s).unwrap().favorites, vec![1]);
    drop(b);
    f.db().execute_batch("DROP TABLE magazine_entries;DROP TABLE magazine_issues;DROP TABLE magazine_submissions;DROP TABLE file_requests;DROP INDEX discussion_author_posts;DROP TABLE discussion_bookmarks;DROP TABLE discussion_settings;DROP TRIGGER catchup_file_insert;DROP TRIGGER catchup_file_publish;DROP TABLE catchup_favorites;DROP TABLE catchup_members;DROP TABLE catchup_settings;DROP TABLE catchup_publications;PRAGMA user_version=12").unwrap();
    let old = f.0.join("old-export");
    export_local(&f.path(), &old).unwrap();
    verify_export(&old).unwrap();
    assert_eq!(
        f.db()
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        12
    );
    let restored = f.0.join("old-restored");
    restore_local(&old, &restored).unwrap();
    let mut b = Board::open(&restored).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert!(!b.catchup_config(&s).unwrap().enabled);
    assert_eq!(
        rusqlite::Connection::open(restored.join("faction.sqlite3"))
            .unwrap()
            .query_row("SELECT published_at FROM catchup_publications", [], |r| r
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        0
    );
}
