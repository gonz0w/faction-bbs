use faction_core::{
    Board, ChatKind, Error, SNAPSHOT_BYTES, Session, Status, export_local, request_id,
    restore_local, verify_export,
};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt, symlink},
    path::{Path, PathBuf},
    sync::{Arc, Barrier},
    thread,
};

struct Fixture {
    parent: PathBuf,
    board: Board,
    root: Session,
    secret: String,
}
impl Fixture {
    fn new() -> Self {
        let parent =
            std::env::temp_dir().join(format!("faction-snapshot-{}", request_id().unwrap()));
        fs::DirBuilder::new().mode(0o700).create(&parent).unwrap();
        let mut board = Board::open(&parent.join("source")).unwrap();
        let secret = request_id().unwrap();
        board.bootstrap("sysop", &secret).unwrap();
        let root = board.authenticate("sysop", &secret).unwrap();
        Self {
            parent,
            board,
            root,
            secret,
        }
    }
    fn path(&self, name: &str) -> PathBuf {
        self.parent.join(name)
    }
    fn export(&self) -> PathBuf {
        let out = self.path("export.sqlite3");
        self.board.export_local(&out).unwrap();
        out
    }
    fn mutate(path: &Path, sql: &str) {
        let db = rusqlite::Connection::open(path).unwrap();
        db.execute_batch(sql).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.parent);
    }
}
fn no_stage(f: &Fixture) {
    assert!(!fs::read_dir(&f.parent).unwrap().any(|p| {
        p.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".faction-stage-")
    }));
}

#[test]
fn wal_roundtrip_preserves_durable_records_and_drops_live_authority() {
    let mut f = Fixture::new();
    for name in ["alice", "pending", "revoked"] {
        f.board.register(name, &f.secret).unwrap();
    }
    f.board
        .set_status(&f.root, "alice", Status::Approved)
        .unwrap();
    f.board
        .set_status(&f.root, "revoked", Status::Revoked)
        .unwrap();
    let a = f.board.authenticate("alice", &f.secret).unwrap();
    let room = f
        .board
        .create_room(&f.root, "workshop", "Workshop")
        .unwrap();
    let req = request_id().unwrap();
    let post = f
        .board
        .post(&a, room, &req, "durable content", None)
        .unwrap();
    let reply = f
        .board
        .post(
            &f.root,
            room,
            &request_id().unwrap(),
            "reply",
            Some(post.id),
        )
        .unwrap();
    f.board.mark_read(&a, room, post.id).unwrap();
    f.board.chat_create(&f.root, "#private", true).unwrap();
    f.board
        .chat_allow(&f.root, "#private", "alice", true)
        .unwrap();
    let lease = f.board.chat_open(&a).unwrap();
    f.board.chat_join(&lease, "#private").unwrap();
    f.board
        .chat_send(
            &lease,
            "#private",
            1,
            "retained local history",
            ChatKind::Say,
        )
        .unwrap();
    assert!(
        f.path("source/faction.sqlite3-wal")
            .metadata()
            .unwrap()
            .len()
            > 0
    );
    let export = f.export();
    let before = fs::read(&export).unwrap();
    let summary = verify_export(&export).unwrap();
    assert_eq!(
        (
            summary.members,
            summary.rooms,
            summary.posts,
            summary.channels,
            summary.chat_messages
        ),
        (4, 2, 2, 2, 1)
    );
    // Taking a snapshot does not end the source console or change its leases.
    f.board.chat_heartbeat(&lease).unwrap();
    f.board
        .set_status(&f.root, "alice", Status::Revoked)
        .unwrap();
    let restored = f.path("restored");
    assert_eq!(restore_local(&export, &restored).unwrap(), summary);
    assert_eq!(fs::read(&export).unwrap(), before);
    assert_eq!(
        fs::metadata(&export).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(&restored).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(restored.join("faction.sqlite3"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let mut b = Board::open(&restored).unwrap();
    // Old in-memory room/chat capabilities cannot transplant, including a rollback
    // to an earlier account epoch. Only the new login can access restored content.
    assert!(matches!(b.rooms(&a), Err(Error::Denied)));
    assert!(b.chat_read(&lease, "#private", 0).is_err());
    let a2 = b.authenticate("alice", &f.secret).unwrap();
    let root2 = b.authenticate("sysop", &f.secret).unwrap();
    assert_eq!(b.posts(&a2, room, 0).unwrap()[1].reply_to, Some(post.id));
    assert_eq!(b.unread_cursor(&a2, room).unwrap(), post.id);
    assert_eq!(
        b.post(&a2, room, &req, "durable content", None).unwrap().id,
        post.id
    );
    assert!(
        b.post(&a2, room, &request_id().unwrap(), "after restore", None)
            .unwrap()
            .id
            > reply.id
    );
    let pending = b.authenticate("pending", &f.secret).unwrap();
    assert_eq!(b.whoami(&pending).unwrap().status, Status::Pending);
    let revoked = b.authenticate("revoked", &f.secret).unwrap();
    assert!(b.rooms(&revoked).is_err());
    let lease2 = b.chat_open(&a2).unwrap();
    b.chat_join(&lease2, "#private").unwrap();
    assert!(
        b.chat_read(&lease2, "#private", 0)
            .unwrap()
            .messages
            .is_empty()
    );
    assert_eq!(b.members(&root2).unwrap().len(), 4);
    let db = rusqlite::Connection::open(restored.join("faction.sqlite3")).unwrap();
    assert_eq!(
        db.query_row("SELECT body FROM chat_messages", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "retained local history"
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM chat_sessions", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    no_stage(&f);
}

#[test]
fn export_and_restore_never_overwrite_existing_paths() {
    let f = Fixture::new();
    let export = f.export();
    let before = fs::read(&export).unwrap();
    assert!(f.board.export_local(&export).is_err());
    assert_eq!(fs::read(&export).unwrap(), before);
    for name in ["empty", "populated"] {
        let target = f.path(name);
        fs::DirBuilder::new().mode(0o700).create(&target).unwrap();
        if name == "populated" {
            fs::write(target.join("sentinel"), b"keep").unwrap();
        }
        assert!(restore_local(&export, &target).is_err());
        assert!(target.is_dir());
        assert!(!target.join("faction.sqlite3").exists());
    }
    assert!(restore_local(&export, &f.path("source")).is_err());
    assert!(restore_local(&export, &export).is_err());
    no_stage(&f);
}

#[test]
fn reject_truncated_corrupt_foreign_future_and_executable_catalogs() {
    for mutation in [
        "PRAGMA application_id=123",
        "PRAGMA user_version=99",
        "UPDATE faction_export SET format=2", // requires bypassing CHECK in fixture
        "CREATE VIEW leak AS SELECT password_hash FROM members",
        "CREATE TRIGGER destroy AFTER UPDATE ON board BEGIN DELETE FROM posts; END",
        "DROP TABLE unread",
        "INSERT INTO unread VALUES(999,1,0)",
        "UPDATE members SET password_hash=replace(password_hash,'m=19456','m=999999999')",
        "UPDATE members SET handle='bad handle'",
        "INSERT INTO posts(room_id,author_id,request_id,body) VALUES(1,1,'0123456789abcdef',char(27)||'[2J')",
        "UPDATE members SET status='revoked'",
        "INSERT INTO chat_messages VALUES(1,1,1,'hello','say')", // tail remains zero
        "INSERT INTO chat_sessions(id,member,epoch,touched) VALUES(zeroblob(16),1,1,0)",
    ] {
        let f = Fixture::new();
        let export = f.export();
        Fixture::mutate(
            &export,
            &format!("PRAGMA foreign_keys=OFF; PRAGMA ignore_check_constraints=ON; {mutation}"),
        );
        let bytes = fs::read(&export).unwrap();
        assert!(verify_export(&export).is_err(), "accepted {mutation}");
        assert!(
            restore_local(&export, &f.path("restore")).is_err(),
            "restored {mutation}"
        );
        assert!(!f.path("restore").exists());
        assert_eq!(fs::read(&export).unwrap(), bytes);
        no_stage(&f);
    }
    for content in [
        b"not a database".as_slice(),
        b"SQLite format 3\0".as_slice(),
    ] {
        let f = Fixture::new();
        let export = f.export();
        fs::write(&export, content).unwrap();
        assert!(restore_local(&export, &f.path("restore")).is_err());
        assert!(!f.path("restore").exists());
    }
    let f = Fixture::new();
    let export = f.export();
    let mut data = fs::read(&export).unwrap();
    data[4096..8192].fill(0xff);
    fs::write(&export, data).unwrap();
    assert!(restore_local(&export, &f.path("restore")).is_err());
}

#[test]
fn reject_unsafe_files_parents_and_sidecars() {
    let f = Fixture::new();
    let export = f.export();
    let link = f.path("linked");
    symlink(&export, &link).unwrap();
    assert!(verify_export(&link).is_err());
    assert!(f.board.export_local(&link).is_err());
    let target = f.path("target-link");
    symlink(f.path("source"), &target).unwrap();
    assert!(restore_local(&export, &target).is_err());
    fs::set_permissions(&export, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(verify_export(&export).is_err());
    fs::set_permissions(&export, fs::Permissions::from_mode(0o600)).unwrap();
    for suffix in ["-wal", "-shm", "-journal"] {
        let side = f.path(&format!("export.sqlite3{suffix}"));
        fs::write(&side, b"").unwrap();
        assert!(verify_export(&export).is_err());
        fs::remove_file(side).unwrap();
    }
    let shared = f.path("shared");
    fs::create_dir(&shared).unwrap();
    fs::set_permissions(&shared, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(f.board.export_local(&shared.join("out")).is_err());
    assert!(restore_local(&export, &shared.join("restore")).is_err());
    no_stage(&f);
}

#[test]
fn size_cap_refuses_sparse_large_artifacts_without_allocating_them() {
    let f = Fixture::new();
    let export = f.export();
    fs::OpenOptions::new()
        .write(true)
        .open(&export)
        .unwrap()
        .set_len(SNAPSHOT_BYTES + 1)
        .unwrap();
    assert!(verify_export(&export).is_err());
    assert!(restore_local(&export, &f.path("restore")).is_err());
    assert!(!f.path("restore").exists());
    no_stage(&f);
}

#[test]
fn readonly_export_never_initializes_or_migrates_source() {
    let f = Fixture::new();
    let out = f.path("out");
    assert!(export_local(&f.path("missing"), &out).is_err());
    assert!(!f.path("missing").exists());
    assert!(!out.exists());
    let old = f.path("old");
    fs::DirBuilder::new().mode(0o700).create(&old).unwrap();
    let file = old.join("faction.sqlite3");
    fs::write(&file, b"").unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
    Fixture::mutate(
        &file,
        "PRAGMA application_id=1178682196; PRAGMA user_version=1; CREATE TABLE sentinel(id INTEGER)",
    );
    let bytes = fs::read(&file).unwrap();
    assert!(export_local(&old, &out).is_err());
    assert_eq!(fs::read(&file).unwrap(), bytes);
    assert!(!out.exists());
    let valid = f.path("read-only.sqlite3");
    export_local(&f.path("source"), &valid).unwrap();
    verify_export(&valid).unwrap();
    no_stage(&f);
}

#[test]
fn concurrent_restore_publish_has_one_winner_and_no_partial_database() {
    let f = Fixture::new();
    let export = f.export();
    let barrier = Arc::new(Barrier::new(3));
    let target = f.path("race");
    let mut jobs = Vec::new();
    for _ in 0..2 {
        let a = export.clone();
        let b = target.clone();
        let barrier = barrier.clone();
        jobs.push(thread::spawn(move || {
            barrier.wait();
            restore_local(&a, &b).is_ok()
        }));
    }
    barrier.wait();
    assert_eq!(
        jobs.into_iter()
            .map(|j| usize::from(j.join().unwrap()))
            .sum::<usize>(),
        1
    );
    let mut board = Board::open(&target).unwrap();
    assert!(board.authenticate("sysop", &f.secret).is_ok());
    no_stage(&f);
}

#[test]
fn snapshot_during_concurrent_wal_writes_is_consistent() {
    let mut f = Fixture::new();
    // Ensure the backup spans many page steps; keep payloads synthetic and bounded.
    for _ in 0..300 {
        f.board
            .post(&f.root, 1, &request_id().unwrap(), &"x".repeat(1024), None)
            .unwrap();
    }
    let barrier = Arc::new(Barrier::new(2));
    let b = barrier.clone();
    let source = f.path("source");
    let secret = f.secret.clone();
    let writer = thread::spawn(move || {
        let mut board = Board::open(&source).unwrap();
        let s = board.authenticate("sysop", &secret).unwrap();
        b.wait();
        for _ in 0..80 {
            board
                .post(&s, 1, &request_id().unwrap(), "concurrent", None)
                .unwrap();
        }
    });
    barrier.wait();
    let export = f.path("concurrent.sqlite3");
    export_local(&f.path("source"), &export).unwrap();
    writer.join().unwrap();
    let n = verify_export(&export).unwrap().posts;
    assert!((300..=380).contains(&n));
    restore_local(&export, &f.path("restore")).unwrap();
    no_stage(&f);
}

#[test]
fn exported_bytes_exclude_discarded_chat_receipts_and_freelist_content() {
    let mut f = Fixture::new();
    let a = f.board.chat_open(&f.root).unwrap();
    f.board.chat_join(&a, "#lobby").unwrap();
    let marker = format!("discarded-receipt-{}", request_id().unwrap());
    f.board
        .chat_send(&a, "#lobby", 1, &marker, ChatKind::Say)
        .unwrap();
    let b = f.board.chat_open(&f.root).unwrap();
    f.board.chat_join(&b, "#lobby").unwrap();
    for n in 1..=65 {
        f.board
            .chat_send(&b, "#lobby", n, "retained", ChatKind::Say)
            .unwrap();
    }
    let out = f.export();
    assert!(
        !fs::read(&out)
            .unwrap()
            .windows(marker.len())
            .any(|w| w == marker.as_bytes())
    );
    assert_eq!(verify_export(&out).unwrap().chat_messages, 64);
    f.board.chat_heartbeat(&a).unwrap();
    f.board.chat_heartbeat(&b).unwrap();
}

#[test]
fn busy_offline_source_fails_without_publishing_then_recovers() {
    let f = Fixture::new();
    let exported = f.export();
    let offline = f.path("offline");
    restore_local(&exported, &offline).unwrap();
    let output = f.path("busy.sqlite3");
    let db = rusqlite::Connection::open(offline.join("faction.sqlite3")).unwrap();
    db.execute_batch("PRAGMA journal_mode=DELETE; BEGIN EXCLUSIVE")
        .unwrap();
    assert!(export_local(&offline, &output).is_err());
    assert!(!output.exists());
    no_stage(&f);
    db.execute_batch("ROLLBACK").unwrap();
    drop(db);
    export_local(&offline, &output).unwrap();
    verify_export(&output).unwrap();
    no_stage(&f);
}
