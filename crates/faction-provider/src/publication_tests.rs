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
struct Homes {
    pair: Pair,
    observers: [Certificate; 2],
    callbacks: [AtomicUsize; 2],
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
        };
        for i in 0..2 {
            h.install(i, 2, 0, 2, 0, |_| ());
        }
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
