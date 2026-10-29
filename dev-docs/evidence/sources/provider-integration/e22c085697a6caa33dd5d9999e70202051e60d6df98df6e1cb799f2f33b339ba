#![cfg(feature = "network-mail")]
use faction_client::mail_blob_wire;
use faction_files::private_blobs::{Binding, Principal, Request, Response};
fn binding() -> Binding {
    Binding {
        sender: Principal {
            account: "alice@home.test".into(),
            device: 101,
            identity: [1; 32],
        },
        recipient: Principal {
            account: "bob@away.test".into(),
            device: 202,
            identity: [2; 32],
        },
        id: [3; 32],
        size: 16,
        created: 100,
        expires: 200,
    }
}
#[test]
fn strict_iq_wrapper_preserves_binary_contract_and_exact_addresses() {
    let request = Request::Begin(binding());
    let xml = mail_blob_wire::request("one", "alice@home.test", "home.test", &request).unwrap();
    let decoded = mail_blob_wire::decode_request(&xml, "alice@home.test", "home.test").unwrap();
    assert_eq!(
        faction_files::private_blobs::encode_request(&decoded).unwrap(),
        faction_files::private_blobs::encode_request(&request).unwrap()
    );
    let response = Response::Progress {
        binding: binding(),
        offset: 16,
        committed: true,
    };
    let reply = mail_blob_wire::response(&xml, "home.test", Ok(response)).unwrap();
    assert!(matches!(
        mail_blob_wire::decode_response(&reply, "one", "alice@home.test", "home.test").unwrap(),
        Response::Progress {
            committed: true,
            ..
        }
    ));
    for (id, own, service) in [
        ("wrong", "alice@home.test", "home.test"),
        ("one", "bob@away.test", "home.test"),
        ("one", "alice@home.test", "evil.test"),
    ] {
        assert!(mail_blob_wire::decode_response(&reply, id, own, service).is_err());
    }
    let duplicate:faction_xmpp::Element="<iq xmlns='jabber:client' id='one' type='set' from='alice@home.test' to='home.test'><blob xmlns='urn:faction:blob:1'>AAAA</blob><blob xmlns='urn:faction:blob:1'>AAAA</blob></iq>".parse().unwrap();
    assert!(mail_blob_wire::decode_request(&duplicate, "alice@home.test", "home.test").is_err());
}
#[test]
fn denied_and_nonexistent_objects_have_one_identical_remote_error() {
    let xml = mail_blob_wire::request(
        "one",
        "alice@home.test",
        "home.test",
        &Request::Status { id: [3; 32] },
    )
    .unwrap();
    let denied = mail_blob_wire::response(
        &xml,
        "home.test",
        Err(faction_files::private_blobs::Error::Denied),
    )
    .unwrap();
    for error in [
        faction_files::private_blobs::Error::NotFound,
        faction_files::private_blobs::Error::Expired,
        faction_files::private_blobs::Error::Clock,
        faction_files::private_blobs::Error::Invalid,
    ] {
        assert_eq!(
            mail_blob_wire::response(&xml, "home.test", Err(error)).unwrap(),
            denied
        );
    }
}
