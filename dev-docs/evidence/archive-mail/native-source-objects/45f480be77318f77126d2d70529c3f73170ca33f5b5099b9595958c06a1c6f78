//! A publisher-lifetime observation, separate from snapshot age and responsiveness.
use super::Snapshot;
use rustix::fs::{FlockOperation, OFlags, flock};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};

pub const LEASE_FILE: &str = "operator.lease";
const LEASE_BYTES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Unverified,
    Unavailable,
    Released,
    Different,
    Held,
}

pub struct Lease {
    // Keep the persistent inode locked until the publisher is dropped. Never unlink.
    _file: File,
    instance: String,
}
impl Lease {
    pub fn acquire(directory: &Path) -> io::Result<Self> {
        let parent = fs::symlink_metadata(directory)?;
        let uid = rustix::process::geteuid().as_raw();
        if !parent.is_dir() || parent.uid() != uid || parent.mode() & 0o077 != 0 {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags((OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC).bits() as i32)
            .open(directory.join(LEASE_FILE))?;
        let m = file.metadata()?;
        if !m.is_file()
            || m.nlink() != 1
            || m.uid() != uid
            || m.mode() & 0o077 != 0
            || m.len() > LEASE_BYTES as u64
        {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        flock(&file, FlockOperation::NonBlockingLockExclusive)?;
        let instance = crate::request_id().map_err(io::Error::other)?;
        file.set_len(0)?;
        write!(file, "pid={}\ninstance={}\n", std::process::id(), instance)?;
        Ok(Self {
            _file: file,
            instance,
        })
    }
    pub fn instance(&self) -> &str {
        &self.instance
    }
}

pub(super) fn valid_instance(raw: &str) -> bool {
    raw.len() == 32
        && raw
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

/// Only observes a bounded private file and tries a nonblocking shared lock.
/// Never creates/truncates files, takes a DB lock, sends signals or connects.
pub fn observe(directory: &Path, snapshot: &Snapshot) -> State {
    let Some(instance) = &snapshot.instance else {
        return State::Unverified;
    };
    fn read(directory: &Path) -> io::Result<(File, u32, String)> {
        let file = super::open_private(directory, LEASE_FILE, LEASE_BYTES)?;
        let mut raw = String::new();
        (&file)
            .take(LEASE_BYTES as u64 + 1)
            .read_to_string(&mut raw)?;
        let mut lines = raw.lines();
        let pid = lines
            .next()
            .and_then(|s| s.strip_prefix("pid="))
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|n| *n > 0);
        let instance = lines
            .next()
            .and_then(|s| s.strip_prefix("instance="))
            .filter(|s| valid_instance(s));
        match (pid, instance) {
            (Some(pid), Some(id)) if raw.len() <= LEASE_BYTES && lines.next().is_none() => {
                Ok((file, pid, id.to_owned()))
            }
            _ => Err(io::ErrorKind::InvalidData.into()),
        }
    }
    let Ok((file, pid, current)) = read(directory) else {
        return State::Unavailable;
    };
    match flock(&file, FlockOperation::NonBlockingLockShared) {
        Ok(()) => State::Released, // This separate descriptor drops its shared lock now.
        Err(rustix::io::Errno::WOULDBLOCK) if pid == snapshot.pid && current == *instance => {
            State::Held
        }
        Err(rustix::io::Errno::WOULDBLOCK) => State::Different,
        Err(_) => State::Unavailable,
    }
}
