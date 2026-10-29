//! Bounded synthetic offline authority for the unchanged ciphertext store.
//!
//! A certificate digest is routing policy, not proof of key possession. Only the
//! trusted crate TLS adapter may pass a verified observation to `authenticate`.
//! The default runtime keeps policy in memory. Explicit durable runtimes restore
//! validated public policy and digest history, while always minting fresh handles.
use faction_files::private_blobs::{
    Access, Action, Binding, Error, Principal, Request, Response, Store,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

pub const MAX_BINDINGS: usize = 32;
pub const MAX_HOMES: usize = 32;
pub const MAX_HANDLES: usize = 8;
pub const HANDLE_LIFETIME_MS: u64 = 10_000;

/// Explicit operator input. Public fields confer no authority before validation.
#[derive(Clone, Debug)]
pub struct CredentialBinding {
    pub certificate_sha256: [u8; 32],
    pub principal: Principal,
    pub valid_from: u64,
    pub valid_until: u64,
    pub approved: bool,
    pub account_enabled: bool,
    pub device_enabled: bool,
    pub generation: u64,
}

/// Validated immutable snapshot; no wire decoder or enrollment endpoint exists.
#[derive(Clone, Debug)]
pub struct Registry {
    generation: u64,
    reset_epoch: u64,
    homes: Vec<String>,
    bindings: Vec<CredentialBinding>,
}
impl Registry {
    pub fn new(
        generation: u64,
        reset_epoch: u64,
        homes: Vec<String>,
        bindings: Vec<CredentialBinding>,
    ) -> Result<Self, Error> {
        if generation == 0 || homes.len() > MAX_HOMES || bindings.len() > MAX_BINDINGS {
            return Err(Error::Invalid);
        }
        for (i, home) in homes.iter().enumerate() {
            Principal {
                account: format!("policy@{home}"),
                device: 1,
                identity: [1; 32],
            }
            .validate()?;
            if homes[..i].contains(home) {
                return Err(Error::Invalid);
            }
        }
        for (i, entry) in bindings.iter().enumerate() {
            entry.principal.validate()?;
            if entry.certificate_sha256 == [0; 32]
                || entry.valid_from >= entry.valid_until
                || entry.generation != generation
                || bindings[..i].iter().any(|old| {
                    old.certificate_sha256 == entry.certificate_sha256
                        || (old.principal.account == entry.principal.account
                            && old.account_enabled != entry.account_enabled)
                        || (old.principal.account == entry.principal.account
                            && old.principal.device == entry.principal.device)
                })
            {
                return Err(Error::Invalid);
            }
        }
        Ok(Self {
            generation,
            reset_epoch,
            homes,
            bindings,
        })
    }
    /// Validated public policy metadata; these values confer no runtime authority.
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn reset_epoch(&self) -> u64 {
        self.reset_epoch
    }
    pub fn homes(&self) -> &[String] {
        &self.homes
    }
    pub fn bindings(&self) -> &[CredentialBinding] {
        &self.bindings
    }
    fn current(&self, principal: &Principal, now: u64) -> Result<&CredentialBinding, Error> {
        let entry = self
            .bindings
            .iter()
            .find(|e| &e.principal == principal)
            .ok_or(Error::Denied)?;
        let (_, home) = principal.account.split_once('@').ok_or(Error::Denied)?;
        if !entry.approved
            || !entry.account_enabled
            || !entry.device_enabled
            || entry.generation != self.generation
            || now < entry.valid_from
            || now >= entry.valid_until
            || !self.homes.iter().any(|h| h == home)
        {
            return Err(Error::Denied);
        }
        Ok(entry)
    }
}
struct LocalServiceAccess<'a> {
    registry: &'a Registry,
    home: &'a str,
}
impl Access for LocalServiceAccess<'_> {
    fn require(
        &self,
        actor: &Principal,
        binding: &Binding,
        action: Action,
        now: u64,
    ) -> Result<(), Error> {
        for principal in [actor, &binding.sender, &binding.recipient] {
            if principal.account.split_once('@').map(|(_, home)| home) != Some(self.home) {
                return Err(Error::Denied);
            }
        }
        self.registry.require(actor, binding, action, now)
    }
}
impl Access for Registry {
    fn require(
        &self,
        actor: &Principal,
        binding: &Binding,
        action: Action,
        now: u64,
    ) -> Result<(), Error> {
        self.current(actor, now)?;
        self.current(&binding.sender, now)?;
        self.current(&binding.recipient, now)?;
        let allowed = match action {
            Action::Begin | Action::PutChunk | Action::Commit | Action::Delete => {
                actor == &binding.sender
            }
            Action::GetChunk => actor == &binding.recipient,
            Action::Status => actor == &binding.sender || actor == &binding.recipient,
        };
        if allowed { Ok(()) } else { Err(Error::Denied) }
    }
}

/// An owning connection context, minted by its Authority and closed on drop.
/// Its fields cannot be reconstructed from client bytes or routing data.
pub(crate) struct Connection {
    authority: Arc<()>,
    closed: Arc<AtomicBool>,
}
impl Drop for Connection {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::Relaxed);
    }
}
/// Opaque, nonserializable authority. It cannot be cloned or constructed by clients.
pub(crate) struct Handle {
    authority: Arc<()>,
    connection: Arc<AtomicBool>,
    token: Arc<()>,
}
struct Slot {
    token: Arc<()>,
    connection: Arc<AtomicBool>,
    certificate_sha256: [u8; 32],
    principal: Principal,
    generation: u64,
    reset_epoch: u64,
    expires_ms: u64,
    last_ms: AtomicU64,
    live: AtomicBool,
}
impl Slot {
    fn current(&self, mono_ms: u64) -> bool {
        if !self.live.load(Ordering::Relaxed) || self.connection.load(Ordering::Relaxed) {
            return false;
        }
        let previous = self.last_ms.fetch_max(mono_ms, Ordering::Relaxed);
        if mono_ms < previous || mono_ms >= self.expires_ms {
            self.live.store(false, Ordering::Relaxed);
            return false;
        }
        true
    }
}

/// Owns one bounded in-memory registry and connection-handle set. The additional
/// conservative lifetime limit is 32 distinct digest tombstones: a removed digest
/// can be explicitly reapproved only for its original exact Principal. Restart
/// invalidates every handle but is not persisted enrollment or anti-rollback.
pub(crate) struct Authority {
    instance: Arc<()>,
    registry: Option<Registry>,
    generation: u64,
    reset_epoch: u64,
    history: Vec<([u8; 32], Principal)>,
    slots: Vec<Slot>,
}
impl Authority {
    pub fn new(registry: Option<Registry>) -> Self {
        let generation = registry.as_ref().map_or(0, |r| r.generation);
        let reset_epoch = registry.as_ref().map_or(0, |r| r.reset_epoch);
        let history = registry.as_ref().map_or_else(Vec::new, |r| {
            r.bindings
                .iter()
                .map(|e| (e.certificate_sha256, e.principal.clone()))
                .collect()
        });
        Self {
            instance: Arc::new(()),
            registry,
            generation,
            reset_epoch,
            history,
            slots: Vec::new(),
        }
    }
    #[cfg(feature = "durable-policy")]
    pub(crate) fn snapshot(&self) -> crate::durable::Snapshot {
        crate::durable::Snapshot {
            generation: self.generation,
            reset_epoch: self.reset_epoch,
            registry: self.registry.clone(),
            history: self.history.clone(),
        }
    }
    #[cfg(feature = "durable-policy")]
    pub(crate) fn restore(snapshot: crate::durable::Snapshot) -> Result<Self, Error> {
        snapshot.validate()?;
        let mut authority = Self::new(None);
        authority.install(snapshot);
        Ok(authority)
    }
    #[cfg(feature = "durable-policy")]
    pub(crate) fn install(&mut self, snapshot: crate::durable::Snapshot) {
        self.generation = snapshot.generation;
        self.reset_epoch = snapshot.reset_epoch;
        self.registry = snapshot.registry;
        self.history = snapshot.history;
        self.slots.clear();
    }
    pub fn open_connection(&self) -> Connection {
        Connection {
            authority: self.instance.clone(),
            closed: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn close_connection(&mut self, connection: &Connection) {
        if Arc::ptr_eq(&self.instance, &connection.authority) {
            connection.closed.store(true, Ordering::Relaxed);
            self.slots
                .retain(|s| !Arc::ptr_eq(&s.connection, &connection.closed));
        }
    }
    /// Install explicit validated policy. Any error leaves the prior policy and
    /// handles unchanged; the caller must mark unavailable policy with `unavailable`.
    pub fn replace_registry(&mut self, registry: Registry) -> Result<(), Error> {
        if registry.generation <= self.generation || registry.reset_epoch < self.reset_epoch {
            return Err(Error::Invalid);
        }
        let mut history = self.history.clone();
        for entry in &registry.bindings {
            if let Some((_, principal)) = history
                .iter()
                .find(|(digest, _)| digest == &entry.certificate_sha256)
            {
                if principal != &entry.principal {
                    return Err(Error::Invalid);
                }
            } else {
                if history.len() == MAX_BINDINGS {
                    return Err(Error::Quota);
                }
                history.push((entry.certificate_sha256, entry.principal.clone()));
            }
        }
        self.generation = registry.generation;
        self.reset_epoch = registry.reset_epoch;
        self.history = history;
        self.registry = Some(registry);
        self.slots.clear();
        Ok(())
    }
    /// A missing/corrupt source is explicitly unavailable, never a fallback.
    pub fn unavailable(&mut self) {
        self.registry = None;
        self.slots.clear();
    }
    /// Password/account reset grants no device binding or key recovery.
    pub fn reset(&mut self, reset_epoch: u64) -> Result<(), Error> {
        if reset_epoch <= self.reset_epoch {
            return Err(Error::Invalid);
        }
        let generation = self.generation.checked_add(1).ok_or(Error::Invalid)?;
        if let Some(registry) = &mut self.registry {
            registry.bindings.clear();
            registry.generation = generation;
            registry.reset_epoch = reset_epoch;
        }
        self.generation = generation;
        self.reset_epoch = reset_epoch;
        self.slots.clear();
        Ok(())
    }
    /// Trusted adapter only: caller must verify the actual TLS connection and
    /// independently enforce X509 validity, handshake and cycle deadlines. Tests
    /// call this with explicitly fake observations, proving no key possession.
    pub(crate) fn authenticate(
        &mut self,
        connection: &Connection,
        verified_certificate_sha256: [u8; 32],
        now: u64,
        mono_ms: u64,
    ) -> Result<Handle, Error> {
        if !Arc::ptr_eq(&self.instance, &connection.authority)
            || connection.closed.load(Ordering::Relaxed)
        {
            return Err(Error::Denied);
        }
        let registry = self.registry.as_ref().ok_or(Error::Denied)?;
        let entry = registry
            .bindings
            .iter()
            .find(|e| e.certificate_sha256 == verified_certificate_sha256)
            .ok_or(Error::Denied)?;
        registry.current(&entry.principal, now)?;
        let expires_ms = mono_ms
            .checked_add(HANDLE_LIFETIME_MS)
            .ok_or(Error::Denied)?;
        self.slots.retain(|s| s.current(mono_ms));
        if self.slots.len() == MAX_HANDLES
            || self
                .slots
                .iter()
                .any(|s| Arc::ptr_eq(&s.connection, &connection.closed))
        {
            return Err(Error::Denied);
        }
        let token = Arc::new(());
        self.slots.push(Slot {
            token: token.clone(),
            connection: connection.closed.clone(),
            certificate_sha256: verified_certificate_sha256,
            principal: entry.principal.clone(),
            generation: self.generation,
            reset_epoch: self.reset_epoch,
            expires_ms,
            last_ms: AtomicU64::new(mono_ms),
            live: AtomicBool::new(true),
        });
        Ok(Handle {
            authority: self.instance.clone(),
            connection: connection.closed.clone(),
            token,
        })
    }
    pub(crate) fn dispatch_routed(
        &self,
        connection: &Connection,
        handle: &Handle,
        route: &str,
        store: &mut Store,
        request: Request,
        times: (u64, u64),
    ) -> Result<Response, Error> {
        let slot = self
            .slots
            .iter()
            .find(|s| Arc::ptr_eq(&s.token, &handle.token))
            .ok_or(Error::Denied)?;
        if slot.principal.account != route {
            return Err(Error::Denied);
        }
        self.dispatch(connection, handle, store, request, times.0, times.1)
    }
    pub(crate) fn dispatch_local(
        &self,
        connection: &Connection,
        handle: &Handle,
        home: &str,
        store: &mut Store,
        request: Request,
        times: (u64, u64),
    ) -> Result<Response, Error> {
        let slot = self
            .slots
            .iter()
            .find(|s| Arc::ptr_eq(&s.token, &handle.token))
            .ok_or(Error::Denied)?;
        if slot
            .principal
            .account
            .split_once('@')
            .map(|(_, domain)| domain)
            != Some(home)
        {
            return Err(Error::Denied);
        }
        self.dispatch_with(connection, handle, store, request, times, Some(home))
    }
    /// Resolve the actor solely from current connection-owned authority. The
    /// request contains data, never an actor selector. Both principals and invited
    /// homes are revalidated through current Access on every unchanged Store call.
    pub(crate) fn dispatch(
        &self,
        connection: &Connection,
        handle: &Handle,
        store: &mut Store,
        request: Request,
        now: u64,
        mono_ms: u64,
    ) -> Result<Response, Error> {
        self.dispatch_with(connection, handle, store, request, (now, mono_ms), None)
    }
    fn dispatch_with(
        &self,
        connection: &Connection,
        handle: &Handle,
        store: &mut Store,
        request: Request,
        times: (u64, u64),
        local_home: Option<&str>,
    ) -> Result<Response, Error> {
        let (now, mono_ms) = times;
        if !Arc::ptr_eq(&self.instance, &connection.authority)
            || !Arc::ptr_eq(&self.instance, &handle.authority)
            || !Arc::ptr_eq(&connection.closed, &handle.connection)
            || connection.closed.load(Ordering::Relaxed)
        {
            return Err(Error::Denied);
        }
        let registry = self.registry.as_ref().ok_or(Error::Denied)?;
        let slot = self
            .slots
            .iter()
            .find(|s| Arc::ptr_eq(&s.token, &handle.token))
            .ok_or(Error::Denied)?;
        if !slot.current(mono_ms)
            || slot.generation != registry.generation
            || slot.reset_epoch != registry.reset_epoch
            || !Arc::ptr_eq(&slot.connection, &connection.closed)
        {
            return Err(Error::Denied);
        }
        let current = registry.current(&slot.principal, now).inspect_err(|_| {
            slot.live.store(false, Ordering::Relaxed);
        })?;
        if current.certificate_sha256 != slot.certificate_sha256 {
            return Err(Error::Denied);
        }
        match local_home {
            Some(home) => store.dispatch(
                &slot.principal,
                &LocalServiceAccess { registry, home },
                request,
                now,
            ),
            None => store.dispatch(&slot.principal, registry, request, now),
        }
    }
}
