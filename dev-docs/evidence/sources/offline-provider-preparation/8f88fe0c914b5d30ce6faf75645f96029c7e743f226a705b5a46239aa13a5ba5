#![cfg(feature = "preparation")]
//! Public inert preparation contract; no file, key, runtime or authority access.
use faction_files::private_blobs::Principal;
use faction_provider::preparation::{Card, MAX_CARD, MAX_PEM, certificate_sha256};
use openssl::{
    asn1::Asn1Time,
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    nid::Nid,
    pkey::PKey,
    sha::sha256,
    x509::{X509, X509NameBuilder},
};

fn principal() -> Principal {
    Principal {
        account: "alice@home.test".into(),
        device: 101,
        identity: [0xab; 32],
    }
}
fn text() -> String {
    format!(
        "version = 1\naccount = \"alice@home.test\"\ndevice = 101\nidentity = \"{}\"\ncertificate_sha256 = \"{}\"\n",
        "ab".repeat(32),
        "cd".repeat(32)
    )
}
#[test]
fn card_roundtrip_contains_exactly_five_public_fields() {
    assert_eq!(MAX_CARD, 8192);
    assert_eq!(MAX_PEM, 65_536);
    let card = Card::new(principal(), [0xcd; 32]).unwrap();
    let encoded = card.encode().unwrap();
    assert!(encoded.len() <= MAX_CARD);
    let fields: toml::Table = std::str::from_utf8(&encoded).unwrap().parse().unwrap();
    assert_eq!(fields.len(), 5);
    assert_eq!(fields["version"].as_integer(), Some(1));
    assert_eq!(fields["account"].as_str(), Some("alice@home.test"));
    assert_eq!(fields["device"].as_integer(), Some(101));
    assert_eq!(fields["identity"].as_str(), Some("ab".repeat(32).as_str()));
    assert_eq!(
        fields["certificate_sha256"].as_str(),
        Some("cd".repeat(32).as_str())
    );
    let decoded = Card::decode(&encoded).unwrap();
    assert_eq!(decoded.principal, principal());
    assert_eq!(decoded.certificate_sha256, [0xcd; 32]);
    assert_eq!(decoded.encode().unwrap(), encoded);
    assert_eq!(
        Card::decode(text().as_bytes()).unwrap().principal,
        principal()
    );
}
#[test]
fn card_refuses_unknown_duplicate_malformed_and_noncanonical_values() {
    let valid = text();
    let cases = [
        String::new(),
        valid.replace("version = 1", "version = 2"),
        valid.replace("version = 1\n", ""),
        valid.replace("device = 101", "device = 0"),
        valid.replace("device = 101", "device = -1"),
        valid.replace("device = 101", "device = 4294967296"),
        valid.replace("device = 101", "device = \"101\""),
        valid.replace("alice@home.test", "Alice@home.test"),
        valid.replace("alice@home.test", "alice@HOME.test"),
        valid.replace("alice@home.test", "alice@-home.test"),
        valid.replace(&"ab".repeat(32), &"AB".repeat(32)),
        valid.replace(&"ab".repeat(32), &"ab".repeat(31)),
        valid.replace(&"ab".repeat(32), &"00".repeat(32)),
        valid.replace(&"cd".repeat(32), &"CD".repeat(32)),
        valid.replace(&"cd".repeat(32), &"00".repeat(32)),
        format!("{valid}endpoint = \"127.0.0.1:9\"\n"),
        format!("{valid}account = \"alice@home.test\"\n"),
        format!("{valid}[transport]\nkey = \"forbidden\"\n"),
        format!("{valid}#{}", "x".repeat(MAX_CARD)),
    ];
    for (index, raw) in cases.iter().enumerate() {
        assert!(
            Card::decode(raw.as_bytes()).is_err(),
            "accepted case {index}"
        );
    }
    assert!(Card::decode(&[0xff]).is_err());
}
#[test]
fn public_mutation_cannot_bypass_encode_validation() {
    let mut card = Card::new(principal(), [1; 32]).unwrap();
    card.principal.device = 0;
    assert!(card.encode().is_err());
    card.principal = principal();
    card.principal.identity = [0; 32];
    assert!(card.encode().is_err());
    card.principal = principal();
    card.principal.account = "Alice@home.test".into();
    assert!(card.encode().is_err());
    card.principal = principal();
    card.certificate_sha256 = [0; 32];
    assert!(card.encode().is_err());
    assert!(Card::new(principal(), [0; 32]).is_err());
    let mut invalid = principal();
    invalid.device = 0;
    assert!(Card::new(invalid, [1; 32]).is_err());
}
fn certificate() -> X509 {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let key = PKey::from_ec_key(EcKey::generate(&group).unwrap()).unwrap();
    let mut name = X509NameBuilder::new().unwrap();
    name.append_entry_by_text("CN", "synthetic-public-leaf")
        .unwrap();
    let name = name.build();
    let mut cert = X509::builder().unwrap();
    cert.set_version(2).unwrap();
    cert.set_subject_name(&name).unwrap();
    cert.set_issuer_name(&name).unwrap();
    cert.set_pubkey(&key).unwrap();
    cert.set_not_before(&Asn1Time::from_unix(1_700_000_000).unwrap())
        .unwrap();
    cert.set_not_after(&Asn1Time::from_unix(4_102_444_800).unwrap())
        .unwrap();
    cert.sign(&key, MessageDigest::sha256()).unwrap();
    cert.build()
}
#[test]
fn digest_is_actual_der_and_refuses_ambiguous_or_unbounded_pem() {
    let cert = certificate();
    let pem = cert.to_pem().unwrap();
    assert_eq!(
        certificate_sha256(&pem).unwrap(),
        sha256(&cert.to_der().unwrap())
    );
    let mut padded = b" \n\t".to_vec();
    padded.extend_from_slice(&pem);
    padded.extend_from_slice(b"\r\n ");
    assert_eq!(
        certificate_sha256(&padded).unwrap(),
        sha256(&cert.to_der().unwrap())
    );
    let mut concatenated = pem.clone();
    concatenated.extend_from_slice(&pem);
    let mut junk_before = b"untrusted text\n".to_vec();
    junk_before.extend_from_slice(&pem);
    let mut junk_after = pem.clone();
    junk_after.extend_from_slice(b"untrusted text\n");
    let mut private_after = pem.clone();
    private_after
        .extend_from_slice(b"-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n");
    for bytes in [
        Vec::new(),
        cert.to_der().unwrap(),
        concatenated,
        junk_before,
        junk_after,
        private_after,
        vec![b' '; MAX_PEM + 1],
        b"-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n".to_vec(),
    ] {
        let error = certificate_sha256(&bytes).expect_err("ambiguous PEM accepted");
        assert!(error.to_string().len() < 128);
        assert!(!error.to_string().contains("untrusted text"));
    }
}
