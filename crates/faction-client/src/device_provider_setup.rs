//! Explicit offline preparation. No endpoint connection or checkpoint/spool write.
use crate::device_provider_profile as profile;
use faction_client::{mail_checkpoint::CheckpointVault, mail_data::hex};
use faction_files::private_blobs::Principal;
use faction_provider::preparation::{Card, MAX_CARD, MAX_PEM, certificate_sha256};
use rustix::fs::{self, AtFlags, Mode, OFlags};
use std::{
    fs::File,
    io::Write,
    net::SocketAddr,
    os::unix::fs::MetadataExt,
    path::{Component, Path, PathBuf},
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Error {
    Invalid,
    BeforePublication,
    PublicationUncertain,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "offline preparation refused",
            Self::BeforePublication => "offline preparation stopped before publication",
            Self::PublicationUncertain => "offline preparation publication may have completed",
        })
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn mapped<T, E>(result: std::result::Result<T, E>) -> Result<T> {
    result.map_err(|_| Error::Invalid)
}
fn canonical(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|p| matches!(p, Component::ParentDir | Component::CurDir))
        && path.components().collect::<PathBuf>().as_os_str() == path.as_os_str()
}
pub(super) struct Selection {
    pub vault: PathBuf,
    input: PathBuf,
    output: PathBuf,
    pub unicode: bool,
    card: bool,
}
impl Selection {
    pub fn parse(args: &[String]) -> Result<Self> {
        if !(args.len() == 4 || (args.len() == 5 && args[4] == "--unicode")) {
            return Err(Error::Invalid);
        }
        let card = match args[0].as_str() {
            "--device-provider-prepare" => false,
            "--device-provider-card" => true,
            _ => return Err(Error::Invalid),
        };
        let vault = PathBuf::from(&args[1]);
        let input = PathBuf::from(&args[2]);
        let output = PathBuf::from(&args[3]);
        if ![&vault, &input, &output].into_iter().all(|p| canonical(p)) {
            return Err(Error::Invalid);
        }
        Ok(Self {
            vault,
            input,
            output,
            unicode: args.len() == 5,
            card,
        })
    }
}
/// Store::at(false) can create a missing lock. Require the existing lock first;
/// normal unlock may sync existing bytes, but setup never creates store members.
pub(super) fn existing_vault(path: &Path) -> Result<()> {
    if !canonical(path) {
        return Err(Error::Invalid);
    }
    let (dir, name) = mapped(profile::parent(&path.join("mail.lock"), path))?;
    let file = File::from(mapped(fs::openat(
        &dir,
        &name,
        OFlags::RDWR | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    ))?);
    let m = mapped(file.metadata())?;
    if !m.is_file()
        || m.uid() != rustix::process::geteuid().as_raw()
        || m.mode() & 0o7777 != 0o600
        || m.nlink() != 1
        || m.len() != 0
    {
        return Err(Error::Invalid);
    }
    Ok(())
}
fn principal(vault: &CheckpointVault, now: u64) -> Result<Principal> {
    // Inspection alone deliberately does not authorize public preparation.
    let (bundle, _) = mapped(vault.publication(now))?;
    let state = mapped(vault.inspection())?;
    if bundle.identity != *state.identity {
        return Err(Error::Invalid);
    }
    let principal = Principal {
        account: mapped(vault.legacy())?.home.clone(),
        device: state.local_device,
        identity: bundle.identity,
    };
    mapped(principal.validate())?;
    Ok(principal)
}
fn string(t: &mut toml::Table, key: &str) -> Result<String> {
    t.remove(key)
        .and_then(|v| v.as_str().map(str::to_owned))
        .ok_or(Error::Invalid)
}
fn strict_key(bytes: &[u8]) -> Result<()> {
    let text = mapped(std::str::from_utf8(bytes))?.trim_ascii();
    for label in ["PRIVATE KEY", "EC PRIVATE KEY", "RSA PRIVATE KEY"] {
        if let Some(body) = text
            .strip_prefix(&format!("-----BEGIN {label}-----"))
            .and_then(|t| t.strip_suffix(&format!("-----END {label}-----")))
            && !body.is_empty()
            && body.bytes().all(|b| {
                b.is_ascii_alphanumeric() || b"+/=".contains(&b) || b.is_ascii_whitespace()
            })
        {
            return Ok(());
        }
    }
    Err(Error::Invalid)
}
fn credentials(t: &toml::Table, root: &Path) -> Result<[u8; 32]> {
    let mut leaf = None;
    for field in ["ca", "certificate", "key"] {
        let path = PathBuf::from(
            t.get(field)
                .and_then(|v| v.as_str())
                .ok_or(Error::Invalid)?,
        );
        if !canonical(&path) {
            return Err(Error::Invalid);
        }
        let bytes = mapped(profile::private_file(&path, root, MAX_PEM as u64))?;
        if field == "key" {
            strict_key(&bytes)?;
        } else {
            let digest = mapped(certificate_sha256(&bytes))?;
            if field == "certificate" {
                leaf = Some(digest);
            }
        }
    }
    leaf.ok_or(Error::Invalid)
}
pub(super) fn execute(selection: &Selection, vault: &CheckpointVault, now: u64) -> Result<()> {
    let principal = principal(vault, now)?;
    let input_root = selection.input.parent().ok_or(Error::Invalid)?;
    let bytes = mapped(profile::private_file(
        &selection.input,
        input_root,
        MAX_CARD as u64,
    ))?;
    let mut t = mapped(mapped(std::str::from_utf8(&bytes))?.parse::<toml::Table>())?;
    let output = if selection.card {
        // Verify the selected complete profile against this actual unlocked vault.
        mapped(profile::load(&selection.input, vault))?;
        let digest = credentials(&t, input_root)?;
        mapped(mapped(Card::new(principal, digest))?.encode())?
    } else {
        if t.remove("version").and_then(|v| v.as_integer()) != Some(1) {
            return Err(Error::Invalid);
        }
        let mut transport = toml::Table::new();
        transport.insert("version".into(), 1i64.into());
        for key in [
            "home",
            "endpoint",
            "name",
            "pin",
            "ca",
            "certificate",
            "key",
        ] {
            transport.insert(key.into(), string(&mut t, key)?.into());
        }
        if !t.is_empty() {
            return Err(Error::Invalid);
        }
        let address = transport["endpoint"].as_str().ok_or(Error::Invalid)?;
        let parsed = mapped(address.parse::<SocketAddr>())?;
        if parsed.to_string() != address {
            return Err(Error::Invalid);
        }
        let root = selection.output.parent().ok_or(Error::Invalid)?;
        credentials(&transport, root)?;
        transport.insert("account".into(), principal.account.into());
        transport.insert("device".into(), i64::from(principal.device).into());
        transport.insert("identity".into(), hex(&principal.identity).into());
        let bytes = transport.to_string().into_bytes();
        mapped(profile::decode(&bytes, &selection.output, vault))?;
        bytes
    };
    publish(&output, &selection.output)?;
    if !selection.card {
        // A later failure cannot imply rollback after atomic publication.
        profile::load(&selection.output, vault).map_err(|_| Error::PublicationUncertain)?;
    }
    Ok(())
}
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
/// Bounded action-local publication. No replace, cleanup sweep or two-file transaction.
fn publish(bytes: &[u8], output: &Path) -> Result<()> {
    let before = Error::BeforePublication;
    let uncertain = Error::PublicationUncertain;
    if bytes.is_empty() || bytes.len() > MAX_CARD || !canonical(output) {
        return Err(before);
    }
    let root = output.parent().ok_or(before)?;
    let (dir, name) = profile::parent(output, root).map_err(|_| before)?;
    let mut random = [0; 16];
    getrandom::fill(&mut random).map_err(|_| before)?;
    let temp = format!(".faction-provider-prepare-{}", hex(&random));
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
thread_local! { static FAULT: std::cell::Cell<Option<Point>> = const { std::cell::Cell::new(None) }; }
#[cfg(test)]
fn point(point: Point) -> std::io::Result<()> {
    if FAULT.with(|f| f.get() == Some(point)) {
        Err(std::io::Error::other("synthetic publication failure"))
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt, symlink};
    struct Owned(PathBuf);
    impl Owned {
        fn new() -> Self {
            let mut random = [0; 16];
            getrandom::fill(&mut random).unwrap();
            let path = std::env::temp_dir()
                .canonicalize()
                .unwrap()
                .join(format!("faction286-publish-OWNED-{}", hex(&random)));
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&path)
                .unwrap();
            std::fs::write(path.join("OWNED"), b"offline publication unit control").unwrap();
            Self(path)
        }
    }
    impl Drop for Owned {
        fn drop(&mut self) {
            FAULT.with(|f| f.set(None));
            assert_eq!(
                std::fs::read(self.0.join("OWNED")).unwrap(),
                b"offline publication unit control"
            );
            std::fs::remove_dir_all(&self.0).unwrap();
            assert!(!self.0.exists());
        }
    }
    #[test]
    fn publication_faults_preserve_phase_and_never_delete_final_output() {
        for fault in [
            Point::Write,
            Point::FileSync,
            Point::BeforeLink,
            Point::Link,
            Point::AfterLink,
            Point::Remove,
            Point::DirectorySync,
        ] {
            let root = Owned::new();
            let output = root.0.join("candidate.toml");
            FAULT.with(|f| f.set(Some(fault)));
            let error = publish(b"bounded public data", &output).unwrap_err();
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
            assert_eq!(output.exists(), published);
            if published {
                assert_eq!(std::fs::read(&output).unwrap(), b"bounded public data");
            }
            assert!(std::fs::read_dir(&root.0).unwrap().all(|e| {
                !e.unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".faction-provider-prepare-")
            }));
        }
    }
    #[test]
    fn publication_is_private_bounded_no_replace_and_no_follow() {
        let root = Owned::new();
        let output = root.0.join("candidate.toml");
        publish(b"complete candidate", &output).unwrap();
        let m = output.metadata().unwrap();
        assert_eq!(m.mode() & 0o7777, 0o600);
        assert_eq!(m.nlink(), 1);
        assert_eq!(
            publish(b"replacement", &output),
            Err(Error::BeforePublication)
        );
        assert_eq!(std::fs::read(&output).unwrap(), b"complete candidate");
        let link = root.0.join("linked.toml");
        symlink(&output, &link).unwrap();
        assert_eq!(
            publish(b"replacement", &link),
            Err(Error::BeforePublication)
        );
        let hard = root.0.join("hard.toml");
        std::fs::hard_link(&output, &hard).unwrap();
        assert_eq!(
            publish(b"replacement", &hard),
            Err(Error::BeforePublication)
        );
        assert_eq!(std::fs::read(&output).unwrap(), b"complete candidate");
        let alias = root.0.join("alias");
        symlink(&root.0, &alias).unwrap();
        assert_eq!(
            publish(b"data", &alias.join("absent")),
            Err(Error::BeforePublication)
        );
        assert_eq!(
            publish(&vec![b'x'; MAX_CARD + 1], &root.0.join("oversize")),
            Err(Error::BeforePublication)
        );
        assert_eq!(
            publish(b"", &root.0.join("empty")),
            Err(Error::BeforePublication)
        );
        std::fs::set_permissions(&root.0, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(
            publish(b"data", &root.0.join("unsafe")),
            Err(Error::BeforePublication)
        );
        std::fs::set_permissions(&root.0, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(!root.0.join("unsafe").exists());
    }
    #[test]
    fn absent_lock_is_refused_without_creating_store_members() {
        let root = Owned::new();
        assert_eq!(existing_vault(&root.0), Err(Error::Invalid));
        assert!(!root.0.join("mail.lock").exists());
        assert_eq!(std::fs::read_dir(&root.0).unwrap().count(), 1);
    }
    #[test]
    fn command_selection_refuses_implicit_transport_or_noncanonical_paths() {
        let valid = [
            "--device-provider-prepare",
            "/vault",
            "/config/template",
            "/config/profile",
        ]
        .map(str::to_owned);
        assert!(Selection::parse(&valid).is_ok());
        let mut unicode = valid.to_vec();
        unicode.push("--unicode".into());
        assert!(Selection::parse(&unicode).unwrap().unicode);
        for extra in ["--device-provider", "--unknown"] {
            let mut changed = valid.to_vec();
            changed.push(extra.into());
            assert!(Selection::parse(&changed).is_err());
        }
        for path in [
            "relative",
            "/vault/../other",
            "/vault/./other",
            "/vault//other",
        ] {
            let mut changed = valid.clone();
            changed[1] = path.into();
            assert!(Selection::parse(&changed).is_err());
        }
    }
}
