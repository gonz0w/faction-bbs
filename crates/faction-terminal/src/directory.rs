//! Host-curated connection cards: inert metadata, never a connector or authority.
use crate::{Key, Result, Theme, seasons::Date};
use std::{
    fs::{self, File},
    io::Read,
    net::IpAddr,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use toml::{Table, Value};
pub const BYTES: usize = 32768;
pub const LISTINGS: usize = 32;
const PAGE: usize = 4;
#[derive(Clone, Default)]
pub struct Directory {
    entries: Vec<Listing>,
    snapshot: Vec<(i64, String, bool)>,
}
/// One validated published card, supplied by the approved local core adapter.
pub struct Overlay {
    pub id: i64,
    pub revision: String,
    pub catalog: Directory,
    pub expired: bool,
}
#[derive(Clone)]
struct Listing {
    slug: String,
    name: String,
    summary: String,
    host: String,
    port: u16,
    login: String,
    reviewed: Date,
    until: Date,
    reviewed_text: String,
    until_text: String,
    fingerprint: Option<String>,
    owner: bool,
    expired: bool,
}
#[derive(Clone, Debug, Default)]
pub struct View {
    selected: usize,
    detail: bool,
    offset: usize,
    snapshot: Option<Vec<(i64, String, bool)>>,
}
fn text(t: &mut Table, key: &str, max: usize) -> Result<String> {
    match t.remove(key) {
        Some(Value::String(s))
            if !s.is_empty()
                && s.len() <= max
                && s.bytes().all(|b| b.is_ascii_graphic() || b == b' ') =>
        {
            Ok(s)
        }
        _ => Err("directory text must be bounded printable ASCII"),
    }
}
fn host(value: &str) -> Option<String> {
    if let Ok(ip) = value.parse::<IpAddr>() {
        return Some(ip.to_string());
    }
    if value.len() > 96
        || value.split('.').any(|s| {
            s.is_empty()
                || s.len() > 63
                || !s.as_bytes()[0].is_ascii_alphanumeric()
                || !s.as_bytes()[s.len() - 1].is_ascii_alphanumeric()
                || !s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    {
        return None;
    }
    Some(value.to_ascii_lowercase())
}
fn date_now() -> Option<Date> {
    let secs = i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs()).ok()?;
    let (y, m, d) = faction_unix::calendar_date(secs, false)?;
    Date::new(y, m, d)
}
fn status(entry: &Listing, today: Option<Date>) -> &'static str {
    if entry.expired {
        return "OLD";
    }
    match today {
        Some(d) if d.fields() < entry.reviewed.fields() => "DATE?",
        Some(d) if d.fields() > entry.until.fields() => "OLD",
        Some(_) => "LISTED",
        None => "DATE?",
    }
}
fn wrap(text: &str, width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    text.as_bytes()
        .chunks(width)
        .map(|b| String::from_utf8(b.to_vec()).expect("validated ASCII"))
        .collect()
}
impl Directory {
    /// Validated immutable pack conflict keys, for trusted startup configuration.
    pub fn curator_keys(&self) -> Vec<(String, String, u16, String)> {
        self.entries
            .iter()
            .filter(|e| e.owner)
            .map(|e| (e.slug.clone(), e.host.clone(), e.port, e.login.clone()))
            .collect()
    }
    pub fn compose(&self, overlays: Vec<Overlay>) -> Result<Self> {
        if self.entries.len() + overlays.len() > LISTINGS {
            return Err("directory union bound");
        }
        let mut result = self.clone();
        result.snapshot.clear();
        for overlay in overlays {
            if overlay.catalog.entries.len() != 1
                || overlay.id <= 0
                || overlay.revision.is_empty()
                || overlay.revision.len() > 128
                || result.snapshot.iter().any(|(id, _, _)| *id == overlay.id)
            {
                return Err("invalid published directory card");
            }
            let mut entry = overlay.catalog.entries.into_iter().next().unwrap();
            if result.entries.iter().any(|e| {
                e.slug == entry.slug
                    || (e.host == entry.host && e.port == entry.port && e.login == entry.login)
            }) {
                return Err("duplicate published directory card");
            }
            entry.owner = false;
            entry.expired = overlay.expired;
            result.entries.push(entry);
            result
                .snapshot
                .push((overlay.id, overlay.revision, overlay.expired));
        }
        Ok(result)
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn load(root: &Path) -> Result<Self> {
        let uid = rustix::process::geteuid().as_raw();
        let m = fs::symlink_metadata(root).map_err(|_| "directory pack unavailable")?;
        if !m.is_dir() || (m.uid() != 0 && m.uid() != uid) || m.mode() & 0o022 != 0 {
            return Err("directory pack must be host-owned and not writable by others");
        }
        let file = File::options()
            .read(true)
            .custom_flags(
                (rustix::fs::OFlags::NOFOLLOW
                    | rustix::fs::OFlags::NONBLOCK
                    | rustix::fs::OFlags::CLOEXEC)
                    .bits() as i32,
            )
            .open(root.join("boards.toml"))
            .map_err(|_| "directory file unavailable")?;
        let m = file.metadata().map_err(|_| "directory file unavailable")?;
        if !m.is_file()
            || m.nlink() != 1
            || (m.uid() != 0 && m.uid() != uid)
            || m.mode() & 0o022 != 0
            || m.len() > BYTES as u64
        {
            return Err("unsafe or oversized directory file");
        }
        let mut s = String::new();
        file.take(BYTES as u64 + 1)
            .read_to_string(&mut s)
            .map_err(|_| "directory read failed")?;
        Self::parse(&s)
    }
    pub fn parse(source: &str) -> Result<Self> {
        if source.len() > BYTES || source.bytes().filter(|b| *b == b'[').count() > 256 {
            return Err("directory size/table bound");
        }
        let mut t = source
            .parse::<Table>()
            .map_err(|_| "invalid directory TOML")?;
        if t.remove("version").and_then(|v| v.as_integer()) != Some(1) {
            return Err("unsupported directory version");
        }
        let Some(Value::Array(values)) = t.remove("boards") else {
            return Err("boards array required");
        };
        if !t.is_empty() || values.len() > LISTINGS {
            return Err("unknown directory field or too many boards");
        }
        let mut entries = Vec::<Listing>::new();
        for value in values {
            let Value::Table(mut t) = value else {
                return Err("board table required");
            };
            let slug = text(&mut t, "slug", 32)?;
            if !slug
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            {
                return Err("invalid board slug");
            }
            let name = text(&mut t, "name", 26)?;
            let summary = text(&mut t, "summary", 80)?;
            let host = host(&text(&mut t, "host", 96)?).ok_or("invalid SSH host")?;
            let port = t
                .remove("port")
                .and_then(|v| v.as_integer())
                .filter(|n| (1..=65535).contains(n))
                .ok_or("invalid SSH port")? as u16;
            let login = text(&mut t, "login", 32)?;
            if !login.as_bytes()[0].is_ascii_alphanumeric()
                || !login
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
            {
                return Err("invalid SSH login token");
            }
            let reviewed_text = text(&mut t, "reviewed", 10)?;
            let reviewed = Date::parse(&reviewed_text)?;
            let until_text = text(&mut t, "valid_until", 10)?;
            let until = Date::parse(&until_text)?;
            if until.fields() < reviewed.fields() {
                return Err("listing expires before review");
            }
            let fingerprint = if t.contains_key("fingerprint") {
                let v = text(&mut t, "fingerprint", 50)?;
                if v.len() != 50
                    || !v.starts_with("SHA256:")
                    || !v[7..]
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"+/".contains(&b))
                {
                    return Err("expected public SHA256 fingerprint hint");
                }
                Some(v)
            } else {
                None
            };
            if !t.is_empty()
                || entries.iter().any(|e| {
                    e.slug == slug || (e.host == host && e.port == port && e.login == login)
                })
            {
                return Err("unknown board field or duplicate listing");
            }
            entries.push(Listing {
                slug,
                name,
                summary,
                host,
                port,
                login,
                reviewed,
                until,
                reviewed_text,
                until_text,
                fingerprint,
                owner: true,
                expired: false,
            });
        }
        Ok(Self {
            entries,
            snapshot: Vec::new(),
        })
    }
    fn body(&self, index: usize, width: usize, today: Option<Date>) -> Vec<String> {
        let e = &self.entries[index];
        let command = format!("ssh -p {} -l {} {}", e.port, e.login, e.host);
        let mut raw = vec![
            format!("{} [{}]", e.name, status(e, today)),
            e.summary.clone(),
            if e.owner {
                "Owner-managed; pack/restart changes.".into()
            } else {
                "Locally moderated member submission.".into()
            },
            format!("Host: {}", e.host),
            format!("Port: {} | Login: {}", e.port, e.login),
            format!("Reviewed: {}", e.reviewed_text),
            format!("Valid until: {}", e.until_text),
        ];
        if command.len() <= width {
            raw.push(command);
        } else {
            raw.push("SSH: use host, port and login above.".into());
        }
        if let Some(hint) = &e.fingerprint {
            raw.push(format!("Host-key hint: {hint}"));
        }
        raw.extend(
            [
                "Hints are sysop supplied; verify separately.",
                "Your local SSH client checks the host key.",
                "No account, password or device keys shared.",
                "No connection or availability check made.",
            ]
            .map(str::to_owned),
        );
        raw.into_iter().flat_map(|s| wrap(&s, width)).collect()
    }
    /// Complete bounded frame; approved-session enforcement belongs to caller.
    pub fn render(
        &self,
        view: &View,
        columns: u16,
        rows: u16,
        ansi: bool,
        theme: &Theme,
    ) -> String {
        self.render_at(view, columns, rows, ansi, theme, date_now())
    }
    fn render_at(
        &self,
        view: &View,
        columns: u16,
        rows: u16,
        ansi: bool,
        theme: &Theme,
        today: Option<Date>,
    ) -> String {
        let width = usize::from(columns.clamp(40, 120)) - 1;
        let mut lines: Vec<(String, bool)> = vec![
            ("OTHER BOARDS / local directory".into(), true),
            ("Curated here; independent communities.".into(), false),
            (String::new(), false),
        ];
        if view.detail && !self.entries.is_empty() {
            let body = self.body(view.selected.min(self.len() - 1), width, today);
            let count = usize::from(rows.clamp(18, 100)) - 7;
            let offset = view.offset.min(body.len().saturating_sub(1));
            lines.push((
                format!("Connection card {} / {}", view.selected + 1, self.len()),
                false,
            ));
            lines.extend(
                body.iter()
                    .skip(offset)
                    .take(count)
                    .cloned()
                    .map(|s| (s, false)),
            );
            lines.push((
                format!(
                    "Lines {}-{} / {}",
                    offset + 1,
                    (offset + count).min(body.len()),
                    body.len()
                ),
                false,
            ));
            lines.push(("N/P: more L:list S:submit Q:main".into(), false));
        } else {
            let start = view.selected / PAGE * PAGE;
            lines.push((
                format!(
                    "Page {} / {} | A-D/Enter: card",
                    start / PAGE + 1,
                    self.len().div_ceil(PAGE).max(1)
                ),
                false,
            ));
            if self.is_empty() {
                lines.push(("No boards listed yet. Ask the sysop.".into(), false));
            }
            for (i, e) in self.entries.iter().enumerate().skip(start).take(PAGE) {
                lines.push((
                    format!(
                        "{} [{}] {} [{}]",
                        if i == view.selected { ">" } else { " " },
                        (b'A' + (i - start) as u8) as char,
                        e.name,
                        status(e, today)
                    ),
                    i == view.selected,
                ));
                lines.push((format!("  {}", e.summary), false));
            }
            lines.push(("Arrows/Tab N/P:page S:submit Q:back".into(), false));
        }
        lines.push(("Independent login/approval required.".into(), false));
        let (normal, focus, _) = theme.menu_styles();
        let mut out = if ansi {
            "\x1b[2J\x1b[H".to_owned()
        } else {
            String::new()
        };
        for (i, (line, selected)) in lines.iter().enumerate() {
            let line = &line[..line.len().min(width)];
            if ansi {
                out.push_str(&format!(
                    "\x1b[{};1H{}{}\x1b[K",
                    i + 1,
                    if *selected { focus.sgr() } else { normal.sgr() },
                    line
                ));
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
        if ansi {
            out.push_str("\x1b[0m");
        }
        debug_assert!(out.len() <= 8192);
        out
    }
}
impl View {
    /// A changed ordered overlay invalidates numeric selection and open cards.
    pub fn synchronize(&mut self, directory: &Directory) -> bool {
        let changed = self
            .snapshot
            .as_ref()
            .is_some_and(|s| s != &directory.snapshot);
        if changed {
            self.selected = 0;
            self.detail = false;
            self.offset = 0;
        }
        self.snapshot = Some(directory.snapshot.clone());
        changed
    }
    pub fn is_card(&self) -> bool {
        self.detail
    }
    /// true returns to the BBS; never launches a local or remote command.
    pub fn feed(&mut self, key: Key, directory: &Directory, columns: u16, rows: u16) -> bool {
        let changed = self.synchronize(directory);
        if key == Key::Hotkey(b'Q') {
            return true;
        }
        if changed {
            return false;
        }
        if directory.is_empty() {
            return false;
        }
        if self.detail {
            let count = usize::from(rows.clamp(18, 100)) - 7;
            let length = directory
                .body(
                    self.selected,
                    usize::from(columns.clamp(40, 120)) - 1,
                    date_now(),
                )
                .len();
            match key {
                Key::Hotkey(b'L') => {
                    self.detail = false;
                    self.offset = 0;
                }
                Key::Right | Key::Enter | Key::Hotkey(b'N') if self.offset + count < length => {
                    self.offset += count
                }
                Key::Left | Key::Hotkey(b'P') => self.offset = self.offset.saturating_sub(count),
                _ => (),
            }
        } else {
            match key {
                Key::Down | Key::Next => self.selected = (self.selected + 1) % directory.len(),
                Key::Up => self.selected = (self.selected + directory.len() - 1) % directory.len(),
                Key::Right | Key::Hotkey(b'N') => {
                    self.selected = if self.selected / PAGE + 1 < directory.len().div_ceil(PAGE) {
                        (self.selected / PAGE + 1) * PAGE
                    } else {
                        0
                    }
                }
                Key::Left | Key::Hotkey(b'P') => {
                    self.selected = if self.selected >= PAGE {
                        (self.selected / PAGE - 1) * PAGE
                    } else {
                        (directory.len() - 1) / PAGE * PAGE
                    }
                }
                Key::Enter => {
                    self.detail = true;
                    self.offset = 0;
                }
                Key::Hotkey(k @ b'A'..=b'D') => {
                    let i = self.selected / PAGE * PAGE + usize::from(k - b'A');
                    if i < directory.len() {
                        self.selected = i;
                        self.detail = true;
                        self.offset = 0;
                    }
                }
                _ => (),
            }
        }
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const ONE: &str = "version=1\n[[boards]]\nslug='radio'\nname='Radio Board'\nsummary='A private place for night radio.'\nhost='radio.example'\nport=2222\nlogin='bbs'\nreviewed='2026-10-22'\nvalid_until='2027-10-22'\n";
    #[test]
    fn rejects_command_tokens_unknowns_duplicate_endpoints_and_bad_dates() {
        assert_eq!(Directory::parse(ONE).unwrap().len(), 1);
        for bad in [
            ONE.replace("radio.example", "-oProxyCommand=sh"),
            ONE.replace("login='bbs'", "login='bbs;sh'"),
            ONE.replace("2027-10-22", "2025-10-22"),
            ONE.to_owned() + "password='private'\n",
            format!("{ONE}[[boards]]{}", ONE.split_once("[[boards]]").unwrap().1),
        ] {
            assert!(Directory::parse(&bad).is_err());
        }
    }
    #[test]
    fn cards_paginate_and_classic_frames_are_bounded_and_inert() {
        let d = Directory::parse(ONE).unwrap();
        let mut v = View::default();
        assert!(!v.feed(Key::Enter, &d, 40, 18));
        let text = d.render_at(&v, 40, 18, false, &Theme::default(), Date::new(2028, 1, 1));
        assert!(text.contains("OLD"));
        assert!(text.lines().count() <= 18);
        assert!(text.lines().all(|s| s.len() < 40));
        assert!(!text.contains('\x1b'));
        v.feed(Key::Hotkey(b'N'), &d, 40, 18);
        let text = d.render(&v, 40, 18, false, &Theme::default());
        assert!(text.contains("No account"));
        assert!(v.feed(Key::Hotkey(b'Q'), &d, 40, 18));
    }
    #[test]
    fn maximum_catalog_pages_ipv6_and_file_permissions() {
        let body = ONE.split_once("[[boards]]").unwrap().1;
        let mut source = "version=1\n".to_owned();
        for i in 0..32 {
            source.push_str(&format!(
                "[[boards]]{}",
                body.replace("slug='radio'", &format!("slug='radio{i}'"))
                    .replace("radio.example", &format!("radio{i}.example"))
            ));
        }
        let d = Directory::parse(&source).unwrap();
        let mut v = View::default();
        v.feed(Key::Right, &d, 80, 24);
        assert!(
            d.render(&v, 80, 24, false, &Theme::default())
                .contains("Page 2 / 8")
        );
        assert!(Directory::parse(&(source + "[[boards]]" + body)).is_err());
        assert!(Directory::parse(&ONE.replace("radio.example", "2001:db8::1")).is_ok());
        let p = std::env::temp_dir().join(format!(
            "faction-directory-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        let file = p.join("boards.toml");
        fs::write(&file, ONE).unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(Directory::load(&p).is_ok());
        fs::set_permissions(&file, fs::Permissions::from_mode(0o666)).unwrap();
        assert!(Directory::load(&p).is_err());
        fs::remove_file(&file).unwrap();
        std::os::unix::fs::symlink("missing", &file).unwrap();
        assert!(Directory::load(&p).is_err());
        fs::remove_dir_all(p).unwrap();
    }
}
