//! Member-owned plaintext notebook. Fixed names, private directory, one lifetime lock.
use super::passport_data::{Data, MAX_BYTES};
use faction_files::invalid;
use rustix::fs::{self, AtFlags, FlockOperation, Mode, OFlags};
use std::{
    fs::File,
    io::{self, Read, Write},
    os::unix::fs::MetadataExt,
    path::{Component, Path},
};
const NAME: &str = "passport.toml";
fn valid(f: &File) -> io::Result<()> {
    let m = f.metadata()?;
    if !m.is_file()
        || m.uid() != rustix::process::geteuid().as_raw()
        || m.mode() & 0o7777 != 0o600
        || m.nlink() != 1
    {
        return Err(invalid());
    }
    Ok(())
}
pub struct Store {
    dir: File,
    _lock: File,
}
impl Store {
    pub fn at(path: &Path, create: bool) -> io::Result<Self> {
        if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(invalid());
        }
        let mut dir = File::from(fs::open(
            "/",
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )?);
        let mut parts = path.components().peekable();
        while let Some(c) = parts.next() {
            if let Component::Normal(n) = c {
                if create && parts.peek().is_none() {
                    fs::mkdirat(&dir, n, Mode::from_raw_mode(0o700))?;
                }
                dir = File::from(fs::openat(
                    &dir,
                    n,
                    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                    Mode::empty(),
                )?);
            }
        }
        let m = dir.metadata()?;
        if m.uid() != rustix::process::geteuid().as_raw() || m.mode() & 0o7777 != 0o700 {
            return Err(invalid());
        }
        let lock = File::from(fs::openat(
            &dir,
            "passport.lock",
            OFlags::CREATE | OFlags::RDWR | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::from_raw_mode(0o600),
        )?);
        valid(&lock)?;
        fs::flock(&lock, FlockOperation::NonBlockingLockExclusive)?;
        Ok(Self { dir, _lock: lock })
    }
    pub fn read(&self) -> io::Result<Data> {
        let mut f = File::from(fs::openat(
            &self.dir,
            NAME,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::empty(),
        )?);
        valid(&f)?;
        if f.metadata()?.len() > MAX_BYTES as u64 {
            return Err(invalid());
        }
        let mut s = String::new();
        Read::by_ref(&mut f)
            .take(MAX_BYTES as u64 + 1)
            .read_to_string(&mut s)?;
        Data::parse(&s)
    }
    pub fn write(&self, data: &Data, new: bool) -> io::Result<()> {
        let raw = data.encode()?;
        if !new {
            self.read()?;
        }
        let mut b = [0; 16];
        getrandom::fill(&mut b).map_err(|_| invalid())?;
        let temp = format!(
            ".passport-{}",
            b.iter().map(|v| format!("{v:02x}")).collect::<String>()
        );
        let mut f = File::from(fs::openat(
            &self.dir,
            temp.as_str(),
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )?);
        let result = (|| {
            valid(&f)?;
            f.write_all(raw.as_bytes())?;
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
#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    #[test]
    fn private_atomic_store_locks_and_refuses_links_or_reinitialization() {
        let root = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("faction-passport-{}", std::process::id()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        let path = root.join("store");
        let store = Store::at(&path, true).unwrap();
        store.write(&Data::default(), true).unwrap();
        assert!(store.read().unwrap().entries.is_empty());
        assert!(Store::at(&path, false).is_err());
        assert!(Store::at(&path, true).is_err());
        assert!(store.write(&Data::default(), true).is_err());
        drop(store);
        let store = Store::at(&path, false).unwrap();
        std::fs::set_permissions(path.join(NAME), std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(store.read().is_err());
        std::fs::set_permissions(path.join(NAME), std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::remove_file(path.join(NAME)).unwrap();
        std::os::unix::fs::symlink("passport.lock", path.join(NAME)).unwrap();
        assert!(store.read().is_err());
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }
}
