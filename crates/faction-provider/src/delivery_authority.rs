//! Private bridge from established TLS provenance to the one continuation owner.
use super::*;
use crate::{
    Error as ProviderError, Runtime, State,
    retrieval::{Moment, RetrievalAction, RetrievalPolicy, RetrievalRecord, Source},
    services::{ServiceCredential, ServicePolicy},
    tls,
};
use openssl::{sha::sha256, x509::X509};
use std::sync::{Arc, atomic::Ordering};
pub(crate) enum Authenticated {
    Device(Connection, Handle),
    Service(ServiceCredential),
}
impl Authority {
    pub(crate) fn authenticate_composed(
        &mut self,
        leaf: &X509,
        now: u64,
        elapsed: u64,
    ) -> Result<Authenticated, Error> {
        self.relay_policy()?;
        let digest = sha256(&leaf.to_der().map_err(|_| Error::Denied)?);
        if self
            .registry
            .as_ref()
            .is_some_and(|r| r.bindings.iter().any(|b| b.certificate_sha256 == digest))
        {
            let connection = self.open_connection();
            let handle = self.authenticate(&connection, digest, now, elapsed)?;
            Ok(Authenticated::Device(connection, handle))
        } else {
            self.authenticate_service(leaf, now)
                .map(Authenticated::Service)
        }
    }
    pub(crate) fn composed_policy(&self, home: &str) -> Result<ServicePolicy, Error> {
        let p = self.service_policy.as_ref().ok_or(Error::Denied)?;
        if p.local_home() != home
            || p.issuer.is_none()
            || p.resource_associations().is_empty()
            || p.publication_associations().is_empty()
            || self.registry.as_ref().is_none_or(|r| {
                r.bindings()
                    .iter()
                    .any(|b| b.principal.account.split_once('@').map(|(_, h)| h) != Some(home))
            })
        {
            return Err(Error::Denied);
        }
        Ok(p.clone())
    }
    pub(crate) fn capture_delivery_record(
        &mut self,
        state: &Arc<State>,
        source: Source<'_>,
        binding: Binding,
        request: &Request,
    ) -> Result<(RetrievalRecord, Arc<()>, String), ProviderError> {
        let action = match request {
            Request::GetChunk { id, offset } if *id == binding.id && *offset <= binding.size => {
                RetrievalAction::GetChunk { offset: *offset }
            }
            Request::Status { id } if *id == binding.id => RetrievalAction::Status,
            _ => return Err(ProviderError::Store(Error::Denied)),
        };
        let local = binding
            .recipient
            .account
            .split_once('@')
            .map(|(_, h)| h)
            .ok_or(ProviderError::Store(Error::Denied))?;
        let p = self.composed_policy(local).map_err(ProviderError::Store)?;
        let sender = binding
            .sender
            .account
            .split_once('@')
            .map(|(_, h)| h)
            .ok_or(ProviderError::Store(Error::Denied))?;
        let audience = p
            .issuer
            .as_ref()
            .and_then(|issuer| issuer.routes.iter().find(|r| r.sender_home == sender))
            .map(|r| r.audience.clone())
            .ok_or(ProviderError::Store(Error::Denied))?;
        let lease = self
            .records
            .reserve_delegated(Moment::now())
            .map_err(ProviderError::Store)?;
        let token = self
            .create_retrieval(source, binding, action, Moment::now())
            .map_err(ProviderError::Store)?;
        Ok((RetrievalRecord::new(state, token), lease, audience))
    }
}
impl Authority {
    pub(crate) fn relay_policy(&self) -> Result<ServicePolicy, Error> {
        let home = self
            .service_policy
            .as_ref()
            .ok_or(Error::Denied)?
            .local_home();
        self.composed_policy(home)
    }
    pub(crate) fn current_service_use(
        &self,
        c: &ServiceCredential,
        leaf: &X509,
        now: u64,
    ) -> Result<crate::services::ServiceUse, Error> {
        if !c.live
            || c.generation != self.generation
            || c.reset_epoch != self.reset_epoch
            || now < c.wall_highwater
            || !Arc::ptr_eq(&c.context, &self.service_context)
        {
            return Err(Error::Denied);
        }
        let p = self.service_policy.as_ref().ok_or(Error::Denied)?;
        p.current(
            &c.identity,
            c.digest,
            leaf,
            now,
            self.registry.as_ref().ok_or(Error::Denied)?,
        )?;
        p.service_use(&c.identity).ok_or(Error::Denied)
    }
}
impl Runtime {
    pub fn configure_composed_relay(&self, home: &str) -> Result<(), ProviderError> {
        let mut authority = self
            .state
            .authority
            .lock()
            .map_err(|_| ProviderError::Transport(tls::Stage::Denied))?;
        if self.state.stopped.load(Ordering::SeqCst) {
            return Err(ProviderError::Transport(tls::Stage::Denied));
        }
        let p = authority
            .composed_policy(home)
            .map_err(ProviderError::Store)?;
        let audiences = p
            .issuer
            .as_ref()
            .ok_or(ProviderError::Store(Error::Denied))?
            .routes
            .iter()
            .map(|r| (r.sender_home.clone(), r.audience.clone()))
            .collect();
        let policy = RetrievalPolicy::new(home.into(), audiences).map_err(ProviderError::Store)?;
        authority.install_delivery_retrieval_policy(policy);
        Ok(())
    }
}
