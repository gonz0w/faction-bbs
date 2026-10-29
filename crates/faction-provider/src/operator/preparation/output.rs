//! One bounded no-replace publication, distinct from policy application.
use super::Error;
use crate::operator::input;
use rustix::fs::{self, AtFlags, Mode, OFlags};
use std::{fs::File, io::Write, os::unix::fs::MetadataExt, path::Path};
fn owned_name(dir: &File, name: &str, file: &File) -> bool {
    let Ok(a) = fs::fstat(file) else {
        return false;
    };
    let Ok(b) = fs::statat(dir, name, AtFlags::SYMLINK_NOFOLLOW) else {
        return false;
    };
    (a.st_dev, a.st_ino) == (b.st_dev, b.st_ino)
}
fn remove_owned(dir: &File, name: &str, file: &File) -> bool {
    owned_name(dir, name, file) && fs::unlinkat(dir, name, AtFlags::empty()).is_ok()
}
pub(super) fn publish(bytes: &[u8], output: &Path, root: &Path) -> Result<(), Error> {
    let before = Error::BeforePublication;
    let uncertain = Error::PublicationUncertain;
    if bytes.is_empty() || bytes.len() > 65_536 {
        return Err(before);
    }
    let (dir, name) = input::parent(output, root).map_err(|_| before)?;
    let mut random = [0; 16];
    openssl::rand::rand_bytes(&mut random).map_err(|_| before)?;
    let temp = format!(
        ".faction-proposal-{}",
        random
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let mut file = File::from(
        fs::openat(
            &dir,
            temp.as_str(),
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )
        .map_err(|_| before)?,
    );
    let result = (|| {
        fs::fchmod(&file, Mode::from_raw_mode(0o600)).map_err(|_| before)?;
        let m = file.metadata().map_err(|_| before)?;
        if !m.is_file()
            || m.uid() != rustix::process::geteuid().as_raw()
            || m.mode() & 0o7777 != 0o600
            || m.nlink() != 1
        {
            return Err(before);
        }
        #[cfg(test)]
        point(Point::Write).map_err(|_| before)?;
        file.write_all(bytes).map_err(|_| before)?;
        #[cfg(test)]
        point(Point::FileSync).map_err(|_| before)?;
        file.sync_all().map_err(|_| before)?;
        #[cfg(test)]
        point(Point::BeforeLink).map_err(|_| before)?;
        if !owned_name(&dir, &temp, &file) {
            return Err(before);
        }
        #[cfg(test)]
        point(Point::Link).map_err(|_| uncertain)?;
        fs::linkat(&dir, temp.as_str(), &dir, &name, AtFlags::empty()).map_err(|e| {
            if e == rustix::io::Errno::EXIST {
                before
            } else {
                uncertain
            }
        })?;
        #[cfg(test)]
        point(Point::AfterLink).map_err(|_| uncertain)?;
        #[cfg(test)]
        point(Point::Remove).map_err(|_| uncertain)?;
        if !remove_owned(&dir, &temp, &file) {
            return Err(uncertain);
        }
        #[cfg(test)]
        point(Point::DirectorySync).map_err(|_| uncertain)?;
        dir.sync_all().map_err(|_| uncertain)
    })();
    if result.is_err() && remove_owned(&dir, &temp, &file) {
        let _ = dir.sync_all();
    }
    result
}
#[cfg(test)]
#[derive(Clone, Copy, Eq, PartialEq)]
enum Point {
    Write,
    FileSync,
    BeforeLink,
    Link,
    AfterLink,
    Remove,
    DirectorySync,
}
#[cfg(test)]
thread_local! {static FAULT:std::cell::Cell<Option<Point>>=const{std::cell::Cell::new(None)};}
#[cfg(test)]
fn point(p: Point) -> std::io::Result<()> {
    if FAULT.with(|f| f.get() == Some(p)) {
        Err(std::io::Error::other("synthetic publication failure"))
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{os::unix::fs::DirBuilderExt, path::PathBuf};
    struct Owned(PathBuf);
    impl Owned {
        fn new() -> Self {
            let mut nonce = [0; 16];
            openssl::rand::rand_bytes(&mut nonce).unwrap();
            let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
                "faction287-output-OWNED-{}",
                nonce.iter().map(|b| format!("{b:02x}")).collect::<String>()
            ));
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&path)
                .unwrap();
            std::fs::write(path.join("OWNED"), b"issue287 output fault controls").unwrap();
            Self(path)
        }
    }
    impl Drop for Owned {
        fn drop(&mut self) {
            FAULT.with(|f| f.set(None));
            assert_eq!(
                std::fs::read(self.0.join("OWNED")).unwrap(),
                b"issue287 output fault controls"
            );
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn atomic_publication_faults_preserve_before_and_uncertain_phases() {
        for fault in [
            Point::Write,
            Point::FileSync,
            Point::BeforeLink,
            Point::Link,
            Point::AfterLink,
            Point::Remove,
            Point::DirectorySync,
        ] {
            let f = Owned::new();
            let path = f.0.join("candidate.toml");
            FAULT.with(|v| v.set(Some(fault)));
            let error = publish(b"complete approved candidate", &path, &f.0).unwrap_err();
            let attempted = matches!(
                fault,
                Point::Link | Point::AfterLink | Point::Remove | Point::DirectorySync
            );
            let published = matches!(
                fault,
                Point::AfterLink | Point::Remove | Point::DirectorySync
            );
            assert_eq!(
                error,
                if attempted {
                    Error::PublicationUncertain
                } else {
                    Error::BeforePublication
                }
            );
            assert_eq!(path.exists(), published);
            if published {
                assert_eq!(std::fs::read(path).unwrap(), b"complete approved candidate");
            }
            assert!(!std::fs::read_dir(&f.0).unwrap().any(|e| {
                e.unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".faction-proposal-")
            }));
        }
    }
}
