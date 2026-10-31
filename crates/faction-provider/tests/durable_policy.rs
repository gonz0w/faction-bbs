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
#[test]
fn service_bearing_v2_is_preserved_or_refused_by_the_actual_feature_build() {
    let f = Fixture::new();
    let r = Runtime::initialize_durable(
        &f.policy(),
        Registry::new(
            1,
            0,
            vec!["home.test".into(), "foreign.test".into()],
            vec![entry(1, 1)],
        )
        .unwrap(),
    )
    .unwrap();
    drop(r);
    let current = f.policy().join("provider-policy.snapshot");
    let v1 = fs::read(&current).unwrap();
    assert_eq!(&v1[..8], b"FPPY0001");
    let mut bytes = b"FPPY0002".to_vec();
    bytes.extend((v1.len() as u32).to_be_bytes());
    bytes.extend(v1);
    fn string(b: &mut Vec<u8>, s: &str) {
        b.extend((s.len() as u16).to_be_bytes());
        b.extend(s.as_bytes());
    }
    bytes.push(1);
    bytes.extend([44; 32]);
    string(&mut bytes, "foreign.test");
    string(&mut bytes, "client");
    let policy_offset = bytes.len();
    bytes.push(1);
    string(&mut bytes, "home.test");
    bytes.push(1);
    string(&mut bytes, "foreign.test");
    string(&mut bytes, "client");
    string(&mut bytes, "service.test");
    bytes.extend([44; 32]);
    bytes.push(1);
    bytes.extend(1u64.to_be_bytes());
    bytes.extend(10000u64.to_be_bytes());
    fs::write(&current, &bytes).unwrap();
    #[cfg(feature = "delegation-services")]
    {
        let r = Runtime::reopen_durable(&f.policy()).unwrap();
        assert_eq!(r.policy_summary().unwrap().binding_count, 2);
        assert_eq!(r.policy_summary().unwrap().history_count, 2);
    }
    #[cfg(not(feature = "delegation-services"))]
    assert!(Runtime::reopen_durable(&f.policy()).is_err());
    assert_eq!(fs::read(&current).unwrap(), bytes);
    // A device-only update/reset may leave only typed service tombstones. An
    // unsupported build must not reopen that v2 and silently lose their history.
    bytes.truncate(policy_offset);
    bytes.push(0);
    fs::write(&current, &bytes).unwrap();
    #[cfg(feature = "delegation-services")]
    {
        let r = Runtime::reopen_durable(&f.policy()).unwrap();
        assert_eq!(r.policy_summary().unwrap().binding_count, 1);
        assert_eq!(r.policy_summary().unwrap().history_count, 2);
    }
    #[cfg(not(feature = "delegation-services"))]
    assert!(Runtime::reopen_durable(&f.policy()).is_err());
    assert_eq!(fs::read(&current).unwrap(), bytes);
}
#[test]
fn actual_active_and_tombstone_v3_fixtures_require_issuer_support_without_rewrite() {
    for bytes in [
        include_bytes!("fixtures/issuer-active-v3.bin").as_slice(),
        include_bytes!("fixtures/issuer-tombstone-v3.bin").as_slice(),
    ] {
        let f = Fixture::new();
        let r =
            Runtime::initialize_durable(&f.policy(), registry(1, 0, vec![entry(1, 1)])).unwrap();
        drop(r);
        let snapshot = f.policy().join("provider-policy.snapshot");
        fs::write(&snapshot, bytes).unwrap();
        let result = Runtime::reopen_durable(&f.policy());
        assert_eq!(result.is_ok(), cfg!(feature = "delegation-issuer"));
        drop(result);
        assert_eq!(fs::read(snapshot).unwrap(), bytes);
    }
}
#[test]
fn actual_active_and_tombstone_v4_fixtures_require_verifier_support_without_rewrite() {
    for bytes in [
        include_bytes!("fixtures/resource-active-v4.bin").as_slice(),
        include_bytes!("fixtures/resource-tombstone-v4.bin").as_slice(),
    ] {
        let f = Fixture::new();
        let r = Runtime::initialize_durable(&f.policy(), registry(1, 0, vec![])).unwrap();
        drop(r);
        let snapshot = f.policy().join("provider-policy.snapshot");
        fs::write(&snapshot, bytes).unwrap();
        let result = Runtime::reopen_durable(&f.policy());
        assert_eq!(result.is_ok(), cfg!(feature = "delegation-verifier"));
        drop(result);
        assert_eq!(fs::read(snapshot).unwrap(), bytes);
    }
}
#[test]
fn unsupported_combined_resource_and_issuer_policy_is_refused_without_fallback_or_rewrite() {
    let bytes = include_bytes!("fixtures/resource-combined-v4.bin");
    let f = Fixture::new();
    let r = Runtime::initialize_durable(&f.policy(), registry(1, 0, vec![])).unwrap();
    drop(r);
    let path = f.policy().join("provider-policy.snapshot");
    fs::write(&path, bytes).unwrap();
    assert!(Runtime::reopen_durable(&f.policy()).is_err());
    assert_eq!(fs::read(path).unwrap(), bytes);
}
