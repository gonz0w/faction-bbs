use faction_core::{Board, Error, request_id};
use std::{
    fs::{self, File, OpenOptions},
    os::unix::fs::{OpenOptionsExt, PermissionsExt, symlink},
    path::PathBuf,
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("faction-kdf-{}", request_id().unwrap())))
    }
    fn lock(&self, i: usize) -> File {
        let f = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(self.0.join(format!("credential-{i}.lock")))
            .unwrap();
        rustix::fs::flock(&f, rustix::fs::FlockOperation::NonBlockingLockExclusive).unwrap();
        f
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn all_credential_paths_share_two_slots_and_release_on_return() {
    let t = Temp::new();
    let mut b = Board::open(&t.0).unwrap();
    let secret = request_id().unwrap();
    b.bootstrap("sysop", &secret).unwrap();
    let a = t.lock(0);
    let c = t.lock(1);
    assert!(matches!(
        b.authenticate("sysop", &secret),
        Err(Error::CredentialBusy)
    ));
    assert!(matches!(
        b.register("member", &secret),
        Err(Error::CredentialBusy)
    ));
    assert!(matches!(
        b.bootstrap("again", &secret),
        Err(Error::CredentialBusy)
    ));
    assert!(matches!(
        b.recover_local_sysop("sysop", &secret),
        Err(Error::CredentialBusy)
    ));
    drop(c);
    b.authenticate("sysop", &secret).unwrap();
    assert!(matches!(
        b.authenticate("sysop", "wrong password"),
        Err(Error::Credentials)
    ));
    b.authenticate("sysop", &secret).unwrap();
    b.register("member", &secret).unwrap();
    drop(a);
}
#[test]
fn credential_lock_symlinks_hardlinks_and_permissions_fail_closed() {
    let t = Temp::new();
    let mut b = Board::open(&t.0).unwrap();
    let secret = request_id().unwrap();
    b.bootstrap("sysop", &secret).unwrap();
    let path = t.0.join("credential-0.lock");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    assert!(matches!(
        b.authenticate("sysop", &secret),
        Err(Error::UnsafeState)
    ));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::hard_link(&path, t.0.join("extra")).unwrap();
    assert!(matches!(
        b.authenticate("sysop", &secret),
        Err(Error::UnsafeState)
    ));
    fs::remove_file(t.0.join("extra")).unwrap();
    fs::remove_file(&path).unwrap();
    symlink(t.0.join("faction.sqlite3"), &path).unwrap();
    assert!(matches!(
        b.authenticate("sysop", &secret),
        Err(Error::UnsafeState)
    ));
}
#[test]
fn unrelated_boards_do_not_share_credential_budget() {
    let a = Temp::new();
    let mut x = Board::open(&a.0).unwrap();
    let secret = request_id().unwrap();
    x.bootstrap("sysop", &secret).unwrap();
    let _l = a.lock(0);
    let _r = a.lock(1);
    let b = Temp::new();
    let mut y = Board::open(&b.0).unwrap();
    y.bootstrap("sysop", &secret).unwrap();
    y.authenticate("sysop", &secret).unwrap();
}

#[test]
fn stored_costs_cannot_request_more_memory_or_weaker_work() {
    let t = Temp::new();
    let mut b = Board::open(&t.0).unwrap();
    let secret = request_id().unwrap();
    b.bootstrap("sysop", &secret).unwrap();
    let db = rusqlite::Connection::open(t.0.join("faction.sqlite3")).unwrap();
    let original: String = db
        .query_row("SELECT password_hash FROM members", [], |r| r.get(0))
        .unwrap();
    let _left = t.lock(0);
    let _right = t.lock(1);
    for replacement in ["m=99999999,t=2,p=1", "m=19456,t=1,p=1", "m=19456,t=2,p=4"] {
        let changed = original.replace("m=19456,t=2,p=1", replacement);
        db.execute("UPDATE members SET password_hash=?1", [changed])
            .unwrap();
        // Credentials rather than CredentialBusy proves rejection before taking
        // any work slot or running the imported record's KDF parameters.
        assert!(matches!(
            b.authenticate("sysop", &secret),
            Err(Error::Credentials)
        ));
    }
}
