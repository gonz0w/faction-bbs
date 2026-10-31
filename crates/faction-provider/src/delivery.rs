//! One source-bound continuation owner, not a result registry. Correlation only
//! routes a fresh authenticated collect to this exact operation.
use super::*;
use crate::services::{PendingPublication, PendingRetrieval, ServiceIdentity};
use faction_files::private_blobs::Binding;
use std::sync::{
    Mutex,
    atomic::{AtomicU64, AtomicUsize},
};
const MAGIC: &[u8; 6] = b"FDEL01";
const SUBMIT: u8 = 0;
const COLLECT: u8 = 1;
const SUBMITTED: u8 = 2;
const BUSY: u8 = 3;
const RESULT: u8 = 4;
const REFUSED: u8 = 5;
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
    completion: Mutex<Option<Result<Response, Error>>>,
    finished: AtomicBool,
    consumed: AtomicBool,
    // The actual result and all delivery bookkeeping die before this lease owner.
    source: Source,
}
impl Shared {
    fn current(&self) -> Result<(), Error> {
        check(self.source.deadline, &self.source.cancel).map_err(Error::Transport)?;
        let tight = *self
            .tight_deadline
            .try_lock()
            .map_err(|_| Error::Store(faction_files::private_blobs::Error::Busy))?;
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
    fn finish(&self, result: Result<Response, Error>) -> Result<(), Error> {
        let outcome = result.as_ref().map(|_| ()).map_err(|e| *e);
        *self
            .completion
            .try_lock()
            .map_err(|_| Error::Store(faction_files::private_blobs::Error::Busy))? = Some(result);
        self.finished.store(true, Ordering::SeqCst);
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
impl Delivery {
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
                completion: Mutex::new(None),
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
    ) -> Result<Option<Vec<u8>>, Error> {
        // No authority is inferred from routing data or decoded principals.
        if peer != self.shared.source.peer || !Weak::ptr_eq(state, &self.shared.source.state) {
            return Err(Error::Transport(Stage::Denied));
        }
        let message = Message::decode(bytes)?;
        if message.kind != COLLECT || message.correlation != self.shared.correlation {
            return Err(Error::Transport(Stage::Denied));
        }
        if message.purpose != self.shared.source.purpose()
            || message.binding != self.shared.source.binding
            || message.request != self.shared.source.request
            || self.shared.consumed.load(Ordering::SeqCst)
        {
            return Err(Error::Transport(Stage::Denied));
        }
        if !self.shared.finished.load(Ordering::SeqCst) {
            return Ok(None);
        }
        let mut slot = self
            .shared
            .completion
            .try_lock()
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
        if ![SUBMIT, COLLECT].contains(&kind) || ![1, 2, 3].contains(&purpose) {
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
            let reply = delivery.collect(&bytes, peer, &self.state)?;
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
            let reply = delivery.collect(&bytes, peer, &self.state)?;
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
        }
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
        let socket =
            super::tcp_connect::connect(&self.address, handshake, &admission.cancel, || probe())?;
        socket.set_nodelay(true).map_err(|_| Stage::Configuration)?;
        admission.connect(socket)
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
}
pub enum Collection {
    Busy,
    Complete(Response),
}
impl Operation {
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
    fn wait_for_collect(&self) -> Result<(), Stage> {
        if self.attempted >= MAX_OPERATIONS {
            return Err(Stage::Limit);
        }
        loop {
            self.current()?;
            let remaining = self.next_collect.map_or(Duration::ZERO, |when| {
                when.saturating_duration_since(Instant::now())
            });
            if remaining.is_zero() {
                return Ok(());
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
        self.wait_for_collect()?;
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
        let mut client = self.endpoint.connect(
            self.deadline,
            self.cancel.clone(),
            self.current_probe.clone(),
        )?;
        write_frame(&mut client.stream, &bytes)?;
        let reply = read_frame(&mut client.stream)?.ok_or(Stage::Frame)?;
        if reply.len() != 39 || &reply[..6] != MAGIC || reply[6] != SUBMITTED {
            return Err(Stage::Frame);
        }
        message.correlation = reply[7..].try_into().map_err(|_| Stage::Frame)?;
        message.kind = COLLECT;
        message.token = None;
        check(self.deadline, &self.cancel)?;
        drop(client);
        self.ticket = Some(message);
        self.schedule_collect(roundtrip.elapsed(), false);
        Ok(())
    }
    pub fn collect(&mut self) -> Result<Collection, Stage> {
        self.charge()?;
        let bytes = self
            .ticket
            .as_ref()
            .ok_or(Stage::Denied)?
            .encode()
            .map_err(|_| Stage::Frame)?;
        let roundtrip = Instant::now();
        let mut client = self.endpoint.connect(
            self.deadline,
            self.cancel.clone(),
            self.current_probe.clone(),
        )?;
        write_frame(&mut client.stream, &bytes)?;
        let reply = read_frame(&mut client.stream)?.ok_or(Stage::Frame)?;
        check(self.deadline, &self.cancel)?;
        drop(client);
        if reply.len() < 7 || &reply[..6] != MAGIC {
            return Err(Stage::Frame);
        }
        match reply[6] {
            BUSY if reply.len() == 7 => {
                self.schedule_collect(roundtrip.elapsed(), true);
                Ok(Collection::Busy)
            }
            RESULT => {
                let response = decode_response(&reply[7..]).map_err(|_| Stage::Frame)?;
                self.ticket = None;
                self.next_collect = None;
                Ok(Collection::Complete(response))
            }
            REFUSED if reply.len() == 7 => {
                self.ticket = None;
                self.next_collect = None;
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
        })
    }
    fn current(&self) -> Result<(), Error> {
        let state = self
            .state
            .upgrade()
            .ok_or(Error::Transport(Stage::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(Stage::Denied));
        }
        let authority = state
            .authority
            .try_lock()
            .map_err(|_| Error::Store(faction_files::private_blobs::Error::Busy))?;
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
            };
            operation
                .submit_retrieval(
                    self.shared.source.binding.clone(),
                    self.shared.source.request.clone(),
                    token,
                )
                .map_err(Error::Transport)?;
            for _ in 0..MAX_OPERATIONS {
                operation.wait_for_collect().map_err(Error::Transport)?;
                self.shared.charge()?;
                match operation.collect().map_err(Error::Transport)? {
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
}
impl Incoming {
    pub(crate) fn device(session: Session) -> Self {
        Self {
            kind: IncomingKind::Device(session),
        }
    }
    pub(crate) fn service(session: ServiceSession) -> Self {
        Self {
            kind: IncomingKind::Service(session),
        }
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
                let purpose = state
                    .authority
                    .try_lock()
                    .map_err(|_| Error::Store(faction_files::private_blobs::Error::Busy))?
                    .current_service_use(&session.credential, &session.leaf, wall_now())
                    .map_err(Error::Store)?;
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
        if message.kind == COLLECT {
            let delivery = pending.ok_or(Error::Transport(Stage::Denied))?;
            if !charged {
                delivery.begin_collect(&peer, &self.state)?;
            }
            let guard = delivery.shared.clone();
            let reply = delivery.collect(&bytes, peer, &self.state)?;
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
        if message.kind == COLLECT {
            let delivery = pending.ok_or(Error::Transport(Stage::Denied))?;
            if !charged {
                delivery.begin_collect(&peer, &self.state)?;
            }
            let guard = delivery.shared.clone();
            let reply = delivery.collect(&bytes, peer, &self.state)?;
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
