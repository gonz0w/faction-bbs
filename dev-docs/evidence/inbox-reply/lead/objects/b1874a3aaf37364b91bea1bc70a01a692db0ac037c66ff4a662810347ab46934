//! Local-only encrypted state, authenticated key wrapping and create-only backups.
//! Version 1 is not an OMEMO wire format or a home-authentication credential.
use crate::mail_data::invalid;
use crate::mail_data::{Data, MAX_PLAIN, hex};
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use rustix::fs::{self, AtFlags, FlockOperation, Mode, OFlags};
use std::{
    fs::File,
    io::{self, Read, Write},
    os::unix::fs::MetadataExt,
    path::Path,
};
use zeroize::Zeroizing;
type Key = Zeroizing<[u8; 32]>;
type NewKeys = (Vec<u8>, Key, Key);
const MAGIC: &[u8; 8] = b"FACTVLT1";
const HEADER: usize = 184;
const PREFIX: usize = HEADER + 24;
const MAX_FILE: usize = PREFIX + MAX_PLAIN + 16;
const NAME: &str = "mail.fkv";
pub(crate) fn random<const N: usize>() -> io::Result<[u8; N]> {
    let mut a = [0; N];
    getrandom::fill(&mut a).map_err(|_| invalid())?;
    Ok(a)
}
pub(crate) fn kdf(password: &[u8], salt: &[u8]) -> io::Result<Zeroizing<[u8; 32]>> {
    // Fixed local parameters. Untrusted files cannot request more work. These do
    // not alter server password parameters or its two-permit credential budget.
    if !(12..=128).contains(&password.len()) {
        return Err(invalid());
    }
    let params = Params::new(65536, 3, 1, Some(32)).map_err(|_| invalid())?;
    let mut key = Zeroizing::new([0; 32]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password, salt, key.as_mut())
        .map_err(|_| invalid())?;
    Ok(key)
}
pub(crate) fn seal(key: &[u8; 32], nonce: &[u8], plain: &[u8], aad: &[u8]) -> io::Result<Vec<u8>> {
    XChaCha20Poly1305::new(key.into())
        .encrypt(
            <&XNonce>::try_from(nonce).map_err(|_| invalid())?,
            Payload { msg: plain, aad },
        )
        .map_err(|_| invalid())
}
pub(crate) fn open(
    key: &[u8; 32],
    nonce: &[u8],
    cipher: &[u8],
    aad: &[u8],
) -> io::Result<Zeroizing<Vec<u8>>> {
    XChaCha20Poly1305::new(key.into())
        .decrypt(
            <&XNonce>::try_from(nonce).map_err(|_| invalid())?,
            Payload { msg: cipher, aad },
        )
        .map(Zeroizing::new)
        .map_err(|_| invalid())
}
fn aad(header: &[u8], kind: &[u8]) -> Vec<u8> {
    let mut out = b"Faction local mail vault v1\0".to_vec();
    out.extend_from_slice(kind);
    out.extend_from_slice(&header[..40]);
    out
}
pub(crate) fn valid(file: &File) -> io::Result<()> {
    let m = file.metadata()?;
    if !m.is_file()
        || m.uid() != rustix::process::getuid().as_raw()
        || m.mode() & 0o7777 != 0o600
        || m.nlink() != 1
    {
        return Err(invalid());
    }
    Ok(())
}
pub struct Store {
    pub(crate) dir: File,
    _lock: File,
}
impl Store {
    pub fn at(path: &Path, create: bool) -> io::Result<Self> {
        if !path.is_absolute()
            || path
                .components()
                .any(|p| matches!(p, std::path::Component::ParentDir))
        {
            return Err(invalid());
        }
        let mut dir = File::from(fs::open(
            "/",
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )?);
        let mut components = path.components().peekable();
        while let Some(component) = components.next() {
            if let std::path::Component::Normal(name) = component {
                if create && components.peek().is_none() {
                    fs::mkdirat(&dir, name, Mode::from_raw_mode(0o700))?;
                    #[cfg(all(test, feature = "native-mail"))]
                    if FAIL_PARENT_SYNC.with(|v| v.replace(false)) {
                        return Err(io::Error::other("injected parent-directory sync failure"));
                    }
                    dir.sync_all()?;
                }
                dir = File::from(fs::openat(
                    &dir,
                    name,
                    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                    Mode::empty(),
                )?);
            }
        }
        let m = dir.metadata()?;
        if m.uid() != rustix::process::getuid().as_raw() || m.mode() & 0o7777 != 0o700 {
            return Err(invalid());
        }
        let lock = File::from(fs::openat(
            &dir,
            "mail.lock",
            OFlags::CREATE | OFlags::RDWR | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::from_raw_mode(0o600),
        )?);
        valid(&lock)?;
        fs::flock(&lock, FlockOperation::NonBlockingLockExclusive)?;
        Ok(Self { dir, _lock: lock })
    }
    fn read(&self) -> io::Result<Vec<u8>> {
        let mut f = File::from(fs::openat(
            &self.dir,
            NAME,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::empty(),
        )?);
        valid(&f)?;
        let len = f.metadata()?.len() as usize;
        if !(PREFIX + 16..=MAX_FILE).contains(&len) {
            return Err(invalid());
        }
        let mut out = Vec::with_capacity(len);
        Read::by_ref(&mut f)
            .take(MAX_FILE as u64 + 1)
            .read_to_end(&mut out)?;
        if out.len() != len || &out[..8] != MAGIC {
            return Err(invalid());
        }
        Ok(out)
    }
    fn write(&self, raw: &[u8], new: bool) -> io::Result<()> {
        if raw.len() > MAX_FILE {
            return Err(invalid());
        }
        // Keep one lock inode for the directory's lifetime. Never truncate the
        // current vault, follow a link, or clobber an existing initialization.
        if !new {
            self.read()?;
        }
        let temp = format!(".pending-{}", hex(&random::<16>()?));
        let mut f = File::from(fs::openat(
            &self.dir,
            temp.as_str(),
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )?);
        let result = (|| {
            valid(&f)?;
            f.write_all(raw)?;
            f.sync_all()?;
            if new {
                fs::linkat(&self.dir, temp.as_str(), &self.dir, NAME, AtFlags::empty())?;
            } else {
                fs::renameat(&self.dir, temp.as_str(), &self.dir, NAME)?;
            }
            self.dir.sync_all()?;
            Ok(())
        })();
        let _ = fs::unlinkat(&self.dir, temp.as_str(), AtFlags::empty());
        result
    }
}
pub struct Vault {
    store: Store,
    header: Vec<u8>,
    key: Zeroizing<[u8; 32]>,
    pub data: Data,
}
impl Vault {
    #[cfg(feature = "native-mail")]
    pub(crate) fn into_checkpoint_parts(self) -> (Store, Data) {
        (self.store, self.data)
    }
    pub fn create(
        store: Store,
        data: Data,
        password: &[u8],
    ) -> io::Result<(Self, Zeroizing<[u8; 32]>)> {
        let (header, key, recovery) = Self::keys(password)?;
        let vault = Self {
            store,
            header,
            key,
            data,
        };
        vault.persist(true)?;
        Ok((vault, recovery))
    }
    fn keys(password: &[u8]) -> io::Result<NewKeys> {
        let key = Zeroizing::new(random::<32>()?);
        let recovery = Zeroizing::new(random::<32>()?);
        let mut header = MAGIC.to_vec();
        header.extend_from_slice(&random::<16>()?);
        header.extend_from_slice(&random::<16>()?);
        let pw = kdf(password, &header[24..40])?;
        for (wrapping, kind) in [
            (&*pw, b"password".as_slice()),
            (&*recovery, b"recovery".as_slice()),
        ] {
            let nonce = random::<24>()?;
            let encrypted = seal(wrapping, &nonce, &*key, &aad(&header, kind))?;
            header.extend_from_slice(&nonce);
            header.extend_from_slice(&encrypted);
        }
        assert_eq!(header.len(), HEADER);
        Ok((header, key, recovery))
    }
    pub fn unlock(store: Store, secret: &[u8], recovery: bool) -> io::Result<Self> {
        let raw = store.read()?;
        let (wrapping, offset, kind) = if recovery {
            (
                Zeroizing::new(<[u8; 32]>::try_from(secret).map_err(|_| invalid())?),
                112,
                b"recovery".as_slice(),
            )
        } else {
            (kdf(secret, &raw[24..40])?, 40, b"password".as_slice())
        };
        let key_bytes = open(
            &wrapping,
            &raw[offset..offset + 24],
            &raw[offset + 24..offset + 72],
            &aad(&raw, kind),
        )?;
        let key =
            Zeroizing::new(<[u8; 32]>::try_from(key_bytes.as_slice()).map_err(|_| invalid())?);
        let plain = open(&key, &raw[HEADER..PREFIX], &raw[PREFIX..], &raw[..PREFIX])?;
        let data = Data::decode(&plain)?;
        Ok(Self {
            store,
            header: raw[..HEADER].to_vec(),
            key,
            data,
        })
    }
    fn encrypted(&self) -> io::Result<Vec<u8>> {
        let mut prefix = self.header.clone();
        let nonce = random::<24>()?;
        prefix.extend_from_slice(&nonce);
        let plain = self.data.encode()?;
        let cipher = seal(&self.key, &nonce, &plain, &prefix)?;
        prefix.extend_from_slice(&cipher);
        Ok(prefix)
    }
    fn persist(&self, new: bool) -> io::Result<()> {
        self.store.write(&self.encrypted()?, new)
    }
    pub fn save(&self) -> io::Result<()> {
        self.persist(false)
    }
    /// Re-encrypt under a new random local key and rotate both unlocking paths.
    /// Old backups still open with their old secrets and cannot be revoked.
    pub fn reset(&mut self, password: &[u8]) -> io::Result<Zeroizing<[u8; 32]>> {
        let (header, key, recovery) = Self::keys(password)?;
        let (old_h, old_k) = (
            std::mem::replace(&mut self.header, header),
            std::mem::replace(&mut self.key, key),
        );
        if let Err(e) = self.save() {
            self.header = old_h;
            self.key = old_k;
            return Err(e);
        }
        Ok(recovery)
    }
    pub fn backup(&self, path: &Path) -> io::Result<()> {
        let dest = Store::at(path, true)?;
        // Preserve one exact current file; recovery is tested by independently
        // opening the backup, not by assuming a password reset recovered keys.
        dest.write(&self.store.read()?, true)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            Self(
                std::env::temp_dir()
                    .canonicalize()
                    .unwrap()
                    .join(format!("faction-vault-{}", hex(&random::<16>().unwrap()))),
            )
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    const PASS: &[u8] = b"member local test passphrase";
    #[test]
    fn encrypted_persistence_recovery_trust_and_backup() {
        let f = Fixture::new();
        let backup = Fixture::new();
        let mut data = Data::new("alice@home.test".into()).unwrap();
        data.draft(
            "bob@away.test".into(),
            Zeroizing::new("private-subject-marker".into()),
            Zeroizing::new("private-body-marker".into()),
        )
        .unwrap();
        data.pin("bob@away.test".into(), 17, [9; 32]).unwrap();
        let (v, recovery) = Vault::create(Store::at(&f.0, true).unwrap(), data, PASS).unwrap();
        let raw = std::fs::read(f.0.join(NAME)).unwrap();
        for secret in [
            b"private-subject-marker".as_slice(),
            b"private-body-marker",
            b"alice@home.test",
            &*recovery,
        ] {
            assert!(!raw.windows(secret.len()).any(|w| w == secret));
        }
        assert!(Store::at(&f.0, false).is_err()); // concurrent writer is refused
        v.backup(&backup.0).unwrap();
        assert!(v.backup(&backup.0).is_err());
        drop(v);
        assert!(
            Vault::unlock(
                Store::at(&f.0, false).unwrap(),
                b"wrong local password",
                false
            )
            .is_err()
        );
        let mut v = Vault::unlock(Store::at(&f.0, false).unwrap(), &*recovery, true).unwrap();
        assert_eq!(v.data.drafts[0].body.as_str(), "private-body-marker");
        assert_eq!(v.data.pins[0].device, 17);
        let new_recovery = v.reset(b"a different local passphrase").unwrap();
        assert_ne!(*new_recovery, *recovery);
        drop(v);
        assert!(Vault::unlock(Store::at(&f.0, false).unwrap(), PASS, false).is_err());
        assert!(Vault::unlock(Store::at(&f.0, false).unwrap(), &*recovery, true).is_err());
        let v = Vault::unlock(
            Store::at(&f.0, false).unwrap(),
            b"a different local passphrase",
            false,
        )
        .unwrap();
        assert_eq!(v.data.pins[0].fingerprint, [9; 32]);
        drop(v);
        assert!(Vault::unlock(Store::at(&backup.0, false).unwrap(), &*recovery, true).is_ok());
    }
    #[test]
    fn tampering_truncation_trailing_and_work_bounds_fail_closed() {
        let f = Fixture::new();
        let (v, _) = Vault::create(
            Store::at(&f.0, true).unwrap(),
            Data::new("a@home.test".into()).unwrap(),
            PASS,
        )
        .unwrap();
        drop(v);
        let raw = std::fs::read(f.0.join(NAME)).unwrap();
        // Authenticate with the valid password for representative bytes from
        // every header/wrapper/nonce/ciphertext component.
        for offset in [0, 8, 24, 40, 64, 112, 136, 184, 208, raw.len() - 1] {
            let mut changed = raw.clone();
            changed[offset] ^= 1;
            std::fs::write(f.0.join(NAME), changed).unwrap();
            assert!(
                Vault::unlock(Store::at(&f.0, false).unwrap(), PASS, false).is_err(),
                "offset {offset}"
            );
        }
        for len in [0, 7, 40, HEADER, PREFIX, raw.len() - 1] {
            std::fs::write(f.0.join(NAME), &raw[..len]).unwrap();
            assert!(Vault::unlock(Store::at(&f.0, false).unwrap(), PASS, false).is_err());
        }
        let mut trailing = raw.clone();
        trailing.push(0);
        std::fs::write(f.0.join(NAME), trailing).unwrap();
        assert!(Vault::unlock(Store::at(&f.0, false).unwrap(), PASS, false).is_err());
        std::fs::write(f.0.join(NAME), vec![0; MAX_FILE + 1]).unwrap();
        assert!(Vault::unlock(Store::at(&f.0, false).unwrap(), PASS, false).is_err());
    }
    #[test]
    fn private_files_links_and_create_only_initialization() {
        let f = Fixture::new();
        let (v, _) = Vault::create(
            Store::at(&f.0, true).unwrap(),
            Data::new("a@home.test".into()).unwrap(),
            PASS,
        )
        .unwrap();
        drop(v);
        let old = std::fs::read(f.0.join(NAME)).unwrap();
        assert!(
            Vault::create(
                Store::at(&f.0, false).unwrap(),
                Data::new("b@home.test".into()).unwrap(),
                PASS
            )
            .is_err()
        );
        assert_eq!(std::fs::read(f.0.join(NAME)).unwrap(), old);
        std::fs::set_permissions(f.0.join(NAME), std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(Vault::unlock(Store::at(&f.0, false).unwrap(), PASS, false).is_err());
        std::fs::set_permissions(f.0.join(NAME), std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::hard_link(f.0.join(NAME), f.0.join("alias")).unwrap();
        assert!(Vault::unlock(Store::at(&f.0, false).unwrap(), PASS, false).is_err());
        std::fs::remove_file(f.0.join("alias")).unwrap();
        std::fs::rename(f.0.join(NAME), f.0.join("original")).unwrap();
        symlink("original", f.0.join(NAME)).unwrap();
        assert!(Vault::unlock(Store::at(&f.0, false).unwrap(), PASS, false).is_err());
        std::fs::remove_file(f.0.join(NAME)).unwrap();
        std::fs::rename(f.0.join("original"), f.0.join(NAME)).unwrap();
        std::fs::set_permissions(&f.0, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(Store::at(&f.0, false).is_err());
    }
}

#[cfg(all(test, feature = "native-mail"))]
thread_local! { static FAIL_PARENT_SYNC: std::cell::Cell<bool> = const {std::cell::Cell::new(false)}; }
#[cfg(all(test, feature = "native-mail"))]
pub(crate) fn fail_parent_sync_once() {
    FAIL_PARENT_SYNC.with(|v| v.set(true));
}
