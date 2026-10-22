use faction_core::{Board, Error, Status, export_local, request_id, restore_local, verify_export};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("faction-meta-{}", request_id().unwrap()));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        Self { root }
    }
    fn path(&self) -> PathBuf {
        self.root.join("board")
    }
    fn board(&self) -> Board {
        let mut b = Board::open(&self.path()).unwrap();
        b.bootstrap("sysop", "fixture-secret-long-enough").unwrap();
        b
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
const SECRET: &str = "fixture-secret-long-enough";
#[test]
fn new_metadata_is_atomic_stable_on_retry_and_survives_reopen() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("sysop", SECRET).unwrap();
    let key = request_id().unwrap();
    let post = b.post(&s, 1, &key, "new post", None).unwrap();
    let m = b.post_metadata(&s, post.id).unwrap();
    assert!(!m.legacy);
    assert!(m.created_at.unwrap() > 0);
    assert!(m.created_utc.as_ref().unwrap().ends_with(" UTC"));
    assert_eq!(m.subject, "Message #1");
    assert!(m.article_id.starts_with('<') && m.article_id.ends_with("@faction.invalid>"));
    assert_eq!(b.post(&s, 1, &key, "new post", None).unwrap().id, post.id);
    assert_eq!(b.post_metadata(&s, post.id).unwrap(), m);
    drop(b);
    let mut b = Board::open(&f.path()).unwrap();
    let s = b.authenticate("sysop", SECRET).unwrap();
    assert_eq!(b.post_metadata(&s, post.id).unwrap(), m);
    let db = rusqlite::Connection::open(f.path().join("faction.sqlite3")).unwrap();
    db.execute_batch("CREATE TRIGGER reject_metadata BEFORE INSERT ON post_metadata BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
    assert!(
        b.post(&s, 1, &request_id().unwrap(), "must roll back", None)
            .is_err()
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM posts", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}
#[test]
fn schema_five_migration_preserves_posts_replies_unread_and_unknown_dates() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("sysop", SECRET).unwrap();
    let p = b
        .post(&s, 1, &request_id().unwrap(), "legacy body", None)
        .unwrap();
    let reply = b
        .post(&s, 1, &request_id().unwrap(), "legacy reply", Some(p.id))
        .unwrap();
    b.mark_read(&s, 1, p.id).unwrap();
    drop(b);
    let db = rusqlite::Connection::open(f.path().join("faction.sqlite3")).unwrap();
    db.execute_batch("DROP TABLE post_metadata; PRAGMA user_version=5;")
        .unwrap();
    drop(db);
    let mut b = Board::open(&f.path()).unwrap();
    let s = b.authenticate("sysop", SECRET).unwrap();
    let m = b.post_metadata(&s, p.id).unwrap();
    assert!(m.legacy);
    assert_eq!(m.created_at, None);
    assert_eq!(m.created_utc, None);
    assert_eq!(m.subject, "Legacy message #1");
    assert_eq!(b.posts(&s, 1, 0).unwrap()[1].reply_to, Some(p.id));
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), p.id);
    assert_ne!(
        m.article_id,
        b.post_metadata(&s, reply.id).unwrap().article_id
    );
    let before = m.clone();
    drop(b);
    let mut b = Board::open(&f.path()).unwrap();
    let s = b.authenticate("sysop", SECRET).unwrap();
    assert_eq!(b.post_metadata(&s, p.id).unwrap(), before);
}
#[test]
fn old_exports_are_read_only_and_new_snapshots_preserve_public_ids() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("sysop", SECRET).unwrap();
    let p = b.post(&s, 1, &request_id().unwrap(), "keep", None).unwrap();
    let m = b.post_metadata(&s, p.id).unwrap();
    let snapshot = f.root.join("current.sqlite3");
    b.export_local(&snapshot).unwrap();
    verify_export(&snapshot).unwrap();
    let dest = f.root.join("restored");
    restore_local(&snapshot, &dest).unwrap();
    let mut restored = Board::open(&dest).unwrap();
    let rs = restored.authenticate("sysop", SECRET).unwrap();
    assert_eq!(restored.post_metadata(&rs, p.id).unwrap(), m);
    assert!(matches!(
        restored.post_metadata(&s, p.id),
        Err(Error::Denied)
    ));
    let old = f.root.join("old.sqlite3");
    b.export_local(&old).unwrap();
    let db = rusqlite::Connection::open(&old).unwrap();
    db.execute_batch("DROP TABLE post_metadata; PRAGMA user_version=5;")
        .unwrap();
    drop(db);
    let bytes = fs::read(&old).unwrap();
    verify_export(&old).unwrap();
    let dest = f.root.join("old-restored");
    restore_local(&old, &dest).unwrap();
    let mut restored = Board::open(&dest).unwrap();
    let s = restored.authenticate("sysop", SECRET).unwrap();
    assert!(restored.post_metadata(&s, p.id).unwrap().legacy);
    assert_eq!(fs::read(old).unwrap(), bytes);
}
#[test]
fn metadata_checks_approval_and_snapshot_missing_rows() {
    let f = Fixture::new();
    let mut b = f.board();
    let root = b.authenticate("sysop", SECRET).unwrap();
    let p = b
        .post(&root, 1, &request_id().unwrap(), "private local post", None)
        .unwrap();
    b.register("alice", SECRET).unwrap();
    let a = b.authenticate("alice", SECRET).unwrap();
    assert!(matches!(b.post_metadata(&a, p.id), Err(Error::Denied)));
    b.set_status(&root, "alice", Status::Approved).unwrap();
    let a = b.authenticate("alice", SECRET).unwrap();
    b.post_metadata(&a, p.id).unwrap();
    b.set_status(&root, "alice", Status::Revoked).unwrap();
    assert!(b.post_metadata(&a, p.id).is_err());
    let export = f.root.join("bad.sqlite3");
    b.export_local(&export).unwrap();
    let db = rusqlite::Connection::open(&export).unwrap();
    db.execute("DELETE FROM post_metadata WHERE post_id=?1", [p.id])
        .unwrap();
    drop(db);
    assert!(verify_export(&export).is_err());
    assert!(restore_local(&export, &f.root.join("bad-restore")).is_err());
    for (i, sql) in [
        "UPDATE post_metadata SET subject=char(27) || '[2J'",
        "UPDATE post_metadata SET article_id='zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz'",
        "UPDATE post_metadata SET created_at=253402300800",
        "UPDATE post_metadata SET origin='legacy'",
    ]
    .iter()
    .enumerate()
    {
        let path = f.root.join(format!("invalid-{i}.sqlite3"));
        b.export_local(&path).unwrap();
        let db = rusqlite::Connection::open(&path).unwrap();
        db.execute_batch("PRAGMA ignore_check_constraints=ON")
            .unwrap();
        db.execute_batch(sql).unwrap();
        drop(db);
        assert!(verify_export(&path).is_err());
        let dest = f.root.join(format!("invalid-restore-{i}"));
        assert!(restore_local(&path, &dest).is_err());
        assert!(!dest.exists());
    }
}
#[test]
fn conflicting_schema_rolls_back_migration_without_changing_legacy_data() {
    let f = Fixture::new();
    let b = f.board();
    drop(b);
    let db = rusqlite::Connection::open(f.path().join("faction.sqlite3")).unwrap();
    db.execute_batch("DROP TABLE post_metadata; PRAGMA user_version=5; CREATE TABLE post_metadata(unrelated TEXT);").unwrap();
    assert!(Board::open(&f.path()).is_err());
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        5
    );
    db.execute_batch("DROP TABLE post_metadata;").unwrap();
    drop(db);
    Board::open(&f.path()).unwrap();
}
#[test]
fn readonly_export_of_a_legacy_live_store_never_upgrades_it() {
    let f = Fixture::new();
    let b = f.board();
    drop(b);
    let db = rusqlite::Connection::open(f.path().join("faction.sqlite3")).unwrap();
    db.execute_batch("DROP TABLE post_metadata; PRAGMA user_version=5;")
        .unwrap();
    drop(db);
    export_local(&f.path(), &f.root.join("legacy-export.sqlite3")).unwrap();
    let db = rusqlite::Connection::open(f.path().join("faction.sqlite3")).unwrap();
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        5
    );
}
