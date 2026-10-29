//! Bounded exact-field input and descriptor-relative private file reads.
use super::{Result, mapped};
use faction_files::private_blobs::Principal;
use faction_provider::{
    BinaryService,
    authority::{CredentialBinding, Registry},
};
use rustix::fs::{self, Mode, OFlags};
use std::{
    fs::File,
    io::Read,
    net::SocketAddr,
    os::unix::fs::MetadataExt,
    path::{Component, Path, PathBuf},
};
fn absolute(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
}
fn private_dir(dir: &File) -> Result<()> {
    let m = mapped(dir.metadata())?;
    if !m.is_dir() || m.uid() != rustix::process::geteuid().as_raw() || m.mode() & 0o7777 != 0o700 {
        Err(())
    } else {
        Ok(())
    }
}
fn directory(path: &Path) -> Result<File> {
    if !absolute(path) {
        return Err(());
    }
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut dir = File::from(mapped(fs::open("/", flags, Mode::empty()))?);
    for c in path.components() {
        if let Component::Normal(n) = c {
            dir = File::from(mapped(fs::openat(&dir, n, flags, Mode::empty()))?);
        }
    }
    private_dir(&dir)?;
    Ok(dir)
}
pub(super) fn private_file(path: &Path, root: &Path, cap: u64) -> Result<Vec<u8>> {
    if !absolute(path) || !absolute(root) || !path.starts_with(root) || path == root {
        return Err(());
    }
    let mut dir = directory(root)?;
    let relative = mapped(path.strip_prefix(root))?;
    let parts = relative.components().collect::<Vec<_>>();
    let (name, parents) = parts.split_last().ok_or(())?;
    for c in parents {
        let Component::Normal(n) = c else {
            return Err(());
        };
        dir = File::from(mapped(fs::openat(
            &dir,
            *n,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        ))?);
        private_dir(&dir)?;
    }
    let Component::Normal(name) = name else {
        return Err(());
    };
    let file = File::from(mapped(fs::openat(
        &dir,
        *name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    ))?);
    let m = mapped(file.metadata())?;
    if !m.is_file()
        || m.uid() != rustix::process::geteuid().as_raw()
        || m.mode() & 0o7777 != 0o600
        || m.nlink() != 1
        || m.len() == 0
        || m.len() > cap
    {
        return Err(());
    }
    let mut bytes = Vec::new();
    mapped((&file).take(cap + 1).read_to_end(&mut bytes))?;
    let after = mapped(file.metadata())?;
    if bytes.len() as u64 != m.len()
        || after.len() != m.len()
        || after.nlink() != 1
        || after.mode() & 0o7777 != 0o600
        || after.uid() != m.uid()
    {
        return Err(());
    }
    Ok(bytes)
}
fn table(path: &Path, root: &Path, cap: u64) -> Result<toml::Table> {
    let bytes = private_file(path, root, cap)?;
    mapped(mapped(std::str::from_utf8(&bytes))?.parse())
}
fn string(t: &mut toml::Table, key: &str) -> Result<String> {
    t.remove(key)
        .and_then(|v| v.as_str().map(str::to_owned))
        .ok_or(())
}
fn number(t: &mut toml::Table, key: &str) -> Result<u64> {
    t.remove(key)
        .and_then(|v| v.as_integer())
        .and_then(|v| u64::try_from(v).ok())
        .ok_or(())
}
fn boolean(t: &mut toml::Table, key: &str) -> Result<bool> {
    t.remove(key).and_then(|v| v.as_bool()).ok_or(())
}
fn version(t: &mut toml::Table) -> Result<()> {
    if number(t, "version")? != 1 {
        Err(())
    } else {
        Ok(())
    }
}
fn digest(text: &str) -> Result<[u8; 32]> {
    if text.len() != 64 {
        return Err(());
    }
    let mut out = [0; 32];
    for (i, b) in text.as_bytes().chunks_exact(2).enumerate() {
        let n = |c: u8| match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            _ => Err(()),
        };
        out[i] = n(b[0])? * 16 + n(b[1])?;
    }
    Ok(out)
}
pub(super) struct Config {
    pub root: PathBuf,
    pub home: String,
    pub endpoint: SocketAddr,
    pub policy: PathBuf,
    pub blobs: PathBuf,
    pub ca: PathBuf,
    pub certificate: PathBuf,
    pub key: PathBuf,
}
impl Config {
    pub(super) fn load(path: &Path) -> Result<Self> {
        let root = path.parent().ok_or(())?.to_path_buf();
        let mut t = table(path, &root, 8192)?;
        version(&mut t)?;
        let home = string(&mut t, "home")?;
        mapped(BinaryService::new(&home))?;
        let endpoint = mapped(string(&mut t, "endpoint")?.parse::<SocketAddr>())?;
        if !endpoint.ip().is_loopback() {
            return Err(());
        }
        let policy = PathBuf::from(string(&mut t, "policy_root")?);
        let blobs = PathBuf::from(string(&mut t, "blob_root")?);
        let ca = PathBuf::from(string(&mut t, "ca")?);
        let certificate = PathBuf::from(string(&mut t, "certificate")?);
        let key = PathBuf::from(string(&mut t, "key")?);
        if !t.is_empty() || policy.starts_with(&blobs) || blobs.starts_with(&policy) {
            return Err(());
        }
        let a = directory(&policy)?;
        let b = directory(&blobs)?;
        let am = mapped(a.metadata())?;
        let bm = mapped(b.metadata())?;
        if (am.dev(), am.ino()) == (bm.dev(), bm.ino()) {
            return Err(());
        }
        for p in [&ca, &certificate, &key] {
            if !absolute(p) || !p.starts_with(&root) || p == &root {
                return Err(());
            }
        }
        Ok(Self {
            root,
            home,
            endpoint,
            policy,
            blobs,
            ca,
            certificate,
            key,
        })
    }
}
pub(super) fn proposal(path: &Path, root: &Path) -> Result<Registry> {
    let mut t = table(path, root, 65_536)?;
    version(&mut t)?;
    let generation = number(&mut t, "generation")?;
    let reset = number(&mut t, "reset_epoch")?;
    let homes = t
        .remove("homes")
        .and_then(|v| v.as_array().cloned())
        .ok_or(())?;
    if homes.len() > 32 {
        return Err(());
    }
    let homes = homes
        .into_iter()
        .map(|v| v.as_str().map(str::to_owned).ok_or(()))
        .collect::<Result<Vec<_>>>()?;
    let bindings = t
        .remove("bindings")
        .and_then(|v| v.as_array().cloned())
        .ok_or(())?;
    if bindings.len() > 32 || !t.is_empty() {
        return Err(());
    }
    let mut entries = Vec::new();
    for value in bindings {
        let mut b = value.as_table().cloned().ok_or(())?;
        let certificate_sha256 = digest(&string(&mut b, "certificate_sha256")?)?;
        let account = string(&mut b, "account")?;
        let device = u32::try_from(number(&mut b, "device")?).map_err(|_| ())?;
        let identity = digest(&string(&mut b, "identity")?)?;
        let entry = CredentialBinding {
            certificate_sha256,
            principal: Principal {
                account,
                device,
                identity,
            },
            valid_from: number(&mut b, "valid_from")?,
            valid_until: number(&mut b, "valid_until")?,
            approved: boolean(&mut b, "approved")?,
            account_enabled: boolean(&mut b, "account_enabled")?,
            device_enabled: boolean(&mut b, "device_enabled")?,
            generation: number(&mut b, "generation")?,
        };
        if !b.is_empty() {
            return Err(());
        }
        entries.push(entry);
    }
    mapped(Registry::new(generation, reset, homes, entries))
}
