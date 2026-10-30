use super::*;
use faction_provider::{
    issuer::{OperationReference, RetrievalToken},
    retrieval::{RetrievalAction, RetrievalPolicy},
    services::{IssuerRoute, ServicePolicy, ServiceRegistration, ServiceUse},
};
fn registry(c: &Certs, generation: u64) -> Registry {
    let mut entries = c.entries();
    entries.remove(0);
    entries[0].generation = generation;
    Registry::new(
        generation,
        0,
        vec!["home.test".into(), "foreign.test".into()],
        entries,
    )
    .unwrap()
}
fn registration(who: &X509, home: &str, id: &str) -> ServiceRegistration {
    ServiceRegistration {
        home: home.into(),
        client_id: id.into(),
        dns_san: "provider.test".into(),
        certificate_sha256: sha256(&who.to_der().unwrap()),
        enabled: true,
        valid_from: wall_now() - 60,
        valid_until: wall_now() + 3600,
    }
}
fn policy(c: &Certs, inspector: &X509) -> ServicePolicy {
    ServicePolicy::new_issuer(
        "home.test".into(),
        "issuer.test".into(),
        vec![
            (
                registration(&c.alice.0, "home.test", "relay"),
                ServiceUse::Relay,
            ),
            (
                registration(inspector, "foreign.test", "inspector"),
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
fn object() -> Binding {
    let mut b = binding();
    b.sender.account = "sender@foreign.test".into();
    b
}
fn configured(c: &Certs, inspector: &X509) -> Runtime {
    let r = Runtime::new(None);
    r.control()
        .replace_delegation_policy(registry(c, 1), policy(c, inspector))
        .unwrap();
    r.control()
        .replace_retrieval_policy(
            RetrievalPolicy::new(
                "home.test".into(),
                vec![("foreign.test".into(), "foreign.test/attachments".into())],
            )
            .unwrap(),
        )
        .unwrap();
    r
}
fn with_service<T>(
    c: &Certs,
    r: &Runtime,
    who: &(X509, PKey<Private>),
    f: impl FnOnce(&mut faction_provider::ServiceSession) -> T,
) -> T {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::scope(|scope| {
        let peer = scope.spawn(|| {
            c.client(address, Arc::new(AtomicBool::new(false)), Some(who))
                .unwrap()
        });
        let mut session = c
            .server()
            .accept_service(&listener, r, Arc::new(AtomicBool::new(false)))
            .unwrap();
        let _peer = peer.join().unwrap();
        f(&mut session)
    })
}
#[cfg(feature = "durable-policy")]
fn record(c: &Certs, r: &Runtime) -> faction_provider::retrieval::RetrievalRecord {
    with_session(c, r, &c.bob, |mut s| {
        s.retrieval_intent(object(), RetrievalAction::GetChunk { offset: 32 })
            .unwrap()
    })
}
#[test]
fn actual_device_close_relay_issue_close_then_introspection_share_one_runtime() {
    let c = Certs::new();
    let t = wall_now() as i64;
    let inspector = cert(
        Some((&c.ca.0, &c.ca.1)),
        "introspector",
        71,
        t - 60,
        t + 3600,
    );
    let r = configured(&c, &inspector.0);
    let expected = object();
    let record = with_session(&c, &r, &c.bob, |mut s| {
        let rec = s
            .retrieval_intent(expected.clone(), RetrievalAction::GetChunk { offset: 32 })
            .unwrap();
        assert!(r.control().provision_retrieval(&rec).is_err());
        rec
    });
    let reference = r.control().provision_retrieval(&record).unwrap();
    assert_eq!(reference.expose_bytes().len(), 32);
    assert_eq!(
        reference.expose_bytes(),
        r.control()
            .provision_retrieval(&record)
            .unwrap()
            .expose_bytes()
    );
    let issued = with_service(&c, &r, &c.alice, |s| {
        assert!(s.issue_retrieval(&reference, "other").is_err());
        let first = s.issue_retrieval(&reference, "relay").unwrap();
        let retry = s.issue_retrieval(&reference, "relay").unwrap();
        assert_eq!(first.token().expose_bytes(), retry.token().expose_bytes());
        assert_eq!(first.expires_at(), retry.expires_at());
        assert!(retry.expires_in() <= first.expires_in());
        assert!(
            s.observe_recipient(
                object(),
                faction_provider::services::PublicationAction::Begin
            )
            .is_err()
        );
        first
    });
    assert_ne!(reference.expose_bytes(), issued.token().expose_bytes());
    with_service(&c, &r, &inspector, |s| {
        assert!(
            s.introspect_retrieval(issued.token(), "wrong")
                .unwrap()
                .is_none()
        );
        let metadata = s
            .introspect_retrieval(issued.token(), "inspector")
            .unwrap()
            .unwrap();
        assert_eq!(metadata.binding(), &expected);
        assert_eq!(metadata.action(), RetrievalAction::GetChunk { offset: 32 });
        assert_eq!(metadata.audience(), "foreign.test/attachments");
        assert_eq!(metadata.issuer(), "issuer.test");
        assert_eq!(
            metadata.relay_certificate_sha256(),
            sha256(&c.alice.0.to_der().unwrap())
        );
        assert_eq!(metadata.expires_at(), issued.expires_at());
        r.control().reset(1).unwrap();
        assert!(
            s.introspect_retrieval(issued.token(), "inspector")
                .unwrap()
                .is_none()
        );
    });
}
#[test]
fn actual_device_record_requires_issuer_and_recipient_home_alignment() {
    let c = Certs::new();
    let t = wall_now() as i64;
    let inspector = cert(
        Some((&c.ca.0, &c.ca.1)),
        "home-alignment-introspector",
        72,
        t - 60,
        t + 3600,
    );
    for issuer_home in ["home.test", "other.test"] {
        // Both homes are invited, and each public policy is independently valid.
        // The approved recipient and retrieval policy still belong to home.test.
        let mut entries = c.entries();
        entries.remove(0);
        let registry = Registry::new(
            1,
            0,
            vec![
                "home.test".into(),
                "other.test".into(),
                "foreign.test".into(),
            ],
            entries,
        )
        .unwrap();
        let issuer_policy = ServicePolicy::new_issuer(
            issuer_home.into(),
            "issuer.test".into(),
            vec![
                (
                    registration(&c.alice.0, issuer_home, "relay"),
                    ServiceUse::Relay,
                ),
                (
                    registration(&inspector.0, "foreign.test", "inspector"),
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
        .unwrap();
        let r = Runtime::new(None);
        r.control()
            .replace_delegation_policy(registry, issuer_policy)
            .unwrap();
        r.control()
            .replace_retrieval_policy(
                RetrievalPolicy::new(
                    "home.test".into(),
                    vec![("foreign.test".into(), "foreign.test/attachments".into())],
                )
                .unwrap(),
            )
            .unwrap();
        let record = with_session(&c, &r, &c.bob, |mut session| {
            session
                .retrieval_intent(object(), RetrievalAction::GetChunk { offset: 32 })
                .unwrap()
        });
        // with_session has completed normal device-session cleanup before provisioning.
        let provisioned = r.control().provision_retrieval(&record).is_ok();
        assert_eq!(
            provisioned,
            issuer_home == "home.test",
            "recipient home.test with issuer local home {issuer_home}"
        );
    }
}
#[cfg(feature = "durable-policy")]
#[test]
fn actual_v3_publication_drop_reopen_retains_roles_but_not_records_or_tokens() {
    let f = Fixture::new();
    let p = f.root.join("issuer-policy");
    fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
    let c = Certs::new();
    let t = wall_now() as i64;
    let inspector = cert(
        Some((&c.ca.0, &c.ca.1)),
        "introspector",
        72,
        t - 60,
        t + 3600,
    );
    let r = Runtime::initialize_durable(&p, registry(&c, 1)).unwrap();
    let original = fs::read(p.join("provider-policy.snapshot")).unwrap();
    assert_eq!(&original[..8], b"FPPY0001");
    r.control()
        .replace_delegation_policy(registry(&c, 2), policy(&c, &inspector.0))
        .unwrap();
    let bytes = fs::read(p.join("provider-policy.snapshot")).unwrap();
    assert_eq!(&bytes[..8], b"FPPY0003");
    drop(r);
    let r = Runtime::reopen_durable(&p).unwrap();
    assert_eq!(bytes, fs::read(p.join("provider-policy.snapshot")).unwrap());
    assert_eq!(r.policy_summary().unwrap().binding_count, 3);
    r.control()
        .replace_retrieval_policy(
            RetrievalPolicy::new(
                "home.test".into(),
                vec![("foreign.test".into(), "foreign.test/attachments".into())],
            )
            .unwrap(),
        )
        .unwrap();
    let rec = record(&c, &r);
    let reference = r.control().provision_retrieval(&rec).unwrap();
    let issued = with_service(&c, &r, &c.alice, |s| {
        s.issue_retrieval(&reference, "relay").unwrap()
    });
    drop(r);
    let r = Runtime::reopen_durable(&p).unwrap();
    assert!(r.control().provision_retrieval(&rec).is_err());
    with_service(&c, &r, &inspector, |s| {
        assert!(
            s.introspect_retrieval(issued.token(), "inspector")
                .unwrap()
                .is_none()
        )
    });
    r.control().reset(1).unwrap();
    drop(r);
    assert_eq!(
        &fs::read(p.join("provider-policy.snapshot")).unwrap()[..8],
        b"FPPY0003"
    );
}
// These names are part of the authority/privacy API; no public constructor exists.
fn _private_types(_: &OperationReference, _: &RetrievalToken) {}
#[test]
fn denied_issue_and_introspection_attempts_share_the_existing_eight_call_budget() {
    let c = Certs::new();
    let t = wall_now() as i64;
    let inspector = cert(Some((&c.ca.0, &c.ca.1)), "inspector", 73, t - 60, t + 3600);
    let r = configured(&c, &inspector.0);
    let rec = with_session(&c, &r, &c.bob, |mut s| {
        s.retrieval_intent(object(), RetrievalAction::Status)
            .unwrap()
    });
    let reference = r.control().provision_retrieval(&rec).unwrap();
    with_service(&c, &r, &c.alice, |s| {
        for _ in 0..8 {
            assert!(s.issue_retrieval(&reference, "wrong").is_err());
        }
        assert!(matches!(
            s.issue_retrieval(&reference, "relay"),
            Err(Error::Transport(Stage::Limit))
        ));
    });
    let issued = with_service(&c, &r, &c.alice, |s| {
        s.issue_retrieval(&reference, "relay").unwrap()
    });
    with_service(&c, &r, &inspector, |s| {
        for _ in 0..8 {
            assert!(
                s.introspect_retrieval(issued.token(), "wrong")
                    .unwrap()
                    .is_none()
            );
        }
        assert!(matches!(
            s.introspect_retrieval(issued.token(), "inspector"),
            Err(Error::Transport(Stage::Limit))
        ));
    });
}
#[test]
fn explicit_observation_use_remains_valid_but_never_inherits_issuer_uses() {
    let c = Certs::new();
    let t = wall_now() as i64;
    let inspector = cert(Some((&c.ca.0, &c.ca.1)), "inspector", 74, t - 60, t + 3600);
    let observer = cert(Some((&c.ca.0, &c.ca.1)), "observer", 75, t - 60, t + 3600);
    let proposal = ServicePolicy::new_issuer(
        "home.test".into(),
        "issuer.test".into(),
        vec![
            (
                registration(&c.alice.0, "home.test", "relay"),
                ServiceUse::Relay,
            ),
            (
                registration(&inspector.0, "foreign.test", "inspector"),
                ServiceUse::Introspection,
            ),
            (
                registration(&observer.0, "foreign.test", "LEGACY:+/"),
                ServiceUse::Observation,
            ),
        ],
        vec![IssuerRoute {
            sender_home: "foreign.test".into(),
            audience: "foreign.test/attachments".into(),
            relay_client_id: "relay".into(),
            introspection_client_id: "inspector".into(),
        }],
    )
    .unwrap();
    let r = Runtime::new(None);
    r.control()
        .replace_delegation_policy(registry(&c, 1), proposal)
        .unwrap();
    r.control()
        .replace_retrieval_policy(
            RetrievalPolicy::new(
                "home.test".into(),
                vec![("foreign.test".into(), "foreign.test/attachments".into())],
            )
            .unwrap(),
        )
        .unwrap();
    let expected = object();
    let rec = with_session(&c, &r, &c.bob, |mut s| {
        s.retrieval_intent(expected.clone(), RetrievalAction::Status)
            .unwrap()
    });
    let reference = r.control().provision_retrieval(&rec).unwrap();
    let issued = with_service(&c, &r, &c.alice, |s| {
        s.issue_retrieval(&reference, "relay").unwrap()
    });
    with_service(&c, &r, &observer, |s| {
        assert!(s.issue_retrieval(&reference, "LEGACY:+/").is_err());
        assert!(
            s.introspect_retrieval(issued.token(), "LEGACY:+/")
                .unwrap()
                .is_none()
        );
        assert_eq!(
            s.observe_recipient(
                expected.clone(),
                faction_provider::services::PublicationAction::Commit
            )
            .unwrap()
            .binding(),
            &expected
        );
    });
}
#[cfg(feature = "delegation-wire")]
#[path = "../delegation_wire/mod.rs"]
mod wire;
