#![cfg(feature = "delegation-verifier")]
//! Source-bound handoff contracts: actual opaque sessions are consumed; no actor
//! or arbitrary externally supplied response can construct a delivery.
use faction_files::private_blobs::Store;
use faction_provider::tls::HttpEndpoint;
use faction_provider::{
    Error, ServiceSession, Session,
    delivery::{Delivery, PublicationWork, RetrievalWork},
};
use std::sync::Mutex;
#[test]
fn submitting_publication_consumes_its_actual_device_carrier() {
    let _: fn(Session) -> Result<(Delivery, PublicationWork), Error> = Session::submit_publication;
}
#[test]
fn submitting_resource_consumes_its_actual_registered_relay_carrier() {
    let _: fn(ServiceSession) -> Result<(Delivery, RetrievalWork), Error> =
        ServiceSession::submit_retrieval;
}
#[test]
fn completion_is_produced_by_consuming_the_prepared_operation() {
    let _: fn(PublicationWork, &HttpEndpoint, &Mutex<Store>) -> Result<(), Error> =
        PublicationWork::execute;
    let _: fn(RetrievalWork, &HttpEndpoint, &Mutex<Store>) -> Result<(), Error> =
        RetrievalWork::execute;
}
#[test]
fn collect_requires_fresh_owning_tls_sessions() {
    let _: fn(Session, &mut Delivery) -> Result<bool, Error> = Session::collect_delivery;
    let _: fn(ServiceSession, &mut Delivery) -> Result<bool, Error> =
        ServiceSession::collect_delivery;
}

#[test]
fn parent_deadline_constructor_has_no_caller_clock_or_actor_override() {
    use faction_provider::{
        delivery::{CarrierEndpoint, Operation},
        tls::Stage,
    };
    use std::{
        sync::{Arc, atomic::AtomicBool},
        time::Instant,
    };
    let _: fn(&CarrierEndpoint, Instant, Arc<AtomicBool>) -> Result<Operation, Stage> =
        CarrierEndpoint::operation_until;
}
