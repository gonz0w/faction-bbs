use faction_core::{
    Board, Error, Session, Status, export_local, request_id, restore_local, verify_export,
};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-wall-passphrase";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("faction-wall-{}", request_id().unwrap()));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        Self(p)
    }
    fn board(&self) -> (Board, Session) {
        let mut b = Board::open(&self.0.join("board")).unwrap();
        b.bootstrap("sysop", PASS).unwrap();
        let s = b.authenticate("sysop", PASS).unwrap();
        (b, s)
    }
    fn db(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(self.0.join("board/faction.sqlite3")).unwrap()
    }
    fn age(&self) {
        self.db()
            .execute("UPDATE wall_receipts SET last_at=unixepoch()-3601", [])
            .unwrap();
    }
    fn enable(&self, b: &mut Board, s: &Session) {
        let mut c = b.wall_config(s).unwrap();
        assert!(!c.enabled);
        c.enabled = true;
        c.cooldown_seconds = 3600;
        b.configure_wall(s, &c).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn post(b: &mut Board, s: &Session, text: &str) -> i64 {
    let ed = b.wall_page(s, None, 0).unwrap().edition;
    b.wall_post(s, ed, &request_id().unwrap(), text).unwrap()
}
#[test]
fn disabled_approval_epoch_and_board_boundaries() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    assert!(b.wall_page(&s, None, 0).is_err());
    f.enable(&mut b, &s);
    b.register("alice", PASS).unwrap();
    let pending = b.authenticate("alice", PASS).unwrap();
    assert!(b.wall_config(&pending).is_err());
    b.set_status(&s, "alice", Status::Approved).unwrap();
    assert!(b.wall_page(&pending, None, 0).is_err());
    let a = b.authenticate("alice", PASS).unwrap();
    let c = b.wall_config(&a).unwrap();
    assert!(b.configure_wall(&a, &c).is_err());
    let id = post(&mut b, &a, "Local hello");
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(b.wall_page(&a, None, 0).is_err());
    assert!(b.wall_delete(&a, id).is_err());
    assert!(b.wall_edit(&a, id, "Local hello", "Changed").is_err());
    assert!(b.wall_post(&a, 0, &request_id().unwrap(), "No").is_err());
    let other = Fixture::new();
    let (mut ob, _) = other.board();
    assert!(ob.wall_config(&s).is_err());
}
#[test]
fn bounded_ascii_control_and_size_validation() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    for body in [
        "".into(),
        "\x1b[2J".into(),
        "hello\rthere".into(),
        "x\t y".into(),
        "x\x7fy".into(),
        "\u{9b}31m".into(),
        "é".into(),
        "\x07".into(),
        "a\nb\nc\nd\ne".into(),
        "x".repeat(73),
        "a".repeat(241),
    ] {
        assert!(
            b.wall_post(&s, 0, &request_id().unwrap(), &body).is_err(),
            "{body:?}"
        );
    }
    let id = post(&mut b, &s, " /\\\n/  \\\n----\nHi!");
    let p = b.wall_page(&s, None, 0).unwrap();
    assert_eq!(p.note.unwrap().id, id);
    assert_eq!(p.count, 1);
    let mut c = b.wall_config(&s).unwrap();
    for n in [0, 129] {
        c.max_notes = n;
        assert!(b.configure_wall(&s, &c).is_err());
    }
    c.max_notes = 64;
    c.name = "bad\x1b".into();
    assert!(b.configure_wall(&s, &c).is_err());
}
#[test]
fn cooldown_retry_deleted_receipt_and_edit_conflict() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    let key = request_id().unwrap();
    let id = b.wall_post(&s, 0, &key, "First").unwrap();
    assert_eq!(b.wall_post(&s, 0, &key, "First").unwrap(), id);
    assert!(matches!(
        b.wall_post(&s, 0, &key, "Different"),
        Err(Error::RequestConflict)
    ));
    assert!(
        b.wall_post(&s, 0, &request_id().unwrap(), "Second")
            .is_err()
    );
    assert!(b.wall_edit(&s, id, "First", "Edited").is_err());
    f.age();
    b.wall_edit(&s, id, "First", "Edited").unwrap();
    assert!(b.wall_page(&s, None, 0).unwrap().note.unwrap().edited);
    f.age();
    assert!(matches!(
        b.wall_edit(&s, id, "First", "Conflict"),
        Err(Error::RequestConflict)
    ));
    b.wall_delete(&s, id).unwrap();
    assert!(b.wall_post(&s, 0, &key, "First").is_err());
    assert_eq!(b.wall_page(&s, None, 0).unwrap().count, 0);
}
#[test]
fn chronological_limits_retention_and_two_board_policies() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    let mut c = b.wall_config(&s).unwrap();
    c.max_notes = 2;
    c.name = "Basement wall".into();
    c.note_bytes = 32;
    c.retention_days = 1;
    b.configure_wall(&s, &c).unwrap();
    post(&mut b, &s, "One");
    f.age();
    let two = post(&mut b, &s, "Two");
    f.age();
    let three = post(&mut b, &s, "Three");
    let p = b.wall_page(&s, None, 0).unwrap();
    assert_eq!(p.count, 2);
    assert_eq!(p.note.unwrap().id, two);
    assert_eq!(
        b.wall_page(&s, None, usize::MAX).unwrap().note.unwrap().id,
        three
    );
    f.db()
        .execute(
            "UPDATE wall_notes SET posted_at=unixepoch()-86401 WHERE id=?1",
            [two],
        )
        .unwrap();
    assert_eq!(b.wall_page(&s, None, 0).unwrap().count, 1);
    let of = Fixture::new();
    let (mut ob, os) = of.board();
    assert!(!ob.wall_config(&os).unwrap().enabled);
    assert_eq!(ob.wall_config(&os).unwrap().name, "Community wall");
    assert!(ob.wall_page(&s, None, 0).is_err());
    c.note_bytes = 33;
    assert!(b.configure_wall(&s, &c).is_ok());
}
#[test]
fn weekly_rotation_explicit_archives_no_clock_change_and_readonly_edits() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    post(&mut b, &s, "Old local");
    let mut c = b.wall_config(&s).unwrap();
    c.weekly = true;
    c.archive_editions = 1;
    b.configure_wall(&s, &c).unwrap();
    assert_eq!(b.wall_page(&s, None, 0).unwrap().count, 0);
    f.age();
    let id = post(&mut b, &s, "Last week");
    let now = b.wall_page(&s, None, 0).unwrap().edition;
    f.db()
        .execute(
            "UPDATE wall_notes SET edition=?1 WHERE id=?2",
            rusqlite::params![now - 1, id],
        )
        .unwrap();
    let archived = b.wall_archives(&s).unwrap();
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].0, now - 1);
    assert!(!b.wall_page(&s, Some(now - 1), 0).unwrap().current);
    f.age();
    assert!(b.wall_edit(&s, id, "Last week", "Changed").is_err());
    assert!(
        b.wall_post(&s, now - 1, &request_id().unwrap(), "Wrong week")
            .is_err()
    );
    f.db()
        .execute("UPDATE wall_settings SET edition=?1", [now + 1])
        .unwrap();
    assert_eq!(b.wall_page(&s, None, 0).unwrap().edition, now + 1); // rollback cannot reopen a closed week
    c.archive_editions = 0;
    b.configure_wall(&s, &c).unwrap();
    assert!(b.wall_archives(&s).unwrap().is_empty());
    assert!(b.wall_page(&s, Some(now - 1), 0).is_err());
}
#[test]
fn own_edit_delete_sysop_moderation_and_stable_selected_id() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    b.register("alice", PASS).unwrap();
    b.set_status(&s, "alice", Status::Approved).unwrap();
    let a = b.authenticate("alice", PASS).unwrap();
    let first = post(&mut b, &s, "Sysop note");
    let second = post(&mut b, &a, "Member note");
    assert!(b.wall_delete(&a, first).is_err());
    assert!(b.wall_edit(&a, first, "Sysop note", "Forged").is_err());
    b.wall_delete(&s, first).unwrap();
    assert!(b.wall_delete(&s, first).is_err());
    assert_eq!(b.wall_page(&s, None, 0).unwrap().note.unwrap().id, second);
    f.age();
    b.wall_edit(&a, second, "Member note", "My edit").unwrap();
    b.wall_delete(&s, second).unwrap();
    assert_eq!(b.wall_page(&s, None, 0).unwrap().count, 0);
}
#[test]
fn export_restore_wall_and_readonly_schema8_upgrade() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    let id = post(&mut b, &s, "Included local wall");
    drop(b);
    let out = f.0.join("snapshot");
    export_local(&f.0.join("board"), &out).unwrap();
    verify_export(&out).unwrap();
    let restored = f.0.join("restored");
    restore_local(&out, &restored).unwrap();
    let mut b = Board::open(&restored).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert_eq!(b.wall_page(&s, None, 0).unwrap().note.unwrap().id, id);
    assert!(
        b.wall_post(&s, 0, &request_id().unwrap(), "Still cooling")
            .is_err()
    );
    drop(b);
    let db = rusqlite::Connection::open(restored.join("faction.sqlite3")).unwrap();
    db.execute_batch("DROP TABLE directory_submissions;DROP TABLE visit_activity;DROP TABLE social_pages;DROP TABLE social_presence;DROP TABLE social_blocks;DROP TABLE social_prefs;DROP TABLE social_cooldowns;DROP TABLE social_settings;DROP TABLE magazine_entries;DROP TABLE magazine_issues;DROP TABLE magazine_submissions;DROP TABLE file_requests;DROP INDEX discussion_author_posts;DROP TABLE discussion_bookmarks;DROP TABLE discussion_settings;DROP TRIGGER catchup_file_insert; DROP TRIGGER catchup_file_publish; DROP TABLE catchup_favorites; DROP TABLE catchup_members; DROP TABLE catchup_settings; DROP TABLE catchup_publications; DROP TABLE poll_ballots; DROP TABLE poll_choices; DROP TABLE polls; DROP TABLE poll_settings; DROP TABLE gathering_rsvps; DROP TABLE gatherings; DROP TABLE gathering_settings; DROP TABLE member_plans; DROP TABLE member_pages; DROP TABLE page_settings; DROP TABLE wall_receipts; DROP TABLE wall_notes; DROP TABLE wall_settings; PRAGMA user_version=8").unwrap();
    drop(db);
    let old = f.0.join("old");
    export_local(&restored, &old).unwrap();
    verify_export(&old).unwrap();
    let before = fs::read(&old).unwrap();
    let mut b = Board::open(&restored).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert!(!b.wall_config(&s).unwrap().enabled);
    assert_eq!(fs::read(&old).unwrap(), before);
}
#[test]
fn invalid_snapshot_content_and_quota_are_rejected() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    post(&mut b, &s, "Valid");
    drop(b);
    f.db()
        .execute("UPDATE wall_notes SET body=?1", ["\x1b[2J"])
        .unwrap();
    assert!(export_local(&f.0.join("board"), &f.0.join("bad")).is_err());
}
#[test]
fn busy_transaction_does_not_lose_request_or_duplicate() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    let db = f.db();
    db.execute_batch("BEGIN IMMEDIATE").unwrap();
    let key = request_id().unwrap();
    assert!(matches!(
        b.wall_post(&s, 0, &key, "Retry me"),
        Err(Error::Busy)
    ));
    db.execute_batch("ROLLBACK").unwrap();
    let id = b.wall_post(&s, 0, &key, "Retry me").unwrap();
    assert_eq!(b.wall_post(&s, 0, &key, "Retry me").unwrap(), id);
    assert_eq!(b.wall_page(&s, None, 0).unwrap().count, 1);
}
