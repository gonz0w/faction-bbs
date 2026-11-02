//! Real sender Session publication followed by recipient retrieval. No setup Access.
use super::*;
use crate::retrieval::{RetrievalAction, RetrievalPolicy};
use crate::services::composed_tests::{Certificate, Pair};
use crate::tls::{ClientCredentials, HttpEndpoint, test_certificate as cert};
use std::{fs, net::TcpListener, sync::atomic::AtomicUsize, thread};
// Optional private raw captures for the execution harness, never product state.
fn retain_synthetic_fixture(name: &str, bytes: &[u8]) {
    use std::{
        io::Write,
        os::unix::fs::{OpenOptionsExt, PermissionsExt},
    };
    let Some(destination) = std::env::var_os("FACTION318_CAPTURE_DIR") else {
        return;
    };
    let destination = std::path::Path::new(&destination);
    assert_eq!(
        fs::metadata(destination).unwrap().permissions().mode() & 0o777,
        0o700
    );
    fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(destination.join(name))
        .unwrap()
        .write_all(bytes)
        .unwrap();
}
fn capture_carrier_fixture(h: &Homes, prefix: &str, destination: &std::path::Path) {
    use std::{
        io::Write,
        os::unix::fs::{OpenOptionsExt, PermissionsExt},
    };
    assert_eq!(
        fs::metadata(destination).unwrap().permissions().mode() & 0o7777,
        0o700
    );
    let save = |name: &str, bytes: &[u8]| {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(destination.join(format!("{prefix}-{name}")))
            .unwrap()
            .write_all(bytes)
            .unwrap();
    };
    for (i, home) in h.pair.homes.iter().enumerate() {
        for (name, certificate) in [
            ("ca", &home.ca),
            ("server", &home.server),
            ("device", &home.device),
            ("relay", &home.relay),
            ("outgoing", &home.outgoing),
            ("observer", &h.observers[i]),
        ] {
            save(
                &format!("home{i}-{name}.pem"),
                &certificate.0.to_pem().unwrap(),
            );
            save(
                &format!("home{i}-{name}.key"),
                &certificate.1.private_key_to_pem_pkcs8().unwrap(),
            );
        }
        let snapshot = h.pair.root.join(format!("store{i}/private-blobs.snapshot"));
        if snapshot.is_file() {
            save(
                &format!("home{i}-store.snapshot"),
                &fs::read(snapshot).unwrap(),
            );
        }
    }
    for (label, certificate) in h.other_credentials.lock().unwrap().iter() {
        save(
            &format!("extra-{label}.pem"),
            &certificate.0.to_pem().unwrap(),
        );
        save(
            &format!("extra-{label}.key"),
            &certificate.1.private_key_to_pem_pkcs8().unwrap(),
        );
    }
}
fn capture_carrier_optional(h: &Homes, label: &str) {
    static NEXT_CAPTURE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    if let Some(destination) = std::env::var_os("FACTION321_CAPTURE_DIR") {
        capture_carrier_fixture(
            h,
            &format!(
                "{label}-{}-{}",
                std::process::id(),
                NEXT_CAPTURE.fetch_add(1, Ordering::SeqCst)
            ),
            std::path::Path::new(&destination),
        );
    }
}
struct Homes {
    pair: Pair,
    observers: [Certificate; 2],
    callbacks: [AtomicUsize; 2],
    other_credentials: Mutex<Vec<(String, Certificate)>>,
}
impl Homes {
    fn new(durable: bool) -> Self {
        let pair = Pair::unconfigured(durable);
        let t = tls::wall_now() as i64;
        let observers = [
            cert(
                Some((&pair.homes[1].ca.0, &pair.homes[1].ca.1)),
                "a-observer",
                801,
                t - 60,
                t + 3600,
            ),
            cert(
                Some((&pair.homes[0].ca.0, &pair.homes[0].ca.1)),
                "b-observer",
                802,
                t - 60,
                t + 3600,
            ),
        ];
        let h = Self {
            pair,
            observers,
            callbacks: [AtomicUsize::new(0), AtomicUsize::new(0)],
            other_credentials: Mutex::new(Vec::new()),
        };
        for i in 0..2 {
            h.install(i, 2, 0, 2, 0, |_| ());
        }
        capture_carrier_optional(&h, "initialized");
        h
    }
    fn policy(
        &self,
        i: usize,
        g: u64,
        reset: u64,
        change: impl FnOnce(&mut Vec<(ServiceRegistration, ServiceUse)>),
    ) -> ServicePolicy {
        let local = &self.pair.homes[i];
        let foreign = &self.pair.homes[1 - i];
        let old = local.policy(foreign, g, reset);
        let mut entries: Vec<_> = old.registrations.into_iter().zip(old.uses).collect();
        for (owner, purpose) in [
            (i, ServiceUse::OutboundObservation),
            (1 - i, ServiceUse::Observation),
        ] {
            entries.push((
                ServiceRegistration {
                    home: self.pair.homes[owner].name.into(),
                    client_id: "observer".into(),
                    dns_san: "provider.test".into(),
                    certificate_sha256: sha256(&self.observers[owner].0.to_der().unwrap()),
                    enabled: true,
                    valid_from: tls::wall_now() - 60,
                    valid_until: tls::wall_now() + 3600,
                },
                purpose,
            ));
        }
        change(&mut entries);
        let issuer = old.issuer.unwrap();
        ServicePolicy::new_composed_publication(
            local.name.into(),
            issuer.issuer,
            entries,
            issuer.routes,
            old.resource.unwrap(),
            vec![PublicationAssociation {
                recipient_home: foreign.name.into(),
                outbound_client_id: "observer".into(),
                endpoint_id: "endpoint".into(),
                address: foreign.callback.local_addr().unwrap(),
                http_host: foreign.name.into(),
                ca_sha256: sha256(&foreign.ca.0.to_der().unwrap()),
                remote_generation: g,
                remote_reset: reset,
            }],
        )
        .unwrap()
    }
    fn install(
        &self,
        i: usize,
        g: u64,
        reset: u64,
        remote_g: u64,
        remote_reset: u64,
        change: impl FnOnce(&mut Vec<(ServiceRegistration, ServiceUse)>),
    ) {
        let local = &self.pair.homes[i];
        let foreign = &self.pair.homes[1 - i];
        local
            .runtime()
            .control()
            .replace_delegation_policy(
                local.registry(g, reset, foreign),
                self.policy(i, remote_g, remote_reset, change),
            )
            .unwrap();
        local
            .runtime()
            .control()
            .replace_retrieval_policy(
                RetrievalPolicy::new(
                    local.name.into(),
                    vec![(foreign.name.into(), format!("{}/attachments", foreign.name))],
                )
                .unwrap(),
            )
            .unwrap();
    }
    fn endpoint(&self, i: usize) -> HttpEndpoint {
        let foreign = &self.pair.homes[1 - i];
        HttpEndpoint::new(
            foreign.callback.local_addr().unwrap(),
            foreign.name.into(),
            "provider.test".into(),
            sha256(&foreign.server.0.to_der().unwrap()),
            ClientCredentials::from_pem(
                &foreign.ca.0.to_pem().unwrap(),
                &self.observers[i].0.to_pem().unwrap(),
                &self.observers[i].1.private_key_to_pem_pkcs8().unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn prepare(
        &self,
        i: usize,
        b: &Binding,
        request: Request,
    ) -> Result<PendingPublication, Error> {
        let local = &self.pair.homes[i];
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::scope(|scope| {
            let peer = scope.spawn(|| local.client(address, &local.device));
            let session = local
                .server(&local.ca.0)
                .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            let witness = {
                let a = local.runtime().state.authority.lock().unwrap();
                Arc::downgrade(&a.slots[0].connection)
            };
            let pending = session.prepare_publication(b.clone(), request);
            assert!(
                witness.upgrade().is_none(),
                "old Connection/Handle strong owners survived prepare"
            );
            assert!(!local.runtime().state.active.load(Ordering::SeqCst));
            let mut peer = peer.join().unwrap();
            assert!(
                peer.request(&Request::Status { id: b.id }).is_err(),
                "incoming TLS socket survived prepare"
            );
            pending
        })
    }
    fn serve(&self, i: usize) -> Result<(), Error> {
        let home = &self.pair.homes[i];
        let session = home
            .server(&home.ca.0)
            .accept_service(
                &home.callback,
                home.runtime(),
                Arc::new(AtomicBool::new(false)),
            )
            .map_err(Error::Transport)?;
        self.callbacks[i].fetch_add(1, Ordering::SeqCst);
        session.serve_publication_http_once(
            &PublicationWireProfile::new(home.name.into(), vec!["observer".into()]).unwrap(),
        )
    }
    fn complete(
        &self,
        i: usize,
        pending: PendingPublication,
        store: &Mutex<Store>,
    ) -> Result<Response, Error> {
        thread::scope(|scope| {
            let remote = scope.spawn(|| self.serve(1 - i));
            let result = pending.execute(&self.endpoint(i), store);
            let _ = remote.join().unwrap();
            result
        })
    }
    fn issuer_progress_while_callback_is_held(&self, i: usize) {
        use faction_delegation_codec as codec;
        let local = &self.pair.homes[i];
        let foreign = &self.pair.homes[1 - i];
        let binding = self.pair.binding(1 - i);
        let record = crate::retrieval::DeliveryRecord::new(local.record(&binding).unwrap());
        // Actual independent recipient device -> registered local relay mTLS.
        let token = local.issue(record.record());
        let endpoint = foreign.endpoint(local);
        thread::scope(|scope| {
            let caller = scope.spawn(|| {
                foreign.runtime().control().query_introspection(
                    &endpoint,
                    &codec::UnverifiedIntrospectionRequest {
                        client_id: "inspector".into(),
                        token,
                    },
                    Instant::now() + tls::CYCLE_LIMIT,
                    Arc::new(AtomicBool::new(false)),
                )
            });
            local
                .server(&local.ca.0)
                .accept_service(
                    &local.callback,
                    local.runtime(),
                    Arc::new(AtomicBool::new(false)),
                )
                .unwrap()
                .serve_http_once()
                .unwrap();
            let codec::UnverifiedIntrospection::Retrieval { claims, .. } =
                caller.join().unwrap().unwrap()
            else {
                panic!("same-Runtime current introspection did not progress")
            };
            assert_eq!(claims.binding.recipient.account, local.principal.account);
            assert_eq!(claims.binding.sender.account, foreign.principal.account);
            assert_eq!(claims.generation, 2);
            assert_eq!(claims.reset, 0);
        });
        drop(record);
    }
    fn empty(&self, i: usize) -> Mutex<Store> {
        Mutex::new(Store::open(&self.pair.root.join(format!("store{i}"))).unwrap())
    }
    fn bytes(&self, i: usize) -> Vec<u8> {
        fs::read(
            self.pair
                .root
                .join(format!("store{i}/private-blobs.snapshot")),
        )
        .unwrap()
    }
    fn publish(&self, i: usize, b: &Binding, store: &Mutex<Store>) {
        for request in [
            Request::Begin(b.clone()),
            Request::PutChunk {
                id: b.id,
                offset: 0,
                bytes: vec![9; 32],
            },
            Request::Commit { id: b.id },
        ] {
            let pending = self.prepare(i, b, request).unwrap();
            assert!(self.complete(i, pending, store).is_ok());
        }
    }
}
#[test]
fn actual_sender_publication_reopen_then_recipient_get_eof_status_works_both_directions() {
    let h = Homes::new(false);
    for i in 0..2 {
        let b = h.pair.binding(i);
        let store = h.empty(i);
        h.publish(i, &b, &store);
        retain_synthetic_fixture(&format!("sender-{i}-committed.snapshot"), &h.bytes(i));
        assert_eq!(h.callbacks[1 - i].load(Ordering::SeqCst), 3);
        drop(store);
        let store = h.empty(i);
        let local = &h.pair.homes[i];
        let foreign = &h.pair.homes[1 - i];
        for action in [
            RetrievalAction::GetChunk { offset: 0 },
            RetrievalAction::GetChunk { offset: 32 },
            RetrievalAction::Status,
        ] {
            let record =
                foreign.device_call(|mut s| s.retrieval_intent(b.clone(), action).unwrap());
            let token = foreign.issue(&record);
            let request = match action {
                RetrievalAction::GetChunk { offset } => Request::GetChunk { id: b.id, offset },
                RetrievalAction::Status => Request::Status { id: b.id },
            };
            let pending = local.service_call(&foreign.relay, &foreign.ca.0, |s| {
                s.prepare_retrieval(b.clone(), request, token).unwrap()
            });
            let response = local.complete(foreign, pending, &store).unwrap();
            match (action, response) {
                (
                    RetrievalAction::GetChunk { offset },
                    Response::Chunk {
                        offset: got,
                        bytes,
                        total,
                    },
                ) => {
                    assert_eq!(got, offset);
                    assert_eq!(bytes, if offset == 0 { vec![9; 32] } else { vec![] });
                    assert_eq!(total, 32);
                }
                (
                    RetrievalAction::Status,
                    Response::Progress {
                        binding, committed, ..
                    },
                ) => {
                    assert_eq!(binding, b);
                    assert!(committed);
                }
                _ => panic!("wrong recipient Store response"),
            }
        }
        local.device_call(|mut session| {
            let mut wrong = b.clone();
            wrong.id = sha256(&[8; 32]);
            assert!(
                session
                    .dispatch(
                        &mut store.lock().unwrap(),
                        &b.sender.account,
                        Request::Begin(wrong)
                    )
                    .is_err()
            );
        });
        let pending = h.prepare(i, &b, Request::Status { id: b.id }).unwrap();
        assert!(matches!(
            h.complete(i, pending, &store),
            Ok(Response::Progress {
                committed: true,
                ..
            })
        ));
        let pending = h.prepare(i, &b, Request::Delete { id: b.id }).unwrap();
        assert_eq!(h.complete(i, pending, &store).unwrap(), Response::Deleted);
    }
}
#[test]
fn lost_publication_replies_allow_exact_fresh_authority_retry_and_changed_bytes_never_commit() {
    let h = Homes::new(false);
    let b = h.pair.binding(0);
    let store = h.empty(0);
    for request in [
        Request::Begin(b.clone()),
        Request::PutChunk {
            id: b.id,
            offset: 0,
            bytes: vec![9; 32],
        },
        Request::Commit { id: b.id },
    ] {
        for _ in 0..2 {
            let pending = h.prepare(0, &b, request.clone()).unwrap();
            assert!(h.complete(0, pending, &store).is_ok());
        }
    }
    assert_eq!(h.callbacks[1].load(Ordering::SeqCst), 6);
    let before = h.bytes(0);
    let pending = h
        .prepare(
            0,
            &b,
            Request::PutChunk {
                id: b.id,
                offset: 0,
                bytes: vec![8; 32],
            },
        )
        .unwrap();
    assert!(h.complete(0, pending, &store).is_err());
    assert_eq!(h.bytes(0), before);
}
#[test]
fn expired_sender_delete_reclaims_with_fresh_recipient_approval_and_other_actions_still_refuse() {
    let h = Homes::new(false);
    let mut b = h.pair.binding(0);
    b.expires = tls::wall_now() + 3;
    let store = h.empty(0);
    h.publish(0, &b, &store);
    while tls::wall_now() < b.expires {
        thread::sleep(Duration::from_millis(10));
    }
    for request in [
        Request::Begin(b.clone()),
        Request::Status { id: b.id },
        Request::Commit { id: b.id },
    ] {
        assert!(h.prepare(0, &b, request).is_err());
    }
    let pending = h.prepare(0, &b, Request::Delete { id: b.id }).unwrap();
    assert_eq!(h.complete(0, pending, &store).unwrap(), Response::Deleted);
}

#[test]
fn publication_factory_refuses_wrong_sender_binding_selectors_and_preserves_original_attempts() {
    let h = Homes::new(false);
    let b = h.pair.binding(0);
    let store = h.empty(0);
    let before = h.bytes(0);
    for request in [
        Request::GetChunk {
            id: b.id,
            offset: 0,
        },
        Request::Status { id: [8; 32] },
        Request::PutChunk {
            id: b.id,
            offset: 0,
            bytes: vec![],
        },
        Request::PutChunk {
            id: b.id,
            offset: 0,
            bytes: vec![9; CHUNK + 1],
        },
        Request::PutChunk {
            id: b.id,
            offset: 33,
            bytes: vec![9],
        },
        Request::Begin({
            let mut changed = b.clone();
            changed.size += 1;
            changed
        }),
    ] {
        assert!(h.prepare(0, &b, request).is_err());
        assert_eq!(h.bytes(0), before);
    }
    for variant in 0..3 {
        let mut changed = b.clone();
        match variant {
            0 => changed.sender.device += 1,
            1 => changed.sender.identity = [7; 32],
            _ => changed.sender.account = "bob@b.test".into(),
        };
        assert!(
            h.prepare(0, &changed, Request::Begin(changed.clone()))
                .is_err()
        );
    }
    let local = &h.pair.homes[0];
    for attempted in [7, 8] {
        local.device_call(|mut session| {
            for _ in 0..attempted {
                assert!(
                    session
                        .dispatch_bytes(&mut store.lock().unwrap(), &b.sender.account, b"malformed")
                        .is_err()
                );
            }
            let pending = session.prepare_publication(b.clone(), Request::Begin(b.clone()));
            if attempted == 7 {
                assert_eq!(pending.unwrap().context.attempted, 8);
            } else {
                assert!(matches!(pending, Err(Error::Transport(tls::Stage::Limit))));
            }
        });
    }
    assert_eq!(h.callbacks[1].load(Ordering::SeqCst), 0);
    assert_eq!(h.bytes(0), before);
}
#[test]
fn each_publication_endpoint_identity_and_wrong_outgoing_purpose_refuse_before_connect() {
    for variant in 0..6 {
        let h = Homes::new(false);
        let b = h.pair.binding(0);
        let store = h.empty(0);
        let before = h.bytes(0);
        let pending = h.prepare(0, &b, Request::Begin(b.clone())).unwrap();
        let foreign = &h.pair.homes[1];
        let local = &h.pair.homes[0];
        let ca = if variant == 0 {
            &local.ca.0
        } else {
            &foreign.ca.0
        };
        let who = if variant == 1 {
            &local.outgoing
        } else {
            &h.observers[0]
        };
        let endpoint = HttpEndpoint::new(
            if variant == 2 {
                "127.0.0.1:9".parse().unwrap()
            } else {
                foreign.callback.local_addr().unwrap()
            },
            if variant == 3 {
                "other.test"
            } else {
                foreign.name
            }
            .into(),
            if variant == 4 {
                "other.test"
            } else {
                "provider.test"
            }
            .into(),
            if variant == 5 {
                [8; 32]
            } else {
                sha256(&foreign.server.0.to_der().unwrap())
            },
            ClientCredentials::from_pem(
                &ca.to_pem().unwrap(),
                &who.0.to_pem().unwrap(),
                &who.1.private_key_to_pem_pkcs8().unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        foreign.callback.set_nonblocking(true).unwrap();
        assert!(
            pending.execute(&endpoint, &store).is_err(),
            "endpoint variant {variant}"
        );
        assert!(
            matches!(foreign.callback.accept(),Err(e) if e.kind()==std::io::ErrorKind::WouldBlock)
        );
        assert_eq!(h.bytes(0), before);
    }
}
#[test]
fn immutable_request_and_actual_store_binding_refuse_changed_bytes_offset_action_and_identity() {
    let h = Homes::new(false);
    let b = h.pair.binding(0);
    let store = h.empty(0);
    h.publish(0, &b, &store);
    let before = h.bytes(0);
    for request in [
        Request::PutChunk {
            id: b.id,
            offset: 0,
            bytes: vec![8; 32],
        },
        Request::PutChunk {
            id: b.id,
            offset: 1,
            bytes: vec![9; 31],
        },
        Request::Commit { id: b.id },
        Request::Delete { id: b.id },
    ] {
        let mut pending = h
            .prepare(
                0,
                &b,
                Request::PutChunk {
                    id: b.id,
                    offset: 0,
                    bytes: vec![9; 32],
                },
            )
            .unwrap();
        pending.request = request;
        assert!(h.complete(0, pending, &store).is_err());
        assert_eq!(h.bytes(0), before);
    }
    let mut changed = b.clone();
    changed.created -= 1;
    let pending = h
        .prepare(0, &changed, Request::Status { id: b.id })
        .unwrap();
    assert!(h.complete(0, pending, &store).is_err());
    assert_eq!(h.bytes(0), before);
}
#[test]
fn mixed_publication_retrieval_and_issuer_records_share_one_physical_and_eight_aggregate_slots() {
    let h = Homes::new(false);
    let a = &h.pair.homes[0];
    let b = &h.pair.homes[1];
    let outgoing = h.pair.binding(0);
    let incoming = h.pair.binding(1);
    let pending = h
        .prepare(0, &outgoing, Request::Begin(outgoing.clone()))
        .unwrap();
    assert!(matches!(
        a.prepare(
            b,
            &outgoing,
            codec::UnverifiedToken::from_bytes([42; 32]),
            0
        ),
        Err(Error::Store(BlobError::Busy))
    ));
    let records: Vec<_> = (0..7)
        .map(|i| {
            let record = a.record(&incoming).unwrap();
            if i % 2 == 0 {
                let _ = a.issue(&record);
            }
            record
        })
        .collect();
    assert_eq!(a.charged(), 8);
    assert!(matches!(
        a.record(&incoming),
        Err(Error::Store(BlobError::Busy))
    ));
    drop(pending);
    assert_eq!(a.charged(), 7);
    let eighth = a.record(&incoming).unwrap();
    assert!(matches!(
        h.prepare(0, &outgoing, Request::Begin(outgoing.clone())),
        Err(Error::Store(BlobError::Busy))
    ));
    a.runtime().control().reset(1).unwrap();
    h.install(0, 4, 1, 2, 0, |_| ());
    let retrieval = a
        .prepare(
            b,
            &outgoing,
            codec::UnverifiedToken::from_bytes([42; 32]),
            0,
        )
        .unwrap();
    assert!(matches!(
        h.prepare(0, &outgoing, Request::Begin(outgoing.clone())),
        Err(Error::Store(BlobError::Busy))
    ));
    drop(records);
    drop(eighth);
    assert_eq!(a.charged(), 1);
    drop(retrieval);
    assert_eq!(a.charged(), 0);
    let pending = h
        .prepare(0, &outgoing, Request::Begin(outgoing.clone()))
        .unwrap();
    assert_eq!(a.charged(), 1);
    drop(pending);
    assert_eq!(a.charged(), 0);
}
#[test]
fn actual_rename_failure_returns_publication_uncertain_then_owned_fixture_reopen_exact_retry() {
    use std::os::unix::fs::DirBuilderExt;
    let h = Homes::new(false);
    let b = h.pair.binding(0);
    let store = h.empty(0);
    let snapshot = h.pair.root.join("store0/private-blobs.snapshot");
    let saved = h.pair.root.join("OWNED-before-rename-fault.snapshot");
    let original = fs::read(&snapshot).unwrap();
    retain_synthetic_fixture("store-before-rename-fault.snapshot", &original);
    fs::rename(&snapshot, &saved).unwrap();
    fs::DirBuilder::new().mode(0o700).create(&snapshot).unwrap();
    let pending = h.prepare(0, &b, Request::Begin(b.clone())).unwrap();
    assert!(matches!(
        h.complete(0, pending, &store),
        Err(Error::Store(BlobError::PublicationUncertain))
    ));
    retain_synthetic_fixture(
        "store-failed-rename.pending",
        &fs::read(h.pair.root.join("store0/private-blobs.pending")).unwrap(),
    );
    let pending = h.prepare(0, &b, Request::Begin(b.clone())).unwrap();
    assert!(matches!(
        h.complete(0, pending, &store),
        Err(Error::Store(BlobError::Poisoned))
    ));
    assert_eq!(fs::read(&saved).unwrap(), original);
    drop(store);
    // Deliberate known pre-rename fixture sabotage only; not operator uncertain recovery.
    fs::remove_dir(&snapshot).unwrap();
    fs::rename(&saved, &snapshot).unwrap();
    let store = h.empty(0);
    h.publish(0, &b, &store);
    assert_eq!(h.callbacks[1].load(Ordering::SeqCst), 5);
}
#[test]
fn publication_runtime_drop_and_actual_remote_outage_deny_without_store_or_fallback() {
    for runtime_drop in [false, true] {
        let mut h = Homes::new(false);
        let b = h.pair.binding(0);
        let store = h.empty(0);
        let before = h.bytes(0);
        let pending = h.prepare(0, &b, Request::Begin(b.clone())).unwrap();
        let endpoint = h.endpoint(0);
        if runtime_drop {
            drop(h.pair.homes[0].runtime.take());
            h.pair.homes[1].callback.set_nonblocking(true).unwrap();
        } else {
            h.pair.homes[1].callback = TcpListener::bind("127.0.0.1:0").unwrap();
        }
        assert!(pending.execute(&endpoint, &store).is_err());
        assert_eq!(h.bytes(0), before);
        if runtime_drop {
            assert!(
                matches!(h.pair.homes[1].callback.accept(),Err(e) if e.kind()==std::io::ErrorKind::WouldBlock)
            );
        }
    }
}
#[test]
fn complete_higher_generation_reset_proposals_restore_publication_only_after_both_homes_publish() {
    let h = Homes::new(false);
    let bindings = [h.pair.binding(0), h.pair.binding(1)];
    let stores = [h.empty(0), h.empty(1)];
    h.pair.homes[0].runtime().control().reset(1).unwrap();
    h.install(0, 4, 1, 4, 1, |_| ());
    let before = h.bytes(0);
    let pending = h
        .prepare(0, &bindings[0], Request::Begin(bindings[0].clone()))
        .unwrap();
    assert!(h.complete(0, pending, &stores[0]).is_err());
    assert_eq!(h.bytes(0), before);
    h.pair.homes[1].runtime().control().reset(1).unwrap();
    h.install(1, 4, 1, 4, 1, |_| ());
    for i in 0..2 {
        let pending = h
            .prepare(i, &bindings[i], Request::Begin(bindings[i].clone()))
            .unwrap();
        assert!(h.complete(i, pending, &stores[i]).is_ok());
    }
}
fn configure_expiry(h: &Homes, index: usize) -> u64 {
    let until = tls::wall_now() + 3;
    for i in 0..2 {
        let local = &h.pair.homes[i];
        let foreign = &h.pair.homes[1 - i];
        let mut entries = local.registry(4, 0, foreign).bindings().to_vec();
        if i == index {
            entries[0].valid_until = until;
        }
        local
            .runtime()
            .control()
            .replace_delegation_policy(
                Registry::new(4, 0, vec![local.name.into(), foreign.name.into()], entries).unwrap(),
                h.policy(i, 4, 0, |_| ()),
            )
            .unwrap();
    }
    until
}
#[test]
fn held_publication_callback_rechecks_current_authority_and_retains_physical_until_finish() {
    for case in 0..10 {
        let h = Homes::new(false);
        let until = if case >= 8 {
            configure_expiry(&h, if case == 8 { 0 } else { 1 })
        } else {
            0
        };
        let b = h.pair.binding(0);
        let store = h.empty(0);
        let pending = h.prepare(0, &b, Request::Begin(b.clone())).unwrap();
        let context = pending.context.clone();
        let mut endpoint = h.endpoint(0);
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        endpoint.hold_request(entered_tx, release_rx);
        thread::scope(|scope| {
            let remote = scope.spawn(|| h.serve(1));
            let operation = scope.spawn(|| pending.execute(&endpoint, &store));
            let deadline = entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            h.pair.homes[0].ordinary_progress(&store, &b);
            let before = h.bytes(0);
            assert!(!operation.is_finished());
            match case {
                0 | 2 => {
                    let index = if case == 0 { 0 } else { 1 };
                    let local = &h.pair.homes[index];
                    let foreign = &h.pair.homes[1 - index];
                    let mut entries = local.registry(3, 0, foreign).bindings().to_vec();
                    entries[0].approved = false;
                    local
                        .runtime()
                        .control()
                        .replace_delegation_policy(
                            Registry::new(
                                3,
                                0,
                                vec![local.name.into(), foreign.name.into()],
                                entries,
                            )
                            .unwrap(),
                            h.policy(index, 2, 0, |_| ()),
                        )
                        .unwrap();
                }
                1 | 3 => {
                    let index = if case == 1 { 0 } else { 1 };
                    h.pair.homes[index].runtime().control().reset(1).unwrap();
                    h.install(index, 4, 1, 2, 0, |_| ());
                }
                4 => context.cancel.store(true, Ordering::SeqCst),
                5 => h.pair.homes[0].runtime().control().stop().unwrap(),
                6 => {
                    context.clock.lock().unwrap().highwater =
                        SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
                            + Duration::from_secs(60)
                }
                7 => {
                    while Instant::now() < deadline {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
                _ => {
                    while tls::wall_now() < until {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }
            assert_eq!(h.pair.homes[0].charged(), 1, "case {case}");
            assert!(!operation.is_finished(), "case {case}");
            if case == 1 {
                assert!(matches!(
                    h.prepare(0, &b, Request::Begin(b.clone())),
                    Err(Error::Store(BlobError::Busy))
                ));
            }
            release_tx.send(()).unwrap();
            assert!(operation.join().unwrap().is_err(), "case {case}");
            let _ = remote.join().unwrap();
            assert_eq!(h.bytes(0), before, "case {case}");
            assert_eq!(h.pair.homes[0].charged(), 0, "case {case}");
            if case != 5 {
                let home = &h.pair.homes[0];
                let (g, reset, policy) = {
                    let a = home.runtime().state.authority.lock().unwrap();
                    (
                        a.generation,
                        a.reset_epoch,
                        a.service_policy.clone().unwrap(),
                    )
                };
                home.runtime()
                    .control()
                    .replace_delegation_policy(
                        home.registry(g + 1, reset, &h.pair.homes[1]),
                        policy,
                    )
                    .unwrap();
                assert!(h.prepare(0, &b, Request::Begin(b.clone())).is_ok());
            }
        });
    }
}
#[test]
fn held_publication_allows_same_runtime_device_store_and_issuer_introspection_progress() {
    let h = Homes::new(false);
    let b = h.pair.binding(0);
    let incoming = h.pair.binding(1);
    let store = h.empty(0);
    let pending = h.prepare(0, &b, Request::Begin(b.clone())).unwrap();
    let mut endpoint = h.endpoint(0);
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    endpoint.hold_request(entered_tx, release_rx);
    thread::scope(|scope| {
        let remote = scope.spawn(|| h.serve(1));
        let operation = scope.spawn(|| pending.execute(&endpoint, &store));
        entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let a = &h.pair.homes[0];
        a.ordinary_progress(&store, &b);
        let record = a.record(&incoming).unwrap();
        let reference = a.runtime().control().provision_retrieval(&record).unwrap();
        let token = a.service_call(&a.relay, &a.ca.0, |mut session| {
            session.issue_retrieval(&reference, "relay").unwrap()
        });
        a.service_call(&h.pair.homes[1].outgoing, &a.ca.0, |mut session| {
            assert!(
                session
                    .introspect_retrieval(token.token(), "inspector")
                    .unwrap()
                    .is_some()
            )
        });
        assert_eq!(a.charged(), 2);
        release_tx.send(()).unwrap();
        assert!(operation.join().unwrap().is_ok());
        remote.join().unwrap().unwrap();
        assert_eq!(a.charged(), 1);
    });
}

#[cfg(feature = "durable-policy")]
#[test]
fn durable_publication_reopen_discards_old_sender_continuations_and_tokens() {
    let mut h = Homes::new(true);
    let b = h.pair.binding(0);
    let store = h.empty(0);
    h.publish(0, &b, &store);
    drop(store);
    let store = h.empty(0);
    let old_sender = h.prepare(0, &b, Request::Status { id: b.id }).unwrap();
    let token = h.pair.homes[1].token(&b);
    h.pair.reopen();
    let before = h.bytes(0);
    assert!(old_sender.execute(&h.endpoint(0), &store).is_err());
    assert_eq!(h.bytes(0), before);
    let old_recipient = h.pair.homes[0]
        .prepare(&h.pair.homes[1], &b, token, 0)
        .unwrap();
    assert!(
        h.pair.homes[0]
            .complete(&h.pair.homes[1], old_recipient, &store)
            .is_err()
    );
    let current = h.prepare(0, &b, Request::Status { id: b.id }).unwrap();
    assert!(matches!(
        h.complete(0, current, &store),
        Ok(Response::Progress {
            committed: true,
            ..
        })
    ));
    let token = h.pair.homes[1].token(&b);
    let current = h.pair.homes[0]
        .prepare(&h.pair.homes[1], &b, token, 0)
        .unwrap();
    assert!(
        matches!(h.pair.homes[0].complete(&h.pair.homes[1],current,&store),Ok(Response::Chunk{bytes,..}) if bytes==vec![9;32])
    );
}
#[test]
fn old_composed_policy_never_implicitly_grants_publication() {
    let p = Pair::new(false);
    let b = p.binding(0);
    p.homes[0].device_call(|session| {
        assert!(
            session
                .prepare_publication(b.clone(), Request::Begin(b.clone()))
                .is_err()
        )
    });
    assert_eq!(p.homes[0].charged(), 0);
}
#[test]
fn publication_only_remote_home_floors_survive_removal_reset_and_consume_the_home_union() {
    let h = Homes::new(false);
    let a = &h.pair.homes[0];
    let b = &h.pair.homes[1];
    let policy = |generation, reset| {
        let p = h.policy(0, 2, 0, |_| ());
        let mut entries: Vec<_> = p.registrations.into_iter().zip(p.uses).collect();
        entries.push((
            ServiceRegistration {
                home: "c.test".into(),
                client_id: "third-endpoint".into(),
                dns_san: "provider.test".into(),
                certificate_sha256: [33; 32],
                enabled: true,
                valid_from: 1,
                valid_until: u64::MAX,
            },
            ServiceUse::IssuerEndpoint,
        ));
        let mut publications = p.publication.unwrap();
        publications.push(PublicationAssociation {
            recipient_home: "c.test".into(),
            outbound_client_id: "observer".into(),
            endpoint_id: "third-endpoint".into(),
            address: "127.0.0.1:9".parse().unwrap(),
            http_host: "c.test".into(),
            ca_sha256: [34; 32],
            remote_generation: generation,
            remote_reset: reset,
        });
        let issuer = p.issuer.unwrap();
        ServicePolicy::new_composed_publication(
            a.name.into(),
            issuer.issuer,
            entries,
            issuer.routes,
            p.resource.unwrap(),
            publications,
        )
        .unwrap()
    };
    let registry = |g, reset| {
        let entries = a.registry(g, reset, b).bindings().to_vec();
        Registry::new(
            g,
            reset,
            vec![a.name.into(), b.name.into(), "c.test".into()],
            entries,
        )
        .unwrap()
    };
    a.runtime()
        .control()
        .replace_delegation_policy(registry(3, 0), policy(5, 7))
        .unwrap();
    h.install(0, 4, 0, 2, 0, |_| ());
    a.runtime().control().reset(1).unwrap();
    for (g, reset) in [(4, 8), (6, 6)] {
        assert!(
            a.runtime()
                .control()
                .replace_delegation_policy(registry(6, 1), policy(g, reset))
                .is_err()
        );
    }
    let mut homes = vec![a.name.into(), b.name.into()];
    homes.extend((0..30).map(|i| format!("home{i}.test")));
    assert!(
        a.runtime()
            .control()
            .replace_registry(Registry::new(6, 1, homes, vec![]).unwrap())
            .is_err()
    );
    assert_eq!(
        a.runtime()
            .state
            .authority
            .lock()
            .unwrap()
            .remote_history
            .len(),
        2
    );
    a.runtime()
        .control()
        .replace_delegation_policy(registry(6, 1), policy(6, 8))
        .unwrap();
}

fn wait_publication_fixture(deadline: Instant, variant: usize, phase: &str) {
    assert!(
        Instant::now() < deadline,
        "fake publication {phase} deadline, variant {variant}"
    );
    thread::sleep(
        deadline
            .saturating_duration_since(Instant::now())
            .min(Duration::from_millis(5)),
    );
}
fn fake_publication_response(
    h: &Homes,
    pending: PendingPublication,
    store: &Mutex<Store>,
    variant: usize,
) -> Result<Response, Error> {
    use openssl::ssl::{
        ErrorCode, HandshakeError, SslAcceptor, SslMethod, SslVerifyMode, SslVersion,
    };
    let stale = pending.context.clock.lock().unwrap().highwater.as_secs() - 1;
    let expected = query(&pending.context);
    let foreign = &h.pair.homes[1];
    let deadline = pending.context.deadline;
    foreign.callback.set_nonblocking(true).unwrap();
    thread::scope(|scope| {
        let server=scope.spawn(||{
            let (socket,_)=loop{
                assert!(Instant::now()<deadline,"fake accept deadline, variant {variant}");
                match foreign.callback.accept(){
                    Ok(value)=>break value,
                    Err(error) if error.kind()==std::io::ErrorKind::WouldBlock=>wait_publication_fixture(deadline,variant,"accept"),
                    Err(error) if error.kind()==std::io::ErrorKind::Interrupted=>(),
                    Err(error)=>panic!("fake accept variant {variant}: {error}"),
                }
            };
            socket.set_nonblocking(true).unwrap();
            let mut acceptor=SslAcceptor::mozilla_intermediate_v5(SslMethod::tls_server()).unwrap();
            acceptor.set_min_proto_version(Some(SslVersion::TLS1_3)).unwrap();
            acceptor.set_max_proto_version(Some(SslVersion::TLS1_3)).unwrap();
            acceptor.set_certificate(&foreign.server.0).unwrap();acceptor.set_private_key(&foreign.server.1).unwrap();
            acceptor.cert_store_mut().add_cert(foreign.ca.0.clone()).unwrap();
            acceptor.set_verify(SslVerifyMode::PEER|SslVerifyMode::FAIL_IF_NO_PEER_CERT);
            let mut handshake=acceptor.build().accept(socket);
            let mut stream=loop{
                assert!(Instant::now()<deadline,"fake handshake deadline, variant {variant}");
                match handshake{
                    Ok(stream)=>break stream,
                    Err(HandshakeError::WouldBlock(mid))=>{wait_publication_fixture(deadline,variant,"handshake");handshake=mid.handshake();},
                    Err(HandshakeError::Failure(mid)) if mid.error().io_error().is_some_and(|e|e.kind()==std::io::ErrorKind::Interrupted)=>{handshake=mid.handshake();},
                    Err(error)=>panic!("fake handshake variant {variant}: {error}"),
                }
            };
            let observed=stream.ssl().peer_certificate().unwrap();
            let observed_digest=sha256(&observed.to_der().unwrap());
            assert_eq!(observed_digest,sha256(&h.observers[0].0.to_der().unwrap()));
            let mut bytes=[0;codec::MAX_HEADER+codec::MAX_JSON+1];let mut used=0;
            let head_end=loop{
                assert!(Instant::now()<deadline,"fake request head deadline, variant {variant}");
                assert!(used<codec::MAX_HEADER);
                match stream.ssl_read(&mut bytes[used..codec::MAX_HEADER]){
                    Ok(0)=>panic!("fake request head EOF, variant {variant}"),
                    Ok(n)=>{used+=n;if let Some(at)=bytes[..used].windows(4).position(|w|w==b"\r\n\r\n"){break at+4;}},
                    Err(error) if matches!(error.code(),ErrorCode::WANT_READ|ErrorCode::WANT_WRITE)=>wait_publication_fixture(deadline,variant,"request head"),
                    Err(error) if error.io_error().is_some_and(|e|e.kind()==std::io::ErrorKind::Interrupted)=>(),
                    Err(error)=>panic!("fake request head variant {variant}: {error}"),
                }
            };
            let head=codec::decode_http_request_head(&bytes[..head_end]).unwrap();
            assert_eq!(head.endpoint,codec::UnverifiedEndpoint::PublicationCurrent);assert_eq!(head.host,foreign.name);
            assert!(head.content_length<=codec::MAX_JSON);let total=head_end+head.content_length;
            assert!(used<=total);
            while used<total{
                assert!(Instant::now()<deadline,"fake request body deadline, variant {variant}");
                match stream.ssl_read(&mut bytes[used..=total]){
                    Ok(0)=>panic!("fake request body EOF, variant {variant}"),
                    Ok(n)=>{used+=n;assert!(used<=total);},
                    Err(error) if matches!(error.code(),ErrorCode::WANT_READ|ErrorCode::WANT_WRITE)=>wait_publication_fixture(deadline,variant,"request body"),
                    Err(error) if error.io_error().is_some_and(|e|e.kind()==std::io::ErrorKind::Interrupted)=>(),
                    Err(error)=>panic!("fake request body variant {variant}: {error}"),
                }
            }
            assert!(Instant::now()<deadline);assert_eq!(stream.ssl().pending(),0);
            let received=codec::decode_publication_query_http(&bytes[..total]).unwrap();
            assert_eq!(received,expected);
            assert_eq!(received.action,codec::UnverifiedPublicationAction::PutChunk{offset:0,length:32,ciphertext_sha256:hex(&sha256(&[9;32]))});
            let mut result=codec::UnverifiedPublicationObservation::Current{
                query:Box::new(received),home:foreign.name.into(),service_home:h.pair.homes[0].name.into(),
                service_certificate_sha256:hex(&observed_digest),recipient_certificate_sha256:hex(&sha256(&foreign.device.0.to_der().unwrap())),
                generation:2,reset:0,checked_at:tls::wall_now(),
            };
            if let codec::UnverifiedPublicationObservation::Current{query,service_certificate_sha256,recipient_certificate_sha256,generation,reset,checked_at,..}=&mut result{
                match variant{
                    0=>(),1=>query.client_id="inspector".into(),2=>query.binding.sender.account="other@a.test".into(),
                    3=>query.binding.sender.device+=1,4=>query.binding.sender.identity="88".repeat(32),
                    5=>query.binding.recipient.account="other@b.test".into(),6=>query.binding.recipient.device+=1,
                    7=>query.binding.recipient.identity="88".repeat(32),8=>query.binding.id="88".repeat(32),
                    9=>query.binding.size+=1,10=>query.binding.created-=1,11=>query.binding.expires+=1,
                    12=>query.action=codec::UnverifiedPublicationAction::PutChunk{offset:1,length:31,ciphertext_sha256:hex(&sha256(&[9;32]))},
                    13=>query.action=codec::UnverifiedPublicationAction::PutChunk{offset:0,length:32,ciphertext_sha256:"88".repeat(32)},
                    14=>query.action=codec::UnverifiedPublicationAction::Begin,15=>*service_certificate_sha256="88".repeat(32),
                    16=>*recipient_certificate_sha256="00".repeat(32),17=>*generation=1,18=>*generation=3,
                    19=>*reset=1,20=>*checked_at=stale,21=>*checked_at+=60,_=>(),
                }
            }
            if variant==22{result=codec::UnverifiedPublicationObservation::Unavailable;}
            let mut body=codec::encode_publication_response_json(&result).unwrap();
            if variant==23{body=b"{}".to_vec();}
            else if variant==24{body=String::from_utf8(body).unwrap().replace("\"home\":\"b.test\"","\"home\":\"wrong.test\"").into_bytes();}
            else if variant==25{body=String::from_utf8(body).unwrap().replace("\"service_home\":\"a.test\"","\"service_home\":\"wrong.test\"").into_bytes();}
            let mut response=format!("HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",body.len()).into_bytes();response.extend(body);
            let mut sent=0;
            while sent<response.len(){
                assert!(Instant::now()<deadline,"fake response deadline, variant {variant}");
                match stream.ssl_write(&response[sent..]){
                    Ok(0)=>panic!("fake response zero write, variant {variant}"),Ok(n)=>sent+=n,
                    Err(error) if matches!(error.code(),ErrorCode::WANT_READ|ErrorCode::WANT_WRITE)=>wait_publication_fixture(deadline,variant,"response"),
                    Err(error) if error.io_error().is_some_and(|e|e.kind()==std::io::ErrorKind::Interrupted)=>(),
                    Err(error)=>panic!("fake response variant {variant}: {error}"),
                }
            }
            assert!(Instant::now()<deadline);
        });
        let result = pending.execute(&h.endpoint(0), store);
        server.join().unwrap();
        result
    })
}

#[test]
fn actual_authenticated_publication_responses_cannot_substitute_query_certificates_epochs_or_check_time()
 {
    for variant in 0..26 {
        let h = Homes::new(false);
        let b = h.pair.binding(0);
        let store = h.empty(0);
        let pending = h.prepare(0, &b, Request::Begin(b.clone())).unwrap();
        h.complete(0, pending, &store).unwrap();
        let before = h.bytes(0);
        let pending = h
            .prepare(
                0,
                &b,
                Request::PutChunk {
                    id: b.id,
                    offset: 0,
                    bytes: vec![9; 32],
                },
            )
            .unwrap();
        let result = fake_publication_response(&h, pending, &store, variant);
        if variant == 0 {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err(), "variant {variant}");
            assert_eq!(h.bytes(0), before, "variant {variant}");
        }
    }
}
#[test]
fn actual_incoming_observation_role_is_required_and_outgoing_only_certificate_never_mints_a_service_session()
 {
    let h = Homes::new(false);
    h.install(0, 3, 0, 3, 0, |_| ());
    h.install(1, 3, 0, 3, 0, |entries| {
        entries
            .iter_mut()
            .find(|(e, _)| e.home == "a.test" && e.client_id == "observer")
            .unwrap()
            .1 = ServiceUse::Introspection;
    });
    let b = h.pair.binding(0);
    let store = h.empty(0);
    let before = h.bytes(0);
    let pending = h.prepare(0, &b, Request::Begin(b.clone())).unwrap();
    assert!(h.complete(0, pending, &store).is_err());
    assert_eq!(h.bytes(0), before);
    let local = &h.pair.homes[0];
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::scope(|scope| {
        let peer = scope.spawn(|| {
            crate::tls::Client::connect(
                address,
                "provider.test",
                sha256(&local.server.0.to_der().unwrap()),
                &local.ca.0,
                Some((&h.observers[0].0, Some(&h.observers[0].1))),
                Arc::new(AtomicBool::new(false)),
            )
        });
        let session = local.server(&h.pair.homes[1].ca.0).accept_service(
            &listener,
            local.runtime(),
            Arc::new(AtomicBool::new(false)),
        );
        assert!(session.is_err());
        drop(peer.join().unwrap());
    });
}

// Mutex holds below are actual Authority contention. The test-only gates identify
// the factory, endpoint and final-dispatch boundaries, not kernel backpressure.
fn wait_expiry_or_deadline(case: usize, until: u64, deadline: Instant, cancel: &AtomicBool) {
    match case {
        0 => {
            while tls::wall_now() < until {
                thread::sleep(Duration::from_millis(5));
            }
        }
        2 => cancel.store(true, Ordering::SeqCst),
        3 => {
            while Instant::now() < deadline {
                thread::sleep(Duration::from_millis(5));
            }
        }
        _ => (),
    }
}
#[test]
fn actual_sender_factory_resamples_after_queued_authority_for_expiry_stop_cancel_and_original_deadline()
 {
    for case in 0..5 {
        let h = Homes::new(false);
        let until = if case == 0 {
            configure_expiry(&h, 0)
        } else {
            0
        };
        let b = h.pair.binding(0);
        let home = &h.pair.homes[0];
        let store = h.empty(0);
        let before = h.bytes(0);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        thread::scope(|scope| {
            let peer = scope.spawn(|| home.client(address, &home.device));
            let mut session = home
                .server(&home.ca.0)
                .accept(&listener, home.runtime(), cancel.clone())
                .unwrap();
            let _peer = peer.join().unwrap();
            let (entered_tx, entered_rx) = std::sync::mpsc::channel();
            session.capture_publication_before_authority(entered_tx);
            let mut guard = home.runtime().state.authority.lock().unwrap();
            let operation =
                scope.spawn(|| session.prepare_publication(b.clone(), Request::Begin(b.clone())));
            let original = entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            let stop = if case == 1 {
                let control = home.runtime().control();
                Some(scope.spawn(move || control.stop()))
            } else {
                None
            };
            if case == 1 {
                while !home.runtime().state.stopped.load(Ordering::SeqCst) {
                    thread::yield_now();
                }
            }
            if case == 4 {
                let mut entries = home
                    .registry(guard.generation + 1, guard.reset_epoch, &h.pair.homes[1])
                    .bindings()
                    .to_vec();
                entries[0].approved = false;
                let registry = Registry::new(
                    guard.generation + 1,
                    guard.reset_epoch,
                    vec!["a.test".into(), "b.test".into()],
                    entries,
                )
                .unwrap();
                guard
                    .replace_delegation_policy(registry, h.policy(0, 2, 0, |_| ()))
                    .unwrap();
            }
            wait_expiry_or_deadline(case, until, original, &cancel);
            let finished_while_held = operation.is_finished();
            drop(guard);
            let result = operation.join().unwrap();
            if let Some(stop) = stop {
                stop.join().unwrap().unwrap();
            }
            assert!(!finished_while_held);
            assert!(result.is_err(), "case {case}");
        });
        assert_eq!(home.charged(), 0);
        assert_eq!(h.bytes(0), before);
        assert_eq!(h.callbacks[1].load(Ordering::SeqCst), 0);
        drop(store);
    }
}
#[test]
fn exact_prepared_context_probe_resamples_after_queued_authority_for_expiry_stop_cancel_and_deadline()
 {
    for case in 0..5 {
        let h = Homes::new(false);
        let until = if case == 0 {
            configure_expiry(&h, 0)
        } else {
            0
        };
        let b = h.pair.binding(0);
        let mut pending = h.prepare(0, &b, Request::Begin(b.clone())).unwrap();
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        Arc::get_mut(&mut pending.context).unwrap().authority_gates[0] = Some(TestGate {
            entered: entered_tx,
            release: Mutex::new(release_rx),
        });
        let context = pending.context.clone();
        let home = &h.pair.homes[0];
        thread::scope(|scope| {
            let mut guard = home.runtime().state.authority.lock().unwrap();
            let operation = scope.spawn(|| context.current());
            entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            release_tx.send(()).unwrap();
            let stop = if case == 1 {
                let control = home.runtime().control();
                Some(scope.spawn(move || control.stop()))
            } else {
                None
            };
            if case == 1 {
                while !home.runtime().state.stopped.load(Ordering::SeqCst) {
                    thread::yield_now();
                }
            }
            if case == 4 {
                let mut entries = home
                    .registry(guard.generation + 1, guard.reset_epoch, &h.pair.homes[1])
                    .bindings()
                    .to_vec();
                entries[0].approved = false;
                let registry = Registry::new(
                    guard.generation + 1,
                    guard.reset_epoch,
                    vec!["a.test".into(), "b.test".into()],
                    entries,
                )
                .unwrap();
                guard
                    .replace_delegation_policy(registry, h.policy(0, 2, 0, |_| ()))
                    .unwrap();
            }
            wait_expiry_or_deadline(case, until, context.deadline, &context.cancel);
            let finished_while_held = operation.is_finished();
            drop(guard);
            let result = operation.join().unwrap();
            if let Some(stop) = stop {
                stop.join().unwrap().unwrap();
            }
            assert!(!finished_while_held);
            assert!(result.is_err(), "case {case}");
        });
        drop(pending);
        assert_eq!(home.charged(), 0);
    }
}
#[test]
fn endpoint_and_final_dispatch_resample_after_each_authority_wait_without_stale_sender_checks() {
    for stage in 0..2 {
        for case in 0..5 {
            let h = Homes::new(false);
            let until = if case == 0 {
                configure_expiry(&h, 0)
            } else {
                0
            };
            let b = h.pair.binding(0);
            let store = h.empty(0);
            let before = h.bytes(0);
            let mut pending = h.prepare(0, &b, Request::Begin(b.clone())).unwrap();
            let (entered_tx, entered_rx) = std::sync::mpsc::channel();
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            let gate = TestGate {
                entered: entered_tx,
                release: Mutex::new(release_rx),
            };
            let context = Arc::get_mut(&mut pending.context).unwrap();
            context.authority_gates[stage + 1] = Some(gate);
            let context = pending.context.clone();
            let home = &h.pair.homes[0];
            let endpoint = h.endpoint(0);
            thread::scope(|scope| {
                let remote = if stage == 1 {
                    Some(scope.spawn(|| h.serve(1)))
                } else {
                    None
                };
                let operation = scope.spawn(|| pending.execute(&endpoint, &store));
                entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                if let Some(remote) = remote {
                    remote.join().unwrap().unwrap();
                }
                if stage == 0 {
                    assert!(store.try_lock().is_ok());
                } else {
                    assert!(store.try_lock().is_err());
                }
                let mut guard = home.runtime().state.authority.lock().unwrap();
                release_tx.send(()).unwrap();
                let stop = if case == 1 {
                    let control = home.runtime().control();
                    Some(scope.spawn(move || control.stop()))
                } else {
                    None
                };
                if case == 1 {
                    while !home.runtime().state.stopped.load(Ordering::SeqCst) {
                        thread::yield_now();
                    }
                }
                if case == 4 {
                    let mut entries = home
                        .registry(guard.generation + 1, guard.reset_epoch, &h.pair.homes[1])
                        .bindings()
                        .to_vec();
                    entries[0].approved = false;
                    let registry = Registry::new(
                        guard.generation + 1,
                        guard.reset_epoch,
                        vec!["a.test".into(), "b.test".into()],
                        entries,
                    )
                    .unwrap();
                    guard
                        .replace_delegation_policy(registry, h.policy(0, 2, 0, |_| ()))
                        .unwrap();
                }
                wait_expiry_or_deadline(case, until, context.deadline, &context.cancel);
                let finished_while_held = operation.is_finished();
                drop(guard);
                let result = operation.join().unwrap();
                if let Some(stop) = stop {
                    stop.join().unwrap().unwrap();
                }
                assert!(!finished_while_held);
                assert!(result.is_err(), "stage {stage}, case {case}");
            });
            assert_eq!(h.bytes(0), before);
            assert_eq!(home.charged(), 0);
            if stage == 0 {
                h.pair.homes[1].callback.set_nonblocking(true).unwrap();
                assert!(
                    matches!(h.pair.homes[1].callback.accept(),Err(e) if e.kind()==std::io::ErrorKind::WouldBlock)
                );
            }
        }
    }
}

#[test]
fn actual_certificate_and_service_registration_expiry_during_held_publication_refuses() {
    for case in 0..6 {
        let mut h = Homes::new(false);
        let t = tls::wall_now() as i64;
        let until = (t + 3) as u64;
        match case {
            0 => {
                h.pair.homes[0].device = cert(
                    Some((&h.pair.homes[0].ca.0, &h.pair.homes[0].ca.1)),
                    "expiring-sender",
                    901,
                    t - 60,
                    t + 3,
                )
            }
            1 => {
                h.observers[0] = cert(
                    Some((&h.pair.homes[1].ca.0, &h.pair.homes[1].ca.1)),
                    "expiring-observer",
                    902,
                    t - 60,
                    t + 3,
                )
            }
            2 => {
                h.pair.homes[1].server = cert(
                    Some((&h.pair.homes[1].ca.0, &h.pair.homes[1].ca.1)),
                    "expiring-endpoint",
                    903,
                    t - 60,
                    t + 3,
                )
            }
            _ => (),
        }
        for i in 0..2 {
            h.install(i, 4, 0, 4, 0, |entries| {
                if (case == 3 && i == 0) || (case == 4 && i == 1) || (case == 5 && i == 0) {
                    let (home, id) = if case == 5 {
                        ("b.test", "endpoint")
                    } else {
                        ("a.test", "observer")
                    };
                    entries
                        .iter_mut()
                        .find(|(e, _)| e.home == home && e.client_id == id)
                        .unwrap()
                        .0
                        .valid_until = until;
                }
            });
        }
        let b = h.pair.binding(0);
        let store = h.empty(0);
        let before = h.bytes(0);
        let pending = h.prepare(0, &b, Request::Begin(b.clone())).unwrap();
        let mut endpoint = h.endpoint(0);
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        endpoint.hold_request(entered_tx, release_rx);
        thread::scope(|scope| {
            let remote = scope.spawn(|| h.serve(1));
            let operation = scope.spawn(|| pending.execute(&endpoint, &store));
            entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            while tls::wall_now() < until {
                thread::sleep(Duration::from_millis(5));
            }
            assert_eq!(h.pair.homes[0].charged(), 1);
            release_tx.send(()).unwrap();
            assert!(operation.join().unwrap().is_err(), "expiry case {case}");
            let _ = remote.join().unwrap();
        });
        assert_eq!(h.pair.homes[0].charged(), 0);
        assert_eq!(h.bytes(0), before, "expiry case {case}");
    }
}
#[test]
fn expired_delete_does_not_waive_current_sender_recipient_or_observation_service() {
    for case in 0..3 {
        let h = Homes::new(false);
        let mut b = h.pair.binding(0);
        b.expires = tls::wall_now() + 3;
        let store = h.empty(0);
        h.publish(0, &b, &store);
        while tls::wall_now() < b.expires {
            thread::sleep(Duration::from_millis(5));
        }
        let before = h.bytes(0);
        if case == 0 {
            let pending = h.prepare(0, &b, Request::Delete { id: b.id }).unwrap();
            let a = &h.pair.homes[0];
            let mut entries = a.registry(3, 0, &h.pair.homes[1]).bindings().to_vec();
            entries[0].approved = false;
            a.runtime()
                .control()
                .replace_delegation_policy(
                    Registry::new(3, 0, vec!["a.test".into(), "b.test".into()], entries).unwrap(),
                    h.policy(0, 2, 0, |_| ()),
                )
                .unwrap();
            assert!(pending.execute(&h.endpoint(0), &store).is_err());
        } else {
            h.install(0, 3, 0, 3, 0, |_| ());
            let home = &h.pair.homes[1];
            let mut entries = home.registry(3, 0, &h.pair.homes[0]).bindings().to_vec();
            if case == 1 {
                entries[0].approved = false;
            }
            home.runtime()
                .control()
                .replace_delegation_policy(
                    Registry::new(3, 0, vec!["a.test".into(), "b.test".into()], entries).unwrap(),
                    h.policy(1, 3, 0, |services| {
                        if case == 2 {
                            services
                                .iter_mut()
                                .find(|(e, _)| e.home == "a.test" && e.client_id == "observer")
                                .unwrap()
                                .0
                                .enabled = false;
                        }
                    }),
                )
                .unwrap();
            let pending = h.prepare(0, &b, Request::Delete { id: b.id }).unwrap();
            assert!(h.complete(0, pending, &store).is_err());
        }
        assert_eq!(h.bytes(0), before, "delete case {case}");
    }
}

#[test]
fn publication_precise_clock_refuses_same_second_fractional_rollback_and_never_revives() {
    for _ in 0..4 {
        let before = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let mut clock = Clock {
            highwater: Duration::new(before.as_secs(), 999_999_999),
            invalid: false,
        };
        let result = clock.sample();
        let after = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        if after.as_secs() != before.as_secs() {
            continue;
        }
        assert_eq!(result, Err(BlobError::Clock));
        clock.highwater = Duration::ZERO;
        assert_eq!(clock.sample(), Err(BlobError::Clock));
        return;
    }
    panic!("could not obtain a same-second clock observation window");
}
#[test]
fn versioned_publication_submit_close_busy_progress_callback_collect_both_directions() {
    use crate::delivery::{CarrierEndpoint, Collection};
    use std::sync::mpsc;
    let h = Homes::new(false);
    for i in 0..2 {
        let local = &h.pair.homes[i];
        let remote = &h.pair.homes[1 - i];
        let binding = h.pair.binding(i);
        let store = h.empty(i);
        for request in [
            Request::Begin(binding.clone()),
            Request::PutChunk {
                id: binding.id,
                offset: 0,
                bytes: vec![9; 32],
            },
            Request::Commit { id: binding.id },
            Request::Status { id: binding.id },
            Request::Delete { id: binding.id },
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let endpoint = CarrierEndpoint::new(
                listener.local_addr().unwrap(),
                "provider.test".into(),
                sha256(&local.server.0.to_der().unwrap()),
                ClientCredentials::from_pem(
                    &local.ca.0.to_pem().unwrap(),
                    &local.device.0.to_pem().unwrap(),
                    &local.device.1.private_key_to_pem_pkcs8().unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
            let before = local.charged();
            thread::scope(|scope| {
                let (commands, go) = mpsc::channel();
                let (result, observed) = mpsc::channel();
                let client_request = request.clone();
                let client_binding = binding.clone();
                let client = scope.spawn(move || {
                    let mut operation = endpoint.operation(Arc::new(AtomicBool::new(false)));
                    operation
                        .submit_publication(client_binding, client_request)
                        .unwrap();
                    for _ in 0..2 {
                        go.recv_timeout(Duration::from_secs(5)).unwrap();
                        result.send(operation.collect()).unwrap();
                    }
                });
                let session = local
                    .server(&local.ca.0)
                    .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                    .unwrap();
                let witness = {
                    let a = local.runtime().state.authority.lock().unwrap();
                    Arc::downgrade(&a.slots[0].connection)
                };
                let (mut delivery, work) = session.submit_publication().unwrap();
                assert!(
                    witness.upgrade().is_none(),
                    "original Connection/Handle survived wire submit"
                );
                assert!(!local.runtime().state.active.load(Ordering::SeqCst));
                assert_eq!(local.charged(), before + 1);
                let callback = h.endpoint(i);
                let store_ref = &store;
                let worker = scope.spawn(move || work.execute(&callback, store_ref));
                // Actual callback TLS is established and held before processing.
                let held = remote
                    .server(&remote.ca.0)
                    .accept_service(
                        &remote.callback,
                        remote.runtime(),
                        Arc::new(AtomicBool::new(false)),
                    )
                    .unwrap();
                commands.send(()).unwrap();
                let early = local
                    .server(&local.ca.0)
                    .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                    .unwrap();
                assert!(!early.collect_delivery(&mut delivery).unwrap());
                assert!(matches!(
                    observed
                        .recv_timeout(Duration::from_secs(5))
                        .unwrap()
                        .unwrap(),
                    Collection::Busy
                ));
                assert!(!local.runtime().state.active.load(Ordering::SeqCst));
                assert!(!delivery.is_finished());
                local.ordinary_progress(&store, &binding);
                h.issuer_progress_while_callback_is_held(i);
                held.serve_publication_http_once(
                    &PublicationWireProfile::new(remote.name.into(), vec!["observer".into()])
                        .unwrap(),
                )
                .unwrap();
                worker.join().unwrap().unwrap();
                assert!(delivery.is_finished());
                assert_eq!(
                    local.charged(),
                    before + 1,
                    "physical lease freed at callback completion"
                );
                commands.send(()).unwrap();
                let fresh = local
                    .server(&local.ca.0)
                    .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                    .unwrap();
                assert!(fresh.collect_delivery(&mut delivery).unwrap());
                let Collection::Complete(response) = observed
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap()
                    .unwrap()
                else {
                    panic!("missing actual operation result")
                };
                match (&request, response) {
                    (
                        Request::Begin(_),
                        Response::Progress {
                            offset: 0,
                            committed: false,
                            ..
                        },
                    ) => (),
                    (
                        Request::PutChunk { .. },
                        Response::Progress {
                            offset: 32,
                            committed: false,
                            ..
                        },
                    ) => (),
                    (
                        Request::Commit { .. } | Request::Status { .. },
                        Response::Progress {
                            binding: actual,
                            offset: 32,
                            committed: true,
                        },
                    ) => assert_eq!(actual, binding),
                    (Request::Delete { .. }, Response::Deleted) => (),
                    other => panic!("wrong independently expected Store result {other:?}"),
                }
                assert_eq!(
                    local.charged(),
                    before + 1,
                    "lease ended before delivery owner/socket completion"
                );
                client.join().unwrap();
                drop(delivery);
                assert_eq!(local.charged(), before);
                capture_carrier_optional(&h, "publication-completed");
            });
        }
    }
}
#[test]
fn source_bound_malformed_collects_spend_the_original_eight_attempts() {
    use crate::delivery::CarrierEndpoint;
    use std::sync::mpsc;
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let binding = h.pair.binding(0);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let endpoint = CarrierEndpoint::new(
        address,
        "provider.test".into(),
        sha256(&local.server.0.to_der().unwrap()),
        ClientCredentials::from_pem(
            &local.ca.0.to_pem().unwrap(),
            &local.device.0.to_pem().unwrap(),
            &local.device.1.private_key_to_pem_pkcs8().unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    thread::scope(|scope| {
        let (go, wait) = mpsc::channel();
        let client_binding = binding.clone();
        let client = scope.spawn(move || {
            let mut operation = endpoint.operation(Arc::new(AtomicBool::new(false)));
            operation
                .submit_publication(client_binding.clone(), Request::Begin(client_binding))
                .unwrap();
            wait.recv_timeout(Duration::from_secs(5)).unwrap();
            operation.collect()
        });
        let session = local
            .server(&local.ca.0)
            .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let (mut delivery, work) = session.submit_publication().unwrap();
        for _ in 0..7 {
            let malformed = scope.spawn(|| {
                let mut peer = local.client(address, &local.device);
                peer.request(&Request::Status { id: binding.id })
            });
            let fresh = local
                .server(&local.ca.0)
                .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            assert!(fresh.collect_delivery(&mut delivery).is_err());
            assert!(malformed.join().unwrap().is_err());
        }
        go.send(()).unwrap();
        let fresh = local
            .server(&local.ca.0)
            .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        assert!(
            matches!(
                fresh.collect_delivery(&mut delivery),
                Err(Error::Transport(tls::Stage::Limit))
            ),
            "malformed same-source collects renewed the original operation budget"
        );
        assert!(client.join().unwrap().is_err());
        drop(work);
        drop(delivery);
        assert_eq!(local.charged(), 0);
    });
}
#[test]
fn fresh_collect_decode_cannot_wait_past_the_captured_original_deadline() {
    use crate::delivery::Delivery;
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let binding = h.pair.binding(0);
    let pending = h
        .prepare(0, &binding, Request::Begin(binding.clone()))
        .unwrap();
    let mut source = pending.delivery_source();
    // Synthetic conservative narrowing, not a host clock change or timeout increase.
    source.deadline = Instant::now() + Duration::from_millis(100);
    let mut delivery = Delivery::new(source).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::scope(|scope| {
        let client = scope.spawn(|| {
            let _peer = local.client(address, &local.device);
            thread::sleep(Duration::from_millis(400));
        });
        let session = local
            .server(&local.ca.0)
            .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let start = Instant::now();
        let outcome = session.collect_delivery(&mut delivery);
        assert!(
            matches!(outcome, Err(Error::Transport(tls::Stage::Deadline))),
            "fresh collect refreshed its decode window: {outcome:?}"
        );
        assert!(start.elapsed() < Duration::from_millis(250));
        client.join().unwrap();
    });
    drop(pending);
    drop(delivery);
    assert_eq!(local.charged(), 0);
}

#[test]
fn carrier_private_fixture_capture_retains_exact_generated_inputs() {
    use std::os::unix::fs::PermissionsExt;
    let h = Homes::new(false);
    let retained = std::env::var_os("FACTION321_CAPTURE_DIR");
    let destination = retained
        .clone()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            use std::os::unix::fs::DirBuilderExt;
            let path = std::env::temp_dir().join(format!(
                "faction321-retention-OWNED-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
            fs::write(path.join("OWNED"), b"Owned private fixture capture control").unwrap();
            path
        });
    h.other_credentials.lock().unwrap().push((
        "retention-extra".into(),
        (
            h.pair.homes[0].device.0.clone(),
            h.pair.homes[0].device.1.clone(),
        ),
    ));
    capture_carrier_fixture(&h, "retention-control", &destination);
    assert_eq!(
        fs::read(destination.join("retention-control-extra-retention-extra.key")).unwrap(),
        h.pair.homes[0].device.1.private_key_to_pem_pkcs8().unwrap()
    );
    let path = destination.join("retention-control-home0-device.pem");
    assert_eq!(
        fs::read(&path).unwrap(),
        h.pair.homes[0].device.0.to_pem().unwrap()
    );
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o7777,
        0o600
    );
    assert_eq!(
        fs::read(destination.join("retention-control-home1-relay.key")).unwrap(),
        h.pair.homes[1].relay.1.private_key_to_pem_pkcs8().unwrap()
    );
    if retained.is_none() {
        fs::remove_dir_all(&destination).unwrap();
        assert!(!destination.exists());
    }
}
#[test]
fn source_owned_cleanup_finishes_before_the_same_physical_lease_is_released() {
    struct CleanupWitness {
        physical: std::sync::Weak<()>,
        observed: Arc<AtomicBool>,
    }
    impl Drop for CleanupWitness {
        fn drop(&mut self) {
            assert!(
                self.physical.strong_count() > 0,
                "physical reservation released before source-owned cleanup"
            );
            self.observed.store(true, Ordering::SeqCst);
        }
    }
    let h = Homes::new(false);
    let binding = h.pair.binding(0);
    let pending = h
        .prepare(0, &binding, Request::Begin(binding.clone()))
        .unwrap();
    let mut source = pending.delivery_source();
    let observed = Arc::new(AtomicBool::new(false));
    let witness = CleanupWitness {
        physical: Arc::downgrade(&source._lease),
        observed: observed.clone(),
    };
    let current = source.current.clone();
    source.current = Arc::new(move || {
        let _keep_cleanup = &witness;
        current()
    });
    let delivery = crate::delivery::Delivery::new(source).unwrap();
    drop(pending);
    assert_eq!(h.pair.homes[0].charged(), 1);
    drop(delivery);
    assert!(observed.load(Ordering::SeqCst));
    assert_eq!(h.pair.homes[0].charged(), 0);
}
fn partial_delivery_output(reset: bool) {
    use crate::delivery::{CarrierEndpoint, Collection};
    use std::sync::mpsc;
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let remote = &h.pair.homes[1];
    let binding = h.pair.binding(0);
    let store = h.empty(0);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let endpoint = CarrierEndpoint::new(
        address,
        "provider.test".into(),
        sha256(&local.server.0.to_der().unwrap()),
        ClientCredentials::from_pem(
            &local.ca.0.to_pem().unwrap(),
            &local.device.0.to_pem().unwrap(),
            &local.device.1.private_key_to_pem_pkcs8().unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    thread::scope(|scope| {
        let (go, wait) = mpsc::channel();
        let client_binding = binding.clone();
        let client = scope.spawn(move || {
            let mut operation = endpoint.operation(Arc::new(AtomicBool::new(false)));
            operation
                .submit_publication(client_binding.clone(), Request::Begin(client_binding))
                .unwrap();
            wait.recv_timeout(Duration::from_secs(5)).unwrap();
            operation.collect()
        });
        let original_cancel = Arc::new(AtomicBool::new(false));
        let session = local
            .server(&local.ca.0)
            .accept(&listener, local.runtime(), original_cancel.clone())
            .unwrap();
        let (mut delivery, work) = session.submit_publication().unwrap();
        let callback = h.endpoint(0);
        let store_ref = &store;
        let worker = scope.spawn(move || work.execute(&callback, store_ref));
        remote
            .server(&remote.ca.0)
            .accept_service(
                &remote.callback,
                remote.runtime(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap()
            .serve_publication_http_once(
                &PublicationWireProfile::new(remote.name.into(), vec!["observer".into()]).unwrap(),
            )
            .unwrap();
        worker.join().unwrap().unwrap();
        go.send(()).unwrap();
        let mut collect = local
            .server(&local.ca.0)
            .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let (written, partial) = mpsc::channel::<Instant>();
        let (resume, release) = mpsc::channel();
        let positive = collect.hold_delivery_after_prefix(written, release);
        let collector = scope.spawn(move || {
            let outcome = collect.collect_delivery(&mut delivery);
            (outcome, delivery)
        });
        partial.recv_timeout(Duration::from_secs(5)).unwrap();
        // The original source lease remains owned after a real positive TLS BIO
        // prefix write and while the rest of this same result is unfinished.
        assert_eq!(local.charged(), 1);
        assert!(
            positive.load(Ordering::SeqCst) > 0,
            "no actual positive TLS BIO output preceded the gate"
        );
        let drain = Instant::now();
        if reset {
            local.runtime().control().reset(1).unwrap();
        } else {
            original_cancel.store(true, Ordering::SeqCst);
        }
        assert_eq!(
            local.charged(),
            1,
            "reset freed physically unfinished response output"
        );
        resume.send(()).unwrap();
        let (outcome, delivery) = collector.join().unwrap();
        assert!(outcome.is_err());
        assert!(
            !matches!(client.join().unwrap(), Ok(Collection::Complete(_))),
            "reset disclosed result after the held partial write"
        );
        assert!(drain.elapsed() < Duration::from_secs(5));
        assert_eq!(local.charged(), 1);
        if reset {
            h.install(0, 4, 1, 2, 0, |_| ());
        }
        assert!(
            matches!(
                h.prepare(0, &binding, Request::Begin(binding.clone())),
                Err(Error::Store(BlobError::Busy))
            ),
            "invalidation freed a still-owned physical reservation"
        );
        drop(delivery);
        assert_eq!(local.charged(), 0);
        let fresh = h
            .prepare(0, &binding, Request::Begin(binding.clone()))
            .unwrap();
        drop(fresh);
        assert_eq!(local.charged(), 0);
    });
}

#[test]
fn wire_collect_retains_reservation_across_actual_partial_tls_output_and_reset() {
    partial_delivery_output(true)
}
#[test]
fn wire_collect_retains_reservation_across_actual_partial_tls_output_and_cancel() {
    partial_delivery_output(false)
}
#[test]
fn own_home_record_and_same_physical_continuation_share_the_aggregate_eight() {
    use crate::delivery::{CarrierEndpoint, Event};
    for existing in [6, 7] {
        let h = Homes::new(false);
        let local = &h.pair.homes[0];
        let foreign = &h.pair.homes[1];
        let binding = h.pair.binding(1);
        let store = h.empty(0);
        let records = (0..existing)
            .map(|_| local.record(&binding).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(local.charged(), existing);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = CarrierEndpoint::new(
            listener.local_addr().unwrap(),
            "provider.test".into(),
            sha256(&local.server.0.to_der().unwrap()),
            ClientCredentials::from_pem(
                &local.ca.0.to_pem().unwrap(),
                &local.device.0.to_pem().unwrap(),
                &local.device.1.private_key_to_pem_pkcs8().unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        thread::scope(|scope| {
            let client_binding = binding.clone();
            let client = scope.spawn(move || {
                let mut operation = endpoint.operation(Arc::new(AtomicBool::new(false)));
                operation
                    .submit_own_home(
                        client_binding.clone(),
                        Request::Status {
                            id: client_binding.id,
                        },
                    )
                    .map(|_| operation)
            });
            let server = tls::Server::new_composed(
                &local.server.0,
                &local.server.1,
                &[local.ca.0.clone(), foreign.ca.0.clone()],
            )
            .unwrap();
            let event = server
                .accept_composed(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap()
                .serve(
                    None,
                    &store,
                    local.name,
                    &PublicationWireProfile::new(local.name.into(), vec!["observer".into()])
                        .unwrap(),
                );
            if existing == 6 {
                let Event::Submitted(delivery, work) = event.unwrap() else {
                    panic!("actual device record submission missing")
                };
                assert!(client.join().unwrap().is_ok());
                assert_eq!(
                    local.charged(),
                    8,
                    "record and physical reservation were not jointly charged"
                );
                drop(work);
                assert_eq!(
                    local.charged(),
                    8,
                    "dropping unexecuted work freed retained delivery ownership"
                );
                drop(delivery);
                assert_eq!(local.charged(), 6);
            } else {
                assert!(
                    matches!(event, Err(Error::Store(BlobError::Busy))),
                    "seventh existing record bypassed aggregate ceiling"
                );
                assert!(client.join().unwrap().is_err());
                assert_eq!(
                    local.charged(),
                    7,
                    "failed own-home capture retained a physical lease or extra record"
                );
            }
        });
        local.runtime().control().reset(1).unwrap();
        drop(records);
        assert_eq!(local.charged(), 0);
    }
}
#[test]
fn old_owned_record_drop_after_reset_cannot_remove_a_new_record() {
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let binding = h.pair.binding(1);
    let old = crate::retrieval::DeliveryRecord::new(local.record(&binding).unwrap());
    assert_eq!(local.charged(), 1);
    local.runtime().control().reset(1).unwrap();
    h.install(0, 4, 1, 2, 0, |_| ());
    let new = crate::retrieval::DeliveryRecord::new(local.record(&binding).unwrap());
    assert_eq!(local.charged(), 1);
    drop(old);
    assert_eq!(local.charged(), 1);
    drop(new);
    assert_eq!(local.charged(), 0);
}
fn carrier_endpoint(
    home: &crate::services::composed_tests::Home,
    who: &Certificate,
    address: std::net::SocketAddr,
) -> crate::delivery::CarrierEndpoint {
    crate::delivery::CarrierEndpoint::new(
        address,
        "provider.test".into(),
        sha256(&home.server.0.to_der().unwrap()),
        ClientCredentials::from_pem(
            &home.ca.0.to_pem().unwrap(),
            &who.0.to_pem().unwrap(),
            &who.1.private_key_to_pem_pkcs8().unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn collect_request_parts(raw: &[u8]) -> (usize, usize, Binding, Request) {
    let binding_len = u16::from_be_bytes([raw[40], raw[41]]) as usize;
    let binding_start = 42;
    let request_length_at = binding_start + binding_len;
    let request_start = request_length_at + 2;
    let Request::Begin(binding) =
        faction_files::private_blobs::decode_request(&raw[binding_start..request_length_at])
            .unwrap()
    else {
        panic!("wrong versioned binding field")
    };
    let request = faction_files::private_blobs::decode_request(&raw[request_start..]).unwrap();
    (request_length_at, request_start, binding, request)
}
#[test]
fn fresh_collect_rejects_stolen_routing_wrong_credentials_and_changed_exact_fields() {
    use crate::{
        authority::{CredentialBinding, Registry},
        delivery::Collection,
    };
    #[derive(Clone, Copy, Debug)]
    enum Refusal {
        OtherDevice,
        ServicePurpose,
        Purpose,
        Binding,
        Body,
        Offset,
        Correlation,
        Action,
    }
    for refusal in [
        Refusal::OtherDevice,
        Refusal::ServicePurpose,
        Refusal::Purpose,
        Refusal::Binding,
        Refusal::Body,
        Refusal::Offset,
        Refusal::Correlation,
        Refusal::Action,
    ] {
        let h = Homes::new(false);
        let local = &h.pair.homes[0];
        let foreign = &h.pair.homes[1];
        let t = tls::wall_now() as i64;
        let other = cert(
            Some((&local.ca.0, &local.ca.1)),
            "other-approved-local-device",
            8801,
            t - 60,
            t + 3600,
        );
        h.other_credentials
            .lock()
            .unwrap()
            .push(("refusal-other".into(), (other.0.clone(), other.1.clone())));
        capture_carrier_optional(&h, "extra-enrolled");
        let mut entries = local.registry(3, 0, foreign).bindings().to_vec();
        entries.push(CredentialBinding {
            certificate_sha256: sha256(&other.0.to_der().unwrap()),
            principal: Principal {
                account: "other@a.test".into(),
                device: 9,
                identity: [3; 32],
            },
            valid_from: tls::wall_now() - 60,
            valid_until: tls::wall_now() + 3600,
            approved: true,
            account_enabled: true,
            device_enabled: true,
            generation: 3,
        });
        local
            .runtime()
            .control()
            .replace_delegation_policy(
                Registry::new(3, 0, vec![local.name.into(), foreign.name.into()], entries).unwrap(),
                h.policy(0, 3, 0, |_| ()),
            )
            .unwrap();
        h.install(1, 3, 0, 3, 0, |_| ());
        let binding = h.pair.binding(0);
        let store = h.empty(0);
        h.publish(0, &binding, &store);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let endpoint = carrier_endpoint(local, &local.device, address);
        let (mut operation, mut delivery, work) = thread::scope(|scope| {
            let client_endpoint = endpoint.clone();
            let client_binding = binding.clone();
            let client = scope.spawn(move || {
                let mut operation = client_endpoint.operation(Arc::new(AtomicBool::new(false)));
                operation
                    .submit_publication(
                        client_binding.clone(),
                        Request::PutChunk {
                            id: client_binding.id,
                            offset: 0,
                            bytes: vec![9; 32],
                        },
                    )
                    .unwrap();
                operation
            });
            let session = local
                .server(&local.ca.0)
                .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            let (delivery, work) = session.submit_publication().unwrap();
            (client.join().unwrap(), delivery, work)
        });
        thread::scope(|scope| {
            let callback = scope.spawn(|| h.serve(1));
            work.execute(&h.endpoint(0), &store).unwrap();
            callback.join().unwrap().unwrap();
        });
        let original = operation.test_collect_frame();
        let mut changed = original.clone();
        let (request_length_at, request_start, mut changed_binding, mut changed_request) =
            collect_request_parts(&changed);
        match refusal {
            Refusal::Purpose => changed[7] = 2,
            Refusal::Binding => {
                changed_binding.recipient.identity[0] ^= 1;
                let bytes =
                    faction_files::private_blobs::encode_request(&Request::Begin(changed_binding))
                        .unwrap();
                changed[42..request_length_at].copy_from_slice(&bytes);
            }
            Refusal::Body => {
                if let Request::PutChunk { bytes, .. } = &mut changed_request {
                    bytes[0] ^= 1;
                }
                let bytes = faction_files::private_blobs::encode_request(&changed_request).unwrap();
                changed[request_start..].copy_from_slice(&bytes);
            }
            Refusal::Offset => {
                if let Request::PutChunk { offset, .. } = &mut changed_request {
                    *offset = 1;
                }
                let bytes = faction_files::private_blobs::encode_request(&changed_request).unwrap();
                changed[request_start..].copy_from_slice(&bytes);
            }
            Refusal::Correlation => changed[8] ^= 1,
            Refusal::Action => {
                let bytes = faction_files::private_blobs::encode_request(&Request::Delete {
                    id: binding.id,
                })
                .unwrap();
                changed.truncate(request_length_at);
                changed.extend((bytes.len() as u16).to_be_bytes());
                changed.extend(bytes);
            }
            _ => (),
        }
        let attacker = match refusal {
            Refusal::OtherDevice => carrier_endpoint(local, &other, address),
            Refusal::ServicePurpose => carrier_endpoint(local, &local.relay, address),
            _ => endpoint.clone(),
        };
        let before = h.bytes(0);
        thread::scope(|scope| {
            let client = scope.spawn(|| attacker.test_exchange(&changed));
            if matches!(refusal, Refusal::ServicePurpose) {
                let session = local
                    .server(&local.ca.0)
                    .accept_service(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                    .unwrap();
                assert!(
                    session.collect_delivery(&mut delivery).is_err(),
                    "{refusal:?}"
                );
            } else {
                let session = local
                    .server(&local.ca.0)
                    .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                    .unwrap();
                assert!(
                    session.collect_delivery(&mut delivery).is_err(),
                    "{refusal:?}"
                );
            }
            assert!(
                client.join().unwrap().is_empty(),
                "refused {refusal:?} disclosed application bytes"
            );
        });
        assert_eq!(
            h.bytes(0),
            before,
            "refused {refusal:?} mutated the committed Store"
        );
        assert!(!local.runtime().state.active.load(Ordering::SeqCst));
        assert_eq!(local.charged(), 1);
        thread::scope(|scope| {
            let client = scope.spawn(|| operation.collect());
            let session = local
                .server(&local.ca.0)
                .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            assert!(session.collect_delivery(&mut delivery).unwrap());
            assert!(matches!(
                client.join().unwrap().unwrap(),
                Collection::Complete(Response::Progress {
                    offset: 32,
                    committed: true,
                    ..
                })
            ));
        });
        // Even the original exact device cannot replay a consumed correlation.
        thread::scope(|scope| {
            let client = scope.spawn(|| endpoint.test_exchange(&original));
            let session = local
                .server(&local.ca.0)
                .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            assert!(session.collect_delivery(&mut delivery).is_err());
            assert!(client.join().unwrap().is_empty());
        });
        capture_carrier_optional(&h, "fresh-collect-refusals");
        drop(delivery);
        assert_eq!(local.charged(), 0);
    }
}
#[test]
fn ready_collect_rechecks_current_local_revocation_reset_and_binding_expiry() {
    for case in 0..3 {
        let h = Homes::new(false);
        let local = &h.pair.homes[0];
        let foreign = &h.pair.homes[1];
        let mut binding = h.pair.binding(0);
        if case == 2 {
            binding.expires = tls::wall_now() + 2;
        }
        let store = h.empty(0);
        h.publish(0, &binding, &store);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let endpoint = carrier_endpoint(local, &local.device, address);
        let (operation, mut delivery, work) = thread::scope(|scope| {
            let client_endpoint = endpoint.clone();
            let b = binding.clone();
            let client = scope.spawn(move || {
                let mut operation = client_endpoint.operation(Arc::new(AtomicBool::new(false)));
                operation
                    .submit_publication(b.clone(), Request::Status { id: b.id })
                    .unwrap();
                operation
            });
            let session = local
                .server(&local.ca.0)
                .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            let (delivery, work) = session.submit_publication().unwrap();
            (client.join().unwrap(), delivery, work)
        });
        thread::scope(|scope| {
            let callback = scope.spawn(|| h.serve(1));
            work.execute(&h.endpoint(0), &store).unwrap();
            callback.join().unwrap().unwrap();
        });
        let copied = operation.test_collect_frame();
        let before = h.bytes(0);
        match case {
            0 => {
                let registry = local.registry(3, 0, foreign);
                let mut entries = registry.bindings().to_vec();
                entries[0].approved = false;
                local
                    .runtime()
                    .control()
                    .replace_registry(
                        crate::authority::Registry::new(3, 0, registry.homes().to_vec(), entries)
                            .unwrap(),
                    )
                    .unwrap();
                h.install(0, 4, 0, 2, 0, |_| ());
            }
            1 => {
                local.runtime().control().reset(1).unwrap();
                h.install(0, 4, 1, 2, 0, |_| ());
            }
            _ => {
                while tls::wall_now() < binding.expires {
                    thread::sleep(Duration::from_millis(20));
                }
            }
        }
        thread::scope(|scope| {
            let client = scope.spawn(|| endpoint.test_exchange(&copied));
            let session = local
                .server(&local.ca.0)
                .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            assert!(
                session.collect_delivery(&mut delivery).is_err(),
                "stale ready result survived case {case}"
            );
            assert!(
                client.join().unwrap().is_empty(),
                "stale ready result disclosed application bytes in case {case}"
            );
        });
        assert_eq!(h.bytes(0), before);
        assert_eq!(local.charged(), 1);
        drop(delivery);
        assert_eq!(local.charged(), 0);
    }
}
#[test]
fn ready_resource_collect_keeps_the_tighter_actual_issued_token_expiry() {
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let foreign = &h.pair.homes[1];
    let binding = h.pair.binding(0);
    let store = h.empty(0);
    h.publish(0, &binding, &store);
    let registry = foreign.registry(3, 0, local);
    let mut entries = registry.bindings().to_vec();
    let expiry = tls::wall_now() + 3;
    entries[0].valid_until = expiry;
    foreign
        .runtime()
        .control()
        .replace_delegation_policy(
            crate::authority::Registry::new(3, 0, registry.homes().to_vec(), entries).unwrap(),
            h.policy(1, 3, 0, |_| ()),
        )
        .unwrap();
    foreign
        .runtime()
        .control()
        .replace_retrieval_policy(
            RetrievalPolicy::new(
                foreign.name.into(),
                vec![(local.name.into(), format!("{}/attachments", local.name))],
            )
            .unwrap(),
        )
        .unwrap();
    h.install(0, 3, 0, 3, 0, |_| ());
    let record = foreign.record(&binding).unwrap();
    let token = foreign.issue(&record);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let endpoint = carrier_endpoint(local, &foreign.relay, address);
    let (operation, mut delivery, work) = thread::scope(|scope| {
        let client_endpoint = endpoint.clone();
        let b = binding.clone();
        let client = scope.spawn(move || {
            let mut operation = client_endpoint.operation(Arc::new(AtomicBool::new(false)));
            operation
                .submit_retrieval(
                    b.clone(),
                    Request::GetChunk {
                        id: b.id,
                        offset: 0,
                    },
                    token,
                )
                .unwrap();
            operation
        });
        let session = local
            .server(&foreign.ca.0)
            .accept_service(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let (delivery, work) = session.submit_retrieval().unwrap();
        (client.join().unwrap(), delivery, work)
    });
    thread::scope(|scope| {
        let callback = scope.spawn(|| {
            foreign
                .server(&foreign.ca.0)
                .accept_service(
                    &foreign.callback,
                    foreign.runtime(),
                    Arc::new(AtomicBool::new(false)),
                )
                .unwrap()
                .serve_http_once()
        });
        work.execute(&local.endpoint(foreign), &store).unwrap();
        callback.join().unwrap().unwrap();
    });
    assert!(delivery.is_finished());
    let copied = operation.test_collect_frame();
    let before = h.bytes(0);
    while tls::wall_now() < expiry {
        thread::sleep(Duration::from_millis(20));
    }
    // Local sender/service policy remains valid; only the actual remote token
    // lifetime has ended. A fresh relay TLS session cannot obtain saved bytes.
    thread::scope(|scope| {
        let client = scope.spawn(|| endpoint.test_exchange(&copied));
        let session = local
            .server(&foreign.ca.0)
            .accept_service(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        assert!(session.collect_delivery(&mut delivery).is_err());
        assert!(client.join().unwrap().is_empty());
    });
    assert_eq!(h.bytes(0), before);
    capture_carrier_optional(&h, "resource-expiry");
    drop(delivery);
    assert_eq!(local.charged(), 0);
    drop(record);
}

#[test]
fn composed_transport_socket_accepted_stream_has_nodelay() {
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let foreign = &h.pair.homes[1];
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::scope(|scope| {
        let peer = scope.spawn(|| local.client(address, &local.device));
        let server = tls::Server::new_composed(
            &local.server.0,
            &local.server.1,
            &[local.ca.0.clone(), foreign.ca.0.clone()],
        )
        .unwrap();
        let incoming = server
            .accept_composed(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let actual = local
            .runtime()
            .state
            .socket
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .nodelay()
            .unwrap();
        drop(incoming);
        drop(peer.join().unwrap());
        assert!(
            actual,
            "actual new composed accepted TCP socket retained Nagle"
        );
    });
}
#[test]
fn composed_transport_socket_carrier_outbound_has_nodelay() {
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let endpoint = carrier_endpoint(local, &local.device, address);
    thread::scope(|scope| {
        let peer = scope.spawn(|| endpoint.test_connect_nodelay());
        let incoming = local
            .server(&local.ca.0)
            .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let actual = peer.join().unwrap().unwrap();
        drop(incoming);
        assert!(
            actual,
            "actual CarrierEndpoint outbound TCP socket retained Nagle"
        );
    });
}
#[test]
fn composed_transport_socket_shared_http_outbound_has_nodelay() {
    let h = Homes::new(false);
    let binding = h.pair.binding(0);
    let store = h.empty(0);
    let pending = h
        .prepare(0, &binding, Request::Begin(binding.clone()))
        .unwrap();
    let mut endpoint = h.endpoint(0);
    let (observed, flags) = std::sync::mpsc::channel();
    endpoint.observe_socket_nodelay(observed);
    thread::scope(|scope| {
        let recipient = scope.spawn(|| h.serve(1));
        assert!(matches!(
            pending.execute(&endpoint, &store),
            Ok(Response::Progress {
                offset: 0,
                committed: false,
                ..
            })
        ));
        recipient.join().unwrap().unwrap();
    });
    assert_eq!(
        flags.try_iter().collect::<Vec<_>>(),
        vec![true],
        "actual shared HttpEndpoint callback TCP socket retained Nagle"
    );
}

#[test]
fn maintained_router_keeps_ordinary_device_store_progress_while_callback_is_held() {
    use crate::{
        authority::{CredentialBinding, Registry},
        delivery::{Collection, Event},
    };
    for same_certificate in [false, true] {
        let h = Homes::new(false);
        let local = &h.pair.homes[0];
        let foreign = &h.pair.homes[1];
        let t = tls::wall_now() as i64;
        let other = cert(
            Some((&local.ca.0, &local.ca.1)),
            "router-other-approved-device",
            8901,
            t - 60,
            t + 3600,
        );
        let other_principal = Principal {
            account: "other@a.test".into(),
            device: 9,
            identity: [3; 32],
        };
        h.other_credentials
            .lock()
            .unwrap()
            .push(("router-other".into(), (other.0.clone(), other.1.clone())));
        capture_carrier_optional(&h, "router-extra-credential");
        let mut entries = local.registry(3, 0, foreign).bindings().to_vec();
        entries.push(CredentialBinding {
            certificate_sha256: sha256(&other.0.to_der().unwrap()),
            principal: other_principal.clone(),
            valid_from: tls::wall_now() - 60,
            valid_until: tls::wall_now() + 3600,
            approved: true,
            account_enabled: true,
            device_enabled: true,
            generation: 3,
        });
        local
            .runtime()
            .control()
            .replace_delegation_policy(
                Registry::new(3, 0, vec![local.name.into(), foreign.name.into()], entries).unwrap(),
                h.policy(0, 3, 0, |_| ()),
            )
            .unwrap();
        h.install(1, 3, 0, 3, 0, |_| ());
        let binding = h.pair.binding(0);
        let store = h.empty(0);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let endpoint = carrier_endpoint(local, &local.device, address);
        let server = tls::Server::new_composed(
            &local.server.0,
            &local.server.1,
            &[local.ca.0.clone(), foreign.ca.0.clone()],
        )
        .unwrap();
        let profile =
            PublicationWireProfile::new(local.name.into(), vec!["observer".into()]).unwrap();
        let (mut operation, mut delivery, work) = thread::scope(|scope| {
            let b = binding.clone();
            let client = scope.spawn(move || {
                let mut op = endpoint.operation(Arc::new(AtomicBool::new(false)));
                op.submit_publication(b.clone(), Request::Begin(b)).unwrap();
                op
            });
            let incoming = server
                .accept_composed(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            let Event::Submitted(delivery, work) =
                incoming.serve(None, &store, local.name, &profile).unwrap()
            else {
                panic!("no actual router submission")
            };
            (client.join().unwrap(), delivery, work)
        });
        thread::scope(|scope| {
            let routes_endpoint = h.endpoint(0);
            // The generic Work deliberately cannot accept arbitrary responses;
            // this test obtains its source-bound publication task through the
            // actual maintained Incoming router and uses configured RelayRoutes.
            let local_endpoint =
                carrier_endpoint(local, &local.relay, local.callback.local_addr().unwrap());
            let peers = vec![crate::delivery::RelayPeer {
                home: foreign.name.into(),
                publication: routes_endpoint,
                introspection: local.endpoint(foreign),
                resource: carrier_endpoint(
                    foreign,
                    &local.relay,
                    foreign.callback.local_addr().unwrap(),
                ),
            }];
            let routes =
                crate::delivery::RelayRoutes::new(local.runtime(), local_endpoint, peers).unwrap();
            let store_ref = &store;
            let worker = scope.spawn(move || work.execute(&routes, store_ref));
            let held = foreign
                .server(&foreign.ca.0)
                .accept_service(
                    &foreign.callback,
                    foreign.runtime(),
                    Arc::new(AtomicBool::new(false)),
                )
                .unwrap();
            let early = scope.spawn(|| operation.collect());
            let incoming = server
                .accept_composed(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            assert!(matches!(
                incoming
                    .serve(Some(&mut delivery), &store, local.name, &profile)
                    .unwrap(),
                Event::Collected(false)
            ));
            assert!(matches!(early.join().unwrap().unwrap(), Collection::Busy));
            let before = delivery.test_original_attempts();
            assert_eq!(before, 3);
            let (credential, principal) = if same_certificate {
                (&local.device, local.principal.clone())
            } else {
                (&other, other_principal.clone())
            };
            let ordinary = Binding {
                sender: principal.clone(),
                recipient: principal,
                id: sha256(&[7; 32]),
                size: 32,
                created: tls::wall_now() - 5,
                expires: tls::wall_now() + 300,
            };
            for request in [
                Request::Begin(ordinary.clone()),
                Request::PutChunk {
                    id: ordinary.id,
                    offset: 0,
                    bytes: vec![7; 32],
                },
                Request::Commit { id: ordinary.id },
                Request::GetChunk {
                    id: ordinary.id,
                    offset: 0,
                },
            ] {
                let client_request = request.clone();
                let client = scope.spawn(move || {
                    let mut c = local.client(address, credential);
                    c.request(&client_request)
                });
                let incoming = server
                    .accept_composed(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                    .unwrap();
                assert!(matches!(
                    incoming
                        .serve(Some(&mut delivery), &store, local.name, &profile)
                        .unwrap(),
                    Event::Served
                ));
                let response = client.join().unwrap().unwrap();
                match request {
                    Request::GetChunk { .. } => assert!(
                        matches!(response,Response::Chunk{offset:0,bytes,total:32} if bytes==vec![7;32])
                    ),
                    _ => assert!(matches!(response, Response::Progress { .. })),
                }
                assert_eq!(local.charged(), 1);
                assert!(!delivery.is_finished());
            }
            assert_eq!(
                delivery.test_original_attempts(),
                if same_certificate { 7 } else { 3 },
                "router budget effects differ from the authenticated credential selection"
            );
            held.serve_publication_http_once(
                &PublicationWireProfile::new(foreign.name.into(), vec!["observer".into()]).unwrap(),
            )
            .unwrap();
            worker.join().unwrap().unwrap();
        });
        thread::scope(|scope| {
            let client = scope.spawn(|| operation.collect());
            let incoming = server
                .accept_composed(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            assert!(matches!(
                incoming
                    .serve(Some(&mut delivery), &store, local.name, &profile)
                    .unwrap(),
                Event::Collected(true)
            ));
            assert!(matches!(
                client.join().unwrap().unwrap(),
                Collection::Complete(Response::Progress {
                    offset: 0,
                    committed: false,
                    ..
                })
            ));
        });
        assert_eq!(
            delivery.test_original_attempts(),
            if same_certificate { 8 } else { 4 }
        );
        capture_carrier_optional(&h, "router-progress");
        drop(delivery);
        assert_eq!(local.charged(), 0);
    }
}

#[test]
fn maintained_router_preserves_one_legacy_same_home_client_for_all_eight_requests() {
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let foreign = &h.pair.homes[1];
    let store = h.empty(0);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = tls::Server::new_composed(
        &local.server.0,
        &local.server.1,
        &[local.ca.0.clone(), foreign.ca.0.clone()],
    )
    .unwrap();
    let profile = PublicationWireProfile::new(local.name.into(), vec!["observer".into()]).unwrap();
    let binding = Binding {
        sender: local.principal.clone(),
        recipient: local.principal.clone(),
        id: sha256(&[21; 32]),
        size: 32,
        created: tls::wall_now() - 5,
        expires: tls::wall_now() + 300,
    };
    thread::scope(|scope| {
        let client = scope.spawn(|| {
            let mut client = local.client(address, &local.device);
            for (number, request) in [Request::Begin(binding.clone()), Request::PutChunk{id:binding.id,offset:0,bytes:vec![21;32]}, Request::Commit{id:binding.id}, Request::Commit{id:binding.id}, Request::Status{id:binding.id}, Request::GetChunk{id:binding.id,offset:0}, Request::GetChunk{id:binding.id,offset:32}, Request::Delete{id:binding.id}].into_iter().enumerate() {
                let response=client.request(&request).unwrap_or_else(|error| panic!("same retained real TLS client failed request{number}: {error:?}"));
                match request { Request::GetChunk{offset:0,..}=>assert!(matches!(response,Response::Chunk{offset:0,bytes,total:32} if bytes==vec![21;32])), Request::GetChunk{offset:32,..}=>assert!(matches!(response,Response::Chunk{offset:32,bytes,total:32} if bytes.is_empty())), Request::Delete{..}=>assert!(matches!(response,Response::Deleted)), _=>assert!(matches!(response,Response::Progress{..})) }
            }
            assert!(client.request(&Request::Status{id:binding.id}).is_err(),"ninth request must retain the original eight-attempt session limit");
        });
        let incoming = server
            .accept_composed(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let result = incoming.serve(None, &store, local.name, &profile);
        client.join().unwrap();
        assert!(
            matches!(result, Err(crate::Error::Transport(tls::Stage::Limit))),
            "same original session must reach its original limit"
        );
    });
    capture_carrier_optional(&h, "retained-legacy");
    assert_eq!(local.charged(), 0);
}

// macOS has no reliable bounded listener-backlog saturation precondition here.
// Linux/NetBSD exercise a genuinely pending TCP connect on a never-accepted
// backlog-one listener, two owned filler sockets and one selected endpoint.
fn pending_tcp_endpoint_cancel(http: bool) {
    use rustix::{
        event::{PollFd, PollFlags, Timespec, poll},
        net::{self, AddressFamily, SocketFlags, SocketType},
    };
    if !cfg!(any(target_os = "linux", target_os = "netbsd")) {
        eprintln!(
            "UNAVAILABLE pending TCP backlog precondition on this host; no pending-connect runtime claim"
        );
        return;
    }
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let foreign = &h.pair.homes[1];
    let listener = net::socket_with(
        AddressFamily::INET,
        SocketType::STREAM,
        SocketFlags::empty(),
        Some(net::ipproto::TCP),
    )
    .unwrap();
    rustix::io::fcntl_setfd(&listener, rustix::io::FdFlags::CLOEXEC).unwrap();
    net::bind(
        &listener,
        &std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, 0),
    )
    .unwrap();
    net::listen(&listener, 1).unwrap();
    let address = std::net::SocketAddr::try_from(net::getsockname(&listener).unwrap()).unwrap();
    let mut fillers = Vec::new();
    let mut completed = 0;
    for _ in 0..2 {
        let fd = net::socket_with(
            AddressFamily::INET,
            SocketType::STREAM,
            SocketFlags::empty(),
            Some(net::ipproto::TCP),
        )
        .unwrap();
        rustix::io::fcntl_setfd(&fd, rustix::io::FdFlags::CLOEXEC).unwrap();
        let flags = rustix::fs::fcntl_getfl(&fd).unwrap();
        rustix::fs::fcntl_setfl(&fd, flags | rustix::fs::OFlags::NONBLOCK).unwrap();
        let result = net::connect(&fd, &address);
        assert!(
            result.is_ok() || result == Err(rustix::io::Errno::INPROGRESS),
            "owned backlog filler connect setup: {result:?}"
        );
        let mut fds = [PollFd::new(&fd, PollFlags::OUT)];
        poll(
            &mut fds,
            Some(&Timespec::try_from(Duration::from_millis(100)).unwrap()),
        )
        .unwrap();
        net::sockopt::socket_error(&fd).unwrap().unwrap();
        completed += usize::from(fds[0].revents().contains(PollFlags::OUT));
        fillers.push(fd);
    }
    assert!(
        completed >= 1,
        "backlog control needs at least one established owned filler"
    );
    let carrier = carrier_endpoint(local, &local.device, address);
    let endpoint = HttpEndpoint::new(
        address,
        foreign.name.into(),
        "provider.test".into(),
        sha256(&foreign.server.0.to_der().unwrap()),
        ClientCredentials::from_pem(
            &foreign.ca.0.to_pem().unwrap(),
            &local.outgoing.0.to_pem().unwrap(),
            &local.outgoing.1.private_key_to_pem_pkcs8().unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let worker_cancel = cancel.clone();
    let binding = h.pair.binding(0);
    let control = local.runtime().control();
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    thread::scope(|scope| {
        let worker = scope.spawn(move || {
            started_tx.send(()).unwrap();
            let start = Instant::now();
            let result = if http {
                control
                    .query_introspection(
                        &endpoint,
                        &faction_delegation_codec::UnverifiedIntrospectionRequest {
                            client_id: "observer".into(),
                            token: faction_delegation_codec::UnverifiedToken::from_bytes([41; 32]),
                        },
                        start + tls::CYCLE_LIMIT,
                        worker_cancel,
                    )
                    .map(|_| ())
            } else {
                let mut operation = carrier.operation(worker_cancel);
                operation
                    .submit_publication(binding.clone(), Request::Begin(binding))
                    .map_err(crate::Error::Transport)
            };
            done_tx.send(()).unwrap();
            (result, start.elapsed())
        });
        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        thread::sleep(Duration::from_millis(100));
        assert!(
            done_rx.try_recv().is_err(),
            "backlog must hold the selected actual endpoint before cancellation"
        );
        let cancelled_at = Instant::now();
        cancel.store(true, Ordering::SeqCst);
        let promptly_done = done_rx.recv_timeout(Duration::from_secs(1)).is_ok();
        let (result, total) = worker.join().unwrap();
        let after_cancel = cancelled_at.elapsed();
        eprintln!(
            "pending TCP actual endpoint http={http} filler_completed={completed} control_sockets=4 total_ms={} cancel_join_ms={} result={result:?}",
            total.as_millis(),
            after_cancel.as_millis()
        );
        assert!(
            promptly_done && after_cancel < Duration::from_secs(1),
            "pending TCP cancellation must join below1s and existing5s drain, without shortening the original5s admission"
        );
        assert!(matches!(
            result,
            Err(crate::Error::Transport(tls::Stage::Cancelled))
        ));
    });
    drop(fillers);
    drop(listener);
    capture_carrier_optional(&h, "pending-tcp-cancel");
}
#[test]
fn composed_pending_tcp_carrier_connect_cancels_within_existing_drain() {
    pending_tcp_endpoint_cancel(false);
}
#[test]
fn composed_pending_tcp_shared_http_connect_cancels_within_existing_drain() {
    pending_tcp_endpoint_cancel(true);
}

#[test]
fn bounded_collector_delivers_a_valid_delayed_callback_within_original_attempts() {
    use crate::delivery::Collection;
    use std::sync::mpsc;
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let remote = &h.pair.homes[1];
    let binding = h.pair.binding(0);
    let store = h.empty(0);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let endpoint = carrier_endpoint(local, &local.device, address);
    let began = Instant::now();
    thread::scope(|scope| {
        let b = binding.clone();
        let client = scope.spawn(move || {
            let mut operation = endpoint.operation(Arc::new(AtomicBool::new(false)));
            operation
                .submit_publication(b.clone(), Request::Begin(b))
                .unwrap();
            operation
        });
        let mut session = local
            .server(&local.ca.0)
            .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let (entered, waiting) = mpsc::channel();
        let (release, resume) = mpsc::channel();
        let positive = session.hold_delivery_after_prefix(entered, resume);
        let release_receipt = scope.spawn(move || {
            waiting.recv_timeout(Duration::from_secs(5)).unwrap();
            thread::sleep(Duration::from_millis(100));
            release.send(()).unwrap();
        });
        let (mut delivery, work) = session.submit_publication().unwrap();
        release_receipt.join().unwrap();
        assert!(
            positive.load(Ordering::SeqCst) > 0,
            "receipt RTT includes an actual positive TLS prefix and delayed remainder"
        );
        let mut operation = client.join().unwrap();
        let callback = h.endpoint(0);
        let store_ref = &store;
        let worker = scope.spawn(move || work.execute(&callback, store_ref));
        let response = scope.spawn(|| {
            let service = remote
                .server(&remote.ca.0)
                .accept_service(
                    &remote.callback,
                    remote.runtime(),
                    Arc::new(AtomicBool::new(false)),
                )
                .unwrap();
            thread::sleep(Duration::from_millis(700));
            service
                .serve_publication_http_once(
                    &PublicationWireProfile::new(remote.name.into(), vec!["observer".into()])
                        .unwrap(),
                )
                .unwrap();
        });
        let mut collected = false;
        let mut collects = 0;
        for _ in 0..6 {
            collects += 1;
            let (ready, result) = thread::scope(|iteration| {
                let collector = iteration.spawn(|| operation.collect_paced());
                let session = local
                    .server(&local.ca.0)
                    .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                    .unwrap();
                let ready = session.collect_delivery(&mut delivery).unwrap();
                (ready, collector.join().unwrap().unwrap())
            });
            match result {
                Collection::Busy => {
                    assert!(!ready);
                    thread::sleep(Duration::from_millis(100));
                }
                Collection::Complete(value) => {
                    assert!(ready);
                    assert!(matches!(
                        value,
                        Response::Progress {
                            offset: 0,
                            committed: false,
                            ..
                        }
                    ));
                    collected = true;
                    break;
                }
            }
        }
        response.join().unwrap();
        worker.join().unwrap().unwrap();
        assert!(
            began.elapsed() < tls::CYCLE_LIMIT,
            "the real authenticated callback completed inside the original window"
        );
        assert!(delivery.is_finished());
        assert!(delivery.test_original_attempts() <= 8);
        assert_eq!(delivery.test_original_attempts(), 2 + collects);
        assert_eq!(local.charged(), 1);
        assert!(
            h.bytes(0)
                .windows(binding.id.len())
                .any(|bytes| bytes == binding.id)
        );
        capture_carrier_optional(&h, "delayed-collector");
        assert!(
            collected,
            "six greedy fresh collects exhausted shared8 before a valid delayed callback could be collected"
        );
        drop(delivery);
        assert_eq!(local.charged(), 0);
    });
}

fn paced_measured_wait_control(deadline: bool) {
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let binding = h.pair.binding(0);
    let _store = h.empty(0);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let endpoint = carrier_endpoint(local, &local.device, address);
    let cancel = Arc::new(AtomicBool::new(false));
    thread::scope(|scope| {
        let b = binding.clone();
        let original_cancel = cancel.clone();
        let client = scope.spawn(move || {
            let mut op = endpoint.operation(original_cancel);
            op.submit_publication(b.clone(), Request::Begin(b)).unwrap();
            op
        });
        let mut session = local
            .server(&local.ca.0)
            .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let (entered, waiting) = std::sync::mpsc::channel();
        let (release, resume) = std::sync::mpsc::channel();
        let positive = session.hold_delivery_after_prefix(entered, resume);
        let release_receipt = scope.spawn(move || {
            waiting.recv_timeout(Duration::from_secs(5)).unwrap();
            thread::sleep(Duration::from_millis(100));
            release.send(()).unwrap();
        });
        let (delivery, work) = session.submit_publication().unwrap();
        release_receipt.join().unwrap();
        let mut operation = client.join().unwrap();
        assert!(positive.load(Ordering::SeqCst) > 0);
        assert_eq!(operation.test_attempts(), 1);
        assert_eq!(delivery.test_original_attempts(), 1);
        assert_eq!(local.charged(), 1);
        if deadline {
            operation.test_narrow_deadline(Duration::from_millis(20));
        }
        let (entered, waiting) = std::sync::mpsc::channel();
        let (done, finished) = std::sync::mpsc::channel();
        let began = Instant::now();
        let collector = scope.spawn(move || {
            entered.send(()).unwrap();
            let result = operation.collect_paced();
            done.send(()).unwrap();
            (result, operation)
        });
        waiting.recv_timeout(Duration::from_secs(1)).unwrap();
        if !deadline {
            thread::sleep(Duration::from_millis(25));
            assert!(
                finished.try_recv().is_err(),
                "the real measured local wait must still be in progress"
            );
            cancel.store(true, Ordering::SeqCst);
        }
        let (result, operation) = collector.join().unwrap();
        assert!(began.elapsed() < Duration::from_secs(1));
        assert_eq!(
            result.err(),
            Some(if deadline {
                tls::Stage::Deadline
            } else {
                tls::Stage::Cancelled
            })
        );
        assert_eq!(operation.test_attempts(), 1);
        assert_eq!(delivery.test_original_attempts(), 1);
        listener.set_nonblocking(true).unwrap();
        assert!(
            matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock),
            "a cancelled/expired local pacing wait must not open another TCP connection"
        );
        assert_eq!(local.charged(), 1);
        assert_eq!(h.bytes(0).len(), 17);
        capture_carrier_optional(&h, "paced-wait-control");
        drop(work);
        drop(delivery);
        assert_eq!(local.charged(), 0);
    });
}
#[test]
fn paced_collect_cancellation_during_measured_wait_spends_no_extra_connection_or_attempt() {
    paced_measured_wait_control(false);
}
#[test]
fn paced_collect_conservative_deadline_during_measured_wait_does_not_refresh() {
    paced_measured_wait_control(true);
}
#[test]
fn paced_collect_exhaustion_keeps_raw_busy_immediate_and_never_refreshes_attempts() {
    use crate::delivery::Collection;
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let binding = h.pair.binding(0);
    let _store = h.empty(0);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let endpoint = carrier_endpoint(local, &local.device, address);
    thread::scope(|scope| {
        let b = binding.clone();
        let client = scope.spawn(move || {
            let mut op = endpoint.operation(Arc::new(AtomicBool::new(false)));
            op.submit_publication(b.clone(), Request::Begin(b)).unwrap();
            op
        });
        let session = local
            .server(&local.ca.0)
            .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let (mut delivery, work) = session.submit_publication().unwrap();
        let mut operation = client.join().unwrap();
        for expected in 2..=8 {
            thread::scope(|iteration| {
                let client = iteration.spawn(|| operation.collect());
                let session = local
                    .server(&local.ca.0)
                    .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                    .unwrap();
                let began = Instant::now();
                assert!(!session.collect_delivery(&mut delivery).unwrap());
                assert!(began.elapsed() < Duration::from_millis(100));
                assert!(matches!(client.join().unwrap().unwrap(), Collection::Busy));
            });
            assert_eq!(operation.test_attempts(), expected);
            assert_eq!(delivery.test_original_attempts(), expected);
        }
        for _ in 0..3 {
            let began = Instant::now();
            assert_eq!(operation.collect_paced().err(), Some(tls::Stage::Limit));
            assert!(began.elapsed() < Duration::from_millis(100));
            assert_eq!(operation.test_attempts(), 8);
            assert_eq!(delivery.test_original_attempts(), 8);
        }
        listener.set_nonblocking(true).unwrap();
        assert!(
            matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
        assert_eq!(local.charged(), 1);
        assert_eq!(h.bytes(0).len(), 17);
        capture_carrier_optional(&h, "paced-attempt-control");
        drop(work);
        drop(delivery);
        assert_eq!(local.charged(), 0);
    });
}

#[test]
fn carrier_parent_deadline_and_cancellation_refuse_before_any_tcp_operation() {
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let binding = h.pair.binding(0);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let endpoint = carrier_endpoint(local, &local.device, listener.local_addr().unwrap());
    assert!(matches!(
        endpoint.operation_until(
            Instant::now() - Duration::from_millis(1),
            Arc::new(AtomicBool::new(false))
        ),
        Err(tls::Stage::Deadline)
    ));
    assert!(matches!(
        endpoint.operation_until(
            Instant::now() + tls::CYCLE_LIMIT,
            Arc::new(AtomicBool::new(true))
        ),
        Err(tls::Stage::Cancelled)
    ));
    let parent = Instant::now() + Duration::from_millis(20);
    let mut operation = endpoint
        .operation_until(parent, Arc::new(AtomicBool::new(false)))
        .unwrap();
    thread::sleep(Duration::from_millis(40));
    assert_eq!(
        operation
            .submit_publication(binding.clone(), Request::Begin(binding))
            .err(),
        Some(tls::Stage::Deadline)
    );
    assert!(matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock));
    assert_eq!(local.charged(), 0);
    capture_carrier_optional(&h, "parent-deadline-no-io");
}

#[cfg(feature = "durable-policy")]
#[test]
fn admitted_introspection_survives_short_contended_service_purpose_lookup() {
    service_purpose_contention(false);
}
#[cfg(feature = "durable-policy")]
#[test]
fn admitted_introspection_cancelled_during_contended_service_purpose_lookup() {
    service_purpose_contention(true);
}
#[cfg(feature = "durable-policy")]
fn service_purpose_contention(cancelled: bool) {
    use faction_delegation_codec as codec;
    use std::sync::mpsc;
    let h = Homes::new(true);
    let local = &h.pair.homes[0];
    let foreign = &h.pair.homes[1];
    let binding = h.pair.binding(1);
    let record = crate::retrieval::DeliveryRecord::new(local.record(&binding).unwrap());
    let reference = local
        .runtime()
        .control()
        .provision_retrieval(record.record())
        .unwrap();
    let issued = local.service_call(&local.relay, &local.ca.0, |mut session| {
        session.issue_retrieval(&reference, "relay").unwrap()
    });
    let request = codec::UnverifiedIntrospectionRequest {
        client_id: "inspector".into(),
        token: codec::UnverifiedToken::from_bytes(*issued.token().expose_bytes()),
    };
    let prefix = format!(
        "service-purpose-contention-{}-cancelled-{cancelled}",
        std::process::id()
    );
    retain_synthetic_fixture(&format!("{prefix}-reference.bin"), reference.expose_bytes());
    retain_synthetic_fixture(
        &format!("{prefix}-request.http"),
        &codec::encode_introspection_http(local.name, &request).unwrap(),
    );
    retain_synthetic_fixture(
        &format!("{prefix}-binding.bin"),
        &faction_files::private_blobs::encode_request(&Request::Begin(binding.clone())).unwrap(),
    );
    for i in 0..2 {
        retain_synthetic_fixture(
            &format!("{prefix}-policy{i}.snapshot"),
            &fs::read(
                h.pair
                    .root
                    .join(format!("policy{i}/provider-policy.snapshot")),
            )
            .unwrap(),
        );
    }
    let store = h.empty(0);
    let before = h.bytes(0);
    let endpoint = foreign.endpoint(local);
    let start = Instant::now();
    let server_cancel = Arc::new(AtomicBool::new(false));
    let (served, reply, attempted, held, positive, response_started) = thread::scope(|scope| {
        let caller = scope.spawn(|| {
            foreign.runtime().control().query_introspection(
                &endpoint,
                &request,
                start + tls::CYCLE_LIMIT,
                Arc::new(AtomicBool::new(false)),
            )
        });
        // Admission uses the real registered foreign service leaf and finishes
        // before this synthetic, short Authority critical section begins.
        let mut incoming = local
            .server(&local.ca.0)
            .accept_composed(&local.callback, local.runtime(), server_cancel.clone())
            .unwrap();
        let (entered, observed) = mpsc::sync_channel(1);
        incoming.observe_service_purpose_attempt(entered);
        let (write_entered, write_observed) = mpsc::channel();
        let (write_release, write_resume) = mpsc::channel();
        write_release.send(()).unwrap();
        let positive = incoming.observe_service_response_prefix(write_entered, write_resume);
        let held_at = Instant::now();
        let authority = local.runtime().state.authority.lock().unwrap();
        assert_eq!(authority.registry.as_ref().unwrap().generation(), 2);
        let served = scope.spawn(|| {
            incoming
                .serve(
                    None,
                    &store,
                    local.name,
                    &PublicationWireProfile::new(local.name.into(), vec!["observer".into()])
                        .unwrap(),
                )
                .map(|event| matches!(event, tls::delivery::Event::Served))
        });
        let attempted = observed.recv_timeout(Duration::from_secs(1));
        if cancelled {
            server_cancel.store(true, Ordering::SeqCst);
        }
        // Release even on a missing witness, before joins/response assertions.
        // ServiceSession drop closes the socket; it does not lock Authority.
        drop(authority);
        let held = held_at.elapsed();
        (
            served.join().unwrap(),
            caller.join().unwrap(),
            attempted,
            held,
            positive.load(Ordering::SeqCst),
            write_observed.try_recv().is_ok(),
        )
    });
    capture_carrier_optional(&h, "service-purpose-contention-after");
    assert_eq!(
        attempted.unwrap(),
        0,
        "the admitted service did not start with its original fresh budget"
    );
    assert!(
        held < Duration::from_secs(1),
        "the controlled short critical section outlasted its witness bound"
    );
    assert!(
        start.elapsed() < tls::CYCLE_LIMIT,
        "introspection outlasted the original operation window"
    );
    assert_eq!(
        h.bytes(0),
        before,
        "metadata lookup changed actual Store bytes"
    );
    eprintln!(
        "controlled service-purpose contention cancelled={cancelled} attempts_before_http=0 server={served:?} client_error={:?} positive_tls_prefix={positive} response_started={response_started} held_us={} elapsed_ms={}",
        reply.as_ref().err(),
        held.as_micros(),
        start.elapsed().as_millis()
    );
    if cancelled {
        assert_eq!(
            positive, 0,
            "cancellation disclosed post-admission TLS response bytes"
        );
        assert!(
            !response_started,
            "cancelled response reached its socket gate"
        );
        assert!(
            reply.is_err(),
            "cancelled service returned authenticated metadata"
        );
        assert_eq!(
            served,
            Err(Error::Transport(tls::Stage::Cancelled)),
            "contended lookup must recheck the original cancellation after acquiring Authority"
        );
        return;
    }
    assert!(
        matches!(served, Ok(true)),
        "admitted current Introspection connection was discarded: {served:?}"
    );
    let codec::UnverifiedIntrospection::Retrieval { claims, action } = reply.unwrap() else {
        panic!("current token did not return an active authenticated response")
    };
    assert_eq!(claims.binding.recipient.account, binding.recipient.account);
    assert_eq!(claims.binding.sender.account, binding.sender.account);
    assert_eq!(claims.generation, 2);
    assert_eq!(claims.reset, 0);
    assert_eq!(
        action,
        codec::UnverifiedRetrievalAction::GetChunk { offset: 0 }
    );
    assert!(
        positive > 0 && response_started,
        "the healthy authenticated response must exercise the real post-admission TLS write observer"
    );
}

#[cfg(feature = "durable-policy")]
fn metadata_capture_work(
    h: &Homes,
    store: &Mutex<Store>,
    own_home: bool,
    cancel: Arc<AtomicBool>,
) -> (
    TcpListener,
    crate::delivery::Operation,
    crate::delivery::Delivery,
    crate::delivery::Work,
    Binding,
) {
    let local = &h.pair.homes[0];
    let binding = h.pair.binding(usize::from(own_home));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = carrier_endpoint(local, &local.device, listener.local_addr().unwrap());
    let (operation, delivery, work) = thread::scope(|scope| {
        let captured = binding.clone();
        let client = scope.spawn(move || {
            let mut operation = endpoint.operation(Arc::new(AtomicBool::new(false)));
            if own_home {
                operation
                    .submit_own_home(captured.clone(), Request::Status { id: captured.id })
                    .unwrap();
            } else {
                operation
                    .submit_publication(captured.clone(), Request::Begin(captured))
                    .unwrap();
            }
            operation
        });
        let incoming = local
            .server(&local.ca.0)
            .accept_composed(&listener, local.runtime(), cancel)
            .unwrap();
        let crate::delivery::Event::Submitted(delivery, work) = incoming
            .serve(
                None,
                store,
                local.name,
                &PublicationWireProfile::new(local.name.into(), vec!["observer".into()]).unwrap(),
            )
            .unwrap()
        else {
            panic!("real device did not submit a source-bound operation")
        };
        (client.join().unwrap(), delivery, work)
    });
    (listener, operation, delivery, work, binding)
}
#[cfg(feature = "durable-policy")]
fn metadata_collect(
    h: &Homes,
    listener: &TcpListener,
    operation: &mut crate::delivery::Operation,
    delivery: &mut crate::delivery::Delivery,
    store: &Mutex<Store>,
) -> (
    Result<bool, Error>,
    Result<crate::delivery::Collection, tls::Stage>,
) {
    let local = &h.pair.homes[0];
    thread::scope(|scope| {
        let client = scope.spawn(|| operation.collect());
        let incoming = local
            .server(&local.ca.0)
            .accept_composed(listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let served = incoming
            .serve(
                Some(delivery),
                store,
                local.name,
                &PublicationWireProfile::new(local.name.into(), vec!["observer".into()]).unwrap(),
            )
            .map(|event| match event {
                crate::delivery::Event::Collected(ready) => ready,
                _ => panic!("actual collect used a different route"),
            });
        (served, client.join().unwrap())
    })
}
#[cfg(feature = "durable-policy")]
fn metadata_capture_inputs(h: &Homes, operation: &crate::delivery::Operation, label: &str) {
    let prefix = format!("{label}-{}", std::process::id());
    retain_synthetic_fixture(
        &format!("{prefix}-collect-frame.bin"),
        &operation.test_collect_frame(),
    );
    for i in 0..2 {
        retain_synthetic_fixture(
            &format!("{prefix}-policy{i}.snapshot"),
            &fs::read(
                h.pair
                    .root
                    .join(format!("policy{i}/provider-policy.snapshot")),
            )
            .unwrap(),
        );
    }
}
#[cfg(feature = "durable-policy")]
fn metadata_deadline_contention(cancelled: bool) {
    use crate::delivery::Collection;
    let h = Homes::new(true);
    let local = &h.pair.homes[0];
    let store = h.empty(0);
    let before = h.bytes(0);
    let cancel = Arc::new(AtomicBool::new(false));
    let start = Instant::now();
    let (listener, mut operation, mut delivery, work, _) =
        metadata_capture_work(&h, &store, true, cancel.clone());
    metadata_capture_inputs(
        &h,
        &operation,
        &format!("deadline-metadata-cancelled-{cancelled}"),
    );
    assert!(!delivery.is_finished());
    assert_eq!(delivery.test_original_attempts(), 1);
    assert_eq!(
        local.charged(),
        2,
        "real own-home record and physical owner remain charged"
    );
    let (entered, observed) = std::sync::mpsc::sync_channel(1);
    delivery.observe_deadline_metadata_attempt(entered);
    let (expired, attempted, held) = thread::scope(|scope| {
        let guard = delivery.hold_deadline_metadata();
        let original = *guard;
        let held_at = Instant::now();
        let contender = scope.spawn(|| delivery.is_expired());
        let attempted = observed.recv_timeout(Duration::from_secs(1));
        if cancelled {
            cancel.store(true, Ordering::SeqCst);
        }
        drop(guard);
        let held = held_at.elapsed();
        let expired = contender.join().unwrap();
        assert_eq!(
            *delivery.hold_deadline_metadata(),
            original,
            "metadata witness changed the actual deadline"
        );
        (expired, attempted, held)
    });
    assert_eq!(
        attempted.unwrap(),
        (true, 1),
        "the actual deadline lookup must observe WouldBlock without a charge"
    );
    assert!(held < Duration::from_secs(1));
    assert_eq!(delivery.test_original_attempts(), 1);
    assert_eq!(local.charged(), 2);
    assert!(
        !delivery.is_finished(),
        "genuine unfinished Work must remain retained"
    );
    let (served, collected) =
        metadata_collect(&h, &listener, &mut operation, &mut delivery, &store);
    if cancelled {
        assert!(
            served.is_err() && collected.is_err(),
            "cancelled source disclosed a response"
        );
    } else {
        assert_eq!(served, Ok(false));
        assert!(
            matches!(collected, Ok(Collection::Busy)),
            "healthy unfinished operation must still collect immediate Busy"
        );
    }
    assert_eq!(delivery.test_original_attempts(), 2);
    assert_eq!(local.charged(), 2);
    assert_eq!(h.bytes(0), before);
    assert!(start.elapsed() < tls::CYCLE_LIMIT);
    capture_carrier_optional(&h, "deadline-metadata-contention");
    eprintln!(
        "deadline metadata cancelled={cancelled} expired={expired} metadata_attempts=1 after_collect_attempts=2 physical_and_record=2 held_us={}",
        held.as_micros()
    );
    drop(work);
    drop(delivery);
    assert_eq!(local.charged(), 0);
    assert_eq!(
        expired, cancelled,
        "short unchanged deadline metadata contention falsely expired a healthy delivery"
    );
}
#[cfg(feature = "durable-policy")]
#[test]
fn metadata_read_side_deadline_contention_keeps_live_unfinished_delivery() {
    metadata_deadline_contention(false);
}
#[cfg(feature = "durable-policy")]
#[test]
fn metadata_read_side_deadline_contention_preserves_cancellation_refusal() {
    metadata_deadline_contention(true);
}

#[cfg(feature = "durable-policy")]
fn metadata_routes_contention(cancelled: bool) {
    use crate::delivery::{Collection, RelayPeer, RelayRoutes};
    let h = Homes::new(true);
    let local = &h.pair.homes[0];
    let foreign = &h.pair.homes[1];
    let store = h.empty(0);
    let recipient_store = h.empty(1);
    let before = h.bytes(0);
    let recipient_before = h.bytes(1);
    // Fully validate fixed routes before capturing the actual sender Work.
    let mut routes = RelayRoutes::new(
        local.runtime(),
        carrier_endpoint(local, &local.relay, local.callback.local_addr().unwrap()),
        vec![RelayPeer {
            home: foreign.name.into(),
            publication: h.endpoint(0),
            introspection: local.endpoint(foreign),
            resource: carrier_endpoint(
                foreign,
                &local.relay,
                foreign.callback.local_addr().unwrap(),
            ),
        }],
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let start = Instant::now();
    let (listener, mut operation, mut delivery, work, binding) =
        metadata_capture_work(&h, &store, false, cancel.clone());
    metadata_capture_inputs(
        &h,
        &operation,
        &format!("routes-metadata-cancelled-{cancelled}"),
    );
    assert_eq!(delivery.test_original_attempts(), 1);
    assert_eq!(local.charged(), 1);
    let (entered, observed) = std::sync::mpsc::sync_channel(1);
    routes.observe_authority_metadata_attempt(entered);
    let callback_cancel = Arc::new(AtomicBool::new(false));
    let (result, callback, attempted, held) = thread::scope(|scope| {
        let callback_flag = callback_cancel.clone();
        let callback = scope.spawn(|| {
            let incoming = foreign
                .server(&foreign.ca.0)
                .accept_composed(&foreign.callback, foreign.runtime(), callback_flag)
                .map_err(Error::Transport)?;
            h.callbacks[1].fetch_add(1, Ordering::SeqCst);
            incoming
                .serve(
                    None,
                    &recipient_store,
                    foreign.name,
                    &PublicationWireProfile::new(foreign.name.into(), vec!["observer".into()])
                        .unwrap(),
                )
                .map(|event| matches!(event, crate::delivery::Event::Served))
        });
        let authority = local.runtime().state.authority.lock().unwrap();
        let held_at = Instant::now();
        assert_eq!(authority.registry.as_ref().unwrap().generation(), 2);
        let worker = scope.spawn(|| work.execute(&routes, &store));
        let attempted = observed.recv_timeout(Duration::from_secs(1));
        if cancelled {
            cancel.store(true, Ordering::SeqCst);
        }
        drop(authority);
        let held = held_at.elapsed();
        let result = worker.join().unwrap();
        if result.is_err() {
            callback_cancel.store(true, Ordering::SeqCst);
        }
        (result, callback.join().unwrap(), attempted, held)
    });
    assert!(
        attempted.unwrap(),
        "the actual routes policy lookup must observe WouldBlock"
    );
    assert!(held < Duration::from_secs(1));
    assert!(delivery.is_finished());
    assert_eq!(local.charged(), 1);
    let before_collect = delivery.test_original_attempts();
    assert!((1..=2).contains(&before_collect));
    let (served, collected) =
        metadata_collect(&h, &listener, &mut operation, &mut delivery, &store);
    assert_eq!(delivery.test_original_attempts(), before_collect + 1);
    assert!(
        delivery.test_original_attempts() < tls::MAX_OPERATIONS,
        "no response may be accepted after spending the source budget"
    );
    assert_eq!(local.charged(), 1);
    assert_eq!(
        h.bytes(1),
        recipient_before,
        "recipient-current observation must not mutate recipient Store"
    );
    assert!(start.elapsed() < tls::CYCLE_LIMIT);
    let callbacks = h.callbacks[1].load(Ordering::SeqCst);
    eprintln!(
        "routes metadata cancelled={cancelled} work={result:?} callback={callback:?} callbacks={callbacks} collect_error={:?} attempts={} held_us={}",
        collected.as_ref().err(),
        delivery.test_original_attempts(),
        held.as_micros()
    );
    capture_carrier_optional(&h, "routes-metadata-contention");
    drop(delivery);
    assert_eq!(local.charged(), 0);
    if cancelled {
        assert!(result.is_err());
        assert_eq!(callbacks, 0);
        assert!(
            collected.is_err() && served.is_err(),
            "cancelled publication disclosed a result"
        );
        assert_eq!(h.bytes(0), before);
        return;
    }
    assert_eq!(
        result,
        Ok(()),
        "short approved-routes metadata contention refused valid publication before callback"
    );
    assert_eq!(callback, Ok(true));
    assert_eq!(callbacks, 1);
    assert_eq!(served, Ok(true));
    assert_eq!(before_collect, 2);
    let Ok(Collection::Complete(response)) = collected else {
        panic!("valid actual publication did not authenticate its stored result")
    };
    assert_eq!(
        response,
        Response::Progress {
            binding: binding.clone(),
            offset: 0,
            committed: false
        }
    );
    let stored = h.bytes(0);
    assert_eq!(&stored[..8], b"FBCS0001");
    let clock = u64::from_be_bytes(stored[8..16].try_into().unwrap());
    assert!(clock >= binding.created && clock < binding.expires);
    let mut expected = b"FBCS0001".to_vec();
    expected.extend_from_slice(&stored[8..16]);
    expected.push(1);
    expected.extend_from_slice(
        &faction_files::private_blobs::encode_request(&Request::Begin(binding)).unwrap()[9..],
    );
    expected.push(0);
    expected.extend_from_slice(&0u32.to_be_bytes());
    assert_eq!(
        stored, expected,
        "actual Begin Store must contain only the exact authenticated Binding and zero ciphertext"
    );
}
#[cfg(feature = "durable-policy")]
#[test]
fn metadata_read_side_routes_contention_keeps_valid_publication() {
    metadata_routes_contention(false);
}
#[cfg(feature = "durable-policy")]
#[test]
fn metadata_read_side_routes_contention_preserves_cancellation_refusal() {
    metadata_routes_contention(true);
}

#[cfg(feature = "operator")]
#[path = "operator/composed_loop_tests.rs"]
mod composed_loop_tests;

#[test]
fn diagnostic_interface_starts_explicitly_unobserved() {
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = carrier_endpoint(local, &local.device, listener.local_addr().unwrap());
    let operation = endpoint.operation(Arc::new(AtomicBool::new(false)));
    assert!(operation.diagnostic_snapshot().is_none());
    assert_eq!(operation.test_attempts(), 0);
}

#[test]
fn diagnostic_actual_submit_exchange_survives_cancellation() {
    use crate::delivery::DiagnosticPhase;
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let binding = h.pair.binding(0);
    let _store = h.empty(0);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = carrier_endpoint(local, &local.device, listener.local_addr().unwrap());
    let cancel = Arc::new(AtomicBool::new(false));
    thread::scope(|scope| {
        let original_cancel = cancel.clone();
        let client = scope.spawn(move || {
            let mut operation = endpoint.operation(original_cancel);
            let result = operation.submit_publication(binding.clone(), Request::Begin(binding));
            (result, operation)
        });
        let mut session = local
            .server(&local.ca.0)
            .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let (entered, waiting) = std::sync::mpsc::channel();
        let (release, resume) = std::sync::mpsc::channel();
        let positive = session.hold_delivery_after_prefix(entered, resume);
        let server = scope.spawn(move || session.submit_publication());
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        cancel.store(true, Ordering::SeqCst);
        let (result, operation) = client.join().unwrap();
        release.send(()).unwrap();
        drop(server.join().unwrap());
        capture_carrier_optional(&h, "diagnostic-submit");
        // Existing partial-frame read classification stays Frame; diagnostic
        // cancellation origin is independent and must not remap that error.
        assert!(cancel.load(Ordering::SeqCst));
        assert_eq!(result, Err(tls::Stage::Frame));
        assert!(positive.load(Ordering::SeqCst) > 0);
        assert_eq!(operation.test_attempts(), 1);
        assert_eq!(h.bytes(0).len(), 17);
        listener.set_nonblocking(true).unwrap();
        assert!(
            matches!(listener.accept(), Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
        let snapshot = operation
            .diagnostic_snapshot()
            .expect("actual cancelled exchange must be retained");
        assert_eq!(snapshot.phase, DiagnosticPhase::SubmitExchange);
        assert_eq!(
            (snapshot.attempted, snapshot.busy, snapshot.connections),
            (1, 0, 1)
        );
        assert!(snapshot.timings.submit_connect_us > 0);
        assert!(snapshot.timings.submit_exchange_us > 0);
        assert_eq!(snapshot.timings.actual_wait_us, 0);
    });
}

#[test]
fn diagnostic_actual_collect_wait_survives_cancellation_without_connection() {
    use crate::delivery::DiagnosticPhase;
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let binding = h.pair.binding(0);
    let _store = h.empty(0);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = carrier_endpoint(local, &local.device, listener.local_addr().unwrap());
    let cancel = Arc::new(AtomicBool::new(false));
    thread::scope(|scope| {
        let original_cancel = cancel.clone();
        let (entered, waiting) = std::sync::mpsc::sync_channel(0);
        let client = scope.spawn(move || {
            let mut operation = endpoint.operation(original_cancel);
            operation.test_observe_collect_wait(entered);
            operation
                .submit_publication(binding.clone(), Request::Begin(binding))
                .unwrap();
            let result = operation.collect_paced();
            (result, operation)
        });
        let mut session = local
            .server(&local.ca.0)
            .accept(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let (prefix, receipt_waiting) = std::sync::mpsc::channel();
        let (release, resume) = std::sync::mpsc::channel();
        let positive = session.hold_delivery_after_prefix(prefix, resume);
        let receipt = scope.spawn(move || {
            receipt_waiting
                .recv_timeout(Duration::from_secs(5))
                .unwrap();
            // Preserve the existing synthetic receipt-RTT setup; entry below,
            // rather than a sleep, is the oracle for the actual pacing phase.
            thread::sleep(Duration::from_millis(100));
            release.send(()).unwrap();
        });
        let (delivery, work) = session.submit_publication().unwrap();
        receipt.join().unwrap();
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        cancel.store(true, Ordering::SeqCst);
        let (result, operation) = client.join().unwrap();
        assert_eq!(result.err(), Some(tls::Stage::Cancelled));
        assert!(positive.load(Ordering::SeqCst) > 0);
        assert_eq!(operation.test_attempts(), 1);
        assert_eq!(delivery.test_original_attempts(), 1);
        assert_eq!(local.charged(), 1);
        assert_eq!(h.bytes(0).len(), 17);
        listener.set_nonblocking(true).unwrap();
        assert!(
            matches!(listener.accept(), Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
        capture_carrier_optional(&h, "diagnostic-wait");
        drop(work);
        drop(delivery);
        assert_eq!(local.charged(), 0);
        let snapshot = operation
            .diagnostic_snapshot()
            .expect("actual cancelled pacing wait must be retained");
        assert_eq!(snapshot.phase, DiagnosticPhase::CollectWait);
        assert_eq!(
            (snapshot.attempted, snapshot.busy, snapshot.connections),
            (1, 0, 1)
        );
        assert!(snapshot.timings.planned_wait_us > 0);
        assert!(snapshot.timings.actual_wait_us > 0);
        assert_eq!(snapshot.timings.collect_connect_us, 0);
        assert_eq!(snapshot.timings.collect_exchange_us, 0);
    });
}

#[test]
fn waiting_resource_collect_returns_the_real_held_introspection_result() {
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let foreign = &h.pair.homes[1];
    let binding = h.pair.binding(0);
    let store = h.empty(0);
    h.publish(0, &binding, &store);
    let record = foreign.record(&binding).unwrap();
    let token = foreign.issue(&record);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = carrier_endpoint(local, &foreign.relay, listener.local_addr().unwrap());
    let (operation, mut delivery, work) = thread::scope(|scope| {
        let endpoint = endpoint.clone();
        let binding = binding.clone();
        let client = scope.spawn(move || {
            let mut operation = endpoint.operation(Arc::new(AtomicBool::new(false)));
            operation
                .submit_retrieval(
                    binding.clone(),
                    Request::GetChunk {
                        id: binding.id,
                        offset: 0,
                    },
                    token,
                )
                .unwrap();
            operation
        });
        let session = local
            .server(&foreign.ca.0)
            .accept_service(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let (delivery, work) = session.submit_retrieval().unwrap();
        (client.join().unwrap(), delivery, work)
    });
    let mut frame = operation.test_collect_frame();
    assert_eq!(frame[6], 1, "legacy ticket changed before opt-in");
    frame[6] = 6;
    let (server_result, raw) = thread::scope(|scope| {
        let endpoint_http = local.endpoint(foreign);
        let store_ref = &store;
        let worker = scope.spawn(move || work.execute(&endpoint_http, store_ref));
        let held = foreign
            .server(&foreign.ca.0)
            .accept_service(
                &foreign.callback,
                foreign.runtime(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        assert!(!delivery.is_finished());
        let client = scope.spawn(|| endpoint.test_exchange(&frame));
        let session = local
            .server(&foreign.ca.0)
            .accept_service(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let collector = scope.spawn(|| session.collect_delivery(&mut delivery));
        // The actual independent introspection response completes the real Work.
        held.serve_http_once().unwrap();
        worker.join().unwrap().unwrap();
        (collector.join().unwrap(), client.join().unwrap())
    });
    assert!(delivery.is_finished());
    capture_carrier_optional(&h, "waiting-resource-real-result");
    drop(delivery);
    assert_eq!(local.charged(), 0);
    drop(record);
    assert!(
        matches!(server_result, Ok(true)),
        "waiting collect failed: {server_result:?}"
    );
    assert!(
        raw.len() >= 11,
        "waiting collect disclosed no framed result"
    );
    let length = u32::from_be_bytes(raw[..4].try_into().unwrap()) as usize;
    assert_eq!(length, raw.len() - 4);
    assert_eq!(&raw[4..11], b"FDEL01\x04");
    match faction_files::private_blobs::decode_response(&raw[11..]).unwrap() {
        Response::Chunk {
            offset,
            bytes,
            total,
        } => {
            assert_eq!(offset, 0);
            assert_eq!(bytes, vec![9; 32]);
            assert_eq!(total, 32);
        }
        _ => panic!("waiting collect substituted the actual committed result"),
    }
}

#[test]
fn waiting_resource_collect_interface_is_explicit_and_refuses_device_purpose() {
    let h = Homes::new(false);
    let local = &h.pair.homes[0];
    let endpoint = carrier_endpoint(local, &local.device, local.callback.local_addr().unwrap());
    let mut operation = endpoint.operation(Arc::new(AtomicBool::new(false)));
    assert_eq!(operation.collect_waiting().err(), Some(tls::Stage::Denied));
    assert_eq!(operation.test_attempts(), 0);
    capture_carrier_optional(&h, "waiting-interface-refused-no-ticket");
}

struct WaitingResource {
    listener: TcpListener,
    endpoint: crate::delivery::CarrierEndpoint,
    operation: crate::delivery::Operation,
    delivery: crate::delivery::Delivery,
    work: crate::delivery::RetrievalWork,
    record: crate::retrieval::RetrievalRecord,
    source_cancel: Arc<AtomicBool>,
}
fn waiting_resource(h: &Homes, i: usize) -> WaitingResource {
    let local = &h.pair.homes[i];
    let foreign = &h.pair.homes[1 - i];
    let binding = h.pair.binding(i);
    let record = foreign.record(&binding).unwrap();
    let token = foreign.issue(&record);
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    retain_synthetic_fixture(
        &format!(
            "waiting-token-{}-{}.bin",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ),
        &token.decode_bytes().unwrap(),
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = carrier_endpoint(local, &foreign.relay, listener.local_addr().unwrap());
    let source_cancel = Arc::new(AtomicBool::new(false));
    let (operation, delivery, work) = thread::scope(|scope| {
        let endpoint = endpoint.clone();
        let client = scope.spawn(move || {
            let mut operation = endpoint.operation(Arc::new(AtomicBool::new(false)));
            operation
                .submit_retrieval(
                    binding.clone(),
                    Request::GetChunk {
                        id: binding.id,
                        offset: 0,
                    },
                    token,
                )
                .unwrap();
            operation
        });
        let session = local
            .server(&foreign.ca.0)
            .accept_service(&listener, local.runtime(), source_cancel.clone())
            .unwrap();
        let (delivery, work) = session.submit_retrieval().unwrap();
        (client.join().unwrap(), delivery, work)
    });
    WaitingResource {
        listener,
        endpoint,
        operation,
        delivery,
        work,
        record,
        source_cancel,
    }
}
fn assert_waiting_chunk(result: crate::delivery::Collection) {
    let crate::delivery::Collection::Complete(Response::Chunk {
        offset,
        bytes,
        total,
    }) = result
    else {
        panic!("missing real resource result")
    };
    assert_eq!(offset, 0);
    assert_eq!(bytes, vec![9; 32]);
    assert_eq!(total, 32);
}

#[test]
fn waiting_resource_collect_cap_spurious_wakes_and_legacy_collect_release_admission() {
    use crate::delivery::Collection;
    use std::sync::mpsc;
    let h = Homes::new(false);
    let store = h.empty(0);
    h.publish(0, &h.pair.binding(0), &store);
    let local = &h.pair.homes[0];
    let foreign = &h.pair.homes[1];
    let WaitingResource {
        listener,
        mut operation,
        mut delivery,
        work,
        record,
        ..
    } = waiting_resource(&h, 0);
    let witness = delivery.test_completion_witness();
    let (entered, waiting) = mpsc::sync_channel(1);
    delivery.test_observe_waiting_collect(entered);
    thread::scope(|scope| {
        let endpoint = local.endpoint(foreign);
        let store_ref = &store;
        let worker = scope.spawn(move || work.execute(&endpoint, store_ref));
        let held = foreign
            .server(&foreign.ca.0)
            .accept_service(
                &foreign.callback,
                foreign.runtime(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        thread::scope(|iteration| {
            let client = iteration.spawn(|| operation.collect_waiting());
            let session = local
                .server(&foreign.ca.0)
                .accept_service(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            let collector = iteration.spawn(|| session.collect_delivery(&mut delivery));
            let (start, end) = waiting.recv_timeout(Duration::from_secs(2)).unwrap();
            assert!(end <= start + Duration::from_millis(100));
            assert!(end > start);
            witness.notify_spurious();
            witness.notify_spurious();
            assert!(!collector.join().unwrap().unwrap());
            assert!(matches!(client.join().unwrap().unwrap(), Collection::Busy));
        });
        assert!(!local.runtime().state.active.load(Ordering::SeqCst));
        assert_eq!(local.charged(), 1);
        assert!(!delivery.is_finished());
        assert_eq!(
            operation.test_collect_frame()[6],
            1,
            "opt-in mutated the stored legacy ticket"
        );
        thread::scope(|iteration| {
            let client = iteration.spawn(|| operation.collect());
            let session = local
                .server(&foreign.ca.0)
                .accept_service(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            assert!(!session.collect_delivery(&mut delivery).unwrap());
            assert!(matches!(client.join().unwrap().unwrap(), Collection::Busy));
        });
        assert!(!local.runtime().state.active.load(Ordering::SeqCst));
        held.serve_http_once().unwrap();
        worker.join().unwrap().unwrap();
        thread::scope(|iteration| {
            let client = iteration.spawn(|| operation.collect_waiting());
            let session = local
                .server(&foreign.ca.0)
                .accept_service(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            assert!(session.collect_delivery(&mut delivery).unwrap());
            assert_waiting_chunk(client.join().unwrap().unwrap());
        });
    });
    assert_eq!(delivery.test_original_attempts(), 5);
    assert_eq!(operation.test_attempts(), 4);
    capture_carrier_optional(&h, "waiting-cap-legacy");
    drop(witness);
    drop(delivery);
    drop(record);
    assert_eq!(local.charged(), 0);
}

#[test]
fn waiting_resource_collect_completion_mutex_collision_keeps_the_actual_result() {
    use std::sync::mpsc;
    let h = Homes::new(false);
    let store = h.empty(0);
    h.publish(0, &h.pair.binding(0), &store);
    let local = &h.pair.homes[0];
    let foreign = &h.pair.homes[1];
    let WaitingResource {
        listener,
        mut operation,
        mut delivery,
        work,
        record,
        ..
    } = waiting_resource(&h, 0);
    let witness = delivery.test_completion_witness();
    let completion = witness.hold();
    let (entered, waiting) = mpsc::sync_channel(1);
    delivery.test_observe_waiting_collect(entered);
    let (finish, finishing) = mpsc::sync_channel(1);
    witness.observe_finish(finish);
    thread::scope(|scope| {
        let endpoint = local.endpoint(foreign);
        let store_ref = &store;
        let worker = scope.spawn(move || work.execute(&endpoint, store_ref));
        let held = foreign
            .server(&foreign.ca.0)
            .accept_service(
                &foreign.callback,
                foreign.runtime(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        let client = scope.spawn(|| operation.collect_waiting());
        let session = local
            .server(&foreign.ca.0)
            .accept_service(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let collector = scope.spawn(|| session.collect_delivery(&mut delivery));
        waiting.recv_timeout(Duration::from_secs(2)).unwrap();
        held.serve_http_once().unwrap();
        finishing.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(
            !worker.is_finished(),
            "actual finish did not contend with the held completion lock"
        );
        drop(completion);
        worker.join().unwrap().unwrap();
        assert!(collector.join().unwrap().unwrap());
        assert_waiting_chunk(client.join().unwrap().unwrap());
    });
    assert!(delivery.is_consumed());
    capture_carrier_optional(&h, "waiting-completion-collision");
    drop(witness);
    drop(delivery);
    drop(record);
    assert_eq!(local.charged(), 0);
}

#[test]
fn waiting_resource_collect_already_finished_is_not_lost_before_wait_registration() {
    use std::sync::mpsc;
    let h = Homes::new(false);
    let store = h.empty(0);
    h.publish(0, &h.pair.binding(0), &store);
    let local = &h.pair.homes[0];
    let foreign = &h.pair.homes[1];
    let WaitingResource {
        listener,
        mut operation,
        mut delivery,
        work,
        record,
        ..
    } = waiting_resource(&h, 0);
    thread::scope(|scope| {
        let callback = scope.spawn(|| {
            foreign
                .server(&foreign.ca.0)
                .accept_service(
                    &foreign.callback,
                    foreign.runtime(),
                    Arc::new(AtomicBool::new(false)),
                )
                .unwrap()
                .serve_http_once()
        });
        work.execute(&local.endpoint(foreign), &store).unwrap();
        callback.join().unwrap().unwrap();
    });
    assert!(delivery.is_finished());
    let (entered, waiting) = mpsc::sync_channel(1);
    delivery.test_observe_waiting_collect(entered);
    thread::scope(|scope| {
        let client = scope.spawn(|| operation.collect_waiting());
        let session = local
            .server(&foreign.ca.0)
            .accept_service(&listener, local.runtime(), Arc::new(AtomicBool::new(false)))
            .unwrap();
        assert!(session.collect_delivery(&mut delivery).unwrap());
        assert_waiting_chunk(client.join().unwrap().unwrap());
    });
    assert!(
        waiting.try_recv().is_err(),
        "finished result entered pending wait"
    );
    capture_carrier_optional(&h, "waiting-already-finished");
    drop(delivery);
    drop(record);
    assert_eq!(local.charged(), 0);
}

#[test]
fn waiting_resource_collect_rechecks_source_session_policy_and_tighter_deadline() {
    use std::sync::mpsc;
    for case in 0..4 {
        let h = Homes::new(false);
        let store = h.empty(0);
        h.publish(0, &h.pair.binding(0), &store);
        let local = &h.pair.homes[0];
        let foreign = &h.pair.homes[1];
        let WaitingResource {
            listener,
            endpoint,
            operation,
            mut delivery,
            work,
            record,
            source_cancel,
        } = waiting_resource(&h, 0);
        let mut frame = operation.test_collect_frame();
        frame[6] = 6;
        let (entered, waiting) = mpsc::sync_channel(1);
        delivery.test_observe_waiting_collect(entered);
        let incoming_cancel = Arc::new(AtomicBool::new(false));
        thread::scope(|scope| {
            let endpoint_http = local.endpoint(foreign);
            let store_ref = &store;
            let worker = scope.spawn(move || work.execute(&endpoint_http, store_ref));
            let held = foreign
                .server(&foreign.ca.0)
                .accept_service(
                    &foreign.callback,
                    foreign.runtime(),
                    Arc::new(AtomicBool::new(false)),
                )
                .unwrap();
            let client = scope.spawn(|| endpoint.test_exchange(&frame));
            let session = local
                .server(&foreign.ca.0)
                .accept_service(&listener, local.runtime(), incoming_cancel.clone())
                .unwrap();
            // Narrow only after both actual TLS fixtures are admitted; the
            // original source clock is never extended by test setup.
            let tight = Instant::now() + Duration::from_millis(80);
            if case == 3 {
                *delivery.hold_deadline_metadata() = tight;
            }
            let collector = scope.spawn(|| session.collect_delivery(&mut delivery));
            let (start, end) = waiting.recv_timeout(Duration::from_secs(2)).unwrap();
            assert!(end <= start + Duration::from_millis(100));
            match case {
                0 => source_cancel.store(true, Ordering::SeqCst),
                1 => incoming_cancel.store(true, Ordering::SeqCst),
                2 => h.install(0, 3, 0, 3, 0, |_| ()),
                3 => assert!(end <= tight),
                _ => unreachable!(),
            }
            assert!(
                collector.join().unwrap().is_err(),
                "invalidation case {case} returned Busy/result"
            );
            assert!(
                client.join().unwrap().is_empty(),
                "invalidation disclosed application bytes"
            );
            assert!(!local.runtime().state.active.load(Ordering::SeqCst));
            drop(held);
            assert!(worker.join().unwrap().is_err());
        });
        capture_carrier_optional(&h, "waiting-invalidation");
        drop(delivery);
        drop(record);
        assert_eq!(local.charged(), 0);
    }
}

#[test]
fn waiting_finished_slot_contention_legacy_collect_retains_the_actual_result() {
    finished_slot_contention(false);
}
#[test]
fn waiting_finished_slot_contention_opt_in_collect_retains_the_actual_result() {
    finished_slot_contention(true);
}
fn finished_slot_contention(waiting: bool) {
    use std::sync::mpsc;
    let h=Homes::new(false);let store=h.empty(0);h.publish(0,&h.pair.binding(0),&store);
    let local=&h.pair.homes[0];let foreign=&h.pair.homes[1];
    let WaitingResource{listener,mut operation,mut delivery,work,record,..}=waiting_resource(&h,0);
    thread::scope(|scope| {
        let callback=scope.spawn(||foreign.server(&foreign.ca.0).accept_service(&foreign.callback,foreign.runtime(),Arc::new(AtomicBool::new(false))).unwrap().serve_http_once());
        work.execute(&local.endpoint(foreign),&store).unwrap();callback.join().unwrap().unwrap();
    });
    assert!(delivery.is_finished());
    let witness=delivery.test_completion_witness();let completion=witness.hold();
    let (entered,entry)=mpsc::sync_channel(1);let(returned,returning)=mpsc::sync_channel(1);
    let (early,server_result,client_result)=thread::scope(|scope| {
        let client=scope.spawn(||if waiting {operation.collect_waiting()} else {operation.collect()});
        let session=local.server(&foreign.ca.0).accept_service(&listener,local.runtime(),Arc::new(AtomicBool::new(false))).unwrap();
        let collector=scope.spawn(|| {
            entered.send(()).unwrap();
            let result=session.collect_delivery(&mut delivery);
            returned.send(result).unwrap();
        });
        entry.recv_timeout(Duration::from_secs(2)).unwrap();
        let early=returning.recv_timeout(Duration::from_millis(50));
        // Always release and drain before judging either old or new behavior.
        drop(completion);collector.join().unwrap();
        let server_result=match early { Ok(result)=>result,Err(mpsc::RecvTimeoutError::Timeout)=>returning.recv_timeout(Duration::from_secs(2)).unwrap(),Err(other)=>panic!("collector disappeared: {other:?}") };
        (early,server_result,client.join().unwrap())
    });
    capture_carrier_optional(&h,"waiting-finished-slot-contention");
    drop(witness);drop(delivery);drop(record);assert_eq!(local.charged(),0);
    assert!(matches!(early,Err(mpsc::RecvTimeoutError::Timeout)),"finished collect returned before its actual result-slot lock was released: {early:?}");
    assert!(matches!(server_result,Ok(true)),"finished collect lost its actual response: {server_result:?}");
    assert_waiting_chunk(client_result.unwrap());
}
