//! Explicit authenticated binary attachment transport on the member device.
//! A foreground cycle owns one bounded TLS worker and joins it on every exit.
pub(crate) use crate::mail_blob_transfer::TransferDirection;
use crate::mail_checkpoint::CheckpointVault;
use faction_files::private_blobs::{Binding, Principal, Request, Response, encode_request};
pub use faction_provider::tls::ClientCredentials as Credentials;
use faction_provider::tls::{CYCLE_LIMIT, ClientAdmission, HANDSHAKE_LIMIT, Stage};
use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tokio::sync::oneshot;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeviceRoute {
    Ordinary,
    #[cfg(feature = "own-home-provider")]
    Publication,
    #[cfg(feature = "own-home-provider")]
    OwnHomeRetrieval,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Configuration,
    Identity,
    Keylog,
    Transport(Stage),
    Worker,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("authenticated attachment transport unavailable")
    }
}
impl std::error::Error for Error {}
/// Trusted local assertions. They never establish server-side device authority.
pub struct Config {
    home: String,
    address: SocketAddr,
    expected_name: String,
    leaf_pin: [u8; 32],
    expected: Principal,
    #[cfg(feature = "own-home-provider")]
    own_home: bool,
}
impl Config {
    pub fn new(
        home: &str,
        address: SocketAddr,
        expected_name: &str,
        leaf_pin: [u8; 32],
        expected: Principal,
    ) -> Result<Self, Error> {
        for domain in [home, expected_name] {
            Principal {
                account: format!("policy@{domain}"),
                device: 1,
                identity: [1; 32],
            }
            .validate()
            .map_err(|_| Error::Configuration)?;
        }
        expected.validate().map_err(|_| Error::Configuration)?;
        if !address.ip().is_loopback()
            || address.port() == 0
            || leaf_pin == [0; 32]
            || expected.account.split_once('@').map(|(_, domain)| domain) != Some(home)
        {
            return Err(Error::Configuration);
        }
        Ok(Self {
            home: home.into(),
            address,
            expected_name: expected_name.into(),
            leaf_pin,
            expected,
            #[cfg(feature = "own-home-provider")]
            own_home: false,
        })
    }
    /// Explicit device-to-own-home selection. This local mode grants no server
    /// invitation or actor authority and performs no I/O.
    #[cfg(feature = "own-home-provider")]
    pub fn new_own_home(
        home: &str,
        address: SocketAddr,
        expected_name: &str,
        leaf_pin: [u8; 32],
        expected: Principal,
    ) -> Result<Self, Error> {
        let mut config = Self::new(home, address, expected_name, leaf_pin, expected)?;
        config.own_home = true;
        Ok(config)
    }
}
/// Independent transport credentials, distinct from the unlocked OMEMO vault.
/// Construction performs no network or filesystem operation.
pub struct DeviceProvider {
    config: Config,
    credentials: Credentials,
}
impl DeviceProvider {
    pub fn new(config: Config, credentials: Credentials) -> Self {
        Self {
            config,
            credentials,
        }
    }
    /// Check the current local vault before opening a connection or changing a
    /// transfer spool. Configuration and certificate subjects are only assertions.
    pub fn validate(&self, vault: &CheckpointVault) -> Result<(), Error> {
        keylog()?;
        let state = vault.inspection().map_err(|_| Error::Identity)?;
        if vault.legacy().map_err(|_| Error::Identity)?.home != self.config.expected.account
            || state.local_device != self.config.expected.device
            || *state.identity != self.config.expected.identity
        {
            return Err(Error::Identity);
        }
        Ok(())
    }
    /// Whether the caller explicitly selected the own-home transport profile.
    /// This is local routing configuration, never remote approval.
    pub fn uses_own_home(&self) -> bool {
        #[cfg(feature = "own-home-provider")]
        {
            self.config.own_home
        }
        #[cfg(not(feature = "own-home-provider"))]
        {
            false
        }
    }
    fn classify(
        &self,
        binding: &Binding,
        direction: TransferDirection,
    ) -> Result<DeviceRoute, Error> {
        binding.validate().map_err(|_| Error::Configuration)?;
        let own = match direction {
            TransferDirection::Upload => &binding.sender,
            TransferDirection::Fetch => &binding.recipient,
        };
        if own != &self.config.expected {
            return Err(Error::Identity);
        }
        let local = |principal: &Principal| {
            principal.account.split_once('@').map(|(_, home)| home)
                == Some(self.config.home.as_str())
        };
        if local(&binding.sender) && local(&binding.recipient) {
            return Ok(DeviceRoute::Ordinary);
        }
        #[cfg(feature = "own-home-provider")]
        if self.config.own_home {
            return match direction {
                TransferDirection::Upload if !local(&binding.recipient) => {
                    Ok(DeviceRoute::Publication)
                }
                TransferDirection::Fetch if !local(&binding.sender) => {
                    Ok(DeviceRoute::OwnHomeRetrieval)
                }
                _ => Err(Error::Configuration),
            };
        }
        Err(Error::Configuration)
    }
    pub(crate) fn permits_binding(&self, binding: &Binding, direction: TransferDirection) -> bool {
        self.classify(binding, direction).is_ok()
    }
    pub(crate) async fn open(
        &self,
        binding: &Binding,
        direction: TransferDirection,
    ) -> Result<Worker, Error> {
        keylog()?;
        let route = self.classify(binding, direction)?;
        let start = Instant::now();
        match route {
            DeviceRoute::Ordinary => {
                self.open_ordinary(binding, direction, start, || {
                    tokio::net::TcpStream::connect(self.config.address)
                })
                .await
            }
            #[cfg(feature = "own-home-provider")]
            DeviceRoute::Publication | DeviceRoute::OwnHomeRetrieval => {
                let endpoint = faction_provider::delivery::CarrierEndpoint::new(
                    self.config.address,
                    self.config.expected_name.clone(),
                    self.config.leaf_pin,
                    self.credentials.clone(),
                )
                .map_err(Error::Transport)?;
                spawn_worker(
                    binding.clone(),
                    direction,
                    start,
                    Arc::new(AtomicBool::new(false)),
                    move || Ok(Transport::Delivery(endpoint, route)),
                )
                .await
            }
        }
    }
    async fn open_ordinary<F, C>(
        &self,
        binding: &Binding,
        direction: TransferDirection,
        start: Instant,
        connect: F,
    ) -> Result<Worker, Error>
    where
        F: FnOnce() -> C,
        C: std::future::Future<Output = std::io::Result<tokio::net::TcpStream>>,
    {
        keylog()?;
        // Classification precedes even creating the connection future. Ordinary
        // mode retains one legacy binary session and its original admission.
        if self.classify(binding, direction)? != DeviceRoute::Ordinary {
            return Err(Error::Configuration);
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let admission = ClientAdmission::new(
            &self.config.expected_name,
            self.config.leaf_pin,
            self.credentials.clone(),
            cancel.clone(),
        )
        .map_err(Error::Transport)?;
        let socket = tokio::time::timeout_at(
            tokio::time::Instant::from_std(start + HANDSHAKE_LIMIT),
            connect(),
        )
        .await
        .map_err(|_| Error::Transport(Stage::Deadline))?
        .map_err(|_| Error::Transport(Stage::Admission))?
        .into_std()
        .map_err(|_| Error::Transport(Stage::Admission))?;
        spawn_worker(binding.clone(), direction, start, cancel, move || {
            admission
                .connect(socket)
                .map(Transport::Ordinary)
                .map_err(Error::Transport)
        })
        .await
    }
    #[cfg(test)]
    async fn open_with<F, C>(&self, connect: F) -> Result<Worker, Error>
    where
        F: FnOnce() -> C,
        C: std::future::Future<Output = std::io::Result<tokio::net::TcpStream>>,
    {
        let binding = Binding {
            sender: self.config.expected.clone(),
            recipient: self.config.expected.clone(),
            id: [3; 32],
            size: 32,
            created: 1,
            expires: 100,
        };
        self.open_ordinary(&binding, TransferDirection::Upload, Instant::now(), connect)
            .await
    }
}

enum Transport {
    Ordinary(faction_provider::tls::Client),
    #[cfg(feature = "own-home-provider")]
    Delivery(faction_provider::delivery::CarrierEndpoint, DeviceRoute),
}
impl Transport {
    fn request(
        &mut self,
        binding: &Binding,
        request: Request,
        deadline: Instant,
        cancel: Arc<AtomicBool>,
        #[cfg(feature = "own-home-provider")] diagnostic: &mut WorkerSnapshot,
    ) -> Result<Response, Error> {
        #[cfg(not(feature = "own-home-provider"))]
        let _ = (binding, deadline, cancel);
        match self {
            Self::Ordinary(client) => client.request(&request).map_err(Error::Transport),
            #[cfg(feature = "own-home-provider")]
            Self::Delivery(endpoint, route) => {
                use faction_provider::delivery::Collection;
                let mut operation = endpoint
                    .operation_until(deadline, cancel)
                    .map_err(Error::Transport)?;
                let result = (|| {
                    match route {
                        DeviceRoute::Publication => {
                            operation.submit_publication(binding.clone(), request)
                        }
                        DeviceRoute::OwnHomeRetrieval => {
                            operation.submit_own_home(binding.clone(), request)
                        }
                        DeviceRoute::Ordinary => return Err(Error::Configuration),
                    }
                    .map_err(Error::Transport)?;
                    // One submit and bounded fresh collects on this SAME operation.
                    // Busy neither renews the parent clock nor changes the route.
                    for _ in 0..6 {
                        let collected = match route {
                            DeviceRoute::OwnHomeRetrieval => operation.collect_own_home_paced(),
                            DeviceRoute::Publication => operation.collect_paced(),
                            DeviceRoute::Ordinary => return Err(Error::Configuration),
                        }
                        .map_err(Error::Transport)?;
                        match collected {
                            Collection::Complete(response) => return Ok(response),
                            Collection::Busy => (),
                        }
                    }
                    Err(Error::Transport(Stage::Limit))
                })();
                if let Some(snapshot) = operation.diagnostic_snapshot() {
                    diagnostic.operation(snapshot);
                }
                result
            }
        }
    }
}
fn request_matches(binding: &Binding, direction: TransferDirection, request: &Request) -> bool {
    match request {
        Request::Begin(value) => direction == TransferDirection::Upload && value == binding,
        Request::Status { id } => *id == binding.id,
        Request::PutChunk { id, .. } | Request::Commit { id } | Request::Delete { id } => {
            direction == TransferDirection::Upload && *id == binding.id
        }
        Request::GetChunk { id, .. } => direction == TransferDirection::Fetch && *id == binding.id,
    }
}
async fn spawn_worker(
    binding: Binding,
    direction: TransferDirection,
    start: Instant,
    cancel: Arc<AtomicBool>,
    initialize: impl FnOnce() -> Result<Transport, Error> + Send + 'static,
) -> Result<Worker, Error> {
    let (send, receive) = mpsc::sync_channel::<Command>(1);
    let (ready, initialized) = oneshot::channel();
    let cancelled = cancel.clone();
    let pinned = binding.clone();
    let deadline = start + CYCLE_LIMIT;
    #[cfg(test)]
    let observation = Arc::new(RequestObservation::default());
    #[cfg(test)]
    let worker_observation = observation.clone();
    let thread = thread::Builder::new()
        .name("faction-device-tls".into())
        .spawn(move || {
            #[cfg(feature = "own-home-provider")]
            let mut diagnostic = WorkerSnapshot::default();
            let mut transport = match initialize() {
                Ok(transport) => {
                    if ready.send(Ok(())).is_err() {
                        return unobserved_worker_exit();
                    }
                    transport
                }
                Err(error) => {
                    let _ = ready.send(Err(error));
                    return unobserved_worker_exit();
                }
            };
            let mut attempted = 0;
            while !cancelled.load(Ordering::SeqCst) && Instant::now() < deadline {
                match receive.recv_timeout(Duration::from_millis(100)) {
                    Ok(Command {
                        binding,
                        direction: requested_direction,
                        request,
                        response,
                    }) => {
                        if cancelled.load(Ordering::SeqCst) || response.is_closed() {
                            break;
                        }
                        let result = if Instant::now() >= deadline {
                            Err(Error::Transport(Stage::Deadline))
                        } else if attempted >= 8 {
                            Err(Error::Transport(Stage::Limit))
                        } else if binding != pinned
                            || requested_direction != direction
                            || !request_matches(&pinned, direction, &request)
                        {
                            Err(Error::Configuration)
                        } else {
                            attempted += 1;
                            #[cfg(test)]
                            let observation_index =
                                worker_observation.entered.fetch_add(1, Ordering::SeqCst);
                            #[cfg(feature = "own-home-provider")]
                            {
                                diagnostic.request = diagnostic_request(&request);
                                diagnostic.phase =
                                    faction_provider::delivery::DiagnosticPhase::Unobserved;
                                diagnostic.phase_us = 0;
                                diagnostic.transport = None;
                                diagnostic.terminal_carrier_attempts = 0;
                                diagnostic.terminal_busy = 0;
                                diagnostic.exchange_origin =
                                    faction_provider::delivery::ExchangeOrigin::Unobserved;
                                diagnostic.attempted = diagnostic.attempted.saturating_add(1);
                            }
                            let result = transport.request(
                                &pinned,
                                request,
                                deadline,
                                cancelled.clone(),
                                #[cfg(feature = "own-home-provider")]
                                &mut diagnostic,
                            );
                            #[cfg(feature = "own-home-provider")]
                            {
                                if result.is_ok() {
                                    diagnostic.completed = diagnostic.completed.saturating_add(1);
                                }
                                if let Err(Error::Transport(stage)) = result.as_ref() {
                                    diagnostic.transport = Some(*stage);
                                }
                            }
                            #[cfg(test)]
                            if let Some(slot) = worker_observation
                                .results
                                .lock()
                                .unwrap()
                                .get_mut(observation_index)
                            {
                                *slot = Some(result.as_ref().map(|_| ()).map_err(|error| *error));
                            }
                            result
                        };
                        let failed = result.is_err();
                        if response.send(result).is_err() || failed {
                            break;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => (),
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            #[cfg(feature = "own-home-provider")]
            {
                diagnostic
            }
        })
        .map_err(|_| Error::Worker)?;
    #[cfg(test)]
    WORKER_STARTS.with(|count| count.set(count.get() + 1));
    let worker = Worker {
        send: Some(send),
        thread: Some(thread),
        cancel,
        binding,
        direction,
        deadline,
        #[cfg(feature = "own-home-provider")]
        diagnostic_sink: None,
        #[cfg(test)]
        observation,
    };
    initialized.await.map_err(|_| Error::Worker)??;
    Ok(worker)
}

fn keylog() -> Result<(), Error> {
    if ["SSLKEYLOGFILE", "TLSKEYLOGFILE"]
        .iter()
        .any(|key| std::env::var_os(key).is_some())
    {
        Err(Error::Keylog)
    } else {
        Ok(())
    }
}
struct Command {
    binding: Binding,
    direction: TransferDirection,
    request: Request,
    response: oneshot::Sender<Result<Response, Error>>,
}
/// No vault, OMEMO key, plaintext or descriptor key is sent to this worker.
/// There is one request slot, one result slot and one owning foreground future.
#[cfg(test)]
#[derive(Default)]
struct RequestObservation {
    entered: std::sync::atomic::AtomicUsize,
    results: std::sync::Mutex<[Option<Result<(), Error>>; 8]>,
}
#[cfg(feature = "own-home-provider")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DiagnosticRequest {
    #[default]
    Unobserved,
    Begin,
    Status,
    Put,
    Commit,
    Get,
    Delete,
}
#[cfg(feature = "own-home-provider")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SnapshotState {
    #[default]
    Unavailable,
    Observed,
    JoinPanicked,
}
#[cfg(feature = "own-home-provider")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorkerSnapshot {
    pub state: SnapshotState,
    pub request: DiagnosticRequest,
    pub phase: faction_provider::delivery::DiagnosticPhase,
    pub transport: Option<Stage>,
    pub attempted: u8,
    pub completed: u8,
    pub carrier_attempts: u8,
    pub busy: u8,
    pub connections: u8,
    pub terminal_carrier_attempts: u8,
    pub terminal_busy: u8,
    pub exchange_origin: faction_provider::delivery::ExchangeOrigin,
    pub timings: faction_provider::delivery::DiagnosticTimings,
    pub phase_us: u64,
}
#[cfg(feature = "own-home-provider")]
type WorkerExit = WorkerSnapshot;
#[cfg(not(feature = "own-home-provider"))]
type WorkerExit = ();
fn unobserved_worker_exit() -> WorkerExit {
    #[cfg(feature = "own-home-provider")]
    {
        WorkerSnapshot::default()
    }
}
#[cfg(feature = "own-home-provider")]
impl WorkerSnapshot {
    fn operation(&mut self, operation: faction_provider::delivery::OperationSnapshot) {
        self.state = SnapshotState::Observed;
        self.phase = operation.phase;
        self.phase_us = operation.phase_us;
        self.terminal_carrier_attempts = operation.attempted;
        self.terminal_busy = operation.busy;
        self.exchange_origin = operation.exchange_origin;
        self.carrier_attempts = self.carrier_attempts.saturating_add(operation.attempted);
        self.busy = self.busy.saturating_add(operation.busy);
        self.connections = self.connections.saturating_add(operation.connections);
        self.timings = self.timings.saturating_add(operation.timings);
    }
}
#[cfg(feature = "own-home-provider")]
fn diagnostic_request(request: &Request) -> DiagnosticRequest {
    match request {
        Request::Begin(_) => DiagnosticRequest::Begin,
        Request::Status { .. } => DiagnosticRequest::Status,
        Request::PutChunk { .. } => DiagnosticRequest::Put,
        Request::Commit { .. } => DiagnosticRequest::Commit,
        Request::GetChunk { .. } => DiagnosticRequest::Get,
        Request::Delete { .. } => DiagnosticRequest::Delete,
    }
}
pub(crate) struct Worker {
    send: Option<SyncSender<Command>>,
    thread: Option<JoinHandle<WorkerExit>>,
    cancel: Arc<AtomicBool>,
    binding: Binding,
    direction: TransferDirection,
    deadline: Instant,
    #[cfg(feature = "own-home-provider")]
    diagnostic_sink: Option<Arc<std::sync::OnceLock<WorkerSnapshot>>>,
    #[cfg(test)]
    observation: Arc<RequestObservation>,
}
impl Worker {
    #[cfg(feature = "own-home-provider")]
    pub(crate) fn observe_final(&mut self, sink: Arc<std::sync::OnceLock<WorkerSnapshot>>) {
        self.diagnostic_sink = Some(sink);
    }
    #[cfg(test)]
    fn request_observation(&self) -> Arc<RequestObservation> {
        self.observation.clone()
    }

    #[cfg(test)]
    async fn request(&mut self, request: Request) -> Result<Response, Error> {
        self.request_bound(&self.binding.clone(), self.direction, request)
            .await
    }
    pub(crate) async fn request_bound(
        &mut self,
        binding: &Binding,
        direction: TransferDirection,
        request: Request,
    ) -> Result<Response, Error> {
        if self.cancel.load(Ordering::SeqCst) {
            return Err(Error::Transport(Stage::Cancelled));
        }
        if Instant::now() >= self.deadline {
            return Err(Error::Transport(Stage::Deadline));
        }
        if binding != &self.binding
            || direction != self.direction
            || !request_matches(binding, direction, &request)
        {
            return Err(Error::Configuration);
        }
        // Bound caller-owned input before queueing it; Client rechecks framing and
        // its original operation/deadline budget on the actual TLS connection.
        encode_request(&request).map_err(|_| Error::Transport(Stage::Frame))?;
        let (response, receive) = oneshot::channel();
        self.send
            .as_ref()
            .ok_or(Error::Worker)?
            .try_send(Command {
                binding: binding.clone(),
                direction,
                request,
                response,
            })
            .map_err(|_| Error::Worker)?;
        receive.await.map_err(|_| Error::Worker)?
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::SeqCst);
        self.send.take();
        // Client TCP is asynchronous; all worker TLS syscalls use the existing
        // 100-ms cancellation slices. Joining cannot leave a detached TLS task.
        if let Some(thread) = self.thread.take() {
            let joined = thread.join();
            #[cfg(feature = "own-home-provider")]
            if let Some(sink) = self.diagnostic_sink.take() {
                let snapshot = joined.unwrap_or_else(|_| WorkerSnapshot {
                    state: SnapshotState::JoinPanicked,
                    ..WorkerSnapshot::default()
                });
                // Only the existing join publishes: no live polling channel or
                // lock is held across a request, cancellation, or network I/O.
                let _ = sink.set(snapshot);
            }
            #[cfg(not(feature = "own-home-provider"))]
            let _ = joined;
        }
    }
}

#[cfg(test)]
thread_local! { static WORKER_STARTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "own-home-provider")]
    async fn collection_route_waiting_control(own_home: bool) {
        use std::io::BufRead;
        let (fixture, mut provider) = provider();
        let script = r#"import pathlib, socket, ssl, sys
root = pathlib.Path(sys.argv[1])
purpose, kind = map(int, sys.argv[2:])
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.minimum_version = context.maximum_version = ssl.TLSVersion.TLSv1_3
context.options |= ssl.OP_NO_TICKET
context.num_tickets = 0
context.load_cert_chain(root/'cert.pem', root/'key.pem')
context.load_verify_locations(root/'cert.pem')
context.verify_mode = ssl.CERT_REQUIRED
listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
listener.bind(('127.0.0.1',0)); listener.listen(1); listener.settimeout(5)
print('PORT',listener.getsockname()[1],flush=True)
def receive(connection):
    def exact(size):
        data=b''
        while len(data)<size:
            part=connection.recv(size-len(data)); assert part
            data+=part
        return data
    size=int.from_bytes(exact(4),'big'); assert 0<size<=8192
    return exact(size)
def reply(connection, data):
    connection.sendall(len(data).to_bytes(4,'big')+data)
correlation=bytes([47])*32
for index in range(3):
    raw,address=listener.accept(); assert address[0]=='127.0.0.1'; raw.settimeout(5)
    with context.wrap_socket(raw,server_side=True) as connection:
        frame=receive(connection)
        assert frame[:6]==b'FDEL01' and frame[7]==purpose
        if index==0:
            assert frame[6]==0 and frame[8:40]==bytes(32)
            submitted=frame
            reply(connection,b'FDEL01'+bytes([2])+correlation)
        else:
            assert frame[6]==kind, 'wrong explicit collection route'
            assert frame[8:40]==correlation and frame[40:]==submitted[40:]
            reply(connection,b'FDEL01'+(bytes([3]) if index==1 else bytes([4])+(root/'response').read_bytes()))
listener.close()
print('EXACT_ROUTE_AND_BUSY',flush=True)
"#;
        let purpose = if own_home { 3 } else { 1 };
        let kind = if own_home { 6 } else { 1 };
        let path = fixture.0.join("collection-route.py");
        fs::write(&path, script).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let own = provider.config.expected.clone();
        let foreign = Principal {
            account: "bob@away.test".into(),
            device: 202,
            identity: [2; 32],
        };
        let binding = Binding {
            sender: if own_home {
                foreign.clone()
            } else {
                own.clone()
            },
            recipient: if own_home { own } else { foreign },
            id: [3; 32],
            size: 17,
            created: 1,
            expires: 100,
        };
        let expected = Response::Progress {
            binding: binding.clone(),
            offset: 17,
            committed: true,
        };
        fs::write(
            fixture.0.join("response"),
            faction_files::private_blobs::encode_response(&expected).unwrap(),
        )
        .unwrap();
        fs::set_permissions(
            fixture.0.join("response"),
            fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        let mut child = std::process::Command::new("python3")
            .arg("-I")
            .arg(&path)
            .arg(&fixture.0)
            .arg(purpose.to_string())
            .arg(kind.to_string())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let output = std::io::BufReader::new(child.stdout.take().unwrap());
        let mut peer = FaultPeer {
            address: "127.0.0.1:1".parse().unwrap(),
            child,
            output,
        };
        let mut line = String::new();
        peer.output.read_line(&mut line).unwrap();
        peer.address
            .set_port(line.trim().strip_prefix("PORT ").unwrap().parse().unwrap());
        provider.config.address = peer.address;
        provider.config.own_home = true;
        let direction = if own_home {
            TransferDirection::Fetch
        } else {
            TransferDirection::Upload
        };
        let sink = Arc::new(std::sync::OnceLock::new());
        let mut worker = provider.open(&binding, direction).await.unwrap();
        worker.observe_final(sink.clone());
        let response = worker
            .request_bound(&binding, direction, Request::Status { id: binding.id })
            .await;
        drop(worker);
        let status = peer.child.wait().unwrap();
        assert!(
            status.success(),
            "actual SDK emitted wrong collection wire kind"
        );
        assert_eq!(response, Ok(expected));
        line.clear();
        peer.output.read_line(&mut line).unwrap();
        assert_eq!(line, "EXACT_ROUTE_AND_BUSY\n");
        let snapshot = sink.get().unwrap();
        assert_eq!(
            (
                snapshot.carrier_attempts,
                snapshot.busy,
                snapshot.connections
            ),
            (3, 1, 3)
        );
        assert!(
            snapshot.timings.planned_wait_us > 0 && snapshot.timings.actual_wait_us > 0,
            "route skipped measured submit/Busy pacing"
        );
        assert!(std::net::TcpStream::connect(peer.address).is_err());
    }
    #[cfg(feature = "own-home-provider")]
    #[tokio::test(flavor = "current_thread")]
    async fn collection_route_own_home_wait_keeps_submit_and_busy_pacing() {
        collection_route_waiting_control(true).await;
    }
    #[cfg(feature = "own-home-provider")]
    #[tokio::test(flavor = "current_thread")]
    async fn collection_route_publication_keeps_ordinary_paced_collect() {
        collection_route_waiting_control(false).await;
    }

    #[cfg(feature = "own-home-provider")]
    #[test]
    fn diagnostic_terminal_operation_counters_replace_while_totals_accumulate() {
        use faction_provider::delivery::{ExchangeOrigin, OperationSnapshot};
        let mut worker = WorkerSnapshot::default();
        worker.operation(OperationSnapshot {
            attempted: 2,
            busy: 1,
            exchange_origin: ExchangeOrigin::ReadError,
            ..Default::default()
        });
        worker.operation(OperationSnapshot {
            attempted: 5,
            busy: 3,
            exchange_origin: ExchangeOrigin::Refused,
            ..Default::default()
        });
        assert_eq!((worker.carrier_attempts, worker.busy), (7, 4));
        assert_eq!(
            (
                worker.terminal_carrier_attempts,
                worker.terminal_busy,
                worker.exchange_origin
            ),
            (5, 3, ExchangeOrigin::Refused)
        );
        worker.operation(OperationSnapshot {
            attempted: 1,
            ..Default::default()
        });
        assert_eq!(
            (
                worker.terminal_carrier_attempts,
                worker.terminal_busy,
                worker.exchange_origin
            ),
            (1, 0, ExchangeOrigin::Unobserved)
        );
        assert_eq!((worker.carrier_attempts, worker.busy), (8, 4));
    }
    use std::{
        fs,
        os::unix::fs::{DirBuilderExt, PermissionsExt},
        path::PathBuf,
        pin::Pin,
        sync::atomic::AtomicUsize,
        task::{Context, Poll},
    };
    include!("test_fixture_retention.rs");
    struct Owned(PathBuf);
    impl Drop for Owned {
        fn drop(&mut self) {
            assert_eq!(
                fs::read(self.0.join("OWNED")).unwrap(),
                b"issue279 private connector unit"
            );
            if let Some(sink) = std::env::var_os("FACTION322_CORE_FIXTURE_RETENTION") {
                capture_owned_test_fixture(&self.0, &PathBuf::from(sink));
            }
            fs::remove_dir_all(&self.0).unwrap();
            assert!(!self.0.exists());
        }
    }
    fn provider() -> (Owned, DeviceProvider) {
        let mut random = [0; 16];
        getrandom::fill(&mut random).unwrap();
        let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "faction279-unit-{}",
            crate::mail_data::hex(&random)
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        fs::write(root.join("OWNED"), b"issue279 private connector unit").unwrap();
        let fixture = Owned(root);
        let openssl = if cfg!(target_os = "macos") {
            "/opt/homebrew/opt/openssl@3/bin/openssl"
        } else {
            "/usr/bin/openssl"
        };
        let openssl_config = fixture.0.join("openssl.cnf");
        fs::write(&openssl_config, b"[req]\ndistinguished_name=dn\n[dn]\n").unwrap();
        fs::set_permissions(&openssl_config, fs::Permissions::from_mode(0o600)).unwrap();
        let output = std::process::Command::new(openssl)
            .args(["req", "-config"])
            .arg(&openssl_config)
            .args([
                "-x509",
                "-newkey",
                "ec",
                "-pkeyopt",
                "ec_paramgen_curve:prime256v1",
                "-nodes",
                "-keyout",
                "key.pem",
                "-out",
                "cert.pem",
                "-subj",
                "/CN=provider.test",
                "-addext",
                "subjectAltName=DNS:provider.test",
                "-days",
                "1",
            ])
            .current_dir(&fixture.0)
            .output()
            .unwrap();
        assert!(output.status.success());
        for entry in fs::read_dir(&fixture.0).unwrap() {
            fs::set_permissions(entry.unwrap().path(), fs::Permissions::from_mode(0o600)).unwrap();
        }
        let certificate = fs::read(fixture.0.join("cert.pem")).unwrap();
        let key = fs::read(fixture.0.join("key.pem")).unwrap();
        let credentials = Credentials::from_pem(&certificate, &certificate, &key).unwrap();
        use sha2::{Digest, Sha256};
        let der = std::process::Command::new(openssl)
            .args(["x509", "-in", "cert.pem", "-outform", "DER"])
            .current_dir(&fixture.0)
            .output()
            .unwrap();
        assert!(der.status.success());
        let pin = Sha256::digest(&der.stdout).into();
        let config = Config::new(
            "home.test",
            "127.0.0.1:1234".parse().unwrap(),
            "provider.test",
            pin,
            Principal {
                account: "alice@home.test".into(),
                device: 101,
                identity: [1; 32],
            },
        )
        .unwrap();
        (fixture, DeviceProvider::new(config, credentials))
    }
    struct PendingConnect {
        polls: Arc<AtomicUsize>,
        dropped: Arc<AtomicBool>,
    }
    impl std::future::Future for PendingConnect {
        type Output = std::io::Result<tokio::net::TcpStream>;
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            self.polls.fetch_add(1, Ordering::SeqCst);
            Poll::Pending
        }
    }
    impl Drop for PendingConnect {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::SeqCst);
        }
    }
    #[tokio::test(flavor = "current_thread")]
    async fn pending_connector_is_polled_once_and_dropped_without_worker_or_mail_release() {
        let (_fixture, provider) = provider();
        let calls = Arc::new(AtomicUsize::new(0));
        let polls = Arc::new(AtomicUsize::new(0));
        let dropped = Arc::new(AtomicBool::new(false));
        let calls_inside = calls.clone();
        let polls_inside = polls.clone();
        let dropped_inside = dropped.clone();
        let mut future = Box::pin(provider.open_with(move || {
            calls_inside.fetch_add(1, Ordering::SeqCst);
            PendingConnect {
                polls: polls_inside,
                dropped: dropped_inside,
            }
        }));
        let start = Instant::now();
        tokio::select! {result=&mut future=>panic!("pending connector returned {}",result.is_ok()),_=tokio::time::sleep(Duration::from_millis(20))=>{}}
        drop(future);
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(polls.load(Ordering::SeqCst) > 0);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(dropped.load(Ordering::SeqCst));
        WORKER_STARTS.with(|count| assert_eq!(count.get(), 0));
    }
    #[test]
    fn worker_start_observer_has_a_real_tcp_positive_control() {
        let (_fixture, mut provider) = provider();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        provider.config.address = listener.local_addr().unwrap();
        thread::scope(|scope| {
            let peer = scope.spawn(move || {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut bytes = [0; 8192];
                while std::io::Read::read(&mut socket, &mut bytes).unwrap_or(0) != 0 {}
            });
            tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
                let mut call=Box::pin(provider.open_with(||tokio::net::TcpStream::connect(provider.config.address)));
                tokio::select! {result=&mut call=>panic!("stalled TLS returned {}",result.is_ok()),_=tokio::time::sleep(Duration::from_millis(40))=>{}}
                drop(call);
            });
            peer.join().unwrap();
            WORKER_STARTS.with(|count| assert_eq!(count.get(), 1));
        });
    }
    struct FaultPeer {
        address: std::net::SocketAddr,
        child: std::process::Child,
        output: std::io::BufReader<std::process::ChildStdout>,
    }
    impl FaultPeer {
        fn start(fixture: &Owned, frames: &[u8]) -> Self {
            use std::io::BufRead;
            fs::write(fixture.0.join("replies"), frames).unwrap();
            let script = r#"import pathlib, socket, ssl, sys
root = pathlib.Path(sys.argv[1])
frames = (root / 'replies').read_bytes()
position = 0
count = 0
while position < len(frames):
    size = int.from_bytes(frames[position:position+4], 'big')
    assert 0 < size <= 8192
    position += 4 + size
    count += 1
assert position == len(frames) and count == 8
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.minimum_version = ssl.TLSVersion.TLSv1_3
context.maximum_version = ssl.TLSVersion.TLSv1_3
context.options |= ssl.OP_NO_TICKET
context.num_tickets = 0
context.load_cert_chain(root / 'cert.pem', root / 'key.pem')
context.load_verify_locations(root / 'cert.pem')
context.verify_mode = ssl.CERT_REQUIRED
listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
listener.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 1024)
listener.bind(('127.0.0.1', 0))
listener.listen(1)
listener.settimeout(5)
print('PORT', listener.getsockname()[1], flush=True)
raw, address = listener.accept()
assert address[0] == '127.0.0.1'
raw.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 1024)
raw.settimeout(5)
with context.wrap_socket(raw, server_side=True) as connection:
    connection.settimeout(5)
    assert len(frames) < 16384
    sent = connection.send(frames)
    assert sent == len(frames)
    print('QUEUED', count, 'SINGLE', sent, flush=True)
    # Do not read any application request. All replies were already sent.
    sys.stdin.buffer.read(1)
listener.close()
"#;
            let path = fixture.0.join("write-fault-peer.py");
            fs::write(&path, script).unwrap();
            for file in ["replies", "write-fault-peer.py"] {
                fs::set_permissions(fixture.0.join(file), fs::Permissions::from_mode(0o600))
                    .unwrap();
            }
            let mut child = std::process::Command::new("python3")
                .arg("-I")
                .arg(&path)
                .arg(&fixture.0)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            let output = std::io::BufReader::new(child.stdout.take().unwrap());
            let mut peer = Self {
                address: "127.0.0.1:1".parse().unwrap(),
                child,
                output,
            };
            let mut line = String::new();
            assert!(
                peer.output.read_line(&mut line).unwrap() > 0,
                "fault peer did not start"
            );
            let port: u16 = line.trim().strip_prefix("PORT ").unwrap().parse().unwrap();
            assert_ne!(port, 0);
            peer.address.set_port(port);
            peer
        }
        fn queued_single_batch(&mut self) -> (usize, usize) {
            use std::io::BufRead;
            let mut line = String::new();
            assert!(
                self.output.read_line(&mut line).unwrap() > 0,
                "fault peer did not queue a single batch"
            );
            let fields: Vec<_> = line.split_whitespace().collect();
            assert_eq!(fields.len(), 4);
            assert_eq!(fields[0], "QUEUED");
            assert_eq!(fields[2], "SINGLE");
            (fields[1].parse().unwrap(), fields[3].parse().unwrap())
        }
        fn stop(&mut self) {
            let start = Instant::now();
            if self.child.try_wait().unwrap().is_none() {
                self.child.kill().unwrap();
            }
            self.child.wait().unwrap();
            assert!(start.elapsed() < Duration::from_secs(5));
        }
    }
    impl Drop for FaultPeer {
        fn drop(&mut self) {
            self.stop();
        }
    }

    struct FilledDuplicate {
        bytes: usize,
        blocked: std::io::ErrorKind,
    }
    fn fill_owned_duplicate(
        mut socket: std::net::TcpStream,
        cycle_start: Instant,
    ) -> std::io::Result<FilledDuplicate> {
        use std::io::Write;
        // This fixed test-only corruption budget is not TLS/RPC transport data
        // or storage authority. The final syscall is capped to remaining bytes.
        const MAX_FAULT_BYTES: usize = 8_388_736;
        std::thread::scope(|scope| {
            scope
                .spawn(move || {
                    let result = (|| {
                        if rustix::fs::fcntl_getfl(&socket)?.contains(rustix::fs::OFlags::NONBLOCK)
                        {
                            return Err(std::io::Error::other(
                                "owned duplicate unexpectedly nonblocking",
                            ));
                        }
                        let scratch = [0x5a; 8192];
                        let mut total = 0;
                        // The original cycle remains <=10s. Reserve its latter half for
                        // the measured request, cancellation and unchanged <=5s drain.
                        let fill_deadline = cycle_start + Duration::from_secs(5);
                        while total < MAX_FAULT_BYTES {
                            let remaining = fill_deadline.saturating_duration_since(Instant::now());
                            if remaining.is_zero() {
                                return Err(std::io::Error::other("raw-fault fill deadline"));
                            }
                            socket.set_write_timeout(Some(
                                remaining.min(Duration::from_millis(100)),
                            ))?;
                            let count = (MAX_FAULT_BYTES - total).min(scratch.len());
                            match socket.write(&scratch[..count]) {
                                Ok(0) => return Err(std::io::ErrorKind::WriteZero.into()),
                                Ok(bytes) => total += bytes,
                                Err(error)
                                    if matches!(
                                        error.kind(),
                                        std::io::ErrorKind::WouldBlock
                                            | std::io::ErrorKind::TimedOut
                                    ) =>
                                {
                                    return Ok(FilledDuplicate {
                                        bytes: total,
                                        blocked: error.kind(),
                                    });
                                }
                                Err(error) => return Err(error),
                            }
                        }
                        Err(std::io::Error::other(
                            "raw-fault byte ceiling exhausted without backpressure",
                        ))
                    })();
                    // Closing only this CLOEXEC descriptor does not shut down the shared
                    // connection; the real worker remains its sole transport owner.
                    drop(socket);
                    result
                })
                .join()
                .unwrap()
        })
    }

    #[test]
    fn actual_tls_write_backpressure_cancels_and_joins_with_unconsumed_replies() {
        use faction_files::private_blobs::{Binding, CHUNK, Response, encode_response};
        let (fixture, mut provider) = provider();
        let principal = provider.config.expected.clone();
        let expected = Response::Progress {
            binding: Binding {
                sender: principal.clone(),
                recipient: principal,
                id: [3; 32],
                size: 32,
                created: 1,
                expires: 100,
            },
            offset: 0,
            committed: false,
        };
        let response = encode_response(&expected).unwrap();
        let mut frames = Vec::new();
        for _ in 0..8 {
            frames.extend_from_slice(&(response.len() as u32).to_be_bytes());
            frames.extend_from_slice(&response);
        }
        assert!(frames.len() < 16_384);
        let mut peer = FaultPeer::start(&fixture, &frames);
        provider.config.address = peer.address;
        let mut completed = 0;
        let mut duplicate = None;
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
            let cycle_start = Instant::now();
            let duplicate_slot = &mut duplicate;
            let mut worker = provider.open_with(|| async {
                let socket = tokio::net::TcpStream::connect(peer.address).await?;
                let copy = rustix::io::fcntl_dupfd_cloexec(&socket, 0)?;
                assert!(rustix::io::fcntl_getfd(&copy)?.contains(rustix::io::FdFlags::CLOEXEC));
                *duplicate_slot = Some(std::net::TcpStream::from(copy));
                Ok(socket)
            }).await.unwrap();
            let observed = worker.request_observation();
            let (queued, batch_bytes) = peer.queued_single_batch();
            assert_eq!(queued, 8);
            assert_eq!(batch_bytes, frames.len());
            assert_eq!(worker.request(Request::PutChunk {id: [3; 32], offset: 0, bytes: vec![9; CHUNK]}).await.unwrap(), expected);
            completed += 1;
            assert_eq!(observed.entered.load(Ordering::SeqCst), 1);
            assert_eq!(observed.results.lock().unwrap()[0], Some(Ok(())));
            // One SSL_write shorter than 16KiB forms one authenticated TLS
            // record. A decoded first frame proves the remaining seven frames
            // are already buffered in Client TLS, rather than an absent reply.
            assert_eq!(queued - completed, 7);
            assert!(peer.child.try_wait().unwrap().is_none());
            // Synthetic raw fault bytes corrupt only this owned connection.
            // Filler joins and closes its CLOEXEC duplicate before measurement.
            let fill = fill_owned_duplicate(duplicate.take().unwrap(), cycle_start).unwrap();
            assert!(fill.bytes > 0 && fill.bytes <= 8_388_736);
            assert!(matches!(fill.blocked, std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut));
            assert!(peer.child.try_wait().unwrap().is_none());
            assert!(cycle_start.elapsed() < Duration::from_secs(5));
            let measured = Instant::now();
            let mut request = Box::pin(worker.request(Request::PutChunk {id: [3; 32], offset: 0, bytes: vec![9; CHUNK]}));
            let pending = tokio::select! {
                result = &mut request => { assert!(result.is_ok()); completed += 1; false },
                _ = tokio::time::sleep(Duration::from_millis(150)) => true,
            };
            assert!(pending && completed < 8, "actual worker request did not remain pending while timer advanced");
            assert!(measured.elapsed() >= Duration::from_millis(150));
            assert_eq!(observed.entered.load(Ordering::SeqCst), 2);
            assert_eq!(observed.results.lock().unwrap()[1], None);
            assert!(8 - completed > 0);
            assert!(peer.child.try_wait().unwrap().is_none());
            drop(request);
            assert!(worker.thread.as_ref().is_some_and(|thread| !thread.is_finished()));
            let drain = Instant::now();
            drop(worker);
            assert!(drain.elapsed() < Duration::from_secs(1));
            assert!(drain.elapsed() < Duration::from_secs(5));
            assert!(cycle_start.elapsed() < CYCLE_LIMIT);
            assert_eq!(observed.results.lock().unwrap()[1], Some(Err(Error::Transport(Stage::Cancelled))));
            assert_eq!(observed.entered.load(Ordering::SeqCst), 2);
            assert!(peer.child.try_wait().unwrap().is_none());
            eprintln!("synthetic-composed-TLS-write queued={queued} batch_bytes={batch_bytes} completed={completed} entered=2 raw_fault_bytes={} socket_block={:?} measured_ms={} drain_ms={} cycle_ms={} result=Cancelled peer_alive=true",fill.bytes,fill.blocked,measured.elapsed().as_millis(),drain.elapsed().as_millis(),cycle_start.elapsed().as_millis());
        });
        WORKER_STARTS.with(|count| assert_eq!(count.get(), 1));
        peer.stop();
    }
    #[cfg(feature = "own-home-provider")]
    #[tokio::test(flavor = "current_thread")]
    async fn full_binding_direction_is_classified_before_any_socket_or_worker() {
        use faction_files::private_blobs::Binding;
        let (_fixture, mut provider) = provider();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        provider.config.address = listener.local_addr().unwrap();
        let own = provider.config.expected.clone();
        let mut binding = Binding {
            sender: own.clone(),
            recipient: Principal {
                account: "bob@home.test".into(),
                device: 202,
                identity: [2; 32],
            },
            id: [3; 32],
            size: 32,
            created: 1,
            expires: 100,
        };
        assert_eq!(
            provider.classify(&binding, TransferDirection::Upload),
            Ok(DeviceRoute::Ordinary)
        );
        binding.recipient.account = "bob@foreign.test".into();
        assert!(
            provider
                .classify(&binding, TransferDirection::Upload)
                .is_err()
        );
        provider.config.own_home = true;
        assert_eq!(
            provider.classify(&binding, TransferDirection::Upload),
            Ok(DeviceRoute::Publication)
        );
        let reverse = Binding {
            sender: binding.recipient.clone(),
            recipient: own.clone(),
            ..binding.clone()
        };
        assert_eq!(
            provider.classify(&reverse, TransferDirection::Fetch),
            Ok(DeviceRoute::OwnHomeRetrieval)
        );
        let mut invalid = Vec::new();
        invalid.push((binding.clone(), TransferDirection::Fetch));
        invalid.push((reverse.clone(), TransferDirection::Upload));
        for field in 0..3 {
            let mut changed = binding.clone();
            match field {
                0 => changed.sender.account = "other@home.test".into(),
                1 => changed.sender.device += 1,
                _ => changed.sender.identity = [9; 32],
            }
            invalid.push((changed, TransferDirection::Upload));
            let mut changed = reverse.clone();
            match field {
                0 => changed.recipient.account = "other@home.test".into(),
                1 => changed.recipient.device += 1,
                _ => changed.recipient.identity = [9; 32],
            }
            invalid.push((changed, TransferDirection::Fetch));
        }
        let mut neither = binding.clone();
        neither.sender.account = "alice@elsewhere.test".into();
        invalid.push((neither, TransferDirection::Upload));
        for (binding, direction) in invalid {
            assert!(provider.classify(&binding, direction).is_err());
            assert!(provider.open(&binding, direction).await.is_err());
        }
        assert!(
            matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
        WORKER_STARTS.with(|count| assert_eq!(count.get(), 0));
    }

    #[cfg(feature = "own-home-provider")]
    #[tokio::test(flavor = "current_thread")]
    async fn pinned_carrier_rejects_changed_binding_direction_and_action_without_io() {
        use faction_files::private_blobs::Binding;
        let (_fixture, mut provider) = provider();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        provider.config.address = listener.local_addr().unwrap();
        provider.config.own_home = true;
        let binding = Binding {
            sender: provider.config.expected.clone(),
            recipient: Principal {
                account: "bob@foreign.test".into(),
                device: 202,
                identity: [2; 32],
            },
            id: [3; 32],
            size: 32,
            created: 1,
            expires: 100,
        };
        let mut worker = provider
            .open(&binding, TransferDirection::Upload)
            .await
            .unwrap();
        assert!(
            matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
        let mut changed = binding.clone();
        changed.recipient.identity = [9; 32];
        assert!(
            worker
                .request_bound(
                    &changed,
                    TransferDirection::Upload,
                    Request::Begin(changed.clone())
                )
                .await
                .is_err()
        );
        assert!(
            worker
                .request_bound(
                    &binding,
                    TransferDirection::Fetch,
                    Request::Status { id: binding.id }
                )
                .await
                .is_err()
        );
        assert!(
            worker
                .request_bound(
                    &binding,
                    TransferDirection::Upload,
                    Request::GetChunk {
                        id: binding.id,
                        offset: 0
                    }
                )
                .await
                .is_err()
        );
        assert!(
            worker
                .request_bound(
                    &binding,
                    TransferDirection::Upload,
                    Request::Commit { id: [8; 32] }
                )
                .await
                .is_err()
        );
        assert!(
            worker
                .request_bound(&binding, TransferDirection::Upload, Request::Begin(changed))
                .await
                .is_err()
        );
        drop(worker);
        assert!(
            matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
        WORKER_STARTS.with(|count| assert_eq!(count.get(), 1));
    }
    #[cfg(feature = "own-home-provider")]
    #[tokio::test(flavor = "current_thread")]
    async fn carrier_idle_time_before_first_command_cannot_renew_parent_deadline() {
        let (_fixture, mut provider) = provider();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        provider.config.address = listener.local_addr().unwrap();
        provider.config.own_home = true;
        let binding = Binding {
            sender: provider.config.expected.clone(),
            recipient: Principal {
                account: "bob@foreign.test".into(),
                device: 202,
                identity: [2; 32],
            },
            id: [3; 32],
            size: 32,
            created: 1,
            expires: 100,
        };
        let start = Instant::now();
        let mut worker = provider
            .open(&binding, TransferDirection::Upload)
            .await
            .unwrap();
        tokio::time::sleep(CYCLE_LIMIT + Duration::from_millis(20)).await;
        assert_eq!(
            worker
                .request_bound(
                    &binding,
                    TransferDirection::Upload,
                    Request::Begin(binding.clone())
                )
                .await,
            Err(Error::Transport(Stage::Deadline))
        );
        let drain = Instant::now();
        drop(worker);
        assert!(drain.elapsed() < Duration::from_secs(1));
        assert!(start.elapsed() >= CYCLE_LIMIT);
        assert!(
            matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
        WORKER_STARTS.with(|count| assert_eq!(count.get(), 1));
    }
    #[cfg(feature = "own-home-provider")]
    #[tokio::test(flavor = "current_thread")]
    async fn diagnostic_actual_worker_join_retains_cancelled_authenticated_exchange() {
        use std::{
            future::Future,
            io::{BufRead, Write},
            task::Poll,
        };
        let (fixture, mut provider) = provider();
        let script = r#"import pathlib, socket, ssl, sys
root = pathlib.Path(sys.argv[1])
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.minimum_version = ssl.TLSVersion.TLSv1_3
context.maximum_version = ssl.TLSVersion.TLSv1_3
context.options |= ssl.OP_NO_TICKET
context.num_tickets = 0
context.load_cert_chain(root/'cert.pem',root/'key.pem')
context.load_verify_locations(root/'cert.pem')
context.verify_mode = ssl.CERT_REQUIRED
listener = socket.socket(socket.AF_INET,socket.SOCK_STREAM)
listener.bind(('127.0.0.1',0));listener.listen(1);listener.settimeout(5)
print('PORT',listener.getsockname()[1],flush=True)
raw,address = listener.accept()
assert address[0]=='127.0.0.1'
raw.settimeout(5)
with context.wrap_socket(raw,server_side=True) as connection:
    def exact(size):
        result=b''
        while len(result)<size:
            chunk=connection.recv(size-len(result))
            assert chunk
            result+=chunk
        return result
    size=int.from_bytes(exact(4),'big')
    assert 0<size<=8192
    frame=exact(size)
    assert frame[:6]==b'FDEL01' and frame[6]==0 and frame[7]==1
    print('AUTHENTICATED_SUBMIT',flush=True)
    sys.stdin.buffer.read(1)
listener.close()
"#;
        let path = fixture.0.join("held-request.py");
        fs::write(&path, script).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let mut child = std::process::Command::new("python3")
            .arg("-I")
            .arg(&path)
            .arg(&fixture.0)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let output = std::io::BufReader::new(child.stdout.take().unwrap());
        let mut peer = FaultPeer {
            address: "127.0.0.1:1".parse().unwrap(),
            child,
            output,
        };
        let mut line = String::new();
        peer.output.read_line(&mut line).unwrap();
        peer.address
            .set_port(line.trim().strip_prefix("PORT ").unwrap().parse().unwrap());
        provider.config.address = peer.address;
        provider.config.own_home = true;
        let binding = Binding {
            sender: provider.config.expected.clone(),
            recipient: Principal {
                account: "bob@away.test".into(),
                device: 202,
                identity: [2; 32],
            },
            id: [3; 32],
            size: 32,
            created: 1,
            expires: 100,
        };
        let sink = Arc::new(std::sync::OnceLock::new());
        let mut worker = provider
            .open(&binding, TransferDirection::Upload)
            .await
            .unwrap();
        worker.observe_final(sink.clone());
        let mut request = Box::pin(worker.request_bound(
            &binding,
            TransferDirection::Upload,
            Request::Begin(binding.clone()),
        ));
        std::future::poll_fn(|cx| {
            assert!(request.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        line.clear();
        peer.output.read_line(&mut line).unwrap();
        assert_eq!(line, "AUTHENTICATED_SUBMIT\n");
        assert!(
            sink.get().is_none(),
            "no publication while the actual worker is alive"
        );
        drop(request);
        let drained = Instant::now();
        drop(worker);
        assert!(drained.elapsed() < Duration::from_secs(5));
        peer.child.stdin.as_mut().unwrap().write_all(&[1]).unwrap();
        let snapshot = sink
            .get()
            .expect("joined actual authenticated exchange must publish its final snapshot");
        assert_eq!(snapshot.state, SnapshotState::Observed);
        assert_eq!(snapshot.request, DiagnosticRequest::Begin);
        assert_eq!(
            snapshot.phase,
            faction_provider::delivery::DiagnosticPhase::SubmitExchange
        );
        assert_eq!(snapshot.transport, Some(Stage::Frame));
        assert_eq!(
            (
                snapshot.attempted,
                snapshot.completed,
                snapshot.busy,
                snapshot.connections
            ),
            (1, 0, 0, 1)
        );
        assert!(snapshot.timings.submit_connect_us > 0);
        assert!(snapshot.timings.submit_exchange_us > 0);
        assert_eq!(snapshot.timings.actual_wait_us, 0);
        assert!(peer.child.wait().unwrap().success());
    }

    #[cfg(feature = "own-home-provider")]
    #[test]
    fn diagnostic_actual_join_panic_is_unavailable_not_observed_zero() {
        let sink = Arc::new(std::sync::OnceLock::new());
        let own = Principal {
            account: "alice@home.test".into(),
            device: 101,
            identity: [1; 32],
        };
        let binding = Binding {
            sender: own.clone(),
            recipient: own,
            id: [3; 32],
            size: 32,
            created: 1,
            expires: 100,
        };
        let mut worker = Worker {
            send: None,
            thread: Some(thread::spawn(|| panic!("synthetic join failure"))),
            cancel: Arc::new(AtomicBool::new(false)),
            binding,
            direction: TransferDirection::Upload,
            deadline: Instant::now() + CYCLE_LIMIT,
            diagnostic_sink: None,
            observation: Arc::new(RequestObservation::default()),
        };
        worker.observe_final(sink.clone());
        drop(worker);
        assert_eq!(sink.get().unwrap().state, SnapshotState::JoinPanicked);
        assert_ne!(sink.get().unwrap().state, SnapshotState::Observed);
    }

    #[cfg(feature = "own-home-provider")]
    #[test]
    fn diagnostic_actual_ordinary_tls_has_no_observed_carrier_timings() {
        use faction_provider::{
            Runtime,
            authority::{CredentialBinding, Registry},
            tls::Server,
        };
        let (fixture, mut provider) = provider();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        provider.config.address = listener.local_addr().unwrap();
        provider.config.own_home = true; // Explicit own-home profile can select ordinary same-home.
        let binding = Binding {
            sender: provider.config.expected.clone(),
            recipient: Principal {
                account: "bob@home.test".into(),
                device: 202,
                identity: [2; 32],
            },
            id: [3; 32],
            size: 32,
            created: 1,
            expires: 100,
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Future-only private input capture: retain the exact already-used
        // registry clock value, never reconstruct an earlier execution's now().
        let clock_record = fixture.0.join("diagnostic-registry-seconds");
        fs::write(&clock_record, now.to_le_bytes()).unwrap();
        fs::set_permissions(&clock_record, fs::Permissions::from_mode(0o600)).unwrap();
        let registry = Registry::new(
            1,
            0,
            vec!["home.test".into()],
            vec![CredentialBinding {
                certificate_sha256: provider.config.leaf_pin,
                principal: provider.config.expected.clone(),
                valid_from: now - 60,
                valid_until: now + 3600,
                approved: true,
                account_enabled: true,
                device_enabled: true,
                generation: 1,
            }],
        )
        .unwrap();
        let runtime = Runtime::new(Some(registry));
        let pem = fs::read(fixture.0.join("cert.pem")).unwrap();
        let server =
            Server::from_pem(&pem, &fs::read(fixture.0.join("key.pem")).unwrap(), &pem).unwrap();
        let sink = Arc::new(std::sync::OnceLock::new());
        thread::scope(|scope| {
            let (entered, waiting) = std::sync::mpsc::channel();
            let (release, resume) = std::sync::mpsc::channel();
            let peer = scope.spawn(move || {
                let session = server
                    .accept(&listener, &runtime, Arc::new(AtomicBool::new(false)))
                    .unwrap();
                entered.send(()).unwrap();
                resume.recv_timeout(Duration::from_secs(5)).unwrap();
                drop(session);
            });
            let mut worker = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(provider.open(&binding, TransferDirection::Upload))
                .unwrap();
            waiting.recv_timeout(Duration::from_secs(5)).unwrap();
            worker.observe_final(sink.clone());
            drop(worker);
            release.send(()).unwrap();
            peer.join().unwrap();
            let snapshot = sink.get().unwrap();
            assert_eq!(
                snapshot.phase,
                faction_provider::delivery::DiagnosticPhase::Unobserved
            );
            assert_eq!((snapshot.attempted, snapshot.completed), (0, 0));
            assert_eq!(
                snapshot.state,
                SnapshotState::Unavailable,
                "ordinary TLS provides no observed carrier-phase timing record"
            );
        });
    }
}
