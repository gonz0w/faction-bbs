//! One source-bound continuation owner, not a result registry. Correlation only
//! routes a fresh authenticated collect to this exact operation.
use super::*;
use crate::services::{PendingPublication, PendingRetrieval, ServiceIdentity};
use faction_files::private_blobs::Binding;
use std::sync::{
    Condvar, Mutex,
    atomic::{AtomicU64, AtomicUsize},
};
const MAGIC: &[u8; 6] = b"FDEL01";
const SUBMIT: u8 = 0;
const COLLECT: u8 = 1;
const SUBMITTED: u8 = 2;
const BUSY: u8 = 3;
const RESULT: u8 = 4;
const REFUSED: u8 = 5;
const WAIT_COLLECT: u8 = 6;
#[derive(Clone, PartialEq)]
pub(crate) enum Peer {
    Device([u8; 32]),
    Service([u8; 32], ServiceIdentity),
}
pub(crate) struct Source {
    pub(crate) state: Weak<State>,
    pub(crate) peer: Peer,
    pub(crate) purpose: u8,
    pub(crate) binding: Binding,
    pub(crate) request: Request,
    pub(crate) deadline: Instant,
    pub(crate) cancel: Arc<AtomicBool>,
    pub(crate) attempted: usize,
    pub(crate) current: Arc<dyn Fn() -> bool + Send + Sync>,
    // Destroy source-owned record/context before the physical reservation.
    pub(crate) _lease: Arc<()>,
}
struct Shared {
    correlation: [u8; 32],
    attempted: AtomicUsize,
    expires: AtomicU64,
    tight_deadline: Mutex<Instant>,
    #[cfg(all(test, feature = "durable-policy"))]
    deadline_attempt: Mutex<Option<std::sync::mpsc::SyncSender<(bool, usize)>>>,
    completion: Mutex<Option<Result<Response, Error>>>,
    completed: Condvar,
    #[cfg(test)]
    wait_observer: Mutex<Option<std::sync::mpsc::SyncSender<(Instant, Instant)>>>,
    #[cfg(test)]
    finish_observer: Mutex<Option<std::sync::mpsc::SyncSender<()>>>,
    #[cfg(test)]
    trace: std::sync::OnceLock<TraceAttachment>,
    finished: AtomicBool,
    consumed: AtomicBool,
    // The actual result and all delivery bookkeeping die before this lease owner.
    source: Source,
}
impl Shared {
    fn current(&self) -> Result<(), Error> {
        check(self.source.deadline, &self.source.cancel).map_err(Error::Transport)?;
        let lookup = self.tight_deadline.try_lock();
        #[cfg(all(test, feature = "durable-policy"))]
        if let Some(entered) = self.deadline_attempt.lock().unwrap().take() {
            entered
                .send((
                    matches!(&lookup, Err(std::sync::TryLockError::WouldBlock)),
                    self.attempted.load(Ordering::SeqCst),
                ))
                .unwrap();
        }
        let tight = {
            let guard = match lookup {
                Ok(guard) => guard,
                Err(std::sync::TryLockError::WouldBlock) => self
                    .tight_deadline
                    .lock()
                    .map_err(|_| Error::Store(faction_files::private_blobs::Error::Busy))?,
                Err(std::sync::TryLockError::Poisoned(_)) => {
                    return Err(Error::Store(faction_files::private_blobs::Error::Busy));
                }
            };
            *guard
        };
        check(self.source.deadline, &self.source.cancel).map_err(Error::Transport)?;
        check(tight, &self.source.cancel).map_err(Error::Transport)?;
        let expires = self.expires.load(Ordering::SeqCst);
        if expires != 0
            && SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| Error::Store(faction_files::private_blobs::Error::Clock))?
                >= Duration::from_secs(expires)
        {
            return Err(Error::Transport(Stage::Deadline));
        }
        if !(self.source.current)() {
            return Err(Error::Transport(Stage::Denied));
        }
        Ok(())
    }
    fn charge(&self) -> Result<(), Error> {
        self.attempted
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                (n < MAX_OPERATIONS).then_some(n + 1)
            })
            .map_err(|_| Error::Transport(Stage::Limit))?;
        self.current()
    }
    // Only this condition wait is capped; existing current-authority lookups
    // still resample their original absolute clocks after contention.
    fn wait_completion(
        &self,
        session_deadline: Instant,
        current_session: &mut impl FnMut() -> Result<(), Error>,
        #[cfg(test)] trace: Option<&collection_trace::Collect<'_>>,
    ) -> Result<(), Error> {
        current_session()?;
        self.current()?;
        if self.finished.load(Ordering::SeqCst) {
            #[cfg(test)]
            if let Some(trace) = trace {
                trace.already_finished();
            }
            return Ok(());
        }
        let start = Instant::now();
        let mut end = (start + CANCEL_SLICE)
            .min(session_deadline)
            .min(self.source.deadline);
        let tight = *self
            .tight_deadline
            .lock()
            .map_err(|_| Error::Store(faction_files::private_blobs::Error::Busy))?;
        end = end.min(tight);
        // Token lifetime is authoritative only once real introspection publishes
        // it. Unknown expiry supplies no guessed or client-derived authority.
        let expires = self.expires.load(Ordering::SeqCst);
        if expires != 0 {
            let wall = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| Error::Store(faction_files::private_blobs::Error::Clock))?;
            let now = Instant::now();
            let remaining = Duration::from_secs(expires)
                .saturating_sub(wall)
                .min(end.saturating_duration_since(now));
            end = end.min(now + remaining);
        }
        #[cfg(test)]
        if let Some(trace) = trace {
            trace.bounds(start, end);
        }
        #[cfg(test)]
        if let Some(observer) = self.wait_observer.lock().unwrap().take() {
            // This witnesses validated entry, never an OS park or exact wake.
            let _ = observer.try_send((start, end));
        }
        loop {
            current_session()?;
            self.current()?;
            let slot = self
                .completion
                .lock()
                .map_err(|_| Error::Store(faction_files::private_blobs::Error::Busy))?;
            if slot.is_some() {
                drop(slot);
                break;
            }
            let remaining = end.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                drop(slot);
                break;
            }
            let (slot, _) = self
                .completed
                .wait_timeout(slot, remaining)
                .map_err(|_| Error::Store(faction_files::private_blobs::Error::Busy))?;
            drop(slot);
            // No authority/metadata/policy lookup is made under completion.
        }
        #[cfg(test)]
        if let Some(trace) = trace {
            trace.wait_end();
        }
        current_session()?;
        self.current()
    }
    fn finish(&self, result: Result<Response, Error>) -> Result<(), Error> {
        let outcome = result.as_ref().map(|_| ()).map_err(|e| *e);
        #[cfg(test)]
        if let Some(observer) = self.finish_observer.lock().unwrap().take() {
            let _ = observer.try_send(());
        }
        // Both owners hold this mutex only to inspect or move the bounded slot.
        // Contention with a collector must never discard the actual completion.
        let mut slot = self
            .completion
            .lock()
            .map_err(|_| Error::Store(faction_files::private_blobs::Error::Busy))?;
        *slot = Some(result);
        #[cfg(test)]
        if let Some(trace) = self.trace.get() {
            trace.trace.finish_before(trace.role, outcome.is_ok());
        }
        self.finished.store(true, Ordering::SeqCst);
        #[cfg(test)]
        if let Some(trace) = self.trace.get() {
            trace.trace.finish_after(trace.role, outcome.is_ok());
        }
        drop(slot);
        self.completed.notify_all();
        outcome
    }
}
/// Noncloneable private owner for exactly one captured operation. It cannot be
/// constructed from a principal, a correlation, or an arbitrary Store response.
/// ```compile_fail
/// let fabricated = faction_provider::delivery::Delivery {};
/// ```
/// ```compile_fail
/// fn copy(d: faction_provider::delivery::Delivery) { let _ = d.clone(); }
/// ```
/// ```compile_fail
/// fn log(d: faction_provider::delivery::Delivery) { println!("{d:?}"); }
/// ```
pub struct Delivery {
    shared: Arc<Shared>,
}
/// Content-free state sampled without authority lookups or lifetime changes.
#[derive(Clone, Copy, Default)]
pub struct DeliverySnapshot {
    pub attempted: u8,
    pub finished: bool,
    pub consumed: bool,
}
impl Delivery {
    pub fn diagnostic_snapshot(&self) -> DeliverySnapshot {
        DeliverySnapshot {
            attempted: self
                .shared
                .attempted
                .load(Ordering::SeqCst)
                .min(MAX_OPERATIONS) as u8,
            finished: self.shared.finished.load(Ordering::SeqCst),
            consumed: self.shared.consumed.load(Ordering::SeqCst),
        }
    }
    pub(crate) fn new(source: Source) -> Result<Self, Error> {
        let mut correlation = [0; 32];
        openssl::rand::rand_bytes(&mut correlation).map_err(|_| Error::Transport(Stage::Denied))?;
        let attempted = source.attempted;
        let tight_deadline = Mutex::new(source.deadline);
        let result = Self {
            shared: Arc::new(Shared {
                source,
                correlation,
                attempted: AtomicUsize::new(attempted),
                expires: AtomicU64::new(0),
                tight_deadline,
                #[cfg(all(test, feature = "durable-policy"))]
                deadline_attempt: Mutex::new(None),
                completion: Mutex::new(None),
                completed: Condvar::new(),
                #[cfg(test)]
                wait_observer: Mutex::new(None),
                #[cfg(test)]
                finish_observer: Mutex::new(None),
                #[cfg(test)]
                trace: std::sync::OnceLock::new(),
                finished: AtomicBool::new(false),
                consumed: AtomicBool::new(false),
            }),
        };
        result.shared.current()?;
        Ok(result)
    }
    #[cfg(test)]
    pub(crate) fn test_original_attempts(&self) -> usize {
        self.shared.attempted.load(Ordering::SeqCst)
    }
    #[cfg(all(test, feature = "durable-policy"))]
    pub(crate) fn observe_deadline_metadata_attempt(
        &self,
        entered: std::sync::mpsc::SyncSender<(bool, usize)>,
    ) {
        *self.shared.deadline_attempt.lock().unwrap() = Some(entered);
    }
    #[cfg(test)]
    pub(crate) fn hold_deadline_metadata(&self) -> std::sync::MutexGuard<'_, Instant> {
        self.shared.tight_deadline.lock().unwrap()
    }
    pub fn is_finished(&self) -> bool {
        self.shared.finished.load(Ordering::SeqCst)
    }
    pub fn is_expired(&self) -> bool {
        self.shared.current().is_err()
            || self.shared.attempted.load(Ordering::SeqCst) >= MAX_OPERATIONS
    }
    pub fn is_consumed(&self) -> bool {
        self.shared.consumed.load(Ordering::SeqCst)
    }
    fn receipt(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.push(SUBMITTED);
        out.extend_from_slice(&self.shared.correlation);
        out
    }
    fn begin_collect(&self, peer: &Peer, state: &Weak<State>) -> Result<(), Error> {
        if peer != &self.shared.source.peer || !Weak::ptr_eq(state, &self.shared.source.state) {
            return Err(Error::Transport(Stage::Denied));
        }
        self.shared.charge()
    }
    fn collect(
        &self,
        bytes: &[u8],
        peer: Peer,
        state: &Weak<State>,
        session_deadline: Option<Instant>,
        mut current_session: impl FnMut() -> Result<(), Error>,
    ) -> Result<Option<Vec<u8>>, Error> {
        // No authority is inferred from routing data or decoded principals.
        if peer != self.shared.source.peer || !Weak::ptr_eq(state, &self.shared.source.state) {
            return Err(Error::Transport(Stage::Denied));
        }
        let message = Message::decode(bytes)?;
        if ![COLLECT, WAIT_COLLECT].contains(&message.kind)
            || message.correlation != self.shared.correlation
        {
            return Err(Error::Transport(Stage::Denied));
        }
        if message.purpose != self.shared.source.purpose()
            || message.binding != self.shared.source.binding
            || message.request != self.shared.source.request
            || self.shared.consumed.load(Ordering::SeqCst)
        {
            return Err(Error::Transport(Stage::Denied));
        }
        if message.kind == WAIT_COLLECT
            && !matches!(
                (&peer, message.purpose),
                (Peer::Service(..), 2) | (Peer::Device(_), 3)
            )
        {
            return Err(Error::Transport(Stage::Denied));
        }
        #[cfg(test)]
        let trace = self.shared.trace.get().and_then(|attachment| {
            attachment.trace.validated(
                attachment.role,
                if message.kind == WAIT_COLLECT {
                    collection_trace::WireKind::Wait
                } else {
                    collection_trace::WireKind::Collect
                },
            )
        });
        if message.kind == WAIT_COLLECT {
            self.shared.wait_completion(
                session_deadline.ok_or(Error::Transport(Stage::Denied))?,
                &mut current_session,
                #[cfg(test)]
                trace.as_ref(),
            )?;
        }
        if !self.shared.finished.load(Ordering::SeqCst) {
            #[cfg(test)]
            if let Some(trace) = &trace {
                trace.outcome(collection_trace::Outcome::Busy);
            }
            return Ok(None);
        }
        let mut slot = self
            .shared
            .completion
            .lock()
            .map_err(|_| Error::Store(faction_files::private_blobs::Error::Busy))?;
        let Some(result) = slot.take() else {
            return Err(Error::Transport(Stage::Denied));
        };
        self.shared.consumed.store(true, Ordering::SeqCst);
        drop(slot);
        self.shared.current()?;
        let mut response = MAGIC.to_vec();
        match result {
            Ok(result) => {
                response.push(RESULT);
                response.extend(encode_response(&result).map_err(Error::Store)?);
            }
            Err(_) => response.push(REFUSED),
        }
        #[cfg(test)]
        if let Some(trace) = &trace {
            trace.outcome(if response[6] == RESULT {
                collection_trace::Outcome::Result
            } else {
                collection_trace::Outcome::Refused
            });
        }
        Ok(Some(response))
    }
}
impl Source {
    fn purpose(&self) -> u8 {
        self.purpose
    }
}
/// Consumes only a publication prepared from an actual authenticated sender.
pub struct PublicationWork {
    pending: PendingPublication,
    shared: Arc<Shared>,
}
impl PublicationWork {
    pub fn execute(self, endpoint: &HttpEndpoint, store: &Mutex<Store>) -> Result<(), Error> {
        let result = self
            .shared
            .charge()
            .and_then(|_| self.pending.execute(endpoint, store));
        self.shared.finish(result)
    }
}
/// Consumes only a resource operation prepared from an actual registered relay.
pub struct RetrievalWork {
    pending: PendingRetrieval,
    shared: Arc<Shared>,
}
impl RetrievalWork {
    pub fn execute(self, endpoint: &HttpEndpoint, store: &Mutex<Store>) -> Result<(), Error> {
        #[cfg(test)]
        let _setup_scope = self.shared.trace.get().map(|attached| {
            attached
                .trace
                .setup
                .scope(crate::setup_trace::Role::Resource)
        });
        let result = self
            .shared
            .charge()
            .and_then(|_| self.pending.execute_delivery_response(endpoint, store))
            .map(|(response, expires)| {
                self.shared.expires.store(expires, Ordering::SeqCst);
                response
            });
        self.shared.finish(result)
    }
}
struct Message {
    kind: u8,
    purpose: u8,
    correlation: [u8; 32],
    binding: Binding,
    request: Request,
    token: Option<[u8; 32]>,
}
impl Message {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        let binding =
            encode_request(&Request::Begin(self.binding.clone())).map_err(Error::Store)?;
        let request = encode_request(&self.request).map_err(Error::Store)?;
        let mut out = MAGIC.to_vec();
        out.extend([self.kind, self.purpose]);
        out.extend(self.correlation);
        for bytes in [&binding, &request] {
            out.extend((bytes.len() as u16).to_be_bytes());
            out.extend(bytes);
        }
        if let Some(token) = self.token {
            out.extend(token);
        }
        if out.len() > MAX_WIRE {
            return Err(Error::Transport(Stage::Frame));
        }
        Ok(out)
    }
    fn decode(mut bytes: &[u8]) -> Result<Self, Error> {
        let frame = || Error::Transport(Stage::Frame);
        if bytes.len() > MAX_WIRE || bytes.len() < 44 || &bytes[..6] != MAGIC {
            return Err(frame());
        }
        let kind = bytes[6];
        let purpose = bytes[7];
        if ![SUBMIT, COLLECT, WAIT_COLLECT].contains(&kind)
            || ![1, 2, 3].contains(&purpose)
            || (kind == WAIT_COLLECT && ![2, 3].contains(&purpose))
        {
            return Err(frame());
        }
        let correlation = bytes[8..40].try_into().map_err(|_| frame())?;
        bytes = &bytes[40..];
        fn field<'a>(bytes: &mut &'a [u8]) -> Result<&'a [u8], Error> {
            if bytes.len() < 2 {
                return Err(Error::Transport(Stage::Frame));
            }
            let length = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
            if bytes.len() < 2 + length {
                return Err(Error::Transport(Stage::Frame));
            }
            let result = &bytes[2..2 + length];
            *bytes = &bytes[2 + length..];
            Ok(result)
        }
        let Request::Begin(binding) = decode_request(field(&mut bytes)?).map_err(Error::Store)?
        else {
            return Err(frame());
        };
        let request = decode_request(field(&mut bytes)?).map_err(Error::Store)?;
        let token = if purpose == 2 && kind == SUBMIT {
            if bytes.len() != 32 {
                return Err(frame());
            }
            let value = Some(bytes.try_into().map_err(|_| frame())?);
            bytes = &[];
            value
        } else {
            None
        };
        if !bytes.is_empty() || (kind == SUBMIT && correlation != [0; 32]) {
            return Err(frame());
        }
        Ok(Self {
            kind,
            purpose,
            correlation,
            binding,
            request,
            token,
        })
    }
}
impl Session {
    /// Read one exact versioned sender submission, acknowledge routing only, and
    /// fully destroy the incoming TLS/Connection/Handle before returning work.
    pub fn submit_publication(mut self) -> Result<(Delivery, PublicationWork), Error> {
        let state = self.attempt()?;
        let bytes = read_frame(&mut self.stream)
            .map_err(Error::Transport)?
            .ok_or(Error::Transport(Stage::Frame))?;
        let message = Message::decode(&bytes)?;
        if message.kind != SUBMIT || message.purpose != 1 {
            return Err(Error::Transport(Stage::Denied));
        }
        let pending = self.capture_publication(&state, message.binding, message.request)?;
        let delivery = Delivery::new(pending.delivery_source())?;
        self.write_delivery(Some(delivery.receipt()), delivery.shared.clone())?;
        delivery.shared.current()?;
        let shared = delivery.shared.clone();
        drop(self);
        Ok((delivery, PublicationWork { pending, shared }))
    }
    /// Fresh authentication precedes exact current source validation. A pending
    /// callback returns Busy immediately and releases this incoming session.
    pub fn collect_delivery(mut self, delivery: &mut Delivery) -> Result<bool, Error> {
        self.attempt()?;
        let guard = delivery.shared.clone();
        let result = (|| {
            let peer = Peer::Device(sha256(
                &self
                    .leaf
                    .to_der()
                    .map_err(|_| Error::Transport(Stage::Denied))?,
            ));
            delivery.begin_collect(&peer, &self.state)?;
            self.deadline = self.deadline.min(guard.source.deadline);
            self.stream.get_mut().deadline = self.deadline;
            let probe = guard.clone();
            self.stream.get_mut().policy_probe = Some(Arc::new(move || probe.current().is_ok()));
            let bytes = read_frame(&mut self.stream)
                .map_err(|s| Error::Transport(classify(self.deadline, &guard.source.cancel, s)))?
                .ok_or(Error::Transport(Stage::Frame))?;
            let reply = delivery.collect(&bytes, peer, &self.state, None, || Ok(()))?;
            self.write_delivery(reply, guard.clone())
        })();
        drop(self);
        drop(guard);
        result
    }
    fn write_delivery(
        &mut self,
        reply: Option<Vec<u8>>,
        shared: Arc<Shared>,
    ) -> Result<bool, Error> {
        let ready = reply.is_some();
        let reply = reply.unwrap_or_else(|| {
            let mut b = MAGIC.to_vec();
            b.push(BUSY);
            b
        });
        self.deadline = self.deadline.min(shared.source.deadline);
        self.stream.get_mut().deadline = self.deadline;
        let leaf = self.leaf.clone();
        self.stream.get_mut().policy_probe = Some(Arc::new(move || {
            shared.current().is_ok() && leaf_current(&leaf, wall_now())
        }));
        write_frame(&mut self.stream, &reply).map_err(Error::Transport)?;
        check(self.deadline, &self.cancel).map_err(Error::Transport)?;
        Ok(ready)
    }
}
impl ServiceSession {
    pub fn submit_retrieval(mut self) -> Result<(Delivery, RetrievalWork), Error> {
        let state = self.issuer_attempt()?;
        let bytes = read_frame(&mut self.stream)
            .map_err(Error::Transport)?
            .ok_or(Error::Transport(Stage::Frame))?;
        let message = Message::decode(&bytes)?;
        if message.kind != SUBMIT || message.purpose != 2 {
            return Err(Error::Transport(Stage::Denied));
        }
        let pending = self.capture_retrieval_charged(
            &state,
            message.binding,
            message.request,
            faction_delegation_codec::UnverifiedToken::from_bytes(
                message.token.ok_or(Error::Transport(Stage::Frame))?,
            ),
        )?;
        let delivery = Delivery::new(pending.delivery_source())?;
        let probe = delivery.shared.clone();
        self.stream.get_mut().policy_probe = Some(Arc::new(move || probe.current().is_ok()));
        write_frame(&mut self.stream, &delivery.receipt()).map_err(Error::Transport)?;
        delivery.shared.current()?;
        let shared = delivery.shared.clone();
        drop(self);
        Ok((delivery, RetrievalWork { pending, shared }))
    }
    pub fn collect_delivery(mut self, delivery: &mut Delivery) -> Result<bool, Error> {
        self.issuer_attempt()?;
        let guard = delivery.shared.clone();
        let result = (|| {
            let peer = Peer::Service(self.credential.digest, self.credential.identity.clone());
            delivery.begin_collect(&peer, &self.state)?;
            self.deadline = self.deadline.min(guard.source.deadline);
            self.stream.get_mut().deadline = self.deadline;
            let probe = guard.clone();
            self.stream.get_mut().policy_probe = Some(Arc::new(move || probe.current().is_ok()));
            let bytes = read_frame(&mut self.stream)
                .map_err(|s| Error::Transport(classify(self.deadline, &guard.source.cancel, s)))?
                .ok_or(Error::Transport(Stage::Frame))?;
            let peer = Peer::Service(self.credential.digest, self.credential.identity.clone());
            let state = self
                .state
                .upgrade()
                .ok_or(Error::Transport(Stage::Denied))?;
            let source_state = self.state.clone();
            let deadline = self.deadline;
            let reply = delivery.collect(&bytes, peer, &source_state, Some(deadline), || {
                self.issuer_check(&state)
            })?;
            let ready = reply.is_some();
            let reply = reply.unwrap_or_else(|| {
                let mut b = MAGIC.to_vec();
                b.push(BUSY);
                b
            });
            self.deadline = self.deadline.min(guard.source.deadline);
            self.stream.get_mut().deadline = self.deadline;
            let probe = guard.clone();
            let leaf = self.leaf.clone();
            self.stream.get_mut().policy_probe = Some(Arc::new(move || {
                probe.current().is_ok() && leaf_current(&leaf, wall_now())
            }));
            write_frame(&mut self.stream, &reply).map_err(Error::Transport)?;
            let state = guard
                .source
                .state
                .upgrade()
                .ok_or(Error::Transport(Stage::Denied))?;
            self.issuer_check(&state)?;
            Ok(ready)
        })();
        drop(self);
        drop(guard);
        result
    }
}
/// Fixed trusted endpoint; it is never obtained from a blob locator or a request.
#[derive(Clone)]
pub struct CarrierEndpoint {
    address: SocketAddr,
    server_name: String,
    leaf_pin: [u8; 32],
    credentials: ClientCredentials,
}
impl CarrierEndpoint {
    pub fn new(
        address: SocketAddr,
        server_name: String,
        leaf_pin: [u8; 32],
        credentials: ClientCredentials,
    ) -> Result<Self, Stage> {
        if !address.ip().is_loopback() || address.port() == 0 {
            return Err(Stage::Configuration);
        }
        ClientAdmission::new(
            &server_name,
            leaf_pin,
            credentials.clone(),
            Arc::new(AtomicBool::new(false)),
        )?;
        Ok(Self {
            address,
            server_name,
            leaf_pin,
            credentials,
        })
    }
    pub fn operation(&self, cancel: Arc<AtomicBool>) -> Operation {
        Operation {
            endpoint: self.clone(),
            deadline: Instant::now() + CYCLE_LIMIT,
            cancel,
            attempted: 0,
            current_probe: None,
            next_collect: None,
            collect_delay: Duration::ZERO,
            ticket: None,
            diagnostic: None,
            diagnostic_start: None,
            #[cfg(test)]
            wait_entered: None,
        }
    }
    /// Start one logical request inside an existing foreground deadline.
    /// Construction performs no I/O and cannot renew the parent's window.
    pub fn operation_until(
        &self,
        parent_deadline: Instant,
        cancel: Arc<AtomicBool>,
    ) -> Result<Operation, Stage> {
        let mut operation = self.operation(cancel);
        operation.deadline = operation.deadline.min(parent_deadline);
        check(operation.deadline, &operation.cancel)?;
        Ok(operation)
    }
    fn connect(
        &self,
        deadline: Instant,
        cancel: Arc<AtomicBool>,
        current_probe: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    ) -> Result<Client, Stage> {
        check(deadline, &cancel)?;
        let mut admission = ClientAdmission::new(
            &self.server_name,
            self.leaf_pin,
            self.credentials.clone(),
            cancel,
        )?;
        admission.operation_deadline = Some(deadline);
        let client_leaf = self.credentials.certificate.clone();
        let probe = Arc::new(move || {
            current_probe.as_ref().is_none_or(|p| p()) && leaf_current(&client_leaf, wall_now())
        });
        admission.current_probe = Some(probe.clone());
        let handshake = (admission.start + HANDSHAKE_LIMIT).min(deadline);
        #[cfg(test)]
        {
            admission.setup_trace =
                crate::setup_trace::Connection::begin(crate::setup_trace::Kind::Binary);
        }
        #[cfg(test)]
        let setup_tcp = admission
            .setup_trace
            .as_ref()
            .map(|trace| trace.interval(crate::setup_trace::Phase::Tcp, 0));
        let socket =
            super::tcp_connect::connect(&self.address, handshake, &admission.cancel, || probe())?;
        #[cfg(test)]
        crate::setup_trace::Interval::finish_optional(setup_tcp);
        socket.set_nodelay(true).map_err(|_| Stage::Configuration)?;
        admission.connect(socket)
    }
}
/// Fixed local diagnostic categories, never protocol data or remote authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DiagnosticPhase {
    #[default]
    Unobserved,
    SubmitConnect,
    SubmitExchange,
    CollectWait,
    CollectConnect,
    CollectExchange,
    Idle,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DiagnosticTimings {
    pub submit_connect_us: u64,
    pub submit_exchange_us: u64,
    pub collect_connect_us: u64,
    pub collect_exchange_us: u64,
    pub planned_wait_us: u64,
    pub actual_wait_us: u64,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ExchangeOrigin {
    #[default]
    Unobserved,
    Write,
    ReadError,
    ReadEof,
    Decode,
    Refused,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OperationSnapshot {
    pub phase: DiagnosticPhase,
    pub attempted: u8,
    pub busy: u8,
    pub connections: u8,
    pub exchange_origin: ExchangeOrigin,
    pub timings: DiagnosticTimings,
    pub phase_us: u64,
}
fn diagnostic_micros(duration: Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
}
impl OperationSnapshot {
    fn add_phase(&mut self, micros: u64) {
        let value = match self.phase {
            DiagnosticPhase::SubmitConnect => &mut self.timings.submit_connect_us,
            DiagnosticPhase::SubmitExchange => &mut self.timings.submit_exchange_us,
            DiagnosticPhase::CollectConnect => &mut self.timings.collect_connect_us,
            DiagnosticPhase::CollectExchange => &mut self.timings.collect_exchange_us,
            DiagnosticPhase::CollectWait => &mut self.timings.actual_wait_us,
            DiagnosticPhase::Unobserved | DiagnosticPhase::Idle => return,
        };
        *value = value.saturating_add(micros);
    }
}
impl DiagnosticTimings {
    pub fn saturating_add(self, other: Self) -> Self {
        Self {
            submit_connect_us: self
                .submit_connect_us
                .saturating_add(other.submit_connect_us),
            submit_exchange_us: self
                .submit_exchange_us
                .saturating_add(other.submit_exchange_us),
            collect_connect_us: self
                .collect_connect_us
                .saturating_add(other.collect_connect_us),
            collect_exchange_us: self
                .collect_exchange_us
                .saturating_add(other.collect_exchange_us),
            planned_wait_us: self.planned_wait_us.saturating_add(other.planned_wait_us),
            actual_wait_us: self.actual_wait_us.saturating_add(other.actual_wait_us),
        }
    }
}
/// One original clock and bounded attempt window across all fresh connections.
/// No automatic polling is performed by this interface.
pub struct Operation {
    endpoint: CarrierEndpoint,
    deadline: Instant,
    cancel: Arc<AtomicBool>,
    attempted: usize,
    current_probe: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    next_collect: Option<Instant>,
    collect_delay: Duration,
    ticket: Option<Message>,
    diagnostic: Option<OperationSnapshot>,
    diagnostic_start: Option<Instant>,
    #[cfg(test)]
    wait_entered: Option<std::sync::mpsc::SyncSender<()>>,
}
pub enum Collection {
    Busy,
    Complete(Response),
}
impl Operation {
    fn exchange_origin(&mut self, origin: ExchangeOrigin) {
        if let Some(snapshot) = &mut self.diagnostic {
            snapshot.exchange_origin = origin;
        }
    }
    fn exchange(
        &mut self,
        stream: &mut SslStream<DeadlineIo>,
        bytes: &[u8],
    ) -> Result<Vec<u8>, Stage> {
        write_frame(stream, bytes).inspect_err(|_| {
            self.exchange_origin(ExchangeOrigin::Write);
        })?;
        match read_frame(stream) {
            Ok(Some(reply)) => Ok(reply),
            Ok(None) => {
                self.exchange_origin(ExchangeOrigin::ReadEof);
                Err(Stage::Frame)
            }
            Err(error) => {
                self.exchange_origin(ExchangeOrigin::ReadError);
                Err(error)
            }
        }
    }
    /// Local timings are cumulative client observations. Exchange includes all
    /// unobserved server callback, Store and scheduling time; it is not CPU time.
    pub fn diagnostic_snapshot(&self) -> Option<OperationSnapshot> {
        let mut snapshot = self.diagnostic?;
        snapshot.attempted = u8::try_from(self.attempted).unwrap_or(u8::MAX);
        if let Some(start) = self.diagnostic_start {
            snapshot.phase_us = diagnostic_micros(start.elapsed());
            snapshot.add_phase(snapshot.phase_us);
        }
        Some(snapshot)
    }
    fn diagnostic_phase(&mut self, phase: DiagnosticPhase) {
        let mut snapshot = self.diagnostic_snapshot().unwrap_or_default();
        snapshot.phase = phase;
        snapshot.phase_us = 0;
        self.diagnostic = Some(snapshot);
        self.diagnostic_start = (phase != DiagnosticPhase::Idle).then(Instant::now);
    }
    fn diagnostic_connect(&mut self, phase: DiagnosticPhase) {
        self.exchange_origin(ExchangeOrigin::Unobserved);
        self.diagnostic_phase(phase);
        let snapshot = self.diagnostic.as_mut().unwrap();
        snapshot.connections = snapshot.connections.saturating_add(1);
    }
    fn current(&self) -> Result<(), Stage> {
        check(self.deadline, &self.cancel)?;
        if !leaf_current(&self.endpoint.credentials.certificate, wall_now())
            || self.current_probe.as_ref().is_some_and(|probe| !probe())
        {
            return Err(Stage::Denied);
        }
        check(self.deadline, &self.cancel)
    }
    fn charge(&mut self) -> Result<(), Stage> {
        self.current()?;
        if self.attempted >= MAX_OPERATIONS {
            return Err(Stage::Limit);
        }
        self.attempted += 1;
        Ok(())
    }
    fn schedule_collect(&mut self, roundtrip: Duration, busy: bool) {
        let depth = if self
            .ticket
            .as_ref()
            .is_some_and(|ticket| ticket.purpose == 3)
        {
            3
        } else {
            1
        };
        let measured = roundtrip.saturating_mul(depth);
        self.collect_delay = if busy {
            self.collect_delay.saturating_mul(2).max(measured)
        } else {
            measured
        };
        self.collect_delay = self
            .collect_delay
            .min(self.deadline.saturating_duration_since(Instant::now()));
        self.next_collect = Some((Instant::now() + self.collect_delay).min(self.deadline));
    }
    fn wait_for_collect(&mut self) -> Result<(), Stage> {
        if self.attempted >= MAX_OPERATIONS {
            return Err(Stage::Limit);
        }
        self.diagnostic_phase(DiagnosticPhase::CollectWait);
        let planned = self.next_collect.map_or(Duration::ZERO, |when| {
            when.saturating_duration_since(Instant::now())
        });
        if let Some(snapshot) = &mut self.diagnostic {
            snapshot.timings.planned_wait_us = snapshot
                .timings
                .planned_wait_us
                .saturating_add(diagnostic_micros(planned));
        }
        loop {
            self.current()?;
            let remaining = self.next_collect.map_or(Duration::ZERO, |when| {
                when.saturating_duration_since(Instant::now())
            });
            if remaining.is_zero() {
                self.diagnostic_phase(DiagnosticPhase::Idle);
                return Ok(());
            }
            #[cfg(test)]
            if let Some(entered) = self.wait_entered.take() {
                entered.send(()).unwrap();
            }
            std::thread::sleep(
                remaining
                    .min(CANCEL_SLICE)
                    .min(self.deadline.saturating_duration_since(Instant::now())),
            );
        }
    }
    /// Wait before one fresh collect using only this operation's measured TLS
    /// round trips and Busy responses. The original deadline and attempt budget
    /// remain authoritative; this method does not retry or guarantee completion.
    /// It owns no socket or Session while waiting. `collect` remains immediate.
    pub fn collect_paced(&mut self) -> Result<Collection, Stage> {
        #[cfg(test)]
        let setup_wait = crate::setup_trace::Connection::begin(crate::setup_trace::Kind::Paced);
        #[cfg(test)]
        let setup_interval = setup_wait
            .as_ref()
            .map(|trace| trace.interval(crate::setup_trace::Phase::PacedWait, 0));
        self.wait_for_collect()?;
        #[cfg(test)]
        if let Some(interval) = setup_interval {
            interval.finish();
        }
        self.collect()
    }
    pub fn submit_publication(&mut self, binding: Binding, request: Request) -> Result<(), Stage> {
        self.submit(1, binding, request, None)
    }
    pub fn submit_own_home(&mut self, binding: Binding, request: Request) -> Result<(), Stage> {
        self.submit(3, binding, request, None)
    }
    pub fn submit_retrieval(
        &mut self,
        binding: Binding,
        request: Request,
        token: faction_delegation_codec::UnverifiedToken,
    ) -> Result<(), Stage> {
        self.submit(
            2,
            binding,
            request,
            Some(token.decode_bytes().map_err(|_| Stage::Frame)?),
        )
    }
    fn submit(
        &mut self,
        purpose: u8,
        binding: Binding,
        request: Request,
        token: Option<[u8; 32]>,
    ) -> Result<(), Stage> {
        self.charge()?;
        if self.ticket.is_some() {
            return Err(Stage::Denied);
        }
        let mut message = Message {
            kind: SUBMIT,
            purpose,
            correlation: [0; 32],
            binding,
            request,
            token,
        };
        let bytes = message.encode().map_err(|_| Stage::Frame)?;
        let roundtrip = Instant::now();
        self.diagnostic_connect(DiagnosticPhase::SubmitConnect);
        let mut client = self.endpoint.connect(
            self.deadline,
            self.cancel.clone(),
            self.current_probe.clone(),
        )?;
        self.diagnostic_phase(DiagnosticPhase::SubmitExchange);
        let reply = self.exchange(&mut client.stream, &bytes)?;
        self.exchange_origin(ExchangeOrigin::Decode);
        if reply.len() != 39 || &reply[..6] != MAGIC || reply[6] != SUBMITTED {
            return Err(Stage::Frame);
        }
        message.correlation = reply[7..].try_into().map_err(|_| Stage::Frame)?;
        message.kind = COLLECT;
        message.token = None;
        self.exchange_origin(ExchangeOrigin::Unobserved);
        check(self.deadline, &self.cancel)?;
        drop(client);
        self.ticket = Some(message);
        self.schedule_collect(roundtrip.elapsed(), false);
        self.exchange_origin(ExchangeOrigin::Unobserved);
        self.diagnostic_phase(DiagnosticPhase::Idle);
        Ok(())
    }
    /// Explicitly wait at a service-purpose2 resource for at most one server
    /// cancellation slice. A refusal from an older server fails closed; this
    /// call never falls back, resubmits, retries or renews the original window.
    pub fn collect_waiting(&mut self) -> Result<Collection, Stage> {
        if self
            .ticket
            .as_ref()
            .is_none_or(|ticket| ticket.purpose != 2)
        {
            return Err(Stage::Denied);
        }
        self.collect_kind(WAIT_COLLECT)
    }
    /// Opt in to bounded waiting only for this device's own-home retrieval.
    /// The original operation window and shared attempt budget remain binding.
    pub fn collect_own_home_waiting(&mut self) -> Result<Collection, Stage> {
        if self
            .ticket
            .as_ref()
            .is_none_or(|ticket| ticket.purpose != 3)
        {
            return Err(Stage::Denied);
        }
        self.collect_kind(WAIT_COLLECT)
    }
    /// Preserve local round-trip/Busy pacing before one own-home waiting collect.
    pub fn collect_own_home_paced(&mut self) -> Result<Collection, Stage> {
        if self
            .ticket
            .as_ref()
            .is_none_or(|ticket| ticket.purpose != 3)
        {
            return Err(Stage::Denied);
        }
        self.wait_for_collect()?;
        self.collect_own_home_waiting()
    }
    pub fn collect(&mut self) -> Result<Collection, Stage> {
        self.collect_kind(COLLECT)
    }
    fn collect_kind(&mut self, kind: u8) -> Result<Collection, Stage> {
        self.charge()?;
        let mut bytes = self
            .ticket
            .as_ref()
            .ok_or(Stage::Denied)?
            .encode()
            .map_err(|_| Stage::Frame)?;
        bytes[6] = kind;
        let roundtrip = Instant::now();
        self.diagnostic_connect(DiagnosticPhase::CollectConnect);
        let mut client = self.endpoint.connect(
            self.deadline,
            self.cancel.clone(),
            self.current_probe.clone(),
        )?;
        self.diagnostic_phase(DiagnosticPhase::CollectExchange);
        let reply = self.exchange(&mut client.stream, &bytes)?;
        check(self.deadline, &self.cancel)?;
        drop(client);
        self.exchange_origin(ExchangeOrigin::Decode);
        if reply.len() < 7 || &reply[..6] != MAGIC {
            return Err(Stage::Frame);
        }
        match reply[6] {
            BUSY if reply.len() == 7 => {
                self.schedule_collect(roundtrip.elapsed(), true);
                if let Some(snapshot) = &mut self.diagnostic {
                    snapshot.busy = snapshot.busy.saturating_add(1);
                }
                self.exchange_origin(ExchangeOrigin::Unobserved);
                self.diagnostic_phase(DiagnosticPhase::Idle);
                Ok(Collection::Busy)
            }
            RESULT => {
                let response = decode_response(&reply[7..]).map_err(|_| Stage::Frame)?;
                self.ticket = None;
                self.next_collect = None;
                self.exchange_origin(ExchangeOrigin::Unobserved);
                self.diagnostic_phase(DiagnosticPhase::Idle);
                Ok(Collection::Complete(response))
            }
            REFUSED if reply.len() == 7 => {
                self.ticket = None;
                self.next_collect = None;
                self.exchange_origin(ExchangeOrigin::Refused);
                Err(Stage::Denied)
            }
            _ => Err(Stage::Frame),
        }
    }
}
/// Complete trusted endpoints for one explicitly invited peer, never wire input.
pub struct RelayPeer {
    pub home: String,
    pub publication: HttpEndpoint,
    pub introspection: HttpEndpoint,
    pub resource: CarrierEndpoint,
}
pub struct RelayRoutes {
    state: Weak<State>,
    policy: crate::services::ServicePolicy,
    local: CarrierEndpoint,
    peers: Vec<RelayPeer>,
    #[cfg(all(test, feature = "durable-policy"))]
    authority_attempt: Option<std::sync::mpsc::SyncSender<bool>>,
}
impl RelayRoutes {
    pub fn new(
        runtime: &Runtime,
        local: CarrierEndpoint,
        peers: Vec<RelayPeer>,
    ) -> Result<Self, Error> {
        let authority = runtime
            .state
            .authority
            .lock()
            .map_err(|_| Error::Transport(Stage::Denied))?;
        let policy = authority.relay_policy().map_err(Error::Store)?;
        if peers.is_empty()
            || peers.len() != policy.resource_associations().len()
            || peers.len() != policy.publication_associations().len()
        {
            return Err(Error::Transport(Stage::Configuration));
        }
        for (i, peer) in peers.iter().enumerate() {
            if peers[..i].iter().any(|p| p.home == peer.home) {
                return Err(Error::Transport(Stage::Configuration));
            }
            let resource = policy
                .resource_associations()
                .iter()
                .find(|a| a.recipient_home == peer.home)
                .ok_or(Error::Transport(Stage::Configuration))?;
            let publication = policy
                .publication_associations()
                .iter()
                .find(|a| a.recipient_home == peer.home)
                .ok_or(Error::Transport(Stage::Configuration))?;
            let route = policy
                .issuer
                .as_ref()
                .and_then(|p| p.routes.iter().find(|r| r.sender_home == peer.home))
                .ok_or(Error::Transport(Stage::Configuration))?;
            let relay = policy
                .resource_registration(
                    policy.local_home(),
                    &route.relay_client_id,
                    crate::services::ServiceUse::Relay,
                )
                .map_err(Error::Store)?;
            let outgoing = policy
                .resource_registration(
                    policy.local_home(),
                    &resource.outbound_client_id,
                    crate::services::ServiceUse::OutboundIntrospection,
                )
                .map_err(Error::Store)?;
            let observer = policy
                .resource_registration(
                    policy.local_home(),
                    &publication.outbound_client_id,
                    crate::services::ServiceUse::OutboundObservation,
                )
                .map_err(Error::Store)?;
            let endpoint = policy
                .resource_registration(
                    &peer.home,
                    &resource.endpoint_id,
                    crate::services::ServiceUse::IssuerEndpoint,
                )
                .map_err(Error::Store)?;
            if !peer
                .publication
                .matches_publication(publication, observer, endpoint)
                || !peer
                    .introspection
                    .matches_resource(resource, outgoing, endpoint)
                || peer.resource.address != resource.address
                || peer.resource.server_name != endpoint.dns_san
                || peer.resource.leaf_pin != endpoint.certificate_sha256
                || sha256(
                    &peer
                        .resource
                        .credentials
                        .ca
                        .to_der()
                        .map_err(|_| Error::Transport(Stage::Configuration))?,
                ) != resource.ca_sha256
                || sha256(
                    &peer
                        .resource
                        .credentials
                        .certificate
                        .to_der()
                        .map_err(|_| Error::Transport(Stage::Configuration))?,
                ) != relay.certificate_sha256
                || sha256(
                    &local
                        .credentials
                        .certificate
                        .to_der()
                        .map_err(|_| Error::Transport(Stage::Configuration))?,
                ) != relay.certificate_sha256
            {
                return Err(Error::Transport(Stage::Configuration));
            }
        }
        drop(authority);
        Ok(Self {
            state: Arc::downgrade(&runtime.state),
            policy,
            local,
            peers,
            #[cfg(all(test, feature = "durable-policy"))]
            authority_attempt: None,
        })
    }
    #[cfg(all(test, feature = "durable-policy"))]
    pub(crate) fn observe_authority_metadata_attempt(
        &mut self,
        entered: std::sync::mpsc::SyncSender<bool>,
    ) {
        self.authority_attempt = Some(entered);
    }
    fn current(&self) -> Result<(), Error> {
        let state = self
            .state
            .upgrade()
            .ok_or(Error::Transport(Stage::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(Stage::Denied));
        }
        let lookup = state.authority.try_lock();
        #[cfg(all(test, feature = "durable-policy"))]
        if let Some(entered) = &self.authority_attempt {
            entered
                .send(matches!(&lookup, Err(std::sync::TryLockError::WouldBlock)))
                .unwrap();
        }
        let authority = match lookup {
            Ok(authority) => authority,
            Err(std::sync::TryLockError::WouldBlock) => state
                .authority
                .lock()
                .map_err(|_| Error::Store(faction_files::private_blobs::Error::Busy))?,
            Err(std::sync::TryLockError::Poisoned(_)) => {
                return Err(Error::Store(faction_files::private_blobs::Error::Busy));
            }
        };
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(Stage::Denied));
        }
        if authority.relay_policy().map_err(Error::Store)? != self.policy {
            return Err(Error::Transport(Stage::Denied));
        }
        Ok(())
    }
    fn peer(&self, home: &str) -> Result<&RelayPeer, Error> {
        self.current()?;
        self.peers
            .iter()
            .find(|p| p.home == home)
            .ok_or(Error::Transport(Stage::Denied))
    }
}
struct OwnHomeWork {
    record: Arc<crate::retrieval::DeliveryRecord>,
    shared: Arc<Shared>,
    audience: String,
}
enum WorkKind {
    Publication(Box<PublicationWork>),
    Retrieval(Box<RetrievalWork>),
    OwnHome(Box<OwnHomeWork>),
}
/// A source-qualified consuming operation; no result injection API exists.
pub struct Work {
    kind: WorkKind,
}
impl Work {
    pub fn execute(self, routes: &RelayRoutes, store: &Mutex<Store>) -> Result<(), Error> {
        match self.kind {
            WorkKind::Publication(work) => {
                let home = work
                    .shared
                    .source
                    .binding
                    .recipient
                    .account
                    .split_once('@')
                    .map(|(_, h)| h)
                    .ok_or(Error::Transport(Stage::Denied))?;
                match routes.peer(home) {
                    Ok(peer) => (*work).execute(&peer.publication, store),
                    Err(e) => work.shared.finish(Err(e)),
                }
            }
            WorkKind::Retrieval(work) => {
                let home = work
                    .shared
                    .source
                    .binding
                    .recipient
                    .account
                    .split_once('@')
                    .map(|(_, h)| h)
                    .ok_or(Error::Transport(Stage::Denied))?;
                match routes.peer(home) {
                    Ok(peer) => (*work).execute(&peer.introspection, store),
                    Err(e) => work.shared.finish(Err(e)),
                }
            }
            WorkKind::OwnHome(work) => (*work).execute(routes),
        }
    }
}
impl OwnHomeWork {
    fn execute(self, routes: &RelayRoutes) -> Result<(), Error> {
        #[cfg(test)]
        let _setup_scope = self.shared.trace.get().map(|attached| {
            attached
                .trace
                .setup
                .scope(crate::setup_trace::Role::OwnHome)
        });
        let result = (|| {
            self.shared.charge()?;
            let state = self
                .shared
                .source
                .state
                .upgrade()
                .ok_or(Error::Transport(Stage::Denied))?;
            let control = crate::Control {
                state: Arc::downgrade(&state),
            };
            let reference = control.provision_retrieval(self.record.record())?;
            let sender = self
                .shared
                .source
                .binding
                .sender
                .account
                .split_once('@')
                .map(|(_, h)| h)
                .ok_or(Error::Transport(Stage::Denied))?;
            let peer = routes.peer(sender)?;
            let route = routes
                .policy
                .issuer
                .as_ref()
                .and_then(|p| {
                    p.routes
                        .iter()
                        .find(|r| r.sender_home == sender && r.audience == self.audience)
                })
                .ok_or(Error::Transport(Stage::Denied))?;
            let endpoint = HttpEndpoint::new(
                routes.local.address,
                routes
                    .policy
                    .issuer
                    .as_ref()
                    .ok_or(Error::Transport(Stage::Denied))?
                    .issuer
                    .clone(),
                routes.local.server_name.clone(),
                routes.local.leaf_pin,
                routes.local.credentials.clone(),
            )
            .map_err(Error::Transport)?;
            let shared = self.shared.clone();
            let probe = Arc::new(move || shared.current().is_ok());
            let (token, expires) = endpoint
                .issue_checked(
                    &route.relay_client_id,
                    *reference.expose_bytes(),
                    self.shared.source.deadline,
                    self.shared.source.cancel.clone(),
                    probe,
                )
                .map_err(Error::Transport)?;
            #[cfg(test)]
            if let Some(trace) = self.shared.trace.get() {
                trace.trace.issue_success();
            }
            *self
                .shared
                .tight_deadline
                .lock()
                .map_err(|_| Error::Transport(Stage::Denied))? = expires;
            self.shared.current()?;
            // Account for the fresh resource operation and its one introspection
            // callback in this original source window before starting either.
            self.shared.charge()?;
            self.shared.charge()?;
            let mut operation = Operation {
                endpoint: peer.resource.clone(),
                deadline: expires.min(self.shared.source.deadline),
                cancel: self.shared.source.cancel.clone(),
                attempted: 0,
                current_probe: Some({
                    let shared = self.shared.clone();
                    Arc::new(move || shared.current().is_ok())
                }),
                next_collect: None,
                collect_delay: Duration::ZERO,
                ticket: None,
                diagnostic: None,
                diagnostic_start: None,
                #[cfg(test)]
                wait_entered: None,
            };
            operation
                .submit_retrieval(
                    self.shared.source.binding.clone(),
                    self.shared.source.request.clone(),
                    token,
                )
                .map_err(Error::Transport)?;
            for number in 0..MAX_OPERATIONS {
                if number != 0 {
                    operation.wait_for_collect().map_err(Error::Transport)?;
                }
                self.shared.charge()?;
                match operation.collect_waiting().map_err(Error::Transport)? {
                    Collection::Complete(response) => {
                        self.shared.current()?;
                        return Ok(response);
                    }
                    Collection::Busy => {
                        self.shared.current()?;
                        std::thread::sleep(
                            Duration::from_millis(25)
                                .min(expires.saturating_duration_since(Instant::now())),
                        );
                    }
                }
            }
            Err(Error::Transport(Stage::Limit))
        })();
        self.shared.finish(result)
    }
}
pub enum Event {
    Submitted(Delivery, Work),
    Collected(bool),
    Served,
}
enum IncomingKind {
    Device(Session),
    Service(ServiceSession),
}
/// Created only by real role-classifying mTLS admission. Its fields stay private.
pub struct Incoming {
    kind: IncomingKind,
    #[cfg(all(test, feature = "durable-policy"))]
    service_purpose_attempt: Option<std::sync::mpsc::SyncSender<usize>>,
}
impl Incoming {
    pub(crate) fn device(session: Session) -> Self {
        Self {
            kind: IncomingKind::Device(session),
            #[cfg(all(test, feature = "durable-policy"))]
            service_purpose_attempt: None,
        }
    }
    pub(crate) fn service(session: ServiceSession) -> Self {
        Self {
            kind: IncomingKind::Service(session),
            #[cfg(all(test, feature = "durable-policy"))]
            service_purpose_attempt: None,
        }
    }
    // Observe the original lookup attempt, without changing its result or
    // holding a test gate through Session drop. No production build has this.
    #[cfg(all(test, feature = "durable-policy"))]
    pub(crate) fn observe_service_purpose_attempt(
        &mut self,
        entered: std::sync::mpsc::SyncSender<usize>,
    ) {
        self.service_purpose_attempt = Some(entered);
    }
    #[cfg(test)]
    pub(crate) fn observe_service_response_prefix(
        &mut self,
        entered: std::sync::mpsc::Sender<Instant>,
        release: std::sync::mpsc::Receiver<()>,
    ) -> Arc<AtomicUsize> {
        let IncomingKind::Service(session) = &mut self.kind else {
            panic!("test response observation requires an admitted service");
        };
        let positive = Arc::new(AtomicUsize::new(0));
        session.stream.get_mut().delivery_prefix_gate = Some(Arc::new(DeliveryPrefixGate {
            gate: ResponseWriteGate {
                entered,
                release: Mutex::new(release),
            },
            positive: positive.clone(),
        }));
        positive
    }
    pub fn serve(
        self,
        pending: Option<&mut Delivery>,
        store: &Mutex<Store>,
        home: &str,
        publication_profile: &crate::services::PublicationWireProfile,
    ) -> Result<Event, Error> {
        match self.kind {
            IncomingKind::Device(session) => session.serve_delivery(pending, store, home),
            IncomingKind::Service(mut session) => {
                let state = session
                    .state
                    .upgrade()
                    .ok_or(Error::Transport(Stage::Denied))?;
                session.issuer_check(&state)?;
                let lookup = state.authority.try_lock();
                #[cfg(all(test, feature = "durable-policy"))]
                if let Some(entered) = self.service_purpose_attempt {
                    entered.send(session.original_service_attempts()).unwrap();
                }
                let purpose = {
                    let authority = match lookup {
                        Ok(authority) => authority,
                        Err(std::sync::TryLockError::WouldBlock) => state
                            .authority
                            .lock()
                            .map_err(|_| Error::Store(faction_files::private_blobs::Error::Busy))?,
                        Err(std::sync::TryLockError::Poisoned(_)) => {
                            return Err(Error::Store(faction_files::private_blobs::Error::Busy));
                        }
                    };
                    session.issuer_check(&state)?;
                    authority
                        .current_service_use(&session.credential, &session.leaf, wall_now())
                        .map_err(Error::Store)?
                };
                // The short authority guard is gone before any HTTP/network I/O.
                match purpose {
                    crate::services::ServiceUse::Observation => {
                        session.serve_publication_http_once(publication_profile)?;
                        Ok(Event::Served)
                    }
                    crate::services::ServiceUse::Relay
                    | crate::services::ServiceUse::Introspection => {
                        session.serve_http_once()?;
                        Ok(Event::Served)
                    }
                    crate::services::ServiceUse::ResourceClient => session.serve_delivery(pending),
                    _ => Err(Error::Transport(Stage::Denied)),
                }
            }
        }
    }
}
impl Session {
    fn submit_publication_message(
        mut self,
        state: &Arc<State>,
        message: Message,
    ) -> Result<(Delivery, PublicationWork), Error> {
        let pending = self.capture_publication(state, message.binding, message.request)?;
        let delivery = Delivery::new(pending.delivery_source())?;
        self.write_delivery(Some(delivery.receipt()), delivery.shared.clone())?;
        let shared = delivery.shared.clone();
        drop(self);
        Ok((delivery, PublicationWork { pending, shared }))
    }
    fn submit_own_home_message(
        mut self,
        state: &Arc<State>,
        message: Message,
    ) -> Result<(Delivery, OwnHomeWork), Error> {
        let (record, lease, audience) = {
            let mut authority = state
                .authority
                .lock()
                .map_err(|_| Error::Transport(Stage::Denied))?;
            authority.capture_delivery_record(
                state,
                crate::retrieval::Source {
                    connection: &self.connection,
                    handle: &self.handle,
                    leaf: &self.leaf,
                    deadline: self.deadline,
                    elapsed_ms: self.start.elapsed().as_millis() as u64,
                },
                message.binding.clone(),
                &message.request,
            )?
        };
        let record = Arc::new(crate::retrieval::DeliveryRecord::new(record));
        let current_record = record.clone();
        let current_binding = message.binding.clone();
        let current_audience = audience.clone();
        let control = crate::Control {
            state: Arc::downgrade(state),
        };
        let action = match &message.request {
            Request::GetChunk { offset, .. } => {
                crate::retrieval::RetrievalAction::GetChunk { offset: *offset }
            }
            Request::Status { .. } => crate::retrieval::RetrievalAction::Status,
            _ => return Err(Error::Transport(Stage::Denied)),
        };
        let source = Source {
            state: Arc::downgrade(state),
            peer: Peer::Device(sha256(
                &self
                    .leaf
                    .to_der()
                    .map_err(|_| Error::Transport(Stage::Denied))?,
            )),
            purpose: 3,
            binding: message.binding,
            request: message.request,
            deadline: self.deadline,
            cancel: self.cancel.clone(),
            attempted: self.attempted,
            _lease: lease,
            current: Arc::new(move || {
                control
                    .check_retrieval(
                        current_record.record(),
                        &current_binding,
                        action,
                        &current_audience,
                    )
                    .is_ok()
            }),
        };
        let delivery = Delivery::new(source)?;
        self.write_delivery(Some(delivery.receipt()), delivery.shared.clone())?;
        let shared = delivery.shared.clone();
        drop(self);
        Ok((
            delivery,
            OwnHomeWork {
                record,
                shared,
                audience,
            },
        ))
    }
    fn serve_delivery(
        mut self,
        pending: Option<&mut Delivery>,
        store: &Mutex<Store>,
        home: &str,
    ) -> Result<Event, Error> {
        let state = self.attempt()?;
        let peer = Peer::Device(sha256(
            &self
                .leaf
                .to_der()
                .map_err(|_| Error::Transport(Stage::Denied))?,
        ));
        let mut charged = false;
        if let Some(d) = pending.as_deref()
            && d.shared.source.peer == peer
        {
            d.begin_collect(&peer, &self.state)?;
            charged = true;
            self.deadline = self.deadline.min(d.shared.source.deadline);
            self.stream.get_mut().deadline = self.deadline;
            let probe = d.shared.clone();
            self.stream.get_mut().policy_probe = Some(Arc::new(move || probe.current().is_ok()));
        }
        #[cfg(test)]
        FrameObservation::phase(FramePhase::InitialRead);
        let bytes = read_frame(&mut self.stream)
            .map_err(|s| Error::Transport(classify(self.deadline, &self.cancel, s)))?
            .ok_or(Error::Transport(Stage::Frame))?;
        if !bytes.starts_with(MAGIC) {
            // The first frame pins this connection to the existing binary
            // carrier. Preserve its original Session for every later request.
            let service = BinaryService::new(home)?;
            let mut bytes = bytes;
            let mut state = state;
            loop {
                #[cfg(test)]
                FrameObservation::phase(FramePhase::DecodeDispatch);
                if bytes.starts_with(MAGIC) {
                    return Err(Error::Transport(Stage::Denied));
                }
                let request = decode_request(&bytes).map_err(Error::Store)?;
                let mut guard = store
                    .try_lock()
                    .map_err(|_| Error::Store(faction_files::private_blobs::Error::Busy))?;
                let response = self.dispatch_authorized(
                    &state,
                    &mut guard,
                    DispatchRoute::Local(&service),
                    request,
                )?;
                drop(guard);
                #[cfg(test)]
                FrameObservation::phase(FramePhase::ResponseWrite);
                write_frame(
                    &mut self.stream,
                    &encode_response(&response).map_err(Error::Store)?,
                )
                .map_err(|s| Error::Transport(classify(self.deadline, &self.cancel, s)))?;
                check(self.deadline, &self.cancel).map_err(Error::Transport)?;
                state = self.attempt()?;
                if let Some(delivery) = pending.as_deref().filter(|_| charged) {
                    delivery.shared.current()?;
                    if delivery.shared.attempted.load(Ordering::SeqCst) >= MAX_OPERATIONS {
                        return Err(Error::Transport(Stage::Limit));
                    }
                }
                #[cfg(test)]
                FrameObservation::phase(FramePhase::NextRead);
                let Some(next) = read_frame(&mut self.stream)
                    .map_err(|s| Error::Transport(classify(self.deadline, &self.cancel, s)))?
                else {
                    return Ok(Event::Served);
                };
                // EOF spends no additional pending-operation attempt. Every
                // actual subsequent frame is charged before any decoding.
                if let Some(delivery) = pending.as_deref().filter(|_| charged) {
                    delivery.begin_collect(&peer, &self.state)?;
                }
                bytes = next;
            }
        }
        let message = Message::decode(&bytes)?;
        if [COLLECT, WAIT_COLLECT].contains(&message.kind) {
            let delivery = pending.ok_or(Error::Transport(Stage::Denied))?;
            if !charged {
                delivery.begin_collect(&peer, &self.state)?;
            }
            let guard = delivery.shared.clone();
            let source_state = self.state.clone();
            let deadline = self.deadline;
            let reply = delivery.collect(&bytes, peer, &source_state, Some(deadline), || {
                check(deadline, &self.cancel).map_err(Error::Transport)?;
                if state.stopped.load(Ordering::SeqCst) || !leaf_current(&self.leaf, wall_now()) {
                    return Err(Error::Transport(Stage::Denied));
                }
                Ok(())
            })?;
            let result = self.write_delivery(reply, guard.clone());
            drop(self);
            drop(guard);
            return result.map(Event::Collected);
        }
        if pending.is_some() {
            return Err(Error::Store(faction_files::private_blobs::Error::Busy));
        }
        match message.purpose {
            1 => self
                .submit_publication_message(&state, message)
                .map(|(d, w)| {
                    Event::Submitted(
                        d,
                        Work {
                            kind: WorkKind::Publication(Box::new(w)),
                        },
                    )
                }),
            3 => self.submit_own_home_message(&state, message).map(|(d, w)| {
                Event::Submitted(
                    d,
                    Work {
                        kind: WorkKind::OwnHome(Box::new(w)),
                    },
                )
            }),
            _ => Err(Error::Transport(Stage::Denied)),
        }
    }
}
impl ServiceSession {
    fn serve_delivery(mut self, pending: Option<&mut Delivery>) -> Result<Event, Error> {
        let state = self.issuer_attempt()?;
        let peer = Peer::Service(self.credential.digest, self.credential.identity.clone());
        let mut charged = false;
        if let Some(d) = pending.as_deref()
            && d.shared.source.peer == peer
        {
            d.begin_collect(&peer, &self.state)?;
            charged = true;
            self.deadline = self.deadline.min(d.shared.source.deadline);
            self.stream.get_mut().deadline = self.deadline;
            let probe = d.shared.clone();
            self.stream.get_mut().policy_probe = Some(Arc::new(move || probe.current().is_ok()));
        }
        let bytes = read_frame(&mut self.stream)
            .map_err(|s| Error::Transport(classify(self.deadline, &self.cancel, s)))?
            .ok_or(Error::Transport(Stage::Frame))?;
        let message = Message::decode(&bytes)?;
        if [COLLECT, WAIT_COLLECT].contains(&message.kind) {
            let delivery = pending.ok_or(Error::Transport(Stage::Denied))?;
            if !charged {
                delivery.begin_collect(&peer, &self.state)?;
            }
            let guard = delivery.shared.clone();
            let source_state = self.state.clone();
            let deadline = self.deadline;
            let reply = delivery.collect(&bytes, peer, &source_state, Some(deadline), || {
                self.issuer_check(&state)
            })?;
            let ready = reply.is_some();
            let reply = reply.unwrap_or_else(|| {
                let mut b = MAGIC.to_vec();
                b.push(BUSY);
                b
            });
            self.deadline = self.deadline.min(guard.source.deadline);
            self.stream.get_mut().deadline = self.deadline;
            let probe = guard.clone();
            self.stream.get_mut().policy_probe = Some(Arc::new(move || probe.current().is_ok()));
            write_frame(&mut self.stream, &reply).map_err(Error::Transport)?;
            self.issuer_check(&state)?;
            drop(self);
            drop(guard);
            return Ok(Event::Collected(ready));
        }
        if pending.is_some() || message.purpose != 2 {
            return Err(Error::Store(faction_files::private_blobs::Error::Busy));
        }
        let pending = self.capture_retrieval_charged(
            &state,
            message.binding,
            message.request,
            faction_delegation_codec::UnverifiedToken::from_bytes(
                message.token.ok_or(Error::Transport(Stage::Frame))?,
            ),
        )?;
        let delivery = Delivery::new(pending.delivery_source())?;
        let probe = delivery.shared.clone();
        self.stream.get_mut().policy_probe = Some(Arc::new(move || probe.current().is_ok()));
        write_frame(&mut self.stream, &delivery.receipt()).map_err(Error::Transport)?;
        let shared = delivery.shared.clone();
        drop(self);
        Ok(Event::Submitted(
            delivery,
            Work {
                kind: WorkKind::Retrieval(Box::new(RetrievalWork { pending, shared })),
            },
        ))
    }
}

#[cfg(test)]
impl Operation {
    pub(crate) fn test_observe_collect_wait(&mut self, entered: std::sync::mpsc::SyncSender<()>) {
        self.wait_entered = Some(entered);
    }
    pub(crate) fn test_attempts(&self) -> usize {
        self.attempted
    }
    pub(crate) fn test_narrow_deadline(&mut self, remaining: Duration) {
        // Conservative test-only narrowing; never renew the original window.
        self.deadline = self.deadline.min(Instant::now() + remaining);
    }
    pub(crate) fn test_collect_frame(&self) -> Vec<u8> {
        self.ticket
            .as_ref()
            .expect("actual submitted ticket")
            .encode()
            .unwrap()
    }
}
#[cfg(test)]
impl CarrierEndpoint {
    pub(crate) fn test_connect_nodelay(&self) -> Result<bool, Stage> {
        let client = self.connect(
            Instant::now() + CYCLE_LIMIT,
            Arc::new(AtomicBool::new(false)),
            None,
        )?;
        client
            .stream
            .get_ref()
            .socket
            .nodelay()
            .map_err(|_| Stage::Configuration)
    }
    /// Test-only raw application output capture, with real fresh mTLS. Routing
    /// bytes remain unverified; no actor, response or authority can be injected.
    pub(crate) fn test_exchange(&self, bytes: &[u8]) -> Vec<u8> {
        let deadline = Instant::now() + CYCLE_LIMIT;
        let mut client = self
            .connect(deadline, Arc::new(AtomicBool::new(false)), None)
            .unwrap();
        let _sent = write_frame(&mut client.stream, bytes);
        let mut received = Vec::new();
        let mut scratch = [0; 1024];
        loop {
            match client.stream.read(&mut scratch) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    assert!(received.len() + n <= MAX_WIRE + 4);
                    received.extend_from_slice(&scratch[..n]);
                }
            }
        }
        received
    }
}

// Test witnesses expose only contention/notification, never fabricated results.
#[cfg(test)]
impl Session {
    // An authenticated test peer deliberately closes or emits invalid framing;
    // this helper neither dispatches a Store action nor changes normal TLS I/O.
    pub(crate) fn test_delivery_raw_reply(mut self, bytes: &[u8]) {
        assert!(read_frame(&mut self.stream).unwrap().is_some());
        self.stream.write_all(bytes).unwrap();
        self.stream.flush().unwrap();
        let _ = self.stream.shutdown();
    }
}
#[cfg(test)]
pub(crate) struct CompletionWitness {
    shared: Arc<Shared>,
}
#[cfg(test)]
impl CompletionWitness {
    #[cfg(feature = "operator")]
    pub(crate) fn test_original_attempts(&self) -> usize {
        self.shared.attempted.load(Ordering::SeqCst)
    }
    pub(crate) fn hold(&self) -> std::sync::MutexGuard<'_, Option<Result<Response, Error>>> {
        self.shared.completion.lock().unwrap()
    }
    pub(crate) fn notify_spurious(&self) {
        self.shared.completed.notify_all();
    }
    pub(crate) fn observe_finish(&self, observer: std::sync::mpsc::SyncSender<()>) {
        *self.shared.finish_observer.lock().unwrap() = Some(observer);
    }
}
#[cfg(test)]
impl Delivery {
    pub(crate) fn test_completion_witness(&self) -> CompletionWitness {
        CompletionWitness {
            shared: self.shared.clone(),
        }
    }
    pub(crate) fn test_observe_waiting_collect(
        &self,
        observer: std::sync::mpsc::SyncSender<(Instant, Instant)>,
    ) {
        *self.shared.wait_observer.lock().unwrap() = Some(observer);
    }
}

#[cfg(test)]
impl ServiceSession {
    // Emulate the previous decoder's supported-kind boundary over real mTLS.
    // This is not an executed historical server or interoperability evidence.
    pub(crate) fn test_legacy_collect_only(mut self) -> Result<(), Error> {
        self.issuer_attempt()?;
        let bytes = read_frame(&mut self.stream)
            .map_err(Error::Transport)?
            .ok_or(Error::Transport(Stage::Frame))?;
        assert!(bytes.starts_with(MAGIC));
        assert_eq!(
            bytes.get(6),
            Some(&WAIT_COLLECT),
            "test did not send actual opt-in wire kind6"
        );
        if ![SUBMIT, COLLECT].contains(&bytes[6]) {
            return Err(Error::Transport(Stage::Frame));
        }
        Message::decode(&bytes)?;
        Ok(())
    }
}

#[cfg(test)]
struct TraceAttachment {
    trace: Arc<collection_trace::Trace>,
    role: collection_trace::Role,
}
#[cfg(test)]
fn trace_get_request_matches(binding_id: &[u8; 32], request: &Request) -> bool {
    matches!(request, Request::GetChunk { id, offset: 0 } if id == binding_id)
}
#[cfg(all(test, feature = "operator"))]
impl Delivery {
    pub(crate) fn test_attach_collection_trace(&mut self, trace: Arc<collection_trace::Trace>) {
        let role = match (&self.shared.source.peer, self.shared.source.purpose) {
            (Peer::Device(_), 3) => collection_trace::Role::Outer,
            (Peer::Service(..), 2) => collection_trace::Role::Inner,
            _ => return,
        };
        if !trace_get_request_matches(&self.shared.source.binding.id, &self.shared.source.request) {
            trace.invalid();
            return;
        }
        if self
            .shared
            .trace
            .set(TraceAttachment {
                trace: trace.clone(),
                role,
            })
            .is_err()
        {
            trace.invalid();
        } else {
            trace.attached(role);
        }
    }
}

// Test-owned numeric chronology only. It retains no delivery, authority, token,
// Session or lease. The fixed atomics never send, allocate or wait while recording.
#[cfg(test)]
pub(crate) mod collection_trace {
    use super::*;
    use std::cell::Cell;
    const CAPACITY: usize = 64;
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub(crate) enum Role {
        Outer,
        Inner,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub(crate) enum WireKind {
        None,
        Collect,
        Wait,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub(crate) enum Event {
        Validated,
        // Computed permitted condition-wait window, not a Condvar park interval.
        WaitBounds,
        WaitEnd,
        AlreadyFinished,
        // Reply decision before socket output; transport success is independent.
        Outcome,
        IssueSuccess,
        FinishBefore,
        FinishAfter,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub(crate) enum Outcome {
        None,
        Busy,
        Result,
        Refused,
        Error,
    }
    struct Slot {
        ready: AtomicBool,
        tag: AtomicU64,
        micros: AtomicU64,
        first_us: AtomicU64,
        last_us: AtomicU64,
    }
    impl Slot {
        fn new() -> Self {
            Self {
                ready: AtomicBool::new(false),
                tag: AtomicU64::new(0),
                micros: AtomicU64::new(0),
                first_us: AtomicU64::new(0),
                last_us: AtomicU64::new(0),
            }
        }
    }
    pub(crate) struct Trace {
        origin: Instant,
        pub(crate) setup: Arc<crate::setup_trace::Trace>,
        enabled: AtomicBool,
        bad: AtomicBool,
        attached: AtomicUsize,
        next: AtomicUsize,
        attempts: [AtomicUsize; 2],
        slots: [Slot; CAPACITY],
    }
    #[derive(Clone, Debug)]
    pub(crate) struct Point {
        // Atomic slot reservation order, not a cross-thread clock order.
        pub(crate) sequence: usize,
        pub(crate) micros: u64,
        pub(crate) role: Role,
        pub(crate) kind: WireKind,
        pub(crate) attempt: usize,
        pub(crate) event: Event,
        pub(crate) outcome: Outcome,
        pub(crate) first_us: u64,
        pub(crate) last_us: u64,
    }
    #[derive(Debug)]
    pub(crate) struct Snapshot {
        pub(crate) points: Vec<Point>,
        pub(crate) inconclusive: bool,
    }
    impl Trace {
        pub(crate) fn new(origin: Instant) -> Arc<Self> {
            Arc::new(Self {
                origin,
                setup: crate::setup_trace::Trace::new(origin),
                enabled: AtomicBool::new(false),
                bad: AtomicBool::new(false),
                attached: AtomicUsize::new(0),
                next: AtomicUsize::new(0),
                attempts: std::array::from_fn(|_| AtomicUsize::new(0)),
                slots: std::array::from_fn(|_| Slot::new()),
            })
        }
        pub(crate) fn arm(&self) {
            self.enabled.store(true, Ordering::SeqCst);
            self.setup.arm();
        }
        pub(super) fn invalid(&self) {
            self.bad.store(true, Ordering::SeqCst);
        }
        pub(crate) fn attached(&self, role: Role) {
            if self.attached.fetch_or(1 << role as usize, Ordering::SeqCst) & (1 << role as usize)
                != 0
            {
                self.invalid();
            }
        }
        fn micros(&self, at: Instant) -> u64 {
            at.saturating_duration_since(self.origin)
                .as_micros()
                .try_into()
                .unwrap_or(u64::MAX)
        }
        fn emit(
            &self,
            role: Role,
            kind: WireKind,
            attempt: usize,
            event: Event,
            outcome: Outcome,
            bounds: Option<(Instant, Instant)>,
        ) {
            if !self.enabled.load(Ordering::SeqCst) {
                return;
            }
            let stamp = Instant::now();
            let Ok(index) = self
                .next
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                    (n < CAPACITY).then_some(n + 1)
                })
            else {
                self.invalid();
                return;
            };
            let slot = &self.slots[index];
            let tag = role as u64
                | ((kind as u64) << 8)
                | ((event as u64) << 16)
                | ((attempt as u64) << 24)
                | ((outcome as u64) << 32);
            slot.tag.store(tag, Ordering::Relaxed);
            slot.micros.store(self.micros(stamp), Ordering::Relaxed);
            if let Some((first, last)) = bounds {
                slot.first_us.store(self.micros(first), Ordering::Relaxed);
                slot.last_us.store(self.micros(last), Ordering::Relaxed);
            }
            slot.ready.store(true, Ordering::Release);
        }
        pub(crate) fn validated(&self, role: Role, kind: WireKind) -> Option<Collect<'_>> {
            if !self.enabled.load(Ordering::SeqCst) {
                return None;
            }
            let Ok(attempt) = self.attempts[role as usize].fetch_update(
                Ordering::SeqCst,
                Ordering::SeqCst,
                |n| (n < 8).then_some(n + 1),
            ) else {
                self.invalid();
                return None;
            };
            self.emit(role, kind, attempt, Event::Validated, Outcome::None, None);
            Some(Collect {
                trace: self,
                role,
                kind,
                attempt,
                completed: Cell::new(false),
            })
        }
        pub(crate) fn issue_success(&self) {
            self.emit(
                Role::Outer,
                WireKind::None,
                0,
                Event::IssueSuccess,
                Outcome::Result,
                None,
            );
        }
        pub(crate) fn finish_before(&self, role: Role, success: bool) {
            self.emit(
                role,
                WireKind::None,
                0,
                Event::FinishBefore,
                if success {
                    Outcome::Result
                } else {
                    Outcome::Error
                },
                None,
            );
        }
        pub(crate) fn finish_after(&self, role: Role, success: bool) {
            self.emit(
                role,
                WireKind::None,
                0,
                Event::FinishAfter,
                if success {
                    Outcome::Result
                } else {
                    Outcome::Error
                },
                None,
            );
        }
        pub(crate) fn snapshot(&self) -> Snapshot {
            let mut points = Vec::with_capacity(CAPACITY);
            let mut bad = self.bad.load(Ordering::SeqCst);
            for (index, slot) in self.slots[..self.next.load(Ordering::SeqCst)]
                .iter()
                .enumerate()
            {
                if !slot.ready.load(Ordering::Acquire) {
                    bad = true;
                    continue;
                }
                let t = slot.tag.load(Ordering::Relaxed);
                let role = [Role::Outer, Role::Inner][(t & 255) as usize];
                let kind =
                    [WireKind::None, WireKind::Collect, WireKind::Wait][((t >> 8) & 255) as usize];
                let event = [
                    Event::Validated,
                    Event::WaitBounds,
                    Event::WaitEnd,
                    Event::AlreadyFinished,
                    Event::Outcome,
                    Event::IssueSuccess,
                    Event::FinishBefore,
                    Event::FinishAfter,
                ][((t >> 16) & 255) as usize];
                let outcome = [
                    Outcome::None,
                    Outcome::Busy,
                    Outcome::Result,
                    Outcome::Refused,
                    Outcome::Error,
                ][((t >> 32) & 255) as usize];
                points.push(Point {
                    sequence: index + 1,
                    micros: slot.micros.load(Ordering::Relaxed),
                    role,
                    kind,
                    attempt: ((t >> 24) & 255) as usize,
                    event,
                    outcome,
                    first_us: slot.first_us.load(Ordering::Relaxed),
                    last_us: slot.last_us.load(Ordering::Relaxed),
                });
            }
            let registered = self.attached.load(Ordering::SeqCst);
            let mut states = [[0u8; 8]; 2];
            let mut kinds = [[WireKind::None; 8]; 2];
            let mut finish = [0u8; 2];
            let mut finish_start = [0u64; 2];
            let mut finish_result = [Outcome::None; 2];
            let mut issues = 0usize;
            for p in &points {
                let r = p.role as usize;
                if matches!(p.event, Event::FinishBefore | Event::FinishAfter)
                    && p.outcome != Outcome::Result
                {
                    bad = true;
                }
                if p.event == Event::Outcome
                    && !matches!(p.outcome, Outcome::Busy | Outcome::Result)
                {
                    bad = true;
                }
                if registered & (1 << r) == 0 {
                    bad = true;
                }
                match p.event {
                    Event::IssueSuccess => {
                        issues += 1;
                        if p.role != Role::Outer {
                            bad = true;
                        }
                    }
                    Event::FinishBefore => {
                        if finish[r] != 0 {
                            bad = true;
                        }
                        finish[r] = 1;
                        finish_start[r] = p.micros;
                        finish_result[r] = p.outcome;
                    }
                    Event::FinishAfter => {
                        if finish[r] != 1
                            || p.micros < finish_start[r]
                            || p.outcome != finish_result[r]
                        {
                            bad = true;
                        }
                        finish[r] = 2;
                    }
                    _ => {
                        if p.attempt >= 8 {
                            bad = true;
                            continue;
                        }
                        let state = &mut states[r][p.attempt];
                        if p.event != Event::Validated && kinds[r][p.attempt] != p.kind {
                            bad = true;
                        }
                        if p.role == Role::Outer && p.kind != WireKind::Collect {
                            bad = true;
                        }
                        match p.event {
                            Event::Validated if *state == 0 && p.kind != WireKind::None => {
                                *state = 1;
                                kinds[r][p.attempt] = p.kind;
                            }
                            Event::WaitBounds
                                if *state == 1
                                    && p.kind == WireKind::Wait
                                    && p.first_us <= p.last_us
                                    && p.last_us - p.first_us <= 100_000 =>
                            {
                                *state = 2
                            }
                            Event::WaitEnd if *state == 2 => *state = 3,
                            Event::AlreadyFinished if *state == 1 && p.kind == WireKind::Wait => {
                                *state = 3
                            }
                            Event::Outcome
                                if (*state == 1 && p.kind == WireKind::Collect)
                                    || (*state == 3 && p.kind == WireKind::Wait) =>
                            {
                                *state = 4
                            }
                            _ => bad = true,
                        }
                    }
                }
            }
            if self.enabled.load(Ordering::SeqCst) {
                if registered == 0 {
                    bad = true;
                }
                for r in 0..2 {
                    if registered & (1 << r) != 0 {
                        let n = self.attempts[r].load(Ordering::SeqCst);
                        if n == 0 || finish[r] != 2 || states[r][..n].iter().any(|&s| s != 4) {
                            bad = true;
                        }
                    }
                }
                if issues != usize::from(registered & 1 != 0) {
                    bad = true;
                }
            }
            Snapshot {
                points,
                inconclusive: bad,
            }
        }
    }
    pub(crate) struct Collect<'a> {
        trace: &'a Trace,
        role: Role,
        kind: WireKind,
        attempt: usize,
        completed: Cell<bool>,
    }
    impl Collect<'_> {
        pub(crate) fn bounds(&self, start: Instant, end: Instant) {
            self.trace.emit(
                self.role,
                self.kind,
                self.attempt,
                Event::WaitBounds,
                Outcome::None,
                Some((start, end)),
            );
        }
        pub(crate) fn wait_end(&self) {
            self.trace.emit(
                self.role,
                self.kind,
                self.attempt,
                Event::WaitEnd,
                Outcome::None,
                None,
            );
        }
        pub(crate) fn already_finished(&self) {
            self.trace.emit(
                self.role,
                self.kind,
                self.attempt,
                Event::AlreadyFinished,
                Outcome::None,
                None,
            );
        }
        pub(crate) fn outcome(&self, outcome: Outcome) {
            if self.completed.replace(true) {
                self.trace.invalid();
                return;
            }
            self.trace.emit(
                self.role,
                self.kind,
                self.attempt,
                Event::Outcome,
                outcome,
                None,
            );
        }
    }
    impl Drop for Collect<'_> {
        fn drop(&mut self) {
            if !self.completed.get() {
                self.trace.invalid();
                self.trace.emit(
                    self.role,
                    self.kind,
                    self.attempt,
                    Event::Outcome,
                    Outcome::Error,
                    None,
                );
            }
        }
    }
    #[cfg(test)]
    mod controls {
        use super::*;
        fn retain(name: &str, snapshot: &Snapshot) {
            use std::{io::Write, os::unix::fs::OpenOptionsExt};
            if let Some(root) = std::env::var_os("FACTION318_CAPTURE_DIR") {
                std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .mode(0o600)
                    .open(std::path::Path::new(&root).join(name))
                    .unwrap()
                    .write_all(format!("{snapshot:?}\n").as_bytes())
                    .unwrap();
            }
        }
        fn complete_outer(trace: &Trace) {
            trace.attached(Role::Outer);
            trace.issue_success();
            trace
                .validated(Role::Outer, WireKind::Collect)
                .unwrap()
                .outcome(Outcome::Busy);
            trace.finish_before(Role::Outer, true);
            trace.finish_after(Role::Outer, true);
        }
        #[test]
        fn collection_trace_controls_disabled_enabled_and_no_self_retention() {
            let trace = Trace::new(Instant::now());
            trace.issue_success();
            trace.finish_before(Role::Outer, true);
            trace.finish_after(Role::Outer, true);
            let disabled = trace.snapshot();
            retain("trace-disabled.txt", &disabled);
            assert!(disabled.points.is_empty());
            assert!(!disabled.inconclusive);
            trace.arm();
            complete_outer(&trace);
            let enabled = trace.snapshot();
            retain("trace-enabled.txt", &enabled);
            assert!(!enabled.inconclusive);
            assert_eq!(enabled.points.len(), 5);
            let weak = Arc::downgrade(&trace);
            drop(trace);
            assert!(weak.upgrade().is_none());
        }
        #[test]
        fn collection_trace_controls_wait_order_bounds_and_already_finished() {
            let origin = Instant::now();
            let trace = Trace::new(origin);
            trace.arm();
            trace.attached(Role::Inner);
            let collect = trace.validated(Role::Inner, WireKind::Wait).unwrap();
            collect.bounds(origin, origin + CANCEL_SLICE);
            collect.wait_end();
            collect.outcome(Outcome::Busy);
            let ready = trace.validated(Role::Inner, WireKind::Wait).unwrap();
            ready.already_finished();
            ready.outcome(Outcome::Result);
            trace.finish_before(Role::Inner, true);
            trace.finish_after(Role::Inner, true);
            let snapshot = trace.snapshot();
            retain("trace-order-bounds.txt", &snapshot);
            assert!(!snapshot.inconclusive);
            assert_eq!(
                snapshot.points.iter().map(|p| p.event).collect::<Vec<_>>(),
                vec![
                    Event::Validated,
                    Event::WaitBounds,
                    Event::WaitEnd,
                    Event::Outcome,
                    Event::Validated,
                    Event::AlreadyFinished,
                    Event::Outcome,
                    Event::FinishBefore,
                    Event::FinishAfter
                ]
            );
            assert_eq!(snapshot.points[1].first_us, 0);
            assert_eq!(snapshot.points[1].last_us, CANCEL_SLICE.as_micros() as u64);
            assert!(
                snapshot
                    .points
                    .windows(2)
                    .all(|w| w[0].sequence + 1 == w[1].sequence)
            );
        }
        #[test]
        fn collection_trace_controls_missing_duplicate_and_order_are_inconclusive() {
            let missing = Trace::new(Instant::now());
            missing.arm();
            missing.attached(Role::Inner);
            drop(missing.validated(Role::Inner, WireKind::Wait));
            let snapshot = missing.snapshot();
            retain("trace-missing.txt", &snapshot);
            assert!(snapshot.inconclusive);
            let duplicate = Trace::new(Instant::now());
            duplicate.arm();
            complete_outer(&duplicate);
            duplicate.attached(Role::Outer);
            let snapshot = duplicate.snapshot();
            retain("trace-duplicate.txt", &snapshot);
            assert!(snapshot.inconclusive);
            let order = Trace::new(Instant::now());
            order.arm();
            order.attached(Role::Inner);
            let collect = order.validated(Role::Inner, WireKind::Wait).unwrap();
            collect.wait_end();
            collect.already_finished();
            collect.outcome(Outcome::Result);
            order.finish_before(Role::Inner, true);
            order.finish_after(Role::Inner, true);
            let snapshot = order.snapshot();
            retain("trace-invalid-order.txt", &snapshot);
            assert!(snapshot.inconclusive);
        }
        #[test]
        fn collection_trace_controls_attempt_and_event_overflow_are_bounded() {
            let trace = Trace::new(Instant::now());
            trace.arm();
            trace.attached(Role::Outer);
            trace.issue_success();
            for _ in 0..8 {
                trace
                    .validated(Role::Outer, WireKind::Collect)
                    .unwrap()
                    .outcome(Outcome::Busy);
            }
            assert!(trace.validated(Role::Outer, WireKind::Collect).is_none());
            trace.finish_before(Role::Outer, false);
            trace.finish_after(Role::Outer, false);
            let snapshot = trace.snapshot();
            retain("trace-attempt-overflow.txt", &snapshot);
            assert!(snapshot.inconclusive);
            assert_eq!(snapshot.points.len(), 19);
            let trace = Trace::new(Instant::now());
            trace.arm();
            trace.attached(Role::Inner);
            for _ in 0..=CAPACITY {
                trace.finish_before(Role::Inner, true);
            }
            let snapshot = trace.snapshot();
            retain("trace-event-overflow.txt", &snapshot);
            assert!(snapshot.inconclusive);
            assert_eq!(snapshot.points.len(), CAPACITY);
        }
        #[test]
        fn collection_trace_controls_numeric_snapshot_has_no_payload_or_identity_fields() {
            let private_token = "TRACE_PRIVATE_TOKEN_SENTINEL";
            let private_payload = "TRACE_PRIVATE_PAYLOAD_SENTINEL";
            let trace = Trace::new(Instant::now());
            trace.arm();
            complete_outer(&trace);
            let snapshot = trace.snapshot();
            retain("trace-privacy.txt", &snapshot);
            let text = format!("{snapshot:?}");
            assert!(!text.contains(private_token));
            assert!(!text.contains(private_payload));
            assert!(!snapshot.inconclusive);
            assert_eq!(
                std::mem::size_of_val(&trace.slots),
                std::mem::size_of::<Slot>() * CAPACITY
            );
        }

        #[test]
        fn collection_trace_controls_review_failed_finish_and_terminal_outcomes() {
            let cases = [
                (Outcome::Result, false),
                (Outcome::Busy, false),
                (Outcome::Refused, true),
                (Outcome::Error, true),
                (Outcome::None, true),
                (Outcome::Refused, false),
                (Outcome::Error, false),
            ];
            let mut observed = Vec::new();
            for (index, (outcome, success)) in cases.into_iter().enumerate() {
                let trace = Trace::new(Instant::now());
                trace.arm();
                trace.attached(Role::Outer);
                trace.issue_success();
                trace
                    .validated(Role::Outer, WireKind::Collect)
                    .unwrap()
                    .outcome(outcome);
                trace.finish_before(Role::Outer, success);
                trace.finish_after(Role::Outer, success);
                let snapshot = trace.snapshot();
                retain(&format!("trace-review-outcome-{index}.txt"), &snapshot);
                observed.push((outcome, success, snapshot.inconclusive));
            }
            assert!(
                observed.iter().all(|(_, _, bad)| *bad),
                "failed or empty metadata must be inconclusive: {observed:?}"
            );
        }
        #[test]
        fn collection_trace_controls_review_exact_planned_get_request_guard() {
            use std::{io::Write, os::unix::fs::OpenOptionsExt};
            let id = [37; 32];
            let requests = [
                Request::GetChunk { id, offset: 0 },
                Request::Status { id },
                Request::GetChunk {
                    id: [38; 32],
                    offset: 0,
                },
                Request::GetChunk { id, offset: 1 },
            ];
            let observed = requests
                .iter()
                .map(|request| trace_get_request_matches(&id, request))
                .collect::<Vec<_>>();
            if let Some(root) = std::env::var_os("FACTION318_CAPTURE_DIR") {
                for (index, request) in requests.iter().enumerate() {
                    let bytes = encode_request(request).unwrap();
                    std::fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .mode(0o600)
                        .open(
                            std::path::Path::new(&root)
                                .join(format!("trace-review-guard-request-{index}.bin")),
                        )
                        .unwrap()
                        .write_all(&bytes)
                        .unwrap();
                }
                std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .mode(0o600)
                    .open(std::path::Path::new(&root).join("trace-review-guard-results.txt"))
                    .unwrap()
                    .write_all(format!("{observed:?}\n").as_bytes())
                    .unwrap();
            }
            assert_eq!(observed, vec![true, false, false, false]);
        }
    }
}
