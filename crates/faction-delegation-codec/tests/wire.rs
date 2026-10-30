use faction_delegation_codec::*;

const REFERENCE: &str = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE";
const TOKEN_FORM: &[u8] = b"grant_type=client_credentials&client_id=relay-1&scope=AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE";
const INTROSPECTION_FORM: &[u8] =
    b"token=secret%2B%2F%3D&token_type_hint=access_token&client_id=sender-1";
const TOKEN_JSON: &[u8] =
    b"{\"access_token\":\"secret+/=\",\"expires_in\":10,\"token_type\":\"Bearer\"}";
const ACTIVE: &str = r#"{"active":true,"iss":"recipient.example","sub":"bob@recipient.example","client_id":"relay-1","aud":"sender.example/attachments","iat":100,"exp":110,"cnf":{"x5t#S256":"AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE"},"faction_purpose":"retrieval","faction_issuer_home":"recipient.example","faction_generation":1,"faction_reset":0,"faction_binding":{"sender":{"account":"alice@sender.example","device":1,"identity":"0101010101010101010101010101010101010101010101010101010101010101"},"recipient":{"account":"bob@recipient.example","device":2,"identity":"0202020202020202020202020202020202020202020202020202020202020202"},"id":"0303030303030303030303030303030303030303030303030303030303030303","size":4112,"created":90,"expires":200},"faction_action":"get_chunk","faction_offset":4096}"#;

fn request() -> UnverifiedTokenRequest {
    UnverifiedTokenRequest {
        client_id: "relay-1".into(),
        scope: UnverifiedReference::parse(REFERENCE).unwrap(),
    }
}
fn introspection() -> UnverifiedIntrospectionRequest {
    UnverifiedIntrospectionRequest {
        client_id: "sender-1".into(),
        token: UnverifiedToken::parse("secret+/=").unwrap(),
    }
}
fn http_request(route: &str, body: &[u8]) -> Vec<u8> {
    let mut bytes = format!("POST {route} HTTP/1.1\r\nHost: recipient.example\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
    bytes.extend_from_slice(body);
    bytes
}
fn http_response(body: &[u8]) -> Vec<u8> {
    let mut bytes = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nCache-Control: no-store\r\nPragma: no-cache\r\n\r\n", body.len()).into_bytes();
    bytes.extend_from_slice(body);
    bytes
}

#[test]
fn fixed_standard_forms_and_application_scope_round_trip() {
    assert_eq!(encode_token_form(&request()).unwrap(), TOKEN_FORM);
    let parsed = decode_token_form(TOKEN_FORM).unwrap();
    assert_eq!(parsed.client_id, "relay-1");
    assert_eq!(parsed.scope.encoded(), REFERENCE);
    assert_eq!(
        encode_introspection_form(&introspection()).unwrap(),
        INTROSPECTION_FORM
    );
    let parsed = decode_introspection_form(INTROSPECTION_FORM).unwrap();
    assert_eq!(parsed.token.expose(), "secret+/=");
    assert_eq!(parsed.client_id, "sender-1");
}

#[test]
fn forms_refuse_ambiguous_or_injected_parameters() {
    let valid = std::str::from_utf8(TOKEN_FORM).unwrap();
    // Preserve the valid reference and unrelated fields in each mutation.
    for body in [
        format!("{valid}&%73cope={REFERENCE}"),
        valid.replace(REFERENCE, "%"),
        valid.replace(REFERENCE, "%0g"),
        valid.replace("relay-1", "%FF"),
        valid.replace("relay-1", "%ED%A0%80"),
        valid.replace("client_credentials", "password"),
        format!("{valid}&account=bob"),
        format!("{valid}&"),
        valid.replace("&client_id", "&&client_id"),
        format!("{valid}=extra"),
        valid.replace("relay-1", "relay%0A1"),
        format!("{valid}&%63lient_id=relay-1"),
        format!("{valid}&grant_type=client_credentials"),
    ] {
        assert!(decode_token_form(body.as_bytes()).is_err());
    }
    let valid = std::str::from_utf8(INTROSPECTION_FORM).unwrap();
    for body in [
        format!("{valid}&%74oken=secret%2B%2F%3D"),
        valid.replace("access_token", "refresh_token"),
        valid.replace("&token_type_hint=access_token", ""),
        valid.replace("secret%2B%2F%3D", ""),
        valid.replace("secret%2B%2F%3D", "%FF"),
        format!("{valid}&%63lient_id=sender-1"),
    ] {
        assert!(decode_introspection_form(body.as_bytes()).is_err());
    }
}

#[test]
fn secret_and_reference_bounds_are_canonical() {
    assert!(UnverifiedToken::parse(&"a".repeat(256)).is_ok());
    for token in [
        "".into(),
        "a".repeat(257),
        "secret\n".into(),
        "sec ret".into(),
        "☃".into(),
        "a=b".into(),
    ] {
        assert!(UnverifiedToken::parse(&token).is_err());
    }
    for value in [
        format!("{REFERENCE}="),
        "AA".into(),
        format!("{}F", &REFERENCE[..42]),
        "-".repeat(43),
    ] {
        assert!(UnverifiedReference::parse(&value).is_err());
    }
    assert!(decode_token_form(&vec![b'a'; 8193]).is_err());
    assert!(decode_introspection_form(&vec![b'a'; 8193]).is_err());
}

#[test]
fn token_response_fixed_json_round_trip() {
    let parsed = decode_token_json(TOKEN_JSON).unwrap();
    assert_eq!(parsed.token.expose(), "secret+/=");
    assert_eq!(parsed.expires_in, 10);
    assert_eq!(encode_token_json(&parsed).unwrap(), TOKEN_JSON);
    for body in [
        r#"{"access_token":"one","access_token":"two","token_type":"Bearer","expires_in":1}"#,
        r#"{"access_token":"one","token_type":"Bearer","expires_in":1.0}"#,
        r#"{"access_token":"one","token_type":"Bearer","expires_in":11}"#,
        r#"{"access_token":"one","token_type":"Bearer","expires_in":0}"#,
        r#"{"access_token":"one","token_type":"Basic","expires_in":1}"#,
        r#"{"access_token":"one","token_type":"Bearer","expires_in":1,"refresh_token":"x"}"#,
    ] {
        assert!(decode_token_json(body.as_bytes()).is_err());
    }
}

#[test]
fn complete_active_retrieval_and_service_observation_are_distinct() {
    let parsed = decode_introspection_json(ACTIVE.as_bytes()).unwrap();
    match &parsed {
        UnverifiedIntrospection::Retrieval {
            claims,
            action: UnverifiedRetrievalAction::GetChunk { offset },
        } => {
            assert_eq!(*offset, 4096);
            assert_eq!(claims.binding.recipient.account, "bob@recipient.example");
        }
        _ => panic!("wrong unverified purpose"),
    }
    let encoded = encode_introspection_json(&parsed).unwrap();
    assert!(matches!(
        decode_introspection_json(&encoded).unwrap(),
        UnverifiedIntrospection::Retrieval { .. }
    ));
    let service = ACTIVE
        .replace("\"sub\":\"bob@recipient.example\"", "\"sub\":\"sender-1\"")
        .replace("\"client_id\":\"relay-1\"", "\"client_id\":\"sender-1\"")
        .replace("\"retrieval\"", "\"recipient_current\"")
        .replace("\"get_chunk\",\"faction_offset\":4096", "\"publish\"");
    let parsed = decode_introspection_json(service.as_bytes()).unwrap();
    assert!(matches!(
        parsed,
        UnverifiedIntrospection::RecipientCurrent {
            action: UnverifiedObservationAction::Publish,
            ..
        }
    ));
    assert!(
        decode_introspection_json(
            service
                .replace("\"sub\":\"sender-1\"", "\"sub\":\"bob@recipient.example\"")
                .as_bytes()
        )
        .is_err()
    );
    assert!(
        decode_introspection_json(service.replace("\"publish\"", "\"get_chunk\"").as_bytes())
            .is_err()
    );
    assert!(matches!(
        decode_introspection_json(br#"{"active":false}"#).unwrap(),
        UnverifiedIntrospection::Inactive
    ));
    assert_eq!(
        encode_introspection_json(&UnverifiedIntrospection::Inactive).unwrap(),
        br#"{"active":false}"#
    );
    assert!(decode_introspection_json(br#"{"active":false,"sub":"private-member"}"#).is_err());
}

#[test]
fn json_rejects_duplicates_in_every_nested_object_and_escape_aliases() {
    for (old, new) in [
        ("\"active\":true", "\"active\":true,\"\\u0061ctive\":true"),
        (
            "\"cnf\":{",
            "\"cnf\":{\"x5t#S256\":\"AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE\",",
        ),
        ("\"sender\":{", "\"sender\":{\"device\":1,"),
        ("\"size\":4112", "\"size\":4112,\"size\":4112"),
    ] {
        assert!(decode_introspection_json(ACTIVE.replace(old, new).as_bytes()).is_err());
    }
    assert!(decode_introspection_json(br#"{"active":true,"active":false}"#).is_err());
}

#[test]
fn complete_syntax_and_numeric_invariants_fail_closed() {
    for (old, new) in [
        ("\"retrieval\"", "\"unknown\""),
        ("\"get_chunk\"", "\"delete\""),
        ("\"device\":2", "\"device\":0"),
        ("\"device\":2", "\"device\":4294967296"),
        ("\"device\":2", "\"device\":2.0"),
        ("\"device\":2", "\"device\":-2"),
        ("\"exp\":110", "\"exp\":111"),
        ("\"exp\":110", "\"exp\":99"),
        ("\"iat\":100", "\"iat\":18446744073709551616"),
        ("\"iat\":100", "\"iat\":1e2"),
        ("\"size\":4112", "\"size\":15"),
        ("\"size\":4112", "\"size\":1048593"),
        ("\"faction_offset\":4096", "\"faction_offset\":4113"),
        ("bob@recipient.example", "bob@recipient"),
        ("bob@recipient.example", "Bob@recipient.example"),
        ("\"faction_generation\":1", "\"faction_generation\":0"),
        ("\"created\":90", "\"created\":201"),
        (
            "\"iss\":\"recipient.example\"",
            "\"iss\":\"https://evil.example/path\"",
        ),
        (
            "\"aud\":\"sender.example/attachments\"",
            "\"aud\":\"https://sender.example/attachments\"",
        ),
        (
            "\"sub\":\"bob@recipient.example\"",
            "\"sub\":\"alice@sender.example\"",
        ),
    ] {
        assert!(
            decode_introspection_json(ACTIVE.replace(old, new).as_bytes()).is_err(),
            "accepted mutation {new}"
        );
    }
    for field in [
        "active",
        "iss",
        "sub",
        "client_id",
        "aud",
        "iat",
        "exp",
        "cnf",
        "faction_purpose",
        "faction_issuer_home",
        "faction_generation",
        "faction_reset",
        "faction_binding",
        "faction_action",
        "faction_offset",
    ] {
        let mut value: serde_json::Value = serde_json::from_str(ACTIVE).unwrap();
        value.as_object_mut().unwrap().remove(field);
        assert!(decode_introspection_json(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    for hex in [
        "00".repeat(32),
        "AB".repeat(32),
        "02".repeat(31),
        "gg".repeat(32),
    ] {
        assert!(
            decode_introspection_json(ACTIVE.replace(&"02".repeat(32), &hex).as_bytes()).is_err()
        );
    }
    assert!(
        decode_introspection_json(
            ACTIVE
                .replace(REFERENCE, &(REFERENCE.to_owned() + "="))
                .as_bytes()
        )
        .is_err()
    );
}

#[test]
fn json_bounds_depth_types_and_unicode_are_strict() {
    for body in [
        "[]".into(),
        "null".into(),
        "true".into(),
        "{\"active\":false} {}".into(),
        "[".repeat(129) + &"]".repeat(129),
        "{\"active\":false,\"extra\":".to_owned() + &"[".repeat(9) + "0" + &"]".repeat(9) + "}",
        " ".repeat(8193),
        "{\"active\":\"false\"}".into(),
        "{\"active\":false,\"x\":\"\\uD800\"}".into(),
    ] {
        assert!(decode_introspection_json(body.as_bytes()).is_err());
    }
    assert!(decode_introspection_json(b"{\"active\":false,\"x\":\"\xff\"}").is_err());
    let mut exact = b"{\"active\":false}".to_vec();
    exact.resize(8192, b' ');
    assert!(decode_introspection_json(&exact).is_ok());
    exact.push(b' ');
    assert!(decode_introspection_json(&exact).is_err());
}

#[test]
fn fixed_http_requests_and_responses_are_exact() {
    let expected = with_connection_close(http_request("/oauth/token", TOKEN_FORM));
    assert_eq!(
        encode_token_http("recipient.example", &request()).unwrap(),
        expected
    );
    assert_eq!(decode_token_http(&expected).unwrap().client_id, "relay-1");
    let expected = with_connection_close(http_request("/oauth/introspect", INTROSPECTION_FORM));
    assert_eq!(
        encode_introspection_http("recipient.example", &introspection()).unwrap(),
        expected
    );
    assert_eq!(
        decode_introspection_http(&expected).unwrap().token.expose(),
        "secret+/="
    );
    let token = decode_token_json(TOKEN_JSON).unwrap();
    assert_eq!(
        encode_token_response_http(&token).unwrap(),
        with_connection_close(http_response(TOKEN_JSON))
    );
    assert_eq!(
        decode_token_response_http(&http_response(TOKEN_JSON))
            .unwrap()
            .expires_in,
        10
    );
    let inactive = UnverifiedIntrospection::Inactive;
    assert_eq!(
        encode_introspection_response_http(&inactive).unwrap(),
        with_connection_close(http_response(br#"{"active":false}"#))
    );
    assert!(matches!(
        decode_introspection_response_http(&http_response(br#"{"active":false}"#)).unwrap(),
        UnverifiedIntrospection::Inactive
    ));
}

#[test]
fn http_refuses_framing_and_protocol_ambiguity() {
    let request = String::from_utf8(http_request("/oauth/token", TOKEN_FORM)).unwrap();
    for (old, new) in [
        ("POST ", "GET "),
        ("/oauth/token ", "/oauth/token?x=1 "),
        ("HTTP/1.1", "HTTP/1.0"),
        (
            "Content-Type: application/x-www-form-urlencoded",
            "Content-Type: text/plain",
        ),
        (
            "Content-Type: application/x-www-form-urlencoded",
            "Content-Type: application/x-www-form-urlencoded; charset=utf-8",
        ),
        ("Host: recipient.example\r\n", ""),
        (
            "Host: recipient.example\r\n",
            "Host: recipient.example\r\nHost: other.example\r\n",
        ),
        (
            "Content-Length: ",
            "Transfer-Encoding: chunked\r\nContent-Length: ",
        ),
        (
            "Content-Length: ",
            "Content-Encoding: gzip\r\nContent-Length: ",
        ),
        ("Content-Length: ", "Content-Length: 0\r\nContent-Length: "),
        ("Content-Length: ", "Content-Length: +"),
        ("Content-Length: ", "Content-Length: 0"),
        ("Content-Length: ", "Content-Length: 184467440737095516160"),
        (
            "Host: recipient.example",
            "Host: recipient.example\r\n X: folded",
        ),
    ] {
        assert!(
            decode_token_http(request.replace(old, new).as_bytes()).is_err(),
            "accepted {new}"
        );
    }
    for suffix in [b"\r\n".as_slice(), b"GET / HTTP/1.1\r\n\r\n"] {
        let mut bad = request.as_bytes().to_vec();
        bad.extend_from_slice(suffix);
        assert!(decode_token_http(&bad).is_err());
    }
    for end in [
        0,
        3,
        request.find("\r\n\r\n").unwrap() + 3,
        request.len() - 1,
    ] {
        assert!(decode_token_http(&request.as_bytes()[..end]).is_err());
    }
    for body in [b"HTTP/1.1 302 Found\r\nLocation: https://other.example/\r\nContent-Type: application/json\r\nContent-Length: 0\r\n\r\n".as_slice(),
        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 16\r\nContent-Encoding: gzip\r\n\r\n{\"active\":false}",
        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 16\r\nTransfer-Encoding: identity\r\n\r\n{\"active\":false}"] {
        assert!(decode_introspection_response_http(body).is_err());
    }
}

#[test]
fn independent_http_header_and_body_bounds_and_field_count() {
    let request = http_request("/oauth/token", TOKEN_FORM);
    let text = String::from_utf8(request).unwrap();
    let marker = "Content-Length:";
    let boundary = text.replace(
        marker,
        &format!(
            "X-Pad: {}\r\n{marker}",
            "x".repeat(8192 - text.find("\r\n\r\n").unwrap() - 4 - 9)
        ),
    );
    assert_eq!(boundary.find("\r\n\r\n").unwrap() + 4, 8192);
    assert!(decode_token_http(boundary.as_bytes()).is_ok());
    assert!(decode_token_http(boundary.replace("X-Pad: ", "X-Pad: x").as_bytes()).is_err());
    let headers = (0..16).map(|i| format!("X-{i}: x\r\n")).collect::<String>();
    assert!(decode_token_http(text.replace(marker, &(headers + marker)).as_bytes()).is_err());
    assert!(decode_token_http(&http_request("/oauth/token", &vec![b'x'; 8193])).is_err());
    assert!(decode_token_response_http(&http_response(&vec![b'x'; 8193])).is_err());
}

#[test]
fn refusal_responses_are_fixed_and_nonreflecting() {
    for refusal in [
        UnverifiedRefusal::InvalidRequest,
        UnverifiedRefusal::InvalidToken,
        UnverifiedRefusal::Unavailable,
    ] {
        let bytes = encode_refusal_http(refusal);
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("secret"));
        assert!(text.contains("Cache-Control: no-store\r\n"));
        assert!(!text.contains("error_description"));
    }
    assert_eq!(
        format!("{}", decode_token_form(b"token=secret").err().unwrap()),
        "invalid delegation wire data"
    );
}

#[test]
fn all_purpose_specific_actions_and_encoder_mutations() {
    let status = ACTIVE.replace("\"get_chunk\",\"faction_offset\":4096", "\"status\"");
    assert!(matches!(
        decode_introspection_json(status.as_bytes()).unwrap(),
        UnverifiedIntrospection::Retrieval {
            action: UnverifiedRetrievalAction::Status,
            ..
        }
    ));
    assert!(
        decode_introspection_json(ACTIVE.replace("\"get_chunk\"", "\"status\"").as_bytes())
            .is_err()
    );
    for action in ["publish", "status", "delete"] {
        let service = ACTIVE
            .replace("\"sub\":\"bob@recipient.example\"", "\"sub\":\"sender-1\"")
            .replace("\"client_id\":\"relay-1\"", "\"client_id\":\"sender-1\"")
            .replace("\"retrieval\"", "\"recipient_current\"")
            .replace(
                "\"get_chunk\",\"faction_offset\":4096",
                &format!("\"{action}\""),
            );
        let parsed = decode_introspection_json(service.as_bytes()).unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(
                &encode_introspection_json(&parsed).unwrap()
            )
            .unwrap(),
            serde_json::from_str::<serde_json::Value>(&service).unwrap()
        );
    }
    let mut parsed = decode_introspection_json(ACTIVE.as_bytes()).unwrap();
    if let UnverifiedIntrospection::Retrieval { claims, .. } = &mut parsed {
        claims.binding.recipient.device = 0;
    }
    assert!(encode_introspection_json(&parsed).is_err());
    let mut parsed = decode_introspection_json(ACTIVE.as_bytes()).unwrap();
    if let UnverifiedIntrospection::Retrieval { action, .. } = &mut parsed {
        *action = UnverifiedRetrievalAction::GetChunk { offset: 4113 };
    }
    assert!(encode_introspection_json(&parsed).is_err());
    let mut parsed = decode_token_json(TOKEN_JSON).unwrap();
    parsed.expires_in = 11;
    assert!(encode_token_json(&parsed).is_err());
    let mut parsed = request();
    parsed.client_id = "x".repeat(65);
    assert!(encode_token_form(&parsed).is_err());
    assert!(encode_token_http("recipient.example\r\nInjected: yes", &request()).is_err());
    let unknown = ACTIVE.replace("\"cnf\":{", "\"cnf\":{\"unknown\":0,");
    assert!(decode_introspection_json(unknown.as_bytes()).is_err());
    for replacement in [
        "\"size\":null",
        "\"size\":[]",
        "\"size\":\"4112\"",
        "\"size\":true",
    ] {
        assert!(
            decode_introspection_json(ACTIVE.replace("\"size\":4112", replacement).as_bytes())
                .is_err()
        );
    }
}

#[test]
fn canonical_active_output_matches_independent_fixed_bytes() {
    let canonical = r#"{"active":true,"aud":"sender.example/attachments","client_id":"relay-1","cnf":{"x5t#S256":"AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE"},"exp":110,"faction_action":"get_chunk","faction_binding":{"created":90,"expires":200,"id":"0303030303030303030303030303030303030303030303030303030303030303","recipient":{"account":"bob@recipient.example","device":2,"identity":"0202020202020202020202020202020202020202020202020202020202020202"},"sender":{"account":"alice@sender.example","device":1,"identity":"0101010101010101010101010101010101010101010101010101010101010101"},"size":4112},"faction_generation":1,"faction_issuer_home":"recipient.example","faction_offset":4096,"faction_purpose":"retrieval","faction_reset":0,"iat":100,"iss":"recipient.example","sub":"bob@recipient.example"}"#;
    assert_eq!(
        encode_introspection_json(&decode_introspection_json(ACTIVE.as_bytes()).unwrap()).unwrap(),
        canonical.as_bytes()
    );
}

#[test]
fn http_rejects_bare_line_feeds_and_leading_empty_lines() {
    let request = String::from_utf8(http_request("/oauth/token", TOKEN_FORM)).unwrap();
    let response = String::from_utf8(http_response(TOKEN_JSON)).unwrap();
    for bytes in [
        request
            .replace("Host: recipient.example\r\n", "Host: recipient.example\n")
            .into_bytes(),
        format!("\r\n{request}").into_bytes(),
        format!("\n{request}").into_bytes(),
        request
            .replacen("HTTP/1.1\r\n", "HTTP/1.1\n", 1)
            .into_bytes(),
    ] {
        assert!(decode_token_http(&bytes).is_err());
    }
    for bytes in [
        response
            .replace(
                "Content-Type: application/json\r\n",
                "Content-Type: application/json\n",
            )
            .into_bytes(),
        format!("\r\n{response}").into_bytes(),
        format!("\n{response}").into_bytes(),
        response.replacen("200 OK\r\n", "200 OK\n", 1).into_bytes(),
    ] {
        assert!(decode_token_response_http(&bytes).is_err());
    }
}
#[test]
fn retrieval_eof_offset_is_preserved_by_decoder() {
    let eof = ACTIVE.replace("\"faction_offset\":4096", "\"faction_offset\":4112");
    let parsed = decode_introspection_json(eof.as_bytes()).unwrap();
    assert!(matches!(
        parsed,
        UnverifiedIntrospection::Retrieval {
            action: UnverifiedRetrievalAction::GetChunk { offset: 4112 },
            ..
        }
    ));
}
#[test]
fn retrieval_eof_offset_is_preserved_by_encoder() {
    let mut parsed = decode_introspection_json(ACTIVE.as_bytes()).unwrap();
    if let UnverifiedIntrospection::Retrieval { action, .. } = &mut parsed {
        *action = UnverifiedRetrievalAction::GetChunk { offset: 4112 };
    } else {
        panic!();
    }
    let encoded = encode_introspection_json(&parsed).unwrap();
    let parsed = decode_introspection_json(&encoded).unwrap();
    assert!(matches!(
        parsed,
        UnverifiedIntrospection::Retrieval {
            action: UnverifiedRetrievalAction::GetChunk { offset: 4112 },
            ..
        }
    ));
}
#[test]
fn explicit_connection_close_is_supported_without_other_connection_modes() {
    let original = String::from_utf8(http_request("/oauth/token", TOKEN_FORM)).unwrap();
    let close = original.replace("\r\nHost:", "\r\nConnection: close\r\nHost:");
    assert!(decode_token_http(close.as_bytes()).is_ok());
    for value in ["keep-alive", "upgrade", "close, keep-alive", "Close", ""] {
        assert!(
            decode_token_http(
                close
                    .replace("Connection: close", &format!("Connection: {value}"))
                    .as_bytes()
            )
            .is_err()
        );
    }
    assert!(
        decode_token_http(
            close
                .replace(
                    "Connection: close",
                    "Connection: close\r\nconnection: close"
                )
                .as_bytes()
        )
        .is_err()
    );
}
#[test]
fn encoders_explicitly_close_the_single_operation_carrier() {
    let request = encode_token_http("recipient.example", &request()).unwrap();
    assert!(
        request
            .windows(b"\r\nConnection: close\r\n".len())
            .any(|w| w == b"\r\nConnection: close\r\n")
    );
    let response = encode_refusal_http(UnverifiedRefusal::InvalidRequest);
    assert!(
        response
            .windows(b"\r\nConnection: close\r\n".len())
            .any(|w| w == b"\r\nConnection: close\r\n")
    );
}
#[test]
fn canonical_unverified_bytes_helpers_reuse_the_qualified_base64_codec() {
    let bytes = [23; 32];
    let reference = UnverifiedReference::from_bytes(bytes);
    assert_eq!(reference.expose_bytes(), &bytes);
    assert_eq!(
        UnverifiedReference::parse(&reference.encoded())
            .unwrap()
            .expose_bytes(),
        &bytes
    );
    let token = UnverifiedToken::from_bytes(bytes);
    assert_eq!(token.decode_bytes().unwrap(), bytes);
    assert!(
        UnverifiedToken::parse("secret+/=")
            .unwrap()
            .decode_bytes()
            .is_err()
    );
}
#[test]
fn unverified_head_bounds_content_length_and_preserves_exact_host_and_endpoint() {
    let full = http_request("/oauth/token", TOKEN_FORM);
    let end = full.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
    let head = decode_http_request_head(&full[..end]).unwrap();
    assert_eq!(head.host, "recipient.example");
    assert_eq!(head.content_length, TOKEN_FORM.len());
    assert_eq!(head.endpoint, UnverifiedEndpoint::Token);
    assert!(decode_http_request_head(&full).is_err());
    assert!(decode_http_request_head(&full[..end - 1]).is_err());
}

fn with_connection_close(message: Vec<u8>) -> Vec<u8> {
    let text = String::from_utf8(message).unwrap();
    text.replacen("\r\n", "\r\nConnection: close\r\n", 1)
        .into_bytes()
}
