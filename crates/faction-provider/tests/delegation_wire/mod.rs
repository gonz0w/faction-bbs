use super::*;
use faction_delegation_codec as codec;
use openssl::ssl::{SslConnector, SslMethod, SslStream, SslVerifyMode, SslVersion};
use std::io::Read;
fn peer(c: &Certs, who: &(X509, PKey<Private>), address: SocketAddr) -> SslStream<TcpStream> {
    let mut b = SslConnector::builder(SslMethod::tls_client()).unwrap();
    b.set_min_proto_version(Some(SslVersion::TLS1_3)).unwrap();
    b.set_max_proto_version(Some(SslVersion::TLS1_3)).unwrap();
    b.set_verify(SslVerifyMode::PEER);
    b.cert_store_mut().add_cert(c.ca.0.clone()).unwrap();
    b.set_certificate(&who.0).unwrap();
    b.set_private_key(&who.1).unwrap();
    let socket = TcpStream::connect(address).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(4)))
        .unwrap();
    socket
        .set_write_timeout(Some(Duration::from_secs(4)))
        .unwrap();
    let peer = b.build().connect("provider.test", socket).unwrap();
    assert_eq!(
        sha256(&peer.ssl().peer_certificate().unwrap().to_der().unwrap()),
        sha256(&c.server.0.to_der().unwrap())
    );
    peer
}
fn response(peer: &mut SslStream<TcpStream>) -> Vec<u8> {
    let mut result = Vec::new();
    let mut buffer = [0; 1024];
    loop {
        match peer.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                assert!(result.len() + n <= codec::MAX_HEADER + codec::MAX_JSON);
                result.extend_from_slice(&buffer[..n]);
            }
        }
    }
    result
}
fn exchange(
    c: &Certs,
    r: &Runtime,
    who: &(X509, PKey<Private>),
    bytes: &[u8],
) -> (Result<(), Error>, Vec<u8>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::scope(|scope| {
        let client = scope.spawn(|| {
            let mut peer = peer(c, who, address);
            peer.write_all(bytes).unwrap();
            response(&mut peer)
        });
        let service = c
            .server()
            .accept_service(&listener, r, Arc::new(AtomicBool::new(false)))
            .unwrap();
        let result = service.serve_http_once();
        (result, client.join().unwrap())
    })
}
fn token_request(reference: &OperationReference) -> Vec<u8> {
    codec::encode_token_http(
        "issuer.test",
        &codec::UnverifiedTokenRequest {
            client_id: "relay".into(),
            scope: codec::UnverifiedReference::from_bytes(*reference.expose_bytes()),
        },
    )
    .unwrap()
}
#[test]
fn actual_one_runtime_device_close_token_http_close_and_introspection_http_preserve_eof_and_der_binding()
 {
    let c = Certs::new();
    let t = wall_now() as i64;
    let inspector = cert(Some((&c.ca.0, &c.ca.1)), "inspector", 80, t - 60, t + 3600);
    let r = configured(&c, &inspector.0);
    let expected = object();
    let rec = with_session(&c, &r, &c.bob, |mut s| {
        s.retrieval_intent(
            expected.clone(),
            RetrievalAction::GetChunk {
                offset: expected.size,
            },
        )
        .unwrap()
    });
    let reference = r.control().provision_retrieval(&rec).unwrap();
    let (result, bytes) = exchange(&c, &r, &c.alice, &token_request(&reference));
    assert!(result.is_ok());
    let issued = codec::decode_token_response_http(&bytes).unwrap();
    assert!((1..=10).contains(&issued.expires_in));
    let secret = issued.token.decode_bytes().unwrap();
    let (result, retry) = exchange(&c, &r, &c.alice, &token_request(&reference));
    assert!(result.is_ok());
    let retry = codec::decode_token_response_http(&retry).unwrap();
    assert_eq!(secret, retry.token.decode_bytes().unwrap());
    assert!(retry.expires_in <= issued.expires_in);
    let request = codec::encode_introspection_http(
        "issuer.test",
        &codec::UnverifiedIntrospectionRequest {
            client_id: "inspector".into(),
            token: issued.token,
        },
    )
    .unwrap();
    let (result, bytes) = exchange(&c, &r, &inspector, &request);
    assert!(result.is_ok());
    match codec::decode_introspection_response_http(&bytes).unwrap() {
        codec::UnverifiedIntrospection::Retrieval { claims, action } => {
            assert_eq!(
                action,
                codec::UnverifiedRetrievalAction::GetChunk { offset: 32 }
            );
            assert_eq!(claims.binding.recipient.account, expected.recipient.account);
            assert_eq!(
                claims.binding.recipient.identity,
                expected
                    .recipient
                    .identity
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            );
            assert_eq!(claims.binding.size, expected.size);
            assert_eq!(claims.audience, "foreign.test/attachments");
            assert_eq!(claims.issuer, "issuer.test");
            assert_eq!(
                claims.certificate_thumbprint.expose_bytes(),
                &sha256(&c.alice.0.to_der().unwrap())
            );
        }
        _ => panic!("not retrieval metadata"),
    }
}
fn form(path: &str, host: &str, body: &[u8]) -> Vec<u8> {
    let mut bytes=format!("POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n",body.len()).into_bytes();
    bytes.extend_from_slice(body);
    bytes
}
#[test]
fn actual_http_refuses_wrong_host_path_fields_and_framing_then_releases_the_lane() {
    let c = Certs::new();
    let t = wall_now() as i64;
    let inspector = cert(Some((&c.ca.0, &c.ca.1)), "inspector", 81, t - 60, t + 3600);
    let r = configured(&c, &inspector.0);
    let rec = with_session(&c, &r, &c.bob, |mut s| {
        s.retrieval_intent(object(), RetrievalAction::Status)
            .unwrap()
    });
    let reference = r.control().provision_retrieval(&rec).unwrap();
    let good = token_request(&reference);
    let text = String::from_utf8(good.clone()).unwrap();
    let body = text.split_once("\r\n\r\n").unwrap().1;
    let cases = vec![
        form("/oauth/token", "wrong.test", body.as_bytes()),
        form("/oauth/other", "issuer.test", body.as_bytes()),
        form(
            "/oauth/token",
            "issuer.test",
            format!("{body}&client_id=relay").as_bytes(),
        ),
        form(
            "/oauth/token",
            "issuer.test",
            format!("{body}&subject=other").as_bytes(),
        ),
        form(
            "/oauth/token",
            "issuer.test",
            body.replace("client_id=relay", "client_id=%FF").as_bytes(),
        ),
        form("/oauth/token", "issuer.test", &vec![b'x'; 8193]),
        text.replace("Content-Length:", "Content-Length: 1\r\nContent-Length:")
            .into_bytes(),
        text.replace("Connection: close", "Connection: keep-alive")
            .into_bytes(),
        text.replace(
            "Content-Length:",
            "Transfer-Encoding: chunked\r\nContent-Length:",
        )
        .into_bytes(),
        text.replace("Host:", &format!("X-Pad: {}\r\nHost:", "x".repeat(8192)))
            .into_bytes(),
    ];
    for bytes in cases {
        let (result, response) = exchange(&c, &r, &c.alice, &bytes);
        assert!(result.is_err());
        assert!(response.starts_with(b"HTTP/1.1 400"));
        assert!(!String::from_utf8_lossy(&response).contains("recipient"));
    }
    let (result, response) = exchange(&c, &r, &c.alice, &good);
    assert!(result.is_ok());
    assert!(codec::decode_token_response_http(&response).is_ok());
}
#[test]
fn unknown_unauthorized_and_revoked_introspection_are_identical_inactive_metadata() {
    let c = Certs::new();
    let t = wall_now() as i64;
    let inspector = cert(Some((&c.ca.0, &c.ca.1)), "inspector", 82, t - 60, t + 3600);
    let r = configured(&c, &inspector.0);
    let rec = with_session(&c, &r, &c.bob, |mut s| {
        s.retrieval_intent(object(), RetrievalAction::Status)
            .unwrap()
    });
    let reference = r.control().provision_retrieval(&rec).unwrap();
    let (result, bytes) = exchange(&c, &r, &c.alice, &token_request(&reference));
    assert!(result.is_ok());
    let issued = codec::decode_token_response_http(&bytes).unwrap();
    let raw = issued.token.decode_bytes().unwrap();
    let mut baseline = None;
    for (who, id, token) in [
        (&c.alice, "relay", codec::UnverifiedToken::from_bytes(raw)),
        (
            &inspector,
            "wrong-client",
            codec::UnverifiedToken::from_bytes(raw),
        ),
        (
            &inspector,
            "inspector",
            codec::UnverifiedToken::from_bytes([77; 32]),
        ),
        (
            &inspector,
            "inspector",
            codec::UnverifiedToken::parse("unknown").unwrap(),
        ),
    ] {
        let request = codec::encode_introspection_http(
            "issuer.test",
            &codec::UnverifiedIntrospectionRequest {
                client_id: id.into(),
                token,
            },
        )
        .unwrap();
        let (result, bytes) = exchange(&c, &r, who, &request);
        assert!(result.is_ok());
        assert!(matches!(
            codec::decode_introspection_response_http(&bytes).unwrap(),
            codec::UnverifiedIntrospection::Inactive
        ));
        if let Some(old) = &baseline {
            assert_eq!(old, &bytes);
        } else {
            baseline = Some(bytes);
        }
    }
    let mut entries = registry(&c, 2).bindings().to_vec();
    entries[0].approved = false;
    r.control()
        .replace_delegation_policy(
            Registry::new(
                2,
                0,
                vec!["home.test".into(), "foreign.test".into()],
                entries,
            )
            .unwrap(),
            policy(&c, &inspector.0),
        )
        .unwrap();
    let request = codec::encode_introspection_http(
        "issuer.test",
        &codec::UnverifiedIntrospectionRequest {
            client_id: "inspector".into(),
            token: codec::UnverifiedToken::from_bytes(raw),
        },
    )
    .unwrap();
    let (result, bytes) = exchange(&c, &r, &inspector, &request);
    assert!(result.is_ok());
    assert_eq!(baseline.unwrap(), bytes);
}
#[test]
fn fragmented_actual_tls_request_succeeds_without_eof_and_truncation_never_issues() {
    for partial in [false, true] {
        let c = Certs::new();
        let t = wall_now() as i64;
        let inspector = cert(Some((&c.ca.0, &c.ca.1)), "inspector", 83, t - 60, t + 3600);
        let r = configured(&c, &inspector.0);
        let rec = with_session(&c, &r, &c.bob, |mut s| {
            s.retrieval_intent(object(), RetrievalAction::Status)
                .unwrap()
        });
        let reference = r.control().provision_retrieval(&rec).unwrap();
        let bytes = token_request(&reference);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::scope(|scope| {
            let client = scope.spawn(|| {
                let mut peer = peer(&c, &c.alice, address);
                let end = if partial {
                    bytes.len() - 5
                } else {
                    bytes.len()
                };
                for chunk in bytes[..end].chunks(3) {
                    peer.write_all(chunk).unwrap();
                }
                if partial {
                    peer.get_mut().shutdown(std::net::Shutdown::Write).unwrap();
                }
                response(&mut peer)
            });
            let service = c
                .server()
                .accept_service(&listener, &r, Arc::new(AtomicBool::new(false)))
                .unwrap();
            let result = service.serve_http_once();
            let response = client.join().unwrap();
            if partial {
                assert!(result.is_err());
                assert!(response.is_empty() || response.starts_with(b"HTTP/1.1 400"));
            } else {
                assert!(result.is_ok());
                assert!(codec::decode_token_response_http(&response).is_ok());
            }
        });
    }
}
