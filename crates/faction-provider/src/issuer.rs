//! Private, current-checked retrieval token authority. No Store or sender grant.
use super::*;
use crate::services::{IssuerRoute, ServiceCredential, ServiceRegistration, ServiceUse};
use openssl::{asn1::Asn1Time, memcmp, rand::rand_bytes};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Sensitive server reference. Only provisioning an authenticated record constructs it.
/// ```compile_fail
/// use faction_provider::issuer::OperationReference;
/// let forged = OperationReference([0;32]);
/// ```
/// ```compile_fail
/// fn log(x: faction_provider::issuer::OperationReference) { println!("{x:?}"); }
/// ```
/// ```compile_fail
/// fn log(x: faction_provider::issuer::OperationReference) { println!("{x}"); }
/// ```
pub struct OperationReference([u8; 32]);
impl OperationReference {
    pub fn expose_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
/// Sensitive certificate-bound token. Exposure is explicit; it grants no local actor.
/// ```compile_fail
/// use faction_provider::issuer::RetrievalToken;
/// let forged = RetrievalToken([0;32]);
/// ```
/// ```compile_fail
/// fn log(x: faction_provider::issuer::RetrievalToken) { println!("{x:?}"); }
/// ```
/// ```compile_fail
/// fn log(x: faction_provider::issuer::RetrievalToken) { println!("{x}"); }
/// ```
pub struct RetrievalToken([u8; 32]);
impl RetrievalToken {
    pub fn expose_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
pub struct IssuedToken {
    token: RetrievalToken,
    expires_in: u64,
    expires_at: u64,
}
impl IssuedToken {
    pub fn token(&self) -> &RetrievalToken {
        &self.token
    }
    pub fn expires_in(&self) -> u64 {
        self.expires_in
    }
    pub fn expires_at(&self) -> u64 {
        self.expires_at
    }
}
/// Current issuer metadata, never a device actor, Access, sender check or object proof.
/// ```compile_fail
/// use faction_provider::issuer::RetrievalMetadata;
/// let forged = RetrievalMetadata {};
/// ```
/// ```compile_fail
/// fn grant<T:faction_files::private_blobs::Access>() {}
/// grant::<faction_provider::issuer::RetrievalMetadata>();
/// ```
pub struct RetrievalMetadata {
    binding: Binding,
    action: RetrievalAction,
    audience: String,
    issuer: String,
    issuer_home: String,
    client_id: String,
    relay_certificate_sha256: [u8; 32],
    issued_at: u64,
    expires_at: u64,
    generation: u64,
    reset_epoch: u64,
}
impl RetrievalMetadata {
    pub fn binding(&self) -> &Binding {
        &self.binding
    }
    pub fn action(&self) -> RetrievalAction {
        self.action
    }
    pub fn audience(&self) -> &str {
        &self.audience
    }
    pub fn issuer(&self) -> &str {
        &self.issuer
    }
    pub fn issuer_home(&self) -> &str {
        &self.issuer_home
    }
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
    pub fn relay_certificate_sha256(&self) -> [u8; 32] {
        self.relay_certificate_sha256
    }
    pub fn issued_at(&self) -> u64 {
        self.issued_at
    }
    pub fn expires_at(&self) -> u64 {
        self.expires_at
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn reset_epoch(&self) -> u64 {
        self.reset_epoch
    }
}
pub(super) struct ReferenceState {
    reference: [u8; 32],
    issuer: String,
    local_home: String,
    route: IssuerRoute,
    relay: ServiceRegistration,
    introspector: ServiceRegistration,
    context: Arc<()>,
    token: Option<TokenState>,
}
struct TokenState {
    secret: [u8; 32],
    leaf: X509,
    issued_at: u64,
    expires_at: u64,
    deadline: Instant,
}
/// Samples bracket the precise wall read. Tests may construct it only inside this module.
#[derive(Clone, Copy)]
struct PreciseMoment {
    wall: Duration,
    before: Instant,
    after: Instant,
}
impl PreciseMoment {
    fn now() -> Result<Self, BlobError> {
        let before = Instant::now();
        let wall = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| BlobError::Clock)?;
        let after = Instant::now();
        Ok(Self {
            wall,
            before,
            after,
        })
    }
}
fn certificate_expiry(leaf: &X509) -> Result<u64, BlobError> {
    let epoch = Asn1Time::from_unix(0).map_err(|_| BlobError::Denied)?;
    let diff = epoch
        .diff(leaf.not_after())
        .map_err(|_| BlobError::Denied)?;
    let seconds = i64::from(diff.days)
        .checked_mul(86400)
        .and_then(|d| d.checked_add(i64::from(diff.secs)))
        .ok_or(BlobError::Denied)?;
    u64::try_from(seconds).map_err(|_| BlobError::Denied)
}
fn remaining(token: &TokenState, now: PreciseMoment) -> Result<u64, BlobError> {
    let mono = token
        .deadline
        .checked_duration_since(now.after)
        .ok_or(BlobError::Expired)?;
    let wall = Duration::from_secs(token.expires_at)
        .checked_sub(now.wall)
        .ok_or(BlobError::Expired)?;
    let seconds = mono.min(wall).as_secs();
    if seconds == 0 {
        Err(BlobError::Expired)
    } else {
        Ok(seconds)
    }
}
fn lifetime(
    slot: &RecordSlot,
    relay: &ServiceRegistration,
    leaf: &X509,
    device_until: u64,
    now: PreciseMoment,
) -> Result<(u64, Instant), BlobError> {
    let available = slot
        .deadline
        .checked_duration_since(now.after)
        .ok_or(BlobError::Expired)?
        .min(tls::CYCLE_LIMIT);
    let cap = now
        .wall
        .checked_add(available)
        .ok_or(BlobError::Expired)?
        .as_secs();
    let exp = cap
        .min(slot.binding.expires)
        .min(device_until)
        .min(certificate_expiry(&slot.leaf)?)
        .min(relay.valid_until)
        .min(certificate_expiry(leaf)?);
    let delta = Duration::from_secs(exp)
        .checked_sub(now.wall)
        .ok_or(BlobError::Expired)?;
    let deadline = now
        .before
        .checked_add(delta)
        .ok_or(BlobError::Expired)?
        .min(slot.deadline);
    let token = TokenState {
        secret: [0; 32],
        leaf: leaf.clone(),
        issued_at: now.wall.as_secs(),
        expires_at: exp,
        deadline,
    };
    remaining(&token, now)?;
    Ok((exp, deadline))
}
impl Authority {
    fn live_moment(&mut self) -> Result<PreciseMoment, BlobError> {
        PreciseMoment::now().inspect_err(|_| self.records.invalidate())
    }
    fn issuer_prune(&mut self, now: PreciseMoment) -> Result<(), BlobError> {
        if now.before > now.after
            || self
                .records
                .precise_highwater
                .is_some_and(|old| now.wall < old)
        {
            self.records.invalidate();
            return Err(BlobError::Clock);
        }
        self.records.precise_highwater = Some(
            self.records
                .precise_highwater
                .map_or(now.wall, |old| old.max(now.wall)),
        );
        self.records.prune(Moment {
            wall: now.wall.as_secs(),
            mono: now.after,
        })
    }
    fn issuer_slot(&mut self, token: &Arc<()>, now: PreciseMoment) -> Result<usize, BlobError> {
        self.issuer_prune(now)?;
        let s = self
            .records
            .slots
            .iter()
            .find(|s| Arc::ptr_eq(&s.token, token))
            .ok_or(BlobError::Denied)?;
        let (b, a, aud) = (s.binding.clone(), s.action, s.audience.clone());
        let policy = self.service_policy.as_ref().ok_or(BlobError::Denied)?;
        if home(&b.recipient)? != policy.local_home() {
            return Err(BlobError::Denied);
        }
        self.check_retrieval_at(
            token,
            &b,
            a,
            &aud,
            Moment {
                wall: now.wall.as_secs(),
                mono: now.after,
            },
        )?;
        self.records
            .slots
            .iter()
            .position(|s| Arc::ptr_eq(&s.token, token))
            .ok_or(BlobError::Denied)
    }
    fn random_secret(
        &self,
        rng: &mut impl FnMut(&mut [u8]) -> Result<(), BlobError>,
    ) -> Result<[u8; 32], BlobError> {
        for _ in 0..4 {
            let mut secret = [0; 32];
            rng(&mut secret)?;
            if self
                .records
                .slots
                .iter()
                .filter_map(|s| s.issuer.as_ref())
                .all(|s| {
                    !memcmp::eq(&s.reference, &secret)
                        && s.token
                            .as_ref()
                            .is_none_or(|t| !memcmp::eq(&t.secret, &secret))
                })
            {
                return Ok(secret);
            }
        }
        Err(BlobError::Busy)
    }
    fn provision_at(
        &mut self,
        token: &Arc<()>,
        now: PreciseMoment,
        rng: &mut impl FnMut(&mut [u8]) -> Result<(), BlobError>,
    ) -> Result<OperationReference, BlobError> {
        let index = self.issuer_slot(token, now)?;
        let slot = &self.records.slots[index];
        // Some(true) still owns the original Connection/Handle during drop cleanup.
        if slot.origin.upgrade().is_some() {
            return Err(BlobError::Denied);
        }
        if let Some(old) = &slot.issuer {
            self.reference_current(index, now.wall.as_secs())?;
            return Ok(OperationReference(old.reference));
        }
        let policy = self.service_policy.as_ref().ok_or(BlobError::Denied)?;
        let issuer = policy.issuer.as_ref().ok_or(BlobError::Denied)?;
        let sender = home(&slot.binding.sender)?;
        let route = issuer
            .routes
            .iter()
            .find(|r| r.sender_home == sender && r.audience == slot.audience)
            .ok_or(BlobError::Denied)?
            .clone();
        let relay = policy
            .registrations()
            .iter()
            .find(|e| {
                e.home == policy.local_home()
                    && e.client_id == route.relay_client_id
                    && policy.service_use(&e.identity()) == Some(ServiceUse::Relay)
            })
            .ok_or(BlobError::Denied)?
            .clone();
        let introspector = policy
            .registrations()
            .iter()
            .find(|e| {
                e.home == route.sender_home
                    && e.client_id == route.introspection_client_id
                    && policy.service_use(&e.identity()) == Some(ServiceUse::Introspection)
            })
            .ok_or(BlobError::Denied)?
            .clone();
        for e in [&relay, &introspector] {
            if !e.enabled
                || now.wall.as_secs() < e.valid_from
                || now.wall.as_secs() >= e.valid_until
            {
                return Err(BlobError::Denied);
            }
        }
        let issuer = issuer.issuer.clone();
        let local_home = policy.local_home().to_owned();
        let reference = self.random_secret(rng)?;
        self.records.slots[index].issuer = Some(ReferenceState {
            reference,
            issuer,
            local_home,
            route,
            relay,
            introspector,
            context: self.service_context.clone(),
            token: None,
        });
        Ok(OperationReference(reference))
    }
    fn reference_current(&self, index: usize, wall: u64) -> Result<(), BlobError> {
        let reference = self.records.slots[index]
            .issuer
            .as_ref()
            .ok_or(BlobError::Denied)?;
        let p = self.service_policy.as_ref().ok_or(BlobError::Denied)?;
        let issuer = p.issuer.as_ref().ok_or(BlobError::Denied)?;
        if !Arc::ptr_eq(&reference.context, &self.service_context)
            || reference.issuer != issuer.issuer
            || reference.local_home != p.local_home()
            || !issuer.routes.contains(&reference.route)
        {
            return Err(BlobError::Denied);
        }
        let registry = self.registry.as_ref().ok_or(BlobError::Denied)?;
        for (entry, purpose) in [
            (&reference.relay, ServiceUse::Relay),
            (&reference.introspector, ServiceUse::Introspection),
        ] {
            if !entry.enabled
                || wall < entry.valid_from
                || wall >= entry.valid_until
                || !registry.homes().contains(&entry.home)
                || !p.registrations().contains(entry)
                || p.service_use(&entry.identity()) != Some(purpose)
            {
                return Err(BlobError::Denied);
            }
        }
        Ok(())
    }
    fn issuer_service(
        &self,
        c: &mut ServiceCredential,
        leaf: &X509,
        id: &str,
        purpose: ServiceUse,
        wall: u64,
    ) -> Result<(), BlobError> {
        if !c.live || wall < c.wall_highwater {
            c.live = false;
            return Err(BlobError::Denied);
        }
        c.wall_highwater = wall;
        if c.identity.client_id != id
            || c.generation != self.generation
            || c.reset_epoch != self.reset_epoch
            || !Arc::ptr_eq(&c.context, &self.service_context)
        {
            return Err(BlobError::Denied);
        }
        let p = self.service_policy.as_ref().ok_or(BlobError::Denied)?;
        let r = self.registry.as_ref().ok_or(BlobError::Denied)?;
        if p.service_use(&c.identity) != Some(purpose) {
            return Err(BlobError::Denied);
        }
        p.current(&c.identity, c.digest, leaf, wall, r)
    }
    fn issue_at(
        &mut self,
        c: &mut ServiceCredential,
        leaf: &X509,
        reference: &[u8; 32],
        id: &str,
        now: PreciseMoment,
        rng: &mut impl FnMut(&mut [u8]) -> Result<(), BlobError>,
    ) -> Result<IssuedToken, BlobError> {
        self.issuer_prune(now)?;
        self.issuer_service(c, leaf, id, ServiceUse::Relay, now.wall.as_secs())?;
        let token = self
            .records
            .slots
            .iter()
            .find(|s| {
                s.issuer
                    .as_ref()
                    .is_some_and(|i| memcmp::eq(&i.reference, reference))
            })
            .ok_or(BlobError::Denied)?
            .token
            .clone();
        let index = self.issuer_slot(&token, now)?;
        self.reference_current(index, now.wall.as_secs())?;
        let slot = &self.records.slots[index];
        let reference = slot.issuer.as_ref().ok_or(BlobError::Denied)?;
        if c.identity != reference.relay.identity()
            || c.digest != reference.relay.certificate_sha256
        {
            return Err(BlobError::Denied);
        }
        if let Some(token) = &reference.token {
            return Ok(IssuedToken {
                token: RetrievalToken(token.secret),
                expires_at: token.expires_at,
                expires_in: remaining(token, now)?,
            });
        }
        let device_until = self
            .registry
            .as_ref()
            .ok_or(BlobError::Denied)?
            .current(&slot.principal, now.wall.as_secs())?
            .valid_until;
        let (exp, deadline) = lifetime(slot, &reference.relay, leaf, device_until, now)?;
        let secret = self.random_secret(rng)?;
        let token = TokenState {
            secret,
            leaf: leaf.clone(),
            issued_at: now.wall.as_secs(),
            expires_at: exp,
            deadline,
        };
        let expires_in = remaining(&token, now)?;
        self.records.slots[index]
            .issuer
            .as_mut()
            .ok_or(BlobError::Denied)?
            .token = Some(token);
        Ok(IssuedToken {
            token: RetrievalToken(secret),
            expires_at: exp,
            expires_in,
        })
    }
    fn introspect_at(
        &mut self,
        c: &mut ServiceCredential,
        leaf: &X509,
        token: &[u8; 32],
        id: &str,
        now: PreciseMoment,
    ) -> Result<RetrievalMetadata, BlobError> {
        self.issuer_prune(now)?;
        self.issuer_service(c, leaf, id, ServiceUse::Introspection, now.wall.as_secs())?;
        let slot = self
            .records
            .slots
            .iter()
            .find(|s| {
                s.issuer
                    .as_ref()
                    .and_then(|r| r.token.as_ref())
                    .is_some_and(|t| memcmp::eq(&t.secret, token))
            })
            .ok_or(BlobError::Denied)?
            .token
            .clone();
        let index = self.issuer_slot(&slot, now)?;
        self.reference_current(index, now.wall.as_secs())?;
        let slot = &self.records.slots[index];
        let reference = slot.issuer.as_ref().ok_or(BlobError::Denied)?;
        let token = reference.token.as_ref().ok_or(BlobError::Denied)?;
        if c.identity != reference.introspector.identity()
            || c.digest != reference.introspector.certificate_sha256
        {
            return Err(BlobError::Denied);
        }
        // Introspection remains active until the exact deadline; issuing/retrying
        // still requires a positive whole-second wire lifetime.
        if now.after >= token.deadline || now.wall >= Duration::from_secs(token.expires_at) {
            return Err(BlobError::Expired);
        }
        self.service_policy
            .as_ref()
            .ok_or(BlobError::Denied)?
            .current(
                &reference.relay.identity(),
                reference.relay.certificate_sha256,
                &token.leaf,
                now.wall.as_secs(),
                self.registry.as_ref().ok_or(BlobError::Denied)?,
            )?;
        Ok(RetrievalMetadata {
            binding: slot.binding.clone(),
            action: slot.action,
            audience: slot.audience.clone(),
            issuer: reference.issuer.clone(),
            issuer_home: reference.local_home.clone(),
            client_id: reference.relay.client_id.clone(),
            relay_certificate_sha256: reference.relay.certificate_sha256,
            issued_at: token.issued_at,
            expires_at: token.expires_at,
            generation: slot.generation,
            reset_epoch: slot.reset_epoch,
        })
    }
}
fn secure_random(bytes: &mut [u8]) -> Result<(), BlobError> {
    rand_bytes(bytes).map_err(|_| BlobError::Denied)
}
impl Control {
    /// Provision only after the exact original device's owning session is gone.
    pub fn provision_retrieval(
        &self,
        record: &RetrievalRecord,
    ) -> Result<OperationReference, Error> {
        let state = self.state()?;
        if !Weak::ptr_eq(&self.state, &record.state) {
            return Err(Error::Transport(tls::Stage::Denied));
        }
        let mut a = state
            .authority
            .lock()
            .map_err(|_| Error::Transport(tls::Stage::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(tls::Stage::Denied));
        }
        let now = a.live_moment().map_err(Error::Store)?;
        a.provision_at(&record.token, now, &mut secure_random)
            .map_err(Error::Store)?;
        // Randomness may wait: the returned reference still needs fresh original life.
        let now = a.live_moment().map_err(Error::Store)?;
        a.provision_at(&record.token, now, &mut secure_random)
            .map_err(Error::Store)
    }
}
impl tls::ServiceSession {
    pub fn issue_retrieval(
        &mut self,
        reference: &OperationReference,
        asserted_client_id: &str,
    ) -> Result<IssuedToken, Error> {
        let state = self.issuer_attempt()?;
        let mut a = state
            .authority
            .lock()
            .map_err(|_| Error::Transport(tls::Stage::Denied))?;
        self.issuer_check(&state)?;
        let now = a.live_moment().map_err(Error::Store)?;
        a.issue_at(
            &mut self.credential,
            &self.leaf,
            reference.expose_bytes(),
            asserted_client_id,
            now,
            &mut secure_random,
        )
        .map_err(|_| Error::Transport(tls::Stage::Denied))?;
        // Recheck after CSPRNG/DER work. Exact retry reuses the token, reducing
        // expires_in by actual remaining precise wall and monotonic lifetime.
        let now = a.live_moment().map_err(Error::Store)?;
        let result = a
            .issue_at(
                &mut self.credential,
                &self.leaf,
                reference.expose_bytes(),
                asserted_client_id,
                now,
                &mut secure_random,
            )
            .map_err(|_| Error::Transport(tls::Stage::Denied));
        self.issuer_check(&state)?;
        result
    }
    /// Unknown, expired and unauthorized tokens have the same metadata-free result.
    pub fn introspect_retrieval(
        &mut self,
        token: &RetrievalToken,
        asserted_client_id: &str,
    ) -> Result<Option<RetrievalMetadata>, Error> {
        let state = self.issuer_attempt()?;
        let mut a = state
            .authority
            .lock()
            .map_err(|_| Error::Transport(tls::Stage::Denied))?;
        self.issuer_check(&state)?;
        let now = a.live_moment().map_err(Error::Store)?;
        let result = a
            .introspect_at(
                &mut self.credential,
                &self.leaf,
                token.expose_bytes(),
                asserted_client_id,
                now,
            )
            .ok();
        self.issuer_check(&state)?;
        Ok(result)
    }
}
#[cfg(feature = "delegation-wire")]
#[path = "issuer_http.rs"]
mod http;
#[cfg(test)]
#[path = "issuer_tests.rs"]
mod tests;
#[cfg(feature = "delegation-wire")]
pub(crate) use http::{HttpInput, HttpReply};

#[cfg(all(test, feature = "delegation-wire"))]
impl Authority {
    pub(crate) fn test_issuer_counts(&self) -> (usize, usize, usize) {
        (
            self.records.slots.len(),
            self.records
                .slots
                .iter()
                .filter(|s| s.issuer.is_some())
                .count(),
            self.records
                .slots
                .iter()
                .filter(|s| s.issuer.as_ref().is_some_and(|i| i.token.is_some()))
                .count(),
        )
    }
}
