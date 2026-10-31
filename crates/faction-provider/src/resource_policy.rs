//! Canonical resource-client associations. No key, token, session or Access state.
use super::*;
use std::{collections::BTreeSet, net::SocketAddr};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceAssociation {
    pub recipient_home: String,
    pub issuer: String,
    pub audience: String,
    pub relay_client_id: String,
    pub outbound_client_id: String,
    pub endpoint_id: String,
    pub address: SocketAddr,
    pub http_host: String,
    pub ca_sha256: [u8; 32],
    pub remote_generation: u64,
    pub remote_reset: u64,
}
/// Exact recipient-home publication endpoint and separate outgoing permission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicationAssociation {
    pub recipient_home: String,
    pub outbound_client_id: String,
    pub endpoint_id: String,
    pub address: SocketAddr,
    pub http_host: String,
    pub ca_sha256: [u8; 32],
    pub remote_generation: u64,
    pub remote_reset: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RemoteFloor {
    pub(crate) home: String,
    pub(crate) generation: u64,
    pub(crate) reset: u64,
}
fn wire_host(home: &str) -> Result<(), BlobError> {
    faction_delegation_codec::encode_introspection_http(
        home,
        &faction_delegation_codec::UnverifiedIntrospectionRequest {
            client_id: "validation".into(),
            token: faction_delegation_codec::UnverifiedToken::from_bytes([1; 32]),
        },
    )
    .map(|_| ())
    .map_err(|_| BlobError::Invalid)
}
impl ServicePolicy {
    /// Explicit complete resource profile; the legacy constructors add no roles.
    pub fn new_resource(
        local_home: String,
        entries: Vec<(ServiceRegistration, ServiceUse)>,
        associations: Vec<ResourceAssociation>,
    ) -> Result<Self, BlobError> {
        Self::new_resource_profile(local_home, entries, associations, None, None)
    }
    /// Explicit complete issuer and resource profile on one canonical registry.
    /// Directional constructors retain their original accepted purposes.
    pub fn new_composed(
        local_home: String,
        issuer: String,
        entries: Vec<(ServiceRegistration, ServiceUse)>,
        routes: Vec<IssuerRoute>,
        associations: Vec<ResourceAssociation>,
    ) -> Result<Self, BlobError> {
        validate_home(&issuer)?;
        wire_host(&issuer)?;
        validate_issuer_routes(&local_home, &entries, &routes)?;
        Self::new_resource_profile(
            local_home,
            entries,
            associations,
            Some(IssuerPolicy { issuer, routes }),
            None,
        )
    }
    /// Explicit complete publication profile; older constructors add no outgoing permission.
    pub fn new_composed_publication(
        local_home: String,
        issuer: String,
        entries: Vec<(ServiceRegistration, ServiceUse)>,
        routes: Vec<IssuerRoute>,
        associations: Vec<ResourceAssociation>,
        publications: Vec<PublicationAssociation>,
    ) -> Result<Self, BlobError> {
        validate_home(&issuer)?;
        wire_host(&issuer)?;
        validate_issuer_routes(&local_home, &entries, &routes)?;
        if publications.is_empty() || publications.len() > MAX_BINDINGS {
            return Err(BlobError::Invalid);
        }
        Self::new_resource_profile(
            local_home,
            entries,
            associations,
            Some(IssuerPolicy { issuer, routes }),
            Some(publications),
        )
    }
    fn new_resource_profile(
        local_home: String,
        entries: Vec<(ServiceRegistration, ServiceUse)>,
        associations: Vec<ResourceAssociation>,
        issuer: Option<IssuerPolicy>,
        publication: Option<Vec<PublicationAssociation>>,
    ) -> Result<Self, BlobError> {
        validate_home(&local_home)?;
        wire_host(&local_home)?;
        if entries.len() > MAX_BINDINGS
            || associations.is_empty()
            || associations.len() > MAX_BINDINGS
        {
            return Err(BlobError::Invalid);
        }
        for (i, (e, purpose)) in entries.iter().enumerate() {
            e.validate()?;
            if entries[..i].iter().any(|(old, _)| {
                old.identity() == e.identity() || old.certificate_sha256 == e.certificate_sha256
            }) {
                return Err(BlobError::Invalid);
            }
            match purpose {
                ServiceUse::Observation
                | ServiceUse::ResourceClient
                | ServiceUse::IssuerEndpoint
                    if e.home != local_home => {}
                ServiceUse::OutboundIntrospection if e.home == local_home => (),
                ServiceUse::OutboundObservation
                    if publication.is_some() && e.home == local_home => {}
                ServiceUse::Relay if issuer.is_some() && e.home == local_home => (),
                ServiceUse::Introspection if issuer.is_some() && e.home != local_home => (),
                _ => return Err(BlobError::Invalid),
            }
            if matches!(
                purpose,
                ServiceUse::ResourceClient
                    | ServiceUse::OutboundIntrospection
                    | ServiceUse::OutboundObservation
                    | ServiceUse::Relay
                    | ServiceUse::Introspection
            ) {
                issuer_client_id(&e.client_id)?;
            }
            if *purpose != ServiceUse::Observation {
                wire_host(&e.dns_san)?;
            }
        }
        let (registrations, uses) = entries.into_iter().unzip();
        let policy = Self {
            local_home,
            registrations,
            uses,
            issuer,
            resource: Some(associations),
            publication,
        };
        for (i, a) in policy.resource_associations().iter().enumerate() {
            a.validate(&policy)?;
            if policy.resource_associations()[..i]
                .iter()
                .any(|old| old.recipient_home == a.recipient_home)
            {
                return Err(BlobError::Invalid);
            }
        }
        for (i, a) in policy.publication_associations().iter().enumerate() {
            a.validate(&policy)?;
            if policy.publication_associations()[..i]
                .iter()
                .any(|old| old.recipient_home == a.recipient_home)
            {
                return Err(BlobError::Invalid);
            }
        }
        Ok(policy)
    }
    pub fn publication_associations(&self) -> &[PublicationAssociation] {
        self.publication.as_deref().unwrap_or(&[])
    }
    fn remote_tuples(&self) -> impl Iterator<Item = (&str, u64, u64)> {
        self.resource_associations()
            .iter()
            .map(|a| {
                (
                    a.recipient_home.as_str(),
                    a.remote_generation,
                    a.remote_reset,
                )
            })
            .chain(self.publication_associations().iter().map(|a| {
                (
                    a.recipient_home.as_str(),
                    a.remote_generation,
                    a.remote_reset,
                )
            }))
    }
    pub fn resource_associations(&self) -> &[ResourceAssociation] {
        self.resource.as_deref().unwrap_or(&[])
    }
    pub(crate) fn resource_registration(
        &self,
        home: &str,
        id: &str,
        purpose: ServiceUse,
    ) -> Result<&ServiceRegistration, BlobError> {
        self.registrations
            .iter()
            .find(|e| {
                e.home == home
                    && e.client_id == id
                    && self.service_use(&e.identity()) == Some(purpose)
            })
            .ok_or(BlobError::Denied)
    }
}
impl ResourceAssociation {
    fn validate(&self, policy: &ServicePolicy) -> Result<(), BlobError> {
        wire_host(&self.recipient_home)?;
        wire_host(&self.issuer)?;
        wire_host(&self.http_host)?;
        if self.recipient_home == policy.local_home
            || self.http_host != self.issuer
            || self.audience != format!("{}/attachments", policy.local_home)
            || self.ca_sha256 == [0; 32]
            || self.address.port() == 0
            || self.remote_generation == 0
        {
            return Err(BlobError::Invalid);
        }
        issuer_client_id(&self.relay_client_id)?;
        issuer_client_id(&self.outbound_client_id)?;
        validate_client_id(&self.endpoint_id)?;
        policy.resource_registration(
            &self.recipient_home,
            &self.relay_client_id,
            ServiceUse::ResourceClient,
        )?;
        policy.resource_registration(
            &policy.local_home,
            &self.outbound_client_id,
            ServiceUse::OutboundIntrospection,
        )?;
        policy.resource_registration(
            &self.recipient_home,
            &self.endpoint_id,
            ServiceUse::IssuerEndpoint,
        )?;
        Ok(())
    }
}
impl PublicationAssociation {
    fn validate(&self, policy: &ServicePolicy) -> Result<(), BlobError> {
        wire_host(&self.recipient_home)?;
        wire_host(&self.http_host)?;
        issuer_client_id(&self.outbound_client_id)?;
        validate_client_id(&self.endpoint_id)?;
        if self.recipient_home == policy.local_home
            || self.http_host != self.recipient_home
            || self.ca_sha256 == [0; 32]
            || self.address.port() == 0
            || self.remote_generation == 0
        {
            return Err(BlobError::Invalid);
        }
        policy.resource_registration(
            &policy.local_home,
            &self.outbound_client_id,
            ServiceUse::OutboundObservation,
        )?;
        policy.resource_registration(
            &self.recipient_home,
            &self.endpoint_id,
            ServiceUse::IssuerEndpoint,
        )?;
        if policy.resource_associations().iter().any(|a| {
            a.recipient_home == self.recipient_home
                && (a.remote_generation != self.remote_generation
                    || a.remote_reset != self.remote_reset)
        }) {
            return Err(BlobError::Invalid);
        }
        Ok(())
    }
}
impl Authority {
    pub(crate) fn validate_remote_homes(
        &self,
        registry: &Registry,
        history: &[RemoteFloor],
    ) -> Result<(), BlobError> {
        let mut homes: BTreeSet<&str> = registry.homes().iter().map(String::as_str).collect();
        for h in history {
            homes.insert(&h.home);
        }
        if homes.len() > crate::authority::MAX_HOMES {
            return Err(BlobError::Quota);
        }
        Ok(())
    }
    pub(crate) fn proposed_remote_history(
        &self,
        registry: &Registry,
        policy: &ServicePolicy,
    ) -> Result<Vec<RemoteFloor>, BlobError> {
        let mut history = self.remote_history.clone();
        for (home, generation, reset) in policy.remote_tuples() {
            if let Some(floor) = history.iter_mut().find(|r| r.home == home) {
                if generation < floor.generation || reset < floor.reset {
                    return Err(BlobError::Denied);
                }
                floor.generation = generation;
                floor.reset = reset;
            } else {
                history.push(RemoteFloor {
                    home: home.into(),
                    generation,
                    reset,
                });
            }
        }
        self.validate_remote_homes(registry, &history)?;
        Ok(history)
    }
}
#[cfg(feature = "durable-policy")]
impl crate::durable::Snapshot {
    pub(crate) fn validate_resource_snapshot(&self) -> Result<(), BlobError> {
        if (self.service_format_v5 && !self.service_format_v4)
            || (!self.service_format_v5
                && self
                    .service_policy
                    .as_ref()
                    .is_some_and(|p| p.publication.is_some()))
        {
            return Err(BlobError::Invalid);
        }
        if !self.service_format_v4
            && (!self.remote_history.is_empty()
                || self
                    .service_policy
                    .as_ref()
                    .is_some_and(|p| p.resource.is_some()))
        {
            return Err(BlobError::Invalid);
        }
        if self.remote_history.len() > crate::authority::MAX_HOMES {
            return Err(BlobError::Quota);
        }
        let mut homes: BTreeSet<&str> = self.registry.as_ref().map_or_else(BTreeSet::new, |r| {
            r.homes().iter().map(String::as_str).collect()
        });
        for (i, floor) in self.remote_history.iter().enumerate() {
            wire_host(&floor.home)?;
            if floor.generation == 0
                || self.remote_history[..i]
                    .iter()
                    .any(|old| old.home == floor.home)
            {
                return Err(BlobError::Invalid);
            }
            homes.insert(&floor.home);
        }
        if homes.len() > crate::authority::MAX_HOMES {
            return Err(BlobError::Quota);
        }
        if let Some(p) = &self.service_policy {
            for a in p.resource_associations() {
                a.validate(p)?;
            }
            for a in p.publication_associations() {
                a.validate(p)?;
            }
            for (home, generation, reset) in p.remote_tuples() {
                if !self
                    .remote_history
                    .iter()
                    .any(|h| h.home == home && h.generation == generation && h.reset == reset)
                {
                    return Err(BlobError::Invalid);
                }
            }
        }
        Ok(())
    }
}
