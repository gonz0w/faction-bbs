#![cfg(feature = "delegation-verifier")]
use faction_provider::{
    Runtime,
    authority::Registry,
    services::{ResourceAssociation, ServicePolicy, ServiceRegistration, ServiceUse},
};
fn entry(d: u8, home: &str, id: &str) -> ServiceRegistration {
    ServiceRegistration {
        home: home.into(),
        client_id: id.into(),
        dns_san: "service.test".into(),
        certificate_sha256: [d; 32],
        enabled: true,
        valid_from: 1,
        valid_until: u64::MAX,
    }
}
fn entries() -> Vec<(ServiceRegistration, ServiceUse)> {
    vec![
        (
            entry(10, "remote.test", "relay"),
            ServiceUse::ResourceClient,
        ),
        (
            entry(11, "local.test", "inspector"),
            ServiceUse::OutboundIntrospection,
        ),
        (
            entry(12, "remote.test", "endpoint"),
            ServiceUse::IssuerEndpoint,
        ),
    ]
}
fn association() -> ResourceAssociation {
    ResourceAssociation {
        recipient_home: "remote.test".into(),
        issuer: "issuer.test".into(),
        audience: "local.test/attachments".into(),
        relay_client_id: "relay".into(),
        outbound_client_id: "inspector".into(),
        endpoint_id: "endpoint".into(),
        address: "127.0.0.1:443".parse().unwrap(),
        http_host: "issuer.test".into(),
        ca_sha256: [13; 32],
        remote_generation: 3,
        remote_reset: 2,
    }
}
fn registry(g: u64) -> Registry {
    Registry::new(
        g,
        0,
        vec!["local.test".into(), "remote.test".into()],
        vec![],
    )
    .unwrap()
}
#[test]
fn explicit_resource_policy_joins_three_canonical_credentials_and_complete_remote_tuple() {
    let p =
        ServicePolicy::new_resource("local.test".into(), entries(), vec![association()]).unwrap();
    assert_eq!(p.registrations().len(), 3);
    assert_eq!(p.resource_associations(), &[association()]);
    let r = Runtime::new(Some(registry(1)));
    r.control()
        .replace_delegation_policy(registry(2), p)
        .unwrap();
    for variant in 0..8 {
        let mut a = association();
        match variant {
            0 => a.recipient_home = "local.test".into(),
            1 => a.audience = "remote.test/attachments".into(),
            2 => a.relay_client_id = "missing".into(),
            3 => a.outbound_client_id = "relay".into(),
            4 => a.endpoint_id = "relay".into(),
            5 => a.http_host = "other.test".into(),
            6 => a.ca_sha256 = [0; 32],
            _ => a.remote_generation = 0,
        };
        assert!(ServicePolicy::new_resource("local.test".into(), entries(), vec![a]).is_err());
    }
    let mut e = entries();
    e[1].0.home = "remote.test".into();
    assert!(ServicePolicy::new_resource("local.test".into(), e, vec![association()]).is_err());
    assert!(
        ServicePolicy::new_issuer("local.test".into(), "issuer.test".into(), entries(), vec![])
            .is_err()
    );
}
#[test]
fn remote_generation_and_reset_floors_survive_disable_reset_and_complete_higher_local_generation() {
    let r = Runtime::new(Some(registry(1)));
    r.control()
        .replace_delegation_policy(
            registry(2),
            ServicePolicy::new_resource("local.test".into(), entries(), vec![association()])
                .unwrap(),
        )
        .unwrap();
    r.control().replace_registry(registry(3)).unwrap();
    for (g, reset) in [(2, 2), (3, 1)] {
        let mut a = association();
        a.remote_generation = g;
        a.remote_reset = reset;
        assert!(
            r.control()
                .replace_delegation_policy(
                    registry(4),
                    ServicePolicy::new_resource("local.test".into(), entries(), vec![a]).unwrap()
                )
                .is_err()
        );
    }
    r.control().reset(1).unwrap();
    let next = Registry::new(
        5,
        1,
        vec!["local.test".into(), "remote.test".into()],
        vec![],
    )
    .unwrap();
    let mut a = association();
    a.remote_generation = 4;
    a.remote_reset = 2;
    r.control()
        .replace_delegation_policy(
            next,
            ServicePolicy::new_resource("local.test".into(), entries(), vec![a]).unwrap(),
        )
        .unwrap();
}
#[cfg(feature = "durable-policy")]
struct Owned(std::path::PathBuf);
#[cfg(feature = "durable-policy")]
impl Owned {
    fn new() -> Self {
        use std::os::unix::fs::DirBuilderExt;
        let path = std::env::temp_dir().join(format!(
            "f315-policy-OWNED-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .unwrap();
        std::fs::write(
            path.join("OWNED"),
            b"Faction315 synthetic private policy fixture",
        )
        .unwrap();
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(path.join("policy"))
            .unwrap();
        Self(path)
    }
}
#[cfg(feature = "durable-policy")]
impl Drop for Owned {
    fn drop(&mut self) {
        assert_eq!(
            std::fs::read(self.0.join("OWNED")).unwrap(),
            b"Faction315 synthetic private policy fixture"
        );
        std::fs::remove_dir_all(&self.0).unwrap();
        assert!(!self.0.exists());
    }
}
#[cfg(feature = "durable-policy")]
#[test]
fn v4_atomic_publish_reopen_and_tombstone_retain_remote_floors_without_transient_authority() {
    let f = Owned::new();
    let path = f.0.join("policy");
    let r = Runtime::initialize_durable(&path, registry(1)).unwrap();
    r.control()
        .replace_delegation_policy(
            registry(2),
            ServicePolicy::new_resource("local.test".into(), entries(), vec![association()])
                .unwrap(),
        )
        .unwrap();
    let raw = std::fs::read(path.join("provider-policy.snapshot")).unwrap();
    assert_eq!(&raw[..8], b"FPPY0004");
    assert_eq!(raw, include_bytes!("fixtures/resource-active-v4.bin"));
    drop(r);
    let r = Runtime::reopen_durable(&path).unwrap();
    r.control().replace_registry(registry(3)).unwrap();
    drop(r);
    let raw = std::fs::read(path.join("provider-policy.snapshot")).unwrap();
    assert_eq!(&raw[..8], b"FPPY0004");
    assert_eq!(raw, include_bytes!("fixtures/resource-tombstone-v4.bin"));
    let r = Runtime::reopen_durable(&path).unwrap();
    let mut stale = association();
    stale.remote_reset = 1;
    assert!(
        r.control()
            .replace_delegation_policy(
                registry(4),
                ServicePolicy::new_resource("local.test".into(), entries(), vec![stale]).unwrap()
            )
            .is_err()
    );
    assert_eq!(
        raw,
        std::fs::read(path.join("provider-policy.snapshot")).unwrap()
    );
    r.control().reset(1).unwrap();
    drop(r);
    let r = Runtime::reopen_durable(&path).unwrap();
    let before = std::fs::read(path.join("provider-policy.snapshot")).unwrap();
    let mut stale = association();
    stale.remote_generation = 2;
    assert!(
        r.control()
            .replace_delegation_policy(
                Registry::new(
                    5,
                    1,
                    vec!["local.test".into(), "remote.test".into()],
                    vec![]
                )
                .unwrap(),
                ServicePolicy::new_resource("local.test".into(), entries(), vec![stale]).unwrap()
            )
            .is_err()
    );
    assert_eq!(
        before,
        std::fs::read(path.join("provider-policy.snapshot")).unwrap()
    );
    drop(r);
}
#[test]
fn endpoint_outgoing_credentials_share_all_active_history_caps_and_remote_homes_share_the_home_cap()
{
    let mut full = entries();
    for i in 0..29 {
        full.push((
            entry(40 + i, "remote.test", &format!("observer{i}")),
            ServiceUse::Observation,
        ));
    }
    let full_policy =
        ServicePolicy::new_resource("local.test".into(), full.clone(), vec![association()])
            .unwrap();
    let r = Runtime::new(Some(registry(1)));
    r.control()
        .replace_delegation_policy(registry(2), full_policy.clone())
        .unwrap();
    r.control()
        .replace_delegation_policy(
            registry(3),
            ServicePolicy::new_resource("local.test".into(), entries(), vec![association()])
                .unwrap(),
        )
        .unwrap();
    let mut rotated = entries();
    rotated[2].0.certificate_sha256 = [99; 32];
    assert!(
        r.control()
            .replace_delegation_policy(
                registry(4),
                ServicePolicy::new_resource("local.test".into(), rotated, vec![association()])
                    .unwrap()
            )
            .is_err()
    );
    let mut homes = vec!["local.test".into()];
    for i in 0..31 {
        homes.push(format!("home{i}.test"));
    }
    assert!(
        r.control()
            .replace_registry(Registry::new(4, 0, homes, vec![]).unwrap())
            .is_err()
    );
    let device = faction_provider::authority::CredentialBinding {
        certificate_sha256: [100; 32],
        principal: faction_files::private_blobs::Principal {
            account: "device@local.test".into(),
            device: 1,
            identity: [1; 32],
        },
        valid_from: 1,
        valid_until: u64::MAX,
        approved: true,
        account_enabled: true,
        device_enabled: true,
        generation: 2,
    };
    let occupied = Runtime::new(Some(registry(1)));
    assert!(
        occupied
            .control()
            .replace_delegation_policy(
                Registry::new(
                    2,
                    0,
                    vec!["local.test".into(), "remote.test".into()],
                    vec![device]
                )
                .unwrap(),
                full_policy
            )
            .is_err()
    );
}
fn valid_old_issuer_with_extra(
    extra: ServiceUse,
    home: &str,
) -> Result<ServicePolicy, faction_files::private_blobs::Error> {
    ServicePolicy::new_issuer(
        "local.test".into(),
        "issuer.test".into(),
        vec![
            (entry(21, "local.test", "old-relay"), ServiceUse::Relay),
            (
                entry(22, "remote.test", "old-inspector"),
                ServiceUse::Introspection,
            ),
            (entry(23, home, "extra"), extra),
        ],
        vec![faction_provider::services::IssuerRoute {
            sender_home: "remote.test".into(),
            audience: "remote.test/attachments".into(),
            relay_client_id: "old-relay".into(),
            introspection_client_id: "old-inspector".into(),
        }],
    )
}
#[test]
fn valid_legacy_issuer_constructor_never_accepts_appended_resource_client() {
    assert!(valid_old_issuer_with_extra(ServiceUse::ResourceClient, "remote.test").is_err());
}
#[test]
fn valid_legacy_issuer_constructor_never_accepts_appended_endpoint() {
    assert!(valid_old_issuer_with_extra(ServiceUse::IssuerEndpoint, "remote.test").is_err());
}
#[test]
fn valid_legacy_issuer_constructor_never_accepts_any_outbound_role_home() {
    for home in ["local.test", "remote.test"] {
        assert!(valid_old_issuer_with_extra(ServiceUse::OutboundIntrospection, home).is_err());
    }
}
#[test]
fn new_resource_profile_preserves_legacy_direct_observation_identifiers() {
    let legacy = entry(24, "remote.test", &"LEGACY:+/".repeat(12));
    assert!(ServicePolicy::new("local.test".into(), vec![legacy.clone()]).is_ok());
    let mut all = entries();
    all.push((legacy, ServiceUse::Observation));
    assert!(ServicePolicy::new_resource("local.test".into(), all, vec![association()]).is_ok());
}
#[cfg(feature = "durable-policy")]
fn domain(length: usize) -> String {
    let mut labels = Vec::new();
    let mut remaining = length;
    while remaining > 63 {
        labels.push("a".repeat(63));
        remaining -= 64;
    }
    labels.push("b".repeat(remaining));
    labels.join(".")
}
#[cfg(feature = "durable-policy")]
#[test]
fn every_constructible_independent_issuer_host_boundary_publishes_and_reopens() {
    for length in [189, 190, 253] {
        let mut a = association();
        a.issuer = domain(length);
        a.http_host = a.issuer.clone();
        let p = ServicePolicy::new_resource("local.test".into(), entries(), vec![a]).unwrap();
        let f = Owned::new();
        let path = f.0.join("policy");
        let r = Runtime::initialize_durable(&path, registry(1)).unwrap();
        r.control()
            .replace_delegation_policy(registry(2), p)
            .unwrap();
        drop(r);
        assert!(
            Runtime::reopen_durable(&path).is_ok(),
            "constructible issuer/Host length {length} cannot reopen"
        );
    }
}
#[cfg(feature = "durable-policy")]
#[test]
fn retained_v4_floor_survives_an_issuer_only_replacement_and_reopen() {
    let f = Owned::new();
    let path = f.0.join("policy");
    let r = Runtime::initialize_durable(&path, registry(1)).unwrap();
    r.control()
        .replace_delegation_policy(
            registry(2),
            ServicePolicy::new_resource("local.test".into(), entries(), vec![association()])
                .unwrap(),
        )
        .unwrap();
    r.control()
        .replace_delegation_policy(
            registry(3),
            valid_old_issuer_with_extra(ServiceUse::Observation, "remote.test").unwrap(),
        )
        .unwrap();
    drop(r);
    let raw = std::fs::read(path.join("provider-policy.snapshot")).unwrap();
    assert_eq!(&raw[..8], b"FPPY0004");
    let r = Runtime::reopen_durable(&path).unwrap();
    let mut stale = association();
    stale.remote_generation = 2;
    assert!(
        r.control()
            .replace_delegation_policy(
                registry(4),
                ServicePolicy::new_resource("local.test".into(), entries(), vec![stale]).unwrap()
            )
            .is_err()
    );
    assert_eq!(
        std::fs::read(path.join("provider-policy.snapshot")).unwrap(),
        raw
    );
    drop(r);
}

fn composed_entries() -> Vec<(ServiceRegistration, ServiceUse)> {
    let mut all = entries();
    all.extend([
        (entry(21, "local.test", "local-relay"), ServiceUse::Relay),
        (
            entry(22, "remote.test", "remote-inspector"),
            ServiceUse::Introspection,
        ),
    ]);
    all
}
fn composed_routes() -> Vec<faction_provider::services::IssuerRoute> {
    vec![faction_provider::services::IssuerRoute {
        sender_home: "remote.test".into(),
        audience: "remote.test/attachments".into(),
        relay_client_id: "local-relay".into(),
        introspection_client_id: "remote-inspector".into(),
    }]
}
#[test]
fn complete_composed_policy_joins_both_roles_in_one_canonical_collection() {
    let p = ServicePolicy::new_composed(
        "local.test".into(),
        "local-issuer.test".into(),
        composed_entries(),
        composed_routes(),
        vec![association()],
    )
    .unwrap();
    assert_eq!(p.registrations().len(), 5);
    assert_eq!(p.resource_associations(), &[association()]);
    let runtime = Runtime::new(Some(registry(1)));
    runtime
        .control()
        .replace_delegation_policy(registry(2), p)
        .unwrap();
}

#[test]
fn composed_validation_rejects_cross_role_aliases_purpose_confusion_and_missing_references() {
    for variant in 0..19 {
        let mut e = composed_entries();
        let mut routes = composed_routes();
        let mut a = association();
        match variant {
            0 => e[3].0.certificate_sha256 = e[0].0.certificate_sha256,
            1 => {
                e[3].0.home = e[1].0.home.clone();
                e[3].0.client_id = e[1].0.client_id.clone();
            }
            2 => e[3].0.home = "remote.test".into(),
            3 => e[4].0.home = "local.test".into(),
            4 => e[0].1 = ServiceUse::Introspection,
            5 => e[1].1 = ServiceUse::Relay,
            6 => e[2].1 = ServiceUse::Observation,
            7 => routes[0].relay_client_id = "inspector".into(),
            8 => routes[0].introspection_client_id = "relay".into(),
            9 => routes[0].sender_home = "local.test".into(),
            10 => routes[0].audience = "local.test/attachments".into(),
            11 => a.relay_client_id = "remote-inspector".into(),
            12 => a.outbound_client_id = "local-relay".into(),
            13 => a.endpoint_id = "relay".into(),
            14 => a.http_host = "elsewhere.test".into(),
            15 => a.ca_sha256 = [0; 32],
            16 => routes.clear(),
            17 => routes.push(routes[0].clone()),
            _ => e[3].0.client_id = "INVALID+".into(),
        }
        assert!(
            ServicePolicy::new_composed(
                "local.test".into(),
                "local-issuer.test".into(),
                e,
                routes,
                vec![a],
            )
            .is_err(),
            "accepted variant {variant}"
        );
    }
    let e = composed_entries();
    assert!(
        ServicePolicy::new_resource("local.test".into(), e.clone(), vec![association()]).is_err()
    );
    assert!(
        ServicePolicy::new_issuer(
            "local.test".into(),
            "issuer.test".into(),
            e,
            composed_routes()
        )
        .is_err()
    );
    assert!(
        ServicePolicy::new_composed(
            "local.test".into(),
            "issuer.test".into(),
            composed_entries(),
            composed_routes(),
            vec![]
        )
        .is_err()
    );
}
#[cfg(feature = "durable-policy")]
#[test]
fn composed_publish_reopen_disable_reset_and_independent_remote_floors_preserve_bytes() {
    let f = Owned::new();
    let path = f.0.join("policy");
    let mut r = Runtime::initialize_durable(&path, registry(1)).unwrap();
    let policy = |a| {
        ServicePolicy::new_composed(
            "local.test".into(),
            "local-issuer.test".into(),
            composed_entries(),
            composed_routes(),
            vec![a],
        )
        .unwrap()
    };
    r.control()
        .replace_delegation_policy(registry(2), policy(association()))
        .unwrap();
    let active = std::fs::read(path.join("provider-policy.snapshot")).unwrap();
    assert_eq!(&active[..8], b"FPPY0004");
    drop(r);
    r = Runtime::reopen_durable(&path).unwrap();
    assert_eq!(
        std::fs::read(path.join("provider-policy.snapshot")).unwrap(),
        active
    );
    r.control().replace_registry(registry(3)).unwrap();
    drop(r);
    r = Runtime::reopen_durable(&path).unwrap();
    r.control().reset(1).unwrap();
    drop(r);
    r = Runtime::reopen_durable(&path).unwrap();
    let before = std::fs::read(path.join("provider-policy.snapshot")).unwrap();
    for (g, reset) in [(2, 3), (4, 1)] {
        let mut a = association();
        a.remote_generation = g;
        a.remote_reset = reset;
        assert!(
            r.control()
                .replace_delegation_policy(
                    Registry::new(
                        5,
                        1,
                        vec!["local.test".into(), "remote.test".into()],
                        vec![]
                    )
                    .unwrap(),
                    policy(a)
                )
                .is_err()
        );
        assert_eq!(
            std::fs::read(path.join("provider-policy.snapshot")).unwrap(),
            before
        );
    }
    r.control()
        .replace_delegation_policy(
            Registry::new(
                5,
                1,
                vec!["local.test".into(), "remote.test".into()],
                vec![],
            )
            .unwrap(),
            policy(association()),
        )
        .unwrap();
    drop(r);
    assert!(Runtime::reopen_durable(&path).is_ok());
}

#[test]
fn composed_roles_share_all_active_history_and_home_limits() {
    let mut full = composed_entries();
    for i in 0..27 {
        full.push((
            entry(40 + i, "remote.test", &format!("observer{i}")),
            ServiceUse::Observation,
        ));
    }
    let policy = |e| {
        ServicePolicy::new_composed(
            "local.test".into(),
            "issuer.test".into(),
            e,
            composed_routes(),
            vec![association()],
        )
        .unwrap()
    };
    let runtime = Runtime::new(Some(registry(1)));
    runtime
        .control()
        .replace_delegation_policy(registry(2), policy(full.clone()))
        .unwrap();
    runtime
        .control()
        .replace_delegation_policy(registry(3), policy(composed_entries()))
        .unwrap();
    let mut rotated = composed_entries();
    rotated[3].0.certificate_sha256 = [99; 32];
    assert!(
        runtime
            .control()
            .replace_delegation_policy(registry(4), policy(rotated))
            .is_err()
    );
    let mut homes = vec!["local.test".into()];
    homes.extend((0..31).map(|i| format!("home{i}.test")));
    assert!(
        runtime
            .control()
            .replace_registry(Registry::new(4, 0, homes, vec![]).unwrap())
            .is_err()
    );
    let device = faction_provider::authority::CredentialBinding {
        certificate_sha256: [100; 32],
        principal: faction_files::private_blobs::Principal {
            account: "device@local.test".into(),
            device: 1,
            identity: [1; 32],
        },
        valid_from: 1,
        valid_until: u64::MAX,
        approved: true,
        account_enabled: true,
        device_enabled: true,
        generation: 2,
    };
    let occupied = Runtime::new(Some(registry(1)));
    assert!(
        occupied
            .control()
            .replace_delegation_policy(
                Registry::new(
                    2,
                    0,
                    vec!["local.test".into(), "remote.test".into()],
                    vec![device]
                )
                .unwrap(),
                policy(full)
            )
            .is_err()
    );
    for digest in [10, 21] {
        let mut aliases = composed_entries();
        aliases[4].0.certificate_sha256 = [digest; 32];
        assert!(
            ServicePolicy::new_composed(
                "local.test".into(),
                "issuer.test".into(),
                aliases,
                composed_routes(),
                vec![association()]
            )
            .is_err()
        );
    }
}

#[test]
fn explicit_composed_publication_joins_a_distinct_outgoing_permission() {
    use faction_provider::services::PublicationAssociation;
    let mut all = composed_entries();
    all.push((
        entry(23, "local.test", "observer"),
        ServiceUse::OutboundObservation,
    ));
    let publication = PublicationAssociation {
        recipient_home: "remote.test".into(),
        outbound_client_id: "observer".into(),
        endpoint_id: "endpoint".into(),
        address: "127.0.0.1:443".parse().unwrap(),
        http_host: "remote.test".into(),
        ca_sha256: [13; 32],
        remote_generation: 3,
        remote_reset: 2,
    };
    assert!(
        ServicePolicy::new_composed(
            "local.test".into(),
            "issuer.test".into(),
            all.clone(),
            composed_routes(),
            vec![association()]
        )
        .is_err()
    );
    let policy = ServicePolicy::new_composed_publication(
        "local.test".into(),
        "issuer.test".into(),
        all,
        composed_routes(),
        vec![association()],
        vec![publication.clone()],
    )
    .unwrap();
    assert_eq!(policy.publication_associations(), &[publication]);
    let runtime = Runtime::new(Some(registry(1)));
    runtime
        .control()
        .replace_delegation_policy(registry(2), policy)
        .unwrap();
}

#[cfg(feature = "durable-policy")]
#[test]
fn publication_policy_publishes_v5_and_reopens_without_transient_sender_authority() {
    use faction_provider::services::PublicationAssociation;
    let mut all = composed_entries();
    all.push((
        entry(23, "local.test", "observer"),
        ServiceUse::OutboundObservation,
    ));
    let publication = PublicationAssociation {
        recipient_home: "remote.test".into(),
        outbound_client_id: "observer".into(),
        endpoint_id: "endpoint".into(),
        address: "127.0.0.1:443".parse().unwrap(),
        http_host: "remote.test".into(),
        ca_sha256: [13; 32],
        remote_generation: 3,
        remote_reset: 2,
    };
    let policy = ServicePolicy::new_composed_publication(
        "local.test".into(),
        "issuer.test".into(),
        all,
        composed_routes(),
        vec![association()],
        vec![publication],
    )
    .unwrap();
    let f = Owned::new();
    let path = f.0.join("policy");
    let runtime = Runtime::initialize_durable(&path, registry(1)).unwrap();
    runtime
        .control()
        .replace_delegation_policy(registry(2), policy)
        .unwrap();
    let active = std::fs::read(path.join("provider-policy.snapshot")).unwrap();
    assert_eq!(&active[..8], b"FPPY0005");
    drop(runtime);
    let runtime = Runtime::reopen_durable(&path).unwrap();
    assert_eq!(
        std::fs::read(path.join("provider-policy.snapshot")).unwrap(),
        active
    );
    runtime.control().replace_registry(registry(3)).unwrap();
    drop(runtime);
    let tombstone = std::fs::read(path.join("provider-policy.snapshot")).unwrap();
    assert_eq!(&tombstone[..8], b"FPPY0005");
    assert!(Runtime::reopen_durable(&path).is_ok());
    if let Some(destination) = std::env::var_os("FACTION318_CAPTURE_POLICY") {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        for (name, bytes) in [
            ("publication-active-v5.bin", active),
            ("publication-tombstone-v5.bin", tombstone),
        ] {
            std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(std::path::Path::new(&destination).join(name))
                .unwrap()
                .write_all(&bytes)
                .unwrap();
        }
    }
}

fn publication_entries() -> Vec<(ServiceRegistration, ServiceUse)> {
    let mut entries = composed_entries();
    entries.push((
        entry(23, "local.test", "observer"),
        ServiceUse::OutboundObservation,
    ));
    entries
}
fn publication_association() -> faction_provider::services::PublicationAssociation {
    faction_provider::services::PublicationAssociation {
        recipient_home: "remote.test".into(),
        outbound_client_id: "observer".into(),
        endpoint_id: "endpoint".into(),
        address: "127.0.0.1:443".parse().unwrap(),
        http_host: "remote.test".into(),
        ca_sha256: [13; 32],
        remote_generation: 3,
        remote_reset: 2,
    }
}
fn publication_policy(
    entries: Vec<(ServiceRegistration, ServiceUse)>,
    association: faction_provider::services::PublicationAssociation,
) -> ServicePolicy {
    ServicePolicy::new_composed_publication(
        "local.test".into(),
        "issuer.test".into(),
        entries,
        composed_routes(),
        vec![self::association()],
        vec![association],
    )
    .unwrap()
}
#[test]
fn publication_profile_refuses_purpose_confusion_substitution_and_mismatched_remote_tuples() {
    for variant in 0..12 {
        let mut entries = publication_entries();
        let mut a = publication_association();
        match variant {
            0 => entries[5].1 = ServiceUse::OutboundIntrospection,
            1 => entries[5].0.home = "remote.test".into(),
            2 => a.outbound_client_id = "inspector".into(),
            3 => a.endpoint_id = "remote-inspector".into(),
            4 => a.recipient_home = "local.test".into(),
            5 => a.http_host = "issuer.test".into(),
            6 => a.ca_sha256 = [0; 32],
            7 => a.remote_generation = 4,
            8 => a.remote_reset = 3,
            9 => entries[5].0.certificate_sha256 = [21; 32],
            10 => entries[5].0.client_id = "inspector".into(),
            _ => a.address = "127.0.0.1:0".parse().unwrap(),
        };
        assert!(
            ServicePolicy::new_composed_publication(
                "local.test".into(),
                "issuer.test".into(),
                entries,
                composed_routes(),
                vec![association()],
                vec![a]
            )
            .is_err(),
            "variant {variant}"
        );
    }
    assert!(
        ServicePolicy::new_resource(
            "local.test".into(),
            publication_entries(),
            vec![association()]
        )
        .is_err()
    );
    assert!(
        ServicePolicy::new_issuer(
            "local.test".into(),
            "issuer.test".into(),
            publication_entries(),
            composed_routes()
        )
        .is_err()
    );
    let a = publication_association();
    assert!(
        ServicePolicy::new_composed_publication(
            "local.test".into(),
            "issuer.test".into(),
            publication_entries(),
            composed_routes(),
            vec![association()],
            vec![a.clone(), a]
        )
        .is_err()
    );
}
#[test]
fn publication_credentials_use_the_same_active_and_history_capacity_as_all_other_roles() {
    let mut full = publication_entries();
    for i in 0..26 {
        full.push((
            entry(40 + i, "remote.test", &format!("observer{i}")),
            ServiceUse::Observation,
        ));
    }
    let runtime = Runtime::new(Some(registry(1)));
    runtime
        .control()
        .replace_delegation_policy(
            registry(2),
            publication_policy(full.clone(), publication_association()),
        )
        .unwrap();
    runtime
        .control()
        .replace_delegation_policy(
            registry(3),
            publication_policy(publication_entries(), publication_association()),
        )
        .unwrap();
    let mut rotated = publication_entries();
    rotated[5].0.certificate_sha256 = [99; 32];
    assert!(
        runtime
            .control()
            .replace_delegation_policy(
                registry(4),
                publication_policy(rotated, publication_association())
            )
            .is_err()
    );
    let device = faction_provider::authority::CredentialBinding {
        certificate_sha256: [100; 32],
        principal: faction_files::private_blobs::Principal {
            account: "device@local.test".into(),
            device: 1,
            identity: [1; 32],
        },
        valid_from: 1,
        valid_until: u64::MAX,
        approved: true,
        account_enabled: true,
        device_enabled: true,
        generation: 2,
    };
    let runtime = Runtime::new(Some(registry(1)));
    assert!(
        runtime
            .control()
            .replace_delegation_policy(
                Registry::new(
                    2,
                    0,
                    vec!["local.test".into(), "remote.test".into()],
                    vec![device]
                )
                .unwrap(),
                publication_policy(full, publication_association())
            )
            .is_err()
    );
}
#[cfg(feature = "durable-policy")]
#[test]
fn v5_directional_and_observation_replacements_reset_and_restore_keep_marker_and_floors() {
    let f = Owned::new();
    let path = f.0.join("policy");
    let mut runtime = Runtime::initialize_durable(&path, registry(1)).unwrap();
    runtime
        .control()
        .replace_delegation_policy(
            registry(2),
            publication_policy(publication_entries(), publication_association()),
        )
        .unwrap();
    let issuer_entries = composed_entries()
        .into_iter()
        .filter(|(_, u)| matches!(u, ServiceUse::Relay | ServiceUse::Introspection))
        .collect();
    for (g, policy) in [
        (
            3,
            ServicePolicy::new_resource("local.test".into(), entries(), vec![association()])
                .unwrap(),
        ),
        (
            4,
            ServicePolicy::new_issuer(
                "local.test".into(),
                "issuer.test".into(),
                issuer_entries,
                composed_routes(),
            )
            .unwrap(),
        ),
        (
            5,
            ServicePolicy::new(
                "local.test".into(),
                vec![entry(24, "remote.test", "LEGACY:+/")],
            )
            .unwrap(),
        ),
    ] {
        runtime
            .control()
            .replace_delegation_policy(registry(g), policy)
            .unwrap();
        let before = std::fs::read(path.join("provider-policy.snapshot")).unwrap();
        assert_eq!(&before[..8], b"FPPY0005");
        drop(runtime);
        runtime = Runtime::reopen_durable(&path).unwrap();
        assert_eq!(
            std::fs::read(path.join("provider-policy.snapshot")).unwrap(),
            before
        );
    }
    runtime.control().replace_registry(registry(6)).unwrap();
    runtime.control().reset(1).unwrap();
    drop(runtime);
    runtime = Runtime::reopen_durable(&path).unwrap();
    let before = std::fs::read(path.join("provider-policy.snapshot")).unwrap();
    for (generation, reset) in [(2, 3), (4, 1)] {
        let mut a = publication_association();
        a.remote_generation = generation;
        a.remote_reset = reset;
        let mut resource = association();
        resource.remote_generation = generation;
        resource.remote_reset = reset;
        let policy = ServicePolicy::new_composed_publication(
            "local.test".into(),
            "issuer.test".into(),
            publication_entries(),
            composed_routes(),
            vec![resource],
            vec![a],
        )
        .unwrap();
        assert!(
            runtime
                .control()
                .replace_delegation_policy(
                    Registry::new(
                        8,
                        1,
                        vec!["local.test".into(), "remote.test".into()],
                        vec![]
                    )
                    .unwrap(),
                    policy
                )
                .is_err()
        );
        assert_eq!(
            std::fs::read(path.join("provider-policy.snapshot")).unwrap(),
            before
        );
    }
    runtime
        .control()
        .replace_delegation_policy(
            Registry::new(
                8,
                1,
                vec!["local.test".into(), "remote.test".into()],
                vec![],
            )
            .unwrap(),
            publication_policy(publication_entries(), publication_association()),
        )
        .unwrap();
    drop(runtime);
    assert!(Runtime::reopen_durable(&path).is_ok());
}
