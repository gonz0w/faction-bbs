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
    pub doors_uid: Option<u32>,
    pub classic_games: Vec<super::classic::Game>,
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
        let doors_uid = if t.contains_key("doors_uid") {
            Some(integer(&mut t, "doors_uid", None, 1, u32::MAX as u64)? as u32)
        } else {
            None
        };
        let mut classic_games = Vec::<super::classic::Game>::new();
        if let Some(value) = t.remove("classic_games") {
            let Value::Array(entries) = value else {
                return Err(denied());
            };
            if entries.len() > 6 {
                return Err(denied());
            }
            for entry in entries {
                let Value::Table(mut entry) = entry else {
                    return Err(denied());
                };
                let mut string = |key| match entry.remove(key) {
                    Some(Value::String(s)) => Ok(s),
                    _ => Err(denied()),
                };
                let slug = string("slug")?;
                let title = string("title")?;
                let address = string("address")?.parse().map_err(|_| denied())?;
                let game = super::classic::Game {
                    slug,
                    title,
                    address,
                };
                if !entry.is_empty()
                    || !game.valid()
                    || classic_games
                        .iter()
                        .any(|g| g.slug == game.slug || g.address == game.address)
                {
                    return Err(denied());
                }
                classic_games.push(game);
            }
        }
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
            doors_uid,
            classic_games,
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
    fn classic_destinations_are_fixed_bounded_and_disabled_by_default() {
        assert!(Config::parse(BASE).unwrap().classic_games.is_empty());
        let entry = "[[classic_games]]\nslug='trade-wars'\ntitle='TradeWars host'\naddress='127.0.0.1:2002'\n";
        assert_eq!(
            Config::parse(&format!("{BASE}{entry}"))
                .unwrap()
                .classic_games
                .len(),
            1
        );
        for bad in [
            entry.replace("127.0.0.1:2002", "example.org:2002"),
            entry.replace("127.0.0.1:2002", "0.0.0.0:2002"),
            entry.replace("127.0.0.1:2002", "224.0.0.1:2002"),
            entry.replace("127.0.0.1:2002", "127.0.0.1:0"),
            entry.replace("trade-wars", "../shell"),
            format!("{entry}shell='sh'\n"),
            format!("{entry}{entry}"),
        ] {
            assert!(Config::parse(&format!("{BASE}{bad}")).is_err(), "{bad}");
        }
        let many = (0..7)
            .map(|i| {
                entry
                    .replace("trade-wars", &format!("game-{i}"))
                    .replace("2002", &(2002 + i).to_string())
            })
            .collect::<String>();
        assert!(Config::parse(&format!("{BASE}{many}")).is_err());
    }
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
