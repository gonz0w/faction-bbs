#![cfg(feature = "durable-policy")]
use faction_files::private_blobs::Principal;
use faction_provider::{
    Runtime,
    authority::{CredentialBinding, Registry},
};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "faction283-owned-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        fs::write(p.join("OWNED"), b"issue283 synthetic authority").unwrap();
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
            b"issue283 synthetic authority"
        );
        fs::remove_dir_all(&self.0).unwrap();
        assert!(!self.0.exists());
    }
}
fn entry(n: u8, generation: u64) -> CredentialBinding {
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
        generation,
    }
}
fn registry(g: u64, r: u64, entries: Vec<CredentialBinding>) -> Registry {
    Registry::new(g, r, vec!["home.test".into()], entries).unwrap()
}
#[test]
fn explicit_init_reopen_and_no_bootstrap() {
    let f = Fixture::new();
    assert!(Runtime::reopen_durable(&f.policy()).is_err());
    let rt = Runtime::initialize_durable(&f.policy(), registry(1, 0, vec![entry(1, 1)])).unwrap();
    assert!(Runtime::reopen_durable(&f.policy()).is_err());
    assert!(Runtime::initialize_durable(&f.policy(), registry(1, 0, vec![])).is_err());
    assert_eq!(rt.policy_summary().unwrap().generation, 1);
    rt.stop();
    drop(rt);
    let rt = Runtime::reopen_durable(&f.policy()).unwrap();
    let s = rt.policy_summary().unwrap();
    assert!(s.available);
    assert_eq!(s.binding_count, 1);
    assert_eq!(s.history_count, 1);
    assert_eq!(
        fs::metadata(f.policy().join("provider-policy.lock"))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(fs::read_dir(f.policy()).unwrap().count(), 2);
}
#[test]
fn removal_history_reset_and_exact_reapproval_survive_reopen() {
    let f = Fixture::new();
    let rt = Runtime::initialize_durable(&f.policy(), registry(1, 0, vec![entry(1, 1)])).unwrap();
    let old = rt.control();
    old.replace_registry(registry(2, 0, vec![])).unwrap();
    drop(rt);
    let rt = Runtime::reopen_durable(&f.policy()).unwrap();
    assert!(old.reset(9).is_err());
    let mut remapped = entry(2, 3);
    remapped.certificate_sha256 = [1; 32];
    assert!(
        rt.control()
            .replace_registry(registry(3, 0, vec![remapped]))
            .is_err()
    );
    assert_eq!(rt.policy_summary().unwrap().generation, 2);
    rt.control()
        .replace_registry(registry(3, 0, vec![entry(1, 3)]))
        .unwrap();
    rt.control().reset(7).unwrap();
    drop(rt);
    let rt = Runtime::reopen_durable(&f.policy()).unwrap();
    let s = rt.policy_summary().unwrap();
    assert_eq!(
        (
            s.generation,
            s.reset_epoch,
            s.binding_count,
            s.history_count
        ),
        (4, 7, 0, 1)
    );
    assert!(
        rt.control()
            .replace_registry(registry(5, 6, vec![entry(1, 5)]))
            .is_err()
    );
    rt.control().unavailable().unwrap();
    drop(rt);
    let rt = Runtime::reopen_durable(&f.policy()).unwrap();
    assert!(!rt.policy_summary().unwrap().available);
    rt.control()
        .replace_registry(registry(5, 7, vec![entry(1, 5)]))
        .unwrap();
}
#[test]
fn lifetime_history_cap_and_invalid_proposals_preserve_bytes() {
    let f = Fixture::new();
    let rt = Runtime::initialize_durable(
        &f.policy(),
        registry(1, 0, (1..=32).map(|n| entry(n, 1)).collect()),
    )
    .unwrap();
    rt.control()
        .replace_registry(registry(2, 0, vec![]))
        .unwrap();
    let before = fs::read(f.policy().join("provider-policy.snapshot")).unwrap();
    assert!(
        rt.control()
            .replace_registry(registry(3, 0, vec![entry(33, 3)]))
            .is_err()
    );
    assert!(
        rt.control()
            .replace_registry(registry(2, 0, vec![entry(1, 2)]))
            .is_err()
    );
    assert_eq!(
        before,
        fs::read(f.policy().join("provider-policy.snapshot")).unwrap()
    );
    drop(rt);
    let rt = Runtime::reopen_durable(&f.policy()).unwrap();
    assert_eq!(rt.policy_summary().unwrap().history_count, 32);
}
#[test]
fn missing_corrupt_unknown_truncated_and_oversized_never_recover_from_pending() {
    for bytes in [vec![], vec![1; 12], vec![0; 65537], b"FPPY9999".to_vec()] {
        let f = Fixture::new();
        let rt =
            Runtime::initialize_durable(&f.policy(), registry(1, 0, vec![entry(1, 1)])).unwrap();
        drop(rt);
        fs::copy(
            f.policy().join("provider-policy.snapshot"),
            f.policy().join("provider-policy.pending"),
        )
        .unwrap();
        fs::write(f.policy().join("provider-policy.snapshot"), &bytes).unwrap();
        assert!(Runtime::reopen_durable(&f.policy()).is_err());
    }
    let f = Fixture::new();
    let rt = Runtime::initialize_durable(&f.policy(), registry(1, 0, vec![])).unwrap();
    drop(rt);
    fs::remove_file(f.policy().join("provider-policy.snapshot")).unwrap();
    assert!(Runtime::reopen_durable(&f.policy()).is_err());
    assert!(Runtime::initialize_durable(&f.policy(), registry(1, 0, vec![])).is_err());
}
#[test]
fn unsafe_paths_modes_links_types_and_unrelated_entries_refused() {
    for case in 0..8 {
        let f = Fixture::new();
        let rt = Runtime::initialize_durable(&f.policy(), registry(1, 0, vec![])).unwrap();
        drop(rt);
        let s = f.policy().join("provider-policy.snapshot");
        match case {
            0 => fs::set_permissions(f.policy(), fs::Permissions::from_mode(0o755)).unwrap(),
            1 => fs::set_permissions(&s, fs::Permissions::from_mode(0o644)).unwrap(),
            2 => fs::hard_link(&s, f.0.join("hardlink")).unwrap(),
            3 => {
                fs::remove_file(&s).unwrap();
                std::os::unix::fs::symlink(f.0.join("OWNED"), &s).unwrap();
            }
            4 => {
                fs::remove_file(&s).unwrap();
                fs::create_dir(&s).unwrap();
            }
            5 => fs::write(f.policy().join("unrelated"), b"refuse").unwrap(),
            6 => fs::write(f.policy().join("provider-policy.lock"), b"not zero").unwrap(),
            _ => {
                fs::remove_file(f.policy().join("provider-policy.lock")).unwrap();
                std::os::unix::fs::symlink(
                    f.0.join("OWNED"),
                    f.policy().join("provider-policy.lock"),
                )
                .unwrap();
            }
        }
        assert!(Runtime::reopen_durable(&f.policy()).is_err(), "case {case}");
    }
    let f = Fixture::new();
    std::os::unix::fs::symlink(f.policy(), f.0.join("alias")).unwrap();
    assert!(Runtime::initialize_durable(&f.0.join("alias"), registry(1, 0, vec![])).is_err());
}
#[test]
fn initialization_requires_an_empty_root_and_reopen_requires_original_lock() {
    let f = Fixture::new();
    fs::write(f.policy().join("unrelated"), b"do not touch").unwrap();
    assert!(Runtime::initialize_durable(&f.policy(), registry(1, 0, vec![])).is_err());
    assert_eq!(fs::read_dir(f.policy()).unwrap().count(), 1);
    let f = Fixture::new();
    let rt = Runtime::initialize_durable(&f.policy(), registry(1, 0, vec![])).unwrap();
    drop(rt);
    fs::remove_file(f.policy().join("provider-policy.lock")).unwrap();
    assert!(Runtime::reopen_durable(&f.policy()).is_err());
    assert!(!f.policy().join("provider-policy.lock").exists());
}
