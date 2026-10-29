//! Experimental complete-buffer device-side attachment bytes. No file or network I/O.
//!
//! Authenticate the descriptor's sender/recipient envelope before calling `verify`.
//! A verified value authenticates bytes against supplied metadata, not a sender,
//! recipient, locator, permission, or complete file-sharing profile.
use crate::{Error, Result, attachment::Descriptor};
use zeroize::Zeroizing;

/// Experimental device-library bound, separate from message and transport limits.
pub const MAX_BLOB_BYTES: usize = 1_048_576;
const TAG_BYTES: usize = 16;
const MAX_CIPHERTEXT: usize = MAX_BLOB_BYTES + TAG_BYTES;

unsafe extern "C" {
    fn faction_blob_prepare() -> i32;
    fn faction_blob_crypt(
        encrypt: i32,
        key: *const u8,
        iv: *const u8,
        input: *const u8,
        n: usize,
        output: *mut u8,
        capacity: usize,
        written: *mut usize,
    ) -> i32;
    fn faction_blob_sha256(input: *const u8, n: usize, hash: *mut u8) -> i32;
}

/// Ciphertext and private material for a trusted caller to construct a descriptor.
/// Keys, IV and private hash are zeroized on drop. The container has no Debug.
pub struct EncryptedBlob {
    pub ciphertext: Vec<u8>,
    pub key: Zeroizing<[u8; 32]>,
    pub iv: Zeroizing<[u8; 12]>,
    pub hash: Zeroizing<[u8; 32]>,
    pub size: u64,
}

/// Plaintext released only after authentication, exact size and hash validation.
/// Construction and mutation remain private. No sender identity is implied.
pub struct VerifiedBlob {
    bytes: Zeroizing<Vec<u8>>,
}
impl VerifiedBlob {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

fn prepare() -> Result<()> {
    // No Rust secret allocation precedes the blob-only runtime/core guard.
    if unsafe { faction_blob_prepare() } == 0 {
        Ok(())
    } else {
        Err(Error)
    }
}
fn digest(plain: &[u8]) -> Result<Zeroizing<[u8; 32]>> {
    let mut hash = Zeroizing::new([0; 32]);
    // Rust owns a valid input slice and exactly 32 writable output bytes.
    if unsafe { faction_blob_sha256(plain.as_ptr(), plain.len(), hash.as_mut_ptr()) } != 0 {
        return Err(Error);
    }
    Ok(hash)
}
fn crypt(encrypt: bool, input: &[u8], key: &[u8; 32], iv: &[u8; 12]) -> Result<Zeroizing<Vec<u8>>> {
    let expected = if encrypt {
        if input.len() > MAX_BLOB_BYTES {
            return Err(Error);
        }
        input.len() + TAG_BYTES
    } else {
        if !(TAG_BYTES..=MAX_CIPHERTEXT).contains(&input.len()) {
            return Err(Error);
        }
        input.len() - TAG_BYTES
    };
    let capacity = if encrypt { expected } else { input.len() };
    let mut output = Zeroizing::new(vec![0; capacity]);
    let mut written = 0;
    // All pointers are valid, disjoint and remain owned for this complete call.
    // Capacity includes tag/final scratch; no C-owned plaintext pointer escapes.
    let result = unsafe {
        faction_blob_crypt(
            i32::from(encrypt),
            key.as_ptr(),
            iv.as_ptr(),
            input.as_ptr(),
            input.len(),
            output.as_mut_ptr(),
            output.len(),
            &mut written,
        )
    };
    if result != 0 || written != expected {
        return Err(Error);
    }
    output.truncate(written);
    Ok(output)
}

/// Encrypt actual bounded bytes with new random key/IV for every call.
/// There is deliberately no public caller-supplied encryption material override.
pub fn encrypt(plaintext: &[u8]) -> Result<EncryptedBlob> {
    if plaintext.len() > MAX_BLOB_BYTES {
        return Err(Error);
    }
    prepare()?;
    let mut key = Zeroizing::new([0; 32]);
    let mut iv = Zeroizing::new([0; 12]);
    getrandom::fill(&mut *key).map_err(|_| Error)?;
    getrandom::fill(&mut *iv).map_err(|_| Error)?;
    let hash = digest(plaintext)?;
    let ciphertext = crypt(true, plaintext, &key, &iv)?.to_vec();
    Ok(EncryptedBlob {
        ciphertext,
        key,
        iv,
        hash,
        size: plaintext.len() as u64,
    })
}

/// Verify bytes against an already-authenticated descriptor. No fetch or save.
/// Bounds and exact asserted size reject before allocation or cryptography.
pub fn verify(ciphertext: &[u8], descriptor: &Descriptor) -> Result<VerifiedBlob> {
    if descriptor.size > MAX_BLOB_BYTES as u64
        || !(TAG_BYTES..=MAX_CIPHERTEXT).contains(&ciphertext.len())
        || ciphertext.len() - TAG_BYTES != descriptor.size as usize
    {
        return Err(Error);
    }
    descriptor.validate()?;
    prepare()?;
    let bytes = crypt(false, ciphertext, &descriptor.key, &descriptor.iv)?;
    if digest(&bytes)?.as_slice() != descriptor.hash.as_slice() {
        return Err(Error);
    }
    Ok(VerifiedBlob { bytes })
}

#[cfg(test)]
#[path = "blob_tests.rs"]
mod tests;
