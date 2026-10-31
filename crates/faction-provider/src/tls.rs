//! Explicit local TLS1.3 proof adapter, never a provider or native-mail entry.
//!
//! Standard OpenSSL verifies key possession and CA/name trust. Exact leaf pins
//! and current leaf validity are independent checks. Policy mappings prove no
//! OMEMO private identity possession. Blocking filesystem dispatch is cooperative:
//! deadlines are checked before and after it, but cannot preempt a filesystem call.
use crate::{
    BinaryService, Error, Runtime, State,
    authority::{Connection, Handle},
};
use faction_files::private_blobs::{
    MAX_WIRE, Request, Response, Store, decode_request, decode_response, encode_request,
    encode_response,
};
use openssl::{
    asn1::Asn1Time,
    pkey::{PKey, Private},
    sha::sha256,
    ssl::{
        SslAcceptor, SslConnector, SslMethod, SslOptions, SslSessionCacheMode, SslStream,
        SslVerifyMode, SslVersion,
    },
    x509::{X509, X509VerifyResult},
};
use std::{
    io::{self, Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc, Weak,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub const HANDSHAKE_LIMIT: Duration = Duration::from_secs(5);
pub const CYCLE_LIMIT: Duration = Duration::from_secs(10);
pub const MAX_OPERATIONS: usize = 8;
const CANCEL_SLICE: Duration = Duration::from_millis(100);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    Configuration,
    Admission,
    Handshake,
    Pin,
    Denied,
    Frame,
    Limit,
    Deadline,
    Cancelled,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Report {
    pub stage: Stage,
    pub dispatched: usize,
}
pub(crate) fn wall_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |t| t.as_secs())
}
fn check(deadline: Instant, cancel: &AtomicBool) -> Result<(), Stage> {
    if cancel.load(Ordering::Relaxed) {
        Err(Stage::Cancelled)
    } else if Instant::now() >= deadline {
        Err(Stage::Deadline)
    } else {
        Ok(())
    }
}
fn classify(deadline: Instant, cancel: &AtomicBool, fallback: Stage) -> Stage {
    check(deadline, cancel).err().unwrap_or(fallback)
}
pub(crate) fn leaf_current(leaf: &X509, now: u64) -> bool {
    let Ok(now) = i64::try_from(now)
        .ok()
        .and_then(|n| Asn1Time::from_unix(n).ok())
        .ok_or(())
    else {
        return false;
    };
    leaf.not_before().compare(&now).is_ok_and(|o| !o.is_gt())
        && leaf.not_after().compare(&now).is_ok_and(|o| o.is_gt())
}

/// Every underlying BIO syscall recomputes the original absolute deadline.
/// Progress never restarts the timer; both reads and writes check cancellation.
struct DeadlineIo {
    socket: TcpStream,
    deadline: Instant,
    cancel: Arc<AtomicBool>,
    stop: Option<Arc<AtomicBool>>,
    #[cfg(feature = "delegation-wire")]
    policy_probe: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    #[cfg(feature = "delegation-wire")]
    wire_lifetime: Option<WireLifetime>,
    #[cfg(all(test, feature = "delegation-wire"))]
    read_retries: Option<Arc<std::sync::atomic::AtomicUsize>>,
    #[cfg(all(test, feature = "delegation-wire"))]
    response_write_gate: Option<Arc<ResponseWriteGate>>,
    #[cfg(all(test, feature = "delegation-wire"))]
    response_phase: bool,
    #[cfg(test)]
    write_observation: Option<Arc<socket_bio_tests::WriteRetryObservation>>,
}
impl DeadlineIo {
    fn new(socket: TcpStream, deadline: Instant, cancel: Arc<AtomicBool>) -> Self {
        Self {
            socket,
            deadline,
            cancel,
            stop: None,
            #[cfg(feature = "delegation-wire")]
            policy_probe: None,
            #[cfg(feature = "delegation-wire")]
            wire_lifetime: None,
            #[cfg(all(test, feature = "delegation-wire"))]
            read_retries: None,
            #[cfg(all(test, feature = "delegation-wire"))]
            response_write_gate: None,
            #[cfg(all(test, feature = "delegation-wire"))]
            response_phase: false,
            #[cfg(test)]
            write_observation: None,
        }
    }
    #[cfg(test)]
    fn observe_write_retries(&mut self, observed: Arc<socket_bio_tests::WriteRetryObservation>) {
        self.write_observation = Some(observed);
    }
    fn timeout(&mut self) -> io::Result<Duration> {
        #[cfg(feature = "delegation-wire")]
        if self.policy_probe.as_ref().is_some_and(|p| !p()) {
            return Err(io::Error::other("current HTTP policy unavailable"));
        }
        if self.stop.as_ref().is_some_and(|s| s.load(Ordering::SeqCst)) {
            return Err(io::Error::other("provider stopped"));
        }
        check(self.deadline, &self.cancel)
            .map_err(|stage| io::Error::other(format!("TLS {stage:?}")))?;
        #[cfg(feature = "delegation-wire")]
        if let Some(lifetime) = &mut self.wire_lifetime {
            let wall = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| {
                lifetime.invalid = true;
                io::Error::other("wire clock unavailable")
            })?;
            lifetime.check(wall)?;
        }
        Ok(self
            .deadline
            .saturating_duration_since(Instant::now())
            .min(CANCEL_SLICE)
            .max(Duration::from_micros(1)))
    }
}
impl Read for DeadlineIo {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        loop {
            let timeout = self.timeout()?;
            self.socket.set_read_timeout(Some(timeout))?;
            match self.socket.read(bytes) {
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::Interrupted
                            | io::ErrorKind::WouldBlock
                            | io::ErrorKind::TimedOut
                    ) =>
                {
                    #[cfg(all(test, feature = "delegation-wire"))]
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) && let Some(retries) = &self.read_retries
                    {
                        retries.fetch_add(1, Ordering::SeqCst);
                    }
                    continue;
                }
                result => return result,
            }
        }
    }
}
impl Write for DeadlineIo {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        #[cfg(all(test, feature = "delegation-wire"))]
        if self.response_phase
            && let Some(gate) = self.response_write_gate.take()
        {
            gate.enter(self.deadline)?;
        }
        loop {
            let timeout = self.timeout()?;
            self.socket.set_write_timeout(Some(timeout))?;
            match self.socket.write(bytes) {
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::Interrupted
                            | io::ErrorKind::WouldBlock
                            | io::ErrorKind::TimedOut
                    ) =>
                {
                    #[cfg(test)]
                    if let Some(observed) = &self.write_observation {
                        match e.kind() {
                            io::ErrorKind::WouldBlock => {
                                observed.would_block.fetch_add(1, Ordering::SeqCst);
                            }
                            io::ErrorKind::TimedOut => {
                                observed.timed_out.fetch_add(1, Ordering::SeqCst);
                            }
                            _ => (),
                        }
                        if matches!(
                            e.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                        ) {
                            observed
                                .retry_call
                                .store(observed.in_flight.load(Ordering::SeqCst), Ordering::SeqCst);
                        }
                    }
                    continue;
                }
                result => return result,
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.timeout()?;
        Ok(())
    }
}
impl Drop for DeadlineIo {
    fn drop(&mut self) {
        let _ = self.socket.shutdown(Shutdown::Both);
    }
}

fn read_frame(stream: &mut SslStream<DeadlineIo>) -> Result<Option<Vec<u8>>, Stage> {
    let mut length = [0; 4];
    let n = stream.read(&mut length[..1]).map_err(|_| Stage::Frame)?;
    if n == 0 {
        return Ok(None);
    }
    stream
        .read_exact(&mut length[1..])
        .map_err(|_| Stage::Frame)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > MAX_WIRE {
        return Err(Stage::Frame);
    }
    let mut bytes = vec![0; length];
    stream.read_exact(&mut bytes).map_err(|_| Stage::Frame)?;
    Ok(Some(bytes))
}
fn write_frame<W: Write>(stream: &mut W, bytes: &[u8]) -> Result<(), Stage> {
    if bytes.is_empty() || bytes.len() > MAX_WIRE {
        return Err(Stage::Frame);
    }
    let mut frame = Vec::with_capacity(4 + bytes.len());
    frame.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    frame.extend_from_slice(bytes);
    stream.write_all(&frame).map_err(|_| Stage::Frame)?;
    stream.flush().map_err(|_| Stage::Frame)
}

/// Trusted preloaded TLS configuration. Accepts only caller-owned loopback listeners.
/// Creates no socket, service, thread, enrollment or background task automatically.
pub struct Server {
    acceptor: SslAcceptor,
}
struct AdmissionGuard {
    state: Arc<State>,
    transferred: bool,
}
impl Drop for AdmissionGuard {
    fn drop(&mut self) {
        if !self.transferred {
            if let Ok(mut s) = self.state.socket.lock() {
                s.take();
            }
            self.state.active.store(false, Ordering::SeqCst);
        }
    }
}
impl Server {
    /// Bounded preloaded PEM only. Does not open files or start a listener.
    pub fn from_pem(certificate: &[u8], key: &[u8], client_ca: &[u8]) -> Result<Self, Stage> {
        let credentials = ClientCredentials::from_pem(client_ca, certificate, key)?;
        Self::new(&credentials.certificate, &credentials.key, &credentials.ca)
    }
    pub fn new(certificate: &X509, key: &PKey<Private>, ca: &X509) -> Result<Self, Stage> {
        let mut b = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls_server())
            .map_err(|_| Stage::Configuration)?;
        b.set_min_proto_version(Some(SslVersion::TLS1_3))
            .map_err(|_| Stage::Configuration)?;
        b.set_max_proto_version(Some(SslVersion::TLS1_3))
            .map_err(|_| Stage::Configuration)?;
        b.set_certificate(certificate)
            .map_err(|_| Stage::Configuration)?;
        b.set_private_key(key).map_err(|_| Stage::Configuration)?;
        b.check_private_key().map_err(|_| Stage::Configuration)?;
        let mut store =
            openssl::x509::store::X509StoreBuilder::new().map_err(|_| Stage::Configuration)?;
        store
            .add_cert(ca.clone())
            .map_err(|_| Stage::Configuration)?;
        b.set_cert_store(store.build());
        b.set_verify(SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT);
        b.set_session_cache_mode(SslSessionCacheMode::OFF);
        b.set_options(SslOptions::NO_TICKET);
        b.set_num_tickets(0).map_err(|_| Stage::Configuration)?;
        b.set_max_early_data(0).map_err(|_| Stage::Configuration)?;
        Ok(Self {
            acceptor: b.build(),
        })
    }
    /// Five-second admission includes TCP wait and TLS. One accepted cycle lasts
    /// ten seconds from TCP acceptance; a second active session fails closed.
    pub fn accept(
        &self,
        listener: &TcpListener,
        runtime: &Runtime,
        cancel: Arc<AtomicBool>,
    ) -> Result<Session, Stage> {
        let admission = Instant::now() + HANDSHAKE_LIMIT;
        let state = runtime.state.clone();
        if state.stopped.load(Ordering::SeqCst) || state.active.swap(true, Ordering::SeqCst) {
            return Err(Stage::Denied);
        }
        let mut guard = AdmissionGuard {
            state: state.clone(),
            transferred: false,
        };
        if !listener.local_addr().is_ok_and(|a| a.ip().is_loopback())
            || listener.set_nonblocking(true).is_err()
        {
            return Err(Stage::Configuration);
        }
        let socket = loop {
            check(admission, &cancel)?;
            if state.stopped.load(Ordering::SeqCst) {
                return Err(Stage::Denied);
            }
            match listener.accept() {
                Ok((socket, peer)) => {
                    if !peer.ip().is_loopback() {
                        return Err(Stage::Admission);
                    }
                    break socket;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5))
                }
                Err(_) => return Err(Stage::Admission),
            }
        };
        let start = Instant::now();
        let deadline = start + CYCLE_LIMIT;
        let handshake = admission.min(deadline);
        *state.socket.lock().map_err(|_| Stage::Denied)? =
            Some(socket.try_clone().map_err(|_| Stage::Admission)?);
        let mut io = DeadlineIo::new(socket, handshake, cancel.clone());
        io.stop = Some(state.stopped.clone());
        let mut stream = self
            .acceptor
            .accept(io)
            .map_err(|_| classify(handshake, &cancel, Stage::Handshake))?;
        check(handshake, &cancel)?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Stage::Denied);
        }
        let leaf = stream.ssl().peer_certificate().ok_or(Stage::Handshake)?;
        if stream.ssl().verify_result() != X509VerifyResult::OK || stream.ssl().session_reused() {
            return Err(Stage::Handshake);
        }
        let now = wall_now();
        if !leaf_current(&leaf, now) {
            return Err(Stage::Denied);
        }
        let digest = sha256(&leaf.to_der().map_err(|_| Stage::Handshake)?);
        let (connection, handle) = {
            let mut authority = state.authority.lock().map_err(|_| Stage::Denied)?;
            if state.stopped.load(Ordering::SeqCst) {
                return Err(Stage::Denied);
            }
            let connection = authority.open_connection();
            let handle = authority
                .authenticate(&connection, digest, now, start.elapsed().as_millis() as u64)
                .map_err(|_| Stage::Denied)?;
            (connection, handle)
        };
        check(handshake, &cancel)?;
        stream.get_mut().deadline = deadline;
        guard.transferred = true;
        Ok(Session {
            stream,
            leaf,
            connection,
            handle,
            state: Arc::downgrade(&state),
            start,
            deadline,
            cancel,
            attempted: 0,
            closed: false,
            #[cfg(all(test, feature = "delegation-verifier"))]
            publication_capture: None,
        })
    }
    /// Authenticate one registered service through the same admission lane and
    /// original deadlines as device admission; never create a device handle.
    #[cfg(feature = "delegation-services")]
    pub fn accept_service(
        &self,
        listener: &TcpListener,
        runtime: &Runtime,
        cancel: Arc<AtomicBool>,
    ) -> Result<ServiceSession, Stage> {
        let admission = Instant::now() + HANDSHAKE_LIMIT;
        let state = runtime.state.clone();
        if state.stopped.load(Ordering::SeqCst) || state.active.swap(true, Ordering::SeqCst) {
            return Err(Stage::Denied);
        }
        let mut guard = AdmissionGuard {
            state: state.clone(),
            transferred: false,
        };
        if !listener.local_addr().is_ok_and(|a| a.ip().is_loopback())
            || listener.set_nonblocking(true).is_err()
        {
            return Err(Stage::Configuration);
        }
        let socket = loop {
            check(admission, &cancel)?;
            if state.stopped.load(Ordering::SeqCst) {
                return Err(Stage::Denied);
            }
            match listener.accept() {
                Ok((socket, peer)) => {
                    if !peer.ip().is_loopback() {
                        return Err(Stage::Admission);
                    }
                    break socket;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5))
                }
                Err(_) => return Err(Stage::Admission),
            }
        };
        let start = Instant::now();
        let deadline = start + CYCLE_LIMIT;
        let handshake = admission.min(deadline);
        *state.socket.lock().map_err(|_| Stage::Denied)? =
            Some(socket.try_clone().map_err(|_| Stage::Admission)?);
        let mut io = DeadlineIo::new(socket, handshake, cancel.clone());
        io.stop = Some(state.stopped.clone());
        let mut stream = self
            .acceptor
            .accept(io)
            .map_err(|_| classify(handshake, &cancel, Stage::Handshake))?;
        check(handshake, &cancel)?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Stage::Denied);
        }
        let leaf = stream.ssl().peer_certificate().ok_or(Stage::Handshake)?;
        if stream.ssl().verify_result() != X509VerifyResult::OK || stream.ssl().session_reused() {
            return Err(Stage::Handshake);
        }
        let now = wall_now();
        if !leaf_current(&leaf, now) {
            return Err(Stage::Denied);
        }
        let credential = {
            let authority = state.authority.lock().map_err(|_| Stage::Denied)?;
            if state.stopped.load(Ordering::SeqCst) {
                return Err(Stage::Denied);
            }
            authority
                .authenticate_service(&leaf, now)
                .map_err(|_| Stage::Denied)?
        };
        check(handshake, &cancel)?;
        stream.get_mut().deadline = deadline;
        guard.transferred = true;
        Ok(ServiceSession {
            stream,
            leaf,
            credential,
            state: Arc::downgrade(&state),
            start,
            deadline,
            cancel,
            attempted: 0,
            closed: false,
        })
    }
}
/// Opaque owning proof of one real verified mTLS peer. No identity or stream escape.
/// All application calls charge the same budget, including malformed or denied calls.
/// ```compile_fail
/// use faction_provider::Session;
/// let forged = Session {};
/// ```
/// ```compile_fail
/// use faction_provider::Session;
/// fn escape(s: Session) { let raw = s.stream; }
/// ```
/// ```compile_fail
/// use faction_provider::authority::Authority;
/// let forged = Authority::new(None);
/// ```
/// ```compile_fail
/// use faction_provider::Session;
/// fn escape(s: Session) { let handle = s.handle; }
/// ```
pub struct Session {
    stream: SslStream<DeadlineIo>,
    leaf: X509,
    connection: Connection,
    handle: Handle,
    state: Weak<State>,
    start: Instant,
    deadline: Instant,
    cancel: Arc<AtomicBool>,
    attempted: usize,
    closed: bool,
    #[cfg(all(test, feature = "delegation-verifier"))]
    publication_capture: Option<std::sync::mpsc::Sender<Instant>>,
}
enum DispatchRoute<'a> {
    Account(&'a str),
    Local(&'a BinaryService),
}
impl Session {
    /// Create bounded recipient intent from this current verified session. This
    /// charges the existing attempt budget before validating caller-selected data.
    #[cfg(feature = "delegation-records")]
    pub fn retrieval_intent(
        &mut self,
        binding: faction_files::private_blobs::Binding,
        action: crate::retrieval::RetrievalAction,
    ) -> Result<crate::retrieval::RetrievalRecord, Error> {
        let state = self.attempt()?;
        let mut authority = state
            .authority
            .lock()
            .map_err(|_| Error::Transport(Stage::Denied))?;
        check(self.deadline, &self.cancel).map_err(Error::Transport)?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(Stage::Denied));
        }
        let token = authority
            .create_retrieval(
                crate::retrieval::Source {
                    connection: &self.connection,
                    handle: &self.handle,
                    leaf: &self.leaf,
                    deadline: self.deadline,
                    elapsed_ms: self.start.elapsed().as_millis() as u64,
                },
                binding,
                action,
                crate::retrieval::Moment::now(),
            )
            .map_err(Error::Store)?;
        Ok(crate::retrieval::RetrievalRecord::new(&state, token))
    }
    pub(crate) fn attempt(&mut self) -> Result<Arc<State>, Error> {
        if self.attempted >= MAX_OPERATIONS {
            return Err(Error::Transport(Stage::Limit));
        }
        self.attempted += 1;
        check(self.deadline, &self.cancel).map_err(Error::Transport)?;
        if self.closed {
            return Err(Error::Transport(Stage::Denied));
        }
        let state = self
            .state
            .upgrade()
            .ok_or(Error::Transport(Stage::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(Stage::Denied));
        }
        if !leaf_current(&self.leaf, wall_now()) {
            self.closed = true;
            return Err(Error::Transport(Stage::Denied));
        }
        Ok(state)
    }
    pub(crate) fn dispatch_charged(
        &mut self,
        state: &State,
        store: &mut Store,
        route: &str,
        request: Request,
    ) -> Result<Response, Error> {
        self.dispatch_authorized(state, store, DispatchRoute::Account(route), request)
    }
    fn dispatch_authorized(
        &mut self,
        state: &State,
        store: &mut Store,
        route: DispatchRoute<'_>,
        request: Request,
    ) -> Result<Response, Error> {
        check(self.deadline, &self.cancel).map_err(Error::Transport)?;
        let authority = state
            .authority
            .lock()
            .map_err(|_| Error::Transport(Stage::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(Stage::Denied));
        }
        if !leaf_current(&self.leaf, wall_now()) {
            self.closed = true;
            return Err(Error::Transport(Stage::Denied));
        }
        let times = (wall_now(), self.start.elapsed().as_millis() as u64);
        let result = match route {
            DispatchRoute::Account(account) => authority.dispatch_routed(
                &self.connection,
                &self.handle,
                account,
                store,
                request,
                times,
            ),
            DispatchRoute::Local(service) => authority.dispatch_local(
                &self.connection,
                &self.handle,
                &service.home,
                store,
                request,
                times,
            ),
        }
        .map_err(Error::Store);
        check(self.deadline, &self.cancel).map_err(Error::Transport)?;
        result
    }
    /// Read one existing length-prefixed binary request from this exact TLS peer,
    /// dispatch once and write its bounded response. EOF returns false. Every
    /// call charges before I/O/decode; any failure closes this application stream.
    pub fn serve_next(&mut self, store: &mut Store, route_account: &str) -> Result<bool, Error> {
        self.serve_with(store, DispatchRoute::Account(route_account))
    }
    /// Serve the existing binary carrier with the actor resolved internally from
    /// current certificate authority, restricted to the trusted configured home.
    pub fn serve_service_next(
        &mut self,
        store: &mut Store,
        service: &BinaryService,
    ) -> Result<bool, Error> {
        self.serve_with(store, DispatchRoute::Local(service))
    }
    fn serve_with(&mut self, store: &mut Store, route: DispatchRoute<'_>) -> Result<bool, Error> {
        let result = (|| {
            let state = self.attempt()?;
            let bytes = read_frame(&mut self.stream)
                .map_err(|stage| Error::Transport(classify(self.deadline, &self.cancel, stage)))?;
            let Some(bytes) = bytes else { return Ok(false) };
            let request = decode_request(&bytes).map_err(Error::Store)?;
            let response = self.dispatch_authorized(&state, store, route, request)?;
            let bytes = encode_response(&response).map_err(Error::Store)?;
            write_frame(&mut self.stream, &bytes)
                .map_err(|stage| Error::Transport(classify(self.deadline, &self.cancel, stage)))?;
            check(self.deadline, &self.cancel).map_err(Error::Transport)?;
            Ok(true)
        })();
        if result.is_err() {
            self.closed = true;
            let _ = self.stream.get_mut().socket.shutdown(Shutdown::Both);
        }
        result
    }
    pub fn dispatch(
        &mut self,
        store: &mut Store,
        route_account: &str,
        request: Request,
    ) -> Result<Response, Error> {
        let state = self.attempt()?;
        encode_request(&request).map_err(Error::Store)?;
        self.dispatch_charged(&state, store, route_account, request)
    }
    /// No account parameter exists: only current opaque session authority owns
    /// the actor. Request principals remain assertions checked by current policy.
    pub fn dispatch_service(
        &mut self,
        store: &mut Store,
        service: &BinaryService,
        request: Request,
    ) -> Result<Response, Error> {
        let state = self.attempt()?;
        encode_request(&request).map_err(Error::Store)?;
        self.dispatch_authorized(&state, store, DispatchRoute::Local(service), request)
    }
    pub fn dispatch_bytes(
        &mut self,
        store: &mut Store,
        route_account: &str,
        bytes: &[u8],
    ) -> Result<Response, Error> {
        let state = self.attempt()?;
        if bytes.len() > MAX_WIRE {
            return Err(Error::Transport(Stage::Frame));
        }
        let request = decode_request(bytes).map_err(Error::Store)?;
        self.dispatch_charged(&state, store, route_account, request)
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.stream.get_mut().socket.shutdown(Shutdown::Both);
        if let Some(state) = self.state.upgrade() {
            if let Ok(mut authority) = state.authority.lock() {
                authority.close_connection(&self.connection);
            }
            if let Ok(mut socket) = state.socket.lock() {
                socket.take();
            }
            state.active.store(false, Ordering::SeqCst);
        }
    }
}

/// Opaque verified home service. Its only application operation observes policy.
/// ```compile_fail
/// use faction_provider::ServiceSession;
/// let forged = ServiceSession {};
/// ```
/// ```compile_fail
/// use faction_provider::ServiceSession;
/// fn escape(s: ServiceSession) { let stream = s.stream; }
/// ```
/// ```compile_fail
/// use faction_provider::ServiceSession;
/// fn escape(s: ServiceSession) { let handle = s.handle; }
/// ```
/// ```compile_fail
/// use faction_provider::{ServiceSession, retrieval::RetrievalAction};
/// fn mint(s: &mut ServiceSession, b: faction_files::private_blobs::Binding) {
///     s.retrieval_intent(b, RetrievalAction::Status);
/// }
/// ```
#[cfg(feature = "delegation-services")]
pub struct ServiceSession {
    stream: SslStream<DeadlineIo>,
    pub(crate) leaf: X509,
    pub(crate) credential: crate::services::ServiceCredential,
    state: Weak<State>,
    start: Instant,
    pub(crate) deadline: Instant,
    pub(crate) cancel: Arc<AtomicBool>,
    attempted: usize,
    closed: bool,
}
#[cfg(feature = "delegation-services")]
impl ServiceSession {
    #[cfg(feature = "delegation-verifier")]
    pub(crate) fn original_service_attempts(&self) -> usize {
        self.attempted
    }
    #[cfg(feature = "delegation-issuer")]
    pub(crate) fn issuer_attempt(&mut self) -> Result<Arc<State>, Error> {
        if self.attempted >= MAX_OPERATIONS {
            return Err(Error::Transport(Stage::Limit));
        }
        self.attempted += 1;
        let state = self
            .state
            .upgrade()
            .ok_or(Error::Transport(Stage::Denied))?;
        self.issuer_check(&state)?;
        Ok(state)
    }
    #[cfg(feature = "delegation-issuer")]
    pub(crate) fn issuer_check(&mut self, state: &State) -> Result<(), Error> {
        if let Err(stage) = check(self.deadline, &self.cancel) {
            self.closed = true;
            return Err(Error::Transport(stage));
        }
        if self.closed
            || state.stopped.load(Ordering::SeqCst)
            || !leaf_current(&self.leaf, wall_now())
        {
            self.closed = true;
            return Err(Error::Transport(Stage::Denied));
        }
        Ok(())
    }

    /// Charge first, then recheck the exact registration and recipient. No I/O,
    /// callback, device authority, or store dispatch is performed under this lock.
    pub fn observe_recipient(
        &mut self,
        binding: faction_files::private_blobs::Binding,
        action: crate::services::PublicationAction,
    ) -> Result<crate::services::RecipientCurrentObservation, Error> {
        if self.attempted >= MAX_OPERATIONS {
            return Err(Error::Transport(Stage::Limit));
        }
        self.attempted += 1;
        if self.closed {
            return Err(Error::Transport(Stage::Denied));
        }
        if let Err(stage) = check(self.deadline, &self.cancel) {
            self.closed = true;
            return Err(Error::Transport(stage));
        }
        let state = self
            .state
            .upgrade()
            .ok_or(Error::Transport(Stage::Denied))?;
        let authority = state
            .authority
            .lock()
            .map_err(|_| Error::Transport(Stage::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(Stage::Denied));
        }
        let result = authority
            .observe_recipient(
                &mut self.credential,
                &self.leaf,
                binding,
                action,
                wall_now(),
            )
            .map_err(|_| Error::Transport(Stage::Denied));
        if self.start.elapsed() >= CYCLE_LIMIT {
            self.closed = true;
            return Err(Error::Transport(Stage::Deadline));
        }
        check(self.deadline, &self.cancel).map_err(Error::Transport)?;
        result
    }
}
#[cfg(feature = "delegation-services")]
impl Drop for ServiceSession {
    fn drop(&mut self) {
        let _ = self.stream.get_mut().socket.shutdown(Shutdown::Both);
        if let Some(state) = self.state.upgrade() {
            if let Ok(mut socket) = state.socket.lock() {
                socket.take();
            }
            state.active.store(false, Ordering::SeqCst);
        }
    }
}

/// Verified loopback TLS counterpart using only the existing bounded binary carrier.
pub struct Client {
    stream: SslStream<DeadlineIo>,
    deadline: Instant,
    cancel: Arc<AtomicBool>,
    attempted: usize,
}
impl Client {
    pub fn connect(
        address: SocketAddr,
        expected_name: &str,
        leaf_pin: [u8; 32],
        ca: &X509,
        identity: Option<(&X509, Option<&PKey<Private>>)>,
        cancel: Arc<AtomicBool>,
    ) -> Result<Self, Stage> {
        if !address.ip().is_loopback() {
            return Err(Stage::Configuration);
        }
        let start = Instant::now();
        let deadline = start + CYCLE_LIMIT;
        let handshake = start + HANDSHAKE_LIMIT;
        check(handshake, &cancel)?;
        let mut b =
            SslConnector::builder(SslMethod::tls_client()).map_err(|_| Stage::Configuration)?;
        b.set_min_proto_version(Some(SslVersion::TLS1_3))
            .map_err(|_| Stage::Configuration)?;
        b.set_max_proto_version(Some(SslVersion::TLS1_3))
            .map_err(|_| Stage::Configuration)?;
        let mut store =
            openssl::x509::store::X509StoreBuilder::new().map_err(|_| Stage::Configuration)?;
        store
            .add_cert(ca.clone())
            .map_err(|_| Stage::Configuration)?;
        b.set_cert_store(store.build());
        b.set_verify(SslVerifyMode::PEER);
        b.set_session_cache_mode(SslSessionCacheMode::OFF);
        b.set_options(SslOptions::NO_TICKET);
        b.set_max_early_data(0).map_err(|_| Stage::Configuration)?;
        if let Some((certificate, key)) = identity {
            let key = key.ok_or(Stage::Configuration)?;
            b.set_certificate(certificate)
                .map_err(|_| Stage::Configuration)?;
            b.set_private_key(key).map_err(|_| Stage::Configuration)?;
            b.check_private_key().map_err(|_| Stage::Configuration)?;
        }
        let connector = b.build();
        let mut configuration = connector.configure().map_err(|_| Stage::Configuration)?;
        configuration.set_verify_hostname(true);
        check(handshake, &cancel)?;
        let remaining = handshake.saturating_duration_since(Instant::now());
        let socket = TcpStream::connect_timeout(&address, remaining)
            .map_err(|_| classify(handshake, &cancel, Stage::Admission))?;
        let io = DeadlineIo::new(socket, handshake, cancel.clone());
        let mut stream = configuration
            .connect(expected_name, io)
            .map_err(|_| classify(handshake, &cancel, Stage::Handshake))?;
        check(handshake, &cancel)?;
        if stream.ssl().verify_result() != X509VerifyResult::OK || stream.ssl().session_reused() {
            return Err(Stage::Handshake);
        }
        let leaf = stream.ssl().peer_certificate().ok_or(Stage::Handshake)?;
        if sha256(&leaf.to_der().map_err(|_| Stage::Handshake)?) != leaf_pin {
            return Err(Stage::Pin);
        }
        check(handshake, &cancel)?;
        stream.get_mut().deadline = deadline;
        check(deadline, &cancel)?;
        Ok(Self {
            stream,
            deadline,
            cancel,
            attempted: 0,
        })
    }
    pub fn request(&mut self, request: &Request) -> Result<Response, Stage> {
        check(self.deadline, &self.cancel)?;
        if self.attempted == MAX_OPERATIONS {
            return Err(Stage::Limit);
        }
        self.attempted += 1;
        let bytes = encode_request(request).map_err(|_| Stage::Frame)?;
        write_frame(&mut self.stream, &bytes)
            .map_err(|s| classify(self.deadline, &self.cancel, s))?;
        let bytes = read_frame(&mut self.stream)
            .map_err(|s| classify(self.deadline, &self.cancel, s))?
            .ok_or(Stage::Frame)?;
        check(self.deadline, &self.cancel)?;
        decode_response(&bytes).map_err(|_| Stage::Frame)
    }
}

/// Independently preloaded transport credentials. They contain no mail/vault keys.
/// Each PEM input is bounded to 64 KiB. No file loader or enrollment is provided.
#[derive(Clone)]
pub struct ClientCredentials {
    ca: X509,
    certificate: X509,
    key: PKey<Private>,
}
impl ClientCredentials {
    pub fn from_pem(ca: &[u8], certificate: &[u8], key: &[u8]) -> Result<Self, Stage> {
        if [ca, certificate, key]
            .iter()
            .any(|bytes| bytes.is_empty() || bytes.len() > 65_536)
        {
            return Err(Stage::Configuration);
        }
        let ca = X509::from_pem(ca).map_err(|_| Stage::Configuration)?;
        let certificate = X509::from_pem(certificate).map_err(|_| Stage::Configuration)?;
        // Encrypted inputs cannot start an interactive password prompt.
        let key =
            PKey::private_key_from_pem_callback(key, |_| Err(openssl::error::ErrorStack::get()))
                .map_err(|_| Stage::Configuration)?;
        if !certificate
            .public_key()
            .map_err(|_| Stage::Configuration)?
            .public_eq(&key)
        {
            return Err(Stage::Configuration);
        }
        Ok(Self {
            ca,
            certificate,
            key,
        })
    }
}
/// Single-use opaque admission clock, minted before asynchronous TCP connection.
/// The caller cannot forge a start time, clone it or extend either deadline.
/// ```compile_fail
/// use faction_provider::tls::ClientAdmission;
/// let forged = ClientAdmission {};
/// ```
/// ```compile_fail
/// use faction_provider::tls::ClientAdmission;
/// fn extend(token: ClientAdmission) { let start = token.start; }
/// ```
pub struct ClientAdmission {
    start: Instant,
    expected_name: String,
    leaf_pin: [u8; 32],
    credentials: ClientCredentials,
    cancel: Arc<AtomicBool>,
}
impl ClientAdmission {
    pub fn new(
        expected_name: &str,
        leaf_pin: [u8; 32],
        credentials: ClientCredentials,
        cancel: Arc<AtomicBool>,
    ) -> Result<Self, Stage> {
        let start = Instant::now();
        check(start + HANDSHAKE_LIMIT, &cancel)?;
        faction_files::private_blobs::Principal {
            account: format!("policy@{expected_name}"),
            device: 1,
            identity: [1; 32],
        }
        .validate()
        .map_err(|_| Stage::Configuration)?;
        if leaf_pin == [0; 32] {
            return Err(Stage::Configuration);
        }
        Ok(Self {
            start,
            expected_name: expected_name.into(),
            leaf_pin,
            credentials,
            cancel,
        })
    }
    /// Consume only an already connected loopback socket. TLS still authenticates
    /// its peer; a socket or configuration text supplies no device authority.
    pub fn connect(self, socket: TcpStream) -> Result<Client, Stage> {
        let handshake = self.start + HANDSHAKE_LIMIT;
        let deadline = self.start + CYCLE_LIMIT;
        check(handshake, &self.cancel)?;
        if !socket.peer_addr().is_ok_and(|a| a.ip().is_loopback())
            || !socket.local_addr().is_ok_and(|a| a.ip().is_loopback())
        {
            return Err(Stage::Configuration);
        }
        // Tokio into_std preserves O_NONBLOCK. DeadlineIo requires bounded
        // blocking syscalls, otherwise WouldBlock could spin until its deadline.
        socket
            .set_nonblocking(false)
            .map_err(|_| Stage::Admission)?;
        let mut b =
            SslConnector::builder(SslMethod::tls_client()).map_err(|_| Stage::Configuration)?;
        b.set_min_proto_version(Some(SslVersion::TLS1_3))
            .map_err(|_| Stage::Configuration)?;
        b.set_max_proto_version(Some(SslVersion::TLS1_3))
            .map_err(|_| Stage::Configuration)?;
        let mut store =
            openssl::x509::store::X509StoreBuilder::new().map_err(|_| Stage::Configuration)?;
        store
            .add_cert(self.credentials.ca)
            .map_err(|_| Stage::Configuration)?;
        b.set_cert_store(store.build());
        b.set_verify(SslVerifyMode::PEER);
        b.set_session_cache_mode(SslSessionCacheMode::OFF);
        b.set_options(SslOptions::NO_TICKET);
        b.set_max_early_data(0).map_err(|_| Stage::Configuration)?;
        b.set_certificate(&self.credentials.certificate)
            .map_err(|_| Stage::Configuration)?;
        b.set_private_key(&self.credentials.key)
            .map_err(|_| Stage::Configuration)?;
        b.check_private_key().map_err(|_| Stage::Configuration)?;
        let connector = b.build();
        let mut configuration = connector.configure().map_err(|_| Stage::Configuration)?;
        configuration.set_verify_hostname(true);
        check(handshake, &self.cancel)?;
        let io = DeadlineIo::new(socket, handshake, self.cancel.clone());
        let mut stream = configuration
            .connect(&self.expected_name, io)
            .map_err(|_| classify(handshake, &self.cancel, Stage::Handshake))?;
        check(handshake, &self.cancel)?;
        if stream.ssl().verify_result() != X509VerifyResult::OK || stream.ssl().session_reused() {
            return Err(Stage::Handshake);
        }
        let leaf = stream.ssl().peer_certificate().ok_or(Stage::Handshake)?;
        if sha256(&leaf.to_der().map_err(|_| Stage::Handshake)?) != self.leaf_pin {
            return Err(Stage::Pin);
        }
        check(handshake, &self.cancel)?;
        stream.get_mut().deadline = deadline;
        check(deadline, &self.cancel)?;
        Ok(Client {
            stream,
            deadline,
            cancel: self.cancel,
            attempted: 0,
        })
    }
}

#[cfg(test)]
mod socket_bio_tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    // This is only a finite socket-BIO fixture byte budget. Its numeric equality
    // to the store's reserved-byte ceiling grants no RPC or storage authority.
    const FIXTURE_MAX_BYTES: usize = 8_388_736;

    #[derive(Default)]
    pub(super) struct WriteRetryObservation {
        pub(super) started_calls: AtomicUsize,
        pub(super) in_flight: AtomicUsize,
        pub(super) retry_call: AtomicUsize,
        pub(super) completed_bytes: AtomicUsize,
        pub(super) completed_writes: AtomicUsize,
        pub(super) would_block: AtomicUsize,
        pub(super) timed_out: AtomicUsize,
    }

    #[test]
    fn socket_bio_pending_write_observes_retry_and_cancels_before_deadline() {
        let start = Instant::now();
        let fixture_deadline = start + Duration::from_secs(10);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let socket =
            TcpStream::connect_timeout(&listener.local_addr().unwrap(), HANDSHAKE_LIMIT).unwrap();
        let (peer, _) = listener.accept().unwrap();
        // The one owned peer stays open and never reads. There is no TLS,
        // provider Session, request frame, Store, Worker or mail release here.
        let cancel = Arc::new(AtomicBool::new(false));
        let observed = Arc::new(WriteRetryObservation::default());
        let mut io = DeadlineIo::new(socket, fixture_deadline, cancel.clone());
        io.observe_write_retries(observed.clone());
        let (pending_call, outcome, drain) = thread_scope(&observed, &cancel, start, io);
        drop(peer);
        drop(listener);
        let completed = observed.completed_bytes.load(Ordering::SeqCst);
        let positive_writes = observed.completed_writes.load(Ordering::SeqCst);
        let would_block = observed.would_block.load(Ordering::SeqCst);
        let timed_out = observed.timed_out.load(Ordering::SeqCst);
        eprintln!(
            "socket-BIO-only bytes={completed} positive_writes={positive_writes} would_block={would_block} timed_out={timed_out} pending_call={pending_call:?} drain_ms={} total_ms={} outcome={outcome:?}",
            drain.as_millis(),
            start.elapsed().as_millis()
        );
        assert!(start.elapsed() < Duration::from_secs(10));
        assert!(drain < Duration::from_secs(5));
        assert!(completed <= FIXTURE_MAX_BYTES);
        assert!(
            completed > 0 && positive_writes > 0,
            "no positive socket-write control"
        );
        assert!(
            would_block + timed_out > 0,
            "no actual socket WouldBlock/TimedOut observed"
        );
        assert!(
            pending_call.is_some(),
            "no same socket write remained unfinished before cancellation"
        );
        assert_eq!(outcome.unwrap_err().to_string(), "TLS Cancelled");
    }

    fn thread_scope(
        observed: &Arc<WriteRetryObservation>,
        cancel: &Arc<AtomicBool>,
        start: Instant,
        mut io: DeadlineIo,
    ) -> (Option<usize>, io::Result<usize>, Duration) {
        std::thread::scope(|scope| {
            let writer_observation = observed.clone();
            let writer = scope.spawn(move || {
                let scratch = [0x5a; 8192];
                let mut total = 0;
                while total < FIXTURE_MAX_BYTES {
                    let call = writer_observation
                        .started_calls
                        .fetch_add(1, Ordering::SeqCst)
                        + 1;
                    writer_observation.in_flight.store(call, Ordering::SeqCst);
                    // The last write cannot exceed the fixed fixture ceiling.
                    let remaining = (FIXTURE_MAX_BYTES - total).min(scratch.len());
                    let result = io.write(&scratch[..remaining]);
                    writer_observation.in_flight.store(0, Ordering::SeqCst);
                    match result {
                        Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                        Ok(bytes) => {
                            total += bytes;
                            writer_observation
                                .completed_bytes
                                .store(total, Ordering::SeqCst);
                            writer_observation
                                .completed_writes
                                .fetch_add(1, Ordering::SeqCst);
                        }
                        Err(error) => return Err(error),
                    }
                }
                Ok(total)
            });
            let mut pending = None;
            // Reserve the other half of the absolute ten seconds for cancellation
            // and the unchanged five-second drain. No retry changes this clock.
            while Instant::now() < start + Duration::from_secs(5) {
                let retried = observed.retry_call.load(Ordering::SeqCst);
                if retried != 0
                    && observed.in_flight.load(Ordering::SeqCst) == retried
                    && observed.completed_writes.load(Ordering::SeqCst) > 0
                    && !writer.is_finished()
                {
                    pending = Some(retried);
                    break;
                }
                if writer.is_finished() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            cancel.store(true, Ordering::SeqCst);
            let drain = Instant::now();
            let outcome = writer.join().unwrap();
            (pending, outcome, drain.elapsed())
        })
    }
}

#[cfg(test)]
#[path = "tls_frame_tests.rs"]
mod frame_tests;
#[cfg(test)]
mod admission_cleanup_tests {
    use super::*;
    #[test]
    fn blocked_cleanup_keeps_lane_owned_until_tracked_socket_is_cleared() {
        let r = Runtime::new(None);
        let state = r.state.clone();
        state.active.store(true, Ordering::SeqCst);
        let socket_lock = state.socket.lock().unwrap();
        let (started, ready) = std::sync::mpsc::channel();
        std::thread::scope(|scope| {
            let state2 = state.clone();
            let cleanup = scope.spawn(move || {
                started.send(()).unwrap();
                drop(AdmissionGuard {
                    state: state2,
                    transferred: false,
                });
            });
            ready.recv().unwrap();
            // Wait for the cleanup thread to contend on the held tracking lock.
            std::thread::sleep(Duration::from_millis(30));
            let occupied = state.active.swap(true, Ordering::SeqCst);
            drop(socket_lock);
            cleanup.join().unwrap();
            assert!(
                occupied,
                "cleanup released its lane while still blocked on socket tracking"
            );
            assert!(!state.active.load(Ordering::SeqCst));
        });
    }
}

#[cfg(feature = "delegation-wire")]
#[path = "http.rs"]
mod http;
#[cfg(all(test, feature = "delegation-issuer"))]
#[path = "tls_issuer_tests.rs"]
mod issuer_lifecycle_tests;

// Test-only scheduling inside the actual response BIO write, armed after the
// handshake and request dispatch. This does not simulate kernel backpressure.
#[cfg(all(test, feature = "delegation-wire"))]
struct ResponseWriteGate {
    entered: std::sync::mpsc::Sender<Instant>,
    release: std::sync::Mutex<std::sync::mpsc::Receiver<()>>,
}
#[cfg(all(test, feature = "delegation-wire"))]
impl ResponseWriteGate {
    fn enter(&self, deadline: Instant) -> io::Result<()> {
        self.entered
            .send(deadline)
            .map_err(|_| io::Error::other("test response gate unavailable"))?;
        self.release
            .lock()
            .map_err(|_| io::Error::other("test response gate poisoned"))?
            .recv_timeout(CYCLE_LIMIT + Duration::from_secs(1))
            .map_err(|_| io::Error::other("bounded test response gate expired"))
    }
}

#[cfg(feature = "delegation-wire")]
struct WireLifetime {
    expires: Duration,
    highwater: Duration,
    invalid: bool,
}
#[cfg(feature = "delegation-wire")]
impl WireLifetime {
    fn check(&mut self, now: Duration) -> io::Result<()> {
        if self.invalid || now < self.highwater || now >= self.expires {
            self.invalid = true;
            return Err(io::Error::other("wire lifetime unavailable"));
        }
        self.highwater = now;
        Ok(())
    }
}

#[cfg(feature = "delegation-wire")]
#[path = "http_client.rs"]
mod http_client;
#[cfg(feature = "delegation-wire")]
pub use http_client::HttpEndpoint;

#[cfg(all(test, feature = "delegation-verifier"))]
pub(crate) use issuer_lifecycle_tests::cert as test_certificate;

#[cfg(feature = "delegation-verifier")]
#[path = "publication_session.rs"]
mod publication_session;
