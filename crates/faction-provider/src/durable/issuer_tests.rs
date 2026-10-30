use super::*;
use crate::services::{IssuerRoute, ServicePolicy, ServiceRegistration, ServiceUse};
fn combined(g: u64, reset: u64) -> Registry {
    Registry::new(
        g,
        reset,
        vec!["home.test".into(), "foreign.test".into()],
        vec![entry(1, g)],
    )
    .unwrap()
}
fn policy() -> ServicePolicy {
    let registration = |digest, home: &str, id: &str| ServiceRegistration {
        home: home.into(),
        client_id: id.into(),
        dns_san: "service.test".into(),
        certificate_sha256: [digest; 32],
        enabled: true,
        valid_from: 1,
        valid_until: 10000,
    };
    ServicePolicy::new_issuer(
        "home.test".into(),
        "issuer.test".into(),
        vec![
            (registration(44, "home.test", "relay"), ServiceUse::Relay),
            (
                registration(45, "foreign.test", "inspector"),
                ServiceUse::Introspection,
            ),
        ],
        vec![IssuerRoute {
            sender_home: "foreign.test".into(),
            audience: "foreign.test/attachments".into(),
            relay_client_id: "relay".into(),
            introspection_client_id: "inspector".into(),
        }],
    )
    .unwrap()
}
#[test]
fn actual_v2_read_never_rewrites_or_grants_roles_and_v3_requires_higher_generation() {
    let f = Fixture::new();
    let r = Runtime::initialize_durable(&f.policy(), combined(1, 0)).unwrap();
    let old = ServicePolicy::new(
        "home.test".into(),
        vec![ServiceRegistration {
            home: "foreign.test".into(),
            client_id: "LEGACY:+/".repeat(20),
            dns_san: "service.test".into(),
            certificate_sha256: [46; 32],
            enabled: true,
            valid_from: 1,
            valid_until: 10000,
        }],
    )
    .unwrap();
    r.control()
        .replace_delegation_policy(combined(2, 0), old.clone())
        .unwrap();
    let bytes = fs::read(f.policy().join("provider-policy.snapshot")).unwrap();
    assert_eq!(&bytes[..8], b"FPPY0002");
    drop(r);
    let r = Runtime::reopen_durable(&f.policy()).unwrap();
    assert_eq!(
        bytes,
        fs::read(f.policy().join("provider-policy.snapshot")).unwrap()
    );
    let s = r.state.authority.lock().unwrap().snapshot();
    assert!(!s.service_format_v3);
    assert!(s.service_policy.unwrap().issuer.is_none());
    assert!(
        r.control()
            .replace_delegation_policy(combined(2, 0), policy())
            .is_err()
    );
    assert_eq!(
        bytes,
        fs::read(f.policy().join("provider-policy.snapshot")).unwrap()
    );
    r.control()
        .replace_delegation_policy(combined(3, 0), policy())
        .unwrap();
    drop(r);
    let r = Runtime::reopen_durable(&f.policy()).unwrap();
    let s = r.state.authority.lock().unwrap().snapshot();
    assert!(s.service_format_v3);
    assert_eq!(s.service_history.len(), 3);
    assert_eq!(s.service_policy, Some(policy()));
    r.control().replace_registry(combined(4, 0)).unwrap();
    drop(r);
    let r = Runtime::reopen_durable(&f.policy()).unwrap();
    let s = r.state.authority.lock().unwrap().snapshot();
    assert!(s.service_policy.is_none());
    assert!(s.service_format_v3);
    assert_eq!(s.service_history.len(), 3);
    r.control().reset(5).unwrap();
    drop(r);
    assert_eq!(
        &fs::read(f.policy().join("provider-policy.snapshot")).unwrap()[..8],
        b"FPPY0003"
    );
}
#[test]
fn v3_publication_faults_keep_old_state_or_require_explicit_reopen() {
    for point in POINTS {
        let f = Fixture::new();
        let r = Runtime::initialize_durable(&f.policy(), combined(1, 0)).unwrap();
        let before = fs::read(f.policy().join("provider-policy.snapshot")).unwrap();
        FAULT.set(Fault::Error(point));
        assert!(
            r.control()
                .replace_delegation_policy(combined(2, 0), policy())
                .is_err()
        );
        FAULT.set(Fault::None);
        assert!(r.policy_summary().is_err());
        assert!(r.control().reset(2).is_err());
        drop(r);
        let published = matches!(point, Point::AfterRename | Point::DirectorySync);
        let bytes = fs::read(f.policy().join("provider-policy.snapshot")).unwrap();
        if !published {
            assert_eq!(before, bytes);
        }
        let r = Runtime::reopen_durable(&f.policy()).unwrap();
        let s = r.state.authority.lock().unwrap().snapshot();
        assert_eq!(s.generation, if published { 2 } else { 1 });
        assert_eq!(s.service_format_v3, published);
        assert_eq!(s.service_policy.is_some(), published);
    }
}
#[test]
fn v3_rejects_truncation_unknown_roles_nested_sections_trailing_oversize_and_invalid_history() {
    let mut a = Authority::new(None);
    a.replace_delegation_policy(combined(1, 0), policy())
        .unwrap();
    let s = a.snapshot();
    let bytes = codec::encode(&s).unwrap();
    assert_eq!(
        codec::encode(&codec::decode(&bytes).unwrap()).unwrap(),
        bytes
    );
    for end in 0..bytes.len() {
        assert!(codec::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    let mut bad = bytes.clone();
    bad.push(0);
    assert!(codec::decode(&bad).is_err());
    let mut bad_role = bytes.clone();
    let digest = bad_role.windows(32).rposition(|w| w == [44; 32]).unwrap();
    bad_role[digest + 32 + 1 + 8 + 8] = 3;
    assert!(codec::decode(&bad_role).is_err());
    let mut bad = bytes;
    bad[19] = b'2';
    assert!(codec::decode(&bad).is_err());
    assert!(codec::decode(&vec![0; MAX_SNAPSHOT + 1]).is_err());
    let mut bad = s.clone();
    bad.service_format_v3 = false;
    assert!(codec::encode(&bad).is_err());
    let mut bad = s.clone();
    bad.service_history[0].1.client_id = "other".into();
    assert!(codec::encode(&bad).is_err());
    let mut bad = s.clone();
    bad.service_history.push(bad.service_history[0].clone());
    assert!(codec::encode(&bad).is_err());
    let mut bad = s;
    bad.history.push(([44; 32], entry(2, 1).principal));
    assert!(codec::encode(&bad).is_err());
}
#[test]
fn v3_typed_service_history_floor_and_aggregate_capacity_do_not_gain_an_extra_pool() {
    let f = Fixture::new();
    let registry = Registry::new(
        1,
        0,
        vec!["home.test".into(), "foreign.test".into()],
        (1..=30).map(|n| entry(n, 1)).collect(),
    )
    .unwrap();
    let r = Runtime::initialize_durable(&f.policy(), registry).unwrap();
    r.control()
        .replace_delegation_policy(combined(2, 0), policy())
        .unwrap();
    r.control().reset(3).unwrap();
    drop(r);
    let r = Runtime::reopen_durable(&f.policy()).unwrap();
    assert_eq!(r.policy_summary().unwrap().history_count, 32);
    assert_eq!(r.policy_summary().unwrap().binding_count, 0);
    let before = fs::read(f.policy().join("provider-policy.snapshot")).unwrap();
    assert!(
        r.control()
            .replace_delegation_policy(combined(4, 2), policy())
            .is_err()
    );
    let mut device = entry(31, 4);
    device.certificate_sha256 = [44; 32];
    assert!(
        r.control()
            .replace_registry(Registry::new(4, 3, vec!["home.test".into()], vec![device]).unwrap())
            .is_err()
    );
    assert!(
        r.control()
            .replace_registry(
                Registry::new(4, 3, vec!["home.test".into()], vec![entry(31, 4)]).unwrap()
            )
            .is_err()
    );
    assert_eq!(
        before,
        fs::read(f.policy().join("provider-policy.snapshot")).unwrap()
    );
    r.control()
        .replace_delegation_policy(combined(4, 3), policy())
        .unwrap();
    drop(r);
    let r = Runtime::reopen_durable(&f.policy()).unwrap();
    assert_eq!(r.policy_summary().unwrap().history_count, 32);
}
