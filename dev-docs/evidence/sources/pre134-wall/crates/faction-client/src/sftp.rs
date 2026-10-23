//! One bounded SFTP operation through the profile's fixed OpenSSH subsystem.
use crate::{
    profile::Profile,
    transport::{nonblock, read_all, write_all},
};
use faction_files::invalid;
use rustix::fs::OFlags;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Stdio},
    time::{Duration, Instant},
};
const CHUNK: usize = 32768;
const LIMIT: u64 = 16 * 1024 * 1024;
pub fn filename(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && !s.starts_with('.')
        && !s.contains("..")
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
pub fn source(path: &Path) -> io::Result<File> {
    let f = OpenOptions::new()
        .read(true)
        .custom_flags((OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC).bits() as i32)
        .open(path)?;
    let m = f.metadata()?;
    if !m.is_file() || m.uid() != rustix::process::geteuid().as_raw() || m.len() > LIMIT {
        return Err(invalid());
    }
    Ok(f)
}
struct Partial {
    path: PathBuf,
    file: Option<File>,
}
impl Drop for Partial {
    fn drop(&mut self) {
        if let Some(f) = self.file.take()
            && let (Ok(a), Ok(b)) = (f.metadata(), fs::symlink_metadata(&self.path))
            && a.dev() == b.dev()
            && a.ino() == b.ino()
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}
impl Partial {
    fn new(destination: &Path) -> io::Result<Self> {
        if destination.file_name().is_none() || destination.exists() || destination.is_symlink() {
            return Err(invalid());
        }
        let parent = destination
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let m = fs::symlink_metadata(parent)?;
        if !m.is_dir() || m.uid() != rustix::process::geteuid().as_raw() || m.mode() & 0o022 != 0 {
            return Err(invalid());
        }
        let mut random = [0; 16];
        getrandom::fill(&mut random).map_err(|_| invalid())?;
        let suffix: String = random.iter().map(|b| format!("{b:02x}")).collect();
        let path = parent.join(format!(".faction-{suffix}.part"));
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags((OFlags::NOFOLLOW | OFlags::CLOEXEC).bits() as i32)
            .open(&path)?;
        Ok(Self {
            path,
            file: Some(file),
        })
    }
    fn finish(mut self, destination: &Path) -> io::Result<()> {
        self.file.as_mut().ok_or_else(invalid)?.sync_all()?;
        fs::hard_link(&self.path, destination)?;
        fs::remove_file(&self.path)?;
        self.file.take();
        File::open(
            destination
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new(".")),
        )?
        .sync_all()
    }
}
struct Wire {
    child: Child,
    input: ChildStdin,
    output: ChildStdout,
    sequence: u32,
    wall: Instant,
}
impl Drop for Wire {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
fn string(b: &mut Vec<u8>, s: &[u8]) {
    b.extend((s.len() as u32).to_be_bytes());
    b.extend(s);
}
fn bytes(b: &[u8]) -> io::Result<&[u8]> {
    if b.len() < 4 {
        return Err(invalid());
    }
    let n = u32::from_be_bytes(b[..4].try_into().map_err(|_| invalid())?) as usize;
    if b.len() != n + 4 {
        return Err(invalid());
    }
    Ok(&b[4..])
}
fn okay(b: &[u8]) -> io::Result<()> {
    if b.first() != Some(&101) || b.get(5..9) != Some([0, 0, 0, 0].as_slice()) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "SFTP operation denied",
        ));
    }
    Ok(())
}
impl Wire {
    fn start(profile: &Profile) -> io::Result<Self> {
        let mut child = profile
            .ssh(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let input = child.stdin.take().ok_or_else(invalid)?;
        let output = child.stdout.take().ok_or_else(invalid)?;
        nonblock(&input)?;
        nonblock(&output)?;
        let mut w = Self {
            child,
            input,
            output,
            sequence: 0,
            wall: Instant::now() + Duration::from_secs(300),
        };
        let version = w.packet(&[1, 0, 0, 0, 3])?;
        if version != [2, 0, 0, 0, 3] {
            return Err(invalid());
        }
        Ok(w)
    }
    fn packet(&mut self, b: &[u8]) -> io::Result<Vec<u8>> {
        let deadline = self.wall.min(Instant::now() + Duration::from_secs(15));
        write_all(&mut self.input, &(b.len() as u32).to_be_bytes(), deadline)?;
        write_all(&mut self.input, b, deadline)?;
        let mut h = [0; 4];
        read_all(&mut self.output, &mut h, deadline)?;
        let n = u32::from_be_bytes(h) as usize;
        if !(5..=CHUNK + 1024).contains(&n) {
            return Err(invalid());
        }
        let mut out = vec![0; n];
        read_all(&mut self.output, &mut out, deadline)?;
        Ok(out)
    }
    fn call(&mut self, kind: u8, args: &[u8]) -> io::Result<Vec<u8>> {
        self.sequence = self.sequence.checked_add(1).ok_or_else(invalid)?;
        let mut b = vec![kind];
        b.extend(self.sequence.to_be_bytes());
        b.extend(args);
        let out = self.packet(&b)?;
        if out.get(1..5) != Some(self.sequence.to_be_bytes().as_slice()) {
            return Err(invalid());
        }
        Ok(out)
    }
}
pub fn transfer(
    profile: &Profile,
    row: &[String],
    local: &Path,
    mut upload: Option<File>,
    mut progress: impl FnMut(u64, u64) -> io::Result<()>,
) -> io::Result<String> {
    if row.len() != 6
        || row[0] != "grant"
        || !matches!(row[1].as_str(), "upload" | "download")
        || !filename(&row[2])
        || !faction_files::id(&row[5])
    {
        return Err(invalid());
    }
    let size = row[3].parse::<u64>().map_err(|_| invalid())?;
    let sending = upload.is_some();
    if size > LIMIT
        || sending != (row[1] == "upload")
        || (!sending
            && (row[4].len() != 64
                || !row[4]
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())))
    {
        return Err(invalid());
    }
    let mut partial = if sending {
        None
    } else {
        Some(Partial::new(local)?)
    };
    let mut w = Wire::start(profile)?;
    let mut args = Vec::new();
    string(&mut args, row[2].as_bytes());
    args.extend((if sending { 26u32 } else { 1 }).to_be_bytes());
    args.extend(0u32.to_be_bytes());
    let response = w.call(3, &args)?;
    if response[0] != 102 {
        return Err(invalid());
    }
    let handle = bytes(&response[5..])?.to_vec();
    if handle.len() > 64 {
        return Err(invalid());
    }
    let mut offset = 0;
    let mut hash = Sha256::new();
    progress(0, size)?;
    while offset < size {
        let n = ((size - offset) as usize).min(CHUNK);
        let mut args = Vec::new();
        string(&mut args, &handle);
        args.extend(offset.to_be_bytes());
        if let Some(f) = upload.as_mut() {
            let mut data = vec![0; n];
            f.read_exact(&mut data)?;
            hash.update(&data);
            string(&mut args, &data);
            okay(&w.call(6, &args)?)?;
        } else {
            args.extend((n as u32).to_be_bytes());
            let response = w.call(5, &args)?;
            if response[0] != 103 {
                return Err(invalid());
            }
            let data = bytes(&response[5..])?;
            if data.len() != n {
                return Err(invalid());
            }
            hash.update(data);
            partial
                .as_mut()
                .and_then(|p| p.file.as_mut())
                .ok_or_else(invalid)?
                .write_all(data)?;
        }
        offset += n as u64;
        progress(offset, size)?;
    }
    if let Some(f) = upload.as_mut() {
        let mut extra = [0];
        if f.read(&mut extra)? != 0 {
            return Err(invalid());
        }
    }
    let digest: String = hash
        .finalize()
        .into_iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if !sending && digest != row[4] {
        return Err(io::Error::other("download checksum mismatch"));
    }
    let mut args = Vec::new();
    string(&mut args, &handle);
    okay(&w.call(4, &args)?)?;
    if let Some(p) = partial.take() {
        p.finish(local)?;
    }
    Ok(digest)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_download_is_private_and_never_overwrites_or_leaves_failed_bytes() {
        use std::os::unix::fs::DirBuilderExt;
        let d = std::env::temp_dir().join(format!("faction-client-save-{}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&d).unwrap();
        let target = d.join("save");
        {
            let mut p = Partial::new(&target).unwrap();
            p.file.as_mut().unwrap().write_all(b"partial").unwrap();
        }
        assert_eq!(fs::read_dir(&d).unwrap().count(), 0);
        let mut p = Partial::new(&target).unwrap();
        p.file.as_mut().unwrap().write_all(b"new").unwrap();
        fs::write(&target, b"existing").unwrap();
        assert!(p.finish(&target).is_err());
        assert_eq!(fs::read(&target).unwrap(), b"existing");
        fs::remove_file(&target).unwrap();
        let mut p = Partial::new(&target).unwrap();
        p.file.as_mut().unwrap().write_all(b"complete").unwrap();
        p.finish(&target).unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"complete");
        fs::remove_file(target).unwrap();
        fs::remove_dir(d).unwrap();
    }
}
