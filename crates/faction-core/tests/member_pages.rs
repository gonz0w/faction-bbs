use faction_core::{
    Board, Error, Session, Status, export_local, request_id, restore_local, verify_export,
};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-member-page-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("faction-members-{}", request_id().unwrap()));
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
    fn enable(&self, b: &mut Board, s: &Session) {
        let mut c = b.page_config(s).unwrap();
        c.enabled = true;
        b.configure_pages(s, &c).unwrap();
    }
    fn db(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(self.0.join("board/faction.sqlite3")).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn alice(b: &mut Board, s: &Session) -> Session {
    b.register("alice", PASS).unwrap();
    b.set_status(s, "alice", Status::Approved).unwrap();
    b.authenticate("alice", PASS).unwrap()
}
fn save(b: &mut Board, s: &Session, name: &str) {
    let mut p = b.member_page(s, name).unwrap();
    p.fields = [
        "My bio".into(),
        "Radio and ANSI".into(),
        " /\\\n/  \\".into(),
        "Building a tiny BBS".into(),
    ];
    b.save_member_page(s, &p).unwrap();
}
#[test]
fn explicit_approval_cross_board_and_revocation_hide_authors() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    assert!(!b.page_config(&s).unwrap().enabled);
    assert!(b.member_page(&s, "sysop").is_err());
    f.enable(&mut b, &s);
    let a = alice(&mut b, &s);
    save(&mut b, &a, "alice");
    assert_eq!(b.page_handles(&s, 0).unwrap(), vec!["alice"]);
    let id = b
        .add_member_plan(&a, &request_id().unwrap(), "Current work", true)
        .unwrap();
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(b.member_page(&s, "alice").is_err());
    assert!(b.member_plans(&s, "alice").is_err());
    assert!(b.page_handles(&s, 0).unwrap().is_empty());
    assert!(b.page_handles(&a, 100).is_err());
    assert!(b.remove_member_plan(&a, id).is_err());
    assert!(b.delete_member_page(&a, None).is_err());
    let other = Fixture::new();
    let (mut ob, _) = other.board();
    assert!(ob.page_config(&s).is_err());
    b.set_status(&s, "alice", Status::Approved).unwrap();
    assert_eq!(b.page_handles(&s, 0).unwrap(), vec!["alice"]);
}
#[test]
fn hidden_fields_page_and_plans_do_not_escape_local_visibility() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    let a = alice(&mut b, &s);
    save(&mut b, &a, "alice");
    let mut p = b.member_page(&a, "alice").unwrap();
    p.visible = 2;
    b.save_member_page(&a, &p).unwrap();
    let seen = b.member_page(&s, "alice").unwrap();
    assert!(seen.fields[0].is_empty());
    assert_eq!(seen.fields[1], "Radio and ANSI");
    assert!(seen.fields[2].is_empty());
    assert!(seen.revision.is_none());
    b.add_member_plan(&a, &request_id().unwrap(), "Shared project", true)
        .unwrap();
    b.add_member_plan(&a, &request_id().unwrap(), "Hidden project", false)
        .unwrap();
    assert_eq!(b.member_plans(&s, "alice").unwrap().len(), 1);
    assert_eq!(b.member_plans(&a, "alice").unwrap().len(), 2);
    let mut p = b.member_page(&a, "alice").unwrap();
    p.published = false;
    b.save_member_page(&a, &p).unwrap();
    assert!(b.member_page(&s, "alice").is_err());
    assert!(b.member_plans(&s, "alice").is_err());
    assert!(b.page_handles(&s, 0).unwrap().is_empty());
    assert_eq!(b.member_page(&a, "alice").unwrap().fields[0], "My bio");
}
#[test]
fn owner_edits_conflict_and_disabled_field_policy_preserves_authorship() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    let a = alice(&mut b, &s);
    save(&mut b, &a, "alice");
    let old = b.member_page(&a, "alice").unwrap();
    let mut p = old.clone();
    p.fields[0] = "New bio".into();
    b.save_member_page(&a, &p).unwrap();
    assert!(matches!(
        b.save_member_page(&a, &old),
        Err(Error::RequestConflict)
    ));
    assert!(b.save_member_page(&s, &old).is_err());
    let policy = b.page_config(&s).unwrap();
    assert!(b.configure_pages(&a, &policy).is_err());
    let mut c = b.page_config(&s).unwrap();
    c.fields = 2;
    b.configure_pages(&s, &c).unwrap();
    assert!(b.member_page(&s, "alice").unwrap().fields[0].is_empty());
    let mut p = b.member_page(&a, "alice").unwrap();
    p.fields[0] = "Disallowed new bio".into();
    assert!(b.save_member_page(&a, &p).is_err());
    p.fields[0].clear();
    b.save_member_page(&a, &p).unwrap();
}
#[test]
fn sysop_hide_preserves_text_and_author_cannot_unhide() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    let a = alice(&mut b, &s);
    save(&mut b, &a, "alice");
    let before = b.member_page(&a, "alice").unwrap();
    b.moderate_member_page(&s, "alice", "hide").unwrap();
    assert!(b.member_page(&s, "alice").is_err());
    assert!(b.page_handles(&s, 0).unwrap().is_empty());
    assert!(b.save_member_page(&a, &before).is_err());
    let mut p = b.member_page(&a, "alice").unwrap();
    assert!(p.moderated);
    p.moderated = false;
    p.fields[3] = "New project".into();
    b.save_member_page(&a, &p).unwrap();
    assert!(b.member_page(&a, "alice").unwrap().moderated);
    let before_delete = b.member_page(&a, "alice").unwrap();
    b.delete_member_page(&a, before_delete.revision.as_deref())
        .unwrap();
    let mut recreated = b.member_page(&a, "alice").unwrap();
    assert!(recreated.moderated && !recreated.published);
    assert!(recreated.fields.iter().all(String::is_empty));
    recreated.fields[0] = "My bio".into();
    recreated.published = true;
    recreated.visible = 15;
    b.save_member_page(&a, &recreated).unwrap();
    assert!(b.member_page(&s, "alice").is_err());
    assert!(b.moderate_member_page(&s, "alice", "remove").is_err());
    b.moderate_member_page(&s, "alice", "show").unwrap();
    assert_eq!(b.member_page(&s, "alice").unwrap().fields[0], "My bio");
    let mut c = b.page_config(&s).unwrap();
    c.moderator_remove = true;
    b.configure_pages(&s, &c).unwrap();
    b.moderate_member_page(&s, "alice", "remove").unwrap();
    assert!(b.member_page(&a, "alice").unwrap().revision.is_none());
}
#[test]
fn bounded_dated_history_retry_retention_and_delete() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    save(&mut b, &s, "sysop");
    let mut c = b.page_config(&s).unwrap();
    c.history = 2;
    c.days = 7;
    b.configure_pages(&s, &c).unwrap();
    let key = request_id().unwrap();
    let first = b.add_member_plan(&s, &key, "One", true).unwrap();
    assert_eq!(b.add_member_plan(&s, &key, "One", true).unwrap(), first);
    assert!(b.add_member_plan(&s, &key, "Different", true).is_err());
    let second = b
        .add_member_plan(&s, &request_id().unwrap(), "Two", true)
        .unwrap();
    let third = b
        .add_member_plan(&s, &request_id().unwrap(), "Three", true)
        .unwrap();
    let rows = b.member_plans(&s, "sysop").unwrap();
    assert_eq!(
        rows.iter().map(|r| r.id).collect::<Vec<_>>(),
        vec![third, second]
    );
    assert!(rows[0].posted.ends_with("UTC"));
    f.db()
        .execute(
            "UPDATE member_plans SET posted_at=unixepoch()-604801 WHERE id=?1",
            [second],
        )
        .unwrap();
    assert_eq!(b.member_plans(&s, "sysop").unwrap().len(), 1);
    b.remove_member_plan(&s, third).unwrap();
    assert!(b.member_plans(&s, "sysop").unwrap().is_empty());
}
#[test]
fn malformed_fields_and_update_limits_are_atomic() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    save(&mut b, &s, "sysop");
    for bad in [
        "\x1b[2J".into(),
        "bad\r\n".into(),
        "\t".into(),
        "é".into(),
        "a".repeat(241),
        "a".repeat(73),
        "1\n2\n3\n4\n5".into(),
    ] {
        let mut p = b.member_page(&s, "sysop").unwrap();
        p.fields[0] = bad;
        assert!(b.save_member_page(&s, &p).is_err());
        assert_eq!(b.member_page(&s, "sysop").unwrap().fields[0], "My bio");
    }
    for text in [
        "".into(),
        "a\nb\nc\nd\ne\nf\ng\nh\ni".into(),
        "\x1b".into(),
        "x".repeat(481),
    ] {
        assert!(
            b.add_member_plan(&s, &request_id().unwrap(), &text, true)
                .is_err()
        );
    }
    let mut c = b.page_config(&s).unwrap();
    c.history = 17;
    assert!(b.configure_pages(&s, &c).is_err());
}
#[test]
fn delete_conflict_and_authority_remove_page_and_all_plans() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    let a = alice(&mut b, &s);
    save(&mut b, &a, "alice");
    let id = b
        .add_member_plan(&a, &request_id().unwrap(), "My plan", true)
        .unwrap();
    assert!(b.remove_member_plan(&s, id).is_err());
    assert!(b.delete_member_page(&a, None).is_err());
    let p = b.member_page(&a, "alice").unwrap();
    b.delete_member_page(&a, p.revision.as_deref()).unwrap();
    assert!(b.member_plans(&a, "alice").unwrap().is_empty());
    assert!(b.page_handles(&s, 0).unwrap().is_empty());
}
#[test]
fn owner_export_restore_private_visibility_and_schema9_readonly_upgrade() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    save(&mut b, &s, "sysop");
    b.add_member_plan(
        &s,
        &request_id().unwrap(),
        "Hidden but owner backed up",
        false,
    )
    .unwrap();
    drop(b);
    let out = f.0.join("export");
    export_local(&f.0.join("board"), &out).unwrap();
    verify_export(&out).unwrap();
    let dest = f.0.join("restored");
    restore_local(&out, &dest).unwrap();
    let mut b = Board::open(&dest).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert_eq!(
        b.member_plans(&s, "sysop").unwrap()[0].body,
        "Hidden but owner backed up"
    );
    let p = b.member_page(&s, "sysop").unwrap();
    b.delete_member_page(&s, p.revision.as_deref()).unwrap();
    drop(b);
    let db = rusqlite::Connection::open(dest.join("faction.sqlite3")).unwrap();
    db.execute_batch("DROP TABLE directory_submissions;DROP TABLE visit_activity;DROP TABLE social_pages;DROP TABLE social_presence;DROP TABLE social_blocks;DROP TABLE social_prefs;DROP TABLE social_cooldowns;DROP TABLE social_settings;DROP TABLE magazine_entries;DROP TABLE magazine_issues;DROP TABLE magazine_submissions;DROP TABLE file_requests;DROP INDEX discussion_author_posts;DROP TABLE discussion_bookmarks;DROP TABLE discussion_settings;DROP TRIGGER catchup_file_insert; DROP TRIGGER catchup_file_publish; DROP TABLE catchup_favorites; DROP TABLE catchup_members; DROP TABLE catchup_settings; DROP TABLE catchup_publications; DROP TABLE poll_ballots; DROP TABLE poll_choices; DROP TABLE polls; DROP TABLE poll_settings; DROP TABLE gathering_rsvps; DROP TABLE gatherings; DROP TABLE gathering_settings; DROP TABLE member_plans; DROP TABLE member_pages; DROP TABLE page_settings; PRAGMA user_version=9").unwrap();
    drop(db);
    let old = f.0.join("old");
    export_local(&dest, &old).unwrap();
    let bytes = fs::read(&old).unwrap();
    verify_export(&old).unwrap();
    let mut b = Board::open(&dest).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert!(!b.page_config(&s).unwrap().enabled);
    assert_eq!(fs::read(&old).unwrap(), bytes);
}
#[test]
fn busy_plan_retry_is_once_and_invalid_snapshot_is_refused() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s);
    save(&mut b, &s, "sysop");
    let db = f.db();
    db.execute_batch("BEGIN IMMEDIATE").unwrap();
    let key = request_id().unwrap();
    assert!(matches!(
        b.add_member_plan(&s, &key, "Retry", true),
        Err(Error::Busy)
    ));
    db.execute_batch("ROLLBACK").unwrap();
    let id = b.add_member_plan(&s, &key, "Retry", true).unwrap();
    assert_eq!(b.add_member_plan(&s, &key, "Retry", true).unwrap(), id);
    db.execute("UPDATE member_pages SET bio=?1", ["\x1b[2J"])
        .unwrap();
    drop(b);
    assert!(export_local(&f.0.join("board"), &f.0.join("bad")).is_err());
}
