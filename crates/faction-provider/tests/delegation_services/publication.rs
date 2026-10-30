use super::*;
use faction_delegation_codec as codec;
use faction_provider::services::PublicationWireProfile;
use faction_provider::tls::{ClientCredentials, HttpEndpoint};
fn query(action: codec::UnverifiedPublicationAction) -> codec::UnverifiedPublicationQuery {
    let b = foreign_binding();
    let p = |p: Principal| codec::UnverifiedPrincipal {
        account: p.account,
        device: p.device,
        identity: p.identity.iter().map(|b| format!("{b:02x}")).collect(),
    };
    codec::UnverifiedPublicationQuery {
        client_id: "sender-service".into(),
        action,
        binding: codec::UnverifiedBinding {
            sender: p(b.sender),
            recipient: p(b.recipient),
            id: b.id.iter().map(|b| format!("{b:02x}")).collect(),
            size: b.size,
            created: b.created,
            expires: b.expires,
        },
    }
}
fn endpoint(c: &Certs, address: SocketAddr) -> HttpEndpoint {
    let credentials = ClientCredentials::from_pem(
        &c.ca.0.to_pem().unwrap(),
        &c.alice.0.to_pem().unwrap(),
        &c.alice.1.private_key_to_pem_pkcs8().unwrap(),
    )
    .unwrap();
    HttpEndpoint::new(
        address,
        "home.test".into(),
        "provider.test".into(),
        sha256(&c.server.0.to_der().unwrap()),
        credentials,
    )
    .unwrap()
}
#[test]
fn independently_configured_homes_real_outbound_mtls_preserves_every_exact_publication_selector() {
    let c = Certs::new();
    let r = Runtime::new(None);
    r.control()
        .replace_delegation_policy(policy(&c, 1), services(&c))
        .unwrap();
    let a_c = Certs::new();
    let a = sender_home(&a_c);
    for action in [
        codec::UnverifiedPublicationAction::Begin,
        codec::UnverifiedPublicationAction::PutChunk {
            offset: 0,
            length: 32,
            ciphertext_sha256: "77".repeat(32),
        },
        codec::UnverifiedPublicationAction::Commit,
        codec::UnverifiedPublicationAction::Status,
        codec::UnverifiedPublicationAction::Delete,
    ] {
        let q = query(action);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let ep = endpoint(&c, listener.local_addr().unwrap());
        thread::scope(|s| {
            let caller = s.spawn(|| {
                a.control().query_publication(
                    &ep,
                    &q,
                    Instant::now() + CYCLE_LIMIT,
                    Arc::new(AtomicBool::new(false)),
                )
            });
            let session = c
                .server()
                .accept_service(&listener, &r, Arc::new(AtomicBool::new(false)))
                .unwrap();
            let profile =
                PublicationWireProfile::new("home.test".into(), vec!["sender-service".into()])
                    .unwrap();
            session.serve_publication_http_once(&profile).unwrap();
            match caller.join().unwrap().unwrap() {
                codec::UnverifiedPublicationObservation::Current {
                    query,
                    home,
                    service_home,
                    service_certificate_sha256,
                    recipient_certificate_sha256,
                    generation,
                    reset,
                    ..
                } => {
                    assert_eq!(*query, q);
                    assert_eq!(home, "home.test");
                    assert_eq!(service_home, "foreign.test");
                    assert_eq!(generation, 1);
                    assert_eq!(reset, 0);
                    assert_eq!(
                        service_certificate_sha256,
                        sha256(&c.alice.0.to_der().unwrap())
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>()
                    );
                    assert_eq!(
                        recipient_certificate_sha256,
                        sha256(&c.bob.0.to_der().unwrap())
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>()
                    );
                }
                _ => panic!("missing exact current metadata"),
            }
        });
    }
}
#[test]
fn publication_profile_only_restricts_current_policy_and_wire_compatible_ids() {
    assert!(
        PublicationWireProfile::new("home.test".into(), vec!["legacy observation!".into()])
            .is_err()
    );
    assert!(PublicationWireProfile::new("home.test".into(), vec![]).is_err());
    assert!(
        PublicationWireProfile::new("home.test".into(), vec!["same".into(), "same".into()])
            .is_err()
    );
}
fn sender_home(c: &Certs) -> Runtime {
    let mut entries = c.entries();
    entries.truncate(1);
    entries[0].principal.account = "alice@foreign.test".into();
    Runtime::new(Some(
        Registry::new(
            1,
            0,
            vec!["foreign.test".into(), "home.test".into()],
            entries,
        )
        .unwrap(),
    ))
}
fn profile() -> PublicationWireProfile {
    PublicationWireProfile::new("home.test".into(), vec!["sender-service".into()]).unwrap()
}
fn configured(c: &Certs) -> Runtime {
    let r = Runtime::new(None);
    r.control()
        .replace_delegation_policy(policy(c, 1), services(c))
        .unwrap();
    r
}
fn raw_peer(
    c: &Certs,
    who: &(X509, PKey<Private>),
    addr: SocketAddr,
) -> openssl::ssl::SslStream<TcpStream> {
    use openssl::ssl::*;
    let mut b = SslConnector::builder(SslMethod::tls_client()).unwrap();
    b.set_min_proto_version(Some(SslVersion::TLS1_3)).unwrap();
    b.set_max_proto_version(Some(SslVersion::TLS1_3)).unwrap();
    b.cert_store_mut().add_cert(c.ca.0.clone()).unwrap();
    b.set_certificate(&who.0).unwrap();
    b.set_private_key(&who.1).unwrap();
    let socket = TcpStream::connect(addr).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    socket
        .set_write_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    b.build().connect("provider.test", socket).unwrap()
}
fn raw_exchange(
    c: &Certs,
    r: &Runtime,
    bytes: &[u8],
    p: &PublicationWireProfile,
) -> (Result<(), Error>, Vec<u8>) {
    use std::io::Read;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::scope(|s| {
        let client = s.spawn(|| {
            let mut peer = raw_peer(c, &c.alice, addr);
            peer.write_all(bytes).unwrap();
            let mut result = Vec::new();
            let mut scratch = [0; 1024];
            while let Ok(n) = peer.read(&mut scratch) {
                if n == 0 {
                    break;
                }
                assert!(result.len() + n <= codec::MAX_HEADER + codec::MAX_JSON);
                result.extend_from_slice(&scratch[..n]);
            }
            result
        });
        let session = c
            .server()
            .accept_service(&listener, r, Arc::new(AtomicBool::new(false)))
            .unwrap();
        let result = session.serve_publication_http_once(p);
        (result, client.join().unwrap())
    })
}
#[test]
fn actual_service_http_refuses_changed_recipient_client_home_profile_and_framing_without_metadata()
{
    let c = Certs::new();
    let r = configured(&c);
    let q = query(codec::UnverifiedPublicationAction::Begin);
    let valid = codec::encode_publication_query_http("home.test", &q).unwrap();
    let mut changed = q.clone();
    changed.binding.recipient.identity = "98".repeat(32);
    let mut client = q.clone();
    client.client_id = "other".into();
    let mut device = q.clone();
    device.binding.recipient.device += 1;
    let mut account = q.clone();
    account.binding.recipient.account = "unknown@home.test".into();
    let raw = String::from_utf8(valid.clone()).unwrap();
    for bytes in [
        codec::encode_publication_query_http("home.test", &changed).unwrap(),
        codec::encode_publication_query_http("home.test", &client).unwrap(),
        codec::encode_publication_query_http("home.test", &device).unwrap(),
        codec::encode_publication_query_http("home.test", &account).unwrap(),
        codec::encode_publication_query_http("other.test", &q).unwrap(),
        raw.replace(
            "Content-Length:",
            "Transfer-Encoding: chunked\r\nContent-Length:",
        )
        .into_bytes(),
        [valid.clone(), vec![b'x']].concat(),
        raw.replace("Content-Length:", "Content-Length: 8193\r\nX-Length:")
            .into_bytes(),
    ] {
        let (result, reply) = raw_exchange(&c, &r, &bytes, &profile());
        assert!(result.is_err());
        if !reply.is_empty() {
            assert_eq!(
                codec::decode_publication_response_http(&reply).unwrap(),
                codec::UnverifiedPublicationObservation::Unavailable
            );
        }
        assert!(!String::from_utf8_lossy(&reply).contains("bob@"));
    }
    for p in [
        PublicationWireProfile::new("other.test".into(), vec!["sender-service".into()]).unwrap(),
        PublicationWireProfile::new("home.test".into(), vec!["other".into()]).unwrap(),
    ] {
        let (result, reply) = raw_exchange(&c, &r, &valid, &p);
        assert!(result.is_err());
        assert!(reply.is_empty());
    }
}
#[test]
fn actual_current_recipient_revocation_expiry_invitation_reset_and_other_service_roles_refuse() {
    use faction_provider::services::{IssuerRoute, ServiceUse};
    for variant in 0..7 {
        let c = Certs::new();
        let r = configured(&c);
        if variant < 5 {
            let mut entries = c.entries();
            entries.remove(0);
            for e in &mut entries {
                e.generation = 2;
            }
            let mut homes = vec!["home.test".into(), "foreign.test".into()];
            match variant {
                0 => entries[0].approved = false,
                1 => entries[0].valid_until = wall_now(),
                2 => {
                    homes.pop();
                }
                3 => entries[0].device_enabled = false,
                _ => entries[0].account_enabled = false,
            };
            let registry = Registry::new(2, 0, homes, entries).unwrap();
            if variant == 2 {
                r.control().replace_registry(registry).unwrap();
            } else {
                r.control()
                    .replace_delegation_policy(registry, services(&c))
                    .unwrap();
            }
        } else {
            // A fresh registry and distinct registered credentials preserve typed history.
            let t = wall_now() as i64;
            let inspector = cert(Some((&c.ca.0, &c.ca.1)), "inspector", 91, t - 60, t + 3600);
            let relay = ServiceRegistration {
                home: "home.test".into(),
                client_id: "relay".into(),
                dns_san: "provider.test".into(),
                certificate_sha256: sha256(&inspector.0.to_der().unwrap()),
                enabled: true,
                valid_from: wall_now() - 60,
                valid_until: wall_now() + 3600,
            };
            let mut observer = services(&c).registrations()[0].clone();
            observer.client_id = "sender-service".into();
            let purpose = if variant == 5 {
                ServiceUse::Introspection
            } else {
                ServiceUse::Observation
            };
            let p = ServicePolicy::new_issuer(
                "home.test".into(),
                "issuer.test".into(),
                vec![(relay, ServiceUse::Relay), (observer, purpose)],
                vec![IssuerRoute {
                    sender_home: "foreign.test".into(),
                    audience: "foreign.test/attachments".into(),
                    relay_client_id: "relay".into(),
                    introspection_client_id: if variant == 5 {
                        "sender-service".into()
                    } else {
                        "inspector".into()
                    },
                }],
            );
            if variant == 5 {
                r.control()
                    .replace_delegation_policy(policy(&c, 2), p.unwrap())
                    .unwrap();
            } else {
                r.control().reset(2).unwrap();
            }
        }
        let bytes = codec::encode_publication_query_http(
            "home.test",
            &query(codec::UnverifiedPublicationAction::Begin),
        )
        .unwrap();
        // Removed service/peer policy may deny during authenticated admission itself.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::scope(|s| {
            let client = s.spawn(|| {
                use std::io::Read;
                let mut peer = raw_peer(&c, &c.alice, addr);
                let _ = peer.write_all(&bytes);
                let mut reply = Vec::new();
                let _ = peer.read_to_end(&mut reply);
                reply
            });
            if let Ok(session) =
                c.server()
                    .accept_service(&listener, &r, Arc::new(AtomicBool::new(false)))
            {
                assert!(session.serve_publication_http_once(&profile()).is_err());
            }
            let reply = client.join().unwrap();
            assert!(!String::from_utf8_lossy(&reply).contains("bob@"));
        });
    }
}
fn fake_acceptor(c: &Certs, tls12: bool) -> openssl::ssl::SslAcceptor {
    use openssl::ssl::*;
    let mut b = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls_server()).unwrap();
    let v = if tls12 {
        SslVersion::TLS1_2
    } else {
        SslVersion::TLS1_3
    };
    b.set_min_proto_version(Some(v)).unwrap();
    b.set_max_proto_version(Some(v)).unwrap();
    b.set_certificate(&c.server.0).unwrap();
    b.set_private_key(&c.server.1).unwrap();
    b.cert_store_mut().add_cert(c.ca.0.clone()).unwrap();
    b.set_verify(SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT);
    b.build()
}
fn current_reply(q: codec::UnverifiedPublicationQuery) -> Vec<u8> {
    codec::encode_publication_response_http(&codec::UnverifiedPublicationObservation::Current {
        query: Box::new(q),
        home: "home.test".into(),
        service_home: "foreign.test".into(),
        service_certificate_sha256: "55".repeat(32),
        recipient_certificate_sha256: "66".repeat(32),
        generation: 1,
        reset: 0,
        checked_at: wall_now(),
    })
    .unwrap()
}
#[test]
fn actual_outbound_tls_refuses_ca_name_pin_protocol_and_wrong_client_key() {
    use std::io::Read;
    for variant in 0..5 {
        let c = Certs::new();
        let other = Certs::new();
        let a = sender_home(&other);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let ca = if variant == 0 { &other.ca.0 } else { &c.ca.0 };
        let credentials = ClientCredentials::from_pem(
            &ca.to_pem().unwrap(),
            &c.alice.0.to_pem().unwrap(),
            &c.alice.1.private_key_to_pem_pkcs8().unwrap(),
        )
        .unwrap();
        let ep = HttpEndpoint::new(
            addr,
            "home.test".into(),
            if variant == 1 {
                "wrong.test"
            } else {
                "provider.test"
            }
            .into(),
            if variant == 2 {
                [99; 32]
            } else {
                sha256(&c.server.0.to_der().unwrap())
            },
            credentials,
        )
        .unwrap();
        if variant == 4 {
            assert!(
                ClientCredentials::from_pem(
                    &c.ca.0.to_pem().unwrap(),
                    &c.alice.0.to_pem().unwrap(),
                    &c.bob.1.private_key_to_pem_pkcs8().unwrap()
                )
                .is_err()
            );
            assert!(
                ClientCredentials::from_pem(
                    &c.ca.0.to_pem().unwrap(),
                    &c.alice.0.to_pem().unwrap(),
                    &[]
                )
                .is_err()
            );
            continue;
        }
        thread::scope(|s| {
            let server = s.spawn(|| {
                let acceptor = fake_acceptor(&c, variant == 3);
                let (socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                if let Ok(mut peer) = acceptor.accept(socket) {
                    let mut bytes = [0; 8192];
                    let _ = peer.read(&mut bytes);
                }
            });
            assert!(
                a.control()
                    .query_publication(
                        &ep,
                        &query(codec::UnverifiedPublicationAction::Begin),
                        Instant::now() + CYCLE_LIMIT,
                        Arc::new(AtomicBool::new(false))
                    )
                    .is_err()
            );
            server.join().unwrap();
        });
    }
}
#[test]
fn actual_outbound_tls_refuses_malformed_partial_oversized_surplus_redirect_and_substituted_response()
 {
    use std::io::Read;
    let c = Certs::new();
    let a_c = Certs::new();
    let a = sender_home(&a_c);
    let q = query(codec::UnverifiedPublicationAction::Begin);
    let good = current_reply(q.clone());
    let text = String::from_utf8(good.clone()).unwrap();
    let mut substitutions = Vec::new();
    for field in [
        "\"id\":",
        "\"identity\":",
        "\"client_id\":",
        "\"device\":",
        "\"name\":",
    ] {
        let mut v = q.clone();
        match field {
            "\"id\":" => v.binding.id = "aa".repeat(32),
            "\"identity\":" => v.binding.recipient.identity = "bb".repeat(32),
            "\"client_id\":" => v.client_id = "other".into(),
            "\"device\":" => v.binding.recipient.device += 1,
            _ => v.action = codec::UnverifiedPublicationAction::Commit,
        };
        substitutions.push(current_reply(v));
    }
    for reply in [
        text.replace("200 OK", "302 Found").into_bytes(),
        text.replace(
            "Content-Length:",
            "Transfer-Encoding: chunked\r\nContent-Length:",
        )
        .into_bytes(),
        text.replace("Content-Length:", "Content-Length: 8193\r\nOther:")
            .into_bytes(),
        good[..good.len() - 1].to_vec(),
        [good.clone(), b"x".to_vec()].concat(),
        b"HTTP/1.1 200 OK\r\n".to_vec(),
    ]
    .into_iter()
    .chain(substitutions)
    {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let ep = endpoint(&c, listener.local_addr().unwrap());
        thread::scope(|s| {
            let server = s.spawn(|| {
                let acceptor = fake_acceptor(&c, false);
                let (socket, _) = listener.accept().unwrap();
                let mut peer = acceptor.accept(socket).unwrap();
                let mut scratch = [0; 16384];
                assert!(peer.read(&mut scratch).unwrap() > 0);
                let _ = peer.write_all(&reply);
            });
            assert!(
                a.control()
                    .query_publication(
                        &ep,
                        &q,
                        Instant::now() + CYCLE_LIMIT,
                        Arc::new(AtomicBool::new(false))
                    )
                    .is_err()
            );
            server.join().unwrap();
        });
    }
}
