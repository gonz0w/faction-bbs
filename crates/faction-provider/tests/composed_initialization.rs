#![cfg(all(feature = "durable-policy", feature = "delegation-verifier"))]
use faction_files::private_blobs::Principal;
use faction_provider::{
    Runtime,
    authority::{CredentialBinding, Registry},
    services::{
        IssuerRoute, PublicationAssociation, ResourceAssociation, ServicePolicy,
        ServiceRegistration, ServiceUse,
    },
};
use std::{
    fs,
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    index: u64,
}
impl Fixture {
    fn new() -> Self {
        let index = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "faction321-initialization-OWNED-{}-{index}",
            std::process::id()
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let mut marker = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join("OWNED"))
            .unwrap();
        marker
            .write_all(b"Owned issue321 composed initializer fixture\n")
            .unwrap();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(root.join("policy"))
            .unwrap();
        Self { root, index }
    }
    fn policy(&self) -> PathBuf {
        self.root.join("policy")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert_eq!(
            fs::read(self.root.join("OWNED")).unwrap(),
            b"Owned issue321 composed initializer fixture\n"
        );
        if let Some(capture) = std::env::var_os("FACTION321_CAPTURE_DIR") {
            let capture = PathBuf::from(capture);
            assert!(capture.is_dir() && !capture.is_symlink());
            assert_eq!(
                fs::metadata(&capture).unwrap().permissions().mode() & 0o7777,
                0o700
            );
            let dest = capture.join(format!(
                "initialization-{}-{}",
                std::process::id(),
                self.index
            ));
            fs::DirBuilder::new().mode(0o700).create(&dest).unwrap();
            for item in fs::read_dir(self.policy()).unwrap() {
                let item = item.unwrap();
                assert!(item.file_type().unwrap().is_file());
                let bytes = fs::read(item.path()).unwrap();
                let mut out = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(dest.join(item.file_name()))
                    .unwrap();
                out.write_all(&bytes).unwrap();
            }
        }
        fs::remove_dir_all(&self.root).unwrap();
        assert!(!self.root.exists());
    }
}
fn registry(generation: u64, reset: u64, foreign: bool) -> Registry {
    let mut homes = vec!["home.test".into()];
    if foreign {
        homes.push("foreign.test".into());
    }
    Registry::new(
        generation,
        reset,
        homes,
        vec![CredentialBinding {
            certificate_sha256: [1; 32],
            principal: Principal {
                account: "member@home.test".into(),
                device: 1,
                identity: [1; 32],
            },
            valid_from: 1,
            valid_until: 10000,
            approved: true,
            account_enabled: true,
            device_enabled: true,
            generation,
        }],
    )
    .unwrap()
}
fn policy(remote_generation: u64, remote_reset: u64, collide: bool) -> ServicePolicy {
    let registration = |digest: u8, home: &str, id: &str| ServiceRegistration {
        home: home.into(),
        client_id: id.into(),
        dns_san: "provider.test".into(),
        certificate_sha256: [digest; 32],
        enabled: true,
        valid_from: 1,
        valid_until: 10000,
    };
    ServicePolicy::new_composed_publication(
        "home.test".into(),
        "home.test".into(),
        vec![
            (
                registration(if collide { 1 } else { 2 }, "home.test", "relay"),
                ServiceUse::Relay,
            ),
            (
                registration(3, "foreign.test", "inspector"),
                ServiceUse::Introspection,
            ),
            (
                registration(4, "foreign.test", "relay"),
                ServiceUse::ResourceClient,
            ),
            (
                registration(5, "home.test", "inspector"),
                ServiceUse::OutboundIntrospection,
            ),
            (
                registration(6, "foreign.test", "endpoint"),
                ServiceUse::IssuerEndpoint,
            ),
            (
                registration(7, "home.test", "observer"),
                ServiceUse::OutboundObservation,
            ),
            (
                registration(8, "foreign.test", "observer"),
                ServiceUse::Observation,
            ),
        ],
        vec![IssuerRoute {
            sender_home: "foreign.test".into(),
            audience: "foreign.test/attachments".into(),
            relay_client_id: "relay".into(),
            introspection_client_id: "inspector".into(),
        }],
        vec![ResourceAssociation {
            recipient_home: "foreign.test".into(),
            issuer: "foreign.test".into(),
            audience: "home.test/attachments".into(),
            relay_client_id: "relay".into(),
            outbound_client_id: "inspector".into(),
            endpoint_id: "endpoint".into(),
            address: "127.0.0.1:32101".parse().unwrap(),
            http_host: "foreign.test".into(),
            ca_sha256: [9; 32],
            remote_generation,
            remote_reset,
        }],
        vec![PublicationAssociation {
            recipient_home: "foreign.test".into(),
            outbound_client_id: "observer".into(),
            endpoint_id: "endpoint".into(),
            address: "127.0.0.1:32101".parse().unwrap(),
            http_host: "foreign.test".into(),
            ca_sha256: [9; 32],
            remote_generation,
            remote_reset,
        }],
    )
    .unwrap()
}
#[test]
fn complete_initial_snapshot_keeps_exact_generation_reset_roles_and_remote_floors() {
    let fixture = Fixture::new();
    let runtime = Runtime::initialize_composed_durable(
        &fixture.policy(),
        registry(7, 3, true),
        policy(19, 5, false),
    )
    .unwrap();
    let summary = runtime.policy_summary().unwrap();
    assert_eq!(
        (
            summary.generation,
            summary.reset_epoch,
            summary.binding_count,
            summary.history_count
        ),
        (7, 3, 8, 8)
    );
    assert!(!runtime.policy_matches_home("home.test").unwrap());
    let before = fs::read(fixture.policy().join("provider-policy.snapshot")).unwrap();
    assert_eq!(&before[..8], b"FPPY0005");
    assert!(!fixture.policy().join("provider-policy.pending").exists());
    drop(runtime);
    let runtime = Runtime::reopen_durable(&fixture.policy()).unwrap();
    assert_eq!(runtime.policy_summary().unwrap(), summary);
    assert_eq!(
        fs::read(fixture.policy().join("provider-policy.snapshot")).unwrap(),
        before
    );
    assert!(
        runtime
            .control()
            .replace_delegation_policy(registry(8, 3, true), policy(19, 4, false))
            .is_err()
    );
    assert_eq!(
        fs::read(fixture.policy().join("provider-policy.snapshot")).unwrap(),
        before
    );
    runtime.control().reset(4).unwrap();
    assert_eq!(runtime.policy_summary().unwrap().history_count, 8);
    assert_eq!(runtime.policy_summary().unwrap().binding_count, 0);
    drop(runtime);
    let reset = Runtime::reopen_durable(&fixture.policy())
        .unwrap()
        .policy_summary()
        .unwrap();
    assert_eq!(
        (
            reset.generation,
            reset.reset_epoch,
            reset.binding_count,
            reset.history_count
        ),
        (8, 4, 0, 8)
    );
}
#[test]
fn invalid_complete_policy_is_refused_before_creating_any_state() {
    for (foreign, collide) in [(false, false), (true, true)] {
        let fixture = Fixture::new();
        assert!(
            Runtime::initialize_composed_durable(
                &fixture.policy(),
                registry(7, 3, foreign),
                policy(19, 5, collide)
            )
            .is_err()
        );
        assert_eq!(fs::read_dir(fixture.policy()).unwrap().count(), 0);
    }
}
#[test]
fn initialization_never_replaces_established_history() {
    let fixture = Fixture::new();
    let runtime = Runtime::initialize_composed_durable(
        &fixture.policy(),
        registry(7, 3, true),
        policy(19, 5, false),
    )
    .unwrap();
    let before = fs::read(fixture.policy().join("provider-policy.snapshot")).unwrap();
    assert!(
        Runtime::initialize_composed_durable(
            &fixture.policy(),
            registry(8, 4, true),
            policy(20, 6, false)
        )
        .is_err()
    );
    assert_eq!(
        fs::read(fixture.policy().join("provider-policy.snapshot")).unwrap(),
        before
    );
    drop(runtime);
    assert!(
        Runtime::initialize_composed_durable(
            &fixture.policy(),
            registry(8, 4, true),
            policy(20, 6, false)
        )
        .is_err()
    );
    assert_eq!(
        fs::read(fixture.policy().join("provider-policy.snapshot")).unwrap(),
        before
    );
    assert_eq!(
        Runtime::reopen_durable(&fixture.policy())
            .unwrap()
            .policy_summary()
            .unwrap()
            .generation,
        7
    );
}

#[test]
fn composed_initializer_rejects_directional_profiles_before_publication() {
    let complete = policy(19, 5, false);
    let entries = complete.registrations();
    let route = || IssuerRoute {
        sender_home: "foreign.test".into(),
        audience: "foreign.test/attachments".into(),
        relay_client_id: "relay".into(),
        introspection_client_id: "inspector".into(),
    };
    let observation = ServicePolicy::new("home.test".into(), vec![entries[6].clone()]).unwrap();
    let issuer = ServicePolicy::new_issuer(
        "home.test".into(),
        "home.test".into(),
        vec![
            (entries[0].clone(), ServiceUse::Relay),
            (entries[1].clone(), ServiceUse::Introspection),
        ],
        vec![route()],
    )
    .unwrap();
    let roles = [
        ServiceUse::Relay,
        ServiceUse::Introspection,
        ServiceUse::ResourceClient,
        ServiceUse::OutboundIntrospection,
        ServiceUse::IssuerEndpoint,
    ];
    let without_publication = ServicePolicy::new_composed(
        "home.test".into(),
        "home.test".into(),
        entries[..5].iter().cloned().zip(roles).collect(),
        vec![route()],
        complete.resource_associations().to_vec(),
    )
    .unwrap();
    for selected in [observation, issuer, without_publication] {
        let fixture = Fixture::new();
        assert!(
            Runtime::initialize_composed_durable(&fixture.policy(), registry(7, 3, true), selected)
                .is_err()
        );
        assert_eq!(fs::read_dir(fixture.policy()).unwrap().count(), 0);
    }
}
#[test]
fn composed_initializer_rejects_foreign_device_bindings_before_publication() {
    let fixture = Fixture::new();
    let local = registry(7, 3, true);
    let mut bindings = local.bindings().to_vec();
    let mut foreign = bindings[0].clone();
    foreign.certificate_sha256 = [10; 32];
    foreign.principal = Principal {
        account: "remote@foreign.test".into(),
        device: 2,
        identity: [10; 32],
    };
    bindings.push(foreign);
    let mixed = Registry::new(7, 3, local.homes().to_vec(), bindings).unwrap();
    assert!(
        Runtime::initialize_composed_durable(&fixture.policy(), mixed, policy(19, 5, false))
            .is_err()
    );
    assert_eq!(fs::read_dir(fixture.policy()).unwrap().count(), 0);
}
