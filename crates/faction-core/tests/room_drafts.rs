use faction_core::{
    Board, Error, PostDraft, SavedRoomDraft, Status, export_local, request_id, restore_local,
    verify_export,
};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASSWORD: &str = "synthetic-room-draft-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("faction-drafts-{}", request_id().unwrap()));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        Self(p)
    }
    fn board(&self) -> Board {
        let mut b = Board::open(&self.0.join("board")).unwrap();
        b.bootstrap("sysop", PASSWORD).unwrap();
        b
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
fn draft(body: &str) -> PostDraft<'_> {
    PostDraft {
        subject: "Échos 界",
        body,
        reply_to: None,
    }
}
fn save(b: &mut Board, s: &faction_core::Session) -> SavedRoomDraft {
    b.save_room_draft(s, 1, None, draft("First\n\nSecond"))
        .unwrap()
}
#[test]
fn checkpoints_are_owned_persistent_and_revisions_do_not_overwrite_each_other() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("sysop", PASSWORD).unwrap();
    let saved = save(&mut b, &s);
    assert_eq!(b.room_draft(&s, 1).unwrap(), Some(saved.clone()));
    assert!(matches!(
        b.save_room_draft(&s, 1, None, draft("wrong")),
        Err(Error::RequestConflict)
    ));
    let next = b
        .save_room_draft(&s, 1, Some(&saved.request_id), draft("Replacement"))
        .unwrap();
    assert_ne!(saved.request_id, next.request_id);
    assert!(matches!(
        b.delete_room_draft(&s, 1, &saved.request_id),
        Err(Error::RequestConflict)
    ));
    drop(b);
    let mut b = Board::open(&f.0.join("board")).unwrap();
    let s = b.authenticate("sysop", PASSWORD).unwrap();
    assert_eq!(b.room_draft(&s, 1).unwrap(), Some(next.clone()));
    b.delete_room_draft(&s, 1, &next.request_id).unwrap();
    assert!(b.room_draft(&s, 1).unwrap().is_none());
}
#[test]
fn pending_other_members_and_revoked_sessions_cannot_recover_a_checkpoint() {
    let f = Fixture::new();
    let mut b = f.board();
    let root = b.authenticate("sysop", PASSWORD).unwrap();
    b.register("alice", PASSWORD).unwrap();
    let pending = b.authenticate("alice", PASSWORD).unwrap();
    assert!(b.room_draft(&pending, 1).is_err());
    b.set_status(&root, "alice", Status::Approved).unwrap();
    let alice = b.authenticate("alice", PASSWORD).unwrap();
    save(&mut b, &alice);
    assert!(b.room_draft(&root, 1).unwrap().is_none());
    b.set_status(&root, "alice", Status::Revoked).unwrap();
    assert!(b.room_draft(&alice, 1).is_err());
    b.set_status(&root, "alice", Status::Approved).unwrap();
    let alice = b.authenticate("alice", PASSWORD).unwrap();
    assert!(b.room_draft(&alice, 1).unwrap().is_none());
    let other = Fixture::new();
    let mut other = other.board();
    assert!(other.room_draft(&alice, 1).is_err());
}
#[test]
fn quotas_expiry_and_body_limits_keep_saved_data_bounded() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("sysop", PASSWORD).unwrap();
    for i in 0..4 {
        let room = b.create_room(&s, &format!("room{i}"), "Room").unwrap();
        b.save_room_draft(&s, room, None, draft("small")).unwrap();
    }
    assert!(b.save_room_draft(&s, 1, None, draft("fifth")).is_err());
    f.db()
        .execute("UPDATE room_drafts SET updated_at=0", [])
        .unwrap();
    assert!(b.room_draft(&s, 1).unwrap().is_none());
    for body in ["x".repeat(1025), "\n".repeat(32), "bad\x1b".into()] {
        assert!(b.save_room_draft(&s, 1, None, draft(&body)).is_err());
    }
    save(&mut b, &s);
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM room_drafts", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}
#[test]
fn send_removes_only_its_checkpoint_atomically_and_retry_never_duplicates() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("sysop", PASSWORD).unwrap();
    let saved = save(&mut b, &s);
    let post = b
        .post_article(
            &s,
            1,
            &saved.request_id,
            draft("Final body after checkpoint"),
        )
        .unwrap();
    assert!(b.room_draft(&s, 1).unwrap().is_none());
    assert_eq!(
        b.post_article(
            &s,
            1,
            &saved.request_id,
            draft("Final body after checkpoint")
        )
        .unwrap()
        .id,
        post.id
    );
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
    let saved = save(&mut b, &s);
    let newer = b
        .save_room_draft(&s, 1, Some(&saved.request_id), draft("newer revision"))
        .unwrap();
    b.post_article(
        &s,
        1,
        &saved.request_id,
        draft("older editor's explicit post"),
    )
    .unwrap();
    assert_eq!(b.room_draft(&s, 1).unwrap(), Some(newer));
}
#[test]
fn export_restore_preserves_room_drafts_without_importing_old_sessions() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("sysop", PASSWORD).unwrap();
    let saved = save(&mut b, &s);
    let out = f.0.join("export");
    b.export_local(&out).unwrap();
    verify_export(&out).unwrap();
    restore_local(&out, &f.0.join("restored")).unwrap();
    let mut restored = Board::open(&f.0.join("restored")).unwrap();
    assert!(restored.room_draft(&s, 1).is_err());
    let current = restored.authenticate("sysop", PASSWORD).unwrap();
    assert_eq!(restored.room_draft(&current, 1).unwrap(), Some(saved));
}
#[test]
fn snapshots_reject_malformed_drafts_and_cross_room_reply_bindings() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("sysop", PASSWORD).unwrap();
    save(&mut b, &s);
    let out = f.0.join("export");
    b.export_local(&out).unwrap();
    let db = rusqlite::Connection::open(&out).unwrap();
    db.execute("UPDATE room_drafts SET body=?1", ["\n".repeat(32)])
        .unwrap();
    assert!(verify_export(&out).is_err());
    db.execute("UPDATE room_drafts SET body='ok',epoch=99", [])
        .unwrap();
    assert!(verify_export(&out).is_err());
    let room = b.create_room(&s, "elsewhere", "Elsewhere").unwrap();
    let parent = b
        .post(&s, room, &request_id().unwrap(), "parent", None)
        .unwrap();
    assert!(
        b.save_room_draft(
            &s,
            1,
            None,
            PostDraft {
                subject: "",
                body: "reply",
                reply_to: Some(parent.id)
            }
        )
        .is_err()
    );
}
#[test]
fn schema_seven_exports_stay_readonly_and_upgrade_adds_no_invented_drafts() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("sysop", PASSWORD).unwrap();
    let post = b
        .post(&s, 1, &request_id().unwrap(), "existing", None)
        .unwrap();
    let meta = b.post_metadata(&s, post.id).unwrap();
    drop(b);
    let db = f.db();
    db.execute_batch("DROP TABLE member_plans; DROP TABLE member_pages; DROP TABLE page_settings; DROP TABLE wall_receipts; DROP TABLE wall_notes; DROP TABLE wall_settings; DROP TABLE room_drafts; PRAGMA user_version=7;")
        .unwrap();
    drop(db);
    let out = f.0.join("seven");
    export_local(&f.0.join("board"), &out).unwrap();
    verify_export(&out).unwrap();
    assert_eq!(
        f.db()
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        7
    );
    let mut b = Board::open(&f.0.join("board")).unwrap();
    let s = b.authenticate("sysop", PASSWORD).unwrap();
    assert!(b.room_draft(&s, 1).unwrap().is_none());
    assert_eq!(b.post_metadata(&s, post.id).unwrap(), meta);
    assert_eq!(
        f.db()
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        10
    );
}
#[test]
fn owner_recovery_invalidates_saved_sysop_drafts() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("sysop", PASSWORD).unwrap();
    save(&mut b, &s);
    b.recover_local_sysop("sysop", "replacement-synthetic-password")
        .unwrap();
    let s = b
        .authenticate("sysop", "replacement-synthetic-password")
        .unwrap();
    assert!(b.room_draft(&s, 1).unwrap().is_none());
}
