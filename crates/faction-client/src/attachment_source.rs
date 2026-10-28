//! Explicit member-device selection of bounded local attachment bytes.
//!
//! Paths are literal local selections, never shell input. Each path component is
//! opened without following symlinks. The held regular inode and final name are
//! checked before/after reading; observed changes cause refusal. This is not an
//! atomic snapshot or isolation from hostile root or another process with the
//! same UID. ACL confidentiality and remote filesystem behavior are not qualified.
use rustix::fs::{self, AtFlags, FileType, Mode, OFlags, Stat};
use std::{
    ffi::OsString,
    fmt,
    fs::File,
    io::{self, Read},
    os::unix::ffi::OsStrExt,
    path::{Component, Path},
};
use zeroize::Zeroizing;

const MAX_BYTES: usize = 1024 * 1024;

/// Fixed local status only: no path, operating-system error, metadata or content.
/// This type intentionally has no Debug implementation. The UI may display its
/// fixed Display text or match these variants without retaining source details.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Error {
    /// Invalid path, unsafe object, permissions, ownership, or open failure.
    Refused,
    /// Initial length or the bounded detection byte exceeds the 1 MiB limit.
    TooLarge,
    /// The held inode, selected name, metadata or read length visibly changed.
    Changed,
    /// Reading failed; the underlying error is deliberately discarded.
    ReadFailed,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Refused => "attachment source refused",
            Self::TooLarge => "attachment source exceeds 1 MiB",
            Self::Changed => "attachment source changed while reading",
            Self::ReadFailed => "attachment source could not be read",
        })
    }
}

fn open_source(path: &Path) -> Result<(File, OsString, File), Error> {
    // Inspect raw components before Path::components normalizes interior dots.
    let raw = path.as_os_str().as_bytes();
    if !path.is_absolute()
        || raw.iter().any(|byte| byte.is_ascii_control())
        || path.to_string_lossy().chars().any(char::is_control)
        || raw
            .split(|byte| *byte == b'/')
            .any(|part| part == b"." || part == b"..")
    {
        return Err(Error::Refused);
    }
    let mut parts = path.components().collect::<Vec<_>>();
    let Some(Component::Normal(name)) = parts.pop() else {
        return Err(Error::Refused);
    };
    let flags =
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK;
    let mut parent = File::from(fs::open("/", flags, Mode::empty()).map_err(|_| Error::Refused)?);
    for part in parts {
        match part {
            Component::RootDir => {}
            Component::Normal(name) => {
                parent = File::from(
                    fs::openat(&parent, name, flags, Mode::empty()).map_err(|_| Error::Refused)?,
                );
            }
            _ => return Err(Error::Refused),
        }
    }
    // NONBLOCK must be present on the first leaf open, before its type is known:
    // a FIFO or device cannot be allowed to wait before the fstat refusal.
    let file = File::from(
        fs::openat(
            &parent,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::empty(),
        )
        .map_err(|_| Error::Refused)?,
    );
    Ok((parent, name.to_os_string(), file))
}

fn check_source(stat: &Stat) -> Result<(), Error> {
    if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile
        || stat.st_uid != rustix::process::geteuid().as_raw()
        || stat.st_nlink != 1
        || stat.st_mode & Mode::RUSR.bits() == 0
        || stat.st_mode & (Mode::WGRP | Mode::WOTH).bits() != 0
        || stat.st_size < 0
    {
        return Err(Error::Refused);
    }
    if stat.st_size > MAX_BYTES as _ {
        return Err(Error::TooLarge);
    }
    Ok(())
}

fn unchanged(before: &Stat, after: &Stat) -> bool {
    // atime is excluded because this read may itself update it. ctime/mtime and
    // the permission/identity/length fields detect changes the filesystem exposes.
    before.st_dev == after.st_dev
        && before.st_ino == after.st_ino
        && before.st_mode == after.st_mode
        && before.st_uid == after.st_uid
        && before.st_gid == after.st_gid
        && before.st_nlink == after.st_nlink
        && before.st_size == after.st_size
        && before.st_mtime == after.st_mtime
        && before.st_mtime_nsec == after.st_mtime_nsec
        && before.st_ctime == after.st_ctime
        && before.st_ctime_nsec == after.st_ctime_nsec
}

fn read_bounded(reader: &mut impl Read) -> Result<Zeroizing<Vec<u8>>, Error> {
    // Allocate once, before reading plaintext, so growth never leaves plaintext
    // in a replaced allocation. The extra byte detects a source above the cap.
    let mut bytes = Zeroizing::new(vec![0; MAX_BYTES + 1]);
    let mut used = 0;
    while used < bytes.len() {
        match reader.read(&mut bytes[used..]) {
            Ok(0) => {
                bytes.truncate(used);
                return Ok(bytes);
            }
            Ok(count) => used += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => return Err(Error::ReadFailed),
        }
    }
    Err(Error::TooLarge)
}

/// Load exact bytes from an explicitly selected absolute local path.
///
/// Only a current-effective-UID, owner-readable, single-link regular file without
/// group/other writes is accepted (0644 is valid). Empty files and exactly 1 MiB
/// are accepted. Every plaintext buffer is zeroizing, including failure paths.
/// No content is executed, interpreted, previewed, sent or stored by this loader.
pub fn load(path: &Path) -> Result<Zeroizing<Vec<u8>>, Error> {
    load_with(path, || {})
}

// The callback allows deterministic observed-change tests at the read boundary.
// Production calls provide only the no-op above; no callback is exported.
fn load_with(path: &Path, after_read: impl FnOnce()) -> Result<Zeroizing<Vec<u8>>, Error> {
    let (parent, name, mut file) = open_source(path)?;
    let before = fs::fstat(&file).map_err(|_| Error::Refused)?;
    check_source(&before)?;
    let bytes = read_bounded(&mut file)?;
    after_read();
    let after = fs::fstat(&file).map_err(|_| Error::Changed)?;
    let named = fs::statat(&parent, name.as_os_str(), AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|_| Error::Changed)?;
    if !unchanged(&before, &after)
        || !unchanged(&before, &named)
        || bytes.len() as u64 != before.st_size as u64
    {
        return Err(Error::Changed);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::{
        fs,
        os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };
    use zeroize::Zeroizing;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
                "faction263-source-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed),
            ));
            fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
            let mut marker = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(root.join("OWNED"))
                .unwrap();
            std::io::Write::write_all(&mut marker, b"Owned issue 263 source test").unwrap();
            Self(root)
        }
        fn source(&self) -> PathBuf {
            self.0.join("source")
        }
        fn write(&self, bytes: &[u8], mode: u32) {
            // Restore only our fixture inode after an intentional unreadable-mode
            // case, before preparing the next independent permission assertion.
            if self.source().exists() {
                fs::set_permissions(self.source(), fs::Permissions::from_mode(0o600)).unwrap();
            }
            fs::write(self.source(), bytes).unwrap();
            fs::set_permissions(self.source(), fs::Permissions::from_mode(mode)).unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            assert_eq!(
                fs::read(self.0.join("OWNED")).unwrap(),
                b"Owned issue 263 source test"
            );
            fs::remove_dir_all(&self.0).unwrap();
            assert!(!self.0.exists());
        }
    }

    #[test]
    fn explicit_regular_0644_source_retains_exact_zeroizing_snapshot() {
        let fixture = Fixture::new();
        let expected = b"exact binary attachment bytes\0\xff";
        fixture.write(expected, 0o644);
        let result: Zeroizing<Vec<u8>> = match super::load(&fixture.source()) {
            Ok(bytes) => bytes,
            Err(_) => panic!("explicit current-UID regular 0644 source was refused"),
        };
        assert_eq!(result.as_slice(), expected);
    }

    fn assert_refused(path: &std::path::Path) {
        assert!(super::load(path).is_err(), "unsafe source was accepted");
    }

    #[test]
    fn empty_and_exact_cap_sources_are_accepted_but_cap_plus_one_is_refused() {
        let fixture = Fixture::new();
        for expected in [vec![], vec![0xa5; 1024 * 1024]] {
            fixture.write(&expected, 0o600);
            let actual = match super::load(&fixture.source()) {
                Ok(bytes) => bytes,
                Err(_) => panic!("bounded regular source was refused"),
            };
            assert_eq!(actual.as_slice(), expected.as_slice());
        }
        fixture.write(&vec![0xa5; 1024 * 1024 + 1], 0o600);
        assert!(matches!(
            super::load(&fixture.source()),
            Err(super::Error::TooLarge)
        ));
    }

    #[test]
    fn read_bound_includes_only_one_detection_byte_and_errors_retain_no_io_detail() {
        let mut reader = std::io::Cursor::new(vec![0x5a; 1024 * 1024 + 2]);
        assert!(matches!(
            super::read_bounded(&mut reader),
            Err(super::Error::TooLarge)
        ));
        assert_eq!(reader.position(), 1024 * 1024 + 1);
        struct Failed;
        impl std::io::Read for Failed {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("PRIVATE source path and content"))
            }
        }
        assert!(matches!(
            super::read_bounded(&mut Failed),
            Err(super::Error::ReadFailed)
        ));
    }

    #[test]
    fn explicit_source_is_literal_and_never_expands_shell_syntax() {
        let fixture = Fixture::new();
        let path = fixture.0.join("$(touch EXECUTED);`touch BACKTICK`~$HOME");
        fs::write(&path, b"literal selection").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        let actual = match super::load(&path) {
            Ok(bytes) => bytes,
            Err(_) => panic!("literal safe filename was refused"),
        };
        assert_eq!(actual.as_slice(), b"literal selection");
        assert!(!fixture.0.join("EXECUTED").exists());
        assert!(!fixture.0.join("BACKTICK").exists());
    }

    #[test]
    fn relative_dot_parent_nul_and_terminal_control_paths_are_refused() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::Path};
        let fixture = Fixture::new();
        fixture.write(b"source", 0o600);
        for path in [
            Path::new("source"),
            Path::new("./source"),
            Path::new(""),
            Path::new("/"),
        ] {
            assert_refused(path);
        }
        fs::DirBuilder::new()
            .mode(0o700)
            .create(fixture.0.join("nested"))
            .unwrap();
        for suffix in ["/./source", "/nested/../source"] {
            let path = PathBuf::from(format!("{}{suffix}", fixture.0.display()));
            // Both paths resolve to the existing permitted source if normalized.
            assert!(path.exists());
            assert_refused(&path);
        }
        for name in ["source\n", "source\x1b", "source\u{85}"] {
            let path = fixture.0.join(name);
            fs::write(&path, b"existing control-name source").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
            assert_refused(&path);
        }
        let mut nul = fixture.source().into_os_string().into_vec();
        nul.push(0);
        assert_refused(&PathBuf::from(OsString::from_vec(nul)));
    }

    #[test]
    fn final_and_ancestor_symlinks_are_refused_even_when_target_is_safe() {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        fixture.write(b"safe target", 0o600);
        let leaf = fixture.0.join("leaf-link");
        symlink(fixture.source(), &leaf).unwrap();
        assert_refused(&leaf);
        let ancestor = fixture.0.join("ancestor-link");
        symlink(&fixture.0, &ancestor).unwrap();
        assert_refused(&ancestor.join("source"));
        let dangling = fixture.0.join("dangling-link");
        symlink(fixture.0.join("absent"), &dangling).unwrap();
        assert_refused(&dangling);
    }

    #[test]
    fn fifo_socket_and_directory_are_refused_without_waiting_for_a_writer() {
        let fixture = Fixture::new();
        // Base mkfifo is used only to create this marked disposable fixture;
        // the loader itself never executes a program or invokes a shell.
        assert!(
            std::process::Command::new("/usr/bin/mkfifo")
                .args(["-m", "600"])
                .arg(fixture.0.join("fifo"))
                .status()
                .unwrap()
                .success()
        );
        let socket = fixture.0.join("socket");
        let _listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        let start = std::time::Instant::now();
        assert_refused(&fixture.0.join("fifo"));
        assert_refused(&socket);
        assert_refused(&fixture.0);
        assert!(
            start.elapsed() < std::time::Duration::from_secs(2),
            "nonregular refusal blocked"
        );
    }

    #[test]
    fn owner_unreadable_group_or_other_writable_and_hardlinked_sources_are_refused() {
        let fixture = Fixture::new();
        for mode in [0o000, 0o200, 0o040, 0o620, 0o602, 0o666] {
            fixture.write(b"permission controls", mode);
            assert_refused(&fixture.source());
        }
        fixture.write(b"single-link required", 0o644);
        fs::hard_link(fixture.source(), fixture.0.join("second-name")).unwrap();
        assert_refused(&fixture.source());
    }

    #[test]
    fn current_uid_is_required_even_for_otherwise_readable_metadata() {
        let fixture = Fixture::new();
        fixture.write(b"private selection", 0o644);
        let file = std::fs::File::open(fixture.source()).unwrap();
        let mut metadata = rustix::fs::fstat(&file).unwrap();
        metadata.st_uid ^= 1;
        assert!(matches!(
            super::check_source(&metadata),
            Err(super::Error::Refused)
        ));
    }

    #[test]
    fn observed_length_permissions_timestamps_and_link_changes_refuse_snapshot() {
        let fixture = Fixture::new();
        for change in 0..5 {
            fixture.write(b"original snapshot", 0o600);
            let result = super::load_with(&fixture.source(), || match change {
                0 => fs::OpenOptions::new()
                    .append(true)
                    .open(fixture.source())
                    .unwrap()
                    .write_all(b"growth")
                    .unwrap(),
                1 => std::fs::File::create(fixture.source())
                    .unwrap()
                    .set_len(1)
                    .unwrap(),
                2 => fs::set_permissions(fixture.source(), fs::Permissions::from_mode(0o640))
                    .unwrap(),
                3 => std::fs::File::open(fixture.source())
                    .unwrap()
                    .set_modified(UNIX_EPOCH + std::time::Duration::from_secs(1234))
                    .unwrap(),
                4 => fs::hard_link(fixture.source(), fixture.0.join("changed-link")).unwrap(),
                _ => unreachable!(),
            });
            assert!(matches!(result, Err(super::Error::Changed)));
            if change == 4 {
                fs::remove_file(fixture.0.join("changed-link")).unwrap();
            }
        }
    }

    #[test]
    fn observed_leaf_replacement_deletion_and_symlink_are_refused() {
        let fixture = Fixture::new();
        for change in 0..3 {
            fixture.write(b"original held inode", 0o600);
            let result = super::load_with(&fixture.source(), || {
                fs::rename(fixture.source(), fixture.0.join("held-old")).unwrap();
                match change {
                    0 => fixture.write(b"replacement", 0o600),
                    1 => {}
                    2 => std::os::unix::fs::symlink(fixture.0.join("held-old"), fixture.source())
                        .unwrap(),
                    _ => unreachable!(),
                }
            });
            assert!(matches!(result, Err(super::Error::Changed)));
            if change != 1 {
                fs::remove_file(fixture.source()).unwrap();
            }
            fs::remove_file(fixture.0.join("held-old")).unwrap();
        }
    }

    #[test]
    fn errors_have_only_fixed_private_status_text() {
        for (error, expected) in [
            (super::Error::Refused, "attachment source refused"),
            (super::Error::TooLarge, "attachment source exceeds 1 MiB"),
            (
                super::Error::Changed,
                "attachment source changed while reading",
            ),
            (
                super::Error::ReadFailed,
                "attachment source could not be read",
            ),
        ] {
            assert_eq!(error.to_string(), expected);
        }
    }
}
