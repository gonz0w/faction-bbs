use faction_core::{Board, Error, PAGE_BYTES, POST_BYTES, Role, Session, Status, request_id};
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
    sync::{Arc, Barrier},
    thread,
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("faction-core-{}", request_id().unwrap())))
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
        Self {
            board,
            root,
            secret,
            dir,
        }
    }
    fn member(&mut self) -> (Session, String) {
        let secret = request_id().unwrap();
        self.board.register("member", &secret).unwrap();
        self.board
            .set_status(&self.root, "member", Status::Approved)
            .unwrap();
        let s = self.board.authenticate("member", &secret).unwrap();
        (s, secret)
    }
}
#[test]
fn pending_cannot_access_community_or_admin() {
    let mut f = Fixture::new();
    let password = request_id().unwrap();
    f.board.register("pending", &password).unwrap();
    let s = f.board.authenticate("pending", &password).unwrap();
    assert_eq!(f.board.whoami(&s).unwrap().status, Status::Pending);
    assert!(matches!(f.board.rooms(&s), Err(Error::Denied)));
    assert!(matches!(f.board.posts(&s, 1, 0), Err(Error::Denied)));
    assert!(matches!(
        f.board.post(&s, 1, &request_id().unwrap(), "private", None),
        Err(Error::Denied)
    ));
    assert!(matches!(f.board.mark_read(&s, 1, 1), Err(Error::Denied)));
    assert!(matches!(f.board.members(&s), Err(Error::Denied)));
    assert!(matches!(
        f.board.create_room(&s, "forged", "forged"),
        Err(Error::Denied)
    ));
}
#[test]
fn stale_sessions_do_not_revive_after_reapproval() {
    let mut f = Fixture::new();
    let (session, secret) = f.member();
    let mut other = Board::open(&f.dir.0).unwrap();
    f.board
        .set_status(&f.root, "member", Status::Revoked)
        .unwrap();
    assert!(matches!(other.rooms(&session), Err(Error::StaleSession)));
    let revoked = other.authenticate("member", &secret).unwrap();
    assert!(matches!(other.rooms(&revoked), Err(Error::Denied)));
    f.board
        .set_status(&f.root, "member", Status::Approved)
        .unwrap();
    assert!(matches!(other.rooms(&session), Err(Error::StaleSession)));
    assert!(matches!(other.rooms(&revoked), Err(Error::StaleSession)));
    let fresh = other.authenticate("member", &secret).unwrap();
    assert_eq!(other.rooms(&fresh).unwrap().len(), 1);
}
#[test]
fn session_cannot_move_between_boards() {
    let a = Fixture::new();
    let mut b = Fixture::new();
    assert!(matches!(b.board.rooms(&a.root), Err(Error::Denied)));
    assert!(matches!(
        b.board.set_status(&a.root, "member", Status::Approved),
        Err(Error::Denied)
    ));
}
#[test]
fn member_cannot_manage_users_or_rooms() {
    let mut f = Fixture::new();
    let (s, _) = f.member();
    assert_eq!(f.board.whoami(&s).unwrap().role, Role::Member);
    assert!(matches!(
        f.board.set_status(&s, "member", Status::Approved),
        Err(Error::Denied)
    ));
    assert!(matches!(
        f.board.create_room(&s, "forged", "forged"),
        Err(Error::Denied)
    ));
    assert!(matches!(f.board.members(&s), Err(Error::Denied)));
    assert!(
        f.board
            .set_status(&f.root, "sysop", Status::Revoked)
            .is_err()
    );
}
#[test]
fn retry_reconcile_reply_and_sql_parameters() {
    let mut f = Fixture::new();
    let (s, _) = f.member();
    let key = request_id().unwrap();
    let body = "bound quote '); DROP TABLE posts; --";
    let p = f.board.post(&s, 1, &key, body, None).unwrap();
    assert_eq!(f.board.post(&s, 1, &key, body, None).unwrap(), p);
    assert!(matches!(
        f.board.post(&s, 1, &key, "changed", None),
        Err(Error::RequestConflict)
    ));
    let second = f.board.create_room(&f.root, "other", "Other").unwrap();
    assert!(
        f.board
            .post(&s, second, &request_id().unwrap(), "reply", Some(p.id))
            .is_err()
    );
    let reply = f
        .board
        .post(&s, 1, &request_id().unwrap(), "reply", Some(p.id))
        .unwrap();
    assert!(reply.id > p.id);
    let rows = f.board.posts(&s, 1, 0).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].body, body);
    assert_eq!(rows[1].reply_to, Some(p.id));
    f.board
        .set_status(&f.root, "member", Status::Revoked)
        .unwrap();
    assert!(matches!(
        f.board.post(&s, 1, &key, body, None),
        Err(Error::StaleSession)
    ));
}
#[test]
fn posts_unread_and_retry_survive_reopen() {
    let mut f = Fixture::new();
    let (s, secret) = f.member();
    let key = request_id().unwrap();
    let first = f.board.post(&s, 1, &key, "first", None).unwrap();
    let last = f
        .board
        .post(&s, 1, &request_id().unwrap(), "last", None)
        .unwrap();
    assert_eq!(f.board.rooms(&s).unwrap()[0].unread, 2);
    f.board.mark_read(&s, 1, last.id).unwrap();
    f.board.mark_read(&s, 1, first.id).unwrap();
    assert_eq!(f.board.unread_cursor(&s, 1).unwrap(), last.id);
    let mut reopened = Board::open(&f.dir.0).unwrap();
    let s = reopened.authenticate("member", &secret).unwrap();
    assert_eq!(reopened.rooms(&s).unwrap()[0].unread, 0);
    assert_eq!(
        reopened.post(&s, 1, &key, "first", None).unwrap().id,
        first.id
    );
    assert_eq!(reopened.posts(&s, 1, 0).unwrap().len(), 2);
}
#[test]
fn bounded_pages_and_control_rejection() {
    let mut f = Fixture::new();
    let (s, _) = f.member();
    assert!(
        f.board
            .post(&s, 1, &request_id().unwrap(), "\x1b[2J", None)
            .is_err()
    );
    assert!(
        f.board
            .post(
                &s,
                1,
                &request_id().unwrap(),
                &"a".repeat(POST_BYTES + 1),
                None
            )
            .is_err()
    );
    let body = "x".repeat(POST_BYTES);
    for _ in 0..3 {
        f.board
            .post(&s, 1, &request_id().unwrap(), &body, None)
            .unwrap();
    }
    let rows = f.board.posts(&s, 1, 0).unwrap();
    assert_eq!(rows.len(), 1);
    assert!(
        rows.iter()
            .map(|p| p.body.len() + p.author.len() + 64)
            .sum::<usize>()
            <= PAGE_BYTES
    );
    assert_eq!(f.board.posts(&s, 1, rows[0].id).unwrap().len(), 1);
}
#[test]
fn salted_hashes_and_owner_recovery() {
    let mut f = Fixture::new();
    f.board.register("other", &f.secret).unwrap();
    let db = rusqlite::Connection::open(f.dir.0.join("faction.sqlite3")).unwrap();
    let hashes: Vec<String> = db
        .prepare("SELECT password_hash FROM members")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert!(
        hashes
            .iter()
            .all(|h| h.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"))
    );
    assert!(hashes[0] != hashes[1]);
    assert!(hashes.iter().all(|h| !h.contains(&f.secret)));
    assert!(matches!(
        f.board.authenticate("sysop", &request_id().unwrap()),
        Err(Error::Credentials)
    ));
    assert!(matches!(
        f.board.authenticate("unknown", &f.secret),
        Err(Error::Credentials)
    ));
    let next = request_id().unwrap();
    f.board.recover_local_sysop("sysop", &next).unwrap();
    assert!(matches!(f.board.members(&f.root), Err(Error::StaleSession)));
    assert!(matches!(
        f.board.authenticate("sysop", &f.secret),
        Err(Error::Credentials)
    ));
    let fresh = f.board.authenticate("sysop", &next).unwrap();
    assert!(f.board.members(&fresh).is_ok());
}
#[test]
fn bootstrap_only_once_and_registration_never_squats_sysop() {
    let dir = Temp::new();
    let mut board = Board::open(&dir.0).unwrap();
    let secret = request_id().unwrap();
    assert!(matches!(
        board.register("sysop", &secret),
        Err(Error::NotInitialized)
    ));
    board.bootstrap("sysop", &secret).unwrap();
    assert!(matches!(
        board.bootstrap("another", &secret),
        Err(Error::AlreadyInitialized)
    ));
    assert!(board.register("SYSOP", &secret).is_err());
}
#[test]
fn concurrent_bootstrap_and_duplicate_submit_are_atomic() {
    let dir = Temp::new();
    let a = Board::open(&dir.0).unwrap();
    let b = Board::open(&dir.0).unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let secret = request_id().unwrap();
    let threads: Vec<_> = [a, b]
        .into_iter()
        .map(|mut board| {
            let barrier = barrier.clone();
            let secret = secret.clone();
            thread::spawn(move || {
                barrier.wait();
                board.bootstrap("sysop", &secret)
            })
        })
        .collect();
    let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert!(
        results
            .iter()
            .any(|r| matches!(r, Err(Error::AlreadyInitialized)))
    );
    let mut a = Board::open(&dir.0).unwrap();
    let mut b = Board::open(&dir.0).unwrap();
    let sa = a.authenticate("sysop", &secret).unwrap();
    let sb = b.authenticate("sysop", &secret).unwrap();
    let key = request_id().unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let threads: Vec<_> = [(a, sa), (b, sb)]
        .into_iter()
        .map(|(mut board, s)| {
            let barrier = barrier.clone();
            let key = key.clone();
            thread::spawn(move || {
                barrier.wait();
                board.post(&s, 1, &key, "retry", None)
            })
        })
        .collect();
    let results: Vec<_> = threads
        .into_iter()
        .map(|t| t.join().unwrap().unwrap())
        .collect();
    assert_eq!(results[0].id, results[1].id);
}
#[test]
fn role_is_checked_from_database_not_cached_session() {
    let mut f = Fixture::new();
    let db = rusqlite::Connection::open(f.dir.0.join("faction.sqlite3")).unwrap();
    db.execute("UPDATE members SET role='member' WHERE handle='sysop'", [])
        .unwrap();
    assert!(matches!(f.board.members(&f.root), Err(Error::Denied)));
}
#[test]
fn private_files_symlinks_and_future_schema() {
    let f = Fixture::new();
    assert_eq!(
        fs::metadata(&f.dir.0).unwrap().permissions().mode() & 0o777,
        0o700
    );
    for file in [
        "faction.sqlite3",
        "faction.sqlite3-wal",
        "faction.sqlite3-shm",
    ] {
        let path = f.dir.0.join(file);
        if path.exists() {
            assert_eq!(fs::metadata(path).unwrap().permissions().mode() & 0o077, 0);
        }
    }
    let link = Temp::new();
    symlink(&f.dir.0, &link.0).unwrap();
    assert!(matches!(Board::open(&link.0), Err(Error::UnsafeState)));
    fs::remove_file(&link.0).unwrap();
    let bad = Temp::new();
    fs::create_dir(&bad.0).unwrap();
    fs::set_permissions(&bad.0, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(matches!(Board::open(&bad.0), Err(Error::UnsafeState)));
    let db = rusqlite::Connection::open(f.dir.0.join("faction.sqlite3")).unwrap();
    db.pragma_update(None, "user_version", 3).unwrap();
    assert!(matches!(
        Board::open(&f.dir.0),
        Err(Error::UnsupportedSchema)
    ));
}
#[test]
fn foreign_database_rejected_before_write_configuration() {
    let dir = Temp::new();
    fs::create_dir(&dir.0).unwrap();
    fs::set_permissions(&dir.0, fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.0.join("faction.sqlite3");
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch(
        "CREATE TABLE foreign_data(value TEXT); INSERT INTO foreign_data VALUES('unchanged');",
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(matches!(Board::open(&dir.0), Err(Error::UnsupportedSchema)));
    assert_eq!(
        db.query_row("PRAGMA journal_mode", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "delete"
    );
    assert_eq!(
        db.query_row("SELECT value FROM foreign_data", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "unchanged"
    );
}

#[test]
fn database_and_sidecar_symlinks_are_rejected() {
    let dir = Temp::new();
    fs::create_dir(&dir.0).unwrap();
    fs::set_permissions(&dir.0, fs::Permissions::from_mode(0o700)).unwrap();
    let target = dir.0.join("unrelated");
    fs::write(&target, "untouched").unwrap();
    let database = dir.0.join("faction.sqlite3");
    symlink(&target, &database).unwrap();
    assert!(matches!(Board::open(&dir.0), Err(Error::UnsafeState)));
    fs::remove_file(&database).unwrap();
    let sidecar = dir.0.join("faction.sqlite3-wal");
    symlink(&target, &sidecar).unwrap();
    assert!(matches!(Board::open(&dir.0), Err(Error::UnsafeState)));
    assert_eq!(fs::read_to_string(&target).unwrap(), "untouched");
}
