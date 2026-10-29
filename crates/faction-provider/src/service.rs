//! Trusted local binary service configuration; never a wire account selector.
use crate::{Error, tls::Stage};
use faction_files::private_blobs::Principal;
pub struct BinaryService {
    pub(crate) home: String,
}
impl BinaryService {
    pub fn new(home: &str) -> Result<Self, Error> {
        Principal {
            account: format!("policy@{home}"),
            device: 1,
            identity: [1; 32],
        }
        .validate()
        .map_err(|_| Error::Transport(Stage::Configuration))?;
        Ok(Self { home: home.into() })
    }
}
