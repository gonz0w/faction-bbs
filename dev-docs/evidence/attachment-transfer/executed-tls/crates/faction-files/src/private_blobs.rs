//! Bounded ciphertext-only storage for a trusted authenticated home adapter.
//!
//! `Principal` is routing data, never proof of identity. The adapter must derive
//! the actor from authentication and implement current invitation, account and
//! device policy in `Access`; a stanza/resource name cannot establish that proof.
//! No device key, filename, IV or plaintext hash belongs in this store or codec.
mod codec;
mod disk;
pub use codec::{decode_request, decode_response, encode_request, encode_response};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const MAX_OBJECTS: usize = 8;
pub const MAX_CIPHERTEXT: usize = 1_048_592;
pub const MAX_RESERVED: u64 = 8_388_736;
pub const CHUNK: usize = 4096;
pub const MAX_RETENTION: u64 = 604_800;
pub const MAX_WIRE: usize = 8192;
/// Includes bounded metadata; at most two snapshots and a zero-length lock exist.
pub const MAX_SNAPSHOT: usize = MAX_RESERVED as usize + 8192;
pub const MAX_DISK_BYTES: usize = 2 * MAX_SNAPSHOT;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Invalid,
    Denied,
    NotFound,
    Quota,
    Expired,
    Clock,
    Busy,
    Storage,
    PublicationUncertain,
    Poisoned,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "invalid private attachment operation",
            Self::Denied => "private attachment operation denied",
            Self::NotFound => "private attachment unavailable",
            Self::Quota => "private attachment quota reached",
            Self::Expired => "private attachment expired",
            Self::Clock => "private attachment clock unavailable",
            Self::Busy => "private attachment store busy",
            Self::Storage => "private attachment storage unavailable",
            Self::PublicationUncertain => "private attachment publication may have completed",
            Self::Poisoned => "private attachment store requires reopen",
        })
    }
}
impl std::error::Error for Error {}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Principal {
    pub account: String,
    pub device: u32,
    pub identity: [u8; 32],
}
impl Principal {
    pub fn validate(&self) -> Result<(), Error> {
        let Some((user, host)) = self.account.split_once('@') else {
            return Err(Error::Invalid);
        };
        if self.account.len() > 254
            || user.is_empty()
            || user.len() > 64
            || !user
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
            || !host.contains('.')
            || host.len() > 189
            || !host.split('.').all(|p| {
                !p.is_empty()
                    && p.len() <= 63
                    && !p.starts_with('-')
                    && !p.ends_with('-')
                    && p.bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            })
            || self.device == 0
            || self.identity == [0; 32]
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub sender: Principal,
    pub recipient: Principal,
    pub id: [u8; 32],
    pub size: u64,
    pub created: u64,
    pub expires: u64,
}
impl Binding {
    pub fn validate(&self) -> Result<(), Error> {
        self.sender.validate()?;
        self.recipient.validate()?;
        if !(16..=MAX_CIPHERTEXT as u64).contains(&self.size)
            || self.expires <= self.created
            || self.expires - self.created > MAX_RETENTION
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Begin,
    Status,
    PutChunk,
    Commit,
    GetChunk,
    Delete,
}
/// Mandatory trusted-adapter policy. Recheck both current home invitations and
/// sender/recipient account, device and full public-identity permissions each time.
pub trait Access {
    fn require(
        &self,
        actor: &Principal,
        binding: &Binding,
        action: Action,
        now: u64,
    ) -> Result<(), Error>;
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Request {
    Begin(Binding),
    Status {
        id: [u8; 32],
    },
    PutChunk {
        id: [u8; 32],
        offset: u64,
        bytes: Vec<u8>,
    },
    Commit {
        id: [u8; 32],
    },
    GetChunk {
        id: [u8; 32],
        offset: u64,
    },
    Delete {
        id: [u8; 32],
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Response {
    Progress {
        binding: Binding,
        offset: u64,
        committed: bool,
    },
    Chunk {
        offset: u64,
        bytes: Vec<u8>,
        total: u64,
    },
    Deleted,
}
#[derive(Clone)]
struct Object {
    binding: Binding,
    bytes: Vec<u8>,
    committed: bool,
}
#[derive(Clone, Default)]
struct State {
    clock: u64,
    objects: Vec<Object>,
}
impl State {
    fn validate(&self) -> Result<(), Error> {
        if self.objects.len() > MAX_OBJECTS {
            return Err(Error::Invalid);
        }
        let mut reserved = 0u64;
        for (i, o) in self.objects.iter().enumerate() {
            o.binding.validate()?;
            reserved = reserved.checked_add(o.binding.size).ok_or(Error::Invalid)?;
            if reserved > MAX_RESERVED
                || o.binding.created > self.clock
                || o.bytes.len() as u64 > o.binding.size
                || self.objects[..i]
                    .iter()
                    .any(|old| old.binding.id == o.binding.id)
                || (o.committed
                    && (o.bytes.len() as u64 != o.binding.size
                        || <[u8; 32]>::from(Sha256::digest(&o.bytes)) != o.binding.id))
            {
                return Err(Error::Invalid);
            }
        }
        Ok(())
    }
}
/// Owns one permanent exclusive lock. Reopen after uncertain publication; neither
/// an error nor process death implies rollback. Expired entries remain charged
/// until the sender explicitly deletes them with current `Access` permission.
pub struct Store {
    disk: disk::Disk,
    state: State,
    poisoned: bool,
}
impl Store {
    pub fn open(path: &Path) -> Result<Self, Error> {
        let (disk, state) = disk::Disk::open(path)?;
        Ok(Self {
            disk,
            state,
            poisoned: false,
        })
    }
    fn commit(&mut self, state: State) -> Result<(), Error> {
        if let Err(e) = self.disk.write(&state) {
            self.poisoned = true;
            return Err(e);
        }
        self.state = state;
        Ok(())
    }
    /// The actor is supplied separately from wire requests by the trusted adapter.
    /// Successful increasing timestamps and authorized expiry observations are
    /// durable, including read/status calls.
    pub fn dispatch<A: Access + ?Sized>(
        &mut self,
        actor: &Principal,
        access: &A,
        request: Request,
        now: u64,
    ) -> Result<Response, Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        actor.validate()?;
        if now < self.state.clock {
            return Err(Error::Clock);
        }
        let (id, action) = match &request {
            Request::Begin(b) => (b.id, Action::Begin),
            Request::Status { id } => (*id, Action::Status),
            Request::PutChunk { id, .. } => (*id, Action::PutChunk),
            Request::Commit { id } => (*id, Action::Commit),
            Request::GetChunk { id, .. } => (*id, Action::GetChunk),
            Request::Delete { id } => (*id, Action::Delete),
        };
        let found = self.state.objects.iter().position(|o| o.binding.id == id);
        if let Request::Begin(b) = &request {
            b.validate()?;
        }
        let binding = found
            .map(|i| &self.state.objects[i].binding)
            .or(match &request {
                Request::Begin(b) => Some(b),
                _ => None,
            })
            .ok_or(Error::NotFound)?;
        access.require(actor, binding, action, now)?;
        let sender = actor == &binding.sender;
        let recipient = actor == &binding.recipient;
        match action {
            Action::Begin | Action::PutChunk | Action::Commit | Action::Delete if !sender => {
                return Err(Error::Denied);
            }
            Action::Status
                if !sender
                    && (!recipient
                        || !self.state.objects[found.ok_or(Error::NotFound)?].committed) =>
            {
                return Err(Error::Denied);
            }
            Action::GetChunk
                if !recipient || !self.state.objects[found.ok_or(Error::NotFound)?].committed =>
            {
                return Err(Error::Denied);
            }
            _ => (),
        }
        if action != Action::Delete {
            if now < binding.created {
                return Err(Error::Clock);
            }
            if now >= binding.expires {
                // An authorized observation of expiry must survive reopen, even
                // though no ciphertext/progress is returned. Otherwise a later
                // clock rollback could revive this object. Policy denials above
                // never advance this watermark or mutate the store.
                if now > self.state.clock {
                    let mut expired = self.state.clone();
                    expired.clock = now;
                    self.commit(expired)?;
                }
                return Err(Error::Expired);
            }
        }
        let mut candidate = self.state.clone();
        candidate.clock = now;
        let mut changed = now != self.state.clock;
        let response = match request {
            Request::Begin(b) => {
                if let Some(i) = found {
                    if candidate.objects[i].binding != b {
                        return Err(Error::Invalid);
                    }
                    progress(&candidate.objects[i])
                } else {
                    if candidate.objects.len() == MAX_OBJECTS
                        || candidate
                            .objects
                            .iter()
                            .map(|o| o.binding.size)
                            .sum::<u64>()
                            + b.size
                            > MAX_RESERVED
                    {
                        return Err(Error::Quota);
                    }
                    candidate.objects.push(Object {
                        binding: b,
                        bytes: Vec::new(),
                        committed: false,
                    });
                    // Canonical order does not depend on insertion history.
                    candidate.objects.sort_by_key(|o| o.binding.id);
                    changed = true;
                    progress(
                        candidate
                            .objects
                            .iter()
                            .find(|o| o.binding.id == id)
                            .ok_or(Error::Invalid)?,
                    )
                }
            }
            Request::Status { .. } => progress(&candidate.objects[found.ok_or(Error::NotFound)?]),
            Request::PutChunk { offset, bytes, .. } => {
                let o = &mut candidate.objects[found.ok_or(Error::NotFound)?];
                let end = offset
                    .checked_add(bytes.len() as u64)
                    .ok_or(Error::Invalid)?;
                if bytes.is_empty() || bytes.len() > CHUNK || end > o.binding.size {
                    return Err(Error::Invalid);
                }
                if offset < o.bytes.len() as u64 {
                    if end > o.bytes.len() as u64 || o.bytes[offset as usize..end as usize] != bytes
                    {
                        return Err(Error::Invalid);
                    }
                } else if offset == o.bytes.len() as u64 && !o.committed {
                    o.bytes.extend(bytes);
                    changed = true;
                } else {
                    return Err(Error::Invalid);
                }
                progress(o)
            }
            Request::Commit { .. } => {
                let o = &mut candidate.objects[found.ok_or(Error::NotFound)?];
                if o.bytes.len() as u64 != o.binding.size
                    || <[u8; 32]>::from(Sha256::digest(&o.bytes)) != id
                {
                    return Err(Error::Invalid);
                }
                changed |= !o.committed;
                o.committed = true;
                progress(o)
            }
            Request::GetChunk { offset, .. } => {
                let o = &candidate.objects[found.ok_or(Error::NotFound)?];
                if offset > o.binding.size {
                    return Err(Error::Invalid);
                }
                let start = offset as usize;
                let end = (start + CHUNK).min(o.bytes.len());
                Response::Chunk {
                    offset,
                    bytes: o.bytes[start..end].to_vec(),
                    total: o.binding.size,
                }
            }
            Request::Delete { .. } => {
                candidate.objects.remove(found.ok_or(Error::NotFound)?);
                changed = true;
                Response::Deleted
            }
        };
        if changed {
            self.commit(candidate)?;
        }
        Ok(response)
    }
}
fn progress(o: &Object) -> Response {
    Response::Progress {
        binding: o.binding.clone(),
        offset: o.bytes.len() as u64,
        committed: o.committed,
    }
}
