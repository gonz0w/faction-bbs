//! One exact sender operation after its authenticated incoming session is gone.
use super::*;
use crate::{
    State,
    retrieval::{Moment, Source},
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
// Private scheduling signals exist only in the unit-test binary.
#[cfg(test)]
struct TestGate {
    entered: std::sync::mpsc::Sender<()>,
    release: Mutex<std::sync::mpsc::Receiver<()>>,
}
struct Context {
    state: Weak<State>,
    instance: Arc<()>,
    policy: Arc<()>,
    principal: Principal,
    digest: [u8; 32],
    leaf: X509,
    generation: u64,
    reset: u64,
    binding: Binding,
    association: PublicationAssociation,
    action: PublicationAction,
    attempted: usize,
    deadline: Instant,
    cancel: Arc<AtomicBool>,
    clock: Mutex<Clock>,
    #[cfg(test)]
    authority_gates: [Option<TestGate>; 3],
}
impl Context {
    #[cfg(test)]
    fn authority_gate(&self, index: usize) -> Result<(), Error> {
        if let Some(gate) = &self.authority_gates[index] {
            gate.entered
                .send(())
                .map_err(|_| Error::Transport(tls::Stage::Denied))?;
            gate.release
                .lock()
                .map_err(|_| Error::Transport(tls::Stage::Denied))?
                .recv_timeout(self.deadline.saturating_duration_since(Instant::now()))
                .map_err(|_| Error::Transport(tls::Stage::Deadline))?;
        }
        Ok(())
    }
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
    fn current_locked<'a>(
        &self,
        a: &'a Authority,
        now: Duration,
    ) -> Result<&'a ServicePolicy, BlobError> {
        let wall = now.as_secs();
        if self.attempted == 0
            || self.attempted > tls::MAX_OPERATIONS
            || !Arc::ptr_eq(&self.instance, &a.instance)
            || !Arc::ptr_eq(&self.policy, &a.service_context)
            || self.generation != a.generation
            || self.reset != a.reset_epoch
            || Instant::now() >= self.deadline
            || self.cancel.load(Ordering::SeqCst)
            || !tls::leaf_current(&self.leaf, wall)
        {
            return Err(BlobError::Denied);
        }
        let r = a.registry.as_ref().ok_or(BlobError::Denied)?;
        let p = a.service_policy.as_ref().ok_or(BlobError::Denied)?;
        self.binding.validate()?;
        self.action.validate(&self.binding)?;
        if self.binding.sender != self.principal
            || self.binding.sender.account.split_once('@').map(|(_, h)| h)
                != Some(p.local_home.as_str())
            || self
                .binding
                .recipient
                .account
                .split_once('@')
                .map(|(_, h)| h)
                != Some(self.association.recipient_home.as_str())
            || !r.homes().contains(&p.local_home)
            || !r.homes().contains(&self.association.recipient_home)
            || !p.publication_associations().contains(&self.association)
            || wall < self.binding.created
            || (wall >= self.binding.expires && self.action != PublicationAction::Delete)
            || r.current(&self.principal, wall)?.certificate_sha256 != self.digest
            || !a.remote_history.iter().any(|h| {
                h.home == self.association.recipient_home
                    && h.generation == self.association.remote_generation
                    && h.reset == self.association.remote_reset
            })
        {
            return Err(BlobError::Denied);
        }
        for e in [
            p.resource_registration(
                &p.local_home,
                &self.association.outbound_client_id,
                ServiceUse::OutboundObservation,
            )?,
            p.resource_registration(
                &self.association.recipient_home,
                &self.association.endpoint_id,
                ServiceUse::IssuerEndpoint,
            )?,
        ] {
            if !e.enabled
                || wall < e.valid_from
                || wall >= e.valid_until
                || !r.homes().contains(&e.home)
            {
                return Err(BlobError::Denied);
            }
        }
        Ok(p)
    }
    fn current(&self) -> Result<Arc<State>, Error> {
        self.moment()?;
        let state = self
            .state
            .upgrade()
            .ok_or(Error::Store(BlobError::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Store(BlobError::Denied));
        }
        #[cfg(test)]
        self.authority_gate(0)?;
        {
            let a = state
                .authority
                .lock()
                .map_err(|_| Error::Store(BlobError::Denied))?;
            if state.stopped.load(Ordering::SeqCst) {
                return Err(Error::Store(BlobError::Denied));
            }
            self.current_locked(&a, self.moment()?)
                .map_err(Error::Store)?;
        }
        Ok(state)
    }
}
/// Consumes one exact sender operation; no actor, grant, handle or incoming TLS escapes.
/// ```compile_fail
/// let forged=faction_provider::services::PendingPublication {};
/// ```
/// ```compile_fail
/// fn clone(p:faction_provider::services::PendingPublication){let _=p.clone();}
/// ```
/// ```compile_fail
/// fn log(p:faction_provider::services::PendingPublication){println!("{p:?}");}
/// ```
/// ```compile_fail
/// fn log(p:faction_provider::services::PendingPublication){println!("{p}");}
/// ```
/// ```compile_fail
/// fn grant<T:faction_files::private_blobs::Access>(){}
/// grant::<faction_provider::services::PendingPublication>();
/// ```
pub struct PendingPublication {
    context: Arc<Context>,
    request: Request,
    _lease: Arc<()>,
}
fn exact_action(b: &Binding, request: &Request) -> Result<PublicationAction, BlobError> {
    b.validate()?;
    let action = match request {
        Request::Begin(value) if value == b => PublicationAction::Begin,
        Request::PutChunk { id, offset, bytes }
            if *id == b.id && !bytes.is_empty() && bytes.len() <= CHUNK =>
        {
            PublicationAction::PutChunk {
                offset: *offset,
                length: bytes.len() as u64,
                ciphertext_sha256: sha256(bytes),
            }
        }
        Request::Commit { id } if *id == b.id => PublicationAction::Commit,
        Request::Status { id } if *id == b.id => PublicationAction::Status,
        Request::Delete { id } if *id == b.id => PublicationAction::Delete,
        _ => return Err(BlobError::Denied),
    };
    action.validate(b)?;
    Ok(action)
}
pub(crate) struct PublicationSource<'a> {
    pub(crate) session: Source<'a>,
    pub(crate) attempted: usize,
    pub(crate) cancel: Arc<AtomicBool>,
    pub(crate) initial: Duration,
}
impl Authority {
    pub(crate) fn prepare_publication_source(
        &mut self,
        state: &Arc<State>,
        source: PublicationSource<'_>,
        binding: Binding,
        request: Request,
    ) -> Result<PendingPublication, Error> {
        let PublicationSource {
            session: source,
            attempted,
            cancel,
            initial,
        } = source;
        let action = exact_action(&binding, &request).map_err(Error::Store)?;
        if !Arc::ptr_eq(&self.instance, &source.connection.authority)
            || !Arc::ptr_eq(&self.instance, &source.handle.authority)
            || !Arc::ptr_eq(&source.connection.closed, &source.handle.connection)
            || source.connection.closed.load(Ordering::Relaxed)
        {
            return Err(Error::Store(BlobError::Denied));
        }
        let slot = self
            .slots
            .iter()
            .find(|s| Arc::ptr_eq(&s.token, &source.handle.token))
            .ok_or(Error::Store(BlobError::Denied))?;
        if !slot.current(source.elapsed_ms)
            || slot.generation != self.generation
            || slot.reset_epoch != self.reset_epoch
            || !Arc::ptr_eq(&slot.connection, &source.connection.closed)
            || binding.sender != slot.principal
            || sha256(
                &source
                    .leaf
                    .to_der()
                    .map_err(|_| Error::Store(BlobError::Denied))?,
            ) != slot.certificate_sha256
        {
            return Err(Error::Store(BlobError::Denied));
        }
        let association = self
            .service_policy
            .as_ref()
            .and_then(|p| {
                p.publication_associations().iter().find(|a| {
                    binding.recipient.account.split_once('@').map(|(_, h)| h)
                        == Some(a.recipient_home.as_str())
                })
            })
            .cloned()
            .ok_or(Error::Store(BlobError::Denied))?;
        let context = Arc::new(Context {
            state: Arc::downgrade(state),
            instance: self.instance.clone(),
            policy: self.service_context.clone(),
            principal: slot.principal.clone(),
            digest: slot.certificate_sha256,
            leaf: source.leaf.clone(),
            generation: self.generation,
            reset: self.reset_epoch,
            binding,
            association,
            action,
            attempted,
            deadline: source.deadline,
            cancel,
            clock: Mutex::new(Clock {
                highwater: initial,
                invalid: false,
            }),
            #[cfg(test)]
            authority_gates: std::array::from_fn(|_| None),
        });
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Store(BlobError::Denied));
        }
        let now = context.moment()?;
        context.current_locked(self, now).map_err(Error::Store)?;
        let lease = self
            .records
            .reserve_delegated(Moment {
                wall: now.as_secs(),
                mono: Instant::now(),
            })
            .map_err(Error::Store)?;
        Ok(PendingPublication {
            context,
            request,
            _lease: lease,
        })
    }
}
fn hex(v: &[u8; 32]) -> String {
    v.iter().map(|v| format!("{v:02x}")).collect()
}
fn query(c: &Context) -> codec::UnverifiedPublicationQuery {
    let principal = |p: &Principal| codec::UnverifiedPrincipal {
        account: p.account.clone(),
        device: p.device,
        identity: hex(&p.identity),
    };
    codec::UnverifiedPublicationQuery {
        client_id: c.association.outbound_client_id.clone(),
        binding: codec::UnverifiedBinding {
            sender: principal(&c.binding.sender),
            recipient: principal(&c.binding.recipient),
            id: hex(&c.binding.id),
            size: c.binding.size,
            created: c.binding.created,
            expires: c.binding.expires,
        },
        action: match c.action {
            PublicationAction::Begin => codec::UnverifiedPublicationAction::Begin,
            PublicationAction::PutChunk {
                offset,
                length,
                ciphertext_sha256,
            } => codec::UnverifiedPublicationAction::PutChunk {
                offset,
                length,
                ciphertext_sha256: hex(&ciphertext_sha256),
            },
            PublicationAction::Commit => codec::UnverifiedPublicationAction::Commit,
            PublicationAction::Status => codec::UnverifiedPublicationAction::Status,
            PublicationAction::Delete => codec::UnverifiedPublicationAction::Delete,
        },
    }
}
fn verify(
    c: &Context,
    p: &ServicePolicy,
    query: &codec::UnverifiedPublicationQuery,
    metadata: &codec::UnverifiedPublicationObservation,
    started: Duration,
    now: Duration,
) -> Result<(), BlobError> {
    let codec::UnverifiedPublicationObservation::Current {
        query: observed,
        home,
        service_home,
        service_certificate_sha256,
        recipient_certificate_sha256,
        generation,
        reset,
        checked_at,
    } = metadata
    else {
        return Err(BlobError::Denied);
    };
    let outgoing = p.resource_registration(
        &p.local_home,
        &c.association.outbound_client_id,
        ServiceUse::OutboundObservation,
    )?;
    if observed.as_ref() != query
        || home != &c.association.recipient_home
        || service_home != &p.local_home
        || service_certificate_sha256 != &hex(&outgoing.certificate_sha256)
        || recipient_certificate_sha256 == &"00".repeat(32)
        || *generation != c.association.remote_generation
        || *reset != c.association.remote_reset
        || *checked_at < started.as_secs()
        || *checked_at > now.as_secs()
    {
        return Err(BlobError::Denied);
    }
    Ok(())
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
        self.context
            .current_locked(self.authority, Duration::from_secs(now))?;
        if actor != &self.context.principal
            || binding != &self.context.binding
            || !matches!(
                (self.context.action, action),
                (PublicationAction::Begin, Action::Begin)
                    | (PublicationAction::PutChunk { .. }, Action::PutChunk)
                    | (PublicationAction::Commit, Action::Commit)
                    | (PublicationAction::Status, Action::Status)
                    | (PublicationAction::Delete, Action::Delete)
            )
        {
            return Err(BlobError::Denied);
        }
        Ok(())
    }
}
impl PendingPublication {
    pub fn execute(
        self,
        endpoint: &tls::HttpEndpoint,
        store: &Mutex<Store>,
    ) -> Result<Response, Error> {
        let state = self.context.current()?;
        #[cfg(test)]
        self.context.authority_gate(1)?;
        {
            let a = state
                .authority
                .lock()
                .map_err(|_| Error::Store(BlobError::Denied))?;
            if state.stopped.load(Ordering::SeqCst) {
                return Err(Error::Store(BlobError::Denied));
            }
            let p = self
                .context
                .current_locked(&a, self.context.moment()?)
                .map_err(Error::Store)?;
            let outgoing = p
                .resource_registration(
                    &p.local_home,
                    &self.context.association.outbound_client_id,
                    ServiceUse::OutboundObservation,
                )
                .map_err(Error::Store)?;
            let remote = p
                .resource_registration(
                    &self.context.association.recipient_home,
                    &self.context.association.endpoint_id,
                    ServiceUse::IssuerEndpoint,
                )
                .map_err(Error::Store)?;
            if !endpoint.matches_publication(&self.context.association, outgoing, remote) {
                return Err(Error::Store(BlobError::Denied));
            }
        }
        let query = query(&self.context);
        let started = self.context.moment()?;
        let context = self.context.clone();
        let probe = Arc::new(move || context.current().is_ok());
        let metadata = endpoint
            .publication_checked(
                &query,
                self.context.deadline,
                self.context.cancel.clone(),
                probe,
            )
            .map_err(Error::Transport)?;
        let mut store = store
            .try_lock()
            .map_err(|_| Error::Store(BlobError::Busy))?;
        #[cfg(test)]
        self.context.authority_gate(2)?;
        let a = state
            .authority
            .lock()
            .map_err(|_| Error::Store(BlobError::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Store(BlobError::Denied));
        }
        let now = self.context.moment()?;
        let p = self.context.current_locked(&a, now).map_err(Error::Store)?;
        verify(&self.context, p, &query, &metadata, started, now).map_err(Error::Store)?;
        if exact_action(&self.context.binding, &self.request).map_err(Error::Store)?
            != self.context.action
        {
            return Err(Error::Store(BlobError::Denied));
        }
        let result = store
            .dispatch(
                &self.context.principal,
                &VerifiedAccess {
                    authority: &a,
                    context: &self.context,
                },
                self.request.clone(),
                now.as_secs(),
            )
            .map_err(Error::Store)?;
        self.context
            .current_locked(&a, self.context.moment()?)
            .map_err(Error::Store)?;
        Ok(result)
    }
}

#[cfg(test)]
#[path = "publication_tests.rs"]
mod tests;
