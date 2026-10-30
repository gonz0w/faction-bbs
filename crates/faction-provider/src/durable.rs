//! Explicit restart durability for public provider policy only.
//!
//! Initialization requires an empty existing private root. Reopen requires the
//! original permanent lock and a complete current snapshot; pending bytes never
//! replace missing or corrupt history. This is not protection against an owner
//! or restoration of the whole directory from an older valid backup.
mod codec;
mod disk;
use crate::{
    Runtime, State,
    authority::{Authority, Registry},
};
pub(crate) use disk::Disk;
pub use faction_files::private_blobs::Error;
use faction_files::private_blobs::Principal;
use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
/// Each of at most two snapshots is bounded independently; the lock stays empty.
pub const MAX_SNAPSHOT: usize = 65_536;
pub const MAX_DISK_BYTES: usize = 2 * MAX_SNAPSHOT;
/// Bounded public inspection, without member records or connection authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicySummary {
    pub generation: u64,
    pub reset_epoch: u64,
    pub available: bool,
    pub homes: Vec<String>,
    pub binding_count: usize,
    pub history_count: usize,
}
#[derive(Clone)]
pub(crate) struct Snapshot {
    pub(crate) generation: u64,
    pub(crate) reset_epoch: u64,
    pub(crate) registry: Option<Registry>,
    pub(crate) history: Vec<([u8; 32], Principal)>,
    #[cfg(feature = "delegation-services")]
    pub(crate) service_policy: Option<crate::services::ServicePolicy>,
    #[cfg(feature = "delegation-services")]
    pub(crate) service_history: Vec<([u8; 32], crate::services::ServiceIdentity)>,
    #[cfg(feature = "delegation-services")]
    pub(crate) service_format_v2: bool,
    #[cfg(feature = "delegation-issuer")]
    pub(crate) service_format_v3: bool,
    #[cfg(feature = "delegation-verifier")]
    pub(crate) service_format_v4: bool,
    #[cfg(feature = "delegation-verifier")]
    pub(crate) remote_history: Vec<crate::services::RemoteFloor>,
}
impl Snapshot {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        #[cfg(feature = "delegation-verifier")]
        self.validate_resource_snapshot()?;
        if self.generation == 0 || self.history.len() > crate::authority::MAX_BINDINGS {
            return Err(Error::Invalid);
        }
        for (i, (digest, principal)) in self.history.iter().enumerate() {
            principal.validate()?;
            if *digest == [0; 32] || self.history[..i].iter().any(|(d, _)| d == digest) {
                return Err(Error::Invalid);
            }
        }
        if let Some(registry) = &self.registry {
            let checked = Registry::new(
                registry.generation(),
                registry.reset_epoch(),
                registry.homes().to_vec(),
                registry.bindings().to_vec(),
            )?;
            if checked.generation() != self.generation
                || checked.reset_epoch() != self.reset_epoch
                || checked.bindings().iter().any(|entry| {
                    !self
                        .history
                        .iter()
                        .any(|(d, p)| d == &entry.certificate_sha256 && p == &entry.principal)
                })
            {
                return Err(Error::Invalid);
            }
        }
        #[cfg(feature = "delegation-services")]
        {
            if self.history.len() + self.service_history.len() > crate::authority::MAX_BINDINGS
                || (!self.service_format_v2
                    && (self.service_policy.is_some() || !self.service_history.is_empty()))
            {
                return Err(Error::Invalid);
            }
            for (i, (digest, identity)) in self.service_history.iter().enumerate() {
                crate::services::validate_home(&identity.home)?;
                crate::services::validate_client_id(&identity.client_id)?;
                if *digest == [0; 32]
                    || self.history.iter().any(|(d, _)| d == digest)
                    || self.service_history[..i].iter().any(|(d, _)| d == digest)
                {
                    return Err(Error::Invalid);
                }
            }
            #[cfg(feature = "delegation-issuer")]
            if self
                .service_policy
                .as_ref()
                .is_some_and(|p| p.issuer.is_some())
                && !self.service_format_v3
            {
                return Err(Error::Invalid);
            }
            if let Some(policy) = &self.service_policy {
                policy.validate_registry(self.registry.as_ref().ok_or(Error::Invalid)?)?;
                for e in policy.registrations() {
                    e.validate()?;
                    if !self
                        .service_history
                        .iter()
                        .any(|(d, p)| d == &e.certificate_sha256 && p == &e.identity())
                    {
                        return Err(Error::Invalid);
                    }
                }
            }
        }
        Ok(())
    }
    fn service_binding_count(&self) -> usize {
        #[cfg(feature = "delegation-services")]
        {
            self.service_policy
                .as_ref()
                .map_or(0, |p| p.registrations().len())
        }
        #[cfg(not(feature = "delegation-services"))]
        {
            0
        }
    }
    fn service_history_count(&self) -> usize {
        #[cfg(feature = "delegation-services")]
        {
            self.service_history.len()
        }
        #[cfg(not(feature = "delegation-services"))]
        {
            0
        }
    }
    fn summary(&self) -> PolicySummary {
        PolicySummary {
            generation: self.generation,
            reset_epoch: self.reset_epoch,
            available: self.registry.is_some(),
            homes: self
                .registry
                .as_ref()
                .map_or_else(Vec::new, |r| r.homes().to_vec()),
            binding_count: self.registry.as_ref().map_or(0, |r| r.bindings().len())
                + self.service_binding_count(),
            history_count: self.history.len() + self.service_history_count(),
        }
    }
}
impl Runtime {
    /// Publish an explicitly preapproved initial registry in an empty owned 0700
    /// directory. A failed initialization is never retried as fresh history.
    pub fn initialize_durable(path: &Path, registry: Registry) -> Result<Self, Error> {
        let authority = Authority::new(Some(registry));
        let disk = Disk::initialize(path, &authority.snapshot())?;
        Ok(Self::from_durable(authority, disk))
    }
    /// Recover only validated retained current policy, with a fresh authority,
    /// sessions and handles. No bootstrap policy or older pending fallback exists.
    pub fn reopen_durable(path: &Path) -> Result<Self, Error> {
        let (disk, snapshot) = Disk::reopen(path)?;
        Ok(Self::from_durable(Authority::restore(snapshot)?, disk))
    }
    fn from_durable(authority: Authority, disk: Disk) -> Self {
        Self {
            state: Arc::new(State {
                authority: Mutex::new(authority),
                stopped: Arc::new(AtomicBool::new(false)),
                active: AtomicBool::new(false),
                socket: Mutex::new(None),
                disk: Some(disk),
            }),
        }
    }
    /// Read public bounded metadata from an active durable runtime.
    pub fn policy_summary(&self) -> Result<PolicySummary, Error> {
        let authority = self.state.authority.lock().map_err(|_| Error::Poisoned)?;
        if self.state.disk.is_none() || self.state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Poisoned);
        }
        Ok(authority.snapshot().summary())
    }
    /// Check the complete retained policy before binding a same-home service,
    /// without exposing individual member records to operator diagnostics.
    pub fn policy_matches_home(&self, home: &str) -> Result<bool, Error> {
        let authority = self.state.authority.lock().map_err(|_| Error::Poisoned)?;
        if self.state.disk.is_none() || self.state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Poisoned);
        }
        Ok(authority.snapshot().registry.as_ref().is_some_and(|r| {
            r.homes() == [home]
                && r.bindings()
                    .iter()
                    .all(|e| e.principal.account.split_once('@').map(|(_, h)| h) == Some(home))
        }))
    }
}
#[cfg(test)]
mod tests;
