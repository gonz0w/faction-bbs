#![cfg(feature = "delegation-verifier")]
use faction_files::private_blobs::{Binding, Request};
use faction_provider::{Error, Session, services::PendingPublication};
#[test]
fn publication_preparation_consumes_the_authenticated_sender_session() {
    let _factory: fn(Session, Binding, Request) -> Result<PendingPublication, Error> =
        Session::prepare_publication;
}
