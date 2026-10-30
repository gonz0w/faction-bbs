//! One consuming foreign-recipient retrieval. No actor/grant leaves this module.
use super::*;
use crate::{
    State,
    retrieval::{Moment, RetrievalAction},
};
use faction_delegation_codec as codec;
use faction_files::private_blobs::{Access, Action, Request, Response, Store};
use std::{
    sync::{
        Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
struct Clock {
    highwater: Duration,
    invalid: bool,
}
impl Clock {
    fn sample(&mut self) -> Result<Duration, BlobError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| {
            self.invalid = true;
            BlobError::Clock
        })?;
        if self.invalid || now < self.highwater {
            self.invalid = true;
            return Err(BlobError::Clock);
        }
        self.highwater = now;
        Ok(now)
    }
}
struct Context {
    state: Weak<State>,
    credential: ServiceCredential,
    leaf: X509,
    binding: Binding,
    association: ResourceAssociation,
    action: RetrievalAction,
    attempted: usize,
    deadline: Instant,
    cancel: Arc<AtomicBool>,
    clock: Mutex<Clock>,
}
impl Context {
    fn moment(&self) -> Result<Duration, Error> {
        if self.cancel.load(Ordering::SeqCst) {
            return Err(Error::Transport(tls::Stage::Cancelled));
        }
        if Instant::now() >= self.deadline {
            return Err(Error::Transport(tls::Stage::Deadline));
        }
        self.clock
            .lock()
            .map_err(|_| Error::Store(BlobError::Clock))?
            .sample()
            .map_err(Error::Store)
    }
    fn current(&self) -> Result<Arc<State>, Error> {
        // Fast refusal before waiting; this sample cannot qualify later I/O.
        self.moment()?;
        let state = self
            .state
            .upgrade()
            .ok_or(Error::Store(BlobError::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Store(BlobError::Denied));
        }
        {
            let a = state
                .authority
                .lock()
                .map_err(|_| Error::Store(BlobError::Denied))?;
            if state.stopped.load(Ordering::SeqCst) {
                return Err(Error::Store(BlobError::Denied));
            }
            let now = self.moment()?;
            a.resource_current(self, now.as_secs())
                .map_err(Error::Store)?;
        }
        Ok(state)
    }
}
/// Private noncloneable continuation. Incoming TLS ownership ends before return.
/// No token, authority object, Store borrow or same-carrier reply is exposed.
/// ```compile_fail
/// let forged=faction_provider::services::PendingRetrieval {};
/// ```
/// ```compile_fail
/// fn clone(p:faction_provider::services::PendingRetrieval){let _=p.clone();}
/// ```
/// ```compile_fail
/// fn log(p:faction_provider::services::PendingRetrieval){println!("{p:?}");}
/// ```
/// ```compile_fail
/// fn display(p:faction_provider::services::PendingRetrieval){println!("{p}");}
/// ```
/// ```compile_fail
/// fn grant<T:faction_files::private_blobs::Access>(){}
/// grant::<faction_provider::services::PendingRetrieval>();
/// ```
pub struct PendingRetrieval {
    context: Arc<Context>,
    request: Request,
    token: [u8; 32],
    _lease: Arc<()>,
}
impl tls::ServiceSession {
    pub fn prepare_retrieval(
        mut self,
        binding: Binding,
        request: Request,
        token: codec::UnverifiedToken,
    ) -> Result<PendingRetrieval, Error> {
        let state = self.issuer_attempt()?;
        let action = exact_action(&binding, &request).map_err(Error::Store)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::Store(BlobError::Clock))?;
        let token = token
            .decode_bytes()
            .map_err(|_| Error::Store(BlobError::Denied))?;
        let mut a = state
            .authority
            .lock()
            .map_err(|_| Error::Store(BlobError::Denied))?;
        let association = a
            .service_policy
            .as_ref()
            .and_then(|p| {
                p.resource_associations().iter().find(|r| {
                    binding.recipient.account.split_once('@').map(|(_, h)| h)
                        == Some(r.recipient_home.as_str())
                        && r.relay_client_id == self.credential.identity.client_id
                })
            })
            .cloned()
            .ok_or(Error::Store(BlobError::Denied))?;
        let context = Arc::new(Context {
            state: Arc::downgrade(&state),
            credential: self.credential.clone(),
            leaf: self.leaf.clone(),
            binding,
            association,
            action,
            attempted: self.original_service_attempts(),
            deadline: self.deadline,
            cancel: self.cancel.clone(),
            clock: Mutex::new(Clock {
                highwater: now,
                invalid: false,
            }),
        });
        a.resource_current(&context, now.as_secs())
            .map_err(Error::Store)?;
        let lease = a
            .records
            .reserve_delegated(Moment {
                wall: now.as_secs(),
                mono: Instant::now(),
            })
            .map_err(Error::Store)?;
        drop(a);
        let pending = PendingRetrieval {
            context,
            request,
            token,
            _lease: lease,
        };
        // Fully finish ServiceSession::drop and its TLS fields before returning.
        drop(self);
        Ok(pending)
    }
}
fn exact_action(binding: &Binding, request: &Request) -> Result<RetrievalAction, BlobError> {
    match request {
        Request::GetChunk { id, offset } if *id == binding.id && *offset <= binding.size => {
            Ok(RetrievalAction::GetChunk { offset: *offset })
        }
        Request::Status { id } if *id == binding.id => Ok(RetrievalAction::Status),
        _ => Err(BlobError::Denied),
    }
}
impl Authority {
    fn resource_current<'a>(
        &'a self,
        c: &Context,
        now: u64,
    ) -> Result<&'a ServicePolicy, BlobError> {
        let cred = &c.credential;
        if c.attempted == 0
            || c.attempted > tls::MAX_OPERATIONS
            || !cred.live
            || now < cred.wall_highwater
            || !Arc::ptr_eq(&cred.context, &self.service_context)
            || cred.generation != self.generation
            || cred.reset_epoch != self.reset_epoch
            || Instant::now() >= c.deadline
            || c.cancel.load(Ordering::SeqCst)
        {
            return Err(BlobError::Denied);
        }
        let r = self.registry.as_ref().ok_or(BlobError::Denied)?;
        let p = self.service_policy.as_ref().ok_or(BlobError::Denied)?;
        if p.service_use(&cred.identity) != Some(ServiceUse::ResourceClient)
            || !p.resource_associations().contains(&c.association)
            || c.binding.sender.account.split_once('@').map(|(_, h)| h)
                != Some(p.local_home.as_str())
            || c.binding.recipient.account.split_once('@').map(|(_, h)| h)
                != Some(c.association.recipient_home.as_str())
            || cred.identity.home != c.association.recipient_home
            || cred.identity.client_id != c.association.relay_client_id
        {
            return Err(BlobError::Denied);
        }
        c.binding.validate()?;
        if now < c.binding.created || now >= c.binding.expires {
            return Err(BlobError::Denied);
        }
        p.current(&cred.identity, cred.digest, &c.leaf, now, r)?;
        r.current(&c.binding.sender, now)?;
        if !self.remote_history.iter().any(|h| {
            h.home == c.association.recipient_home
                && h.generation == c.association.remote_generation
                && h.reset == c.association.remote_reset
        }) {
            return Err(BlobError::Denied);
        }
        for entry in [
            p.resource_registration(
                &p.local_home,
                &c.association.outbound_client_id,
                ServiceUse::OutboundIntrospection,
            )?,
            p.resource_registration(
                &c.association.recipient_home,
                &c.association.endpoint_id,
                ServiceUse::IssuerEndpoint,
            )?,
        ] {
            if !entry.enabled
                || now < entry.valid_from
                || now >= entry.valid_until
                || !r.homes().contains(&entry.home)
            {
                return Err(BlobError::Denied);
            }
        }
        Ok(p)
    }
}
fn hex(v: &[u8; 32]) -> String {
    v.iter().map(|b| format!("{b:02x}")).collect()
}
fn wire_principal(p: &Principal) -> codec::UnverifiedPrincipal {
    codec::UnverifiedPrincipal {
        account: p.account.clone(),
        device: p.device,
        identity: hex(&p.identity),
    }
}
fn verify_metadata(
    c: &Context,
    metadata: &codec::UnverifiedIntrospection,
    now: Duration,
) -> Result<u64, BlobError> {
    let codec::UnverifiedIntrospection::Retrieval { claims, action } = metadata else {
        return Err(BlobError::Denied);
    };
    let binding = codec::UnverifiedBinding {
        sender: wire_principal(&c.binding.sender),
        recipient: wire_principal(&c.binding.recipient),
        id: hex(&c.binding.id),
        size: c.binding.size,
        created: c.binding.created,
        expires: c.binding.expires,
    };
    let expected = match c.action {
        RetrievalAction::GetChunk { offset } => {
            codec::UnverifiedRetrievalAction::GetChunk { offset }
        }
        RetrievalAction::Status => codec::UnverifiedRetrievalAction::Status,
    };
    if claims.issuer != c.association.issuer
        || claims.issuer_home != c.association.recipient_home
        || claims.client_id != c.association.relay_client_id
        || claims.audience != c.association.audience
        || claims.subject != c.binding.recipient.account
        || claims.binding != binding
        || *action != expected
        || claims.generation != c.association.remote_generation
        || claims.reset != c.association.remote_reset
        || *claims.certificate_thumbprint.expose_bytes() != c.credential.digest
        || claims.issued_at > now.as_secs()
        || Duration::from_secs(claims.expires_at) <= now
    {
        return Err(BlobError::Denied);
    }
    Ok(claims.expires_at)
}
struct VerifiedAccess<'a> {
    authority: &'a Authority,
    context: &'a Context,
}
impl Access for VerifiedAccess<'_> {
    fn require(
        &self,
        actor: &Principal,
        binding: &Binding,
        action: Action,
        now: u64,
    ) -> Result<(), BlobError> {
        self.authority.resource_current(self.context, now)?;
        if actor != &self.context.binding.recipient
            || binding != &self.context.binding
            || !matches!(
                (self.context.action, action),
                (RetrievalAction::GetChunk { .. }, Action::GetChunk)
                    | (RetrievalAction::Status, Action::Status)
            )
        {
            return Err(BlobError::Denied);
        }
        Ok(())
    }
}
impl PendingRetrieval {
    /// Consume one continuation and return only the bounded Store result to the
    /// trusted embedder. No lock or exclusive Store borrow spans callback I/O.
    pub fn execute(
        self,
        endpoint: &tls::HttpEndpoint,
        store: &Mutex<Store>,
    ) -> Result<Response, Error> {
        let state = self.context.current()?;
        {
            let a = state
                .authority
                .lock()
                .map_err(|_| Error::Store(BlobError::Denied))?;
            let p = a
                .resource_current(&self.context, self.context.moment()?.as_secs())
                .map_err(Error::Store)?;
            let outgoing = p
                .resource_registration(
                    &p.local_home,
                    &self.context.association.outbound_client_id,
                    ServiceUse::OutboundIntrospection,
                )
                .map_err(Error::Store)?;
            let issuer = p
                .resource_registration(
                    &self.context.association.recipient_home,
                    &self.context.association.endpoint_id,
                    ServiceUse::IssuerEndpoint,
                )
                .map_err(Error::Store)?;
            if !endpoint.matches_resource(&self.context.association, outgoing, issuer) {
                return Err(Error::Store(BlobError::Denied));
            }
        }
        let context = self.context.clone();
        let probe = Arc::new(move || context.current().is_ok());
        let metadata = endpoint
            .introspect_checked(
                &codec::UnverifiedIntrospectionRequest {
                    client_id: self.context.association.outbound_client_id.clone(),
                    token: codec::UnverifiedToken::from_bytes(self.token),
                },
                self.context.deadline,
                self.context.cancel.clone(),
                probe,
            )
            .map_err(Error::Transport)?;
        verify_metadata(&self.context, &metadata, self.context.moment()?).map_err(Error::Store)?;
        // Same order as ordinary dispatch: Store, then Authority. No queued wait.
        let mut store = store
            .try_lock()
            .map_err(|_| Error::Store(BlobError::Busy))?;
        let a = state
            .authority
            .lock()
            .map_err(|_| Error::Store(BlobError::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Store(BlobError::Denied));
        }
        let now = self.context.moment()?;
        a.resource_current(&self.context, now.as_secs())
            .map_err(Error::Store)?;
        verify_metadata(&self.context, &metadata, now).map_err(Error::Store)?;
        if exact_action(&self.context.binding, &self.request).map_err(Error::Store)?
            != self.context.action
        {
            return Err(Error::Store(BlobError::Denied));
        }
        let access = VerifiedAccess {
            authority: &a,
            context: &self.context,
        };
        let response = store
            .dispatch(
                &self.context.binding.recipient,
                &access,
                self.request.clone(),
                now.as_secs(),
            )
            .map_err(Error::Store)?;
        verify_metadata(&self.context, &metadata, self.context.moment()?).map_err(Error::Store)?;
        Ok(response)
    }
}

#[cfg(test)]
#[path = "resource_tests.rs"]
mod tests;
