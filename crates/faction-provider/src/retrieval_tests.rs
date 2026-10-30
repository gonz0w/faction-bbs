use super::*;
use crate::authority::{CredentialBinding, Registry};
use openssl::{
    asn1::Asn1Time,
    bn::BigNum,
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    nid::Nid,
    pkey::PKey,
    x509::X509NameBuilder,
};
use std::time::Duration;
const AUDIENCE: &str = "https://sender.test/attachments";
fn recipient() -> Principal {
    Principal {
        account: "recipient@home.test".into(),
        device: 7,
        identity: [7; 32],
    }
}
fn binding() -> Binding {
    Binding {
        sender: Principal {
            account: "sender@sender.test".into(),
            device: 9,
            identity: [9; 32],
        },
        recipient: recipient(),
        id: [4; 32],
        size: 32,
        created: 100,
        expires: 200,
    }
}
fn leaf() -> X509 {
    let key = PKey::from_ec_key(
        EcKey::generate(&EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap()).unwrap(),
    )
    .unwrap();
    let mut name = X509NameBuilder::new().unwrap();
    name.append_entry_by_text("CN", "private deterministic fixture")
        .unwrap();
    let name = name.build();
    let mut b = X509::builder().unwrap();
    b.set_version(2).unwrap();
    b.set_serial_number(&BigNum::from_u32(1).unwrap().to_asn1_integer().unwrap())
        .unwrap();
    b.set_subject_name(&name).unwrap();
    b.set_issuer_name(&name).unwrap();
    b.set_pubkey(&key).unwrap();
    b.set_not_before(&Asn1Time::from_unix(50).unwrap()).unwrap();
    b.set_not_after(&Asn1Time::from_unix(250).unwrap()).unwrap();
    b.sign(&key, MessageDigest::sha256()).unwrap();
    b.build()
}
fn authority(leaf: &X509) -> Authority {
    let mut a = Authority::new(Some(
        Registry::new(
            1,
            0,
            vec!["home.test".into(), "sender.test".into()],
            vec![CredentialBinding {
                certificate_sha256: openssl::sha::sha256(&leaf.to_der().unwrap()),
                principal: recipient(),
                valid_from: 50,
                valid_until: 250,
                approved: true,
                account_enabled: true,
                device_enabled: true,
                generation: 1,
            }],
        )
        .unwrap(),
    ));
    a.records.policy = Some(
        RetrievalPolicy::new(
            "home.test".into(),
            vec![("sender.test".into(), AUDIENCE.into())],
        )
        .unwrap(),
    );
    a
}
fn create(a: &mut Authority, leaf: &X509, b: Binding, now: Moment, deadline: Instant) -> Arc<()> {
    let conn = a.open_connection();
    let handle = a
        .authenticate(
            &conn,
            openssl::sha::sha256(&leaf.to_der().unwrap()),
            now.wall,
            0,
        )
        .unwrap();
    let token = a
        .create_retrieval(
            Source {
                connection: &conn,
                handle: &handle,
                leaf,
                deadline,
                elapsed_ms: 0,
            },
            b,
            RetrievalAction::Status,
            now,
        )
        .unwrap();
    a.close_connection(&conn);
    assert!(a.slots.is_empty());
    drop(handle);
    drop(conn);
    token
}
fn check(a: &mut Authority, token: &Arc<()>, b: &Binding, now: Moment) -> bool {
    a.check_retrieval_at(token, b, RetrievalAction::Status, AUDIENCE, now)
        .is_ok()
}
#[test]
fn absolute_original_deadline_is_not_restarted_by_new_cycles_or_retries() {
    let leaf = leaf();
    let mut a = authority(&leaf);
    let start = Instant::now();
    let b = binding();
    let t = create(
        &mut a,
        &leaf,
        b.clone(),
        Moment {
            wall: 100,
            mono: start,
        },
        start + Duration::from_secs(10),
    );
    for sec in 0..10 {
        assert!(check(
            &mut a,
            &t,
            &b,
            Moment {
                wall: 100 + sec,
                mono: start + Duration::from_secs(sec)
            }
        ));
    }
    let later = create(
        &mut a,
        &leaf,
        b.clone(),
        Moment {
            wall: 109,
            mono: start + Duration::from_secs(9),
        },
        start + Duration::from_secs(19),
    );
    assert!(!check(
        &mut a,
        &t,
        &b,
        Moment {
            wall: 110,
            mono: start + Duration::from_secs(10)
        }
    ));
    assert!(check(
        &mut a,
        &later,
        &b,
        Moment {
            wall: 110,
            mono: start + Duration::from_secs(10)
        }
    ));
    assert!(!check(
        &mut a,
        &t,
        &b,
        Moment {
            wall: 111,
            mono: start + Duration::from_secs(11)
        }
    ));
}
#[test]
fn wall_and_monotonic_rollback_are_sticky_without_tombstone_growth() {
    for wall_rollback in [true, false] {
        let leaf = leaf();
        let mut a = authority(&leaf);
        let start = Instant::now();
        let b = binding();
        let t = create(
            &mut a,
            &leaf,
            b.clone(),
            Moment {
                wall: 100,
                mono: start,
            },
            start + Duration::from_secs(10),
        );
        assert!(check(
            &mut a,
            &t,
            &b,
            Moment {
                wall: 102,
                mono: start + Duration::from_secs(2)
            }
        ));
        let backwards = if wall_rollback {
            Moment {
                wall: 101,
                mono: start + Duration::from_secs(3),
            }
        } else {
            Moment {
                wall: 103,
                mono: start + Duration::from_secs(1),
            }
        };
        assert!(!check(&mut a, &t, &b, backwards));
        assert!(a.records.slots.is_empty());
        assert!(!check(
            &mut a,
            &t,
            &b,
            Moment {
                wall: 104,
                mono: start + Duration::from_secs(4)
            }
        ));
        let fresh = create(
            &mut a,
            &leaf,
            b.clone(),
            Moment {
                wall: 104,
                mono: start + Duration::from_secs(4),
            },
            start + Duration::from_secs(14),
        );
        assert!(check(
            &mut a,
            &fresh,
            &b,
            Moment {
                wall: 104,
                mono: start + Duration::from_secs(4)
            }
        ));
    }
}
#[test]
fn object_and_actual_certificate_expiry_prune_and_cannot_resurrect() {
    for certificate in [false, true] {
        let leaf = leaf();
        let mut a = authority(&leaf);
        let start = Instant::now();
        let mut b = binding();
        if certificate {
            b.expires = 300;
            a.registry.as_mut().unwrap().bindings[0].valid_until = 1000;
        }
        let t = create(
            &mut a,
            &leaf,
            b.clone(),
            Moment {
                wall: 100,
                mono: start,
            },
            start + Duration::from_secs(10),
        );
        assert!(check(
            &mut a,
            &t,
            &b,
            Moment {
                wall: 101,
                mono: start + Duration::from_secs(1)
            }
        ));
        assert!(!check(
            &mut a,
            &t,
            &b,
            Moment {
                wall: if certificate { 250 } else { 200 },
                mono: start + Duration::from_secs(2)
            }
        ));
        assert!(a.records.slots.is_empty());
        assert!(!check(
            &mut a,
            &t,
            &b,
            Moment {
                wall: 101,
                mono: start + Duration::from_secs(3)
            }
        ));
        assert!(!check(
            &mut a,
            &t,
            &b,
            Moment {
                wall: 251,
                mono: start + Duration::from_secs(4)
            }
        ));
    }
}
#[test]
fn bounded_pruning_recovers_all_eight_slots_without_deadline_extension() {
    let leaf = leaf();
    let mut a = authority(&leaf);
    let start = Instant::now();
    let b = binding();
    let mut old = Vec::new();
    for _ in 0..MAX_RECORDS {
        old.push(create(
            &mut a,
            &leaf,
            b.clone(),
            Moment {
                wall: 100,
                mono: start,
            },
            start + Duration::from_secs(10),
        ));
    }
    assert_eq!(a.records.slots.len(), 8);
    let conn = a.open_connection();
    let handle = a
        .authenticate(&conn, openssl::sha::sha256(&leaf.to_der().unwrap()), 100, 0)
        .unwrap();
    assert!(matches!(
        a.create_retrieval(
            Source {
                connection: &conn,
                handle: &handle,
                leaf: &leaf,
                deadline: start + Duration::from_secs(10),
                elapsed_ms: 0
            },
            b.clone(),
            RetrievalAction::Status,
            Moment {
                wall: 100,
                mono: start
            }
        ),
        Err(BlobError::Busy)
    ));
    a.close_connection(&conn);
    for _ in 0..MAX_RECORDS {
        create(
            &mut a,
            &leaf,
            b.clone(),
            Moment {
                wall: 110,
                mono: start + Duration::from_secs(10),
            },
            start + Duration::from_secs(20),
        );
    }
    assert_eq!(a.records.slots.len(), 8);
    for t in old {
        assert!(!check(
            &mut a,
            &t,
            &b,
            Moment {
                wall: 110,
                mono: start + Duration::from_secs(10)
            }
        ));
    }
}
#[test]
fn every_binding_field_action_offset_and_configured_audience_is_exact() {
    let leaf = leaf();
    let mut a = authority(&leaf);
    let now = Moment::now();
    let now = Moment { wall: 100, ..now };
    let b = binding();
    let t = create(
        &mut a,
        &leaf,
        b.clone(),
        now,
        now.mono + Duration::from_secs(10),
    );
    for field in 0..10 {
        let mut changed = b.clone();
        match field {
            0 => changed.id[0] ^= 1,
            1 => changed.size += 1,
            2 => changed.created += 1,
            3 => changed.expires += 1,
            4 => changed.sender.account = "other@sender.test".into(),
            5 => changed.sender.device += 1,
            6 => changed.sender.identity[31] ^= 1,
            7 => changed.recipient.account = "other@home.test".into(),
            8 => changed.recipient.device += 1,
            _ => changed.recipient.identity[31] ^= 1,
        };
        assert!(!check(&mut a, &t, &changed, now));
    }
    assert!(
        a.check_retrieval_at(
            &t,
            &b,
            RetrievalAction::GetChunk { offset: 0 },
            AUDIENCE,
            now
        )
        .is_err()
    );
    assert!(
        a.check_retrieval_at(&t, &b, RetrievalAction::Status, "other", now)
            .is_err()
    );
    assert!(check(&mut a, &t, &b, now));
}
#[cfg(feature = "durable-policy")]
#[test]
fn snapshot_restore_excludes_transient_policy_and_records_install_invalidates() {
    let leaf = leaf();
    let mut a = authority(&leaf);
    let now = Moment {
        wall: 100,
        mono: Instant::now(),
    };
    let b = binding();
    let t = create(
        &mut a,
        &leaf,
        b.clone(),
        now,
        now.mono + Duration::from_secs(10),
    );
    let snapshot = a.snapshot();
    let mut restored = Authority::restore(snapshot.clone()).unwrap();
    assert!(restored.records.policy.is_none());
    assert!(restored.records.slots.is_empty());
    assert!(!check(&mut restored, &t, &b, now));
    a.install(snapshot);
    assert!(a.records.slots.is_empty());
    assert!(!check(&mut a, &t, &b, now));
}

#[test]
fn creation_spends_the_remaining_original_cycle_and_registry_expiry_is_sticky() {
    let leaf = leaf();
    let mut a = authority(&leaf);
    let start = Instant::now();
    let b = binding();
    let t = create(
        &mut a,
        &leaf,
        b.clone(),
        Moment {
            wall: 107,
            mono: start + Duration::from_secs(7),
        },
        start + Duration::from_secs(10),
    );
    assert!(check(
        &mut a,
        &t,
        &b,
        Moment {
            wall: 109,
            mono: start + Duration::from_secs(9)
        }
    ));
    assert!(!check(
        &mut a,
        &t,
        &b,
        Moment {
            wall: 110,
            mono: start + Duration::from_secs(10)
        }
    ));
    let mut a = authority(&leaf);
    a.registry.as_mut().unwrap().bindings[0].valid_until = 105;
    let t = create(
        &mut a,
        &leaf,
        b.clone(),
        Moment {
            wall: 100,
            mono: start,
        },
        start + Duration::from_secs(10),
    );
    assert!(!check(
        &mut a,
        &t,
        &b,
        Moment {
            wall: 105,
            mono: start + Duration::from_secs(1)
        }
    ));
    assert!(!check(
        &mut a,
        &t,
        &b,
        Moment {
            wall: 104,
            mono: start + Duration::from_secs(2)
        }
    ));
    assert!(!check(
        &mut a,
        &t,
        &b,
        Moment {
            wall: 106,
            mono: start + Duration::from_secs(3)
        }
    ));
}
#[test]
fn no_forged_source_certificate_connection_or_stale_handle_can_mint() {
    let leaf = leaf();
    let wrong_leaf = super::tests::leaf();
    let mut a = authority(&leaf);
    let start = Instant::now();
    let now = Moment {
        wall: 100,
        mono: start,
    };
    let deadline = start + Duration::from_secs(10);
    let b = binding();
    let conn = a.open_connection();
    let other = a.open_connection();
    let handle = a
        .authenticate(&conn, openssl::sha::sha256(&leaf.to_der().unwrap()), 100, 0)
        .unwrap();
    assert!(
        a.create_retrieval(
            Source {
                connection: &conn,
                handle: &handle,
                leaf: &wrong_leaf,
                deadline,
                elapsed_ms: 0
            },
            b.clone(),
            RetrievalAction::Status,
            now
        )
        .is_err()
    );
    assert!(
        a.create_retrieval(
            Source {
                connection: &other,
                handle: &handle,
                leaf: &leaf,
                deadline,
                elapsed_ms: 0
            },
            b.clone(),
            RetrievalAction::Status,
            now
        )
        .is_err()
    );
    a.close_connection(&conn);
    assert!(a.slots.is_empty());
    assert!(
        a.create_retrieval(
            Source {
                connection: &conn,
                handle: &handle,
                leaf: &leaf,
                deadline,
                elapsed_ms: 0
            },
            b,
            RetrievalAction::Status,
            now
        )
        .is_err()
    );
    assert!(a.records.slots.is_empty());
}
#[cfg(feature = "delegation-verifier")]
#[test]
fn physical_delegation_reservation_survives_invalidation_reset_and_old_weak_drop_without_an_extra_slot_pool()
 {
    let leaf = leaf();
    let mut a = authority(&leaf);
    let mono = Instant::now();
    let now = Moment { wall: 100, mono };
    let deadline = mono + Duration::from_secs(10);
    let records: Vec<_> = (0..7)
        .map(|_| create(&mut a, &leaf, binding(), now, deadline))
        .collect();
    let lease = a.records.reserve_delegated(now).unwrap();
    assert_eq!(a.records.charged_len(), 8);
    assert!(matches!(
        a.records.reserve_delegated(now),
        Err(BlobError::Busy)
    ));
    let connection = a.open_connection();
    let handle = a
        .authenticate(
            &connection,
            openssl::sha::sha256(&leaf.to_der().unwrap()),
            100,
            0,
        )
        .unwrap();
    assert!(matches!(
        a.create_retrieval(
            Source {
                connection: &connection,
                handle: &handle,
                leaf: &leaf,
                deadline,
                elapsed_ms: 0
            },
            binding(),
            RetrievalAction::Status,
            now
        ),
        Err(BlobError::Busy)
    ));
    a.close_connection(&connection);
    drop(handle);
    drop(connection);
    a.records.invalidate();
    assert_eq!(a.records.charged_len(), 1);
    assert!(matches!(
        a.records.reserve_delegated(now),
        Err(BlobError::Busy)
    ));
    a.reset(1).unwrap();
    assert!(matches!(
        a.records.reserve_delegated(now),
        Err(BlobError::Busy)
    ));
    let old = Arc::downgrade(&lease);
    drop(lease);
    let newer = a.records.reserve_delegated(now).unwrap();
    drop(old);
    assert!(matches!(
        a.records.reserve_delegated(now),
        Err(BlobError::Busy)
    ));
    drop(newer);
    assert!(a.records.reserve_delegated(now).is_ok());
    drop(records);
}
