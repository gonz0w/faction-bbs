use super::{Error, MAX_SNAPSHOT, State, codec};
use rustix::fs::{self, AtFlags, FlockOperation, Mode, OFlags};
use std::{
    fs::File,
    io::{Read, Write},
    os::unix::fs::MetadataExt,
    path::{Component, Path},
};
const NAME: &str = "private-blobs.snapshot";
const TEMP: &str = "private-blobs.pending";
const LOCK: &str = "private-blobs.lock";
pub(super) struct Disk {
    dir: File,
    _lock: File,
}
fn storage<T>(r: std::io::Result<T>) -> Result<T, Error> {
    r.map_err(|_| Error::Storage)
}
fn regular(file: &File, cap: u64) -> Result<(), Error> {
    let m = storage(file.metadata())?;
    if !m.is_file()
        || m.uid() != rustix::process::geteuid().as_raw()
        || m.mode() & 0o7777 != 0o600
        || m.nlink() != 1
        || m.len() > cap
    {
        Err(Error::Invalid)
    } else {
        Ok(())
    }
}
fn open(dir: &File, name: &str, flags: OFlags) -> Result<File, Error> {
    fs::openat(
        dir,
        name,
        flags | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::from_raw_mode(0o600),
    )
    .map(File::from)
    .map_err(|_| Error::Storage)
}
impl Disk {
    pub(super) fn open(path: &Path) -> Result<(Self, State), Error> {
        if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(Error::Invalid);
        }
        let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
        let mut dir = File::from(fs::open("/", flags, Mode::empty()).map_err(|_| Error::Storage)?);
        for c in path.components() {
            match c {
                Component::Normal(name) => {
                    dir = File::from(
                        fs::openat(&dir, name, flags, Mode::empty()).map_err(|_| Error::Storage)?,
                    )
                }
                Component::RootDir | Component::CurDir => (),
                _ => return Err(Error::Invalid),
            }
        }
        let m = storage(dir.metadata())?;
        if m.uid() != rustix::process::geteuid().as_raw() || m.mode() & 0o7777 != 0o700 {
            return Err(Error::Invalid);
        }
        let lock = open(&dir, LOCK, OFlags::CREATE | OFlags::RDWR)?;
        regular(&lock, 0)?;
        fs::flock(&lock, FlockOperation::NonBlockingLockExclusive).map_err(|e| {
            if e == rustix::io::Errno::WOULDBLOCK {
                Error::Busy
            } else {
                Error::Storage
            }
        })?;
        let disk = Self { dir, _lock: lock };
        // Refuse unrelated entries: the physical bound covers exactly one lock,
        // current snapshot and at most one known temporary, never an orphan sweep.
        for entry in fs::Dir::read_from(&disk.dir).map_err(|_| Error::Storage)? {
            let entry = entry.map_err(|_| Error::Storage)?;
            let n = entry.file_name().to_bytes();
            if ![
                b".".as_slice(),
                b"..",
                NAME.as_bytes(),
                TEMP.as_bytes(),
                LOCK.as_bytes(),
            ]
            .contains(&n)
            {
                return Err(Error::Invalid);
            }
        }
        match fs::statat(&disk.dir, TEMP, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(_) => {
                let pending = open(&disk.dir, TEMP, OFlags::RDONLY)?;
                regular(&pending, MAX_SNAPSHOT as u64)?;
                fs::unlinkat(&disk.dir, TEMP, AtFlags::empty()).map_err(|_| Error::Storage)?;
                storage(disk.dir.sync_all())?;
            }
            Err(e) if e == rustix::io::Errno::NOENT => (),
            Err(_) => return Err(Error::Storage),
        }
        let state = match fs::statat(&disk.dir, NAME, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(_) => {
                let mut file = open(&disk.dir, NAME, OFlags::RDONLY)?;
                regular(&file, MAX_SNAPSHOT as u64)?;
                let len = storage(file.metadata())?.len() as usize;
                let mut bytes = Vec::with_capacity(len);
                storage(
                    Read::by_ref(&mut file)
                        .take(MAX_SNAPSHOT as u64 + 1)
                        .read_to_end(&mut bytes),
                )?;
                if bytes.len() != len {
                    return Err(Error::Invalid);
                }
                let state = codec::decode_state(&bytes)?;
                storage(file.sync_all())?;
                storage(disk.dir.sync_all())?;
                state
            }
            Err(e) if e == rustix::io::Errno::NOENT => {
                let state = State::default();
                disk.write(&state)?;
                state
            }
            Err(_) => return Err(Error::Storage),
        };
        Ok((disk, state))
    }
    pub(super) fn write(&self, state: &State) -> Result<(), Error> {
        let bytes = codec::encode_state(state)?;
        // A leftover temp is resolved only on reopen while holding the same lock.
        let mut temp = open(
            &self.dir,
            TEMP,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL,
        )?;
        regular(&temp, 0)?;
        #[cfg(test)]
        test_step(Point::Write, &self.dir)?;
        storage(temp.write_all(&bytes))?;
        #[cfg(test)]
        test_step(Point::FileSync, &self.dir)?;
        storage(temp.sync_all())?;
        // Recheck the exact held temporary before any publication.
        let held = fs::fstat(&temp).map_err(|_| Error::Storage)?;
        let named =
            fs::statat(&self.dir, TEMP, AtFlags::SYMLINK_NOFOLLOW).map_err(|_| Error::Storage)?;
        if held.st_dev != named.st_dev || held.st_ino != named.st_ino {
            return Err(Error::Storage);
        }
        #[cfg(test)]
        test_step(Point::BeforeRename, &self.dir)?;
        fs::renameat(&self.dir, TEMP, &self.dir, NAME).map_err(|_| Error::PublicationUncertain)?;
        #[cfg(test)]
        test_step(Point::AfterRename, &self.dir).map_err(|_| Error::PublicationUncertain)?;
        #[cfg(test)]
        test_step(Point::DirectorySync, &self.dir).map_err(|_| Error::PublicationUncertain)?;
        self.dir.sync_all().map_err(|_| Error::PublicationUncertain)
    }
}
#[cfg(test)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Point {
    Write,
    FileSync,
    BeforeRename,
    AfterRename,
    DirectorySync,
}
#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) enum Fault {
    None,
    Error(Point),
    Death(Point),
}
#[cfg(test)]
thread_local! {pub(super) static FAULT:std::cell::Cell<Fault>=const{std::cell::Cell::new(Fault::None)};}
#[cfg(test)]
fn test_step(point: Point, dir: &File) -> Result<(), Error> {
    match FAULT.get() {
        Fault::Error(p) if p == point => Err(Error::Storage),
        Fault::Death(p) if p == point => {
            let mut mark = open(
                dir,
                "BOUNDARY",
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL,
            )?;
            storage(mark.write_all(b"owned issue260 exact boundary"))?;
            loop {
                std::thread::park();
            }
        }
        _ => Ok(()),
    }
}
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
