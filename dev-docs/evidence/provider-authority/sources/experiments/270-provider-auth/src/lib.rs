//! Explicitly invoked synthetic provider-authority research, outside the product workspace.
//! The crate-private TLS construction boundary is trusted, not a process sandbox.
pub mod authority;

#[cfg(test)]
mod authority_tests;

pub mod tls;
