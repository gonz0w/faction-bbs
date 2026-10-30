use super::*;
use crate::{
    authority::{CredentialBinding, Registry},
    services::{IssuerRoute, ServicePolicy, ServiceUse},
};
use openssl::{
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    nid::Nid,
    pkey::PKey,
    x509::{X509NameBuilder, extension::SubjectAlternativeName},
};
fn certificate(serial: u32, expires: i64) -> X509 {
    let key = PKey::from_ec_key(
        EcKey::generate(&EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap()).unwrap(),
    )
    .unwrap();
    let mut name = X509NameBuilder::new().unwrap();
    name.append_entry_by_text("CN", "unit.test").unwrap();
    let name = name.build();
    let mut b = X509::builder().unwrap();
    b.set_version(2).unwrap();
    b.set_subject_name(&name).unwrap();
    b.set_issuer_name(&name).unwrap();
    b.set_pubkey(&key).unwrap();
    b.set_serial_number(
        &openssl::bn::BigNum::from_u32(serial)
            .unwrap()
            .to_asn1_integer()
            .unwrap(),
    )
    .unwrap();
    b.set_not_before(&Asn1Time::from_unix(1).unwrap()).unwrap();
    b.set_not_after(&Asn1Time::from_unix(expires).unwrap())
        .unwrap();
    b.append_extension(
        SubjectAlternativeName::new()
            .dns("unit.test")
            .build(&b.x509v3_context(None, None))
            .unwrap(),
    )
    .unwrap();
    b.sign(&key, MessageDigest::sha256()).unwrap();
    b.build()
}
fn reg(leaf: &X509, home: &str, id: &str) -> ServiceRegistration {
    ServiceRegistration {
        home: home.into(),
        client_id: id.into(),
        dns_san: "unit.test".into(),
        certificate_sha256: openssl::sha::sha256(&leaf.to_der().unwrap()),
        enabled: true,
        valid_from: 1,
        valid_until: 900,
    }
}
struct Fixture {
    a: Authority,
    device: X509,
    relay: X509,
    inspector: X509,
    base: Instant,
    binding: Binding,
}
impl Fixture {
    fn new() -> Self {
        let device = certificate(1, 1000);
        let relay = certificate(2, 1000);
        let inspector = certificate(3, 1000);
        let base = Instant::now();
        let recipient = Principal {
            account: "recipient@local.test".into(),
            device: 1,
            identity: [1; 32],
        };
        let binding = Binding {
            sender: Principal {
                account: "sender@foreign.test".into(),
                device: 2,
                identity: [2; 32],
            },
            recipient: recipient.clone(),
            id: [7; 32],
            size: 32,
            created: 10,
            expires: 700,
        };
        let registry = Registry::new(
            1,
            0,
            vec!["local.test".into(), "foreign.test".into()],
            vec![CredentialBinding {
                certificate_sha256: openssl::sha::sha256(&device.to_der().unwrap()),
                principal: recipient,
                valid_from: 1,
                valid_until: 900,
                approved: true,
                account_enabled: true,
                device_enabled: true,
                generation: 1,
            }],
        )
        .unwrap();
        let policy = ServicePolicy::new_issuer(
            "local.test".into(),
            "issuer.test".into(),
            vec![
                (reg(&relay, "local.test", "relay"), ServiceUse::Relay),
                (
                    reg(&inspector, "foreign.test", "inspector"),
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
        let mut a = Authority::new(None);
        a.replace_delegation_policy(registry, policy).unwrap();
        a.records.policy = Some(
            RetrievalPolicy::new(
                "local.test".into(),
                vec![("foreign.test".into(), "foreign.test/attachments".into())],
            )
            .unwrap(),
        );
        Self {
            a,
            device,
            relay,
            inspector,
            base,
            binding,
        }
    }
    fn now(&self, millis: u64) -> PreciseMoment {
        PreciseMoment {
            wall: Duration::from_millis(100_900 + millis),
            before: self.base + Duration::from_millis(millis),
            after: self.base + Duration::from_millis(millis),
        }
    }
    fn record(&mut self) -> (Arc<()>, Connection, Handle) {
        let connection = self.a.open_connection();
        let handle = self
            .a
            .authenticate(
                &connection,
                openssl::sha::sha256(&self.device.to_der().unwrap()),
                100,
                0,
            )
            .unwrap();
        let token = self
            .a
            .create_retrieval(
                Source {
                    connection: &connection,
                    handle: &handle,
                    leaf: &self.device,
                    deadline: self.base + Duration::from_secs(10),
                    elapsed_ms: 0,
                },
                self.binding.clone(),
                RetrievalAction::Status,
                Moment {
                    wall: 100,
                    mono: self.base,
                },
            )
            .unwrap();
        (token, connection, handle)
    }
    fn closed_record(&mut self) -> Arc<()> {
        let (token, connection, handle) = self.record();
        self.a.close_connection(&connection);
        drop(handle);
        drop(connection);
        token
    }
    fn provision(&mut self, token: &Arc<()>) -> OperationReference {
        let now = self.now(0);
        self.a.provision_at(token, now, &mut secure_random).unwrap()
    }
    fn issue(&mut self, reference: &OperationReference) -> IssuedToken {
        let now = self.now(0);
        let mut c = self.a.authenticate_service(&self.relay, 100).unwrap();
        self.a
            .issue_at(
                &mut c,
                &self.relay,
                reference.expose_bytes(),
                "relay",
                now,
                &mut secure_random,
            )
            .unwrap()
    }
}
#[test]
fn exact_original_strong_owners_must_end_even_after_closed_true_and_unrelated_admission() {
    let mut f = Fixture::new();
    let (token, connection, handle) = f.record();
    let now = f.now(0);
    assert!(f.a.provision_at(&token, now, &mut secure_random).is_err());
    f.a.close_connection(&connection);
    assert!(connection.closed.load(Ordering::Relaxed));
    assert!(f.a.provision_at(&token, now, &mut secure_random).is_err());
    drop(connection);
    assert!(f.a.provision_at(&token, now, &mut secure_random).is_err());
    drop(handle);
    let unrelated = f.a.open_connection();
    let _handle =
        f.a.authenticate(
            &unrelated,
            openssl::sha::sha256(&f.device.to_der().unwrap()),
            100,
            0,
        )
        .unwrap();
    assert!(f.a.provision_at(&token, now, &mut secure_random).is_ok());
}
#[test]
fn mismatched_recipient_home_refuses_before_rng_or_reference_insertion() {
    let mut f = Fixture::new();
    let mut entries = f.a.registry.as_ref().unwrap().bindings().to_vec();
    for entry in &mut entries {
        entry.generation = 2;
    }
    let registry = Registry::new(
        2,
        0,
        vec![
            "local.test".into(),
            "other.test".into(),
            "foreign.test".into(),
        ],
        entries,
    )
    .unwrap();
    let policy = ServicePolicy::new_issuer(
        "other.test".into(),
        "issuer.test".into(),
        vec![
            (reg(&f.relay, "other.test", "relay"), ServiceUse::Relay),
            (
                reg(&f.inspector, "foreign.test", "inspector"),
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
    f.a = Authority::new(None);
    f.a.replace_delegation_policy(registry, policy).unwrap();
    f.a.records.policy = Some(
        RetrievalPolicy::new(
            "local.test".into(),
            vec![("foreign.test".into(), "foreign.test/attachments".into())],
        )
        .unwrap(),
    );
    let token = f.closed_record();
    let now = f.now(0);
    let mut calls = 0;
    let result = f.a.provision_at(&token, now, &mut |bytes| {
        calls += 1;
        bytes.fill(42);
        Ok(())
    });
    assert_eq!(calls, 0);
    assert!(result.is_err());
    assert_eq!(f.a.records.slots.len(), 1);
    assert!(f.a.records.slots[0].issuer.is_none());
}
#[test]
fn rng_failure_and_four_collisions_are_atomic_and_retries_never_call_rng() {
    let mut f = Fixture::new();
    let token = f.closed_record();
    let now = f.now(0);
    assert!(
        f.a.provision_at(&token, now, &mut |_| Err(BlobError::Denied))
            .is_err()
    );
    assert!(f.a.records.slots[0].issuer.is_none());
    let reference = f.provision(&token);
    let mut c = f.a.authenticate_service(&f.relay, 100).unwrap();
    let mut count = 0;
    assert!(
        f.a.issue_at(
            &mut c,
            &f.relay,
            reference.expose_bytes(),
            "relay",
            now,
            &mut |bytes| {
                count += 1;
                bytes.copy_from_slice(reference.expose_bytes());
                Ok(())
            }
        )
        .is_err()
    );
    assert_eq!(count, 4);
    assert!(
        f.a.records.slots[0]
            .issuer
            .as_ref()
            .unwrap()
            .token
            .is_none()
    );
    assert!(
        f.a.issue_at(
            &mut c,
            &f.relay,
            reference.expose_bytes(),
            "relay",
            now,
            &mut |_| Err(BlobError::Denied)
        )
        .is_err()
    );
    assert!(
        f.a.records.slots[0]
            .issuer
            .as_ref()
            .unwrap()
            .token
            .is_none()
    );
    let first = f.issue(&reference);
    let retry =
        f.a.issue_at(
            &mut c,
            &f.relay,
            reference.expose_bytes(),
            "relay",
            now,
            &mut |_| panic!("retry generated a token"),
        )
        .unwrap();
    assert_eq!(first.token().expose_bytes(), retry.token().expose_bytes());
    assert_eq!(f.a.records.slots.len(), 1);
}
#[test]
fn precise_fractional_lifetime_brackets_remaining_retry_and_subsecond_refusal() {
    let mut f = Fixture::new();
    let token = f.closed_record();
    let reference = f.provision(&token);
    let first = f.issue(&reference);
    assert_eq!(first.expires_at(), 110);
    assert_eq!(first.expires_in(), 9);
    let mut c = f.a.authenticate_service(&f.relay, 100).unwrap();
    let now = f.now(1000);
    let retry =
        f.a.issue_at(
            &mut c,
            &f.relay,
            reference.expose_bytes(),
            "relay",
            now,
            &mut |_| panic!(),
        )
        .unwrap();
    assert_eq!(retry.expires_in(), 8);
    assert_eq!(first.token().expose_bytes(), retry.token().expose_bytes());
    assert_eq!(first.expires_at(), retry.expires_at());
    let near = f.now(8500);
    assert!(
        f.a.issue_at(
            &mut c,
            &f.relay,
            reference.expose_bytes(),
            "relay",
            near,
            &mut |_| panic!()
        )
        .is_err()
    );
    let later = f.now(9200);
    assert!(
        f.a.issue_at(
            &mut c,
            &f.relay,
            reference.expose_bytes(),
            "relay",
            later,
            &mut |_| panic!()
        )
        .is_err()
    );
    assert_eq!(f.a.records.slots.len(), 1);
    assert!(
        f.a.records.slots[0]
            .issuer
            .as_ref()
            .unwrap()
            .token
            .is_some()
    );
    let mut f = Fixture::new();
    let token = f.closed_record();
    let slot = &f.a.records.slots[0];
    assert!(Arc::ptr_eq(&slot.token, &token));
    let mut now = f.now(0);
    now.after += Duration::from_millis(250);
    let (exp, deadline) = lifetime(
        slot,
        &reg(&f.relay, "local.test", "relay"),
        &f.relay,
        900,
        now,
    )
    .unwrap();
    assert_eq!(exp, 110);
    assert_eq!(deadline, f.base + Duration::from_millis(9100));
}
#[test]
fn every_expiry_cap_and_checked_arithmetic_is_conservative() {
    for cap in 0..5 {
        let mut f = Fixture::new();
        f.closed_record();
        let mut relay = reg(&f.relay, "local.test", "relay");
        let mut leaf = f.relay.clone();
        let mut device_until = 900;
        match cap {
            0 => f.a.records.slots[0].binding.expires = 105,
            1 => device_until = 105,
            2 => f.a.records.slots[0].leaf = certificate(9, 105),
            3 => relay.valid_until = 105,
            _ => leaf = certificate(10, 105),
        }
        let (exp, _) =
            lifetime(&f.a.records.slots[0], &relay, &leaf, device_until, f.now(0)).unwrap();
        assert_eq!(exp, 105);
    }
    let mut f = Fixture::new();
    f.closed_record();
    let slot = &f.a.records.slots[0];
    let mut now = f.now(0);
    now.wall = Duration::MAX;
    assert!(
        lifetime(
            slot,
            &reg(&f.relay, "local.test", "relay"),
            &f.relay,
            900,
            now
        )
        .is_err()
    );
    now = f.now(0);
    now.after = f.base + Duration::from_millis(9900);
    assert!(
        lifetime(
            slot,
            &reg(&f.relay, "local.test", "relay"),
            &f.relay,
            900,
            now
        )
        .is_err()
    );
}
#[test]
fn fractional_rollback_and_forward_expiry_are_sticky() {
    for forward in [false, true] {
        let mut f = Fixture::new();
        let token = f.closed_record();
        let reference = f.provision(&token);
        let issued = f.issue(&reference);
        let mut c = f.a.authenticate_service(&f.inspector, 100).unwrap();
        let mut now = f.now(100);
        if forward {
            now.wall = Duration::from_secs(1001)
        } else {
            now.wall = Duration::from_millis(100800)
        }
        assert!(
            f.a.introspect_at(
                &mut c,
                &f.inspector,
                issued.token().expose_bytes(),
                "inspector",
                now
            )
            .is_err()
        );
        assert!(
            f.a.introspect_at(
                &mut c,
                &f.inspector,
                issued.token().expose_bytes(),
                "inspector",
                f.now(200)
            )
            .is_err()
        );
        assert!(f.a.records.slots.is_empty());
    }
}
#[test]
fn all_eight_existing_records_share_the_token_bound_and_no_second_pool() {
    let mut f = Fixture::new();
    let mut records = Vec::new();
    for _ in 0..8 {
        records.push(f.closed_record());
    }
    assert_eq!(f.a.records.slots.len(), 8);
    for record in &records {
        let reference = f.provision(record);
        let issued = f.issue(&reference);
        assert_eq!(issued.token().expose_bytes().len(), 32);
    }
    assert_eq!(f.a.records.slots.len(), 8);
    let connection = f.a.open_connection();
    let handle =
        f.a.authenticate(
            &connection,
            openssl::sha::sha256(&f.device.to_der().unwrap()),
            100,
            0,
        )
        .unwrap();
    assert!(matches!(
        f.a.create_retrieval(
            Source {
                connection: &connection,
                handle: &handle,
                leaf: &f.device,
                deadline: f.base + Duration::from_secs(10),
                elapsed_ms: 0
            },
            f.binding.clone(),
            RetrievalAction::Status,
            Moment {
                wall: 100,
                mono: f.base
            }
        ),
        Err(BlobError::Busy)
    ));
}
#[test]
fn role_scopes_and_wire_identifiers_preserve_legacy_observation_ids() {
    let f = Fixture::new();
    let relay = reg(&f.relay, "local.test", "relay");
    let inspector = reg(&f.inspector, "foreign.test", "inspector");
    let route = IssuerRoute {
        sender_home: "foreign.test".into(),
        audience: "foreign.test/attachments".into(),
        relay_client_id: "relay".into(),
        introspection_client_id: "inspector".into(),
    };
    for kind in 0..8 {
        let mut r = relay.clone();
        let mut i = inspector.clone();
        let mut route = route.clone();
        let mut use_relay = ServiceUse::Relay;
        match kind {
            0 => r.home = "foreign.test".into(),
            1 => i.home = "local.test".into(),
            2 => r.client_id = "Not-wire".into(),
            3 => route.audience = "https://foreign.test/attachments".into(),
            4 => route.sender_home = "another.test".into(),
            5 => route.relay_client_id = "other".into(),
            6 => route.introspection_client_id = "other".into(),
            _ => use_relay = ServiceUse::Observation,
        };
        assert!(
            ServicePolicy::new_issuer(
                "local.test".into(),
                "issuer.test".into(),
                vec![(r, use_relay), (i, ServiceUse::Introspection)],
                vec![route]
            )
            .is_err()
        );
    }
    let mut old = inspector;
    old.client_id = "LEGACY:+/".repeat(20);
    assert!(ServicePolicy::new("local.test".into(), vec![old]).is_ok());
    assert!(ServicePolicy::new("local.test".into(), vec![relay]).is_err());
}
#[test]
fn fresh_introspection_rechecks_every_device_invitation_and_service_boundary() {
    for kind in 0..12 {
        let mut f = Fixture::new();
        let record = f.closed_record();
        let reference = f.provision(&record);
        let issued = f.issue(&reference);
        let mut inspector = f.a.authenticate_service(&f.inspector, 100).unwrap();
        let now = f.now(0);
        let metadata =
            f.a.introspect_at(
                &mut inspector,
                &f.inspector,
                issued.token().expose_bytes(),
                "inspector",
                now,
            )
            .unwrap();
        assert_eq!(metadata.binding(), &f.binding);
        assert_eq!(
            metadata.relay_certificate_sha256(),
            openssl::sha::sha256(&f.relay.to_der().unwrap())
        );
        match kind {
            0 => f.a.registry.as_mut().unwrap().bindings[0].approved = false,
            1 => f.a.registry.as_mut().unwrap().bindings[0].account_enabled = false,
            2 => f.a.registry.as_mut().unwrap().bindings[0].device_enabled = false,
            3 => f.a.registry.as_mut().unwrap().bindings[0].certificate_sha256 = [99; 32],
            4 => f.a.registry.as_mut().unwrap().bindings[0].principal.device += 1,
            5 => {
                f.a.registry.as_mut().unwrap().bindings[0]
                    .principal
                    .identity = [99; 32]
            }
            6 => {
                f.a.registry
                    .as_mut()
                    .unwrap()
                    .homes
                    .retain(|h| h != "foreign.test")
            }
            7 => {
                f.a.registry
                    .as_mut()
                    .unwrap()
                    .homes
                    .retain(|h| h != "local.test")
            }
            8 => f.a.generation += 1,
            9 => f.a.reset_epoch += 1,
            10 => f.a.service_context = Arc::new(()),
            _ => f.a.service_policy = None,
        }
        assert!(
            f.a.introspect_at(
                &mut inspector,
                &f.inspector,
                issued.token().expose_bytes(),
                "inspector",
                now
            )
            .is_err(),
            "boundary {kind}"
        );
    }
}
