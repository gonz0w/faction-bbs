//! Explicit member-device saves of already verified attachment bytes.
//!
//! The trusted local caller selects an absolute destination in an existing owned
//! directory with exact mode 0700. Metadata never selects a path. This bounded
//! contract does not isolate hostile root, another process with the same UID, or
//! ACL access. A failed or interrupted operation can leave a private temporary:
//! inspect it deliberately; there is no automatic cleanup sweep or erasure claim.
use faction_omemo::blob::VerifiedBlob;
use rustix::fs::{self, AtFlags, Mode, OFlags};
#[cfg(test)]
use std::ffi::OsStr;
use std::{
    ffi::OsString,
    fmt,
    fs::File,
    io::Write,
    os::unix::fs::MetadataExt,
    path::{Component, Path},
};

/// Whether this operation could have published the complete verified file.
/// Diagnostics deliberately contain no path, metadata, or private content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveError {
    /// This operation created no final name; an existing/racing object is intact.
    BeforePublication,
    /// The destination may contain the complete file. Never assume rollback.
    PublicationUncertain,
}
impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BeforePublication => "attachment save failed before publication",
            Self::PublicationUncertain => "attachment publication may have completed",
        })
    }
}
impl std::error::Error for SaveError {}

fn publication_error(error: rustix::io::Errno) -> SaveError {
    // A destination collision is a definite refusal. Conservatively retain
    // uncertainty for all other failures after attempting publication, including
    // ambiguous remote-filesystem I/O and timeout responses.
    if error == rustix::io::Errno::EXIST {
        SaveError::BeforePublication
    } else {
        SaveError::PublicationUncertain
    }
}

fn parent(path: &Path) -> Result<(File, OsString), SaveError> {
    let before = SaveError::BeforePublication;
    if !path.is_absolute() || path.components().any(|p| matches!(p, Component::ParentDir)) {
        return Err(before);
    }
    let mut parts = path.components().collect::<Vec<_>>();
    let Some(Component::Normal(name)) = parts.pop() else {
        return Err(before);
    };
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut dir = File::from(fs::open("/", flags, Mode::empty()).map_err(|_| before)?);
    for part in parts {
        match part {
            Component::Normal(name) => {
                dir = File::from(fs::openat(&dir, name, flags, Mode::empty()).map_err(|_| before)?);
            }
            Component::RootDir | Component::CurDir => {}
            _ => return Err(before),
        }
    }
    let metadata = dir.metadata().map_err(|_| before)?;
    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o7777 != 0o700 {
        return Err(before);
    }
    Ok((dir, name.to_os_string()))
}

// Check the held inode before removing a name. Never sweep old temporaries or
// remove a substituted object. Same-UID concurrent mutation is outside isolation.
fn owned_name(dir: &File, name: &str, file: &File) -> bool {
    let Ok(held) = fs::fstat(file) else {
        return false;
    };
    let Ok(named) = fs::statat(dir, name, AtFlags::SYMLINK_NOFOLLOW) else {
        return false;
    };
    held.st_dev == named.st_dev && held.st_ino == named.st_ino
}
fn remove_owned(dir: &File, name: &str, file: &File) -> bool {
    owned_name(dir, name, file) && fs::unlinkat(dir, name, AtFlags::empty()).is_ok()
}

/// Save immutable verified bytes without replacing any destination object.
///
/// The caller must authenticate the descriptor before blob verification and make
/// this call only after deliberate local selection. Cancellation is no call.
/// Publication uses a no-replace hard link after file sync; errors after that
/// boundary report uncertainty and never delete the destination. No power-cut
/// guarantee is implied, including when directory sync fails.
pub fn save_verified(blob: &VerifiedBlob, destination: &Path) -> Result<(), SaveError> {
    let before = SaveError::BeforePublication;
    let uncertain = SaveError::PublicationUncertain;
    let (dir, name) = parent(destination)?;
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).map_err(|_| before)?;
    let temp = format!(
        ".faction-attachment-{}",
        random
            .iter()
            .map(|v| format!("{v:02x}"))
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
        // Set only the exclusively created owned inode, without changing the
        // caller's process-wide umask. A restrictive mask cannot weaken or
        // prevent the exact private output mode.
        fs::fchmod(&file, Mode::from_raw_mode(0o600)).map_err(|_| before)?;
        let metadata = file.metadata().map_err(|_| before)?;
        if !metadata.is_file()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o7777 != 0o600
            || metadata.nlink() != 1
        {
            return Err(before);
        }
        #[cfg(test)]
        test_step(Point::Write, &dir, &temp, &name).map_err(|_| before)?;
        file.write_all(blob.as_bytes()).map_err(|_| before)?;
        #[cfg(test)]
        test_step(Point::FileSync, &dir, &temp, &name).map_err(|_| before)?;
        file.sync_all().map_err(|_| before)?;
        #[cfg(test)]
        test_step(Point::BeforeLink, &dir, &temp, &name).map_err(|_| before)?;
        if !owned_name(&dir, &temp, &file) {
            return Err(before);
        }
        #[cfg(test)]
        test_step(Point::Link, &dir, &temp, &name).map_err(|_| uncertain)?;
        fs::linkat(
            &dir,
            temp.as_str(),
            &dir,
            name.as_os_str(),
            AtFlags::empty(),
        )
        .map_err(publication_error)?;
        #[cfg(test)]
        test_step(Point::AfterLink, &dir, &temp, &name).map_err(|_| uncertain)?;
        #[cfg(test)]
        test_step(Point::Unlink, &dir, &temp, &name).map_err(|_| uncertain)?;
        if !remove_owned(&dir, &temp, &file) {
            return Err(uncertain);
        }
        #[cfg(test)]
        test_step(Point::DirectorySync, &dir, &temp, &name).map_err(|_| uncertain)?;
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
    Unlink,
    DirectorySync,
}
#[cfg(test)]
#[derive(Clone, Copy, Eq, PartialEq)]
enum Fault {
    None,
    Error(Point),
    Race,
    ReplaceTemp,
    Death(Point),
}
#[cfg(test)]
thread_local! { static FAULT: std::cell::Cell<Fault> = const { std::cell::Cell::new(Fault::None) }; }
#[cfg(test)]
fn test_step(point: Point, dir: &File, temp: &str, name: &OsStr) -> std::io::Result<()> {
    use std::io;
    match FAULT.get() {
        Fault::Error(selected) if selected == point => {
            if point == Point::Write {
                let mut partial = File::from(fs::openat(dir, temp, OFlags::WRONLY, Mode::empty())?);
                partial.write_all(b"partial test-only write")?;
            }
            Err(io::Error::other("injected attachment save failure"))
        }
        Fault::Death(selected) if selected == point => {
            let mut marker = File::from(fs::openat(
                dir,
                "BOUNDARY",
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL,
                Mode::from_raw_mode(0o600),
            )?);
            marker.write_all(b"owned child reached exact save boundary")?;
            loop {
                std::thread::park();
            }
        }
        Fault::Race if point == Point::BeforeLink => {
            let mut racing = File::from(fs::openat(
                dir,
                name,
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL,
                Mode::from_raw_mode(0o600),
            )?);
            racing.write_all(b"racing object")
        }
        Fault::ReplaceTemp if point == Point::BeforeLink => {
            fs::unlinkat(dir, temp, AtFlags::empty())?;
            let mut other = File::from(fs::openat(
                dir,
                temp,
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL,
                Mode::from_raw_mode(0o600),
            )?);
            other.write_all(b"unknown replacement")
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use faction_omemo::{attachment::Descriptor, blob};
    use std::{
        fs as disk,
        os::unix::fs::DirBuilderExt,
        path::PathBuf,
        process::{Child, Command, Stdio},
        time::{Duration, Instant},
    };
    use zeroize::Zeroizing;
    const BYTES: &[u8] = b"complete verified bytes, including \0 and \xff";
    #[test]
    fn publication_syscall_errors_preserve_known_phase_results() {
        assert_eq!(
            publication_error(rustix::io::Errno::IO),
            SaveError::PublicationUncertain
        );
        assert_eq!(
            publication_error(rustix::io::Errno::INTR),
            SaveError::PublicationUncertain
        );
        assert_eq!(
            publication_error(rustix::io::Errno::EXIST),
            SaveError::BeforePublication
        );
    }
    #[test]
    fn timeout_after_publication_syscall_attempt_is_uncertain() {
        assert_eq!(
            publication_error(rustix::io::Errno::TIMEDOUT),
            SaveError::PublicationUncertain
        );
    }
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            FAULT.set(Fault::None);
            let mut random = [0; 16];
            getrandom::fill(&mut random).unwrap();
            let suffix = random
                .iter()
                .map(|v| format!("{v:02x}"))
                .collect::<String>();
            let root = std::env::temp_dir()
                .canonicalize()
                .unwrap()
                .join(format!("faction255-fault-{suffix}"));
            disk::DirBuilder::new().mode(0o700).create(&root).unwrap();
            disk::write(root.join("OWNER"), b"Owned issue 255 fault fixture").unwrap();
            disk::DirBuilder::new()
                .mode(0o700)
                .create(root.join("selected"))
                .unwrap();
            Self(root)
        }
        fn parent(&self) -> PathBuf {
            self.0.join("selected")
        }
        fn destination(&self) -> PathBuf {
            self.parent().join("chosen")
        }
        fn temps(&self) -> Vec<PathBuf> {
            disk::read_dir(self.parent())
                .unwrap()
                .map(|v| v.unwrap().path())
                .filter(|v| {
                    v.file_name()
                        .unwrap()
                        .as_encoded_bytes()
                        .starts_with(b".faction-attachment-")
                })
                .collect()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            FAULT.set(Fault::None);
            assert_eq!(
                disk::read(self.0.join("OWNER")).unwrap(),
                b"Owned issue 255 fault fixture"
            );
            disk::remove_dir_all(&self.0).unwrap();
            assert!(!self.0.exists());
        }
    }
    fn verified() -> VerifiedBlob {
        let encrypted = blob::encrypt(BYTES).unwrap();
        let descriptor = Descriptor {
            name: Zeroizing::new("unselected.txt".into()),
            media_type: Zeroizing::new("text/plain".into()),
            size: encrypted.size,
            hash: encrypted.hash,
            key: encrypted.key,
            iv: encrypted.iv,
            locator: Zeroizing::new("https://blob.test/opaque".into()),
        };
        blob::verify(&encrypted.ciphertext, &descriptor).unwrap()
    }
    #[test]
    fn deterministic_failures_keep_publication_phase_and_complete_output() {
        let verified = verified();
        for point in [
            Point::Write,
            Point::FileSync,
            Point::BeforeLink,
            Point::Link,
            Point::AfterLink,
            Point::Unlink,
            Point::DirectorySync,
        ] {
            let f = Fixture::new();
            FAULT.set(Fault::Error(point));
            let result = save_verified(&verified, &f.destination());
            let published = matches!(
                point,
                Point::AfterLink | Point::Unlink | Point::DirectorySync
            );
            let phase = if published || point == Point::Link {
                SaveError::PublicationUncertain
            } else {
                SaveError::BeforePublication
            };
            assert_eq!(result, Err(phase));
            if published {
                assert_eq!(disk::read(f.destination()).unwrap(), BYTES);
            } else {
                assert!(!f.destination().exists());
            }
            assert!(f.temps().is_empty());
        }
        assert_eq!(
            SaveError::BeforePublication.to_string(),
            "attachment save failed before publication"
        );
        assert_eq!(
            SaveError::PublicationUncertain.to_string(),
            "attachment publication may have completed"
        );
    }
    #[test]
    fn racing_destination_and_unknown_temporary_are_never_removed() {
        let verified = verified();
        let f = Fixture::new();
        FAULT.set(Fault::Race);
        assert_eq!(
            save_verified(&verified, &f.destination()),
            Err(SaveError::BeforePublication)
        );
        assert_eq!(disk::read(f.destination()).unwrap(), b"racing object");
        assert!(f.temps().is_empty());
        let f = Fixture::new();
        FAULT.set(Fault::ReplaceTemp);
        assert_eq!(
            save_verified(&verified, &f.destination()),
            Err(SaveError::BeforePublication)
        );
        assert!(!f.destination().exists());
        let unknown = f.temps();
        assert_eq!(unknown.len(), 1);
        assert_eq!(disk::read(&unknown[0]).unwrap(), b"unknown replacement");
        // Fixture owner deliberately inspects/removes its own test replacement.
        disk::remove_file(&unknown[0]).unwrap();
    }
    struct ChildGuard(Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    #[test]
    fn strict_umask_child() {
        let Some(root) = std::env::var_os("FACTION255_OWNED_UMASK_CHILD") else {
            return;
        };
        let root = PathBuf::from(root);
        assert_eq!(
            disk::read(root.join("OWNER")).unwrap(),
            b"Owned issue 255 fault fixture"
        );
        let verified = verified();
        // Only this disposable child changes its process-local creation mask.
        rustix::process::umask(Mode::from_raw_mode(0o777));
        save_verified(&verified, &root.join("selected/chosen")).unwrap();
        assert_eq!(
            disk::metadata(root.join("selected/chosen")).unwrap().mode() & 0o7777,
            0o600
        );
    }
    #[test]
    fn exclusive_owned_file_is_exactly_private_even_with_restrictive_umask() {
        let f = Fixture::new();
        let child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "attachment_save::tests::strict_umask_child",
                "--test-threads=1",
            ])
            .env("FACTION255_OWNED_UMASK_CHILD", &f.0)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut child = ChildGuard(child);
        let deadline = Instant::now() + Duration::from_secs(5);
        let status = loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "bounded umask child wait expired"
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(
            status.success(),
            "strict umask child rejected explicit private save"
        );
        assert_eq!(disk::read(f.destination()).unwrap(), BYTES);
        assert!(f.temps().is_empty());
    }
    #[test]
    fn death_child() {
        let Some(root) = std::env::var_os("FACTION255_OWNED_CHILD") else {
            return;
        };
        let root = PathBuf::from(root);
        assert_eq!(
            disk::read(root.join("OWNER")).unwrap(),
            b"Owned issue 255 fault fixture"
        );
        let point = match std::env::var("FACTION255_BOUNDARY").unwrap().as_str() {
            "before" => Point::BeforeLink,
            "after" => Point::AfterLink,
            _ => panic!("invalid owned test boundary"),
        };
        let verified = verified();
        FAULT.set(Fault::Death(point));
        save_verified(&verified, &root.join("selected/chosen")).unwrap();
        panic!("child passed the selected boundary");
    }
    #[test]
    fn bounded_child_death_before_and_after_publication_retains_recoverable_private_temp() {
        for boundary in ["before", "after"] {
            let f = Fixture::new();
            let child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "attachment_save::tests::death_child",
                    "--test-threads=1",
                ])
                .env("FACTION255_OWNED_CHILD", &f.0)
                .env("FACTION255_BOUNDARY", boundary)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let mut child = ChildGuard(child);
            let deadline = Instant::now() + Duration::from_secs(5);
            while !f.parent().join("BOUNDARY").exists() {
                assert!(
                    child.0.try_wait().unwrap().is_none(),
                    "child exited before boundary"
                );
                assert!(
                    Instant::now() < deadline,
                    "bounded child boundary wait expired"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            child.0.kill().unwrap();
            let status = child.0.wait().unwrap();
            assert!(!status.success());
            if boundary == "before" {
                assert!(!f.destination().exists());
            } else {
                assert_eq!(disk::read(f.destination()).unwrap(), BYTES);
            }
            let temps = f.temps();
            assert_eq!(temps.len(), 1);
            let metadata = disk::metadata(&temps[0]).unwrap();
            assert_eq!(metadata.mode() & 0o7777, 0o600);
            assert_eq!(disk::read(&temps[0]).unwrap(), BYTES);
            // Deliberate owner recovery of the identified test inode, no sweep.
            disk::remove_file(&temps[0]).unwrap();
            disk::remove_file(f.parent().join("BOUNDARY")).unwrap();
            assert!(f.temps().is_empty());
        }
    }
}
