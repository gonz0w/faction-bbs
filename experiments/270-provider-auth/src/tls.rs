//! Explicit local TLS1.3 proof adapter, never a provider or native-mail entry.
//!
//! Standard OpenSSL verifies key possession and CA/name trust. Exact leaf pins
//! and current leaf validity are independent checks. Policy mappings prove no
//! OMEMO private identity possession. Blocking filesystem dispatch is cooperative:
//! deadlines are checked before and after it, but cannot preempt a filesystem call.
use crate::authority::Authority;
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
        Arc,
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
fn wall_now() -> u64 {
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
fn leaf_current(leaf: &X509, now: u64) -> bool {
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
}
impl DeadlineIo {
    fn new(socket: TcpStream, deadline: Instant, cancel: Arc<AtomicBool>) -> Self {
        Self {
            socket,
            deadline,
            cancel,
        }
    }
    fn timeout(&self) -> io::Result<Duration> {
        check(self.deadline, &self.cancel)
            .map_err(|stage| io::Error::other(format!("TLS {stage:?}")))?;
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
                    continue;
                }
                result => return result,
            }
        }
    }
}
impl Write for DeadlineIo {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
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
fn write_frame(stream: &mut SslStream<DeadlineIo>, bytes: &[u8]) -> Result<(), Stage> {
    if bytes.is_empty() || bytes.len() > MAX_WIRE {
        return Err(Stage::Frame);
    }
    stream
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .map_err(|_| Stage::Frame)?;
    stream.write_all(bytes).map_err(|_| Stage::Frame)?;
    stream.flush().map_err(|_| Stage::Frame)
}

/// Trusted local experiment configuration. It creates no listener or background
/// service; `serve_once` accepts only one connection on a caller-owned loopback
/// listener. Callers must serialize active connections and retain local approval.
#[cfg(test)]
type PolicyHook = Box<dyn FnMut(&mut Authority, usize) + Send>;

#[cfg(test)]
type ClientPostHook = Option<Box<dyn FnOnce()>>;
#[cfg(test)]
thread_local! {
    static CLIENT_POST_HANDSHAKE: std::cell::RefCell<ClientPostHook> = const { std::cell::RefCell::new(None) };
}

pub struct Server {
    acceptor: SslAcceptor,
    clock: Arc<dyn Fn() -> u64 + Send + Sync>,
    #[cfg(test)]
    hook: Option<PolicyHook>,
    #[cfg(test)]
    lose_response: bool,
    #[cfg(test)]
    post_handshake: Option<Box<dyn FnMut() + Send>>,
}
impl Server {
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
            clock: Arc::new(wall_now),
            #[cfg(test)]
            hook: None,
            #[cfg(test)]
            lose_response: false,
            #[cfg(test)]
            post_handshake: None,
        })
    }
    pub fn serve_once(
        &mut self,
        listener: &TcpListener,
        authority: &mut Authority,
        store: &mut Store,
        cancel: Arc<AtomicBool>,
    ) -> Report {
        let admission = Instant::now() + HANDSHAKE_LIMIT;
        let mut report = Report {
            stage: Stage::Admission,
            dispatched: 0,
        };
        if !listener.local_addr().is_ok_and(|a| a.ip().is_loopback())
            || listener.set_nonblocking(true).is_err()
        {
            return report;
        }
        let socket = loop {
            if let Err(stage) = check(admission, &cancel) {
                report.stage = stage;
                return report;
            }
            match listener.accept() {
                Ok((socket, peer)) => {
                    if !peer.ip().is_loopback() {
                        return report;
                    }
                    break socket;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5))
                }
                Err(_) => return report,
            }
        };
        let start = Instant::now();
        let cycle = start + CYCLE_LIMIT;
        let handshake = admission.min(cycle);
        let io = DeadlineIo::new(socket, handshake, cancel.clone());
        let mut stream = match self.acceptor.accept(io) {
            Ok(s) => s,
            Err(_) => {
                report.stage = classify(handshake, &cancel, Stage::Handshake);
                return report;
            }
        };
        #[cfg(test)]
        if let Some(hook) = &mut self.post_handshake {
            hook();
        }
        if let Err(stage) = check(handshake, &cancel) {
            report.stage = stage;
            return report;
        }
        stream.get_mut().deadline = cycle;
        let leaf = match stream.ssl().peer_certificate() {
            Some(leaf)
                if stream.ssl().verify_result() == X509VerifyResult::OK
                    && !stream.ssl().session_reused() =>
            {
                leaf
            }
            _ => {
                report.stage = Stage::Handshake;
                return report;
            }
        };
        if !leaf_current(&leaf, (self.clock)()) {
            report.stage = Stage::Denied;
            return report;
        }
        let digest = match leaf.to_der() {
            Ok(der) => sha256(&der),
            Err(_) => {
                report.stage = Stage::Handshake;
                return report;
            }
        };
        let connection = authority.open_connection();
        let now = (self.clock)();
        if let Err(stage) = check(handshake, &cancel) {
            report.stage = stage;
            return report;
        }
        let handle = match authority.authenticate(
            &connection,
            digest,
            now,
            start.elapsed().as_millis() as u64,
        ) {
            Ok(h) => h,
            Err(_) => {
                report.stage = Stage::Denied;
                return report;
            }
        };
        let result = (|| {
            for index in 0..MAX_OPERATIONS {
                check(cycle, &cancel)?;
                let bytes = match read_frame(&mut stream) {
                    Ok(Some(bytes)) => bytes,
                    Ok(None) => return Ok(()),
                    Err(stage) => return Err(classify(cycle, &cancel, stage)),
                };
                check(cycle, &cancel)?;
                let request = decode_request(&bytes).map_err(|_| Stage::Frame)?;
                #[cfg(test)]
                if let Some(hook) = &mut self.hook {
                    hook(authority, index)
                }
                #[cfg(not(test))]
                let _ = index;
                let now = (self.clock)();
                if !leaf_current(&leaf, now) {
                    return Err(Stage::Denied);
                }
                check(cycle, &cancel)?;
                report.dispatched += 1;
                let response = authority
                    .dispatch(
                        &connection,
                        &handle,
                        store,
                        request,
                        now,
                        start.elapsed().as_millis() as u64,
                    )
                    .map_err(|_| Stage::Denied)?;
                check(cycle, &cancel)?;
                #[cfg(test)]
                if self.lose_response {
                    return Err(Stage::Frame);
                }
                let bytes = encode_response(&response).map_err(|_| Stage::Frame)?;
                write_frame(&mut stream, &bytes)
                    .map_err(|stage| classify(cycle, &cancel, stage))?;
                check(cycle, &cancel)?;
            }
            Err(Stage::Limit)
        })();
        authority.close_connection(&connection);
        report.stage = result.err().unwrap_or(Stage::Complete);
        report
    }
}

/// Owns one verified loopback connection. No raw TLS stream or authenticated
/// principal is exposed. The pin is checked before any application bytes.
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
        #[cfg(test)]
        CLIENT_POST_HANDSHAKE.with(|hook| {
            if let Some(hook) = hook.borrow_mut().take() {
                hook();
            }
        });
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
#[cfg(test)]
#[path = "tls_tests.rs"]
mod tests;
