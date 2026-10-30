//! Authenticated home-service policy observations. No device or Store authority.
use crate::{
    Control, Error,
    authority::{Authority, MAX_BINDINGS, Registry},
    tls,
};
use faction_files::private_blobs::{Binding, CHUNK, Error as BlobError, Principal};
use openssl::{sha::sha256, x509::X509};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ServiceIdentity {
    pub(crate) home: String,
    pub(crate) client_id: String,
}
/// Trusted configuration data; construction does not authenticate a client.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceRegistration {
    pub home: String,
    pub client_id: String,
    pub dns_san: String,
    pub certificate_sha256: [u8; 32],
    pub enabled: bool,
    pub valid_from: u64,
    pub valid_until: u64,
}
impl ServiceRegistration {
    pub(crate) fn identity(&self) -> ServiceIdentity {
        ServiceIdentity {
            home: self.home.clone(),
            client_id: self.client_id.clone(),
        }
    }
    pub(crate) fn validate(&self) -> Result<(), BlobError> {
        validate_home(&self.home)?;
        validate_home(&self.dns_san)?;
        validate_client_id(&self.client_id)?;
        if self.certificate_sha256 == [0; 32] || self.valid_from >= self.valid_until {
            return Err(BlobError::Invalid);
        }
        Ok(())
    }
}
pub(crate) fn validate_home(home: &str) -> Result<(), BlobError> {
    Principal {
        account: format!("policy@{home}"),
        device: 1,
        identity: [1; 32],
    }
    .validate()
}
pub(crate) fn validate_client_id(id: &str) -> Result<(), BlobError> {
    if id.is_empty() || id.len() > 254 || !id.bytes().all(|b| b.is_ascii_graphic() && b != b'*') {
        Err(BlobError::Invalid)
    } else {
        Ok(())
    }
}
/// Validated exact local home and public service registrations. Default off.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServicePolicy {
    local_home: String,
    registrations: Vec<ServiceRegistration>,
    uses: Vec<ServiceUse>,
    #[cfg(feature = "delegation-issuer")]
    pub(crate) issuer: Option<IssuerPolicy>,
}
impl ServicePolicy {
    pub fn new(
        local_home: String,
        registrations: Vec<ServiceRegistration>,
    ) -> Result<Self, BlobError> {
        validate_home(&local_home)?;
        if registrations.len() > MAX_BINDINGS {
            return Err(BlobError::Invalid);
        }
        for (i, e) in registrations.iter().enumerate() {
            e.validate()?;
            if e.home == local_home
                || registrations[..i].iter().any(|old| {
                    old.identity() == e.identity() || old.certificate_sha256 == e.certificate_sha256
                })
            {
                return Err(BlobError::Invalid);
            }
        }
        Ok(Self {
            local_home,
            uses: vec![ServiceUse::Observation; registrations.len()],
            registrations,
            #[cfg(feature = "delegation-issuer")]
            issuer: None,
        })
    }
    pub fn local_home(&self) -> &str {
        &self.local_home
    }
    pub fn registrations(&self) -> &[ServiceRegistration] {
        &self.registrations
    }
    pub(crate) fn validate_registry(&self, r: &Registry) -> Result<(), BlobError> {
        if r.bindings().len() + self.registrations.len() > MAX_BINDINGS {
            return Err(BlobError::Quota);
        }
        if !r.homes().contains(&self.local_home)
            || self
                .registrations
                .iter()
                .any(|s| !r.homes().contains(&s.home))
        {
            return Err(BlobError::Invalid);
        }
        Ok(())
    }
    pub(crate) fn current(
        &self,
        identity: &ServiceIdentity,
        digest: [u8; 32],
        leaf: &X509,
        now: u64,
        registry: &Registry,
    ) -> Result<(), BlobError> {
        let e = self
            .registrations
            .iter()
            .find(|e| e.identity() == *identity && e.certificate_sha256 == digest)
            .ok_or(BlobError::Denied)?;
        if !e.enabled
            || now < e.valid_from
            || now >= e.valid_until
            || !registry.homes().contains(&e.home)
            || !registry.homes().contains(&self.local_home)
            || !tls::leaf_current(leaf, now)
            || sha256(&leaf.to_der().map_err(|_| BlobError::Denied)?) != digest
            || !leaf.subject_alt_names().is_some_and(|names| {
                names
                    .iter()
                    .any(|n| n.dnsname() == Some(e.dns_san.as_str()))
            })
        {
            return Err(BlobError::Denied);
        }
        Ok(())
    }
}
/// Publication policy only. Chunk observations bind all bytes through SHA-256.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicationAction {
    Begin,
    PutChunk {
        offset: u64,
        length: u64,
        ciphertext_sha256: [u8; 32],
    },
    Commit,
    Status,
    Delete,
}
impl PublicationAction {
    fn validate(self, b: &Binding) -> Result<(), BlobError> {
        if let Self::PutChunk { offset, length, .. } = self
            && (length == 0
                || length > CHUNK as u64
                || offset.checked_add(length).is_none_or(|end| end > b.size))
        {
            return Err(BlobError::Invalid);
        }
        Ok(())
    }
}
/// Read-only metadata at one check time. Never a grant, cached current policy,
/// device key-possession proof, sender-current proof, or committed-object proof.
/// ```compile_fail
/// use faction_provider::services::RecipientCurrentObservation;
/// let forged = RecipientCurrentObservation {};
/// ```
/// ```compile_fail
/// use faction_provider::services::RecipientCurrentObservation;
/// fn grant<T: faction_files::private_blobs::Access>() {}
/// grant::<RecipientCurrentObservation>();
/// ```
pub struct RecipientCurrentObservation {
    binding: Binding,
    action: PublicationAction,
    service: ServiceIdentity,
    service_certificate_sha256: [u8; 32],
    recipient_certificate_sha256: [u8; 32],
    generation: u64,
    reset_epoch: u64,
    checked_at: u64,
}
impl RecipientCurrentObservation {
    pub fn binding(&self) -> &Binding {
        &self.binding
    }
    pub fn action(&self) -> PublicationAction {
        self.action
    }
    pub fn service_home(&self) -> &str {
        &self.service.home
    }
    pub fn client_id(&self) -> &str {
        &self.service.client_id
    }
    pub fn service_certificate_sha256(&self) -> [u8; 32] {
        self.service_certificate_sha256
    }
    pub fn recipient_certificate_sha256(&self) -> [u8; 32] {
        self.recipient_certificate_sha256
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn reset_epoch(&self) -> u64 {
        self.reset_epoch
    }
    pub fn checked_at(&self) -> u64 {
        self.checked_at
    }
}
#[derive(Clone)]
pub(crate) struct ServiceCredential {
    pub(crate) identity: ServiceIdentity,
    pub(crate) context: Arc<()>,
    pub(crate) digest: [u8; 32],
    pub(crate) generation: u64,
    pub(crate) reset_epoch: u64,
    pub(crate) wall_highwater: u64,
    pub(crate) live: bool,
}
impl Authority {
    pub(crate) fn authenticate_service(
        &self,
        leaf: &X509,
        now: u64,
    ) -> Result<ServiceCredential, BlobError> {
        let digest = sha256(&leaf.to_der().map_err(|_| BlobError::Denied)?);
        let p = self.service_policy.as_ref().ok_or(BlobError::Denied)?;
        let r = self.registry.as_ref().ok_or(BlobError::Denied)?;
        let e = p
            .registrations
            .iter()
            .find(|e| e.certificate_sha256 == digest)
            .ok_or(BlobError::Denied)?;
        p.current(&e.identity(), digest, leaf, now, r)?;
        Ok(ServiceCredential {
            identity: e.identity(),
            context: self.service_context.clone(),
            digest,
            generation: self.generation,
            reset_epoch: self.reset_epoch,
            wall_highwater: now,
            live: true,
        })
    }
    pub(crate) fn observe_recipient(
        &self,
        credential: &mut ServiceCredential,
        leaf: &X509,
        binding: Binding,
        action: PublicationAction,
        now: u64,
    ) -> Result<RecipientCurrentObservation, BlobError> {
        if !credential.live || now < credential.wall_highwater {
            credential.live = false;
            return Err(BlobError::Denied);
        }
        credential.wall_highwater = now;
        let result = (|| {
            if !Arc::ptr_eq(&credential.context, &self.service_context)
                || credential.generation != self.generation
                || credential.reset_epoch != self.reset_epoch
            {
                return Err(BlobError::Denied);
            }
            let r = self.registry.as_ref().ok_or(BlobError::Denied)?;
            let p = self.service_policy.as_ref().ok_or(BlobError::Denied)?;
            p.current(&credential.identity, credential.digest, leaf, now, r)?;
            if p.service_use(&credential.identity) != Some(ServiceUse::Observation) {
                return Err(BlobError::Denied);
            }
            binding.validate()?;
            action.validate(&binding)?;
            if binding.created > now
                || now >= binding.expires
                || binding.sender.account.split_once('@').map(|(_, h)| h)
                    != Some(credential.identity.home.as_str())
                || binding.recipient.account.split_once('@').map(|(_, h)| h)
                    != Some(p.local_home.as_str())
            {
                return Err(BlobError::Denied);
            }
            let recipient = r.current(&binding.recipient, now)?;
            Ok(RecipientCurrentObservation {
                binding,
                action,
                service: credential.identity.clone(),
                service_certificate_sha256: credential.digest,
                recipient_certificate_sha256: recipient.certificate_sha256,
                generation: self.generation,
                reset_epoch: self.reset_epoch,
                checked_at: now,
            })
        })();
        // Failed current credential checks cannot be revived by a later clock.
        if !tls::leaf_current(leaf, now) {
            credential.live = false;
        }
        result
    }
    pub(crate) fn replace_delegation_policy(
        &mut self,
        registry: Registry,
        policy: ServicePolicy,
    ) -> Result<(), BlobError> {
        policy.validate_registry(&registry)?;
        let mut history = self.service_history.clone();
        for e in &policy.registrations {
            if self.history.iter().any(|(d, _)| d == &e.certificate_sha256)
                || registry
                    .bindings()
                    .iter()
                    .any(|e2| e2.certificate_sha256 == e.certificate_sha256)
            {
                return Err(BlobError::Invalid);
            }
            if let Some((_, old)) = history.iter().find(|(d, _)| d == &e.certificate_sha256) {
                if old != &e.identity() {
                    return Err(BlobError::Invalid);
                }
            } else {
                history.push((e.certificate_sha256, e.identity()));
            }
        }
        // replace_registry checks the original service tombstones as well.
        let device_history = self.proposed_history(&registry)?;
        if history.len() + device_history.len() > MAX_BINDINGS {
            return Err(BlobError::Quota);
        }
        self.replace_registry(registry)?;
        self.service_history = history;
        #[cfg(feature = "delegation-issuer")]
        {
            self.service_format_v3 |= policy.issuer.is_some();
        }
        self.service_policy = Some(policy);
        Ok(())
    }
}
impl Control {
    /// Atomically replace the complete public proposal under existing floors.
    pub fn replace_delegation_policy(
        &self,
        registry: Registry,
        policy: ServicePolicy,
    ) -> Result<(), Error> {
        self.state()?
            .update(|a| a.replace_delegation_policy(registry, policy))
    }
}
#[cfg(test)]
#[path = "services_tests.rs"]
mod tests;

/// A registration has exactly one explicit purpose; authentication adds no other use.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceUse {
    Observation,
    #[cfg(feature = "delegation-issuer")]
    Relay,
    #[cfg(feature = "delegation-issuer")]
    Introspection,
}
#[cfg(feature = "delegation-issuer")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerRoute {
    pub sender_home: String,
    pub audience: String,
    pub relay_client_id: String,
    pub introspection_client_id: String,
}
#[cfg(feature = "delegation-issuer")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IssuerPolicy {
    pub(crate) issuer: String,
    pub(crate) routes: Vec<IssuerRoute>,
}
impl ServicePolicy {
    pub(crate) fn service_use(&self, identity: &ServiceIdentity) -> Option<ServiceUse> {
        self.registrations
            .iter()
            .position(|r| r.identity() == *identity)
            .map(|i| self.uses[i])
    }
    #[cfg(feature = "delegation-issuer")]
    pub fn new_issuer(
        local_home: String,
        issuer: String,
        entries: Vec<(ServiceRegistration, ServiceUse)>,
        routes: Vec<IssuerRoute>,
    ) -> Result<Self, BlobError> {
        validate_home(&local_home)?;
        validate_home(&issuer)?;
        if entries.len() > MAX_BINDINGS
            || routes.is_empty()
            || routes.len() > crate::authority::MAX_HOMES
        {
            return Err(BlobError::Invalid);
        }
        for (i, (entry, purpose)) in entries.iter().enumerate() {
            entry.validate()?;
            if (*purpose == ServiceUse::Relay) != (entry.home == local_home)
                || entries[..i].iter().any(|(old, _)| {
                    old.identity() == entry.identity()
                        || old.certificate_sha256 == entry.certificate_sha256
                })
            {
                return Err(BlobError::Invalid);
            }
            if *purpose != ServiceUse::Observation {
                issuer_client_id(&entry.client_id)?;
            }
        }
        for (i, route) in routes.iter().enumerate() {
            validate_home(&route.sender_home)?;
            issuer_client_id(&route.relay_client_id)?;
            issuer_client_id(&route.introspection_client_id)?;
            if route.sender_home == local_home
                || route.audience != format!("{}/attachments", route.sender_home)
                || routes[..i]
                    .iter()
                    .any(|old| old.sender_home == route.sender_home)
                || !entries.iter().any(|(e, u)| {
                    *u == ServiceUse::Relay
                        && e.home == local_home
                        && e.client_id == route.relay_client_id
                })
                || !entries.iter().any(|(e, u)| {
                    *u == ServiceUse::Introspection
                        && e.home == route.sender_home
                        && e.client_id == route.introspection_client_id
                })
            {
                return Err(BlobError::Invalid);
            }
        }
        let (registrations, uses) = entries.into_iter().unzip();
        Ok(Self {
            local_home,
            registrations,
            uses,
            issuer: Some(IssuerPolicy { issuer, routes }),
        })
    }
}
#[cfg(feature = "delegation-issuer")]
fn issuer_client_id(id: &str) -> Result<(), BlobError> {
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
    {
        Err(BlobError::Invalid)
    } else {
        Ok(())
    }
}

/// Explicit transport restriction on installed Observation permissions.
/// It adds no registration, role, device actor or Store authorization.
#[cfg(feature = "delegation-wire")]
#[derive(Clone)]
pub struct PublicationWireProfile {
    home: String,
    clients: Vec<String>,
}
#[cfg(feature = "delegation-wire")]
impl PublicationWireProfile {
    pub fn new(home: String, clients: Vec<String>) -> Result<Self, BlobError> {
        validate_home(&home)?;
        if clients.is_empty() || clients.len() > MAX_BINDINGS {
            return Err(BlobError::Invalid);
        }
        for (i, id) in clients.iter().enumerate() {
            issuer_client_id(id)?;
            if clients[..i].contains(id) {
                return Err(BlobError::Invalid);
            }
        }
        // Apply the exact standalone wire host syntax without changing legacy IDs.
        faction_delegation_codec::encode_introspection_http(
            &home,
            &faction_delegation_codec::UnverifiedIntrospectionRequest {
                client_id: "validation".into(),
                token: faction_delegation_codec::UnverifiedToken::from_bytes([1; 32]),
            },
        )
        .map_err(|_| BlobError::Invalid)?;
        Ok(Self { home, clients })
    }
    pub(crate) fn current(
        &self,
        a: &Authority,
        c: &ServiceCredential,
        leaf: &X509,
        now: u64,
    ) -> Result<(), BlobError> {
        if !c.live
            || now < c.wall_highwater
            || !Arc::ptr_eq(&c.context, &a.service_context)
            || c.generation != a.generation
            || c.reset_epoch != a.reset_epoch
            || !self.clients.contains(&c.identity.client_id)
        {
            return Err(BlobError::Denied);
        }
        let p = a.service_policy.as_ref().ok_or(BlobError::Denied)?;
        if p.local_home != self.home || p.service_use(&c.identity) != Some(ServiceUse::Observation)
        {
            return Err(BlobError::Denied);
        }
        p.current(
            &c.identity,
            c.digest,
            leaf,
            now,
            a.registry.as_ref().ok_or(BlobError::Denied)?,
        )
    }
    pub(crate) fn host(&self) -> &str {
        &self.home
    }
}
#[cfg(feature = "delegation-wire")]
impl Control {
    pub(crate) fn outbound_http_probe(&self) -> Result<Arc<dyn Fn() -> bool + Send + Sync>, Error> {
        let state = self.state()?;
        let context = {
            let a = state
                .authority
                .lock()
                .map_err(|_| Error::Transport(tls::Stage::Denied))?;
            if a.registry.is_none() {
                return Err(Error::Transport(tls::Stage::Denied));
            }
            a.service_context.clone()
        };
        let weak = Arc::downgrade(&state);
        Ok(Arc::new(move || {
            let Some(s) = weak.upgrade() else {
                return false;
            };
            if s.stopped.load(std::sync::atomic::Ordering::SeqCst) {
                return false;
            }
            s.authority
                .lock()
                .is_ok_and(|a| a.registry.is_some() && Arc::ptr_eq(&context, &a.service_context))
        }))
    }
}
