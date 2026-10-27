//! Single-object SFTP v3 adapter over OpenSSH; no host paths from clients.
use faction_files::{id, invalid, rpc};
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    fs::{OFlags, fcntl_getfl, fcntl_setfl},
    process::{Resource, Rlimit, setrlimit},
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::{
        fd::AsFd,
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
const CHUNK: usize = 32768;
const PACKET: usize = CHUNK + 1024;
#[derive(Clone)]
pub struct Config {
    pub owner: u32,
    pub socket: PathBuf,
    pub store: PathBuf,
}
impl Config {
    pub fn load(path: &Path) -> io::Result<Self> {
        let f = OpenOptions::new()
            .read(true)
            .custom_flags((OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK).bits() as i32)
            .open(path)?;
        let m = f.metadata()?;
        let u = rustix::process::geteuid().as_raw();
        if !m.is_file()
            || m.nlink() != 1
            || (m.uid() != 0 && m.uid() != u)
            || m.mode() & 0o022 != 0
            || m.len() > 2048
        {
            return Err(invalid());
        }
        let mut text = String::new();
        f.take(2049).read_to_string(&mut text)?;
        if text.len() > 2048 {
            return Err(invalid());
        }
        let mut t = text.parse::<toml::Table>().map_err(|_| invalid())?;
        let owner = t
            .remove("owner_uid")
            .and_then(|v| v.as_integer())
            .and_then(|v| u32::try_from(v).ok())
            .filter(|v| *v > 0)
            .ok_or_else(invalid)?;
        let mut p = |name| -> io::Result<PathBuf> {
            let s = t
                .remove(name)
                .and_then(|v| v.as_str().map(str::to_owned))
                .ok_or_else(invalid)?;
            let p = PathBuf::from(&s);
            if !p.is_absolute()
                || s.len() > 128
                || s.chars().any(char::is_control)
                || p.components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err(invalid());
            }
            Ok(p)
        };
        let socket = p("socket")?;
        let store = p("store")?;
        if !t.is_empty() {
            return Err(invalid());
        }
        Ok(Self {
            owner,
            socket,
            store,
        })
    }
    pub fn call(&self, f: &[&str]) -> io::Result<Vec<String>> {
        rpc(&self.socket, self.owner, f)
    }
}
fn private_dir(p: &Path) -> io::Result<()> {
    let m = fs::symlink_metadata(p)?;
    if !m.is_dir() || m.uid() != rustix::process::geteuid().as_raw() || m.mode() & 0o077 != 0 {
        return Err(invalid());
    }
    Ok(())
}
pub fn slots(cfg: &Config) -> io::Result<Vec<File>> {
    private_dir(&cfg.store)?;
    let mut held = Vec::new();
    for i in 0..2 {
        let p = cfg.store.join(format!("slot{i}"));
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags((OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK).bits() as i32)
            .open(p)?;
        let m = f.metadata()?;
        if !m.is_file()
            || m.nlink() != 1
            || m.uid() != rustix::process::geteuid().as_raw()
            || m.mode() & 0o077 != 0
        {
            return Err(invalid());
        }
        held.push(f)
    }
    Ok(held)
}
fn lock(f: &File) -> io::Result<()> {
    rustix::fs::flock(f, rustix::fs::FlockOperation::NonBlockingLockExclusive).map_err(Into::into)
}
fn regular(f: &File) -> io::Result<()> {
    let m = f.metadata()?;
    if !m.is_file()
        || m.nlink() != 1
        || m.uid() != rustix::process::geteuid().as_raw()
        || m.mode() & 0o077 != 0
    {
        return Err(invalid());
    }
    Ok(())
}
fn hash(f: &mut File) -> io::Result<String> {
    let mut h = Sha256::new();
    let mut b = [0u8; CHUNK];
    loop {
        let n = f.read(&mut b)?;
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    Ok(h.finalize()
        .into_iter()
        .flat_map(|b| {
            [
                b"0123456789abcdef"[(b >> 4) as usize],
                b"0123456789abcdef"[(b & 15) as usize],
            ]
        })
        .map(char::from)
        .collect())
}
fn limit() -> io::Result<()> {
    for (r, n) in [
        (Resource::Core, 0),
        (Resource::Nproc, 0),
        (Resource::Nofile, 16),
        (Resource::Data, 32 * 1024 * 1024),
        (Resource::Stack, 4 * 1024 * 1024),
        (Resource::Fsize, 16 * 1024 * 1024),
        (Resource::Cpu, 120),
    ] {
        setrlimit(
            r,
            Rlimit {
                current: Some(n),
                maximum: Some(n),
            },
        )?
    }
    Ok(())
}
fn wait(fd: impl AsFd, event: PollFlags, deadline: Instant) -> io::Result<()> {
    let d = deadline
        .checked_duration_since(Instant::now())
        .ok_or_else(|| io::Error::from(io::ErrorKind::TimedOut))?;
    let mut p = [PollFd::new(&fd, event)];
    let ts = Timespec {
        tv_sec: d.as_secs() as _,
        tv_nsec: i64::from(d.subsec_nanos()),
    };
    if poll(&mut p, Some(&ts))? == 0 || !p[0].revents().contains(event) {
        return Err(invalid());
    }
    Ok(())
}
fn read_exact(input: &mut File, b: &mut [u8], deadline: Instant) -> io::Result<()> {
    let mut n = 0;
    while n < b.len() {
        wait(&*input, PollFlags::IN, deadline)?;
        match input.read(&mut b[n..]) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(k) => n += k,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
fn output(out: &mut File, b: &[u8], wall: Instant) -> io::Result<()> {
    let deadline = std::cmp::min(wall, Instant::now() + Duration::from_secs(3));
    let mut n = 0;
    let frame = [(b.len() as u32).to_be_bytes().as_slice(), b].concat();
    while n < frame.len() {
        wait(&*out, PollFlags::OUT, deadline)?;
        match out.write(&frame[n..]) {
            Ok(0) => return Err(invalid()),
            Ok(k) => n += k,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
struct Parse<'a> {
    b: &'a [u8],
    n: usize,
}
impl<'a> Parse<'a> {
    fn u32(&mut self) -> io::Result<u32> {
        let x = self.b.get(self.n..self.n + 4).ok_or_else(invalid)?;
        self.n += 4;
        Ok(u32::from_be_bytes(x.try_into().map_err(|_| invalid())?))
    }
    fn u64(&mut self) -> io::Result<u64> {
        let a = self.u32()?;
        let b = self.u32()?;
        Ok((u64::from(a) << 32) | u64::from(b))
    }
    fn bytes(&mut self) -> io::Result<&'a [u8]> {
        let n = self.u32()? as usize;
        let b = self.b.get(self.n..self.n + n).ok_or_else(invalid)?;
        self.n += n;
        Ok(b)
    }
    fn text(&mut self) -> io::Result<&'a str> {
        std::str::from_utf8(self.bytes()?).map_err(|_| invalid())
    }
    fn end(&self) -> io::Result<()> {
        if self.n != self.b.len() {
            return Err(invalid());
        }
        Ok(())
    }
}
fn string(b: &mut Vec<u8>, s: &[u8]) {
    b.extend((s.len() as u32).to_be_bytes());
    b.extend(s)
}
fn status(id: u32, code: u32) -> Vec<u8> {
    let mut b = vec![101];
    b.extend(id.to_be_bytes());
    b.extend(code.to_be_bytes());
    string(
        &mut b,
        if code == 0 {
            b"ok"
        } else {
            b"file operation denied"
        },
    );
    string(&mut b, b"");
    b
}
fn attrs(bytes: u64, dir: bool) -> Vec<u8> {
    let mut b = 5u32.to_be_bytes().to_vec();
    b.extend(bytes.to_be_bytes());
    b.extend((if dir { 0o040700u32 } else { 0o100600u32 }).to_be_bytes());
    b
}
fn attr_input(p: &mut Parse<'_>) -> io::Result<()> {
    let f = p.u32()?;
    if f & !15 != 0 {
        return Err(invalid());
    }
    if f & 1 != 0 {
        p.u64()?;
    }
    if f & 2 != 0 {
        p.u32()?;
        p.u32()?;
    }
    if f & 4 != 0 {
        p.u32()?;
    }
    if f & 8 != 0 {
        p.u32()?;
        p.u32()?;
    }
    Ok(())
}
struct Transfer {
    cfg: Config,
    id: String,
    lease: String,
    object: String,
    name: String,
    upload: bool,
    size: u64,
    sha: String,
    position: u64,
    file: Option<File>,
    opened: bool,
    done: bool,
    listed: bool,
}
impl Drop for Transfer {
    fn drop(&mut self) {
        if !self.done {
            let _ = self.cfg.call(&["cancel", &self.id, &self.lease]);
        }
    }
}
impl Transfer {
    fn check(&self) -> io::Result<()> {
        self.cfg.call(&["check", &self.id, &self.lease])?;
        Ok(())
    }
    fn path(&self, p: &str) -> bool {
        p == self.name || p == format!("/{}", self.name)
    }
    fn object_path(&self, part: bool) -> PathBuf {
        self.cfg.store.join(if part {
            format!("{}.part", self.object)
        } else {
            self.object.clone()
        })
    }
    fn serve(&mut self, packet: &[u8]) -> io::Result<Vec<u8>> {
        let kind = *packet.first().ok_or_else(invalid)?;
        let mut p = Parse {
            b: &packet[1..],
            n: 0,
        };
        let req = p.u32()?;
        self.check()?;
        let result: io::Result<Vec<u8>> = (|| match kind {
            16 => {
                let path = p.text()?;
                p.end()?;
                if path != "." && path != "/" && !self.path(path) {
                    return Ok(status(req, 2));
                }
                let path = if self.path(path) {
                    format!("/{}", self.name)
                } else {
                    "/".into()
                };
                let mut b = vec![104];
                b.extend(req.to_be_bytes());
                b.extend(1u32.to_be_bytes());
                string(&mut b, path.as_bytes());
                string(&mut b, path.as_bytes());
                b.extend(attrs(self.size, path == "/"));
                Ok(b)
            }
            7 | 17 => {
                let path = p.text()?;
                p.end()?;
                let dir = path == "." || path == "/";
                if !dir && !self.path(path) {
                    return Ok(status(req, 2));
                }
                let mut b = vec![105];
                b.extend(req.to_be_bytes());
                b.extend(attrs(
                    if self.upload {
                        self.position
                    } else {
                        self.size
                    },
                    dir,
                ));
                Ok(b)
            }
            11 => {
                let path = p.text()?;
                p.end()?;
                if path != "." && path != "/" {
                    return Ok(status(req, 3));
                }
                let mut b = vec![102];
                b.extend(req.to_be_bytes());
                string(&mut b, b"dir");
                self.listed = false;
                Ok(b)
            }
            12 => {
                if p.bytes()? != b"dir" {
                    return Ok(status(req, 3));
                }
                p.end()?;
                if self.listed {
                    return Ok(status(req, 1));
                }
                self.listed = true;
                let mut b = vec![104];
                b.extend(req.to_be_bytes());
                b.extend(1u32.to_be_bytes());
                string(&mut b, self.name.as_bytes());
                string(&mut b, self.name.as_bytes());
                b.extend(attrs(
                    if self.upload {
                        self.position
                    } else {
                        self.size
                    },
                    false,
                ));
                Ok(b)
            }
            3 => {
                let path = p.text()?;
                let flags = p.u32()?;
                attr_input(&mut p)?;
                p.end()?;
                if !self.path(path)
                    || self.opened
                    || flags != if self.upload { 2 | 8 | 16 } else { 1 }
                {
                    return Ok(status(req, 3));
                }
                let path = self.object_path(self.upload);
                let mut options = OpenOptions::new();
                options.custom_flags(
                    (OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK).bits() as i32,
                );
                let mut file = if self.upload {
                    options
                        .write(true)
                        .create_new(true)
                        .mode(0o600)
                        .open(path)?
                } else {
                    options.read(true).open(path)?
                };
                regular(&file)?;
                if !self.upload {
                    use std::io::{Seek, SeekFrom};
                    if file.metadata()?.len() != self.size || hash(&mut file)? != self.sha {
                        return Err(invalid());
                    }
                    file.seek(SeekFrom::Start(0))?;
                }
                self.file = Some(file);
                self.opened = true;
                let mut b = vec![102];
                b.extend(req.to_be_bytes());
                string(&mut b, self.lease.as_bytes());
                Ok(b)
            }
            8 => {
                if p.bytes()? != self.lease.as_bytes() || self.file.is_none() {
                    return Ok(status(req, 3));
                }
                p.end()?;
                let mut b = vec![105];
                b.extend(req.to_be_bytes());
                b.extend(attrs(
                    if self.upload {
                        self.position
                    } else {
                        self.size
                    },
                    false,
                ));
                Ok(b)
            }
            5 => {
                if p.bytes()? != self.lease.as_bytes() || self.upload || self.file.is_none() {
                    return Ok(status(req, 3));
                }
                let offset = p.u64()?;
                let len = p.u32()? as usize;
                p.end()?;
                if offset >= self.size {
                    return Ok(status(req, 1));
                }
                if offset != self.position || len == 0 || len > CHUNK {
                    return Ok(status(req, 3));
                }
                let n = len.min((self.size - offset) as usize);
                self.cfg.call(&[
                    "chunk",
                    &self.id,
                    &self.lease,
                    &offset.to_string(),
                    &n.to_string(),
                ])?;
                let mut data = vec![0; n];
                self.file
                    .as_mut()
                    .ok_or_else(invalid)?
                    .read_exact(&mut data)?;
                self.position += n as u64;
                let mut b = vec![103];
                b.extend(req.to_be_bytes());
                string(&mut b, &data);
                Ok(b)
            }
            6 => {
                if p.bytes()? != self.lease.as_bytes() || !self.upload || self.file.is_none() {
                    return Ok(status(req, 3));
                }
                let offset = p.u64()?;
                let data = p.bytes()?;
                p.end()?;
                if offset != self.position
                    || data.len() > CHUNK
                    || offset
                        .checked_add(data.len() as u64)
                        .is_none_or(|n| n > self.size)
                {
                    return Ok(status(req, 3));
                }
                self.cfg.call(&[
                    "chunk",
                    &self.id,
                    &self.lease,
                    &offset.to_string(),
                    &data.len().to_string(),
                ])?;
                self.file.as_mut().ok_or_else(invalid)?.write_all(data)?;
                self.position += data.len() as u64;
                Ok(status(req, 0))
            }
            4 => {
                let h = p.bytes()?;
                p.end()?;
                if h == b"dir" {
                    return Ok(status(req, 0));
                }
                if h != self.lease.as_bytes() || self.file.is_none() {
                    return Ok(status(req, 3));
                }
                if self.position != self.size {
                    return Ok(status(req, 4));
                }
                if self.upload {
                    let f = self.file.take().ok_or_else(invalid)?;
                    f.sync_all()?;
                    drop(f);
                    let mut f = OpenOptions::new()
                        .read(true)
                        .custom_flags(
                            (OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK).bits() as i32,
                        )
                        .open(self.object_path(true))?;
                    regular(&f)?;
                    if f.metadata()?.len() != self.size {
                        return Err(invalid());
                    }
                    let sha = hash(&mut f)?;
                    drop(f);
                    fs::hard_link(self.object_path(true), self.object_path(false))?;
                    fs::remove_file(self.object_path(true))?;
                    File::open(&self.cfg.store)?.sync_all()?;
                    self.cfg.call(&["finish", &self.id, &self.lease, &sha])?;
                } else {
                    self.file.take();
                    self.cfg.call(&["finish", &self.id, &self.lease, ""])?;
                }
                self.done = true;
                Ok(status(req, 0))
            }
            _ => Ok(status(req, 8)),
        })();
        match result {
            Ok(b) => Ok(b),
            Err(_) => Err(invalid()),
        }
    }
}
pub fn sftp(cfg: Config, grant: String) -> io::Result<()> {
    if !id(&grant) {
        return Err(invalid());
    }
    let slots = slots(&cfg)?;
    let _slot = slots.iter().find(|f| lock(f).is_ok()).ok_or_else(invalid)?;
    limit().map_err(|_| io::Error::other("file-helper resource limits unavailable"))?;
    let v = cfg
        .call(&["start", &grant])
        .map_err(|_| io::Error::other("transfer claim denied"))?;
    if v.len() != 6
        || !id(&v[0])
        || !id(&v[5])
        || v[1].is_empty()
        || v[1].len() > 64
        || v[1].starts_with('.')
        || !v[1]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        || !matches!(v[2].as_str(), "upload" | "download")
    {
        return Err(invalid());
    }
    let size: u64 = v[3].parse().map_err(|_| invalid())?;
    if size > 16 * 1024 * 1024 {
        return Err(invalid());
    }
    let mut transfer = Transfer {
        cfg,
        id: grant,
        object: v[0].clone(),
        name: v[1].clone(),
        upload: v[2] == "upload",
        size,
        sha: v[4].clone(),
        lease: v[5].clone(),
        position: 0,
        file: None,
        opened: false,
        done: false,
        listed: false,
    };
    let mut input = File::from(rustix::io::dup(io::stdin())?);
    let mut out = File::from(rustix::io::dup(io::stdout())?);
    fcntl_setfl(&input, fcntl_getfl(&input)? | OFlags::NONBLOCK)?;
    fcntl_setfl(&out, fcntl_getfl(&out)? | OFlags::NONBLOCK)?;
    let wall = Instant::now() + Duration::from_secs(300);
    let mut initialized = false;
    for _ in 0..60000 {
        let deadline = std::cmp::min(wall, Instant::now() + Duration::from_secs(30));
        let mut h = [0; 4];
        match read_exact(&mut input, &mut h, deadline) {
            Ok(()) => (),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        }
        let len = u32::from_be_bytes(h) as usize;
        if !(5..=PACKET).contains(&len) {
            return Err(invalid());
        }
        let mut b = vec![0; len];
        read_exact(&mut input, &mut b, deadline)?;
        if !initialized {
            if b[0] != 1
                || len != 5
                || u32::from_be_bytes(b[1..].try_into().map_err(|_| invalid())?) < 3
            {
                return Err(io::Error::other("unsupported SFTP initialization"));
            }
            output(&mut out, &[2, 0, 0, 0, 3], wall)
                .map_err(|_| io::Error::other("SFTP version response failed"))?;
            initialized = true;
            continue;
        }
        let response = transfer.serve(&b)?;
        output(&mut out, &response, wall)?;
        if transfer.done {
            return Ok(());
        }
    }
    Err(invalid())
}
pub fn gc(cfg: &Config) -> io::Result<u64> {
    let held = slots(cfg)?;
    for f in &held {
        lock(f)?
    }
    let mut after = String::new();
    let mut count = 0;
    loop {
        let rows = cfg.call(&["removed", &after])?;
        if rows.is_empty() {
            break;
        }
        for object in rows {
            if !id(&object) {
                return Err(invalid());
            }
            // A crash between no-clobber publication and unlink can leave exactly
            // these two aliases. Reconcile only the matching private inode pair,
            // under both transfer locks and only after catalog removal.
            let final_path = cfg.store.join(&object);
            let part_path = cfg.store.join(format!("{object}.part"));
            if let (Ok(a), Ok(b)) = (
                fs::symlink_metadata(&final_path),
                fs::symlink_metadata(&part_path),
            ) && a.is_file()
                && b.is_file()
                && a.dev() == b.dev()
                && a.ino() == b.ino()
                && a.nlink() == 2
                && b.nlink() == 2
                && a.uid() == rustix::process::geteuid().as_raw()
                && a.mode() & 0o077 == 0
            {
                fs::remove_file(&part_path)?;
                File::open(&cfg.store)?.sync_all()?;
            }
            for suffix in ["", ".part"] {
                let p = cfg.store.join(format!("{object}{suffix}"));
                match OpenOptions::new()
                    .read(true)
                    .custom_flags(
                        (OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK).bits() as i32,
                    )
                    .open(&p)
                {
                    Ok(f) => {
                        regular(&f)?;
                        drop(f);
                        fs::remove_file(p)?;
                    }
                    Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                    Err(e) => return Err(e),
                }
            }
            File::open(&cfg.store)?.sync_all()?;
            cfg.call(&["reclaimed", &object])?;
            after = object;
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_store_refuses_symlink_hardlink_and_untrusted_mode() {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt, symlink};
        let p = std::env::temp_dir().join(format!("faction-files-store-{}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        let f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(p.join("object"))
            .unwrap();
        regular(&f).unwrap();
        fs::hard_link(p.join("object"), p.join("alias")).unwrap();
        assert!(regular(&f).is_err());
        fs::remove_file(p.join("alias")).unwrap();
        fs::set_permissions(p.join("object"), fs::Permissions::from_mode(0o644)).unwrap();
        assert!(regular(&f).is_err());
        symlink(p.join("object"), p.join("slot0")).unwrap();
        let cfg = Config {
            owner: 1,
            socket: p.join("absent"),
            store: p.clone(),
        };
        assert!(slots(&cfg).is_err());
        fs::remove_file(p.join("slot0")).unwrap();
        let first = slots(&cfg).unwrap();
        for file in &first {
            lock(file).unwrap();
        }
        let second = slots(&cfg).unwrap();
        assert!(second.iter().all(|f| lock(f).is_err()));
        drop(first);
        drop(second);
        drop(f);
        for name in ["object", "slot0", "slot1"] {
            fs::remove_file(p.join(name)).unwrap();
        }
        fs::remove_dir(p).unwrap();
    }
}
