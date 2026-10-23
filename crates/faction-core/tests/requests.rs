use faction_core::{
    Board, Error, FileRequestDraft, FileRequestOffer, Session, Status, export_local, request_id,
    restore_local, verify_export,
};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-poll-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p =
            std::env::temp_dir().join(format!("faction-file-requests-{}", request_id().unwrap()));
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

fn draft() -> FileRequestDraft {
    FileRequestDraft {
        title: "Need a Rust guide".into(),
        description: "An original permitted NetBSD tutorial, please.".into(),
    }
}
fn offer(id: u8) -> FileRequestOffer {
    FileRequestOffer {
        object: format!("{id:032x}"),
        rights: "BSD-3-Clause original fixture".into(),
        provenance: "Written by the fixture author".into(),
    }
}
fn files(f: &Fixture, b: &mut Board, s: &Session) {
    b.file_create_area(s, "public", "Public files", false, 10000)
        .unwrap();
    b.file_create_area(s, "private", "Private files", true, 10000)
        .unwrap();
    for (id, area, state) in [
        (1, 1, "published"),
        (2, 2, "published"),
        (3, 1, "pending"),
        (4, 1, "removed"),
    ] {
        f.db()
            .execute(
                "INSERT INTO file_objects VALUES(?1,?2,1,'guide.txt','Original guide',1,?3,?4)",
                rusqlite::params![format!("{id:032x}"), area, "a".repeat(64), state],
            )
            .unwrap();
    }
}
#[test]
fn create_retry_conflicts_bounds_pages_close_delete_and_current_roles() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    let a = f.member(&mut b, &s, "alice");
    let other = f.member(&mut b, &s, "bob");
    let nonce = request_id().unwrap();
    let id = b.create_file_request(&a, &nonce, &draft()).unwrap();
    assert_eq!(b.create_file_request(&a, &nonce, &draft()).unwrap(), id);
    let mut changed = draft();
    changed.title = "different".into();
    assert!(b.create_file_request(&a, &nonce, &changed).is_err());
    for _ in 0..7 {
        b.create_file_request(&a, &request_id().unwrap(), &draft())
            .unwrap();
    }
    assert!(
        b.create_file_request(&a, &request_id().unwrap(), &draft())
            .is_err()
    );
    let page = b.file_requests(&a, i64::MAX).unwrap();
    assert_eq!(page.len(), 4);
    assert_eq!(b.file_requests(&a, page[3].id - 1).unwrap().len(), 4);
    let r = b.file_request(&a, id).unwrap();
    assert!(
        b.finish_file_request(&other, id, &r.revision, false)
            .is_err()
    );
    b.finish_file_request(&a, id, &r.revision, false).unwrap();
    assert!(b.finish_file_request(&a, id, &r.revision, true).is_err());
    b.create_file_request(&a, &request_id().unwrap(), &draft())
        .unwrap();
    let r = b.file_request(&s, id).unwrap();
    b.finish_file_request(&s, id, &r.revision, true).unwrap();
    assert!(b.file_request(&a, id).is_err());
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(b.file_requests(&a, i64::MAX).is_err());
    assert!(b.file_requests(&other, i64::MAX).unwrap().is_empty());
    assert!(!b.file_requests(&s, i64::MAX).unwrap().is_empty());
}
#[test]
fn fulfillment_requires_both_members_current_catalog_access_and_filters_every_view() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    let a = f.member(&mut b, &s, "alice");
    let bob = f.member(&mut b, &s, "bob");
    files(&f, &mut b, &s);
    let id = b
        .create_file_request(&a, &request_id().unwrap(), &draft())
        .unwrap();
    let r = b.file_request(&a, id).unwrap();
    for i in [2, 3, 4, 5] {
        assert!(
            b.fulfill_file_request(&s, id, &r.revision, &offer(i))
                .is_err()
        );
    }
    b.file_allow(&s, 2, "alice", true).unwrap();
    b.fulfill_file_request(&s, id, &r.revision, &offer(2))
        .unwrap();
    assert!(b.file_request(&bob, id).unwrap().offer.is_none());
    assert!(b.file_request(&a, id).unwrap().offer.is_some());
    b.file_allow(&s, 2, "alice", false).unwrap();
    let r = b.file_request(&a, id).unwrap();
    assert_eq!(r.state, "fulfilled");
    assert!(r.offer.is_none());
    b.file_allow(&s, 2, "alice", true).unwrap();
    f.db()
        .execute(
            "UPDATE file_objects SET state='removed' WHERE id=?1",
            [format!("{:032x}", 2)],
        )
        .unwrap();
    assert!(b.file_request(&s, id).unwrap().offer.is_none());
    f.db()
        .execute(
            "DELETE FROM file_objects WHERE id=?1",
            [format!("{:032x}", 2)],
        )
        .unwrap();
    assert!(b.file_request(&a, id).unwrap().offer.is_none());
    let id = b
        .create_file_request(&a, &request_id().unwrap(), &draft())
        .unwrap();
    let r = b.file_request(&bob, id).unwrap();
    b.fulfill_file_request(&bob, id, &r.revision, &offer(1))
        .unwrap();
    assert!(b.file_request(&a, id).unwrap().offer.is_some());
    assert_eq!(b.unread_cursor(&a, 1).unwrap(), 0);
}
#[test]
fn stale_reviews_busy_retry_revoked_requester_foreign_sessions_and_inert_text() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    let a = f.member(&mut b, &s, "alice");
    files(&f, &mut b, &s);
    let id = b
        .create_file_request(&a, &request_id().unwrap(), &draft())
        .unwrap();
    let r = b.file_request(&a, id).unwrap();
    let lock = f.db();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    assert!(b.file_request(&a, id).is_ok());
    assert!(matches!(
        b.fulfill_file_request(&a, id, &r.revision, &offer(1)),
        Err(Error::Busy)
    ));
    lock.execute_batch("ROLLBACK").unwrap();
    b.finish_file_request(&a, id, &r.revision, false).unwrap();
    assert!(
        b.fulfill_file_request(&s, id, &r.revision, &offer(1))
            .is_err()
    );
    let id = b
        .create_file_request(&a, &request_id().unwrap(), &draft())
        .unwrap();
    let r = b.file_request(&a, id).unwrap();
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(
        b.fulfill_file_request(&s, id, &r.revision, &offer(1))
            .is_err()
    );
    let f2 = Fixture::new();
    let (mut b2, _) = f2.board();
    assert!(b2.file_requests(&s, i64::MAX).is_err());
    for text in ["", "\x1b[2J", "bad\ntext", "faction-file:secret"] {
        let mut d = draft();
        d.title = text.into();
        assert!(d.validate().is_err());
    }
    let mut d = draft();
    d.description = "a".repeat(129);
    assert!(d.validate().is_err());
    assert!(b.file_requests(&s, -1).is_err());
}
#[test]
fn snapshot_roundtrip_preserves_requests_and_exact_schema_fourteen_stays_readonly() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    files(&f, &mut b, &s);
    let id = b
        .create_file_request(&s, &request_id().unwrap(), &draft())
        .unwrap();
    let r = b.file_request(&s, id).unwrap();
    b.fulfill_file_request(&s, id, &r.revision, &offer(1))
        .unwrap();
    drop(b);
    let out = f.0.join("export");
    export_local(&f.path(), &out).unwrap();
    verify_export(&out).unwrap();
    let dest = f.0.join("restored");
    restore_local(&out, &dest).unwrap();
    let mut b = Board::open(&dest).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert!(b.file_request(&s, id).unwrap().offer.is_some());
    drop(b);
    f.db()
        .execute_batch("DROP TABLE social_pages;DROP TABLE social_presence;DROP TABLE social_blocks;DROP TABLE social_prefs;DROP TABLE social_cooldowns;DROP TABLE social_settings;DROP TABLE magazine_entries;DROP TABLE magazine_issues;DROP TABLE magazine_submissions;DROP TABLE file_requests;PRAGMA user_version=14")
        .unwrap();
    let out = f.0.join("old");
    export_local(&f.path(), &out).unwrap();
    verify_export(&out).unwrap();
    assert_eq!(
        f.db()
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        14
    );
    let dest = f.0.join("old-restored");
    restore_local(&out, &dest).unwrap();
    let mut b = Board::open(&dest).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert!(b.file_requests(&s, i64::MAX).unwrap().is_empty());
}
#[test]
fn snapshot_rejects_request_text_quotas_and_malformed_keys() {
    for sql in [
        "UPDATE file_requests SET title=char(27)||'[2J'",
        "UPDATE file_requests SET revision='ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ'",
        "UPDATE file_requests SET description='faction-file:secret'",
    ] {
        let f = Fixture::new();
        let (mut b, s) = f.board();
        b.create_file_request(&s, &request_id().unwrap(), &draft())
            .unwrap();
        drop(b);
        f.db().execute_batch(sql).unwrap();
        assert!(export_local(&f.path(), &f.0.join("export")).is_err());
    }
    let f = Fixture::new();
    let (mut b, s) = f.board();
    let db = f.db();
    for i in 0..256 {
        db.execute("INSERT INTO file_requests(creator,nonce,revision,title,description,state,rights,provenance) VALUES(1,?1,?1,'bounded','bounded','closed','','')",[format!("{i:032x}")]).unwrap();
    }
    assert!(
        b.create_file_request(&s, &request_id().unwrap(), &draft())
            .is_err()
    );
}
