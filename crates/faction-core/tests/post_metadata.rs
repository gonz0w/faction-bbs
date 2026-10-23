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
    db.execute_batch("DROP TABLE gathering_rsvps; DROP TABLE gatherings; DROP TABLE gathering_settings; DROP TABLE member_plans; DROP TABLE member_pages; DROP TABLE page_settings; DROP TABLE wall_receipts; DROP TABLE wall_notes; DROP TABLE wall_settings; DROP TABLE room_drafts; DROP TABLE post_metadata; PRAGMA user_version=5;")
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
    db.execute_batch("DROP TABLE gathering_rsvps; DROP TABLE gatherings; DROP TABLE gathering_settings; DROP TABLE member_plans; DROP TABLE member_pages; DROP TABLE page_settings; DROP TABLE wall_receipts; DROP TABLE wall_notes; DROP TABLE wall_settings; DROP TABLE room_drafts; DROP TABLE post_metadata; PRAGMA user_version=5;")
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
    db.execute_batch("DROP TABLE gathering_rsvps; DROP TABLE gatherings; DROP TABLE gathering_settings; DROP TABLE member_plans; DROP TABLE member_pages; DROP TABLE page_settings; DROP TABLE wall_receipts; DROP TABLE wall_notes; DROP TABLE wall_settings; DROP TABLE room_drafts; DROP TABLE post_metadata; PRAGMA user_version=5; CREATE TABLE post_metadata(unrelated TEXT);").unwrap();
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
    db.execute_batch("DROP TABLE gathering_rsvps; DROP TABLE gatherings; DROP TABLE gathering_settings; DROP TABLE member_plans; DROP TABLE member_pages; DROP TABLE page_settings; DROP TABLE wall_receipts; DROP TABLE wall_notes; DROP TABLE wall_settings; DROP TABLE room_drafts; DROP TABLE post_metadata; PRAGMA user_version=5;")
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

fn schema_six(f: &Fixture) {
    let db = rusqlite::Connection::open(f.path().join("faction.sqlite3")).unwrap();
    db.execute_batch(
        "CREATE TEMP TABLE saved AS SELECT * FROM post_metadata; DROP TABLE gathering_rsvps; DROP TABLE gatherings; DROP TABLE gathering_settings; DROP TABLE member_plans; DROP TABLE member_pages; DROP TABLE page_settings; DROP TABLE wall_receipts; DROP TABLE wall_notes; DROP TABLE wall_settings; DROP TABLE room_drafts; DROP TABLE post_metadata;",
    )
    .unwrap();
    db.execute_batch(include_str!("../src/post_metadata.sql"))
        .unwrap();
    db.execute_batch("DELETE FROM post_metadata; INSERT INTO post_metadata SELECT * FROM saved; PRAGMA user_version=6;").unwrap();
}
#[test]
fn unicode_subjects_preserve_bytes_retries_replies_authority_and_restore() {
    use faction_core::PostDraft;
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("sysop", SECRET).unwrap();
    let subject = "a".to_owned() + &"界".repeat(53); // exactly 160 bytes
    for length in [65, 74] {
        assert!(matches!(
            b.post_article(
                &s,
                1,
                &"r".repeat(length),
                PostDraft {
                    subject: &subject,
                    body: "body",
                    reply_to: None
                }
            ),
            Err(Error::Invalid(_))
        ));
    }
    let key = request_id().unwrap();
    let d = || PostDraft {
        subject: &subject,
        body: "unchanged body",
        reply_to: None,
    };
    let lock = rusqlite::Connection::open(f.path().join("faction.sqlite3")).unwrap();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    assert!(matches!(b.post_article(&s, 1, &key, d()), Err(Error::Busy)));
    lock.execute_batch("ROLLBACK").unwrap();
    let post = b.post_article(&s, 1, &key, d()).unwrap();
    assert_eq!(b.post_article(&s, 1, &key, d()).unwrap().id, post.id);
    assert_eq!(b.post_metadata(&s, post.id).unwrap().subject, subject);
    assert!(matches!(
        b.post_article(
            &s,
            1,
            &key,
            PostDraft {
                subject: "other",
                ..d()
            }
        ),
        Err(Error::RequestConflict)
    ));
    let expected = "Re: a".to_owned() + &"界".repeat(51);
    assert_eq!(expected.len(), 158);
    assert_eq!(b.reply_subject(&s, 1, post.id).unwrap(), expected);
    let reply = b
        .post(&s, 1, &request_id().unwrap(), "reply", Some(post.id))
        .unwrap();
    assert_eq!(b.reply_subject(&s, 1, reply.id).unwrap(), expected);
    for subject in [
        "é".repeat(81),
        "🦀".repeat(41),
        "bad\u{85}".into(),
        "bad\n".into(),
    ] {
        assert!(
            b.post_article(
                &s,
                1,
                &request_id().unwrap(),
                PostDraft {
                    subject: &subject,
                    ..d()
                }
            )
            .is_err()
        );
    }
    for subject in ["é".repeat(80), "🦀".repeat(40), "café e\u{301} שלום".into()] {
        let p = b
            .post_article(
                &s,
                1,
                &request_id().unwrap(),
                PostDraft {
                    subject: &subject,
                    ..d()
                },
            )
            .unwrap();
        let article = b.room_article(&s, p.id).unwrap();
        assert_eq!(article.headers().subject, subject);
        assert_eq!(
            faction_articles::Article::parse(article.to_wire().unwrap().as_bytes())
                .unwrap()
                .headers()
                .subject,
            subject
        );
    }
    let original = b.post_metadata(&s, post.id).unwrap();
    let export = f.root.join("unicode.sqlite3");
    b.export_local(&export).unwrap();
    let dest = f.root.join("unicode-restore");
    restore_local(&export, &dest).unwrap();
    let mut restored = Board::open(&dest).unwrap();
    let rs = restored.authenticate("sysop", SECRET).unwrap();
    assert_eq!(restored.post_metadata(&rs, post.id).unwrap(), original);
    b.register("reader", SECRET).unwrap();
    b.set_status(&s, "reader", Status::Approved).unwrap();
    let reader = b.authenticate("reader", SECRET).unwrap();
    b.set_status(&s, "reader", Status::Revoked).unwrap();
    assert!(matches!(
        b.post_article(&reader, 1, &request_id().unwrap(), d()),
        Err(Error::StaleSession)
    ));
    assert!(matches!(
        b.reply_subject(&reader, 1, post.id),
        Err(Error::StaleSession)
    ));
}
#[test]
fn schema_six_is_read_only_in_exports_and_upgrades_without_changing_metadata() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("sysop", SECRET).unwrap();
    let p = b
        .post(&s, 1, &request_id().unwrap(), "retain", None)
        .unwrap();
    let metadata = b.post_metadata(&s, p.id).unwrap();
    drop(b);
    schema_six(&f);
    let export = f.root.join("six.sqlite3");
    export_local(&f.path(), &export).unwrap();
    let bytes = fs::read(&export).unwrap();
    verify_export(&export).unwrap();
    let dest = f.root.join("restored-six");
    restore_local(&export, &dest).unwrap();
    let mut restored = Board::open(&dest).unwrap();
    let rs = restored.authenticate("sysop", SECRET).unwrap();
    assert_eq!(restored.post_metadata(&rs, p.id).unwrap(), metadata);
    assert_eq!(fs::read(&export).unwrap(), bytes);
    let db = rusqlite::Connection::open(dest.join("faction.sqlite3")).unwrap();
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        11
    );
    assert_eq!(
        db.query_row(
            "SELECT subject FROM post_metadata WHERE post_id=?1",
            [p.id],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        metadata.subject
    );
    assert!(
        db.execute("UPDATE post_metadata SET subject=?1", ["é".repeat(81)])
            .is_err()
    );
}
#[test]
fn old_contract_rejects_unicode_and_new_snapshots_reject_malformed_utf8() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("sysop", SECRET).unwrap();
    b.post(&s, 1, &request_id().unwrap(), "keep", None).unwrap();
    drop(b);
    schema_six(&f);
    let db = rusqlite::Connection::open(f.path().join("faction.sqlite3")).unwrap();
    db.execute("UPDATE post_metadata SET subject='café'", [])
        .unwrap();
    assert!(Board::open(&f.path()).is_err());
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        6
    );
    assert!(export_local(&f.path(), &f.root.join("bad-six.sqlite3")).is_err());
    db.execute("UPDATE post_metadata SET subject='valid'", [])
        .unwrap();
    drop(db);
    let b = Board::open(&f.path()).unwrap();
    let export = f.root.join("bad-seven.sqlite3");
    b.export_local(&export).unwrap();
    let db = rusqlite::Connection::open(&export).unwrap();
    db.execute_batch("UPDATE post_metadata SET subject=CAST(X'ff' AS TEXT)")
        .unwrap();
    drop(db);
    assert!(verify_export(&export).is_err());
    assert!(restore_local(&export, &f.root.join("bad-seven-restore")).is_err());
}

#[test]
fn unknown_schema_six_metadata_catalog_is_not_silently_replaced() {
    let f = Fixture::new();
    let b = f.board();
    drop(b);
    schema_six(&f);
    let db = rusqlite::Connection::open(f.path().join("faction.sqlite3")).unwrap();
    db.execute_batch(
        "CREATE TRIGGER unknown_metadata AFTER INSERT ON post_metadata BEGIN SELECT 1; END;",
    )
    .unwrap();
    assert!(Board::open(&f.path()).is_err());
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        6
    );
    assert!(
        db.prepare("SELECT 1 FROM sqlite_schema WHERE name='unknown_metadata'")
            .unwrap()
            .exists([])
            .unwrap()
    );
}
