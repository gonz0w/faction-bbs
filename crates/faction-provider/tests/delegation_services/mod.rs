use super::*;
use faction_provider::services::{PublicationAction, ServicePolicy, ServiceRegistration};
fn services(c: &Certs) -> ServicePolicy {
    ServicePolicy::new(
        "home.test".into(),
        vec![ServiceRegistration {
            home: "foreign.test".into(),
            client_id: "sender-service".into(),
            dns_san: "provider.test".into(),
            certificate_sha256: sha256(&c.alice.0.to_der().unwrap()),
            enabled: true,
            valid_from: wall_now() - 60,
            valid_until: wall_now() + 3600,
        }],
    )
    .unwrap()
}
fn policy(c: &Certs, generation: u64) -> Registry {
    let mut e = c.entries();
    e.remove(0);
    for x in &mut e {
        x.generation = generation;
    }
    Registry::new(
        generation,
        0,
        vec!["home.test".into(), "foreign.test".into()],
        e,
    )
    .unwrap()
}
fn foreign_binding() -> Binding {
    let mut b = binding();
    b.sender.account = "sender@foreign.test".into();
    b
}
#[test]
fn real_service_observes_foreign_unenrolled_sender_without_a_device_actor() {
    let c = Certs::new();
    let runtime = Runtime::new(None);
    runtime
        .control()
        .replace_delegation_policy(policy(&c, 1), services(&c))
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::scope(|s| {
        let client = s.spawn(|| {
            c.client(addr, Arc::new(AtomicBool::new(false)), Some(&c.alice))
                .unwrap()
        });
        let mut session = c
            .server()
            .accept_service(&listener, &runtime, Arc::new(AtomicBool::new(false)))
            .unwrap();
        let _client = client.join().unwrap();
        let b = foreign_binding();
        let observation = session
            .observe_recipient(b.clone(), PublicationAction::Begin)
            .unwrap();
        assert_eq!(observation.binding(), &b);
        assert_eq!(observation.action(), PublicationAction::Begin);
        assert_eq!(observation.service_home(), "foreign.test");
        assert_eq!(observation.client_id(), "sender-service");
        runtime.control().replace_registry(policy(&c, 2)).unwrap();
        assert!(
            session
                .observe_recipient(b, PublicationAction::Begin)
                .is_err()
        );
    });
}
fn with_service(
    c: &Certs,
    runtime: &Runtime,
    f: impl FnOnce(&mut faction_provider::ServiceSession),
) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::scope(|s| {
        let client = s.spawn(|| {
            c.client(addr, Arc::new(AtomicBool::new(false)), Some(&c.alice))
                .unwrap()
        });
        let mut session = c
            .server()
            .accept_service(&listener, runtime, Arc::new(AtomicBool::new(false)))
            .unwrap();
        let _client = client.join().unwrap();
        f(&mut session);
    });
}
fn enabled(c: &Certs) -> Runtime {
    let r = Runtime::new(None);
    r.control()
        .replace_delegation_policy(policy(c, 1), services(c))
        .unwrap();
    r
}
#[test]
fn exact_selector_and_recipient_tuple_are_observed_and_each_attempt_is_charged() {
    let c = Certs::new();
    let r = enabled(&c);
    with_service(&c, &r, |s| {
        for action in [
            PublicationAction::Begin,
            PublicationAction::PutChunk {
                offset: 0,
                length: 16,
                ciphertext_sha256: [8; 32],
            },
            PublicationAction::Commit,
            PublicationAction::Status,
            PublicationAction::Delete,
        ] {
            let b = foreign_binding();
            let o = s.observe_recipient(b.clone(), action).unwrap();
            assert_eq!(o.binding(), &b);
            assert_eq!(o.action(), action);
            assert_eq!(
                o.recipient_certificate_sha256(),
                sha256(&c.bob.0.to_der().unwrap())
            );
        }
        for mutate in [0, 1, 2] {
            let mut b = foreign_binding();
            match mutate {
                0 => b.recipient.account = "other@home.test".into(),
                1 => b.recipient.device += 1,
                _ => b.recipient.identity[31] ^= 1,
            };
            assert!(s.observe_recipient(b, PublicationAction::Status).is_err())
        }
        assert!(matches!(
            s.observe_recipient(foreign_binding(), PublicationAction::Begin),
            Err(Error::Transport(Stage::Limit))
        ));
    });
}
#[test]
fn wrong_sender_home_local_scope_object_time_and_chunk_bounds_deny() {
    let c = Certs::new();
    let r = enabled(&c);
    with_service(&c, &r, |s| {
        for kind in 0..6 {
            let mut b = foreign_binding();
            let mut a = PublicationAction::Begin;
            match kind {
                0 => b.sender.account = "alice@home.test".into(),
                1 => b.recipient.account = "bob@foreign.test".into(),
                2 => b.created = wall_now() + 20,
                3 => {
                    b.created = wall_now() - 100;
                    b.expires = wall_now();
                }
                4 => {
                    a = PublicationAction::PutChunk {
                        offset: 20,
                        length: 16,
                        ciphertext_sha256: [9; 32],
                    }
                }
                _ => {
                    a = PublicationAction::PutChunk {
                        offset: 0,
                        length: 0,
                        ciphertext_sha256: [9; 32],
                    }
                }
            };
            assert!(s.observe_recipient(b, a).is_err());
        }
        let mut b = foreign_binding();
        b.id[0] ^= 1;
        let o = s
            .observe_recipient(
                b.clone(),
                PublicationAction::PutChunk {
                    offset: 16,
                    length: 16,
                    ciphertext_sha256: [1; 32],
                },
            )
            .unwrap();
        assert_eq!(o.binding(), &b);
        let o2 = s
            .observe_recipient(
                b,
                PublicationAction::PutChunk {
                    offset: 16,
                    length: 16,
                    ciphertext_sha256: [2; 32],
                },
            )
            .unwrap();
        assert_ne!(o.action(), o2.action());
    });
}
#[test]
fn current_recipient_approval_certificate_home_and_generation_changes_deny_old_service() {
    for kind in 0..8 {
        let c = Certs::new();
        let r = enabled(&c);
        with_service(&c, &r, |s| {
            assert!(
                s.observe_recipient(foreign_binding(), PublicationAction::Begin)
                    .is_ok()
            );
            if kind == 6 {
                r.control().reset(1).unwrap();
            } else if kind == 7 {
                r.stop();
            } else {
                let mut entries = policy(&c, 2).bindings().to_vec();
                match kind {
                    0 => entries[0].approved = false,
                    1 => entries[0].account_enabled = false,
                    2 => entries[0].device_enabled = false,
                    3 => entries[0].certificate_sha256 = [90; 32],
                    4 => entries[0].valid_until = wall_now(),
                    _ => (),
                };
                let homes = if kind == 5 {
                    vec!["home.test".into()]
                } else {
                    vec!["home.test".into(), "foreign.test".into()]
                };
                r.control()
                    .replace_registry(Registry::new(2, 0, homes, entries).unwrap())
                    .unwrap();
            }
            assert!(
                s.observe_recipient(foreign_binding(), PublicationAction::Begin)
                    .is_err()
            );
        });
    }
}
#[test]
fn fresh_service_never_bypasses_unapproved_or_uninvited_recipient() {
    for kind in 0..5 {
        let c = Certs::new();
        let r = Runtime::new(None);
        let mut e = policy(&c, 1).bindings().to_vec();
        match kind {
            0 => e[0].approved = false,
            1 => e[0].account_enabled = false,
            2 => e[0].device_enabled = false,
            3 => e[0].valid_from = wall_now() + 20,
            _ => e[0].valid_until = wall_now(),
        };
        let reg = Registry::new(1, 0, vec!["home.test".into(), "foreign.test".into()], e).unwrap();
        r.control()
            .replace_delegation_policy(reg, services(&c))
            .unwrap();
        with_service(&c, &r, |s| {
            assert!(
                s.observe_recipient(foreign_binding(), PublicationAction::Begin)
                    .is_err()
            )
        });
    }
}
#[test]
fn actual_tls_service_refuses_wrong_san_pin_disabled_unregistered_device_and_bad_ca() {
    for kind in 0..7 {
        let c = Certs::new();
        let other = Certs::new();
        let r = Runtime::new(None);
        let mut regs = services(&c).registrations().to_vec();
        match kind {
            0 => regs[0].dns_san = "other.test".into(),
            1 => regs[0].certificate_sha256 = [90; 32],
            2 => regs[0].enabled = false,
            3 => regs.clear(),
            4 => (),
            5 => (),
            _ => regs[0].valid_until = wall_now(),
        };
        r.control()
            .replace_delegation_policy(
                policy(&c, 1),
                ServicePolicy::new("home.test".into(), regs).unwrap(),
            )
            .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::scope(|scope| {
            let client = scope.spawn(|| {
                c.client(
                    addr,
                    Arc::new(AtomicBool::new(false)),
                    Some(if kind == 4 {
                        &c.bob
                    } else if kind == 5 {
                        &other.alice
                    } else {
                        &c.alice
                    }),
                )
            });
            assert!(
                c.server()
                    .accept_service(&listener, &r, Arc::new(AtomicBool::new(false)))
                    .is_err()
            );
            let _ = client.join().unwrap();
        });
        // Failure releases the original lane, with a real device readmission.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::scope(|scope| {
            let client =
                scope.spawn(|| c.client(addr, Arc::new(AtomicBool::new(false)), Some(&c.bob)));
            let device = c
                .server()
                .accept(&listener, &r, Arc::new(AtomicBool::new(false)))
                .unwrap();
            drop(device);
            let _ = client.join().unwrap();
        });
    }
}
#[test]
fn service_and_device_share_one_lane_and_release_it_in_both_directions() {
    let c = Certs::new();
    let r = enabled(&c);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    with_service(&c, &r, |_| {
        assert!(matches!(
            c.server()
                .accept(&listener, &r, Arc::new(AtomicBool::new(false))),
            Err(Stage::Denied)
        ));
        assert!(matches!(
            c.server()
                .accept_service(&listener, &r, Arc::new(AtomicBool::new(false))),
            Err(Stage::Denied)
        ));
    });
    let addr = listener.local_addr().unwrap();
    thread::scope(|scope| {
        let client = scope.spawn(|| {
            c.client(addr, Arc::new(AtomicBool::new(false)), Some(&c.bob))
                .unwrap()
        });
        let device = c
            .server()
            .accept(&listener, &r, Arc::new(AtomicBool::new(false)))
            .unwrap();
        let _client = client.join().unwrap();
        assert!(matches!(
            c.server()
                .accept_service(&listener, &r, Arc::new(AtomicBool::new(false))),
            Err(Stage::Denied)
        ));
        drop(device);
    });
    with_service(&c, &r, |s| {
        assert!(
            s.observe_recipient(foreign_binding(), PublicationAction::Begin)
                .is_ok()
        )
    });
}
#[test]
fn invalid_combined_proposal_preserves_live_policy_and_typed_history_never_rebinds() {
    let c = Certs::new();
    let r = enabled(&c);
    with_service(&c, &r, |s| {
        let mut reg = services(&c).registrations().to_vec();
        reg[0].certificate_sha256 = sha256(&c.bob.0.to_der().unwrap());
        assert!(
            r.control()
                .replace_delegation_policy(
                    policy(&c, 2),
                    ServicePolicy::new("home.test".into(), reg).unwrap()
                )
                .is_err()
        );
        assert!(
            s.observe_recipient(foreign_binding(), PublicationAction::Begin)
                .is_ok()
        );
        let mut reg = services(&c).registrations().to_vec();
        reg[0].client_id = "rebound".into();
        assert!(
            r.control()
                .replace_delegation_policy(
                    policy(&c, 2),
                    ServicePolicy::new("home.test".into(), reg).unwrap()
                )
                .is_err()
        );
        assert!(
            s.observe_recipient(foreign_binding(), PublicationAction::Begin)
                .is_ok()
        );
    });
    r.control().replace_registry(policy(&c, 2)).unwrap();
    let mut e = policy(&c, 3).bindings().to_vec();
    e[0].certificate_sha256 = sha256(&c.alice.0.to_der().unwrap());
    assert!(
        r.control()
            .replace_registry(
                Registry::new(3, 0, vec!["home.test".into(), "foreign.test".into()], e).unwrap()
            )
            .is_err()
    );
}
#[test]
fn aggregate_active_and_lifetime_32_slots_do_not_create_a_service_allowance() {
    let c = Certs::new();
    let r = Runtime::new(None);
    let entries = |g: u64, count: u8| {
        (1..=count)
            .map(|n| CredentialBinding {
                certificate_sha256: [n; 32],
                principal: Principal {
                    account: format!("user{n}@home.test"),
                    device: n.into(),
                    identity: [n; 32],
                },
                valid_from: 1,
                valid_until: wall_now() + 3600,
                approved: true,
                account_enabled: true,
                device_enabled: true,
                generation: g,
            })
            .collect::<Vec<_>>()
    };
    let registry = |g, n| {
        Registry::new(
            g,
            0,
            vec!["home.test".into(), "foreign.test".into()],
            entries(g, n),
        )
        .unwrap()
    };
    assert!(
        r.control()
            .replace_delegation_policy(registry(1, 32), services(&c))
            .is_err()
    );
    r.control()
        .replace_delegation_policy(registry(1, 31), services(&c))
        .unwrap();
    let mut reg = services(&c).registrations().to_vec();
    reg[0].certificate_sha256 = [99; 32];
    assert!(
        r.control()
            .replace_delegation_policy(
                registry(2, 0),
                ServicePolicy::new("home.test".into(), reg).unwrap()
            )
            .is_err()
    );
    r.control().reset(1).unwrap();
    assert!(
        r.control()
            .replace_delegation_policy(registry(3, 0), services(&c))
            .is_err()
    ); // reset floor 1
    let restore = Registry::new(
        3,
        1,
        vec!["home.test".into(), "foreign.test".into()],
        entries(3, 0),
    )
    .unwrap();
    r.control()
        .replace_delegation_policy(restore, services(&c))
        .unwrap();
}
#[test]
fn stalled_service_handshake_cancellation_stop_and_runtime_drop_deny_and_release() {
    for stop in [false, true] {
        let c = Certs::new();
        let r = enabled(&c);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let raw = TcpStream::connect(addr).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let control = r.control();
        let began = Instant::now();
        thread::scope(|scope| {
            let cancel2 = cancel.clone();
            scope.spawn(move || {
                thread::sleep(Duration::from_millis(50));
                if stop {
                    control.stop().unwrap();
                } else {
                    cancel2.store(true, Ordering::SeqCst);
                }
            });
            assert!(c.server().accept_service(&listener, &r, cancel).is_err());
        });
        assert!(began.elapsed() < Duration::from_secs(2));
        drop(raw);
        if !stop {
            with_service(&c, &r, |s| {
                assert!(
                    s.observe_recipient(foreign_binding(), PublicationAction::Begin)
                        .is_ok()
                )
            });
        }
    }
    let c = Certs::new();
    let r = enabled(&c);
    with_service(&c, &r, |s| {
        r.stop();
        assert!(
            s.observe_recipient(foreign_binding(), PublicationAction::Begin)
                .is_err()
        );
    });
    let r = enabled(&c);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::scope(|scope| {
        let client = scope.spawn(|| {
            c.client(addr, Arc::new(AtomicBool::new(false)), Some(&c.alice))
                .unwrap()
        });
        let mut service = c
            .server()
            .accept_service(&listener, &r, Arc::new(AtomicBool::new(false)))
            .unwrap();
        let _client = client.join().unwrap();
        drop(r);
        assert!(
            service
                .observe_recipient(foreign_binding(), PublicationAction::Begin)
                .is_err()
        );
    });
}
#[test]
fn service_original_admission_and_cycle_deadlines_are_enforced() {
    let c = Certs::new();
    let r = enabled(&c);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let raw = TcpStream::connect(addr).unwrap();
    let began = Instant::now();
    assert!(
        c.server()
            .accept_service(&listener, &r, Arc::new(AtomicBool::new(false)))
            .is_err()
    );
    assert!(began.elapsed() >= HANDSHAKE_LIMIT);
    assert!(began.elapsed() < HANDSHAKE_LIMIT + Duration::from_secs(2));
    drop(raw);
    with_service(&c, &r, |s| {
        thread::sleep(CYCLE_LIMIT + Duration::from_millis(20));
        assert!(matches!(
            s.observe_recipient(foreign_binding(), PublicationAction::Begin),
            Err(Error::Transport(Stage::Deadline))
        ));
        assert!(
            s.observe_recipient(foreign_binding(), PublicationAction::Begin)
                .is_err()
        );
    });
}
#[cfg(feature = "durable-policy")]
#[test]
fn durable_services_survive_real_atomic_publication_and_reopen_without_live_authority() {
    let f = Fixture::new();
    let path = f.root.join("policy");
    fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
    let c = Certs::new();
    let r = Runtime::initialize_durable(&path, policy(&c, 1)).unwrap();
    let original = fs::read(path.join("provider-policy.snapshot")).unwrap();
    assert_eq!(&original[..8], b"FPPY0001");
    r.control()
        .replace_delegation_policy(policy(&c, 2), services(&c))
        .unwrap();
    drop(r);
    let r = Runtime::reopen_durable(&path).unwrap();
    with_service(&c, &r, |s| {
        let o = s
            .observe_recipient(foreign_binding(), PublicationAction::Begin)
            .unwrap();
        assert_eq!(o.client_id(), "sender-service");
        assert_eq!(o.generation(), 2);
    });
    let bytes = fs::read(path.join("provider-policy.snapshot")).unwrap();
    assert_eq!(&bytes[..8], b"FPPY0002");
}
#[cfg(feature = "durable-policy")]
#[test]
fn durable_reopen_requires_fresh_service_and_device_only_update_retains_all_tombstones() {
    let f = Fixture::new();
    let path = f.root.join("policy");
    fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
    let c = Certs::new();
    let r = Runtime::initialize_durable(&path, policy(&c, 1)).unwrap();
    r.control()
        .replace_delegation_policy(policy(&c, 2), services(&c))
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::scope(|scope| {
        let client = scope.spawn(|| {
            c.client(addr, Arc::new(AtomicBool::new(false)), Some(&c.alice))
                .unwrap()
        });
        let mut old = c
            .server()
            .accept_service(&listener, &r, Arc::new(AtomicBool::new(false)))
            .unwrap();
        let _client = client.join().unwrap();
        let observation = old
            .observe_recipient(foreign_binding(), PublicationAction::Begin)
            .unwrap();
        let control = r.control();
        drop(r);
        let r = Runtime::reopen_durable(&path).unwrap();
        assert!(control.reset(1).is_err());
        assert!(
            old.observe_recipient(foreign_binding(), PublicationAction::Begin)
                .is_err()
        );
        assert_eq!(observation.generation(), 2);
        drop(old);
        with_service(&c, &r, |s| {
            assert!(
                s.observe_recipient(foreign_binding(), PublicationAction::Begin)
                    .is_ok()
            )
        });
        let original = fs::read(path.join("provider-policy.snapshot")).unwrap();
        let mut conflicting = services(&c).registrations().to_vec();
        conflicting[0].certificate_sha256 = sha256(&c.bob.0.to_der().unwrap());
        assert!(
            r.control()
                .replace_delegation_policy(
                    policy(&c, 3),
                    ServicePolicy::new("home.test".into(), conflicting).unwrap()
                )
                .is_err()
        );
        assert_eq!(
            fs::read(path.join("provider-policy.snapshot")).unwrap(),
            original
        );
        with_service(&c, &r, |s| {
            assert!(
                s.observe_recipient(foreign_binding(), PublicationAction::Begin)
                    .is_ok()
            )
        });
        r.control().replace_registry(policy(&c, 3)).unwrap();
        assert_eq!(r.policy_summary().unwrap().history_count, 2);
        drop(r);
        let r = Runtime::reopen_durable(&path).unwrap();
        let mut rebound = services(&c).registrations().to_vec();
        rebound[0].client_id = "other".into();
        assert!(
            r.control()
                .replace_delegation_policy(
                    policy(&c, 4),
                    ServicePolicy::new("home.test".into(), rebound).unwrap()
                )
                .is_err()
        );
        assert_eq!(r.policy_summary().unwrap().binding_count, 1);
    });
}
#[test]
fn missing_or_mismatched_service_keys_never_create_service_authority() {
    let c = Certs::new();
    let r = enabled(&c);
    for kind in 0..3 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel2 = cancel.clone();
        thread::scope(|scope| {
            let client = scope.spawn(|| {
                let identity = match kind {
                    0 => None,
                    1 => Some((&c.alice.0, None)),
                    _ => Some((&c.alice.0, Some(&c.bob.1))),
                };
                let result = Client::connect(
                    addr,
                    "provider.test",
                    sha256(&c.server.0.to_der().unwrap()),
                    &c.ca.0,
                    identity,
                    cancel2.clone(),
                );
                if kind > 0 {
                    assert!(result.is_err());
                    cancel2.store(true, Ordering::SeqCst);
                }
                result
            });
            assert!(c.server().accept_service(&listener, &r, cancel).is_err());
            let _ = client.join().unwrap();
        });
    }
    with_service(&c, &r, |s| {
        assert!(
            s.observe_recipient(foreign_binding(), PublicationAction::Begin)
                .is_ok()
        )
    });
}
#[test]
fn independently_configured_homes_can_reciprocate_without_an_extra_runtime_lane() {
    // Separate home runtimes are independent realms, not extra lanes for one
    // realm. No callback is executed here; executor/deadlock work remains open.
    let a = Certs::new();
    let b = Certs::new();
    let realm = |local: &str, remote: &str, own: &Certs, foreign: &Certs| {
        let r = Runtime::new(None);
        let now = wall_now();
        let registry = Registry::new(
            1,
            0,
            vec![local.into(), remote.into()],
            vec![CredentialBinding {
                certificate_sha256: sha256(&own.bob.0.to_der().unwrap()),
                principal: Principal {
                    account: format!("bob@{local}"),
                    device: 2,
                    identity: [2; 32],
                },
                valid_from: now - 60,
                valid_until: now + 3600,
                approved: true,
                account_enabled: true,
                device_enabled: true,
                generation: 1,
            }],
        )
        .unwrap();
        let policy = ServicePolicy::new(
            local.into(),
            vec![ServiceRegistration {
                home: remote.into(),
                client_id: format!("sender-{remote}"),
                dns_san: "provider.test".into(),
                certificate_sha256: sha256(&foreign.alice.0.to_der().unwrap()),
                enabled: true,
                valid_from: now - 60,
                valid_until: now + 3600,
            }],
        )
        .unwrap();
        r.control()
            .replace_delegation_policy(registry, policy)
            .unwrap();
        r
    };
    let ra = realm("home.test", "foreign.test", &a, &b);
    let rb = realm("foreign.test", "home.test", &b, &a);
    let sa = Server::new(&a.server.0, &a.server.1, &b.ca.0).unwrap();
    let sb = Server::new(&b.server.0, &b.server.1, &a.ca.0).unwrap();
    let la = TcpListener::bind("127.0.0.1:0").unwrap();
    let lb = TcpListener::bind("127.0.0.1:0").unwrap();
    let aa = la.local_addr().unwrap();
    let ab = lb.local_addr().unwrap();
    thread::scope(|scope| {
        let to_a = scope.spawn(|| {
            Client::connect(
                aa,
                "provider.test",
                sha256(&a.server.0.to_der().unwrap()),
                &a.ca.0,
                Some((&b.alice.0, Some(&b.alice.1))),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap()
        });
        let to_b = scope.spawn(|| {
            Client::connect(
                ab,
                "provider.test",
                sha256(&b.server.0.to_der().unwrap()),
                &b.ca.0,
                Some((&a.alice.0, Some(&a.alice.1))),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap()
        });
        let mut inbound_a = sa
            .accept_service(&la, &ra, Arc::new(AtomicBool::new(false)))
            .unwrap();
        let mut inbound_b = sb
            .accept_service(&lb, &rb, Arc::new(AtomicBool::new(false)))
            .unwrap();
        let _to_a = to_a.join().unwrap();
        let _to_b = to_b.join().unwrap();
        let ba = foreign_binding();
        let mut bb = ba.clone();
        bb.sender.account = "sender@home.test".into();
        bb.recipient.account = "bob@foreign.test".into();
        assert!(
            inbound_a
                .observe_recipient(ba, PublicationAction::Begin)
                .is_ok()
        );
        assert!(
            inbound_b
                .observe_recipient(bb, PublicationAction::Begin)
                .is_ok()
        );
        assert!(matches!(
            sa.accept_service(&la, &ra, Arc::new(AtomicBool::new(false))),
            Err(Stage::Denied)
        ));
        assert!(matches!(
            sb.accept_service(&lb, &rb, Arc::new(AtomicBool::new(false))),
            Err(Stage::Denied)
        ));
        drop(inbound_a);
        drop(inbound_b);
    });
    thread::scope(|scope| {
        let client = scope.spawn(|| {
            Client::connect(
                aa,
                "provider.test",
                sha256(&a.server.0.to_der().unwrap()),
                &a.ca.0,
                Some((&b.alice.0, Some(&b.alice.1))),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap()
        });
        let mut s = sa
            .accept_service(&la, &ra, Arc::new(AtomicBool::new(false)))
            .unwrap();
        let _client = client.join().unwrap();
        assert!(
            s.observe_recipient(foreign_binding(), PublicationAction::Begin)
                .is_ok()
        );
    });
}
