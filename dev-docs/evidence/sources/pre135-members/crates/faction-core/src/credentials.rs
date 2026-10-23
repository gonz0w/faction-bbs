//! Per-board, cross-process budget for memory-hard credential work.
use crate::{Error, Result};
use std::{fs::File, os::unix::fs::MetadataExt, path::Path};

/// Two independent kernel locks, held only during Argon2 work. Never unlink
/// these inodes while any process may use this board. They are not snapshot data.
pub(crate) fn acquire(directory: &Path) -> Result<File> {
    for index in 0..2 {
        let file = File::from(
            rustix::fs::openat(
                rustix::fs::CWD,
                directory.join(format!("credential-{index}.lock")),
                rustix::fs::OFlags::CREATE
                    | rustix::fs::OFlags::RDWR
                    | rustix::fs::OFlags::NOFOLLOW
                    | rustix::fs::OFlags::CLOEXEC,
                rustix::fs::Mode::from_raw_mode(0o600),
            )
            .map_err(|_| Error::UnsafeState)?,
        );
        let m = file.metadata()?;
        if !m.is_file()
            || m.uid() != rustix::process::geteuid().as_raw()
            || m.mode() & 0o077 != 0
            || m.nlink() != 1
        {
            return Err(Error::UnsafeState);
        }
        match rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockExclusive) {
            Ok(()) => return Ok(file),
            Err(rustix::io::Errno::WOULDBLOCK) => (),
            Err(e) => return Err(std::io::Error::from(e).into()),
        }
    }
    Err(Error::CredentialBusy)
}

/// Never trust encoded KDF costs to override this build's memory/CPU policy.
pub(crate) fn validate_record(record: &str) -> Result<()> {
    if record.len() > 128 {
        return Err(Error::Credentials);
    }
    let parsed = argon2::PasswordHash::new(record).map_err(|_| Error::Credentials)?;
    if parsed.algorithm.as_str() != "argon2id"
        || parsed.version != Some(19)
        || parsed.params.to_string() != "m=19456,t=2,p=1"
        || parsed.hash.is_none_or(|h| h.len() != 32)
        || parsed.salt.is_none_or(|s| s.len() != 16)
    {
        return Err(Error::Credentials);
    }
    Ok(())
}
