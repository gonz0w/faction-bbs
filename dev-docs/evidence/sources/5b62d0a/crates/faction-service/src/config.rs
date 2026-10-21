//! Bounded owner-managed service configuration; never supplied by terminals.
use faction_access::{Result, denied};
use rustix::fs::OFlags;
use std::{
    fs::File,
    io::Read,
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Component, Path, PathBuf},
};
use toml::{Table, Value};

pub struct Config {
    pub terminal_uid: u32,
    pub socket: PathBuf,
    pub state_dir: PathBuf,
    pub theme: Option<PathBuf>,
    pub season: Option<PathBuf>,
    pub max_sessions: usize,
    pub launch_ms: u64,
    pub session_seconds: u64,
    pub ansi: bool,
}
impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let mut file = File::options()
            .read(true)
            .custom_flags((OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK).bits() as i32)
            .open(path)?;
        let m = file.metadata()?;
        let owner = rustix::process::geteuid().as_raw();
        if !m.is_file()
            || m.nlink() != 1
            || (m.uid() != 0 && m.uid() != owner)
            || m.mode() & 0o022 != 0
            || m.len() > 8192
        {
            return Err(denied());
        }
        let mut bytes = Vec::new();
        (&mut file).take(8193).read_to_end(&mut bytes)?;
        if bytes.len() > 8192 {
            return Err(denied());
        }
        Self::parse(std::str::from_utf8(&bytes).map_err(|_| denied())?)
    }
    fn parse(text: &str) -> Result<Self> {
        let mut t: Table = text.parse().map_err(|_| denied())?;
        if integer(&mut t, "version", None, 1, 1)? != 1 {
            return Err(denied());
        }
        let terminal_uid = integer(&mut t, "terminal_uid", None, 1, u32::MAX as u64)? as u32;
        let socket =
            path(&mut t, "socket", Some("/var/run/faction/terminal.sock"))?.ok_or_else(denied)?;
        if socket != Path::new("/var/run/faction/terminal.sock")
            || socket.as_os_str().as_bytes().len() > 100
        {
            return Err(denied());
        }
        let state_dir = path(&mut t, "state_dir", Some("/var/db/faction"))?.ok_or_else(denied)?;
        let theme = path(&mut t, "theme", None)?;
        let season = path(&mut t, "season", None)?;
        let max_sessions = integer(&mut t, "max_sessions", Some(8), 1, 250)? as usize;
        let launch_ms = integer(&mut t, "launch_ms", Some(5000), 100, 60000)?;
        let session_seconds = integer(&mut t, "session_seconds", Some(1800), 1, 1800)?;
        let ansi = boolean(&mut t, "ansi", true)?;
        let experimental = boolean(&mut t, "experimental_capacity", false)?;
        if !t.is_empty() || (!experimental && (max_sessions > 8 || launch_ms < 5000)) {
            return Err(denied());
        }
        Ok(Self {
            terminal_uid,
            socket,
            state_dir,
            theme,
            season,
            max_sessions,
            launch_ms,
            session_seconds,
            ansi,
        })
    }
}
fn integer(t: &mut Table, key: &str, default: Option<u64>, min: u64, max: u64) -> Result<u64> {
    let n = match t.remove(key) {
        Some(Value::Integer(n)) => u64::try_from(n).map_err(|_| denied())?,
        None => default.ok_or_else(denied)?,
        _ => return Err(denied()),
    };
    if !(min..=max).contains(&n) {
        return Err(denied());
    }
    Ok(n)
}
fn boolean(t: &mut Table, key: &str, default: bool) -> Result<bool> {
    match t.remove(key) {
        Some(Value::Boolean(v)) => Ok(v),
        None => Ok(default),
        _ => Err(denied()),
    }
}
fn path(t: &mut Table, key: &str, default: Option<&str>) -> Result<Option<PathBuf>> {
    let s = match t.remove(key) {
        Some(Value::String(s)) => s,
        None => match default {
            Some(s) => s.to_owned(),
            None => return Ok(None),
        },
        _ => return Err(denied()),
    };
    let p = PathBuf::from(&s);
    if !p.is_absolute()
        || s.len() > 1024
        || s.bytes().any(|b| b < 32 || b == 127)
        || p.components().any(|c| matches!(c, Component::ParentDir))
    {
        return Err(denied());
    }
    Ok(Some(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
    };
    const BASE: &str = "version=1\nterminal_uid=123\n";
    #[test]
    fn defaults_and_explicit_capacity() {
        let c = Config::parse(BASE).unwrap();
        assert_eq!(
            (c.max_sessions, c.launch_ms, c.session_seconds),
            (8, 5000, 1800)
        );
        assert!(c.ansi);
        assert!(Config::parse(&format!("{BASE}max_sessions=250\n")).is_err());
        assert!(Config::parse(&format!("{BASE}launch_ms=100\n")).is_err());
        let c = Config::parse(&format!(
            "{BASE}max_sessions=250\nlaunch_ms=100\nexperimental_capacity=true\n"
        ))
        .unwrap();
        assert_eq!((c.max_sessions, c.launch_ms), (250, 100));
    }
    #[test]
    fn reject_ambiguous_unbounded_or_unknown_settings() {
        for extra in [
            "max_sessions=251",
            "max_sessions=0",
            "launch_ms=99",
            "session_seconds=1801",
            "terminal_uid=0",
            "ansi=1",
            "unknown=true",
            "theme='../pack'",
            "state_dir='/a/../b'",
            "socket='relative'",
            "socket='/tmp/other.sock'",
            "version=2",
            "max_sessions='8'",
        ] {
            let input = if extra.starts_with("terminal_uid") || extra.starts_with("version") {
                extra.to_owned()
            } else {
                format!("{BASE}{extra}\n")
            };
            assert!(Config::parse(&input).is_err(), "{extra}");
        }
        assert!(Config::parse("terminal_uid=123").is_err());
        assert!(Config::parse("version=1").is_err());
    }
    #[test]
    fn owner_file_permissions_links_and_size() {
        let d = std::env::temp_dir().join(format!("faction-service-config-{}", std::process::id()));
        fs::create_dir(&d).unwrap();
        fs::set_permissions(&d, fs::Permissions::from_mode(0o700)).unwrap();
        let p = d.join("service.toml");
        fs::write(&p, BASE).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(Config::load(&p).is_ok());
        let link = d.join("link");
        symlink(&p, &link).unwrap();
        assert!(Config::load(&link).is_err());
        fs::remove_file(&link).unwrap();
        fs::hard_link(&p, &link).unwrap();
        assert!(Config::load(&p).is_err());
        fs::remove_file(&link).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o622)).unwrap();
        assert!(Config::load(&p).is_err());
        fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&p, vec![b' '; 8193]).unwrap();
        assert!(Config::load(&p).is_err());
        fs::remove_dir_all(d).unwrap();
    }
}
