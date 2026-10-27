use faction_core::{Board, request_id};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt, symlink},
    path::PathBuf,
};
struct Fixture {
    path: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        Self {
            path: std::env::temp_dir().canonicalize().unwrap().join(format!(
                "faction228-existing-OWNED-{}",
                request_id().unwrap()
            )),
        }
    }
    fn create(&self) {
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&self.path)
            .unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if self.path.exists() {
            fs::remove_dir_all(&self.path).unwrap();
        }
        assert!(!self.path.exists());
    }
}
#[test]
fn existing_state_never_initializes_or_migrates_missing_empty_or_old_state() {
    let f = Fixture::new();
    assert!(Board::open_existing(&f.path).is_err());
    assert!(!f.path.exists());
    f.create();
    assert!(Board::open_existing(&f.path).is_err());
    assert!(!f.path.join("faction.sqlite3").exists());
    let db_path = f.path.join("faction.sqlite3");
    fs::write(&db_path, []).unwrap();
    fs::set_permissions(&db_path, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(Board::open_existing(&f.path).is_err());
    assert_eq!(fs::metadata(&db_path).unwrap().len(), 0);
    fs::remove_file(&db_path).unwrap();
    drop(Board::open(&f.path).unwrap());
    let db = rusqlite::Connection::open(&db_path).unwrap();
    db.execute_batch("DROP TABLE roaming_chat; DROP TABLE roaming_transfers; DROP TABLE roaming_bindings; DROP TABLE roaming_issuers; PRAGMA user_version=19; PRAGMA wal_checkpoint(TRUNCATE)").unwrap();
    let before = fs::read(&db_path).unwrap();
    assert!(Board::open_existing(&f.path).is_err());
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        19
    );
    assert_eq!(fs::read(&db_path).unwrap(), before);
}
#[test]
fn existing_state_reads_current_schema_from_real_wal_after_explicit_migration() {
    let f = Fixture::new();
    drop(Board::open(&f.path).unwrap());
    let db_path = f.path.join("faction.sqlite3");
    let setup = rusqlite::Connection::open(&db_path).unwrap();
    setup.execute_batch("DROP TABLE roaming_chat; DROP TABLE roaming_transfers; DROP TABLE roaming_bindings; DROP TABLE roaming_issuers; PRAGMA user_version=19").unwrap();
    drop(setup);
    let holder = rusqlite::Connection::open(&db_path).unwrap();
    assert_eq!(
        holder
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        19
    );
    let migrated = Board::open(&f.path).unwrap();
    assert_eq!(&fs::read(&db_path).unwrap()[60..64], &19_u32.to_be_bytes());
    assert_eq!(
        holder
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        20
    );
    let existing = Board::open_existing(&f.path).unwrap();
    assert_eq!(
        existing.roaming_owner_state().unwrap().board_identity,
        migrated.roaming_owner_state().unwrap().board_identity
    );
    assert!(existing.roaming_owner_state().unwrap().bindings.is_empty());
}
#[test]
fn existing_state_rejects_unsafe_paths_and_unrecognized_application() {
    let f = Fixture::new();
    drop(Board::open(&f.path).unwrap());
    fs::set_permissions(&f.path, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(Board::open_existing(&f.path).is_err());
    fs::set_permissions(&f.path, fs::Permissions::from_mode(0o700)).unwrap();
    let alias = f.path.join("alias");
    symlink(&f.path, &alias).unwrap();
    assert!(Board::open_existing(&alias).is_err());
    let side = f.path.join("faction.sqlite3-wal");
    symlink(f.path.join("faction.sqlite3"), &side).unwrap();
    assert!(Board::open_existing(&f.path).is_err());
    fs::remove_file(side).unwrap();
    let db = rusqlite::Connection::open(f.path.join("faction.sqlite3")).unwrap();
    db.execute_batch("PRAGMA application_id=123").unwrap();
    assert!(Board::open_existing(&f.path).is_err());
    assert_eq!(
        db.query_row("PRAGMA application_id", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        123
    );
}
