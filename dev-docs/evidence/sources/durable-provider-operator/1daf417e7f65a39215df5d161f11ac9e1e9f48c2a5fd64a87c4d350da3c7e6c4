//! Trusted member-local consumption of attachment bytes bound to retained mail.
//!
//! The caller supplies ciphertext separately and deliberately selects an absolute
//! save destination. Stored names and locators remain inert. This module performs
//! no transport, fetch, upload, source-file loading, or automatic save.
use crate::{
    attachment_save::{self, SaveError},
    mail_checkpoint::{CheckpointVault, Direction},
};
use faction_omemo::blob::{self, VerifiedBlob};
use std::{fmt, path::Path};

/// Fixed diagnostics contain no path, record metadata, key, or private bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The vault is unhealthy or has not explicitly enabled attachments.
    State,
    /// The retained received selection or its attachment is unavailable.
    Record,
    /// Supplied ciphertext failed authentication, hash, size, or bounds checks.
    Blob,
    /// Preserve whether file publication may have completed.
    Save(SaveError),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State => f.write_str("attachment history unavailable"),
            Self::Record => f.write_str("attachment selection unavailable"),
            Self::Blob => f.write_str("attachment verification failed"),
            Self::Save(error) => fmt::Display::fmt(error, f),
        }
    }
}
impl std::error::Error for Error {}

/// Verify explicitly supplied bytes using only an authenticated retained record.
///
/// Both the index and exact received-frame identity must match. Local history
/// remains consumable after protocol expiry, revocation, recovery quarantine or
/// fresh-device changes; this function does not grant protocol authority.
pub fn verify_received(
    vault: &CheckpointVault,
    index: usize,
    expected_received_frame: [u8; 32],
    ciphertext: &[u8],
) -> Result<VerifiedBlob, Error> {
    if !vault
        .inspection()
        .map_err(|_| Error::State)?
        .attachments_enabled
    {
        return Err(Error::State);
    }
    let record = vault
        .history()
        .map_err(|_| Error::State)?
        .get(index)
        .ok_or(Error::Record)?;
    if record.direction != Direction::Received
        || record.received_id() != Some(expected_received_frame)
    {
        return Err(Error::Record);
    }
    let descriptor = record.attachment().ok_or(Error::Record)?;
    blob::verify(ciphertext, descriptor).map_err(|_| Error::Blob)
}

/// Verify the retained received selection before any filesystem action, then
/// delegate to the existing private, no-overwrite save operation. Cancellation
/// means the caller makes no call. Metadata never chooses the destination.
pub fn save_received(
    vault: &CheckpointVault,
    index: usize,
    expected_received_frame: [u8; 32],
    ciphertext: &[u8],
    explicit_absolute_destination: &Path,
) -> Result<(), Error> {
    let verified = verify_received(vault, index, expected_received_frame, ciphertext)?;
    attachment_save::save_verified(&verified, explicit_absolute_destination).map_err(Error::Save)
}
