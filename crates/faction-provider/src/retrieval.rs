//! Transient recipient intent, never a remote read grant or store access proof.
//! A foreign sender's current authority and object commitment remain unverified.
use super::{Authority, Connection, Handle, MAX_HOMES};
use crate::{Control, Error, State, tls};
use faction_files::private_blobs::{Binding, Error as BlobError, Principal};
use openssl::x509::X509;
use std::{
    sync::{Arc, Weak, atomic::Ordering},
    time::Instant,
};

pub const MAX_RECORDS: usize = 8;
#[cfg(feature = "delegation-issuer")]
#[path = "issuer.rs"]
pub mod issuer;
/// Trusted local configuration. Construction validates data, not service identity.
/// A request cannot select this policy, its local home or its exact audience.
pub struct RetrievalPolicy {
    local_home: String,
    sender_audiences: Vec<(String, String)>,
    identity: Arc<()>,
}
impl RetrievalPolicy {
    pub fn new(
        local_home: String,
        sender_audiences: Vec<(String, String)>,
    ) -> Result<Self, BlobError> {
        validate_home(&local_home)?;
        if sender_audiences.len() > MAX_HOMES {
            return Err(BlobError::Invalid);
        }
        for (i, (home, audience)) in sender_audiences.iter().enumerate() {
            validate_home(home)?;
            if audience.is_empty()
                || audience.len() > 254
                || !audience.bytes().all(|b| b.is_ascii_graphic() && b != b'*')
                || sender_audiences[..i].iter().any(|(old, _)| old == home)
            {
                return Err(BlobError::Invalid);
            }
        }
        Ok(Self {
            local_home,
            sender_audiences,
            identity: Arc::new(()),
        })
    }
    fn audience(&self, binding: &Binding) -> Result<&str, BlobError> {
        if home(&binding.recipient)? != self.local_home {
            return Err(BlobError::Denied);
        }
        let sender = home(&binding.sender)?;
        self.sender_audiences
            .iter()
            .find(|(h, _)| h == sender)
            .map(|(_, a)| a.as_str())
            .ok_or(BlobError::Denied)
    }
}
fn validate_home(home: &str) -> Result<(), BlobError> {
    Principal {
        account: format!("policy@{home}"),
        device: 1,
        identity: [1; 32],
    }
    .validate()
}
fn home(principal: &Principal) -> Result<&str, BlobError> {
    principal
        .account
        .split_once('@')
        .map(|(_, h)| h)
        .ok_or(BlobError::Invalid)
}
/// Only recipient fetch at one exact offset, or recipient status, is representable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetrievalAction {
    GetChunk { offset: u64 },
    Status,
}
impl RetrievalAction {
    fn validate(self, binding: &Binding) -> Result<(), BlobError> {
        if matches!(self, Self::GetChunk{offset} if offset>binding.size) {
            Err(BlobError::Invalid)
        } else {
            Ok(())
        }
    }
}
/// Opaque nonserializable record, minted only from an actual current Session.
/// Contains neither a connection handle nor a strong runtime owner.
/// ```compile_fail
/// use faction_provider::retrieval::RetrievalRecord;
/// let forged = RetrievalRecord {};
/// ```
/// ```compile_fail
/// use faction_provider::retrieval::RetrievalRecord;
/// fn escape(record: RetrievalRecord) { let raw = record.token; }
/// ```
pub struct RetrievalRecord {
    state: Weak<State>,
    token: Arc<()>,
}
impl RetrievalRecord {
    pub(crate) fn new(state: &Arc<State>, token: Arc<()>) -> Self {
        Self {
            state: Arc::downgrade(state),
            token,
        }
    }
}
/// Read-only metadata from one current local check. This is not Access authority;
/// it establishes neither the sender's current device policy nor object commitment.
/// ```compile_fail
/// use faction_provider::retrieval::CheckedIntent;
/// let forged = CheckedIntent {};
/// ```
/// ```compile_fail
/// use faction_provider::retrieval::CheckedIntent;
/// use faction_files::private_blobs::Access;
/// fn grant<T: Access>() {}
/// grant::<CheckedIntent>();
/// ```
pub struct CheckedIntent {
    binding: Binding,
    action: RetrievalAction,
    audience: String,
}
impl CheckedIntent {
    pub fn binding(&self) -> &Binding {
        &self.binding
    }
    pub fn action(&self) -> RetrievalAction {
        self.action
    }
    pub fn audience(&self) -> &str {
        &self.audience
    }
}
#[derive(Clone, Copy)]
pub(crate) struct Moment {
    pub(super) wall: u64,
    pub(super) mono: Instant,
}
impl Moment {
    pub(crate) fn now() -> Self {
        Self {
            wall: tls::wall_now(),
            mono: Instant::now(),
        }
    }
}
pub(crate) struct Source<'a> {
    pub(crate) connection: &'a Connection,
    pub(crate) handle: &'a Handle,
    pub(crate) leaf: &'a X509,
    pub(crate) deadline: Instant,
    pub(crate) elapsed_ms: u64,
}
struct RecordSlot {
    token: Arc<()>,
    binding: Binding,
    action: RetrievalAction,
    audience: String,
    principal: Principal,
    certificate_sha256: [u8; 32],
    leaf: X509,
    generation: u64,
    reset_epoch: u64,
    policy: Arc<()>,
    deadline: Instant,
    #[cfg(feature = "delegation-issuer")]
    origin: Weak<std::sync::atomic::AtomicBool>,
    #[cfg(feature = "delegation-issuer")]
    issuer: Option<issuer::ReferenceState>,
}
#[derive(Default)]
pub(super) struct Records {
    policy: Option<RetrievalPolicy>,
    slots: Vec<RecordSlot>,
    wall_highwater: u64,
    mono_highwater: Option<Instant>,
    #[cfg(feature = "delegation-issuer")]
    precise_highwater: Option<std::time::Duration>,
}
impl Records {
    pub(super) fn invalidate(&mut self) {
        self.slots.clear();
    }
    fn prune(&mut self, now: Moment) -> Result<(), BlobError> {
        let rollback = now.wall < self.wall_highwater
            || self
                .mono_highwater
                .is_some_and(|previous| now.mono < previous);
        self.wall_highwater = self.wall_highwater.max(now.wall);
        self.mono_highwater = Some(
            self.mono_highwater
                .map_or(now.mono, |old| old.max(now.mono)),
        );
        if rollback {
            self.invalidate();
            return Err(BlobError::Clock);
        }
        self.slots.retain(|s| {
            now.mono < s.deadline
                && now.wall < s.binding.expires
                && tls::leaf_current(&s.leaf, now.wall)
        });
        Ok(())
    }
}
impl Authority {
    pub(crate) fn create_retrieval(
        &mut self,
        source: Source<'_>,
        binding: Binding,
        action: RetrievalAction,
        now: Moment,
    ) -> Result<Arc<()>, BlobError> {
        self.records.prune(now)?;
        binding.validate()?;
        action.validate(&binding)?;
        if now.mono >= source.deadline
            || !tls::leaf_current(source.leaf, now.wall)
            || !Arc::ptr_eq(&self.instance, &source.connection.authority)
            || !Arc::ptr_eq(&self.instance, &source.handle.authority)
            || !Arc::ptr_eq(&source.connection.closed, &source.handle.connection)
            || source.connection.closed.load(Ordering::Relaxed)
        {
            return Err(BlobError::Denied);
        }
        let registry = self.registry.as_ref().ok_or(BlobError::Denied)?;
        let slot = self
            .slots
            .iter()
            .find(|s| Arc::ptr_eq(&s.token, &source.handle.token))
            .ok_or(BlobError::Denied)?;
        if !slot.current(source.elapsed_ms)
            || slot.generation != self.generation
            || slot.reset_epoch != self.reset_epoch
            || !Arc::ptr_eq(&slot.connection, &source.connection.closed)
        {
            return Err(BlobError::Denied);
        }
        let current = registry.current(&slot.principal, now.wall)?;
        let observed = openssl::sha::sha256(&source.leaf.to_der().map_err(|_| BlobError::Denied)?);
        if current.certificate_sha256 != slot.certificate_sha256
            || observed != slot.certificate_sha256
            || binding.recipient != slot.principal
        {
            return Err(BlobError::Denied);
        }
        let policy = self.records.policy.as_ref().ok_or(BlobError::Denied)?;
        let audience = policy.audience(&binding)?.to_owned();
        validate_binding(registry, &binding, now.wall)?;
        if self.records.slots.len() >= MAX_RECORDS {
            return Err(BlobError::Busy);
        }
        let token = Arc::new(());
        self.records.slots.push(RecordSlot {
            token: token.clone(),
            binding,
            action,
            audience,
            principal: slot.principal.clone(),
            certificate_sha256: slot.certificate_sha256,
            leaf: source.leaf.clone(),
            generation: self.generation,
            reset_epoch: self.reset_epoch,
            policy: policy.identity.clone(),
            deadline: source.deadline,
            #[cfg(feature = "delegation-issuer")]
            origin: Arc::downgrade(&source.connection.closed),
            #[cfg(feature = "delegation-issuer")]
            issuer: None,
        });
        Ok(token)
    }
    fn check_retrieval_at(
        &mut self,
        token: &Arc<()>,
        binding: &Binding,
        action: RetrievalAction,
        audience: &str,
        now: Moment,
    ) -> Result<CheckedIntent, BlobError> {
        self.records.prune(now)?;
        let slot = self
            .records
            .slots
            .iter()
            .find(|s| Arc::ptr_eq(&s.token, token))
            .ok_or(BlobError::Denied)?;
        if &slot.binding != binding || slot.action != action || slot.audience != audience {
            return Err(BlobError::Denied);
        }
        let registry = self.registry.as_ref().ok_or(BlobError::Denied)?;
        let policy = self.records.policy.as_ref().ok_or(BlobError::Denied)?;
        if slot.generation != self.generation
            || slot.reset_epoch != self.reset_epoch
            || !Arc::ptr_eq(&slot.policy, &policy.identity)
            || policy.audience(binding)? != audience
            || registry
                .current(&slot.principal, now.wall)?
                .certificate_sha256
                != slot.certificate_sha256
            || binding.recipient != slot.principal
        {
            return Err(BlobError::Denied);
        }
        validate_binding(registry, binding, now.wall)?;
        Ok(CheckedIntent {
            binding: slot.binding.clone(),
            action: slot.action,
            audience: slot.audience.clone(),
        })
    }
}
fn validate_binding(
    registry: &super::Registry,
    binding: &Binding,
    wall: u64,
) -> Result<(), BlobError> {
    binding.validate()?;
    if binding.created > wall || wall >= binding.expires {
        return Err(BlobError::Expired);
    }
    for p in [&binding.sender, &binding.recipient] {
        if !registry
            .homes
            .iter()
            .any(|h| home(p).is_ok_and(|expected| h == expected))
        {
            return Err(BlobError::Denied);
        }
    }
    // The foreign sender is data, never locally fabricated authentication.
    registry.current(&binding.recipient, wall)?;
    Ok(())
}
impl Control {
    /// Replace transient trusted policy. It is never written to a durable snapshot.
    pub fn replace_retrieval_policy(&self, policy: RetrievalPolicy) -> Result<(), Error> {
        let state = self.state()?;
        let mut authority = state
            .authority
            .lock()
            .map_err(|_| Error::Transport(tls::Stage::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(tls::Stage::Denied));
        }
        authority.records.invalidate();
        authority.records.policy = Some(policy);
        Ok(())
    }
    /// Recheck one exact tuple against current local authority. No store dispatch,
    /// resource permission, callback, service authentication or remote grant occurs.
    pub fn check_retrieval(
        &self,
        record: &RetrievalRecord,
        binding: &Binding,
        action: RetrievalAction,
        audience: &str,
    ) -> Result<CheckedIntent, Error> {
        let state = self.state()?;
        if !Weak::ptr_eq(&record.state, &self.state) {
            return Err(Error::Transport(tls::Stage::Denied));
        }
        let mut authority = state
            .authority
            .lock()
            .map_err(|_| Error::Transport(tls::Stage::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(tls::Stage::Denied));
        }
        authority
            .check_retrieval_at(&record.token, binding, action, audience, Moment::now())
            .map_err(Error::Store)
    }
}
#[cfg(test)]
#[path = "retrieval_tests.rs"]
mod tests;
