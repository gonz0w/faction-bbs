//! Binary-local private provider profile; no remote entry or endpoint discovery.
use faction_client::{
    mail_checkpoint::CheckpointVault,
    mail_data::{invalid, unhex},
    mail_device_provider::{Config, Credentials, DeviceProvider},
};
use faction_files::private_blobs::Principal;
use rustix::fs::{self, Mode, OFlags};
use std::{
    ffi::OsString,
    fs::File,
    io::{self, Read},
    net::SocketAddr,
    os::unix::fs::MetadataExt,
    path::{Component, Path, PathBuf},
};
use zeroize::Zeroizing;
fn absolute(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|p| matches!(p, Component::ParentDir | Component::CurDir))
}
fn private_directory(file: &File) -> io::Result<()> {
    let m = file.metadata()?;
    if m.uid() != rustix::process::geteuid().as_raw() || m.mode() & 0o7777 != 0o700 {
        return Err(invalid());
    }
    Ok(())
}
/// Resolve each component by descriptor without following links. Trusted global
/// prefixes need not be private; the selected credential root and descendants do.
pub(super) fn parent(path: &Path, root: &Path) -> io::Result<(File, OsString)> {
    if !absolute(path) || !absolute(root) || !path.starts_with(root) || path == root {
        return Err(invalid());
    }
    let mut dir = File::from(fs::open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )?);
    let components: Vec<_> = path
        .components()
        .filter_map(|p| {
            if let Component::Normal(n) = p {
                Some(n)
            } else {
                None
            }
        })
        .collect();
    let (name, parents) = components.split_last().ok_or_else(invalid)?;
    let mut traversed = PathBuf::from("/");
    for parent in parents {
        dir = File::from(fs::openat(
            &dir,
            *parent,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )?);
        traversed.push(parent);
        if traversed.starts_with(root) {
            private_directory(&dir)?;
        }
    }
    private_directory(&dir)?;
    Ok((dir, name.to_os_string()))
}
pub(super) fn private_file(path: &Path, root: &Path, cap: u64) -> io::Result<Zeroizing<Vec<u8>>> {
    let (dir, name) = parent(path, root)?;
    let file = File::from(fs::openat(
        &dir,
        &name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )?);
    let m = file.metadata()?;
    if !m.is_file()
        || m.nlink() != 1
        || m.uid() != rustix::process::geteuid().as_raw()
        || m.mode() & 0o7777 != 0o600
        || m.len() > cap
    {
        return Err(invalid());
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(cap + 1).read_to_end(&mut bytes)?;
    if bytes.is_empty() || bytes.len() as u64 > cap {
        return Err(invalid());
    }
    Ok(bytes)
}
pub(super) fn load(path: &Path, vault: &CheckpointVault) -> io::Result<DeviceProvider> {
    let root = path.parent().ok_or_else(invalid)?;
    let bytes = private_file(path, root, 8192)?;
    decode(&bytes, path, vault)
}
// Preparation and the maintained file loader validate the exact same schema,
// vault assertions and actual transport credentials before any publication.
pub(super) fn decode(
    bytes: &[u8],
    path: &Path,
    vault: &CheckpointVault,
) -> io::Result<DeviceProvider> {
    let root = path.parent().ok_or_else(invalid)?;
    if bytes.is_empty() || bytes.len() > 8192 {
        return Err(invalid());
    }
    let mut table = std::str::from_utf8(bytes)
        .map_err(|_| invalid())?
        .parse::<toml::Table>()
        .map_err(|_| invalid())?;
    if table.remove("version").and_then(|v| v.as_integer()) != Some(1) {
        return Err(invalid());
    }
    let device = table
        .remove("device")
        .and_then(|v| v.as_integer())
        .and_then(|v| u32::try_from(v).ok())
        .ok_or_else(invalid)?;
    let mut take = |key: &str| {
        table
            .remove(key)
            .and_then(|v| v.as_str().map(str::to_owned))
            .ok_or_else(invalid)
    };
    let account = take("account")?;
    let identity = unhex(&take("identity")?)?;
    let home = take("home")?;
    let address = take("endpoint")?
        .parse::<SocketAddr>()
        .map_err(|_| invalid())?;
    let name = take("name")?;
    let pin = unhex(&take("pin")?)?;
    let ca = PathBuf::from(take("ca")?);
    let certificate = PathBuf::from(take("certificate")?);
    let key = PathBuf::from(take("key")?);
    if !table.is_empty() {
        return Err(invalid());
    }
    // Assert the entire current local identity before reading transport secrets.
    let state = vault.inspection()?;
    if vault.legacy()?.home != account
        || state.local_device != device
        || *state.identity != identity
    {
        return Err(invalid());
    }
    let config = Config::new(
        &home,
        address,
        &name,
        pin,
        Principal {
            account,
            device,
            identity,
        },
    )
    .map_err(|_| invalid())?;
    let ca = private_file(&ca, root, 65536)?;
    let certificate = private_file(&certificate, root, 65536)?;
    let key = private_file(&key, root, 65536)?;
    let credentials = Credentials::from_pem(&ca, &certificate, &key).map_err(|_| invalid())?;
    let provider = DeviceProvider::new(config, credentials);
    provider.validate(vault).map_err(|_| invalid())?;
    Ok(provider)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        os::unix::fs::{DirBuilderExt, PermissionsExt, symlink},
        path::PathBuf,
    };
    fn bounded_output(
        command: &mut std::process::Command,
        limit: std::time::Duration,
    ) -> std::io::Result<std::process::Output> {
        let mut child = command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        let started = std::time::Instant::now();
        loop {
            if child.try_wait()?.is_some() {
                return child.wait_with_output();
            }
            if started.elapsed() >= limit {
                let _ = child.kill();
                let _ = child.wait_with_output()?;
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "bounded synthetic setup stopped",
                ));
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    struct Owned(PathBuf);
    impl Owned {
        fn new() -> Self {
            let mut id = [0; 16];
            getrandom::fill(&mut id).unwrap();
            let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
                "faction280-profile-OWNED-{}",
                faction_client::mail_data::hex(&id)
            ));
            fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
            fs::write(root.join("OWNED"), b"owned profile tests").unwrap();
            Self(root)
        }
        fn file(&self) -> PathBuf {
            let path = self.0.join("credential");
            fs::write(&path, b"bounded secret").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
            path
        }
    }
    impl Drop for Owned {
        fn drop(&mut self) {
            assert_eq!(
                fs::read(self.0.join("OWNED")).unwrap(),
                b"owned profile tests"
            );
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn child_deadline_kills_and_reaps_stalled_setup() {
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "device_provider_profile::tests::bounded_setup_child",
            ])
            .env("FACTION280_STALLED_CHILD", "1");
        let started = std::time::Instant::now();
        let result = bounded_output(&mut command, std::time::Duration::from_millis(100));
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::TimedOut);
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
    }
    #[test]
    fn bounded_setup_child() {
        if std::env::var_os("FACTION280_STALLED_CHILD").is_some() {
            std::thread::sleep(std::time::Duration::from_secs(30));
        }
    }
    #[test]
    fn private_reader_bounds_ownership_modes_links_and_root() {
        let f = Owned::new();
        let path = f.file();
        assert_eq!(
            private_file(&path, &f.0, 32).unwrap().as_slice(),
            b"bounded secret"
        );
        assert!(private_file(&path, &f.0, 3).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(private_file(&path, &f.0, 32).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let link = f.0.join("link");
        symlink(&path, &link).unwrap();
        assert!(private_file(&link, &f.0, 32).is_err());
        let hard = f.0.join("hard");
        fs::hard_link(&path, &hard).unwrap();
        assert!(private_file(&path, &f.0, 32).is_err());
        fs::remove_file(hard).unwrap();
        assert!(private_file(&f.0, &f.0, 32).is_err());
        assert!(private_file(&path, &f.0.join("unrelated"), 32).is_err());
        fs::write(&path, b"").unwrap();
        assert!(private_file(&path, &f.0, 32).is_err());
    }
    #[test]
    fn decoded_profile_binds_current_vault_and_private_credentials_without_spool() {
        const TEST: &str = "device_provider_profile::tests::decoded_profile_binds_current_vault_and_private_credentials_without_spool";
        if std::env::var("FACTION280_PROFILE_CHILD").ok().as_deref() != Some(TEST) {
            let result = bounded_output(
                std::process::Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", TEST, "--nocapture"])
                    .env("FACTION280_PROFILE_CHILD", TEST),
                std::time::Duration::from_secs(30),
            )
            .unwrap();
            assert!(
                result.status.success(),
                "{} {}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
            return;
        }
        let f = Owned::new();
        let openssl = if cfg!(target_os = "macos") {
            "/opt/homebrew/opt/openssl@3/bin/openssl"
        } else {
            "openssl"
        };
        let result = bounded_output(
            std::process::Command::new(openssl)
                .args([
                    "req",
                    "-x509",
                    "-newkey",
                    "ec",
                    "-pkeyopt",
                    "ec_paramgen_curve:P-256",
                    "-nodes",
                    "-days",
                    "2",
                    "-subj",
                    "/CN=home.test",
                    "-keyout",
                ])
                .arg(f.0.join("key.pem"))
                .arg("-out")
                .arg(f.0.join("cert.pem")),
            std::time::Duration::from_secs(30),
        )
        .unwrap();
        assert!(result.status.success());
        for name in ["key.pem", "cert.pem"] {
            fs::set_permissions(f.0.join(name), fs::Permissions::from_mode(0o600)).unwrap();
        }
        let vault = faction_client::mail_checkpoint::CheckpointVault::create(
            faction_client::mail_vault::Store::at(&f.0.join("vault"), true).unwrap(),
            faction_client::mail_data::Data::new("alice@home.test".into()).unwrap(),
            b"synthetic profile test passphrase",
            101,
            1792886400,
        )
        .unwrap()
        .0;
        let identity = faction_client::mail_data::hex(vault.inspection().unwrap().identity);
        let text = format!(
            "version=1\nhome=\"home.test\"\nendpoint=\"127.0.0.1:9\"\nname=\"home.test\"\npin=\"{}\"\naccount=\"alice@home.test\"\ndevice=101\nidentity=\"{}\"\nca=\"{}\"\ncertificate=\"{}\"\nkey=\"{}\"\n",
            "01".repeat(32),
            identity,
            f.0.join("cert.pem").display(),
            f.0.join("cert.pem").display(),
            f.0.join("key.pem").display()
        );
        let path = f.0.join("profile.toml");
        fs::write(&path, &text).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let checkpoint = fs::read(f.0.join("vault/mail.fkv")).unwrap();
        assert!(load(&path, &vault).is_ok());
        for changed in [
            text.replace("alice@home.test", "other@home.test"),
            text.replace("device=101", "device=202"),
            text.replace(&identity, &"02".repeat(32)),
            text.replace("127.0.0.1:9", "192.0.2.1:9"),
            text.replace("version=1", "version=2"),
            format!("{text}password=\"forbidden\"\n"),
        ] {
            fs::write(&path, changed).unwrap();
            assert!(load(&path, &vault).is_err());
        }
        fs::write(&path, &text).unwrap();
        fs::set_permissions(f.0.join("key.pem"), fs::Permissions::from_mode(0o644)).unwrap();
        assert!(load(&path, &vault).is_err());
        fs::set_permissions(f.0.join("key.pem"), fs::Permissions::from_mode(0o600)).unwrap();
        let link = f.0.join("key-link.pem");
        symlink(f.0.join("key.pem"), &link).unwrap();
        fs::write(&path, text.replace("key.pem", "key-link.pem")).unwrap();
        assert!(load(&path, &vault).is_err());
        assert_eq!(fs::read(f.0.join("vault/mail.fkv")).unwrap(), checkpoint);
        assert!(!f.0.join("vault/attachments").exists());
    }
}
