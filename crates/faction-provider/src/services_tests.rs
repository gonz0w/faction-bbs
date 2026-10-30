use super::*;
use crate::authority::CredentialBinding;
use openssl::{
    asn1::Asn1Time,
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    nid::Nid,
    pkey::PKey,
    x509::{X509NameBuilder, extension::SubjectAlternativeName},
};
fn fixture() -> (Authority, X509, Binding) {
    let key = PKey::from_ec_key(
        EcKey::generate(&EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap()).unwrap(),
    )
    .unwrap();
    let mut n = X509NameBuilder::new().unwrap();
    n.append_entry_by_text("CN", "unit.test").unwrap();
    let n = n.build();
    let mut b = X509::builder().unwrap();
    b.set_version(2).unwrap();
    b.set_subject_name(&n).unwrap();
    b.set_issuer_name(&n).unwrap();
    b.set_pubkey(&key).unwrap();
    b.set_not_before(&Asn1Time::from_unix(1).unwrap()).unwrap();
    b.set_not_after(&Asn1Time::from_unix(1000).unwrap())
        .unwrap();
    b.append_extension(
        SubjectAlternativeName::new()
            .dns("unit.test")
            .build(&b.x509v3_context(None, None))
            .unwrap(),
    )
    .unwrap();
    b.sign(&key, MessageDigest::sha256()).unwrap();
    let leaf = b.build();
    let recipient = Principal {
        account: "recipient@local.test".into(),
        device: 1,
        identity: [1; 32],
    };
    let registry = Registry::new(
        1,
        0,
        vec!["local.test".into(), "foreign.test".into()],
        vec![CredentialBinding {
            certificate_sha256: [2; 32],
            principal: recipient.clone(),
            valid_from: 1,
            valid_until: 900,
            approved: true,
            account_enabled: true,
            device_enabled: true,
            generation: 1,
        }],
    )
    .unwrap();
    let policy = ServicePolicy::new(
        "local.test".into(),
        vec![ServiceRegistration {
            home: "foreign.test".into(),
            client_id: "service".into(),
            dns_san: "unit.test".into(),
            certificate_sha256: sha256(&leaf.to_der().unwrap()),
            enabled: true,
            valid_from: 1,
            valid_until: 800,
        }],
    )
    .unwrap();
    let mut a = Authority::new(None);
    a.replace_delegation_policy(registry, policy).unwrap();
    let binding = Binding {
        sender: Principal {
            account: "sender@foreign.test".into(),
            device: 3,
            identity: [3; 32],
        },
        recipient,
        id: [7; 32],
        size: 32,
        created: 10,
        expires: 700,
    };
    (a, leaf, binding)
}
#[test]
fn deterministic_wall_rollback_is_sticky_for_the_actual_service_context() {
    // Synthetic leaf observation here proves timing policy; integration proves mTLS.
    let (a, leaf, b) = fixture();
    let mut c = a.authenticate_service(&leaf, 100).unwrap();
    assert!(
        a.observe_recipient(&mut c, &leaf, b.clone(), PublicationAction::Begin, 110)
            .is_ok()
    );
    assert!(
        a.observe_recipient(&mut c, &leaf, b.clone(), PublicationAction::Begin, 109)
            .is_err()
    );
    assert!(
        a.observe_recipient(&mut c, &leaf, b, PublicationAction::Begin, 111)
            .is_err()
    );
}
#[test]
fn deterministic_registration_leaf_and_object_expiry_never_revive() {
    for expiry in [700, 800, 1000] {
        let (a, leaf, b) = fixture();
        let mut c = a.authenticate_service(&leaf, 100).unwrap();
        assert!(
            a.observe_recipient(&mut c, &leaf, b.clone(), PublicationAction::Begin, expiry)
                .is_err()
        );
        assert!(
            a.observe_recipient(&mut c, &leaf, b, PublicationAction::Begin, 100)
                .is_err()
        );
    }
}
#[test]
fn policy_is_validated_configuration_and_all_homes_consume_existing_invitation_slots() {
    let (a, _, _) = fixture();
    let p = a.service_policy.unwrap();
    let mut e = p.registrations.clone();
    for kind in 0..5 {
        let mut x = e[0].clone();
        match kind {
            0 => x.home = "*.foreign.test".into(),
            1 => x.client_id = "*".into(),
            2 => x.dns_san = "*.unit.test".into(),
            3 => x.certificate_sha256 = [0; 32],
            _ => x.valid_until = x.valid_from,
        };
        assert!(ServicePolicy::new("local.test".into(), vec![x]).is_err());
    }
    e.push(e[0].clone());
    assert!(ServicePolicy::new("local.test".into(), e).is_err());
    let r = Registry::new(2, 0, vec!["local.test".into()], vec![]).unwrap();
    assert!(p.validate_registry(&r).is_err());
    assert!(
        Registry::new(
            1,
            0,
            (0..33).map(|n| format!("home{n}.test")).collect(),
            vec![]
        )
        .is_err()
    );
}
#[test]
fn unchanged_floors_cannot_reuse_a_context_after_authority_invalidation() {
    let (mut a, leaf, b) = fixture();
    let mut c = a.authenticate_service(&leaf, 100).unwrap();
    let r = a.registry.clone();
    let p = a.service_policy.clone();
    a.unavailable();
    a.registry = r;
    a.service_policy = p; // synthetic same-data reinstall, no new floors
    assert!(
        a.observe_recipient(&mut c, &leaf, b, PublicationAction::Begin, 101)
            .is_err()
    );
}
#[test]
fn rechecks_are_independent_of_service_policy_replacement_and_generation() {
    for kind in 0..10 {
        let (mut a, leaf, b) = fixture();
        let mut c = a.authenticate_service(&leaf, 100).unwrap();
        match kind {
            0 => a.registry.as_mut().unwrap().bindings[0].approved = false,
            1 => a.registry.as_mut().unwrap().bindings[0].device_enabled = false,
            2 => a.registry.as_mut().unwrap().bindings[0].account_enabled = false,
            3 => a
                .registry
                .as_mut()
                .unwrap()
                .homes
                .retain(|h| h != "foreign.test"),
            4 => a
                .registry
                .as_mut()
                .unwrap()
                .homes
                .retain(|h| h != "local.test"),
            5 => a.service_policy.as_mut().unwrap().registrations[0].enabled = false,
            6 => a.service_policy.as_mut().unwrap().registrations[0].certificate_sha256 = [9; 32],
            7 => a.service_policy.as_mut().unwrap().registrations[0].dns_san = "wrong.test".into(),
            8 => a.registry.as_mut().unwrap().bindings[0].principal.identity = [8; 32],
            _ => a.registry.as_mut().unwrap().bindings[0].valid_until = 100,
        };
        assert!(
            a.observe_recipient(&mut c, &leaf, b, PublicationAction::Begin, 101)
                .is_err(),
            "isolated mutation {kind}"
        );
    }
}
#[cfg(feature = "durable-policy")]
#[test]
fn identical_durable_install_invalidates_old_service_without_changing_floors() {
    let (mut a, leaf, b) = fixture();
    let mut c = a.authenticate_service(&leaf, 100).unwrap();
    let snapshot = a.snapshot();
    a.install(snapshot);
    assert!(
        a.observe_recipient(&mut c, &leaf, b, PublicationAction::Begin, 101)
            .is_err()
    );
}
