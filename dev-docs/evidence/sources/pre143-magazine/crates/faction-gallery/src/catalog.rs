//! Owner-selected immutable gallery packs. Loading is a host-startup operation.
use faction_terminal::Art;
use std::{
    fs::{self, File},
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};
use toml::{Table, Value};
pub type Result<T> = std::result::Result<T, &'static str>;
pub const CATALOG_BYTES: usize = 32768;
pub const WORKS: usize = 8;
pub const PAGES: usize = 64;
pub const ART_FRAME_BYTES: usize = 7168;
#[derive(Default)]
pub struct Catalog {
    pub(crate) enabled: bool,
    pub(crate) name: String,
    pub(crate) compact: bool,
    pub(crate) exhibits: Vec<Exhibit>,
    pub(crate) works: Vec<Work>,
}
pub(crate) struct Exhibit {
    pub id: String,
    pub title: String,
}
pub(crate) struct Work {
    pub id: String,
    pub exhibit: String,
    pub title: String,
    pub artist: String,
    pub license: String,
    pub provenance: String,
    pub source: String,
    pub description: String,
    pub format: String,
    pub art: Art,
    pub plain: String,
    pub download: Option<String>,
}
fn text(t: &mut Table, key: &str, limit: usize, multiline: bool) -> Result<String> {
    match t.remove(key) {
        Some(Value::String(s))
            if !s.trim().is_empty()
                && s.len() <= limit
                && s.chars()
                    .all(|c| !c.is_control() || (multiline && c == '\n')) =>
        {
            Ok(s)
        }
        _ => Err("gallery metadata needs bounded inert text"),
    }
}
fn boolean(t: &mut Table, key: &str) -> Result<bool> {
    match t.remove(key) {
        Some(Value::Boolean(v)) => Ok(v),
        _ => Err("gallery boolean required"),
    }
}
fn slug(s: &str) -> Result<()> {
    if s.is_empty()
        || s.len() > 32
        || !s
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err("gallery ID needs 1-32 lowercase letters/digits/hyphens");
    }
    Ok(())
}
fn filename(s: &str) -> Result<()> {
    if s.is_empty()
        || s.len() > 48
        || !s.as_bytes()[0].is_ascii_alphanumeric()
        || !s
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
    {
        return Err("gallery asset needs one safe basename");
    }
    Ok(())
}
fn read(root: &Path, name: &str, cap: usize) -> Result<Vec<u8>> {
    filename(name)?;
    let uid = rustix::process::geteuid().as_raw();
    let file = File::options()
        .read(true)
        .custom_flags(
            (rustix::fs::OFlags::NOFOLLOW
                | rustix::fs::OFlags::CLOEXEC
                | rustix::fs::OFlags::NONBLOCK)
                .bits() as i32,
        )
        .open(root.join(name))
        .map_err(|_| "gallery asset unavailable")?;
    let m = file.metadata().map_err(|_| "gallery asset unavailable")?;
    if !m.is_file()
        || m.nlink() != 1
        || (m.uid() != 0 && m.uid() != uid)
        || m.mode() & 0o022 != 0
        || m.len() > cap as u64
    {
        return Err("unsafe or oversized gallery asset");
    }
    let mut bytes = vec![];
    file.take(cap as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "gallery read failed")?;
    if bytes.len() > cap {
        return Err("gallery asset grew past budget");
    }
    Ok(bytes)
}
fn tables(t: &mut Table, key: &str, cap: usize) -> Result<Vec<Table>> {
    match t.remove(key) {
        Some(Value::Array(rows)) if rows.len() <= cap => rows
            .into_iter()
            .map(|v| match v {
                Value::Table(t) => Ok(t),
                _ => Err("gallery rows must be tables"),
            })
            .collect(),
        _ => Err("missing or oversized gallery section"),
    }
}
fn reference(s: &str) -> Result<()> {
    let (board, object) = s
        .strip_prefix("faction-file:")
        .and_then(|s| s.split_once('/'))
        .ok_or("gallery download needs a local canonical file reference")?;
    faction_core::file_id(board).map_err(|_| "invalid gallery file reference")?;
    faction_core::file_id(object).map_err(|_| "invalid gallery file reference")?;
    Ok(())
}
impl Catalog {
    pub fn enabled(&self) -> bool {
        self.enabled && !self.works.is_empty()
    }
    pub fn len(&self) -> usize {
        self.works.len()
    }
    pub fn is_empty(&self) -> bool {
        self.works.is_empty()
    }
    pub fn load(root: &Path) -> Result<Self> {
        let uid = rustix::process::geteuid().as_raw();
        let m = fs::symlink_metadata(root).map_err(|_| "gallery pack unavailable")?;
        if !m.is_dir() || (m.uid() != 0 && m.uid() != uid) || m.mode() & 0o022 != 0 {
            return Err("gallery pack must be owner/root managed and not writable by others");
        }
        let bytes = read(root, "gallery.toml", CATALOG_BYTES)?;
        let raw = std::str::from_utf8(&bytes).map_err(|_| "gallery catalog needs UTF-8")?;
        let mut t: Table = raw.parse().map_err(|_| "invalid gallery catalog")?;
        if t.remove("version") != Some(Value::Integer(1)) {
            return Err("unsupported gallery catalog");
        }
        let enabled = boolean(&mut t, "enabled")?;
        let name = text(&mut t, "name", 24, false)?;
        let compact = boolean(&mut t, "compact")?;
        let formats = match t.remove("formats") {
            Some(Value::Array(v)) if v.len() <= 2 => v
                .into_iter()
                .map(|v| match v {
                    Value::String(s) if s == "ansi" || s == "ascii" => Ok(s),
                    _ => Err("gallery formats are ansi/ascii only"),
                })
                .collect::<Result<Vec<_>>>()?,
            _ => return Err("gallery formats required"),
        };
        if formats.len() == 2 && formats[0] == formats[1] {
            return Err("duplicate gallery format");
        }
        let mut exhibits: Vec<Exhibit> = vec![];
        for mut row in tables(&mut t, "exhibit", 4)? {
            let id = text(&mut row, "id", 32, false)?;
            slug(&id)?;
            let title = text(&mut row, "title", 32, false)?;
            if !row.is_empty() || exhibits.iter().any(|e| e.id == id) {
                return Err("unknown/duplicate gallery exhibit");
            }
            exhibits.push(Exhibit { id, title });
        }
        let mut works = vec![];
        let mut seen = vec![];
        let (mut pages, mut art_bytes, mut plain_bytes) = (0, 0, 0);
        for mut row in tables(&mut t, "work", WORKS)? {
            let id = text(&mut row, "id", 32, false)?;
            slug(&id)?;
            if seen.contains(&id) {
                return Err("duplicate gallery work");
            }
            seen.push(id.clone());
            let exhibit = text(&mut row, "exhibit", 32, false)?;
            if !exhibits.iter().any(|e| e.id == exhibit) {
                return Err("unknown gallery exhibit");
            }
            let title = text(&mut row, "title", 48, false)?;
            let artist = text(&mut row, "artist", 64, false)?;
            let license = text(&mut row, "license", 96, false)?;
            let provenance = text(&mut row, "provenance", 160, false)?;
            let source = text(&mut row, "source", 16, false)?;
            if source != "original" && source != "licensed" {
                return Err("gallery source must declare original/licensed rights");
            }
            let description = text(&mut row, "description", 240, true)?;
            let format = text(&mut row, "format", 8, false)?;
            if format != "ansi" && format != "ascii" {
                return Err("unsupported gallery artwork format");
            }
            if !boolean(&mut row, "display_permission")? {
                return Err("gallery display permission must be explicit");
            }
            let offer = boolean(&mut row, "download_permission")?;
            let download = match row.remove("download_ref") {
                None => None,
                Some(Value::String(s)) => {
                    reference(&s)?;
                    Some(s)
                }
                _ => return Err("invalid gallery download reference"),
            };
            if offer != download.is_some() {
                return Err("gallery download permission and reference must agree");
            }
            let asset = text(&mut row, "asset", 48, false)?;
            filename(&asset)?;
            let fallback = text(&mut row, "fallback", 48, false)?;
            filename(&fallback)?;
            if !row.is_empty() {
                return Err("unknown gallery work field");
            }
            if !enabled || !formats.contains(&format) {
                continue;
            }
            let bytes = read(root, &asset, 65536)?;
            art_bytes += bytes.len();
            if art_bytes > 262144 {
                return Err("gallery aggregate art budget exceeded");
            }
            if format == "ascii"
                && !bytes
                    .iter()
                    .all(|b| *b == b'\n' || *b == b'\r' || (32..=126).contains(b))
            {
                return Err("ASCII artwork must contain printable ASCII lines only");
            }
            let art = Art::parse_long(&bytes)?;
            pages += art.page_count();
            if pages > PAGES {
                return Err("gallery aggregate cell budget exceeded");
            }
            for page in 0..art.page_count() {
                if art.render_page(page).unwrap().len() > ART_FRAME_BYTES {
                    return Err("gallery artwork page exceeds frame budget");
                }
            }
            let bytes = read(root, &fallback, 8192)?;
            plain_bytes += bytes.len();
            if plain_bytes > 32768 {
                return Err("gallery aggregate text budget exceeded");
            }
            let plain = String::from_utf8(bytes).map_err(|_| "gallery fallback needs ASCII")?;
            if plain.trim().is_empty()
                || !plain.bytes().all(|b| b == b'\n' || (32..=126).contains(&b))
                || plain.lines().count() > 128
                || plain.lines().any(|l| l.len() > 72)
            {
                return Err("gallery fallback exceeds bounded printable ASCII lines");
            }
            works.push(Work {
                id,
                exhibit,
                title,
                artist,
                license,
                provenance,
                source,
                description,
                format,
                art,
                plain,
                download,
            });
        }
        if !t.is_empty() {
            return Err("unknown gallery catalog field");
        }
        Ok(Self {
            enabled,
            name,
            compact,
            exhibits,
            works,
        })
    }
    /// Offline owner preview uses the same canonical cells as approved-member views.
    pub fn preview(&self, id: &str, page: usize) -> Result<String> {
        let work = self
            .works
            .iter()
            .find(|w| w.id == id)
            .ok_or("unknown gallery work")?;
        work.art.render_page(page).ok_or("unknown gallery page")
    }
}
