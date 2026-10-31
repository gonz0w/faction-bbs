//! Actual two-home TLS on one composed Runtime per home. Request BIO holds are
//! deterministic scheduling controls, not kernel backpressure measurements.
use super::*;
use crate::{
    Runtime, Session,
    authority::{CredentialBinding, Registry},
    retrieval::{RetrievalAction, RetrievalPolicy, RetrievalRecord},
    tls::{Client, ClientCredentials, HttpEndpoint, Server, test_certificate as cert},
};
use faction_delegation_codec as codec;
use faction_files::private_blobs::{Access, Action, Request, Response, Store};
use openssl::pkey::{PKey, Private};
use std::{
    fs,
    io::Write,
    net::TcpListener,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::PathBuf,
    sync::{Mutex, atomic::AtomicBool, mpsc},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
type Certificate = (X509, PKey<Private>);
struct Home {
    name: &'static str,
    ca: Certificate,
    server: Certificate,
    device: Certificate,
    relay: Certificate,
    outgoing: Certificate,
    runtime: Option<Runtime>,
    callback: TcpListener,
    principal: Principal,
}
impl Home {
    fn runtime(&self) -> &Runtime {
        self.runtime.as_ref().unwrap()
    }
    fn registry(&self, generation: u64, reset: u64, foreign: &Home) -> Registry {
        Registry::new(
            generation,
            reset,
            vec![self.name.into(), foreign.name.into()],
            vec![CredentialBinding {
                certificate_sha256: sha256(&self.device.0.to_der().unwrap()),
                principal: self.principal.clone(),
                valid_from: tls::wall_now() - 60,
                valid_until: tls::wall_now() + 3600,
                approved: true,
                account_enabled: true,
                device_enabled: true,
                generation,
            }],
        )
        .unwrap()
    }
    fn policy(&self, foreign: &Home, generation: u64, reset: u64) -> ServicePolicy {
        let entry = |c: &X509, h: &str, id: &str| ServiceRegistration {
            home: h.into(),
            client_id: id.into(),
            dns_san: "provider.test".into(),
            certificate_sha256: sha256(&c.to_der().unwrap()),
            enabled: true,
            valid_from: tls::wall_now() - 60,
            valid_until: tls::wall_now() + 3600,
        };
        ServicePolicy::new_composed(
            self.name.into(),
            self.name.into(),
            vec![
                (entry(&self.relay.0, self.name, "relay"), ServiceUse::Relay),
                (
                    entry(&foreign.outgoing.0, foreign.name, "inspector"),
                    ServiceUse::Introspection,
                ),
                (
                    entry(&foreign.relay.0, foreign.name, "relay"),
                    ServiceUse::ResourceClient,
                ),
                (
                    entry(&self.outgoing.0, self.name, "inspector"),
                    ServiceUse::OutboundIntrospection,
                ),
                (
                    entry(&foreign.server.0, foreign.name, "endpoint"),
                    ServiceUse::IssuerEndpoint,
                ),
            ],
            vec![IssuerRoute {
                sender_home: foreign.name.into(),
                audience: format!("{}/attachments", foreign.name),
                relay_client_id: "relay".into(),
                introspection_client_id: "inspector".into(),
            }],
            vec![ResourceAssociation {
                recipient_home: foreign.name.into(),
                issuer: foreign.name.into(),
                audience: format!("{}/attachments", self.name),
                relay_client_id: "relay".into(),
                outbound_client_id: "inspector".into(),
                endpoint_id: "endpoint".into(),
                address: foreign.callback.local_addr().unwrap(),
                http_host: foreign.name.into(),
                ca_sha256: sha256(&foreign.ca.0.to_der().unwrap()),
                remote_generation: generation,
                remote_reset: reset,
            }],
        )
        .unwrap()
    }
    fn install(
        &self,
        foreign: &Home,
        generation: u64,
        reset: u64,
        remote_generation: u64,
        remote_reset: u64,
    ) {
        self.runtime()
            .control()
            .replace_delegation_policy(
                self.registry(generation, reset, foreign),
                self.policy(foreign, remote_generation, remote_reset),
            )
            .unwrap();
        self.runtime()
            .control()
            .replace_retrieval_policy(
                RetrievalPolicy::new(
                    self.name.into(),
                    vec![(foreign.name.into(), format!("{}/attachments", foreign.name))],
                )
                .unwrap(),
            )
            .unwrap();
    }
    fn server(&self, client_ca: &X509) -> Server {
        Server::new(&self.server.0, &self.server.1, client_ca).unwrap()
    }
    fn client(&self, address: std::net::SocketAddr, who: &Certificate) -> Client {
        Client::connect(
            address,
            "provider.test",
            sha256(&self.server.0.to_der().unwrap()),
            &self.ca.0,
            Some((&who.0, Some(&who.1))),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
    }
    fn device_call<T: Send>(&self, f: impl FnOnce(Session) -> T) -> T {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::scope(|s| {
            let peer = s.spawn(|| self.client(address, &self.device));
            let session = self
                .server(&self.ca.0)
                .accept(&listener, self.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            let _peer = peer.join().unwrap();
            f(session)
        })
    }
    fn service_call<T: Send>(
        &self,
        who: &Certificate,
        ca: &X509,
        f: impl FnOnce(tls::ServiceSession) -> T,
    ) -> T {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::scope(|s| {
            let peer = s.spawn(|| self.client(address, who));
            let session = self
                .server(ca)
                .accept_service(&listener, self.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            let _peer = peer.join().unwrap();
            f(session)
        })
    }
    fn record(&self, binding: &Binding) -> Result<RetrievalRecord, Error> {
        self.device_call(|mut s| {
            s.retrieval_intent(binding.clone(), RetrievalAction::GetChunk { offset: 0 })
        })
    }
    fn issue(&self, record: &RetrievalRecord) -> codec::UnverifiedToken {
        let reference = self
            .runtime()
            .control()
            .provision_retrieval(record)
            .unwrap();
        self.service_call(&self.relay, &self.ca.0, |mut s| {
            codec::UnverifiedToken::from_bytes(
                *s.issue_retrieval(&reference, "relay")
                    .unwrap()
                    .token()
                    .expose_bytes(),
            )
        })
    }
    fn token(&self, binding: &Binding) -> codec::UnverifiedToken {
        self.issue(&self.record(binding).unwrap())
    }
    fn prepare(
        &self,
        foreign: &Home,
        binding: &Binding,
        token: codec::UnverifiedToken,
        offset: u64,
    ) -> Result<PendingRetrieval, Error> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::scope(|s| {
            let peer = s.spawn(|| self.client(address, &foreign.relay));
            let session = self
                .server(&foreign.ca.0)
                .accept_service(&listener, self.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap();
            let pending = session.prepare_retrieval(
                binding.clone(),
                Request::GetChunk {
                    id: binding.id,
                    offset,
                },
                token,
            );
            let mut peer = peer.join().unwrap();
            assert!(
                peer.request(&Request::Status { id: binding.id }).is_err(),
                "consumed incoming carrier survived"
            );
            pending
        })
    }
    fn endpoint(&self, foreign: &Home) -> HttpEndpoint {
        HttpEndpoint::new(
            foreign.callback.local_addr().unwrap(),
            foreign.name.into(),
            "provider.test".into(),
            sha256(&foreign.server.0.to_der().unwrap()),
            ClientCredentials::from_pem(
                &foreign.ca.0.to_pem().unwrap(),
                &self.outgoing.0.to_pem().unwrap(),
                &self.outgoing.1.private_key_to_pem_pkcs8().unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn complete(
        &self,
        foreign: &Home,
        pending: PendingRetrieval,
        store: &Mutex<Store>,
    ) -> Result<Response, Error> {
        thread::scope(|s| {
            let issuer = s.spawn(|| {
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
            let result = pending.execute(&self.endpoint(foreign), store);
            issuer.join().unwrap().unwrap();
            result
        })
    }
    fn charged(&self) -> usize {
        self.runtime()
            .state
            .authority
            .lock()
            .unwrap()
            .records
            .charged_len()
    }
    fn ordinary_progress(&self, store: &Mutex<Store>, binding: &Binding) {
        self.device_call(|mut session| {
            let mut b = binding.clone();
            b.recipient = b.sender.clone();
            b.id = sha256(&[7; 32]);
            let mut store = store
                .try_lock()
                .expect("network callback retained Store lock");
            for request in [
                Request::Begin(b.clone()),
                Request::PutChunk {
                    id: b.id,
                    offset: 0,
                    bytes: vec![7; 32],
                },
                Request::Commit { id: b.id },
                Request::GetChunk {
                    id: b.id,
                    offset: 0,
                },
            ] {
                assert!(
                    session
                        .dispatch(&mut store, &b.sender.account, request)
                        .is_ok()
                );
            }
        })
    }
}
struct Pair {
    homes: [Home; 2],
    root: PathBuf,
}
impl Pair {
    fn new(durable: bool) -> Self {
        let t = tls::wall_now() as i64;
        let ca0 = cert(None, "CA-a", 701, t - 60, t + 3600);
        let ca1 = cert(None, "CA-b", 702, t - 60, t + 3600);
        let leaf =
            |ca: &Certificate, id, name| cert(Some((&ca.0, &ca.1)), name, id, t - 60, t + 3600);
        let a = Home {
            name: "a.test",
            server: leaf(&ca0, 703, "a-server"),
            device: leaf(&ca0, 704, "a-device"),
            relay: leaf(&ca0, 705, "a-relay"),
            outgoing: leaf(&ca1, 706, "a-outgoing"),
            ca: ca0,
            runtime: None,
            callback: TcpListener::bind("127.0.0.1:0").unwrap(),
            principal: Principal {
                account: "alice@a.test".into(),
                device: 1,
                identity: [1; 32],
            },
        };
        let b = Home {
            name: "b.test",
            server: leaf(&ca1, 707, "b-server"),
            device: leaf(&ca1, 708, "b-device"),
            relay: leaf(&ca1, 709, "b-relay"),
            outgoing: leaf(&a.ca, 710, "b-outgoing"),
            ca: ca1,
            runtime: None,
            callback: TcpListener::bind("127.0.0.1:0").unwrap(),
            principal: Principal {
                account: "bob@b.test".into(),
                device: 2,
                identity: [2; 32],
            },
        };
        let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "f317-composed-OWNED-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join("OWNED"))
            .unwrap()
            .write_all(b"Faction317 private synthetic composed TLS fixture")
            .unwrap();
        let mut p = Self {
            homes: [a, b],
            root,
        };
        for i in 0..2 {
            for kind in ["store", "policy"] {
                fs::DirBuilder::new()
                    .mode(0o700)
                    .create(p.root.join(format!("{kind}{i}")))
                    .unwrap();
            }
            #[cfg(feature = "durable-policy")]
            let runtime = if durable {
                Runtime::initialize_durable(
                    &p.root.join(format!("policy{i}")),
                    p.homes[i].registry(1, 0, &p.homes[1 - i]),
                )
                .unwrap()
            } else {
                Runtime::new(None)
            };
            #[cfg(not(feature = "durable-policy"))]
            let runtime = {
                assert!(!durable);
                Runtime::new(None)
            };
            p.homes[i].runtime = Some(runtime);
        }
        p.homes[0].install(&p.homes[1], 2, 0, 2, 0);
        p.homes[1].install(&p.homes[0], 2, 0, 2, 0);
        p
    }
    fn binding(&self, i: usize) -> Binding {
        Binding {
            sender: self.homes[i].principal.clone(),
            recipient: self.homes[1 - i].principal.clone(),
            id: sha256(&[9; 32]),
            size: 32,
            created: tls::wall_now() - 5,
            expires: tls::wall_now() + 300,
        }
    }
    fn preexisting(&self, i: usize, binding: &Binding) -> Mutex<Store> {
        struct Setup<'a>(&'a Binding);
        impl Access for Setup<'_> {
            fn require(
                &self,
                actor: &Principal,
                b: &Binding,
                action: Action,
                _now: u64,
            ) -> Result<(), BlobError> {
                if actor == &self.0.sender
                    && b == self.0
                    && matches!(action, Action::Begin | Action::PutChunk | Action::Commit)
                {
                    Ok(())
                } else {
                    Err(BlobError::Denied)
                }
            }
        }
        let path = self.root.join(format!("store{i}"));
        let mut store = Store::open(&path).unwrap();
        for request in [
            Request::Begin(binding.clone()),
            Request::PutChunk {
                id: binding.id,
                offset: 0,
                bytes: vec![9; 32],
            },
            Request::Commit { id: binding.id },
        ] {
            store
                .dispatch(&binding.sender, &Setup(binding), request, tls::wall_now())
                .unwrap();
        }
        drop(store);
        Mutex::new(Store::open(&path).unwrap())
    }
    #[cfg(feature = "durable-policy")]
    fn reopen(&mut self) {
        for i in 0..2 {
            drop(self.homes[i].runtime.take());
            self.homes[i].runtime =
                Some(Runtime::reopen_durable(&self.root.join(format!("policy{i}"))).unwrap());
            self.homes[i]
                .runtime()
                .control()
                .replace_retrieval_policy(
                    RetrievalPolicy::new(
                        self.homes[i].name.into(),
                        vec![(
                            self.homes[1 - i].name.into(),
                            format!("{}/attachments", self.homes[1 - i].name),
                        )],
                    )
                    .unwrap(),
                )
                .unwrap();
        }
    }
}
impl Drop for Pair {
    fn drop(&mut self) {
        for h in &mut self.homes {
            drop(h.runtime.take());
        }
        assert_eq!(
            fs::read(self.root.join("OWNED")).unwrap(),
            b"Faction317 private synthetic composed TLS fixture"
        );
        fs::remove_dir_all(&self.root).unwrap();
        assert!(!self.root.exists());
    }
}
fn assert_chunk(response: Result<Response, Error>, offset: u64) {
    assert!(
        matches!(response,Ok(Response::Chunk{offset:o,bytes,total:32}) if o==offset&&bytes==if offset==0{vec![9;32]}else{vec![]})
    );
}
#[test]
fn one_installed_composed_policy_per_home_retrieves_both_directions_without_switching() {
    let p = Pair::new(false);
    for i in 0..2 {
        let local = &p.homes[i];
        let foreign = &p.homes[1 - i];
        let binding = p.binding(i);
        let store = p.preexisting(i, &binding);
        for offset in [0, 32] {
            let record = foreign.device_call(|mut s| {
                s.retrieval_intent(binding.clone(), RetrievalAction::GetChunk { offset })
                    .unwrap()
            });
            let pending = local
                .prepare(foreign, &binding, foreign.issue(&record), offset)
                .unwrap();
            assert_chunk(local.complete(foreign, pending, &store), offset);
        }
    }
}
#[cfg(feature = "durable-policy")]
#[test]
fn composed_durable_reopen_requires_fresh_sessions_and_recovers_both_directions() {
    let mut p = Pair::new(true);
    let bindings = [p.binding(0), p.binding(1)];
    let stores = [
        p.preexisting(0, &bindings[0]),
        p.preexisting(1, &bindings[1]),
    ];
    let old = p.homes[0]
        .prepare(&p.homes[1], &bindings[0], p.homes[1].token(&bindings[0]), 0)
        .unwrap();
    p.reopen();
    assert!(
        old.execute(&p.homes[0].endpoint(&p.homes[1]), &stores[0])
            .is_err()
    );
    for i in 0..2 {
        let local = &p.homes[i];
        let foreign = &p.homes[1 - i];
        let pending = local
            .prepare(foreign, &bindings[i], foreign.token(&bindings[i]), 0)
            .unwrap();
        assert_chunk(local.complete(foreign, pending, &stores[i]), 0);
    }
}
#[test]
fn coordinated_complete_proposals_name_both_final_generation_reset_tuples() {
    let p = Pair::new(false);
    let bindings = [p.binding(0), p.binding(1)];
    let stores = [
        p.preexisting(0, &bindings[0]),
        p.preexisting(1, &bindings[1]),
    ];
    let a = &p.homes[0];
    let b = &p.homes[1];
    a.runtime().control().reset(1).unwrap();
    a.install(b, 4, 1, 4, 1);
    let intermediate = a
        .prepare(b, &bindings[0], b.token(&bindings[0]), 0)
        .unwrap();
    assert!(a.complete(b, intermediate, &stores[0]).is_err());
    b.runtime().control().reset(1).unwrap();
    b.install(a, 4, 1, 4, 1);
    for i in 0..2 {
        let local = &p.homes[i];
        let foreign = &p.homes[1 - i];
        let pending = local
            .prepare(foreign, &bindings[i], foreign.token(&bindings[i]), 0)
            .unwrap();
        assert_chunk(local.complete(foreign, pending, &stores[i]), 0);
        let mut stale = local.policy(foreign, 4, 1);
        stale.resource.as_mut().unwrap()[0].remote_generation = 3;
        assert!(
            local
                .runtime()
                .control()
                .replace_delegation_policy(local.registry(5, 1, foreign), stale)
                .is_err()
        );
        let mut stale = local.policy(foreign, 4, 0);
        assert!(
            local
                .runtime()
                .control()
                .replace_delegation_policy(local.registry(5, 1, foreign), stale.clone())
                .is_err()
        );
        stale.resource.as_mut().unwrap()[0].remote_reset = 1;
        assert!(
            local
                .runtime()
                .control()
                .replace_delegation_policy(local.registry(5, 0, foreign), stale)
                .is_err()
        );
    }
}
#[test]
fn held_resource_callback_allows_same_runtime_issuer_introspection_and_ordinary_store_progress() {
    let p = Pair::new(false);
    let a = &p.homes[0];
    let b = &p.homes[1];
    let ab = p.binding(0);
    let ba = p.binding(1);
    let store = p.preexisting(0, &ab);
    let pending = a.prepare(b, &ab, b.token(&ab), 0).unwrap();
    let mut endpoint = a.endpoint(b);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    endpoint.hold_request(entered_tx, release_rx);
    thread::scope(|s| {
        let remote = s.spawn(|| {
            b.server(&b.ca.0)
                .accept_service(&b.callback, b.runtime(), Arc::new(AtomicBool::new(false)))
                .unwrap()
                .serve_http_once()
        });
        let operation = s.spawn(|| pending.execute(&endpoint, &store));
        entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(!operation.is_finished());
        a.ordinary_progress(&store, &ab);
        let record = a.record(&ba).unwrap();
        let reference = a.runtime().control().provision_retrieval(&record).unwrap();
        let token = a.service_call(&a.relay, &a.ca.0, |mut session| {
            session.issue_retrieval(&reference, "relay").unwrap()
        });
        a.service_call(&b.outgoing, &a.ca.0, |mut session| {
            assert!(
                session
                    .introspect_retrieval(token.token(), "inspector")
                    .unwrap()
                    .is_some()
            );
        });
        assert_eq!(a.charged(), 2);
        release_tx.send(()).unwrap();
        assert_chunk(operation.join().unwrap(), 0);
        remote.join().unwrap().unwrap();
    });
}
#[test]
fn composed_records_tokens_and_resource_continuation_share_eighth_ninth_slots_and_cleanup() {
    let p = Pair::new(false);
    let a = &p.homes[0];
    let b = &p.homes[1];
    let ab = p.binding(0);
    let ba = p.binding(1);
    let pending = a.prepare(b, &ab, b.token(&ab), 0).unwrap();
    let mut records = Vec::new();
    for i in 0..7 {
        let record = a.record(&ba).unwrap();
        if i % 2 == 0 {
            let _token = a.issue(&record);
        }
        records.push(record);
    }
    assert_eq!(a.charged(), 8);
    assert!(matches!(a.record(&ba), Err(Error::Store(BlobError::Busy))));
    drop(pending);
    assert_eq!(a.charged(), 7);
    records.push(a.record(&ba).unwrap());
    assert_eq!(a.charged(), 8);
    assert!(matches!(
        a.prepare(b, &ab, codec::UnverifiedToken::from_bytes([42; 32]), 0),
        Err(Error::Store(BlobError::Busy))
    ));
    a.runtime().control().reset(1).unwrap();
    assert_eq!(a.charged(), 0);
    a.install(b, 4, 1, 2, 0);
    let current = a
        .prepare(b, &ab, codec::UnverifiedToken::from_bytes([42; 32]), 0)
        .unwrap();
    drop(records);
    assert!(matches!(
        a.prepare(b, &ab, codec::UnverifiedToken::from_bytes([42; 32]), 0),
        Err(Error::Store(BlobError::Busy))
    ));
    drop(current);
    assert_eq!(a.charged(), 0);
    assert!(
        a.prepare(b, &ab, codec::UnverifiedToken::from_bytes([42; 32]), 0)
            .is_ok()
    );
}

#[test]
fn mixed_role_reset_and_revocation_during_callback_retain_the_physical_slot_until_socket_finishes()
{
    for reset in [false, true] {
        let p = Pair::new(false);
        let a = &p.homes[0];
        let b = &p.homes[1];
        let ab = p.binding(0);
        let ba = p.binding(1);
        let store = p.preexisting(0, &ab);
        let pending = a.prepare(b, &ab, b.token(&ab), 0).unwrap();
        let records: Vec<_> = (0..7)
            .map(|i| {
                let record = a.record(&ba).unwrap();
                if i % 2 == 0 {
                    let _ = a.issue(&record);
                }
                record
            })
            .collect();
        assert_eq!(a.charged(), 8);
        let mut endpoint = a.endpoint(b);
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        endpoint.hold_request(entered_tx, release_rx);
        thread::scope(|s| {
            let remote = s.spawn(|| {
                let _ = b
                    .server(&b.ca.0)
                    .accept_service(&b.callback, b.runtime(), Arc::new(AtomicBool::new(false)))
                    .unwrap()
                    .serve_http_once();
            });
            let operation = s.spawn(|| pending.execute(&endpoint, &store));
            entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            if reset {
                a.runtime().control().reset(1).unwrap();
                a.install(b, 4, 1, 2, 0);
            } else {
                let mut entries = a.registry(3, 0, b).bindings().to_vec();
                entries[0].approved = false;
                let registry =
                    Registry::new(3, 0, vec![a.name.into(), b.name.into()], entries).unwrap();
                a.runtime()
                    .control()
                    .replace_delegation_policy(registry, a.policy(b, 2, 0))
                    .unwrap();
            }
            assert_eq!(a.charged(), 1);
            drop(records);
            assert_eq!(a.charged(), 1);
            if reset {
                assert!(matches!(
                    a.prepare(b, &ab, codec::UnverifiedToken::from_bytes([42; 32]), 0),
                    Err(Error::Store(BlobError::Busy))
                ));
            }
            assert!(!operation.is_finished());
            release_tx.send(()).unwrap();
            assert!(operation.join().unwrap().is_err());
            remote.join().unwrap();
            assert_eq!(a.charged(), 0);
            if !reset {
                a.install(b, 4, 0, 2, 0);
            }
            assert!(
                a.prepare(b, &ab, codec::UnverifiedToken::from_bytes([42; 32]), 0)
                    .is_ok()
            );
        });
    }
}
#[test]
fn expired_mixed_issuer_records_leave_a_live_physical_reservation_charged() {
    let p = Pair::new(false);
    let a = &p.homes[0];
    let b = &p.homes[1];
    let ab = p.binding(0);
    let mut ba = p.binding(1);
    ba.expires = tls::wall_now() + 3;
    let pending = a.prepare(b, &ab, b.token(&ab), 0).unwrap();
    let records: Vec<_> = (0..7)
        .map(|i| {
            let r = a.record(&ba).unwrap();
            if i % 2 == 0 {
                let _ = a.issue(&r);
            }
            r
        })
        .collect();
    assert_eq!(a.charged(), 8);
    while tls::wall_now() < ba.expires {
        thread::sleep(Duration::from_millis(10));
    }
    let fresh = p.binding(1);
    let fresh_record = a.record(&fresh).unwrap();
    assert_eq!(a.charged(), 2);
    drop(records);
    assert_eq!(a.charged(), 2);
    assert!(matches!(
        a.prepare(b, &ab, codec::UnverifiedToken::from_bytes([42; 32]), 0),
        Err(Error::Store(BlobError::Busy))
    ));
    drop(pending);
    assert_eq!(a.charged(), 1);
    let pending = a
        .prepare(b, &ab, codec::UnverifiedToken::from_bytes([42; 32]), 0)
        .unwrap();
    drop(fresh_record);
    assert_eq!(a.charged(), 2);
    drop(pending);
    assert_eq!(a.charged(), 1);
}
