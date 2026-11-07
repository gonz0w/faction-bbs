//! Deliberate bounded encrypted attachment cycles. No background task or UI.
use crate::{
    mail_checkpoint::{CheckpointVault, Direction, Record, Wire},
    mail_network::{self, Settings},
};
use faction_files::private_blobs::{
    self as blobs, Access, Action, Binding, Principal, Request, Response,
};
use faction_omemo::{
    attachment::Descriptor,
    blob::{self, VerifiedBlob},
};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};
use zeroize::Zeroizing;
const PREFIX: &str = "urn:faction:blob:1:";
pub const CHUNKS_PER_CYCLE: usize = 4;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    State,
    Intent,
    Selection,
    Route,
    Protocol,
    Blob,
    Storage(blobs::Error),
    Transport(faction_xmpp::Error),
    Mail(mail_network::Error),
    #[cfg(feature = "device-provider")]
    Device(crate::mail_device_provider::Error),
    Save(crate::attachment_save::SaveError),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("private attachment operation unavailable")
    }
}
impl std::error::Error for Error {}
fn state<T>(r: std::io::Result<T>) -> Result<T, Error> {
    r.map_err(|_| Error::State)
}
fn store<T>(r: Result<T, blobs::Error>) -> Result<T, Error> {
    r.map_err(Error::Storage)
}
fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
fn id(d: &Descriptor) -> Result<[u8; 32], Error> {
    crate::mail_data::unhex(d.locator.strip_prefix(PREFIX).ok_or(Error::Selection)?)
        .map_err(|_| Error::Selection)
}
fn principal(v: &CheckpointVault) -> Result<Principal, Error> {
    let s = state(v.inspection())?;
    Ok(Principal {
        account: state(v.legacy())?.home.clone(),
        device: s.local_device,
        identity: *s.identity,
    })
}
fn health(v: &CheckpointVault, now: u64) -> Result<Principal, Error> {
    state(v.publication(now))?;
    let s = state(v.inspection())?;
    if !s.attachments_enabled || s.peer.is_none() {
        return Err(Error::State);
    }
    principal(v)
}
fn peer_matches(v: &CheckpointVault, r: &Record) -> Result<(), Error> {
    let s = state(v.inspection())?;
    let p = s.peer.ok_or(Error::State)?;
    if p.revoked || p.address != r.address || p.device != r.device || *p.identity != r.identity {
        return Err(Error::State);
    }
    Ok(())
}
fn binding(own: Principal, r: &Record) -> Result<Binding, Error> {
    let other = Principal {
        account: r.address.clone(),
        device: r.device,
        identity: r.identity,
    };
    let d = r.attachment().ok_or(Error::Selection)?;
    let (sender, recipient) = if r.direction == Direction::Sent {
        (own, other)
    } else {
        (other, own)
    };
    let b = Binding {
        sender,
        recipient,
        id: id(d)?,
        size: d.size.checked_add(16).ok_or(Error::Selection)?,
        created: r.sent_at,
        expires: r
            .sent_at
            .checked_add(blobs::MAX_RETENTION)
            .ok_or(Error::Selection)?,
    };
    store(b.validate())?;
    Ok(b)
}
fn local_binding(own: &Principal, b: &Binding) -> Binding {
    Binding {
        sender: own.clone(),
        recipient: own.clone(),
        ..b.clone()
    }
}
struct Local<'a>(&'a Principal);
impl Access for Local<'_> {
    fn require(&self, a: &Principal, b: &Binding, _: Action, _: u64) -> Result<(), blobs::Error> {
        if a == self.0 && b.sender == *self.0 && b.recipient == *self.0 {
            Ok(())
        } else {
            Err(blobs::Error::Denied)
        }
    }
}
/// A separate member-local ciphertext directory; eight objects, seven days max.
/// Uses the storage implementation's private no-follow paths, lock and recovery.
pub struct Spool {
    store: blobs::Store,
}
impl Spool {
    pub fn open(path: &Path) -> Result<Self, Error> {
        use rustix::fs::{self, Mode, OFlags};
        use std::{fs::File, path::Component};
        if !path.is_absolute() {
            return Err(Error::Selection);
        }
        let mut dir = File::from(
            fs::open(
                "/",
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|_| Error::State)?,
        );
        let parts: Vec<_> = path.components().collect();
        for (i, p) in parts.iter().enumerate() {
            match p {
                Component::RootDir => (),
                Component::Normal(n) => {
                    if i + 1 == parts.len() {
                        match fs::mkdirat(&dir, *n, Mode::from_raw_mode(0o700)) {
                            Ok(()) => state(dir.sync_all())?,
                            Err(e) if e == rustix::io::Errno::EXIST => (),
                            Err(_) => return Err(Error::State),
                        }
                    }
                    dir = File::from(
                        fs::openat(
                            &dir,
                            *n,
                            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                            Mode::empty(),
                        )
                        .map_err(|_| Error::State)?,
                    );
                }
                _ => return Err(Error::Selection),
            }
        }
        Ok(Self {
            store: store(blobs::Store::open(path))?,
        })
    }
    fn call(&mut self, own: &Principal, request: Request, now: u64) -> Result<Response, Error> {
        store(self.store.dispatch(own, &Local(own), request, now))
    }
    fn begin(&mut self, own: &Principal, b: &Binding, now: u64) -> Result<(u64, bool), Error> {
        progress(
            self.call(own, Request::Begin(local_binding(own, b)), now)?,
            &local_binding(own, b),
        )
    }
    fn bytes(&mut self, own: &Principal, b: &Binding, now: u64) -> Result<Vec<u8>, Error> {
        let local = local_binding(own, b);
        let (offset, committed) =
            progress(self.call(own, Request::Status { id: b.id }, now)?, &local)?;
        if !committed || offset != b.size {
            return Err(Error::State);
        }
        let mut bytes = Vec::with_capacity(b.size as usize);
        while bytes.len() < (b.size as usize) {
            match self.call(
                own,
                Request::GetChunk {
                    id: b.id,
                    offset: bytes.len() as u64,
                },
                now,
            )? {
                Response::Chunk {
                    offset,
                    bytes: chunk,
                    total,
                } if offset == bytes.len() as u64
                    && total == b.size
                    && !chunk.is_empty()
                    && chunk.len() <= blobs::CHUNK
                    && bytes.len() + chunk.len() <= b.size as usize =>
                {
                    bytes.extend(chunk)
                }
                _ => return Err(Error::Protocol),
            }
        }
        if digest(&bytes) != b.id {
            return Err(Error::Blob);
        }
        Ok(bytes)
    }
    /// Explicit retirement also works after expiry. Downloaded copies elsewhere remain.
    pub fn delete(&mut self, vault: &CheckpointVault, id: [u8; 32], now: u64) -> Result<(), Error> {
        let own = principal(vault)?;
        match self.call(&own, Request::Delete { id }, now)? {
            Response::Deleted => Ok(()),
            _ => Err(Error::Protocol),
        }
    }
}
/// Trusted local bytes and intent. The API never derives a source path from mail.
pub struct Selection<'a> {
    pub name: &'a str,
    pub media_type: &'a str,
    pub bytes: &'a [u8],
    pub subject: &'a str,
    pub body: &'a str,
    pub sent_at: u64,
}
/// Encrypt once, publish ciphertext durably, then queue the descriptor atomically.
/// A crash before queue may leave an opaque orphan; quotas bound it. No key sidecar.
pub fn prepare(
    vault: &mut CheckpointVault,
    spool: &mut Spool,
    operation: [u8; 16],
    selected: Selection<'_>,
    now: u64,
) -> Result<Wire, Error> {
    let own = health(vault, now)?;
    if operation == [0; 16] || selected.bytes.len() > blob::MAX_BLOB_BYTES {
        return Err(Error::Intent);
    }
    if let Some(r) = state(vault.history())?
        .iter()
        .find(|r| r.direction == Direction::Sent && r.operation() == operation)
    {
        peer_matches(vault, r)?;
        let d = r.attachment().ok_or(Error::Intent)?;
        if d.name.as_str() != selected.name
            || d.media_type.as_str() != selected.media_type
            || r.subject.as_str() != selected.subject
            || r.body.as_str() != selected.body
            || r.sent_at != selected.sent_at
        {
            return Err(Error::Intent);
        }
        let b = binding(own, r)?;
        let bytes = spool.bytes(&b.sender, &b, now)?;
        let verified = blob::verify(&bytes, d).map_err(|_| Error::Blob)?;
        if verified.as_bytes() != selected.bytes {
            return Err(Error::Intent);
        }
        let descriptor = d_copy(d);
        return state(vault.queue_attachment(
            operation,
            selected.subject,
            selected.body,
            &descriptor,
            selected.sent_at,
            now,
        ));
    }
    let encrypted = blob::encrypt(selected.bytes).map_err(|_| Error::Blob)?;
    let cipher_id = digest(&encrypted.ciphertext);
    let d = Descriptor {
        name: Zeroizing::new(selected.name.into()),
        media_type: Zeroizing::new(selected.media_type.into()),
        size: encrypted.size,
        hash: encrypted.hash,
        key: encrypted.key,
        iv: encrypted.iv,
        locator: Zeroizing::new(format!("{PREFIX}{}", crate::mail_data::hex(&cipher_id))),
    };
    d.validate().map_err(|_| Error::Intent)?;
    let s = state(vault.inspection())?;
    let p = s.peer.ok_or(Error::State)?;
    let b = Binding {
        sender: own.clone(),
        recipient: Principal {
            account: p.address.into(),
            device: p.device,
            identity: *p.identity,
        },
        id: cipher_id,
        size: encrypted.ciphertext.len() as u64,
        created: selected.sent_at,
        expires: selected
            .sent_at
            .checked_add(blobs::MAX_RETENTION)
            .ok_or(Error::Intent)?,
    };
    store(b.validate())?;
    spool.begin(&own, &b, now)?;
    for (offset, chunk) in encrypted.ciphertext.chunks(blobs::CHUNK).enumerate() {
        spool.call(
            &own,
            Request::PutChunk {
                id: cipher_id,
                offset: (offset * blobs::CHUNK) as u64,
                bytes: chunk.to_vec(),
            },
            now,
        )?;
    }
    progress(
        spool.call(&own, Request::Commit { id: cipher_id }, now)?,
        &local_binding(&own, &b),
    )?;
    state(vault.queue_attachment(
        operation,
        selected.subject,
        selected.body,
        &d,
        selected.sent_at,
        now,
    ))
}
fn d_copy(d: &Descriptor) -> Descriptor {
    Descriptor {
        name: Zeroizing::new(d.name.to_string()),
        media_type: Zeroizing::new(d.media_type.to_string()),
        size: d.size,
        hash: Zeroizing::new(*d.hash),
        key: Zeroizing::new(*d.key),
        iv: Zeroizing::new(*d.iv),
        locator: Zeroizing::new(d.locator.to_string()),
    }
}
fn progress(r: Response, expected: &Binding) -> Result<(u64, bool), Error> {
    match r {
        Response::Progress {
            binding,
            offset,
            committed,
        } if binding == *expected
            && offset <= binding.size
            && (!committed || offset == binding.size) =>
        {
            Ok((offset, committed))
        }
        _ => Err(Error::Protocol),
    }
}
/// Explicit invited sender homes. A locator cannot supply a network destination.
pub struct InvitedHomes {
    homes: Vec<String>,
}
impl InvitedHomes {
    pub fn new(homes: &[&str]) -> Result<Self, Error> {
        if homes.len() > 32 {
            return Err(Error::Route);
        }
        let mut out = Vec::new();
        for h in homes {
            if !crate::mail_data::address(&format!("service@{h}"))
                || !h.contains('.')
                || out.iter().any(|x| x == h)
            {
                return Err(Error::Route);
            }
            out.push(h.to_string());
        }
        Ok(Self { homes: out })
    }
}
fn home(account: &str) -> Result<&str, Error> {
    account.split_once('@').map(|x| x.1).ok_or(Error::Route)
}
struct Clock {
    epoch: u64,
    start: Instant,
}
impl Clock {
    fn new(epoch: u64) -> Self {
        Self {
            epoch,
            start: Instant::now(),
        }
    }
    fn now(&self) -> u64 {
        self.epoch.saturating_add(self.start.elapsed().as_secs())
    }
}
fn network(v: &CheckpointVault, settings: &Settings, now: u64) -> Result<Principal, Error> {
    let own = health(v, now)?;
    if own.account != settings.account {
        return Err(Error::State);
    }
    Ok(own)
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TransferDirection {
    Upload,
    Fetch,
}
#[derive(Clone, Copy)]
enum TransferRoute<'a> {
    Iq(&'a Settings),
    #[cfg(feature = "device-provider")]
    Device(
        &'a Settings,
        &'a crate::mail_device_provider::DeviceProvider,
        #[cfg(feature = "own-home-provider")] Option<&'a CycleDiagnostics>,
    ),
}
enum Carrier {
    Iq(Box<faction_xmpp::Session>),
    #[cfg(feature = "device-provider")]
    Device(Box<crate::mail_device_provider::Worker>),
}
impl TransferRoute<'_> {
    fn settings(&self) -> &Settings {
        match self {
            Self::Iq(settings) => settings,
            #[cfg(feature = "device-provider")]
            Self::Device(settings, ..) => settings,
        }
    }
    fn check(&self, vault: &CheckpointVault, now: u64) -> Result<Principal, Error> {
        let own = network(vault, self.settings(), now)?;
        #[cfg(feature = "device-provider")]
        if let Self::Device(_, provider, ..) = self {
            provider.validate(vault).map_err(Error::Device)?;
        }
        Ok(own)
    }
    fn binding(&self, binding: &Binding, direction: TransferDirection) -> Result<(), Error> {
        #[cfg(feature = "device-provider")]
        if let Self::Device(_, provider, ..) = self
            && !provider.permits_binding(binding, direction)
        {
            return Err(Error::Route);
        }
        #[cfg(not(feature = "device-provider"))]
        let _ = (binding, direction);
        Ok(())
    }
    fn own_home(&self) -> bool {
        match self {
            Self::Iq(_) => false,
            #[cfg(feature = "device-provider")]
            Self::Device(_, provider, ..) => provider.uses_own_home(),
        }
    }
    async fn connect(
        &self,
        binding: &Binding,
        direction: TransferDirection,
        device: u32,
        password: &str,
    ) -> Result<Carrier, Error> {
        let _ = (binding, direction);
        match self {
            Self::Iq(settings) => {
                let transport = settings.transport(device).map_err(Error::Transport)?;
                faction_xmpp::connect(&transport, password)
                    .await
                    .map(Box::new)
                    .map(Carrier::Iq)
                    .map_err(Error::Transport)
            }
            #[cfg(feature = "device-provider")]
            Self::Device(_, provider, ..) => {
                let worker = provider
                    .open(binding, direction)
                    .await
                    .map_err(Error::Device)?;
                #[cfg(feature = "own-home-provider")]
                let mut worker = worker;
                #[cfg(feature = "own-home-provider")]
                if let Self::Device(_, _, Some(diagnostics)) = self {
                    worker.observe_final(diagnostics.sink.clone());
                }
                Ok(Carrier::Device(Box::new(worker)))
            }
        }
    }
}
#[derive(Clone, Copy)]
struct RequestContext<'a> {
    route: TransferRoute<'a>,
    service: &'a str,
    binding: &'a Binding,
    direction: TransferDirection,
}
async fn request(
    session: &mut Carrier,
    v: &CheckpointVault,
    context: RequestContext<'_>,
    request: Request,
    count: &mut usize,
    clock: &Clock,
) -> Result<Response, Error> {
    let RequestContext {
        route,
        service,
        binding,
        direction,
    } = context;
    route.check(v, clock.now())?;
    let _ = (binding, direction);
    if *count == 8 {
        return Err(Error::Protocol);
    }
    *count += 1;
    let response = match session {
        Carrier::Iq(session) => {
            let id = format!("blob-{}", *count);
            let settings = route.settings();
            let xml = crate::mail_blob_wire::request(&id, &settings.account, service, &request)
                .map_err(|_| Error::Protocol)?;
            session.send(&xml).await.map_err(Error::Transport)?;
            let response = session.next().await.map_err(Error::Transport)?;
            route.check(v, clock.now())?;
            crate::mail_blob_wire::decode_response(&response, &id, &settings.account, service)
                .map_err(|error| match error {
                    crate::mail_blob_wire::Error::Remote(error) => Error::Storage(error),
                    _ => Error::Protocol,
                })?
        }
        #[cfg(feature = "device-provider")]
        Carrier::Device(worker) => worker
            .request_bound(binding, direction, request)
            .await
            .map_err(Error::Device)?,
    };
    route.check(v, clock.now())?;
    Ok(response)
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Report {
    pub offset: u64,
    pub complete: bool,
    pub requests: usize,
    pub sent: usize,
}
/// Internal capability minted only after a pinned authenticated storage reply.
pub(crate) struct Release {
    binding: Binding,
    operation: [u8; 16],
    wire: Wire,
}
impl Release {
    pub(crate) fn permits(&self, v: &CheckpointVault, op: [u8; 16], wire: &Wire, now: u64) -> bool {
        if op != self.operation
            || wire != &self.wire
            || now < self.binding.created
            || now >= self.binding.expires
        {
            return false;
        }
        let Ok(own) = health(v, now) else {
            return false;
        };
        let Ok(rows) = v.history() else { return false };
        rows.iter()
            .find(|r| r.direction == Direction::Sent && r.operation() == op)
            .is_some_and(|r| {
                peer_matches(v, r).is_ok() && binding(own, r).is_ok_and(|b| b == self.binding)
            })
    }
}
pub async fn upload_cycle(
    vault: &mut CheckpointVault,
    spool: &mut Spool,
    settings: &Settings,
    password: &str,
    operation: [u8; 16],
    now: u64,
) -> Result<Report, Error> {
    within_cycle(
        std::time::Duration::from_secs(10),
        upload_inner(
            vault,
            spool,
            TransferRoute::Iq(settings),
            password,
            operation,
            now,
        ),
    )
    .await
}
/// Explicit authenticated device transport. Any failure stays on this route;
/// ordinary XMPP is used only for the existing separate encrypted-mail release.
#[cfg(feature = "device-provider")]
pub async fn upload_device_cycle(
    vault: &mut CheckpointVault,
    spool: &mut Spool,
    settings: &Settings,
    password: &str,
    provider: &crate::mail_device_provider::DeviceProvider,
    operation: [u8; 16],
    now: u64,
) -> Result<Report, Error> {
    within_cycle(
        std::time::Duration::from_secs(10),
        upload_inner(
            vault,
            spool,
            TransferRoute::Device(
                settings,
                provider,
                #[cfg(feature = "own-home-provider")]
                None,
            ),
            password,
            operation,
            now,
        ),
    )
    .await
}
/// Received ciphertext through the explicitly selected device route, without an
/// account password. V1 remains same-home; own-home selection requires the home
/// to authenticate and authorize its configured peer before returning bytes.
#[cfg(feature = "device-provider")]
pub async fn fetch_device_cycle(
    vault: &CheckpointVault,
    spool: &mut Spool,
    settings: &Settings,
    provider: &crate::mail_device_provider::DeviceProvider,
    selected: ReceivedSelection,
    now: u64,
) -> Result<Report, Error> {
    let invited = InvitedHomes { homes: Vec::new() };
    within_cycle(
        std::time::Duration::from_secs(10),
        fetch_inner(
            vault,
            spool,
            TransferRoute::Device(
                settings,
                provider,
                #[cfg(feature = "own-home-provider")]
                None,
            ),
            "",
            &invited,
            selected,
            now,
        ),
    )
    .await
}
#[cfg(feature = "own-home-provider")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum StopOrigin {
    #[default]
    Unobserved,
    ForegroundDeadline,
    SdkDeadline,
    CancelBranch,
    SdkError,
    ProgressBound,
}
#[cfg(feature = "own-home-provider")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum DiagnosticError {
    #[default]
    Unavailable,
    State,
    Intent,
    Selection,
    Route,
    Protocol,
    Blob,
    Storage,
    Xmpp,
    Mail,
    Save,
    DeviceConfiguration,
    DeviceIdentity,
    DeviceKeylog,
    DeviceTransport,
    Worker,
}
/// Local failure attribution only: no credentials, content, route or authority.
#[cfg(feature = "own-home-provider")]
#[derive(Default)]
pub struct CycleDiagnostics {
    sink: std::sync::Arc<std::sync::OnceLock<crate::mail_device_provider::WorkerSnapshot>>,
    origin: std::sync::atomic::AtomicU8,
    error: std::sync::atomic::AtomicU8,
}
#[cfg(feature = "own-home-provider")]
impl CycleDiagnostics {
    pub fn stop_origin(&self) -> StopOrigin {
        [
            StopOrigin::Unobserved,
            StopOrigin::ForegroundDeadline,
            StopOrigin::SdkDeadline,
            StopOrigin::CancelBranch,
            StopOrigin::SdkError,
            StopOrigin::ProgressBound,
        ]
        .get(usize::from(
            self.origin.load(std::sync::atomic::Ordering::SeqCst),
        ))
        .copied()
        .unwrap_or_default()
    }
    pub fn error_kind(&self) -> DiagnosticError {
        [
            DiagnosticError::Unavailable,
            DiagnosticError::State,
            DiagnosticError::Intent,
            DiagnosticError::Selection,
            DiagnosticError::Route,
            DiagnosticError::Protocol,
            DiagnosticError::Blob,
            DiagnosticError::Storage,
            DiagnosticError::Xmpp,
            DiagnosticError::Mail,
            DiagnosticError::Save,
            DiagnosticError::DeviceConfiguration,
            DiagnosticError::DeviceIdentity,
            DiagnosticError::DeviceKeylog,
            DiagnosticError::DeviceTransport,
            DiagnosticError::Worker,
        ]
        .get(usize::from(
            self.error.load(std::sync::atomic::Ordering::SeqCst),
        ))
        .copied()
        .unwrap_or_default()
    }
    pub fn snapshot(&self) -> Option<&crate::mail_device_provider::WorkerSnapshot> {
        self.sink.get()
    }
    pub fn record_stop(&self, origin: StopOrigin) {
        self.origin
            .store(origin as u8, std::sync::atomic::Ordering::SeqCst);
    }
    pub fn record_error(&self, error: &Error) {
        self.error.store(
            diagnostic_error(error) as u8,
            std::sync::atomic::Ordering::SeqCst,
        );
        self.record_stop(StopOrigin::SdkError);
    }
}
#[cfg(feature = "own-home-provider")]
#[derive(Default)]
pub struct ForegroundDiagnostics {
    timings: faction_provider::delivery::DiagnosticTimings,
    logical_completed: u16,
    logical_attempted: u16,
    carrier_attempts: u16,
    busy: u16,
    connect_calls: u16,
    unavailable_cycles: u8,
    cycles: u8,
    invalid: bool,
    clamped: bool,
}
#[cfg(feature = "own-home-provider")]
impl ForegroundDiagnostics {
    pub fn accept_cycle(&mut self, cycle: &CycleDiagnostics) {
        if self.cycles >= 65 {
            self.invalid = true;
            return;
        }
        self.cycles += 1;
        if let Some(snapshot) = cycle
            .snapshot()
            .filter(|s| s.state == crate::mail_device_provider::SnapshotState::Observed)
        {
            if snapshot.attempted > 8
                || snapshot.completed > snapshot.attempted
                || snapshot.carrier_attempts > 64
                || snapshot.busy > 64
                || snapshot.connections > 64
                || self.logical_completed + u16::from(snapshot.completed) > 520
                || self.logical_attempted + u16::from(snapshot.attempted) > 520
                || self.carrier_attempts + u16::from(snapshot.carrier_attempts) > 4160
                || self.busy + u16::from(snapshot.busy) > 4160
                || self.connect_calls + u16::from(snapshot.connections) > 4160
            {
                self.invalid = true;
                return;
            }
            self.clamped |= self
                .timings
                .submit_connect_us
                .checked_add(snapshot.timings.submit_connect_us)
                .is_none()
                || self
                    .timings
                    .submit_exchange_us
                    .checked_add(snapshot.timings.submit_exchange_us)
                    .is_none()
                || self
                    .timings
                    .collect_connect_us
                    .checked_add(snapshot.timings.collect_connect_us)
                    .is_none()
                || self
                    .timings
                    .collect_exchange_us
                    .checked_add(snapshot.timings.collect_exchange_us)
                    .is_none()
                || self
                    .timings
                    .planned_wait_us
                    .checked_add(snapshot.timings.planned_wait_us)
                    .is_none()
                || self
                    .timings
                    .actual_wait_us
                    .checked_add(snapshot.timings.actual_wait_us)
                    .is_none();
            self.timings = self.timings.saturating_add(snapshot.timings);
            self.logical_completed += u16::from(snapshot.completed);
            self.logical_attempted += u16::from(snapshot.attempted);
            self.carrier_attempts += u16::from(snapshot.carrier_attempts);
            self.busy += u16::from(snapshot.busy);
            self.connect_calls += u16::from(snapshot.connections);
        } else {
            self.unavailable_cycles = self.unavailable_cycles.saturating_add(1);
        }
    }
    /// One failure-only local line. Last phase is not proof I/O remains pending;
    /// connect_calls counts entries, not successful sockets or TLS handshakes.
    /// Phase elapsed overlaps cumulative timings, and planned wait is an estimate.
    pub fn failure_line(
        &self,
        cycle: Option<&CycleDiagnostics>,
        origin: StopOrigin,
        fetch: bool,
        sdk_completed: usize,
        offset: u64,
        elapsed: std::time::Duration,
    ) -> String {
        use crate::mail_device_provider::{DiagnosticRequest, SnapshotState};
        use faction_provider::delivery::DiagnosticPhase;
        let raw = cycle.and_then(CycleDiagnostics::snapshot);
        let snapshot = raw.filter(|s| s.state == SnapshotState::Observed);
        let state = match raw.map(|s| s.state) {
            Some(SnapshotState::Observed) => "observed",
            Some(SnapshotState::JoinPanicked) => "join_panicked",
            _ => "unavailable",
        };
        let request = match snapshot.map(|s| s.request) {
            Some(DiagnosticRequest::Begin) => "begin",
            Some(DiagnosticRequest::Status) => "status",
            Some(DiagnosticRequest::Put) => "put",
            Some(DiagnosticRequest::Commit) => "commit",
            Some(DiagnosticRequest::Get) => "get",
            Some(DiagnosticRequest::Delete) => "delete",
            _ => "unavailable",
        };
        let phase = match snapshot.map(|s| s.phase) {
            Some(DiagnosticPhase::SubmitConnect) => "submit_connect",
            Some(DiagnosticPhase::SubmitExchange) => "submit_exchange",
            Some(DiagnosticPhase::CollectWait) => "collect_wait",
            Some(DiagnosticPhase::CollectConnect) => "collect_connect",
            Some(DiagnosticPhase::CollectExchange) => "collect_exchange",
            Some(DiagnosticPhase::Idle) => "idle",
            _ => "unavailable",
        };
        let origin = match origin {
            StopOrigin::ForegroundDeadline => "foreground_deadline",
            StopOrigin::SdkDeadline => "sdk_deadline",
            StopOrigin::CancelBranch => "cancel_branch",
            StopOrigin::SdkError => "sdk_error",
            StopOrigin::ProgressBound => "progress_bound",
            StopOrigin::Unobserved => "unavailable",
        };
        let error = match cycle.map(CycleDiagnostics::error_kind).unwrap_or_default() {
            DiagnosticError::Unavailable => "unavailable",
            DiagnosticError::State => "state",
            DiagnosticError::Intent => "intent",
            DiagnosticError::Selection => "selection",
            DiagnosticError::Route => "route",
            DiagnosticError::Protocol => "protocol",
            DiagnosticError::Blob => "blob",
            DiagnosticError::Storage => "storage",
            DiagnosticError::Xmpp => "xmpp",
            DiagnosticError::Mail => "mail",
            DiagnosticError::Save => "save",
            DiagnosticError::DeviceConfiguration => "device_configuration",
            DiagnosticError::DeviceIdentity => "device_identity",
            DiagnosticError::DeviceKeylog => "device_keylog",
            DiagnosticError::DeviceTransport => "device_transport",
            DiagnosticError::Worker => "worker",
        };
        let transport = match snapshot.and_then(|s| s.transport) {
            Some(faction_provider::tls::Stage::Configuration) => "configuration",
            Some(faction_provider::tls::Stage::Admission) => "admission",
            Some(faction_provider::tls::Stage::Handshake) => "handshake",
            Some(faction_provider::tls::Stage::Pin) => "pin",
            Some(faction_provider::tls::Stage::Denied) => "denied",
            Some(faction_provider::tls::Stage::Frame) => "frame",
            Some(faction_provider::tls::Stage::Limit) => "limit",
            Some(faction_provider::tls::Stage::Deadline) => "deadline",
            Some(faction_provider::tls::Stage::Cancelled) => "cancelled",
            Some(faction_provider::tls::Stage::Complete) => "complete",
            None => "unavailable",
        };
        let number = |value: Option<u64>| {
            value
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unavailable".into())
        };
        let exchange_origin = match snapshot.map(|s| s.exchange_origin) {
            Some(faction_provider::delivery::ExchangeOrigin::Write) => "write",
            Some(faction_provider::delivery::ExchangeOrigin::ReadError) => "read_error",
            Some(faction_provider::delivery::ExchangeOrigin::ReadEof) => "read_eof",
            Some(faction_provider::delivery::ExchangeOrigin::Decode) => "decode",
            Some(faction_provider::delivery::ExchangeOrigin::Refused) => "refused",
            Some(faction_provider::delivery::ExchangeOrigin::Unobserved) | None => "unobserved",
        };
        format!(
            "Attachment diagnostic v2 direction={} origin={} error={} state={} request={} last_phase={} transport={} sdk_completed={} sdk_active={} aggregate_valid={} totals_clamped={} observed_logical_completed={} observed_logical_attempted={} observed_carrier_attempts={} observed_busy={} observed_connect_calls={} unavailable_cycles={} cycle_completed={} cycle_logical_attempted={} cycle_carrier_attempts={} cycle_busy={} cycle_connect_calls={} confirmed_offset={} foreground_us={} last_phase_us={} observed_submit_connect_us={} observed_submit_exchange_us={} observed_collect_connect_us={} observed_collect_exchange_us={} observed_planned_wait_us={} observed_actual_wait_us={} terminal_carrier_attempts={} terminal_busy={} exchange_origin={}\r\n",
            if fetch { "fetch" } else { "upload" },
            origin,
            error,
            state,
            request,
            phase,
            transport,
            sdk_completed,
            if cycle.is_some() {
                sdk_completed.saturating_add(1).min(65)
            } else {
                0
            },
            !self.invalid,
            self.clamped,
            number((!self.invalid).then_some(u64::from(self.logical_completed))),
            number((!self.invalid).then_some(u64::from(self.logical_attempted))),
            number((!self.invalid).then_some(u64::from(self.carrier_attempts))),
            number((!self.invalid).then_some(u64::from(self.busy))),
            number((!self.invalid).then_some(u64::from(self.connect_calls))),
            self.unavailable_cycles,
            number(snapshot.map(|s| u64::from(s.completed))),
            number(snapshot.map(|s| u64::from(s.attempted))),
            number(snapshot.map(|s| u64::from(s.carrier_attempts))),
            number(snapshot.map(|s| u64::from(s.busy))),
            number(snapshot.map(|s| u64::from(s.connections))),
            offset,
            u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX),
            number(snapshot.map(|s| s.phase_us)),
            number((!self.invalid).then_some(self.timings.submit_connect_us)),
            number((!self.invalid).then_some(self.timings.submit_exchange_us)),
            number((!self.invalid).then_some(self.timings.collect_connect_us)),
            number((!self.invalid).then_some(self.timings.collect_exchange_us)),
            number((!self.invalid).then_some(self.timings.planned_wait_us)),
            number((!self.invalid).then_some(self.timings.actual_wait_us)),
            number(snapshot.map(|s| u64::from(s.terminal_carrier_attempts))),
            number(snapshot.map(|s| u64::from(s.terminal_busy))),
            exchange_origin,
        )
    }
}
#[cfg(feature = "own-home-provider")]
fn diagnostic_error(error: &Error) -> DiagnosticError {
    match error {
        Error::State => DiagnosticError::State,
        Error::Intent => DiagnosticError::Intent,
        Error::Selection => DiagnosticError::Selection,
        Error::Route => DiagnosticError::Route,
        Error::Protocol => DiagnosticError::Protocol,
        Error::Blob => DiagnosticError::Blob,
        Error::Storage(_) => DiagnosticError::Storage,
        Error::Transport(_) => DiagnosticError::Xmpp,
        Error::Mail(_) => DiagnosticError::Mail,
        Error::Save(_) => DiagnosticError::Save,
        Error::Device(crate::mail_device_provider::Error::Configuration) => {
            DiagnosticError::DeviceConfiguration
        }
        Error::Device(crate::mail_device_provider::Error::Identity) => {
            DiagnosticError::DeviceIdentity
        }
        Error::Device(crate::mail_device_provider::Error::Keylog) => DiagnosticError::DeviceKeylog,
        Error::Device(crate::mail_device_provider::Error::Transport(_)) => {
            DiagnosticError::DeviceTransport
        }
        Error::Device(crate::mail_device_provider::Error::Worker) => DiagnosticError::Worker,
    }
}
#[cfg(feature = "own-home-provider")]
async fn within_cycle_observed<T>(
    duration: std::time::Duration,
    future: impl std::future::Future<Output = Result<T, Error>>,
    diagnostics: &CycleDiagnostics,
) -> Result<T, Error> {
    match tokio::time::timeout(duration, future).await {
        Ok(result) => {
            if let Err(error) = &result {
                diagnostics.record_error(error);
            }
            result
        }
        Err(_) => {
            diagnostics.record_stop(StopOrigin::SdkDeadline);
            Err(Error::Transport(faction_xmpp::Error::Timeout))
        }
    }
}
/// Optional local observations do not change the selected device or grant authority.
#[cfg(feature = "own-home-provider")]
pub struct ObservedDevice<'a> {
    pub provider: &'a crate::mail_device_provider::DeviceProvider,
    pub diagnostics: &'a CycleDiagnostics,
}
#[cfg(feature = "own-home-provider")]
pub async fn fetch_device_cycle_observed(
    vault: &CheckpointVault,
    spool: &mut Spool,
    settings: &Settings,
    observed: ObservedDevice<'_>,
    selected: ReceivedSelection,
    now: u64,
) -> Result<Report, Error> {
    let invited = InvitedHomes { homes: Vec::new() };
    within_cycle_observed(
        std::time::Duration::from_secs(10),
        fetch_inner(
            vault,
            spool,
            TransferRoute::Device(settings, observed.provider, Some(observed.diagnostics)),
            "",
            &invited,
            selected,
            now,
        ),
        observed.diagnostics,
    )
    .await
}
#[cfg(feature = "own-home-provider")]
pub async fn upload_device_cycle_observed(
    vault: &mut CheckpointVault,
    spool: &mut Spool,
    settings: &Settings,
    password: &str,
    observed: ObservedDevice<'_>,
    operation: [u8; 16],
    now: u64,
) -> Result<Report, Error> {
    within_cycle_observed(
        std::time::Duration::from_secs(10),
        upload_inner(
            vault,
            spool,
            TransferRoute::Device(settings, observed.provider, Some(observed.diagnostics)),
            password,
            operation,
            now,
        ),
        observed.diagnostics,
    )
    .await
}
async fn within_cycle<T>(
    duration: std::time::Duration,
    future: impl std::future::Future<Output = Result<T, Error>>,
) -> Result<T, Error> {
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| Error::Transport(faction_xmpp::Error::Timeout))?
}
async fn upload_inner(
    vault: &mut CheckpointVault,
    spool: &mut Spool,
    route: TransferRoute<'_>,
    password: &str,
    operation: [u8; 16],
    now: u64,
) -> Result<Report, Error> {
    let clock = Clock::new(now);
    let own = route.check(vault, clock.now())?;
    let r = state(vault.history())?
        .iter()
        .find(|r| r.direction == Direction::Sent && r.operation() == operation)
        .ok_or(Error::Selection)?;
    peer_matches(vault, r)?;
    let b = binding(own.clone(), r)?;
    route.binding(&b, TransferDirection::Upload)?;
    let wire = state(vault.pending_deliveries(clock.now()))?
        .into_iter()
        .find(|p| p.operation == operation)
        .ok_or(Error::Selection)?
        .wire;
    let bytes = spool.bytes(&own, &b, clock.now())?;
    let mut session = route
        .connect(&b, TransferDirection::Upload, own.device, password)
        .await?;
    let service = home(&own.account)?;
    let mut requests = 0;
    // Begin is an exact immutable idempotent status/create operation. Remote
    // errors deliberately conceal whether an object exists or why it refused.
    let response = request(
        &mut session,
        vault,
        RequestContext {
            route,
            service,
            binding: &b,
            direction: TransferDirection::Upload,
        },
        Request::Begin(b.clone()),
        &mut requests,
        &clock,
    )
    .await?;
    let (mut offset, mut committed) = progress(response, &b)?;
    for _ in 0..CHUNKS_PER_CYCLE {
        if committed || offset == b.size {
            break;
        }
        let end = (offset as usize + blobs::CHUNK).min(bytes.len());
        let expected = end as u64;
        let response = request(
            &mut session,
            vault,
            RequestContext {
                route,
                service,
                binding: &b,
                direction: TransferDirection::Upload,
            },
            Request::PutChunk {
                id: b.id,
                offset,
                bytes: bytes[offset as usize..end].to_vec(),
            },
            &mut requests,
            &clock,
        )
        .await?;
        let (next, ready) = progress(response, &b)?;
        if next != expected || ready {
            return Err(Error::Protocol);
        }
        offset = next;
    }
    if !committed && offset == b.size {
        let response = request(
            &mut session,
            vault,
            RequestContext {
                route,
                service,
                binding: &b,
                direction: TransferDirection::Upload,
            },
            Request::Commit { id: b.id },
            &mut requests,
            &clock,
        )
        .await?;
        let (next, ready) = progress(response, &b)?;
        offset = next;
        committed = ready;
        if !committed {
            return Err(Error::Protocol);
        }
    }
    drop(session);
    let sent = if committed {
        let release = Release {
            binding: b,
            operation,
            wire,
        };
        mail_network::release_attachment(vault, route.settings(), password, &release, clock.now())
            .await
            .map_err(Error::Mail)?
            .sent
    } else {
        0
    };
    Ok(Report {
        offset,
        complete: committed,
        requests,
        sent,
    })
}
fn received(v: &CheckpointVault, index: usize, frame: [u8; 32]) -> Result<&Record, Error> {
    let r = state(v.history())?.get(index).ok_or(Error::Selection)?;
    if r.direction != Direction::Received
        || r.received_id() != Some(frame)
        || r.attachment().is_none()
    {
        return Err(Error::Selection);
    }
    Ok(r)
}
/// Exact authenticated local received-history selection; never a remote claim.
#[derive(Clone, Copy)]
pub struct ReceivedSelection {
    pub index: usize,
    pub frame: [u8; 32],
}
pub async fn fetch_cycle(
    vault: &CheckpointVault,
    spool: &mut Spool,
    settings: &Settings,
    password: &str,
    invited: &InvitedHomes,
    selected: ReceivedSelection,
    now: u64,
) -> Result<Report, Error> {
    within_cycle(
        std::time::Duration::from_secs(10),
        fetch_inner(
            vault,
            spool,
            TransferRoute::Iq(settings),
            password,
            invited,
            selected,
            now,
        ),
    )
    .await
}
async fn fetch_inner(
    vault: &CheckpointVault,
    spool: &mut Spool,
    route: TransferRoute<'_>,
    password: &str,
    invited: &InvitedHomes,
    selected: ReceivedSelection,
    now: u64,
) -> Result<Report, Error> {
    let clock = Clock::new(now);
    let own = route.check(vault, clock.now())?;
    let ReceivedSelection { index, frame } = selected;
    let r = received(vault, index, frame)?;
    peer_matches(vault, r)?;
    let b = binding(own.clone(), r)?;
    route.binding(&b, TransferDirection::Fetch)?;
    let service = home(&b.sender.account)?;
    if service != home(&own.account)?
        && !route.own_home()
        && !invited.homes.iter().any(|h| h == service)
    {
        return Err(Error::Route);
    }
    let mut session = route
        .connect(&b, TransferDirection::Fetch, own.device, password)
        .await?;
    let mut requests = 0;
    let (offset, committed) = progress(
        request(
            &mut session,
            vault,
            RequestContext {
                route,
                service,
                binding: &b,
                direction: TransferDirection::Fetch,
            },
            Request::Status { id: b.id },
            &mut requests,
            &clock,
        )
        .await?,
        &b,
    )?;
    if !committed || offset != b.size {
        return Err(Error::Protocol);
    }
    let (mut offset, mut complete) = spool.begin(&own, &b, clock.now())?;
    for _ in 0..CHUNKS_PER_CYCLE {
        if complete || offset == b.size {
            break;
        }
        let response = request(
            &mut session,
            vault,
            RequestContext {
                route,
                service,
                binding: &b,
                direction: TransferDirection::Fetch,
            },
            Request::GetChunk { id: b.id, offset },
            &mut requests,
            &clock,
        )
        .await?;
        let chunk = match response {
            Response::Chunk {
                offset: got,
                bytes,
                total,
            } if got == offset
                && total == b.size
                && !bytes.is_empty()
                && bytes.len() <= blobs::CHUNK
                && offset + bytes.len() as u64 <= b.size =>
            {
                bytes
            }
            _ => return Err(Error::Protocol),
        };
        let expected = offset + chunk.len() as u64;
        let (next, ready) = progress(
            spool.call(
                &own,
                Request::PutChunk {
                    id: b.id,
                    offset,
                    bytes: chunk,
                },
                clock.now(),
            )?,
            &local_binding(&own, &b),
        )?;
        if next != expected || ready {
            return Err(Error::Protocol);
        }
        offset = next;
    }
    if !complete && offset == b.size {
        let (next, ready) = progress(
            spool.call(&own, Request::Commit { id: b.id }, clock.now())?,
            &local_binding(&own, &b),
        )?;
        offset = next;
        complete = ready;
    }
    if complete {
        verify_fetched(vault, spool, index, frame, clock.now())?;
    }
    Ok(Report {
        offset,
        complete,
        requests,
        sent: 0,
    })
}
/// Local consumption only. Existing caller-supplied-byte history APIs are unchanged.
pub fn verify_fetched(
    vault: &CheckpointVault,
    spool: &mut Spool,
    index: usize,
    frame: [u8; 32],
    now: u64,
) -> Result<VerifiedBlob, Error> {
    let own = principal(vault)?;
    let r = received(vault, index, frame)?;
    let b = binding(own.clone(), r)?;
    let bytes = spool.bytes(&own, &b, now)?;
    crate::mail_attachments::verify_received(vault, index, frame, &bytes).map_err(|_| Error::Blob)
}
/// Cancellation means no call. Never derives a path from private metadata.
pub fn save_fetched(
    vault: &CheckpointVault,
    spool: &mut Spool,
    index: usize,
    frame: [u8; 32],
    now: u64,
    destination: &Path,
) -> Result<(), Error> {
    let verified = verify_fetched(vault, spool, index, frame, now)?;
    crate::attachment_save::save_verified(&verified, destination).map_err(Error::Save)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test(flavor = "current_thread")]
    async fn one_cycle_deadline_cancels_the_entire_composed_future() {
        let result = within_cycle(
            std::time::Duration::from_millis(20),
            std::future::pending::<Result<(), Error>>(),
        )
        .await;
        assert_eq!(result, Err(Error::Transport(faction_xmpp::Error::Timeout)));
    }
    #[cfg(feature = "own-home-provider")]
    #[tokio::test(flavor = "current_thread")]
    async fn diagnostic_sdk_timeout_origin_is_selected_not_inferred_from_transport() {
        let diagnostics = CycleDiagnostics::default();
        let result = within_cycle_observed(
            std::time::Duration::from_millis(5),
            std::future::pending::<Result<(), Error>>(),
            &diagnostics,
        )
        .await;
        assert_eq!(result, Err(Error::Transport(faction_xmpp::Error::Timeout)));
        assert!(
            diagnostics.snapshot().is_none(),
            "unobserved initialization is not observed zero"
        );
        assert_eq!(diagnostics.stop_origin(), StopOrigin::SdkDeadline);
    }
    #[cfg(feature = "own-home-provider")]
    #[tokio::test(flavor = "current_thread")]
    async fn diagnostic_sdk_error_retains_original_result_and_fixed_category() {
        let diagnostics = CycleDiagnostics::default();
        let result = within_cycle_observed(
            std::time::Duration::from_secs(10),
            async { Err::<(), _>(Error::Protocol) },
            &diagnostics,
        )
        .await;
        assert_eq!(result, Err(Error::Protocol));
        assert_eq!(diagnostics.stop_origin(), StopOrigin::SdkError);
        assert_eq!(diagnostics.error_kind(), DiagnosticError::Protocol);
    }
    #[cfg(feature = "own-home-provider")]
    #[test]
    fn diagnostic_formatter_distinguishes_foreground_total_from_current_cycle() {
        use crate::mail_device_provider::{DiagnosticRequest, SnapshotState, WorkerSnapshot};
        use faction_provider::delivery::{DiagnosticPhase, DiagnosticTimings};
        // Synthetic formatting data conveys no Session, Store or positive actor.
        let cycle = CycleDiagnostics::default();
        cycle
            .sink
            .set(WorkerSnapshot {
                state: SnapshotState::Observed,
                request: DiagnosticRequest::Get,
                phase: DiagnosticPhase::CollectWait,
                completed: 3,
                attempted: 4,
                carrier_attempts: 7,
                busy: 1,
                connections: 6,
                timings: DiagnosticTimings {
                    planned_wait_us: 19,
                    actual_wait_us: 7,
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();
        cycle.record_stop(StopOrigin::SdkDeadline);
        let mut foreground = ForegroundDiagnostics::default();
        foreground.accept_cycle(&cycle);
        let line = foreground.failure_line(
            Some(&cycle),
            StopOrigin::SdkDeadline,
            true,
            52,
            851968,
            std::time::Duration::from_secs(60),
        );
        for field in [
            "direction=fetch",
            "origin=sdk_deadline",
            "last_phase=collect_wait",
            "sdk_completed=52",
            "observed_logical_completed=3",
            "cycle_completed=3",
            "logical_attempted=4",
            "carrier_attempts=7",
            "busy=1",
            "connect_calls=6",
            "planned_wait_us=19",
            "actual_wait_us=7",
        ] {
            assert!(line.contains(field), "missing fixed field {field}");
        }
        assert!(line.len() <= 1024);
        let missing = foreground.failure_line(
            None,
            StopOrigin::ForegroundDeadline,
            true,
            52,
            851968,
            std::time::Duration::from_secs(60),
        );
        assert!(missing.contains("state=unavailable"));
        assert!(missing.contains("cycle_completed=unavailable"));
    }

    #[cfg(feature = "own-home-provider")]
    #[test]
    fn diagnostic_storage_and_saturating_totals_are_bounded() {
        use crate::mail_device_provider::WorkerSnapshot;
        use faction_provider::delivery::{DiagnosticTimings, OperationSnapshot};
        use std::{
            mem::size_of,
            sync::{Arc, OnceLock},
            time::Instant,
        };
        fn send_sync<T: Send + Sync>() {}
        send_sync::<CycleDiagnostics>();
        // Conservative live accounting: worker record, temporary copy, joined
        // value, Operation's optional record+clock, caller context, sink, worker
        // Arc, and FOUR usize words for Arc control (two are used by Rust1.96).
        // Output allocation (separately <=1024 bytes) is not retained state.
        let bytes = 3 * size_of::<WorkerSnapshot>()
            + size_of::<Option<OperationSnapshot>>()
            + size_of::<Option<Instant>>()
            + size_of::<CycleDiagnostics>()
            + size_of::<OnceLock<WorkerSnapshot>>()
            + size_of::<Arc<OnceLock<WorkerSnapshot>>>()
            + 4 * size_of::<usize>()
            + size_of::<ForegroundDiagnostics>();
        println!("owned diagnostic retained-state budget: {bytes} bytes");
        assert!(bytes <= 512, "combined diagnostic storage is {bytes} bytes");
        let max = DiagnosticTimings {
            submit_connect_us: u64::MAX,
            submit_exchange_us: u64::MAX,
            collect_connect_us: u64::MAX,
            collect_exchange_us: u64::MAX,
            planned_wait_us: u64::MAX,
            actual_wait_us: u64::MAX,
        };
        assert_eq!(max.saturating_add(max), max);
        let cycle = CycleDiagnostics::default();
        cycle
            .sink
            .set(WorkerSnapshot {
                state: crate::mail_device_provider::SnapshotState::Observed,
                timings: max,
                phase_us: u64::MAX,
                terminal_carrier_attempts: u8::MAX,
                terminal_busy: u8::MAX,
                exchange_origin: faction_provider::delivery::ExchangeOrigin::Unobserved,
                ..Default::default()
            })
            .unwrap();
        let mut foreground = ForegroundDiagnostics::default();
        foreground.accept_cycle(&cycle);
        let line = foreground.failure_line(
            Some(&cycle),
            StopOrigin::SdkError,
            false,
            65,
            1048592,
            std::time::Duration::MAX,
        );
        assert!(line.len() <= 1024);
        assert!(!line.contains("inf"));
        assert!(line.contains("18446744073709551615"));
    }

    #[cfg(feature = "own-home-provider")]
    #[test]
    fn diagnostic_terminal_logical_counts_and_exchange_origin_are_reported() {
        let cycle = CycleDiagnostics::default();
        cycle
            .sink
            .set(crate::mail_device_provider::WorkerSnapshot {
                state: crate::mail_device_provider::SnapshotState::Observed,
                ..Default::default()
            })
            .unwrap();
        let foreground = ForegroundDiagnostics::default();
        let line = foreground.failure_line(
            Some(&cycle),
            StopOrigin::SdkError,
            true,
            0,
            0,
            std::time::Duration::ZERO,
        );
        assert!(
            line.contains("terminal_carrier_attempts=0"),
            "terminal logical attempt count is absent"
        );
        assert!(
            line.contains("terminal_busy=0"),
            "terminal logical Busy count is absent"
        );
        assert!(
            line.contains("exchange_origin=unobserved"),
            "terminal exchange origin is absent"
        );
        for secret in [
            "PRIVATE_CONTENT_SENTINEL",
            "PRIVATE_TOKEN_SENTINEL",
            "PRIVATE_ENDPOINT_SENTINEL",
        ] {
            assert!(!line.contains(secret));
        }
        assert!(line.len() <= 1024);
        let cycle = CycleDiagnostics::default();
        cycle
            .sink
            .set(crate::mail_device_provider::WorkerSnapshot {
                state: crate::mail_device_provider::SnapshotState::Observed,
                carrier_attempts: 14,
                busy: 6,
                terminal_carrier_attempts: 5,
                terminal_busy: 3,
                exchange_origin: faction_provider::delivery::ExchangeOrigin::ReadEof,
                ..Default::default()
            })
            .unwrap();
        let line = foreground.failure_line(
            Some(&cycle),
            StopOrigin::SdkError,
            true,
            0,
            0,
            std::time::Duration::ZERO,
        );
        assert!(line.contains("cycle_carrier_attempts=14"));
        assert!(line.contains("cycle_busy=6"));
        assert!(line.contains("terminal_carrier_attempts=5"));
        assert!(line.contains("terminal_busy=3"));
        assert!(line.contains("exchange_origin=read_eof"));
        println!("\n{line}");
        assert!(line.starts_with("Attachment diagnostic v2 "));
    }

    #[cfg(feature = "own-home-provider")]
    #[test]
    fn diagnostic_foreground_counters_do_not_saturate_at_one_byte() {
        use crate::mail_device_provider::{SnapshotState, WorkerSnapshot};
        // Synthetic aggregate-bound data; no transport, actor or Store fixture.
        let cycle = CycleDiagnostics::default();
        cycle
            .sink
            .set(WorkerSnapshot {
                state: SnapshotState::Observed,
                attempted: 8,
                completed: 8,
                carrier_attempts: 64,
                busy: 48,
                connections: 64,
                ..Default::default()
            })
            .unwrap();
        let mut foreground = ForegroundDiagnostics::default();
        for _ in 0..65 {
            foreground.accept_cycle(&cycle);
        }
        let line = foreground.failure_line(
            None,
            StopOrigin::ForegroundDeadline,
            true,
            65,
            1048592,
            std::time::Duration::from_secs(60),
        );
        for field in [
            "observed_logical_completed=520",
            "observed_logical_attempted=520",
            "observed_carrier_attempts=4160",
            "observed_busy=3120",
            "observed_connect_calls=4160",
        ] {
            assert!(
                line.contains(field),
                "missing widened foreground counter {field}"
            );
        }
        foreground.accept_cycle(&cycle);
        let invalid = foreground.failure_line(
            None,
            StopOrigin::ForegroundDeadline,
            true,
            65,
            1048592,
            std::time::Duration::from_secs(60),
        );
        assert!(invalid.contains("aggregate_valid=false"));
        assert!(invalid.contains("observed_connect_calls=unavailable"));
    }
}
