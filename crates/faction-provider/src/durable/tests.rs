use super::{
    disk::{FAULT, Fault, Point},
    *,
};
use crate::{
    Runtime,
    authority::{CredentialBinding, Registry},
};
use faction_files::private_blobs::{Error as StoreError, Principal};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "faction283-unit-owned-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        fs::write(p.join("OWNED"), b"issue283 unit synthetic authority").unwrap();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(p.join("policy"))
            .unwrap();
        Self(p)
    }
    fn policy(&self) -> PathBuf {
        self.0.join("policy")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert_eq!(
            fs::read(self.0.join("OWNED")).unwrap(),
            b"issue283 unit synthetic authority"
        );
        fs::remove_dir_all(&self.0).unwrap();
        assert!(!self.0.exists());
    }
}
fn entry(n: u8, g: u64) -> CredentialBinding {
    CredentialBinding {
        certificate_sha256: [n; 32],
        principal: Principal {
            account: format!("member{n}@home.test"),
            device: n.into(),
            identity: [n; 32],
        },
        valid_from: 1,
        valid_until: 10000,
        approved: true,
        account_enabled: true,
        device_enabled: true,
        generation: g,
    }
}
fn registry(g: u64) -> Registry {
    Registry::new(g, 0, vec!["home.test".into()], vec![entry(1, g)]).unwrap()
}
const POINTS: [Point; 5] = [
    Point::Write,
    Point::FileSync,
    Point::BeforeRename,
    Point::AfterRename,
    Point::DirectorySync,
];
#[test]
fn io_failure_stops_all_current_controls_and_authorization_until_reopen() {
    for point in POINTS {
        let f = Fixture::new();
        let rt = Runtime::initialize_durable(&f.policy(), registry(1)).unwrap();
        let (connection, handle) = {
            let mut a = rt.state.authority.lock().unwrap();
            let c = a.open_connection();
            let h = a.authenticate(&c, [1; 32], 2, 1).unwrap();
            (c, h)
        };
        FAULT.set(Fault::Error(point));
        assert!(rt.control().replace_registry(registry(2)).is_err());
        FAULT.set(Fault::None);
        assert!(rt.control().replace_registry(registry(3)).is_err());
        assert!(rt.control().reset(1).is_err());
        assert!(
            rt.state
                .authority
                .lock()
                .unwrap()
                .authenticate(&connection, [1; 32], 2, 2)
                .is_err()
        );
        fs::DirBuilder::new()
            .mode(0o700)
            .create(f.0.join("blobs"))
            .unwrap();
        let mut store = faction_files::private_blobs::Store::open(&f.0.join("blobs")).unwrap();
        let before = fs::read(f.0.join("blobs/private-blobs.snapshot")).unwrap();
        assert!(
            rt.state
                .authority
                .lock()
                .unwrap()
                .dispatch(
                    &connection,
                    &handle,
                    &mut store,
                    faction_files::private_blobs::Request::Begin(
                        faction_files::private_blobs::Binding {
                            sender: entry(1, 1).principal,
                            recipient: entry(1, 1).principal,
                            id: [1; 32],
                            size: 16,
                            created: 2,
                            expires: 100
                        }
                    ),
                    2,
                    2
                )
                .is_err()
        );
        assert_eq!(
            before,
            fs::read(f.0.join("blobs/private-blobs.snapshot")).unwrap()
        );
        drop(handle);
        drop(connection);
        drop(rt);
        let rt = Runtime::reopen_durable(&f.policy()).unwrap();
        assert!(rt.policy_summary().unwrap().available);
        assert_eq!(
            rt.policy_summary().unwrap().generation,
            if matches!(point, Point::AfterRename | Point::DirectorySync) {
                2
            } else {
                1
            }
        );
    }
}
#[test]
fn codec_roundtrip_rejects_every_truncation_duplicates_bad_flags_and_unknown_versions() {
    let a = crate::authority::Authority::new(Some(registry(1)));
    let s = a.snapshot();
    let b = codec::encode(&s).unwrap();
    assert!(b.len() <= MAX_SNAPSHOT);
    assert_eq!(codec::encode(&codec::decode(&b).unwrap()).unwrap(), b);
    for end in 0..b.len() {
        assert!(codec::decode(&b[..end]).is_err(), "prefix {end}");
    }
    let mut bad = b.clone();
    bad[7] = b'9';
    assert!(codec::decode(&bad).is_err());
    let mut bad = b.clone();
    bad.push(0);
    assert!(codec::decode(&bad).is_err());
    let mut bad = b.clone();
    bad[24] = 2;
    assert!(codec::decode(&bad).is_err());
    assert!(codec::decode(&vec![0; MAX_SNAPSHOT + 1]).is_err());
    let mut bad = s.clone();
    bad.history.push(bad.history[0].clone());
    assert!(codec::encode(&bad).is_err());
    let mut bad = s.clone();
    bad.history[0].1.identity = [3; 32];
    assert!(codec::encode(&bad).is_err());
    let mut bad = s.clone();
    bad.generation = 0;
    assert!(codec::encode(&bad).is_err());
    let mut bad = s.clone();
    bad.reset_epoch = 1;
    assert!(codec::encode(&bad).is_err());
}
#[test]
fn fresh_authority_rejects_old_connection_and_handles_without_store_mutation() {
    let f = Fixture::new();
    let rt = Runtime::initialize_durable(&f.policy(), registry(1)).unwrap();
    let (c, h) = {
        let mut a = rt.state.authority.lock().unwrap();
        let c = a.open_connection();
        let h = a.authenticate(&c, [1; 32], 2, 1).unwrap();
        (c, h)
    };
    drop(rt);
    let rt = Runtime::reopen_durable(&f.policy()).unwrap();
    let mut a = rt.state.authority.lock().unwrap();
    assert!(matches!(
        a.authenticate(&c, [1; 32], 2, 2),
        Err(StoreError::Denied)
    ));
    let fresh = a.open_connection();
    assert!(a.authenticate(&fresh, [1; 32], 2, 2).is_ok());
    fs::DirBuilder::new()
        .mode(0o700)
        .create(f.0.join("blobs"))
        .unwrap();
    let mut store = faction_files::private_blobs::Store::open(&f.0.join("blobs")).unwrap();
    let before = fs::read(f.0.join("blobs/private-blobs.snapshot")).unwrap();
    assert!(
        a.dispatch(
            &c,
            &h,
            &mut store,
            faction_files::private_blobs::Request::Status { id: [1; 32] },
            2,
            3
        )
        .is_err()
    );
    assert_eq!(
        before,
        fs::read(f.0.join("blobs/private-blobs.snapshot")).unwrap()
    );
}
#[test]
fn death_child() {
    let Ok(root) = std::env::var("FACTION283_DEATH_ROOT") else {
        return;
    };
    let point: usize = std::env::var("FACTION283_DEATH_POINT")
        .unwrap()
        .parse()
        .unwrap();
    let rt = Runtime::reopen_durable(&PathBuf::from(root).join("policy")).unwrap();
    FAULT.set(Fault::Death(POINTS[point]));
    let _ = rt.control().replace_registry(registry(2));
    panic!("death boundary was not reached");
}
#[test]
fn actual_process_death_preserves_complete_old_or_new_publication_and_lock_inode() {
    use std::os::unix::fs::MetadataExt;
    for (index, point) in POINTS.into_iter().enumerate() {
        let f = Fixture::new();
        let rt = Runtime::initialize_durable(&f.policy(), registry(1)).unwrap();
        drop(rt);
        let lock = f.policy().join("provider-policy.lock");
        let inode = fs::metadata(&lock).unwrap().ino();
        let log = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(f.0.join("child.log"))
            .unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "durable::tests::death_child", "--nocapture"])
            .env("FACTION283_DEATH_ROOT", &f.0)
            .env("FACTION283_DEATH_POINT", index.to_string())
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .spawn()
            .unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        while !f.0.join("BOUNDARY").exists() && Instant::now() < end {
            if let Some(status) = child.try_wait().unwrap() {
                panic!(
                    "child exited {status}: {}",
                    fs::read_to_string(f.0.join("child.log")).unwrap()
                );
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        if !f.0.join("BOUNDARY").exists() {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("death boundary timeout");
        }
        assert!(Runtime::reopen_durable(&f.policy()).is_err());
        child.kill().unwrap();
        assert!(!child.wait().unwrap().success());
        let rt = Runtime::reopen_durable(&f.policy()).unwrap();
        assert_eq!(
            rt.policy_summary().unwrap().generation,
            if matches!(point, Point::AfterRename | Point::DirectorySync) {
                2
            } else {
                1
            }
        );
        assert_eq!(fs::metadata(&lock).unwrap().ino(), inode);
        assert_eq!(fs::read_dir(f.policy()).unwrap().count(), 2);
    }
}
#[test]
fn every_persisted_approval_gate_and_current_rightful_positive_survive_reopen() {
    for disabled in 0..4 {
        let f = Fixture::new();
        let mut denied = entry(1, 1);
        match disabled {
            0 => denied.approved = false,
            1 => denied.account_enabled = false,
            2 => denied.device_enabled = false,
            _ => denied.valid_until = 2,
        };
        let registry =
            Registry::new(1, 0, vec!["home.test".into()], vec![denied, entry(2, 1)]).unwrap();
        let rt = Runtime::initialize_durable(&f.policy(), registry).unwrap();
        drop(rt);
        let rt = Runtime::reopen_durable(&f.policy()).unwrap();
        let mut a = rt.state.authority.lock().unwrap();
        let c = a.open_connection();
        assert!(a.authenticate(&c, [1; 32], 2, 1).is_err());
        assert!(a.authenticate(&c, [2; 32], 2, 1).is_ok());
    }
}
#[test]
fn invalid_typed_proposal_keeps_existing_live_authorization_and_snapshot() {
    let f = Fixture::new();
    let rt = Runtime::initialize_durable(&f.policy(), registry(1)).unwrap();
    let (c, h) = {
        let mut a = rt.state.authority.lock().unwrap();
        let c = a.open_connection();
        let h = a.authenticate(&c, [1; 32], 2, 1).unwrap();
        (c, h)
    };
    let before = fs::read(f.policy().join("provider-policy.snapshot")).unwrap();
    assert!(rt.control().replace_registry(registry(1)).is_err());
    assert_eq!(
        before,
        fs::read(f.policy().join("provider-policy.snapshot")).unwrap()
    );
    assert!(!rt.state.stopped.load(Ordering::SeqCst));
    fs::DirBuilder::new()
        .mode(0o700)
        .create(f.0.join("blobs"))
        .unwrap();
    let mut store = faction_files::private_blobs::Store::open(&f.0.join("blobs")).unwrap();
    rt.state
        .authority
        .lock()
        .unwrap()
        .dispatch(
            &c,
            &h,
            &mut store,
            faction_files::private_blobs::Request::Begin(faction_files::private_blobs::Binding {
                sender: entry(1, 1).principal,
                recipient: entry(1, 1).principal,
                id: [1; 32],
                size: 16,
                created: 2,
                expires: 100,
            }),
            2,
            2,
        )
        .unwrap();
    drop(c);
    let mut a = rt.state.authority.lock().unwrap();
    let c = a.open_connection();
    assert!(a.authenticate(&c, [1; 32], 2, 2).is_ok());
}
#[test]
fn home_consistency_includes_all_binding_domains_and_unavailability() {
    let f = Fixture::new();
    let rt = Runtime::initialize_durable(&f.policy(), registry(1)).unwrap();
    assert!(rt.policy_matches_home("home.test").unwrap());
    assert!(!rt.policy_matches_home("other.test").unwrap());
    let mut foreign = entry(2, 2);
    foreign.principal.account = "member2@foreign.test".into();
    let r = Registry::new(2, 0, vec!["home.test".into()], vec![entry(1, 2), foreign]).unwrap();
    rt.control().replace_registry(r).unwrap();
    assert!(!rt.policy_matches_home("home.test").unwrap());
    rt.control().unavailable().unwrap();
    assert!(!rt.policy_matches_home("home.test").unwrap());
}
#[test]
fn exhausted_generation_and_reset_floors_refuse_without_mutation() {
    let f = Fixture::new();
    let r = Registry::new(
        u64::MAX,
        u64::MAX,
        vec!["home.test".into()],
        vec![entry(1, u64::MAX)],
    )
    .unwrap();
    let rt = Runtime::initialize_durable(&f.policy(), r).unwrap();
    let before = fs::read(f.policy().join("provider-policy.snapshot")).unwrap();
    assert!(rt.control().reset(u64::MAX).is_err());
    assert!(rt.control().reset(1).is_err());
    assert!(rt.control().replace_registry(registry(1)).is_err());
    assert_eq!(
        before,
        fs::read(f.policy().join("provider-policy.snapshot")).unwrap()
    );
    drop(rt);
    let rt = Runtime::reopen_durable(&f.policy()).unwrap();
    assert_eq!(
        (
            rt.policy_summary().unwrap().generation,
            rt.policy_summary().unwrap().reset_epoch
        ),
        (u64::MAX, u64::MAX)
    );
    let f = Fixture::new();
    let r = Registry::new(u64::MAX, 0, vec!["home.test".into()], vec![]).unwrap();
    let rt = Runtime::initialize_durable(&f.policy(), r).unwrap();
    assert!(rt.control().reset(1).is_err());
    assert_eq!(rt.policy_summary().unwrap().reset_epoch, 0);
}
#[test]
fn decoded_duplicate_and_excessive_history_are_refused() {
    let s = crate::authority::Authority::new(Some(registry(1))).snapshot();
    let bytes = codec::encode(&s).unwrap();
    let first_length = 32 + 2 + s.history[0].1.account.len() + 4 + 32;
    let mut duplicate = bytes.clone();
    duplicate[25] = 2;
    duplicate.splice(
        26 + first_length..26 + first_length,
        bytes[26..26 + first_length].iter().copied(),
    );
    assert!(codec::decode(&duplicate).is_err());
    let mut excessive = bytes;
    excessive[25] = 33;
    assert!(codec::decode(&excessive).is_err());
}
#[test]
fn changed_live_root_or_missing_established_snapshot_fails_closed_before_publication() {
    use std::os::unix::fs::PermissionsExt;
    for case in 0..3 {
        let f = Fixture::new();
        let rt = Runtime::initialize_durable(&f.policy(), registry(1)).unwrap();
        match case {
            0 => fs::remove_file(f.policy().join("provider-policy.snapshot")).unwrap(),
            1 => fs::set_permissions(f.policy(), fs::Permissions::from_mode(0o755)).unwrap(),
            _ => fs::rename(
                f.policy().join("provider-policy.lock"),
                f.0.join("original-lock"),
            )
            .unwrap(),
        }
        assert!(
            rt.control().replace_registry(registry(2)).is_err(),
            "case {case}"
        );
        assert!(rt.control().reset(1).is_err());
        if case == 0 {
            assert!(!f.policy().join("provider-policy.snapshot").exists());
        } else if case == 1 {
            fs::set_permissions(f.policy(), fs::Permissions::from_mode(0o700)).unwrap();
        }
    }
}
#[cfg(feature = "delegation-services")]
#[path = "service_tests.rs"]
mod service_tests;
