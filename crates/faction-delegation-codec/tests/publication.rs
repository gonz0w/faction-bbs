use faction_delegation_codec::*;
fn query(action: UnverifiedPublicationAction) -> UnverifiedPublicationQuery {
    UnverifiedPublicationQuery {
        client_id: "sender-observer".into(),
        binding: UnverifiedBinding {
            sender: UnverifiedPrincipal {
                account: "sender@sender.test".into(),
                device: 3,
                identity: "11".repeat(32),
            },
            recipient: UnverifiedPrincipal {
                account: "recipient@recipient.test".into(),
                device: 7,
                identity: "22".repeat(32),
            },
            id: "33".repeat(32),
            size: 32,
            created: 100,
            expires: 200,
        },
        action,
    }
}
#[test]
fn exact_publication_selectors_and_complete_binding_round_trip_separately_from_tokens() {
    for action in [
        UnverifiedPublicationAction::Begin,
        UnverifiedPublicationAction::PutChunk {
            offset: 2,
            length: 30,
            ciphertext_sha256: "44".repeat(32),
        },
        UnverifiedPublicationAction::Commit,
        UnverifiedPublicationAction::Status,
        UnverifiedPublicationAction::Delete,
    ] {
        let q = query(action);
        let bytes = encode_publication_query_http("recipient.test", &q).unwrap();
        let head_end = bytes.windows(4).position(|x| x == b"\r\n\r\n").unwrap() + 4;
        assert_eq!(
            decode_http_request_head(&bytes[..head_end])
                .unwrap()
                .endpoint,
            UnverifiedEndpoint::PublicationCurrent
        );
        assert!(decode_introspection_http(&bytes).is_err());
        let decoded = decode_publication_query_http(&bytes).unwrap();
        assert_eq!(q, decoded);
        let observation = UnverifiedPublicationObservation::Current {
            query: Box::new(q),
            home: "recipient.test".into(),
            service_home: "sender.test".into(),
            service_certificate_sha256: "55".repeat(32),
            recipient_certificate_sha256: "66".repeat(32),
            generation: 3,
            reset: 4,
            checked_at: 150,
        };
        let encoded = encode_publication_response_http(&observation).unwrap();
        assert_eq!(
            observation,
            decode_publication_response_http(&encoded).unwrap()
        );
        assert!(decode_introspection_response_http(&encoded).is_err());
    }
    let refused =
        encode_publication_response_http(&UnverifiedPublicationObservation::Unavailable).unwrap();
    assert!(
        String::from_utf8(refused.clone())
            .unwrap()
            .ends_with("{\"current\":false}")
    );
    assert_eq!(
        decode_publication_response_http(&refused).unwrap(),
        UnverifiedPublicationObservation::Unavailable
    );
}
#[test]
fn publication_schema_refuses_ambiguous_unknown_generic_and_out_of_bounds_data() {
    let q = query(UnverifiedPublicationAction::PutChunk {
        offset: 2,
        length: 30,
        ciphertext_sha256: "44".repeat(32),
    });
    let bytes = encode_publication_query_json(&q).unwrap();
    let raw = String::from_utf8(bytes).unwrap();
    for bad in [
        raw.replacen('{', "{\"client_id\":\"duplicate\",", 1),
        raw.replacen('{', "{\"unknown\":1,", 1),
        raw.replace("put_chunk", "publish"),
        raw.replace("\"offset\":2", "\"offset\":3"),
        raw.replace(&"44".repeat(32), "00"),
        raw.replace("\"length\":30", "\"length\":0"),
    ] {
        assert!(
            decode_publication_query_json(bad.as_bytes()).is_err(),
            "accepted {bad}"
        );
    }
    assert!(decode_publication_query_json(&vec![b' '; MAX_JSON + 1]).is_err());
    let mut q = query(UnverifiedPublicationAction::Begin);
    q.client_id = "legacy observation!".into();
    assert!(encode_publication_query_json(&q).is_err());
    let mut bytes =
        encode_publication_query_http("recipient.test", &query(UnverifiedPublicationAction::Begin))
            .unwrap();
    bytes.push(b'x');
    assert!(decode_publication_query_http(&bytes).is_err());
    let raw = String::from_utf8(bytes[..bytes.len() - 1].to_vec()).unwrap();
    assert!(
        decode_publication_query_http(
            raw.replace(
                "Content-Length:",
                "Transfer-Encoding: chunked\r\nContent-Length:"
            )
            .as_bytes()
        )
        .is_err()
    );
}
#[test]
fn publication_chunk_bound_matches_the_existing_4096_byte_store_selector() {
    let mut q = query(UnverifiedPublicationAction::PutChunk {
        offset: 0,
        length: 4096,
        ciphertext_sha256: "44".repeat(32),
    });
    q.binding.size = 10000;
    assert!(encode_publication_query_json(&q).is_ok());
    q.action = UnverifiedPublicationAction::PutChunk {
        offset: 0,
        length: 4097,
        ciphertext_sha256: "44".repeat(32),
    };
    assert!(encode_publication_query_json(&q).is_err());
}
