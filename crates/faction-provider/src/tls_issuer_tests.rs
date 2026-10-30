use super::*;
use crate::{
    authority::{CredentialBinding, Registry},
    retrieval::{RetrievalAction, RetrievalPolicy},
    services::{IssuerRoute, ServicePolicy, ServiceRegistration, ServiceUse},
};
use faction_files::private_blobs::{Binding, Principal};
use openssl::{
    bn::BigNum,
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    nid::Nid,
    x509::{
        X509NameBuilder,
        extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage, SubjectAlternativeName},
    },
};
fn key() -> PKey<Private> {
    PKey::from_ec_key(
        EcKey::generate(&EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap()).unwrap(),
    )
    .unwrap()
}
pub(super) fn cert(
    ca: Option<(&X509, &PKey<Private>)>,
    name: &str,
    serial: u32,
    start: i64,
    end: i64,
) -> (X509, PKey<Private>) {
    let key = key();
    let mut n = X509NameBuilder::new().unwrap();
    n.append_entry_by_text("CN", name).unwrap();
    let n = n.build();
    let mut b = X509::builder().unwrap();
    b.set_version(2).unwrap();
    b.set_serial_number(&BigNum::from_u32(serial).unwrap().to_asn1_integer().unwrap())
        .unwrap();
    b.set_subject_name(&n).unwrap();
    b.set_issuer_name(ca.map_or(n.as_ref(), |v| v.0.subject_name()))
        .unwrap();
    b.set_pubkey(&key).unwrap();
    b.set_not_before(&Asn1Time::from_unix(start).unwrap())
        .unwrap();
    b.set_not_after(&Asn1Time::from_unix(end).unwrap()).unwrap();
    if ca.is_none() {
        b.append_extension(BasicConstraints::new().critical().ca().build().unwrap())
            .unwrap();
        b.append_extension(KeyUsage::new().key_cert_sign().crl_sign().build().unwrap())
            .unwrap();
    } else {
        b.append_extension(BasicConstraints::new().critical().build().unwrap())
            .unwrap();
        b.append_extension(KeyUsage::new().digital_signature().build().unwrap())
            .unwrap();
        b.append_extension(
            ExtendedKeyUsage::new()
                .client_auth()
                .server_auth()
                .build()
                .unwrap(),
        )
        .unwrap();
        let ext = SubjectAlternativeName::new()
            .dns("provider.test")
            .build(&b.x509v3_context(ca.map(|v| v.0.as_ref()), None))
            .unwrap();
        b.append_extension(ext).unwrap();
    }
    b.sign(ca.map_or(&key, |v| v.1), MessageDigest::sha256())
        .unwrap();
    (b.build(), key)
}

#[test]
fn real_device_stalled_drop_keeps_original_owners_until_socket_cleanup_finishes() {
    let t = wall_now() as i64;
    let ca = cert(None, "CA", 1, t - 60, t + 3600);
    let server = cert(Some((&ca.0, &ca.1)), "provider.test", 2, t - 60, t + 3600);
    let device = cert(Some((&ca.0, &ca.1)), "device", 3, t - 60, t + 3600);
    let relay = cert(Some((&ca.0, &ca.1)), "relay", 4, t - 60, t + 3600);
    let inspector = cert(Some((&ca.0, &ca.1)), "inspector", 5, t - 60, t + 3600);
    let recipient = Principal {
        account: "recipient@home.test".into(),
        device: 1,
        identity: [1; 32],
    };
    let registry = Registry::new(
        1,
        0,
        vec!["home.test".into(), "foreign.test".into()],
        vec![CredentialBinding {
            certificate_sha256: sha256(&device.0.to_der().unwrap()),
            principal: recipient.clone(),
            valid_from: (t - 60) as u64,
            valid_until: (t + 3600) as u64,
            approved: true,
            account_enabled: true,
            device_enabled: true,
            generation: 1,
        }],
    )
    .unwrap();
    let registration = |leaf: &X509, home: &str, id: &str| ServiceRegistration {
        home: home.into(),
        client_id: id.into(),
        dns_san: "provider.test".into(),
        certificate_sha256: sha256(&leaf.to_der().unwrap()),
        enabled: true,
        valid_from: (t - 60) as u64,
        valid_until: (t + 3600) as u64,
    };
    let policy = ServicePolicy::new_issuer(
        "home.test".into(),
        "issuer.test".into(),
        vec![
            (
                registration(&relay.0, "home.test", "relay"),
                ServiceUse::Relay,
            ),
            (
                registration(&inspector.0, "foreign.test", "inspector"),
                ServiceUse::Introspection,
            ),
        ],
        vec![IssuerRoute {
            sender_home: "foreign.test".into(),
            audience: "foreign.test/attachments".into(),
            relay_client_id: "relay".into(),
            introspection_client_id: "inspector".into(),
        }],
    )
    .unwrap();
    let runtime = Runtime::new(None);
    runtime
        .control()
        .replace_delegation_policy(registry, policy)
        .unwrap();
    runtime
        .control()
        .replace_retrieval_policy(
            RetrievalPolicy::new(
                "home.test".into(),
                vec![("foreign.test".into(), "foreign.test/attachments".into())],
            )
            .unwrap(),
        )
        .unwrap();
    let binding = Binding {
        sender: Principal {
            account: "sender@foreign.test".into(),
            device: 2,
            identity: [2; 32],
        },
        recipient,
        id: [7; 32],
        size: 32,
        created: t as u64,
        expires: (t + 100) as u64,
    };
    let acceptor = Server::new(&server.0, &server.1, &ca.0).unwrap();
    let connect = |address| {
        Client::connect(
            address,
            "provider.test",
            sha256(&server.0.to_der().unwrap()),
            &ca.0,
            Some((&device.0, Some(&device.1))),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::scope(|scope| {
        let peer = scope.spawn(|| connect(address));
        let mut session = acceptor
            .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
            .unwrap();
        let _peer = peer.join().unwrap();
        let record = session
            .retrieval_intent(binding, RetrievalAction::Status)
            .unwrap();
        let witness = session.connection.test_lifecycle_witness();
        assert!(runtime.control().provision_retrieval(&record).is_err());
        let socket_lock = runtime.state.socket.lock().unwrap();
        let cleanup = scope.spawn(move || drop(session));
        let end = Instant::now() + Duration::from_secs(2);
        while !witness.upgrade().is_some_and(|w| w.load(Ordering::Relaxed)) && Instant::now() < end
        {
            std::thread::yield_now();
        }
        assert!(witness.upgrade().is_some_and(|w| w.load(Ordering::Relaxed)));
        assert!(runtime.state.active.load(Ordering::SeqCst));
        assert!(runtime.control().provision_retrieval(&record).is_err());
        drop(socket_lock);
        cleanup.join().unwrap();
        assert!(witness.upgrade().is_none());
        assert!(!runtime.state.active.load(Ordering::SeqCst));
        let reference = runtime.control().provision_retrieval(&record).unwrap();
        // A later actual device owns the lane, but cannot change original ownership.
        let peer = scope.spawn(|| connect(address));
        let _later = acceptor
            .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
            .unwrap();
        let _peer = peer.join().unwrap();
        assert_eq!(
            reference.expose_bytes(),
            runtime
                .control()
                .provision_retrieval(&record)
                .unwrap()
                .expose_bytes()
        );
    });
}
