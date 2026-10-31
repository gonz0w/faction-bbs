//! Given preexisting committed ciphertext only; no foreign publication authority.
use super::*;
use faction_delegation_codec as codec;
use faction_provider::{
    retrieval::{RetrievalAction, RetrievalPolicy},
    services::{IssuerRoute, ResourceAssociation, ServicePolicy, ServiceRegistration, ServiceUse},
    tls::{ClientCredentials, HttpEndpoint},
};
struct Homes {
    a: Certs,
    b: Certs,
    inspector: (X509, PKey<Private>),
    ra: Runtime,
    rb: Runtime,
    callback: TcpListener,
    binding: Binding,
}
fn registration(leaf: &X509, home: &str, id: &str) -> ServiceRegistration {
    ServiceRegistration {
        home: home.into(),
        client_id: id.into(),
        dns_san: "provider.test".into(),
        certificate_sha256: sha256(&leaf.to_der().unwrap()),
        enabled: true,
        valid_from: wall_now() - 60,
        valid_until: wall_now() + 3600,
    }
}
fn sender() -> Principal {
    Principal {
        account: "alice@sender.test".into(),
        device: 1,
        identity: [1; 32],
    }
}
fn recipient() -> Principal {
    Principal {
        account: "bob@recipient.test".into(),
        device: 2,
        identity: [2; 32],
    }
}
fn object() -> Binding {
    let mut b = binding();
    b.sender = sender();
    b.recipient = recipient();
    b
}
impl Homes {
    fn new() -> Self {
        let a = Certs::new();
        let b = Certs::new();
        let t = wall_now() as i64;
        let inspector = cert(Some((&b.ca.0, &b.ca.1)), "inspector", 401, t - 60, t + 3600);
        let callback = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut ae = a.entries();
        ae.truncate(1);
        ae[0].principal = sender();
        ae[0].generation = 2;
        let mut be = b.entries();
        be.remove(0);
        be[0].principal = recipient();
        be[0].generation = 2;
        let ra = Runtime::new(None);
        let rb = Runtime::new(None);
        rb.control()
            .replace_delegation_policy(
                Registry::new(
                    2,
                    0,
                    vec!["recipient.test".into(), "sender.test".into()],
                    be,
                )
                .unwrap(),
                ServicePolicy::new_issuer(
                    "recipient.test".into(),
                    "issuer.test".into(),
                    vec![
                        (
                            registration(&b.alice.0, "recipient.test", "relay"),
                            ServiceUse::Relay,
                        ),
                        (
                            registration(&inspector.0, "sender.test", "inspector"),
                            ServiceUse::Introspection,
                        ),
                    ],
                    vec![IssuerRoute {
                        sender_home: "sender.test".into(),
                        audience: "sender.test/attachments".into(),
                        relay_client_id: "relay".into(),
                        introspection_client_id: "inspector".into(),
                    }],
                )
                .unwrap(),
            )
            .unwrap();
        rb.control()
            .replace_retrieval_policy(
                RetrievalPolicy::new(
                    "recipient.test".into(),
                    vec![("sender.test".into(), "sender.test/attachments".into())],
                )
                .unwrap(),
            )
            .unwrap();
        let association = ResourceAssociation {
            recipient_home: "recipient.test".into(),
            issuer: "issuer.test".into(),
            audience: "sender.test/attachments".into(),
            relay_client_id: "relay".into(),
            outbound_client_id: "inspector".into(),
            endpoint_id: "endpoint".into(),
            address: callback.local_addr().unwrap(),
            http_host: "issuer.test".into(),
            ca_sha256: sha256(&b.ca.0.to_der().unwrap()),
            remote_generation: 2,
            remote_reset: 0,
        };
        ra.control()
            .replace_delegation_policy(
                Registry::new(
                    2,
                    0,
                    vec!["sender.test".into(), "recipient.test".into()],
                    ae,
                )
                .unwrap(),
                ServicePolicy::new_resource(
                    "sender.test".into(),
                    vec![
                        (
                            registration(&b.alice.0, "recipient.test", "relay"),
                            ServiceUse::ResourceClient,
                        ),
                        (
                            registration(&inspector.0, "sender.test", "inspector"),
                            ServiceUse::OutboundIntrospection,
                        ),
                        (
                            registration(&b.server.0, "recipient.test", "endpoint"),
                            ServiceUse::IssuerEndpoint,
                        ),
                    ],
                    vec![association],
                )
                .unwrap(),
            )
            .unwrap();
        let mut binding = object();
        binding.created = binding.created.saturating_sub(5);
        Self {
            a,
            b,
            inspector,
            ra,
            rb,
            callback,
            binding,
        }
    }
    fn endpoint(&self) -> HttpEndpoint {
        let credentials = ClientCredentials::from_pem(
            &self.b.ca.0.to_pem().unwrap(),
            &self.inspector.0.to_pem().unwrap(),
            &self.inspector.1.private_key_to_pem_pkcs8().unwrap(),
        )
        .unwrap();
        HttpEndpoint::new(
            self.callback.local_addr().unwrap(),
            "issuer.test".into(),
            "provider.test".into(),
            sha256(&self.b.server.0.to_der().unwrap()),
            credentials,
        )
        .unwrap()
    }
    fn token_with_expiry(&self, action: RetrievalAction) -> (codec::UnverifiedToken, u64) {
        let record = with_session(&self.b, &self.rb, &self.b.bob, |mut s| {
            s.retrieval_intent(self.binding.clone(), action).unwrap()
        });
        let reference = self.rb.control().provision_retrieval(&record).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::scope(|scope| {
            let peer = scope.spawn(|| {
                self.b
                    .client(
                        address,
                        Arc::new(AtomicBool::new(false)),
                        Some(&self.b.alice),
                    )
                    .unwrap()
            });
            let mut session = self
                .b
                .server()
                .accept_service(&listener, &self.rb, Arc::new(AtomicBool::new(false)))
                .unwrap();
            let _peer = peer.join().unwrap();
            {
                let issued = session.issue_retrieval(&reference, "relay").unwrap();
                (
                    codec::UnverifiedToken::from_bytes(*issued.token().expose_bytes()),
                    issued.expires_at(),
                )
            }
        })
    }
    fn token(&self, action: RetrievalAction) -> codec::UnverifiedToken {
        self.token_with_expiry(action).0
    }
    fn prepare_result(
        &self,
        request: Request,
        token: codec::UnverifiedToken,
    ) -> Result<faction_provider::services::PendingRetrieval, Error> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::scope(|scope| {
            let peer = scope.spawn(|| {
                Client::connect(
                    address,
                    "provider.test",
                    sha256(&self.a.server.0.to_der().unwrap()),
                    &self.a.ca.0,
                    Some((&self.b.alice.0, Some(&self.b.alice.1))),
                    Arc::new(AtomicBool::new(false)),
                )
                .unwrap()
            });
            let session = Server::new(&self.a.server.0, &self.a.server.1, &self.b.ca.0)
                .unwrap()
                .accept_service(&listener, &self.ra, Arc::new(AtomicBool::new(false)))
                .unwrap();
            let pending = session.prepare_retrieval(self.binding.clone(), request, token);
            let mut peer = peer.join().unwrap();
            assert!(
                peer.request(&Request::Status {
                    id: self.binding.id
                })
                .is_err(),
                "original incoming TLS socket survived prepare"
            );
            pending
        })
    }
    fn prepare(
        &self,
        request: Request,
        token: codec::UnverifiedToken,
    ) -> faction_provider::services::PendingRetrieval {
        self.prepare_result(request, token).unwrap()
    }
    fn complete(
        &self,
        pending: faction_provider::services::PendingRetrieval,
        store: &std::sync::Mutex<Store>,
    ) -> Result<Response, Error> {
        thread::scope(|scope| {
            let issuer = scope.spawn(|| {
                self.b
                    .server()
                    .accept_service(&self.callback, &self.rb, Arc::new(AtomicBool::new(false)))
                    .unwrap()
                    .serve_http_once()
            });
            let response = pending.execute(&self.endpoint(), store);
            let result = issuer.join().unwrap();
            assert!(result.is_ok());
            response
        })
    }
}
/// Test-only setup authorizes exactly the normal publication writes. Never reads.
struct Setup(Binding);
impl faction_files::private_blobs::Access for Setup {
    fn require(
        &self,
        actor: &Principal,
        binding: &Binding,
        action: faction_files::private_blobs::Action,
        _now: u64,
    ) -> Result<(), faction_files::private_blobs::Error> {
        use faction_files::private_blobs::{Action, Error};
        if actor == &self.0.sender
            && binding == &self.0
            && matches!(action, Action::Begin | Action::PutChunk | Action::Commit)
        {
            Ok(())
        } else {
            Err(Error::Denied)
        }
    }
}
fn preexisting(f: &Fixture, binding: &Binding, committed: bool) -> std::sync::Mutex<Store> {
    let b = binding.clone();
    let mut store = f.store();
    let setup = Setup(b.clone());
    let t = wall_now();
    store
        .dispatch(&b.sender, &setup, Request::Begin(b.clone()), t)
        .unwrap();
    store
        .dispatch(
            &b.sender,
            &setup,
            Request::PutChunk {
                id: b.id,
                offset: 0,
                bytes: vec![9; 32],
            },
            t,
        )
        .unwrap();
    if committed {
        store
            .dispatch(&b.sender, &setup, Request::Commit { id: b.id }, t)
            .unwrap();
    }
    drop(store);
    std::sync::Mutex::new(f.store())
}
#[test]
fn real_two_home_device_close_relay_issue_close_prepare_close_fresh_introspection_private_committed_read()
 {
    for action in [
        RetrievalAction::GetChunk { offset: 0 },
        RetrievalAction::GetChunk { offset: 32 },
        RetrievalAction::Status,
    ] {
        let f = Fixture::new();
        let homes = Homes::new();
        let store = preexisting(&f, &homes.binding, true);
        let token = homes.token(action);
        let request = match action {
            RetrievalAction::GetChunk { offset } => Request::GetChunk {
                id: homes.binding.id,
                offset,
            },
            RetrievalAction::Status => Request::Status {
                id: homes.binding.id,
            },
        };
        let pending = homes.prepare(request, token);
        with_session(&homes.a, &homes.ra, &homes.a.alice, |_s| ());
        thread::scope(|scope| {
            let issuer = scope.spawn(|| {
                homes
                    .b
                    .server()
                    .accept_service(&homes.callback, &homes.rb, Arc::new(AtomicBool::new(false)))
                    .unwrap()
                    .serve_http_once()
            });
            let response = pending.execute(&homes.endpoint(), &store).unwrap();
            issuer.join().unwrap().unwrap();
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
                    assert!(committed);
                    assert_eq!(binding, homes.binding);
                }
                _ => panic!("wrong bounded Store response"),
            }
        });
    }
}
#[test]
fn identical_uncommitted_ciphertext_cannot_be_retrieved_and_local_foreign_publication_remains_denied()
 {
    for action in [
        RetrievalAction::GetChunk { offset: 0 },
        RetrievalAction::Status,
    ] {
        let f = Fixture::new();
        let h = Homes::new();
        let store = preexisting(&f, &h.binding, false);
        let before = f.bytes();
        let token = h.token(action);
        let request = match action {
            RetrievalAction::GetChunk { offset } => Request::GetChunk {
                id: h.binding.id,
                offset,
            },
            RetrievalAction::Status => Request::Status { id: h.binding.id },
        };
        let pending = h.prepare(request, token);
        assert!(h.complete(pending, &store).is_err());
        assert_eq!(f.bytes(), before);
        with_session(&h.a, &h.ra, &h.a.alice, |mut session| {
            let mut store = store.lock().unwrap();
            assert!(
                session
                    .dispatch(
                        &mut store,
                        "alice@sender.test",
                        Request::Begin(h.binding.clone())
                    )
                    .is_err()
            );
        });
        assert_eq!(f.bytes(), before);
    }
}
#[test]
fn incoming_resource_permission_never_authorizes_publication_delete_or_changed_request_offset_id() {
    let f = Fixture::new();
    let h = Homes::new();
    let _store = preexisting(&f, &h.binding, true);
    let before = f.bytes();
    for request in [
        Request::Begin(h.binding.clone()),
        Request::PutChunk {
            id: h.binding.id,
            offset: 0,
            bytes: vec![9; 32],
        },
        Request::Commit { id: h.binding.id },
        Request::Delete { id: h.binding.id },
        Request::GetChunk {
            id: h.binding.id,
            offset: 33,
        },
        Request::GetChunk {
            id: [4; 32],
            offset: 0,
        },
    ] {
        assert!(
            h.prepare_result(request, codec::UnverifiedToken::from_bytes([42; 32]))
                .is_err()
        );
        assert_eq!(f.bytes(), before);
    }
}
#[test]
fn exact_retry_performs_another_fresh_introspection_after_a_lost_application_reply() {
    let f = Fixture::new();
    let h = Homes::new();
    let store = preexisting(&f, &h.binding, true);
    let token = h
        .token(RetrievalAction::GetChunk { offset: 0 })
        .decode_bytes()
        .unwrap();
    let request = Request::GetChunk {
        id: h.binding.id,
        offset: 0,
    };
    let mut results = Vec::new();
    for _ in 0..2 {
        let pending = h.prepare(request.clone(), codec::UnverifiedToken::from_bytes(token));
        results.push(h.complete(pending, &store).unwrap());
    }
    assert_eq!(results[0], results[1]);
    assert!(matches!(&results[1],Response::Chunk {bytes,..} if bytes==&vec![9;32]));
}
fn claimed_response(h: &Homes) -> codec::UnverifiedIntrospection {
    let b = &h.binding;
    let p = |p: &Principal| codec::UnverifiedPrincipal {
        account: p.account.clone(),
        device: p.device,
        identity: p.identity.iter().map(|v| format!("{v:02x}")).collect(),
    };
    let now = wall_now();
    codec::UnverifiedIntrospection::Retrieval {
        claims: codec::UnverifiedClaims {
            issuer: "issuer.test".into(),
            issuer_home: "recipient.test".into(),
            client_id: "relay".into(),
            audience: "sender.test/attachments".into(),
            subject: b.recipient.account.clone(),
            issued_at: now,
            expires_at: now + 5,
            generation: 2,
            reset: 0,
            certificate_thumbprint: codec::UnverifiedReference::from_bytes(sha256(
                &h.b.alice.0.to_der().unwrap(),
            )),
            binding: codec::UnverifiedBinding {
                sender: p(&b.sender),
                recipient: p(&b.recipient),
                id: b.id.iter().map(|v| format!("{v:02x}")).collect(),
                size: b.size,
                created: b.created,
                expires: b.expires,
            },
        },
        action: codec::UnverifiedRetrievalAction::GetChunk { offset: 0 },
    }
}
fn issuer_acceptor(h: &Homes) -> openssl::ssl::SslAcceptor {
    use openssl::ssl::*;
    let mut b = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls_server()).unwrap();
    b.set_min_proto_version(Some(SslVersion::TLS1_3)).unwrap();
    b.set_max_proto_version(Some(SslVersion::TLS1_3)).unwrap();
    b.set_certificate(&h.b.server.0).unwrap();
    b.set_private_key(&h.b.server.1).unwrap();
    b.cert_store_mut().add_cert(h.b.ca.0.clone()).unwrap();
    b.set_verify(SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT);
    b.build()
}
fn fake_reply(
    h: &Homes,
    pending: faction_provider::services::PendingRetrieval,
    store: &std::sync::Mutex<Store>,
    body: Vec<u8>,
) -> Result<Response, Error> {
    use std::io::Read;
    thread::scope(|scope| {
        let issuer=scope.spawn(|| {
            let (socket,_)=h.callback.accept().unwrap();socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();socket.set_write_timeout(Some(Duration::from_secs(3))).unwrap();let mut stream=issuer_acceptor(h).accept(socket).unwrap();let mut scratch=[0;codec::MAX_HEADER+codec::MAX_FORM];let mut used=0;
            loop {let n=stream.read(&mut scratch[used..]).unwrap();assert!(n>0);used+=n;if codec::decode_introspection_http(&scratch[..used]).is_ok(){break;}assert!(used<scratch.len());}
            let request=codec::decode_introspection_http(&scratch[..used]).unwrap();assert_eq!(request.client_id,"inspector");
            let mut bytes=format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).into_bytes();bytes.extend(body);stream.write_all(&bytes).unwrap();
        });
        let result = pending.execute(&h.endpoint(), store);
        issuer.join().unwrap();
        result
    })
}
#[test]
fn actual_fresh_tls_metadata_cannot_substitute_any_claim_tuple_certificate_action_generation_reset_or_lifetime()
 {
    for variant in 0..23 {
        let f = Fixture::new();
        let h = Homes::new();
        let store = preexisting(&f, &h.binding, true);
        let token = h.token(RetrievalAction::GetChunk { offset: 0 });
        let pending = h.prepare(
            Request::GetChunk {
                id: h.binding.id,
                offset: 0,
            },
            token,
        );
        let mut metadata = claimed_response(&h);
        if let codec::UnverifiedIntrospection::Retrieval { claims, action } = &mut metadata {
            match variant {
                0 => {}
                1 => claims.issuer = "other.test".into(),
                2 => claims.issuer_home = "other.test".into(),
                3 => claims.client_id = "inspector".into(),
                4 => claims.audience = "other.test/attachments".into(),
                5 => claims.binding.sender.account = "other@sender.test".into(),
                6 => claims.binding.sender.device += 1,
                7 => claims.binding.sender.identity = "99".repeat(32),
                8 => {
                    claims.binding.recipient.account = "other@recipient.test".into();
                    claims.subject = claims.binding.recipient.account.clone();
                }
                9 => claims.binding.recipient.device += 1,
                10 => claims.binding.recipient.identity = "98".repeat(32),
                11 => claims.binding.id = "97".repeat(32),
                12 => claims.binding.size += 1,
                13 => claims.binding.created = claims.binding.created.saturating_sub(1),
                14 => claims.binding.expires += 1,
                15 => *action = codec::UnverifiedRetrievalAction::Status,
                16 => *action = codec::UnverifiedRetrievalAction::GetChunk { offset: 1 },
                17 => claims.generation = 1,
                18 => claims.generation = 3,
                19 => claims.reset = 1,
                20 => {
                    claims.certificate_thumbprint = codec::UnverifiedReference::from_bytes(sha256(
                        &h.inspector.0.to_der().unwrap(),
                    ))
                }
                21 => {
                    claims.issued_at = wall_now() - 1;
                    claims.expires_at = wall_now();
                }
                _ => {
                    claims.issued_at = wall_now() + 60;
                    claims.expires_at = wall_now() + 65;
                }
            }
        }
        let body = codec::encode_introspection_json(&metadata)
            .unwrap_or_else(|_| panic!("invalid synthetic claim fixture variant {variant}"));
        let before = f.bytes();
        let result = fake_reply(&h, pending, &store, body);
        if variant == 0 {
            assert!(matches!(result,Ok(Response::Chunk {bytes,..}) if bytes==vec![9;32]));
        } else {
            assert!(result.is_err(), "accepted substituted variant {variant}");
            assert_eq!(
                f.bytes(),
                before,
                "denial changed Store for variant {variant}"
            );
        }
    }
}
impl Homes {
    fn a_registry(
        &self,
        generation: u64,
        change: impl FnOnce(&mut Vec<CredentialBinding>),
    ) -> Registry {
        let mut entries = self.a.entries();
        entries.truncate(1);
        entries[0].principal = sender();
        entries[0].generation = generation;
        change(&mut entries);
        Registry::new(
            generation,
            0,
            vec!["sender.test".into(), "recipient.test".into()],
            entries,
        )
        .unwrap()
    }
    fn association(&self) -> ResourceAssociation {
        ResourceAssociation {
            recipient_home: "recipient.test".into(),
            issuer: "issuer.test".into(),
            audience: "sender.test/attachments".into(),
            relay_client_id: "relay".into(),
            outbound_client_id: "inspector".into(),
            endpoint_id: "endpoint".into(),
            address: self.callback.local_addr().unwrap(),
            http_host: "issuer.test".into(),
            ca_sha256: sha256(&self.b.ca.0.to_der().unwrap()),
            remote_generation: 2,
            remote_reset: 0,
        }
    }
    fn a_policy(
        &self,
        a: ResourceAssociation,
        change: impl FnOnce(&mut Vec<(ServiceRegistration, ServiceUse)>),
    ) -> ServicePolicy {
        let mut entries = vec![
            (
                registration(&self.b.alice.0, "recipient.test", "relay"),
                ServiceUse::ResourceClient,
            ),
            (
                registration(&self.inspector.0, "sender.test", "inspector"),
                ServiceUse::OutboundIntrospection,
            ),
            (
                registration(&self.b.server.0, "recipient.test", "endpoint"),
                ServiceUse::IssuerEndpoint,
            ),
        ];
        change(&mut entries);
        ServicePolicy::new_resource("sender.test".into(), entries, vec![a]).unwrap()
    }
}
#[test]
fn removed_expired_disabled_sender_peer_role_and_associations_deny_before_any_callback_or_store_change()
 {
    for variant in 0..9 {
        let f = Fixture::new();
        let h = Homes::new();
        let store = preexisting(&f, &h.binding, true);
        let token = h.token(RetrievalAction::GetChunk { offset: 0 });
        let pending = h.prepare(
            Request::GetChunk {
                id: h.binding.id,
                offset: 0,
            },
            token,
        );
        let before = f.bytes();
        match variant {
            0 => {
                h.ra.control()
                    .replace_delegation_policy(
                        h.a_registry(3, |e| e[0].approved = false),
                        h.a_policy(h.association(), |_| ()),
                    )
                    .unwrap()
            }
            1 => {
                h.ra.control()
                    .replace_delegation_policy(
                        h.a_registry(3, |e| e[0].valid_until = wall_now()),
                        h.a_policy(h.association(), |_| ()),
                    )
                    .unwrap()
            }
            2 => {
                h.ra.control()
                    .replace_delegation_policy(
                        h.a_registry(3, |e| e[0].device_enabled = false),
                        h.a_policy(h.association(), |_| ()),
                    )
                    .unwrap()
            }
            3 => {
                h.ra.control()
                    .replace_delegation_policy(
                        h.a_registry(3, |e| e[0].account_enabled = false),
                        h.a_policy(h.association(), |_| ()),
                    )
                    .unwrap()
            }
            4 => {
                h.ra.control()
                    .replace_registry(
                        Registry::new(
                            3,
                            0,
                            vec!["sender.test".into()],
                            h.a_registry(3, |_| ()).bindings().to_vec(),
                        )
                        .unwrap(),
                    )
                    .unwrap()
            }
            5 => h.ra.control().reset(1).unwrap(),
            6 => {
                h.ra.control()
                    .replace_delegation_policy(
                        h.a_registry(3, |_| ()),
                        h.a_policy(h.association(), |e| e[0].0.enabled = false),
                    )
                    .unwrap()
            }
            7 => {
                h.ra.control()
                    .replace_delegation_policy(
                        h.a_registry(3, |_| ()),
                        h.a_policy(h.association(), |e| e[1].0.enabled = false),
                    )
                    .unwrap()
            }
            _ => {
                h.ra.control()
                    .replace_delegation_policy(
                        h.a_registry(3, |_| ()),
                        h.a_policy(h.association(), |e| e[2].0.valid_until = wall_now()),
                    )
                    .unwrap()
            }
        }
        h.callback.set_nonblocking(true).unwrap();
        assert!(pending.execute(&h.endpoint(), &store).is_err());
        assert!(matches!(h.callback.accept(),Err(ref e) if e.kind()==io::ErrorKind::WouldBlock));
        assert_eq!(f.bytes(), before);
    }
}
#[test]
fn external_endpoint_must_match_every_persisted_trust_and_outgoing_identity_before_connect() {
    for variant in 0..6 {
        let f = Fixture::new();
        let h = Homes::new();
        let store = preexisting(&f, &h.binding, true);
        let token = h.token(RetrievalAction::GetChunk { offset: 0 });
        let pending = h.prepare(
            Request::GetChunk {
                id: h.binding.id,
                offset: 0,
            },
            token,
        );
        let before = f.bytes();
        let ca = if variant == 0 { &h.a.ca.0 } else { &h.b.ca.0 };
        let who = if variant == 1 {
            &h.b.alice
        } else {
            &h.inspector
        };
        let credentials = ClientCredentials::from_pem(
            &ca.to_pem().unwrap(),
            &who.0.to_pem().unwrap(),
            &who.1.private_key_to_pem_pkcs8().unwrap(),
        )
        .unwrap();
        let ep = HttpEndpoint::new(
            if variant == 2 {
                "127.0.0.1:9".parse().unwrap()
            } else {
                h.callback.local_addr().unwrap()
            },
            if variant == 3 {
                "other.test"
            } else {
                "issuer.test"
            }
            .into(),
            if variant == 4 {
                "other.test"
            } else {
                "provider.test"
            }
            .into(),
            if variant == 5 {
                [42; 32]
            } else {
                sha256(&h.b.server.0.to_der().unwrap())
            },
            credentials,
        )
        .unwrap();
        h.callback.set_nonblocking(true).unwrap();
        assert!(pending.execute(&ep, &store).is_err());
        assert!(matches!(h.callback.accept(),Err(ref e) if e.kind()==io::ErrorKind::WouldBlock));
        assert_eq!(f.bytes(), before);
    }
}
#[test]
fn outgoing_and_endpoint_registrations_cannot_authenticate_an_incoming_service_credential() {
    let h = Homes::new();
    for who in [&h.inspector, &h.b.server] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::scope(|scope| {
            let peer = scope.spawn(|| {
                Client::connect(
                    address,
                    "provider.test",
                    sha256(&h.a.server.0.to_der().unwrap()),
                    &h.a.ca.0,
                    Some((&who.0, Some(&who.1))),
                    Arc::new(AtomicBool::new(false)),
                )
            });
            let result = Server::new(&h.a.server.0, &h.a.server.1, &h.b.ca.0)
                .unwrap()
                .accept_service(&listener, &h.ra, Arc::new(AtomicBool::new(false)));
            assert!(result.is_err());
            drop(peer.join().unwrap());
        });
    }
}
#[test]
fn inactive_and_observation_purpose_metadata_never_become_a_recipient_actor() {
    for observation in [false, true] {
        let f = Fixture::new();
        let h = Homes::new();
        let store = preexisting(&f, &h.binding, true);
        let token = h.token(RetrievalAction::GetChunk { offset: 0 });
        let pending = h.prepare(
            Request::GetChunk {
                id: h.binding.id,
                offset: 0,
            },
            token,
        );
        let before = f.bytes();
        let metadata = if observation {
            let codec::UnverifiedIntrospection::Retrieval { mut claims, .. } = claimed_response(&h)
            else {
                unreachable!()
            };
            claims.subject = claims.client_id.clone();
            codec::UnverifiedIntrospection::RecipientCurrent {
                claims,
                action: codec::UnverifiedObservationAction::Publish,
            }
        } else {
            codec::UnverifiedIntrospection::Inactive
        };
        assert!(
            fake_reply(
                &h,
                pending,
                &store,
                codec::encode_introspection_json(&metadata).unwrap()
            )
            .is_err()
        );
        assert_eq!(f.bytes(), before);
    }
}
impl Homes {
    fn b_registry(&self, generation: u64, reset: u64, approved: bool) -> Registry {
        let mut entries = self.b.entries();
        entries.remove(0);
        entries[0].principal = recipient();
        entries[0].generation = generation;
        entries[0].approved = approved;
        Registry::new(
            generation,
            reset,
            vec!["recipient.test".into(), "sender.test".into()],
            entries,
        )
        .unwrap()
    }
    fn b_policy(&self) -> ServicePolicy {
        ServicePolicy::new_issuer(
            "recipient.test".into(),
            "issuer.test".into(),
            vec![
                (
                    registration(&self.b.alice.0, "recipient.test", "relay"),
                    ServiceUse::Relay,
                ),
                (
                    registration(&self.inspector.0, "sender.test", "inspector"),
                    ServiceUse::Introspection,
                ),
            ],
            vec![IssuerRoute {
                sender_home: "sender.test".into(),
                audience: "sender.test/attachments".into(),
                relay_client_id: "relay".into(),
                introspection_client_id: "inspector".into(),
            }],
        )
        .unwrap()
    }
    fn complete_any(
        &self,
        pending: faction_provider::services::PendingRetrieval,
        store: &std::sync::Mutex<Store>,
    ) -> (Result<Response, Error>, Result<(), Error>) {
        thread::scope(|scope| {
            let issuer = scope.spawn(|| {
                match self.b.server().accept_service(
                    &self.callback,
                    &self.rb,
                    Arc::new(AtomicBool::new(false)),
                ) {
                    Ok(session) => session.serve_http_once(),
                    Err(stage) => Err(Error::Transport(stage)),
                }
            });
            let response = pending.execute(&self.endpoint(), store);
            (response, issuer.join().unwrap())
        })
    }
    fn restart_b(self) -> Self {
        let Self {
            a,
            b,
            inspector,
            ra,
            rb,
            callback,
            binding,
        } = self;
        drop(rb);
        let rb = Runtime::new(None);
        let fresh = Self {
            a,
            b,
            inspector,
            ra,
            rb,
            callback,
            binding,
        };
        fresh
            .rb
            .control()
            .replace_delegation_policy(fresh.b_registry(2, 0, true), fresh.b_policy())
            .unwrap();
        fresh
            .rb
            .control()
            .replace_retrieval_policy(
                RetrievalPolicy::new(
                    "recipient.test".into(),
                    vec![("sender.test".into(), "sender.test/attachments".into())],
                )
                .unwrap(),
            )
            .unwrap();
        fresh
    }
}
#[test]
fn actual_remote_revoke_and_reset_deny_without_ciphertext_or_store_clock_change() {
    for reset in [false, true] {
        let f = Fixture::new();
        let h = Homes::new();
        let store = preexisting(&f, &h.binding, true);
        let token = h.token(RetrievalAction::GetChunk { offset: 0 });
        let pending = h.prepare(
            Request::GetChunk {
                id: h.binding.id,
                offset: 0,
            },
            token,
        );
        let before = f.bytes();
        if reset {
            h.rb.control().reset(1).unwrap();
        } else {
            h.rb.control()
                .replace_delegation_policy(h.b_registry(3, 0, false), h.b_policy())
                .unwrap();
        }
        assert!(h.complete_any(pending, &store).0.is_err());
        assert_eq!(f.bytes(), before);
    }
}
#[test]
fn actual_remote_generation_reset_change_requires_a_manual_complete_higher_local_policy_update() {
    let f = Fixture::new();
    let h = Homes::new();
    let store = preexisting(&f, &h.binding, true);
    h.rb.control().reset(1).unwrap();
    h.rb.control()
        .replace_delegation_policy(h.b_registry(4, 1, true), h.b_policy())
        .unwrap();
    let token = h
        .token(RetrievalAction::GetChunk { offset: 0 })
        .decode_bytes()
        .unwrap();
    let request = Request::GetChunk {
        id: h.binding.id,
        offset: 0,
    };
    let before = f.bytes();
    let stale = h.prepare(request.clone(), codec::UnverifiedToken::from_bytes(token));
    assert!(h.complete(stale, &store).is_err());
    assert_eq!(f.bytes(), before);
    let mut association = h.association();
    association.remote_generation = 4;
    association.remote_reset = 1;
    h.ra.control()
        .replace_delegation_policy(h.a_registry(3, |_| ()), h.a_policy(association, |_| ()))
        .unwrap();
    let current = h.prepare(request, codec::UnverifiedToken::from_bytes(token));
    assert!(
        matches!(h.complete(current,&store),Ok(Response::Chunk {bytes,..}) if bytes==vec![9;32])
    );
}
#[test]
fn actual_issuer_restart_discards_token_records_and_each_exact_retry_needs_fresh_authority() {
    let f = Fixture::new();
    let h = Homes::new();
    let store = preexisting(&f, &h.binding, true);
    let token = h.token(RetrievalAction::GetChunk { offset: 0 });
    let pending = h.prepare(
        Request::GetChunk {
            id: h.binding.id,
            offset: 0,
        },
        token,
    );
    let before = f.bytes();
    let h = h.restart_b();
    assert!(h.complete(pending, &store).is_err());
    assert_eq!(f.bytes(), before);
    let fresh = h.prepare(
        Request::GetChunk {
            id: h.binding.id,
            offset: 0,
        },
        h.token(RetrievalAction::GetChunk { offset: 0 }),
    );
    assert!(matches!(h.complete(fresh,&store),Ok(Response::Chunk {bytes,..}) if bytes==vec![9;32]));
}
#[test]
fn actual_expired_remote_token_is_not_renewed_by_a_new_incoming_cycle() {
    let f = Fixture::new();
    let h = Homes::new();
    let store = preexisting(&f, &h.binding, true);
    let (token, expires) = h.token_with_expiry(RetrievalAction::GetChunk { offset: 0 });
    let limit = Instant::now() + CYCLE_LIMIT;
    while SystemTime::now().duration_since(UNIX_EPOCH).unwrap() < Duration::from_secs(expires) {
        assert!(Instant::now() < limit);
        thread::sleep(Duration::from_millis(10));
    }
    let pending = h.prepare(
        Request::GetChunk {
            id: h.binding.id,
            offset: 0,
        },
        token,
    );
    let before = f.bytes();
    assert!(h.complete_any(pending, &store).0.is_err());
    assert_eq!(f.bytes(), before);
}
#[test]
fn actual_issuer_outage_refuses_without_fallback_and_ordinary_local_provider_store_progresses() {
    let f = Fixture::new();
    let h = Homes::new();
    let store = preexisting(&f, &h.binding, true);
    let token = h.token(RetrievalAction::GetChunk { offset: 0 });
    let pending = h.prepare(
        Request::GetChunk {
            id: h.binding.id,
            offset: 0,
        },
        token,
    );
    let endpoint = h.endpoint();
    let before = f.bytes();
    drop(h.callback);
    assert!(pending.execute(&endpoint, &store).is_err());
    assert_eq!(f.bytes(), before);
    with_session(&h.a, &h.ra, &h.a.alice, |mut session| {
        let mut local = h.binding.clone();
        local.recipient = local.sender.clone();
        local.id = sha256(&[7; 32]);
        assert!(
            session
                .dispatch(
                    &mut store.lock().unwrap(),
                    "alice@sender.test",
                    Request::Begin(local)
                )
                .is_ok()
        );
    });
}
