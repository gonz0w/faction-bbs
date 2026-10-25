use faction_core::{Board, ChatCommand, ChatKind, ChatLease, Error, Session, Status, request_id};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Barrier},
    thread,
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("faction-chat-{}", request_id().unwrap())))
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct Fixture {
    board: Board,
    root: Session,
    alice: Session,
    bob: Session,
    secret: String,
    dir: Temp,
}
impl Fixture {
    fn new() -> Self {
        let dir = Temp::new();
        let mut board = Board::open(&dir.0).unwrap();
        let secret = request_id().unwrap();
        board.bootstrap("sysop", &secret).unwrap();
        let root = board.authenticate("sysop", &secret).unwrap();
        for name in ["alice", "bobby"] {
            board.register(name, &secret).unwrap();
            board.set_status(&root, name, Status::Approved).unwrap();
        }
        let alice = board.authenticate("alice", &secret).unwrap();
        let bob = board.authenticate("bobby", &secret).unwrap();
        Self {
            board,
            root,
            alice,
            bob,
            secret,
            dir,
        }
    }
    fn lease(&mut self, alice: bool) -> ChatLease {
        let lease = self
            .board
            .chat_open(if alice { &self.alice } else { &self.bob })
            .unwrap();
        self.board.chat_join(&lease, "#lobby").unwrap();
        lease
    }
    fn db(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(self.dir.0.join("faction.sqlite3")).unwrap()
    }
}
#[test]
fn pending_revoked_reapproved_and_foreign_leases_deny() {
    let mut f = Fixture::new();
    f.board.register("pending", &f.secret).unwrap();
    let p = f.board.authenticate("pending", &f.secret).unwrap();
    assert!(matches!(f.board.chat_open(&p), Err(Error::Denied)));
    let a = f.lease(true);
    let mut other = Fixture::new();
    assert!(matches!(other.board.chat_channels(&a), Err(Error::Denied)));
    assert!(matches!(other.board.chat_close(&a), Err(Error::Denied)));
    f.board
        .set_status(&f.root, "alice", Status::Revoked)
        .unwrap();
    assert!(matches!(
        f.board.chat_channels(&a),
        Err(Error::StaleSession)
    ));
    assert!(f.board.chat_join(&a, "#lobby").is_err());
    assert!(f.board.chat_read(&a, "#lobby", 0).is_err());
    assert!(
        f.board
            .chat_send(&a, "#lobby", 1, "hidden", ChatKind::Say)
            .is_err()
    );
    assert!(f.board.chat_names(&a, "#lobby", None).is_err());
    f.board
        .set_status(&f.root, "alice", Status::Approved)
        .unwrap();
    assert!(f.board.chat_channels(&a).is_err());
    f.board.chat_close(&a).unwrap();
    let fresh = f.board.authenticate("alice", &f.secret).unwrap();
    assert!(f.board.chat_open(&fresh).is_ok());
    assert!(f.board.chat_create(&f.bob, "#fake", false).is_err());
    assert!(f.board.chat_allow(&f.bob, "#lobby", "alice", true).is_err());
    assert!(f.board.chat_kick(&f.bob, "#lobby", "alice").is_err());
}
#[test]
fn two_connections_chronology_retry_and_control_bounds() {
    let mut f = Fixture::new();
    let a = f.lease(true);
    let mut other = Board::open(&f.dir.0).unwrap();
    let b = other.chat_open(&f.bob).unwrap();
    other.chat_join(&b, "#lobby").unwrap();
    let one = f
        .board
        .chat_send(&a, "#lobby", 1, "one ' bound SQL", ChatKind::Say)
        .unwrap();
    assert_eq!(one.sequence, 1);
    assert_eq!(
        f.board
            .chat_send(&a, "#lobby", 1, "one ' bound SQL", ChatKind::Say)
            .unwrap(),
        one
    );
    assert!(matches!(
        f.board.chat_send(&a, "#lobby", 1, "changed", ChatKind::Say),
        Err(Error::RequestConflict)
    ));
    let two = other
        .chat_send(&b, "#lobby", 1, "waves", ChatKind::Emote)
        .unwrap();
    assert_eq!(two.sequence, 2);
    let rows = f.board.chat_read(&a, "#lobby", 0).unwrap();
    assert_eq!(rows.messages, [one, two]);
    assert_eq!(rows.cursor, 2);
    assert!(!rows.gap);
    assert!(
        f.board
            .chat_send(&a, "#lobby", 3, "skipped", ChatKind::Say)
            .is_err()
    );
    assert!(
        f.board
            .chat_send(&a, "#lobby", 2, "\x1b]52;bad", ChatKind::Say)
            .is_err()
    );
    assert!(
        f.board
            .chat_send(&a, "#lobby", 2, &"x".repeat(513), ChatKind::Say)
            .is_err()
    );
    f.board
        .chat_send(&a, "#lobby", 2, "next", ChatKind::Say)
        .unwrap();
    assert!(
        f.board
            .chat_send(&a, "#lobby", 1, "one ' bound SQL", ChatKind::Say)
            .is_err()
    );
}
#[test]
fn private_channels_history_floor_and_regrant_are_current() {
    let mut f = Fixture::new();
    f.board.chat_create(&f.root, "#vault", true).unwrap();
    let admin = f.board.chat_open(&f.root).unwrap();
    f.board.chat_join(&admin, "#vault").unwrap();
    let a = f.lease(true);
    let b = f.lease(false);
    f.board
        .chat_send(&admin, "#vault", 1, "before grant", ChatKind::Say)
        .unwrap();
    assert!(
        !f.board
            .chat_channels(&a)
            .unwrap()
            .iter()
            .any(|c| c.name == "#vault")
    );
    assert!(f.board.chat_join(&a, "#vault").is_err());
    assert!(f.board.chat_read(&b, "#vault", 0).is_err());
    assert!(f.board.chat_names(&b, "#vault", None).is_err());
    f.board
        .chat_allow(&f.root, "#vault", "alice", true)
        .unwrap();
    assert!(f.board.chat_read(&a, "#vault", 0).is_err());
    f.board.chat_join(&a, "#vault").unwrap();
    assert!(
        f.board
            .chat_read(&a, "#vault", 0)
            .unwrap()
            .messages
            .is_empty()
    );
    f.board
        .chat_send(&admin, "#vault", 2, "after grant", ChatKind::Say)
        .unwrap();
    assert_eq!(
        f.board.chat_read(&a, "#vault", 0).unwrap().messages[0].body,
        "after grant"
    );
    f.board
        .chat_send(&a, "#vault", 1, "own private", ChatKind::Say)
        .unwrap();
    f.board
        .chat_allow(&f.root, "#vault", "alice", false)
        .unwrap();
    assert!(f.board.chat_read(&a, "#vault", 0).is_err());
    assert!(
        f.board
            .chat_send(&a, "#vault", 1, "own private", ChatKind::Say)
            .is_err()
    );
    f.board
        .chat_allow(&f.root, "#vault", "alice", true)
        .unwrap();
    assert!(f.board.chat_read(&a, "#vault", 0).is_err());
    f.board.chat_join(&a, "#vault").unwrap();
    assert!(
        f.board
            .chat_read(&a, "#vault", 0)
            .unwrap()
            .messages
            .is_empty()
    );
    assert!(
        f.board
            .chat_send(&a, "#vault", 1, "own private", ChatKind::Say)
            .is_err()
    );
    f.board.chat_kick(&f.root, "#vault", "alice").unwrap();
    assert!(f.board.chat_names(&a, "#vault", None).is_err());
    assert!(f.board.chat_join(&a, "#vault").is_ok());
}
#[test]
fn retention_and_lag_are_bounded_and_explicit() {
    let mut f = Fixture::new();
    let a = f.lease(true);
    let b = f.lease(false);
    let body = "x".repeat(512);
    for n in 1..=80 {
        f.board
            .chat_send(&a, "#lobby", n, &body, ChatKind::Say)
            .unwrap();
    }
    let db = f.db();
    assert_eq!(
        db.query_row("SELECT count(*) FROM chat_messages", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        64
    );
    let first = f.board.chat_read(&b, "#lobby", 0).unwrap();
    assert!(first.gap);
    assert_eq!(first.messages[0].sequence, 17);
    assert!(first.more);
    let mut count = first.messages.len();
    let mut cursor = first.cursor;
    while cursor < 80 {
        let page = f.board.chat_read(&b, "#lobby", cursor).unwrap();
        assert!(!page.gap);
        assert!(page.messages.len() <= 16);
        assert!(
            page.messages
                .iter()
                .map(|m| m.body.len() + m.author.len() + 96)
                .sum::<usize>()
                <= 2048
        );
        assert!(page.cursor > cursor);
        cursor = page.cursor;
        count += page.messages.len();
    }
    assert_eq!(count, 64);
    assert!(f.board.chat_read(&b, "#lobby", 81).is_err());
}
#[test]
fn presence_close_expiry_and_clock_rollback() {
    let mut f = Fixture::new();
    let a = f.lease(true);
    let same = f.lease(true);
    let b = f.lease(false);
    assert_eq!(
        f.board.chat_names(&b, "#lobby", None).unwrap().handles,
        ["alice", "bobby"]
    );
    f.board.chat_close(&a).unwrap();
    assert_eq!(
        f.board
            .chat_names(&b, "#lobby", None)
            .unwrap()
            .handles
            .len(),
        2
    );
    f.board.chat_close(&same).unwrap();
    assert_eq!(
        f.board.chat_names(&b, "#lobby", None).unwrap().handles,
        ["bobby"]
    );
    let db = f.db();
    db.execute("UPDATE chat_sessions SET touched=unixepoch()-61", [])
        .unwrap();
    assert!(matches!(
        f.board.chat_heartbeat(&b),
        Err(Error::StaleSession)
    ));
    f.board.chat_close(&b).unwrap();
    let future = f.lease(true);
    db.execute("UPDATE chat_sessions SET touched=unixepoch()+3600", [])
        .unwrap();
    assert!(f.board.chat_channels(&future).is_err());
    let fresh = f.lease(false);
    assert_eq!(
        f.board.chat_names(&fresh, "#lobby", None).unwrap().handles,
        ["bobby"]
    );
}
#[test]
fn channel_join_and_lease_caps() {
    let mut f = Fixture::new();
    let a = f.lease(true);
    for n in 1..32 {
        f.board
            .chat_create(&f.root, &format!("#chan{n}"), false)
            .unwrap();
    }
    assert!(f.board.chat_create(&f.root, "#overflow", false).is_err());
    for n in 1..8 {
        f.board.chat_join(&a, &format!("#chan{n}")).unwrap();
    }
    assert!(f.board.chat_join(&a, "#chan8").is_err());
    for _ in 1..256 {
        f.board.chat_open(&f.bob).unwrap();
    }
    assert!(f.board.chat_open(&f.bob).is_err());
    f.db()
        .execute("UPDATE chat_sessions SET touched=unixepoch()-61", [])
        .unwrap();
    assert!(f.board.chat_open(&f.bob).is_ok());
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM chat_sessions", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}
#[test]
fn concurrent_send_serializes_channel_sequence() {
    let mut f = Fixture::new();
    let a = f.lease(true);
    let b = f.lease(false);
    let other = Board::open(&f.dir.0).unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let threads: Vec<_> = [(f.board, a), (other, b)]
        .into_iter()
        .map(|(mut board, lease)| {
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                board
                    .chat_send(&lease, "#lobby", 1, "concurrent", ChatKind::Say)
                    .unwrap()
                    .sequence
            })
        })
        .collect();
    let mut results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    results.sort();
    assert_eq!(results, [1, 2]);
}
#[test]
fn schema_one_migration_preserves_identity_rooms_and_unread() {
    let mut f = Fixture::new();
    let post = f
        .board
        .post(&f.alice, 1, &request_id().unwrap(), "retained room", None)
        .unwrap();
    f.board.mark_read(&f.alice, 1, post.id).unwrap();
    let db = f.db();
    db.execute_batch("DROP TABLE visit_activity;DROP TABLE social_pages;DROP TABLE social_presence;DROP TABLE social_blocks;DROP TABLE social_prefs;DROP TABLE social_cooldowns;DROP TABLE social_settings;DROP TABLE magazine_entries;DROP TABLE magazine_issues;DROP TABLE magazine_submissions;DROP TABLE file_requests;DROP INDEX discussion_author_posts;DROP TABLE discussion_bookmarks;DROP TABLE discussion_settings;DROP TRIGGER catchup_file_insert; DROP TRIGGER catchup_file_publish; DROP TABLE catchup_favorites; DROP TABLE catchup_members; DROP TABLE catchup_settings; DROP TABLE catchup_publications; DROP TABLE poll_ballots; DROP TABLE poll_choices; DROP TABLE polls; DROP TABLE poll_settings; DROP TABLE gathering_rsvps; DROP TABLE gatherings; DROP TABLE gathering_settings; DROP TABLE member_plans; DROP TABLE member_pages; DROP TABLE page_settings; DROP TABLE wall_receipts; DROP TABLE wall_notes; DROP TABLE wall_settings; DROP TABLE room_drafts; DROP TABLE post_metadata; DROP TABLE file_namespace; DROP TABLE file_transfers; DROP TABLE file_objects; DROP TABLE file_area_grants; DROP TABLE file_areas; DROP TABLE game_scores; DROP TABLE chat_joins; DROP TABLE chat_messages; DROP TABLE chat_grants; DROP TABLE chat_sessions; DROP TABLE chat_channels; PRAGMA user_version=1;").unwrap();
    // A conflicting table fails the migration transaction without deleting data.
    db.execute_batch("CREATE TABLE chat_messages(unrelated TEXT);")
        .unwrap();
    assert!(Board::open(&f.dir.0).is_err());
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name='chat_channels'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );

    db.execute_batch("DROP TABLE chat_messages;").unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let threads: Vec<_> = (0..2)
        .map(|_| {
            let path = f.dir.0.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                Board::open(&path).unwrap()
            })
        })
        .collect();
    for t in threads {
        let mut board = t.join().unwrap();
        let s = board.authenticate("alice", &f.secret).unwrap();
        assert_eq!(board.posts(&s, 1, 0).unwrap()[0].body, "retained room");
        assert_eq!(board.unread_cursor(&s, 1).unwrap(), post.id);
        let chat = board.chat_open(&s).unwrap();
        assert_eq!(board.chat_channels(&chat).unwrap()[0].name, "#lobby");
    }
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        18
    );
}
#[test]
fn slash_commands_are_bounded_and_not_irc_wire_commands() {
    assert_eq!(
        ChatCommand::parse("/JOIN #local").unwrap(),
        ChatCommand::Join("#local")
    );
    assert_eq!(
        ChatCommand::parse("/me waves hello").unwrap(),
        ChatCommand::Emote("waves hello")
    );
    assert_eq!(
        ChatCommand::parse("//literal").unwrap(),
        ChatCommand::Say("/literal")
    );
    for bad in [
        "/recover",
        "/join #one #two",
        "/create #x public",
        "/quit extra",
        "/me",
        "/join #x\n/quit",
        "/allow #x alice extra",
    ] {
        assert!(ChatCommand::parse(bad).is_err());
    }
    assert!(ChatCommand::parse(&"x".repeat(769)).is_err());
}

#[test]
fn names_are_paginated_and_account_based() {
    let mut f = Fixture::new();
    let viewer = f.lease(false);
    for n in 0..34 {
        let name = format!("member{n:03}");
        f.board.register(&name, &f.secret).unwrap();
        f.board
            .set_status(&f.root, &name, Status::Approved)
            .unwrap();
        let s = f.board.authenticate(&name, &f.secret).unwrap();
        let lease = f.board.chat_open(&s).unwrap();
        f.board.chat_join(&lease, "#lobby").unwrap();
    }
    let first = f.board.chat_names(&viewer, "#lobby", None).unwrap();
    assert_eq!(first.handles.len(), 32);
    let second = f
        .board
        .chat_names(&viewer, "#lobby", first.next.as_deref())
        .unwrap();
    assert_eq!(second.handles.len(), 3);
    assert!(second.next.is_none());
    assert!(first.handles.last().unwrap() < second.handles.first().unwrap());
}
#[test]
fn channel_authority_rechecks_role_even_without_epoch_change() {
    let mut f = Fixture::new();
    f.board.chat_create(&f.root, "#vault", true).unwrap();
    let admin = f.board.chat_open(&f.root).unwrap();
    f.board.chat_join(&admin, "#vault").unwrap();
    f.board
        .chat_allow(&f.root, "#vault", "alice", true)
        .unwrap();
    let a = f.lease(true);
    f.board.chat_join(&a, "#vault").unwrap();
    assert_eq!(
        f.board.chat_names(&a, "#vault", None).unwrap().handles,
        ["alice", "sysop"]
    );
    f.db()
        .execute("UPDATE members SET role='member' WHERE handle='sysop'", [])
        .unwrap();
    assert!(f.board.chat_read(&admin, "#vault", 0).is_err());
    assert!(f.board.chat_create(&f.root, "#forged", false).is_err());
    assert_eq!(
        f.board.chat_names(&a, "#vault", None).unwrap().handles,
        ["alice"]
    );
}
