use faction_core::{
    Board, export_local,
    operator::{self, Snapshot},
    request_id, verify_export,
};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt, symlink},
    path::PathBuf,
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("faction-operator-{}", request_id().unwrap()));
        fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        Self(p)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn sample() -> Snapshot {
    Snapshot {
        sampled: 100,
        pid: 1,
        uptime: 5,
        running: true,
        connected: 1,
        capacity: 8,
        credentials: 0,
        output_bytes: 0,
        doors: 0,
        counts: Some([0; 10]),
    }
}
#[test]
fn aggregate_owner_snapshot_excludes_content_and_keeps_export_independent() {
    let f = Fixture::new();
    let state = f.0.join("board");
    let mut b = Board::open(&state).unwrap();
    let password = request_id().unwrap();
    b.bootstrap("private_sysop", &password).unwrap();
    b.register("pending_person", &password).unwrap();
    let mut s = sample();
    s.counts = b.operator_counts();
    let c = s.counts.unwrap();
    assert_eq!(&c[..6], &[2, 1, 1, 0, 1, 0]);
    operator::publish(&state, &s).unwrap();
    assert_eq!(operator::read(&state).unwrap(), s);
    let raw = fs::read_to_string(state.join(operator::STATUS_FILE)).unwrap();
    assert!(
        !raw.contains(&password)
            && !raw.contains("private_sysop")
            && !raw.contains("pending_person")
    );
    let backup = f.0.join("backup");
    export_local(&state, &backup).unwrap();
    assert_eq!(verify_export(&backup).unwrap().members, 2);
}
#[test]
fn unsafe_snapshot_links_modes_and_unbounded_input_are_denied() {
    let f = Fixture::new();
    let path = f.0.join(operator::STATUS_FILE);
    operator::publish(&f.0, &sample()).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(operator::read(&f.0).is_err());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let extra = f.0.join("hard");
    fs::hard_link(&path, &extra).unwrap();
    assert!(operator::read(&f.0).is_err());
    fs::remove_file(&extra).unwrap();
    fs::remove_file(&path).unwrap();
    symlink(&extra, &path).unwrap();
    assert!(operator::read(&f.0).is_err());
    fs::remove_file(&path).unwrap();
    operator::publish(&f.0, &sample()).unwrap();
    fs::write(&path, vec![b'x'; operator::STATUS_BYTES + 1]).unwrap();
    assert!(operator::read(&f.0).is_err());
    fs::set_permissions(&f.0, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(operator::read(&f.0).is_err());
}
#[test]
fn failed_publication_leaves_last_good_sample_readable() {
    let f = Fixture::new();
    let s = sample();
    operator::publish(&f.0, &s).unwrap();
    fs::write(f.0.join(".operator.status.next"), "occupied").unwrap();
    let mut next = s.clone();
    next.connected = 2;
    assert!(operator::publish(&f.0, &next).is_err());
    assert_eq!(operator::read(&f.0).unwrap(), s);
}
