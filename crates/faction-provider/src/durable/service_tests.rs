use super::*;
use crate::services::{ServicePolicy, ServiceRegistration};
fn combined(g: u64) -> Registry {
    Registry::new(
        g,
        0,
        vec!["home.test".into(), "foreign.test".into()],
        vec![entry(1, g)],
    )
    .unwrap()
}
fn services() -> ServicePolicy {
    ServicePolicy::new(
        "home.test".into(),
        vec![ServiceRegistration {
            home: "foreign.test".into(),
            client_id: "sender".into(),
            dns_san: "service.test".into(),
            certificate_sha256: [44; 32],
            enabled: true,
            valid_from: 1,
            valid_until: 10000,
        }],
    )
    .unwrap()
}
#[test]
fn v1_actual_files_are_never_migrated_on_read_and_explicit_update_retains_all_device_history() {
    let f = Fixture::new();
    let r = Runtime::initialize_durable(&f.policy(), combined(1)).unwrap();
    let original = fs::read(f.policy().join("provider-policy.snapshot")).unwrap();
    drop(r);
    let r = Runtime::reopen_durable(&f.policy()).unwrap();
    assert_eq!(
        fs::read(f.policy().join("provider-policy.snapshot")).unwrap(),
        original
    );
    r.control()
        .replace_registry(
            Registry::new(
                2,
                0,
                vec!["home.test".into(), "foreign.test".into()],
                vec![],
            )
            .unwrap(),
        )
        .unwrap();
    r.control()
        .replace_delegation_policy(combined(3), services())
        .unwrap();
    let before = r.state.authority.lock().unwrap().snapshot();
    drop(r);
    let r = Runtime::reopen_durable(&f.policy()).unwrap();
    let after = r.state.authority.lock().unwrap().snapshot();
    assert_eq!(after.history, before.history);
    assert_eq!(after.service_history, before.service_history);
    assert_eq!(after.service_policy, before.service_policy);
    assert_eq!((after.generation, after.reset_epoch), (3, 0));
    assert_eq!(after.history[0].1, entry(1, 1).principal);
    let bytes = fs::read(f.policy().join("provider-policy.snapshot")).unwrap();
    assert_eq!(&bytes[..8], b"FPPY0002");
    assert_eq!(fs::read_dir(f.policy()).unwrap().count(), 2);
}
#[test]
fn retired_service_and_device_typed_history_cannot_rebind_after_reset_or_restart() {
    let f = Fixture::new();
    let r = Runtime::initialize_durable(&f.policy(), combined(1)).unwrap();
    r.control()
        .replace_delegation_policy(combined(2), services())
        .unwrap();
    r.control().reset(5).unwrap();
    drop(r);
    let r = Runtime::reopen_durable(&f.policy()).unwrap();
    let summary = r.policy_summary().unwrap();
    assert_eq!(
        (
            summary.generation,
            summary.reset_epoch,
            summary.binding_count,
            summary.history_count
        ),
        (3, 5, 0, 2)
    );
    assert!(
        r.control()
            .replace_delegation_policy(combined(2), services())
            .is_err()
    );
    for kind in 0..3 {
        let mut entries = vec![entry(1, 4)];
        let mut regs = services().registrations().to_vec();
        match kind {
            0 => regs[0].client_id = "other".into(),
            1 => regs[0].certificate_sha256 = [1; 32],
            _ => entries[0].certificate_sha256 = [44; 32],
        };
        assert!(
            r.control()
                .replace_delegation_policy(
                    Registry::new(
                        4,
                        5,
                        vec!["home.test".into(), "foreign.test".into()],
                        entries
                    )
                    .unwrap(),
                    ServicePolicy::new("home.test".into(), regs).unwrap()
                )
                .is_err()
        );
    }
    r.control()
        .replace_delegation_policy(
            Registry::new(
                4,
                5,
                vec!["home.test".into(), "foreign.test".into()],
                vec![entry(1, 4)],
            )
            .unwrap(),
            services(),
        )
        .unwrap();
    drop(r);
    let r = Runtime::reopen_durable(&f.policy()).unwrap();
    assert_eq!(r.policy_summary().unwrap().history_count, 2);
}
#[test]
fn failed_combined_publication_never_installs_and_uncertainty_requires_explicit_reopen() {
    for point in POINTS {
        let f = Fixture::new();
        let r = Runtime::initialize_durable(&f.policy(), combined(1)).unwrap();
        let original = fs::read(f.policy().join("provider-policy.snapshot")).unwrap();
        FAULT.set(Fault::Error(point));
        assert!(
            r.control()
                .replace_delegation_policy(combined(2), services())
                .is_err()
        );
        FAULT.set(Fault::None);
        assert!(
            r.control()
                .replace_delegation_policy(combined(3), services())
                .is_err()
        );
        assert!(r.policy_summary().is_err());
        drop(r);
        let bytes = fs::read(f.policy().join("provider-policy.snapshot")).unwrap();
        let published = matches!(point, Point::AfterRename | Point::DirectorySync);
        if !published {
            assert_eq!(bytes, original)
        }
        let r = Runtime::reopen_durable(&f.policy()).unwrap();
        let s = r.state.authority.lock().unwrap().snapshot();
        assert_eq!(s.generation, if published { 2 } else { 1 });
        assert_eq!(s.service_policy.is_some(), published);
        assert_eq!(s.service_history.len(), usize::from(published));
    }
}
#[test]
fn v2_rejects_all_truncations_unknown_types_bad_fields_trailing_and_oversized_state() {
    let mut a = Authority::new(None);
    a.replace_delegation_policy(combined(1), services())
        .unwrap();
    let s = a.snapshot();
    let bytes = codec::encode(&s).unwrap();
    assert_eq!(&bytes[..8], b"FPPY0002");
    assert_eq!(
        codec::encode(&codec::decode(&bytes).unwrap()).unwrap(),
        bytes
    );
    for end in 0..bytes.len() {
        assert!(codec::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    let mut bad = bytes.clone();
    bad[7] = b'9';
    assert!(codec::decode(&bad).is_err());
    let mut bad = bytes.clone();
    bad.push(0);
    assert!(codec::decode(&bad).is_err());
    assert!(codec::decode(&vec![0; MAX_SNAPSHOT + 1]).is_err());
    let mut bad = s.clone();
    bad.service_history.push(bad.service_history[0].clone());
    assert!(codec::encode(&bad).is_err());
    let mut bad = s.clone();
    bad.service_history[0].0 = [1; 32];
    assert!(codec::encode(&bad).is_err());
    let mut bad = s.clone();
    bad.service_history[0].1.client_id = "other".into();
    assert!(codec::encode(&bad).is_err());
    let mut bad = s;
    bad.service_format_v2 = false;
    assert!(codec::encode(&bad).is_err());
}
#[test]
fn maximum_valid_service_strings_and_aggregate_history_survive_actual_reopen() {
    let f = Fixture::new();
    let r = Runtime::initialize_durable(&f.policy(), combined(1)).unwrap();
    let mut regs = services().registrations().to_vec();
    regs[0].client_id = "x".repeat(254);
    regs[0].dns_san = format!("{}.{}.{}", "a".repeat(63), "b".repeat(63), "c".repeat(61));
    assert_eq!(regs[0].dns_san.len(), 189);
    let p = ServicePolicy::new("home.test".into(), regs).unwrap();
    r.control()
        .replace_delegation_policy(combined(2), p.clone())
        .unwrap();
    drop(r);
    let r = Runtime::reopen_durable(&f.policy()).unwrap();
    assert_eq!(
        r.state.authority.lock().unwrap().snapshot().service_policy,
        Some(p)
    );
    assert!(
        fs::metadata(f.policy().join("provider-policy.snapshot"))
            .unwrap()
            .len()
            <= MAX_SNAPSHOT as u64
    );
}
#[test]
fn aggregate_32_typed_tombstones_survive_restart_without_an_extra_pool() {
    let f = Fixture::new();
    let entries = (1..=31).map(|n| entry(n, 1)).collect();
    let r = Runtime::initialize_durable(
        &f.policy(),
        Registry::new(
            1,
            0,
            vec!["home.test".into(), "foreign.test".into()],
            entries,
        )
        .unwrap(),
    )
    .unwrap();
    r.control()
        .replace_delegation_policy(
            Registry::new(
                2,
                0,
                vec!["home.test".into(), "foreign.test".into()],
                vec![],
            )
            .unwrap(),
            services(),
        )
        .unwrap();
    drop(r);
    let r = Runtime::reopen_durable(&f.policy()).unwrap();
    assert_eq!(r.policy_summary().unwrap().history_count, 32);
    let mut regs = services().registrations().to_vec();
    regs[0].certificate_sha256 = [45; 32];
    assert!(
        r.control()
            .replace_delegation_policy(
                combined(3),
                ServicePolicy::new("home.test".into(), regs).unwrap()
            )
            .is_err()
    );
    assert_eq!(r.policy_summary().unwrap().generation, 2);
}
