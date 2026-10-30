//! Bounded, standalone OAuth wire data. Every decoded value is **unverified**.
//!
//! Parsing proves syntax only. It establishes no TLS possession, registration,
//! device approval, invitation, current policy, audience or resource authority.
//! The crate has no provider/product dependency and no authority conversion.
//! RFC 6749 defines grant_type/client_id/scope and token response fields; this
//! private Faction mapping puts one opaque server-operation reference in scope.
//! RFC 7662 defines introspection fields; all `faction_*` fields, claim shapes,
//! purpose/action separation and limits here are Faction application restrictions.
mod form;
mod http;
mod json;
mod types;
pub use form::*;
pub use http::*;
pub use json::*;
pub use types::*;

pub const MAX_HEADER: usize = 8192;
pub const MAX_FORM: usize = 8192;
pub const MAX_JSON: usize = 8192;
pub const MAX_TOKEN: usize = 256;
pub const MAX_HEADERS: usize = 16;
pub const MAX_FIELDS: usize = 24;
pub const MAX_DEPTH: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error;
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid delegation wire data")
    }
}
impl std::error::Error for Error {}
