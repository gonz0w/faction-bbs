//! Immutable owner-curated metadata. File visibility is deliberately not cached here.
use std::{
    fs::{self, File},
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};
use toml::{Table, Value};
pub type Result<T> = std::result::Result<T, &'static str>;
pub const BYTES: usize = 32768;
pub const BOOKS: usize = 32;
pub const SHELVES: usize = 8;
#[derive(Default)]
pub struct Catalog {
    pub(crate) enabled: bool,
    pub(crate) name: String,
    pub(crate) compact: bool,
    pub(crate) requests: bool,
    pub(crate) requests_name: String,
    pub(crate) shelves: Vec<Shelf>,
    pub(crate) books: Vec<Book>,
}
pub(crate) struct Shelf {
    pub id: String,
    pub title: String,
}
pub(crate) struct Book {
    pub id: String,
    pub shelf: String,
    pub reference: String,
    pub note: String,
    pub rights: String,
    pub provenance: String,
    pub room: Option<i64>,
}
fn text(t: &mut Table, key: &str, cap: usize, multiline: bool) -> Result<String> {
    match t.remove(key) {
        Some(Value::String(s))
            if !s.trim().is_empty()
                && s.len() <= cap
                && s.bytes()
                    .all(|b| (32..=126).contains(&b) || (multiline && b == b'\n')) =>
        {
            Ok(s)
        }
        _ => Err("library needs bounded printable ASCII metadata"),
    }
}
fn boolean(t: &mut Table, key: &str) -> Result<bool> {
    match t.remove(key) {
        Some(Value::Boolean(b)) => Ok(b),
        _ => Err("library boolean required"),
    }
}
fn slug(s: &str) -> Result<()> {
    if s.is_empty()
        || s.len() > 32
        || !s
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        Err("invalid library slug")
    } else {
        Ok(())
    }
}
fn tables(t: &mut Table, key: &str, cap: usize) -> Result<Vec<Table>> {
    match t.remove(key) {
        Some(Value::Array(a)) if a.len() <= cap => a
            .into_iter()
            .map(|v| match v {
                Value::Table(t) => Ok(t),
                _ => Err("library table required"),
            })
            .collect(),
        _ => Err("missing/oversized library tables"),
    }
}
fn check_reference(s: &str) -> Result<()> {
    let (b, o) = s
        .strip_prefix("faction-file:")
        .and_then(|s| s.split_once('/'))
        .ok_or("canonical local file reference required")?;
    faction_core::file_id(b)
        .and_then(|_| faction_core::file_id(o))
        .map_err(|_| "invalid local file reference")
}
impl Catalog {
    pub fn enabled(&self) -> bool {
        self.enabled && (!self.books.is_empty() || self.requests)
    }
    pub fn len(&self) -> usize {
        self.books.len()
    }
    pub fn is_empty(&self) -> bool {
        self.books.is_empty()
    }
    pub fn load(root: &Path) -> Result<Self> {
        let uid = rustix::process::geteuid().as_raw();
        let m = fs::symlink_metadata(root).map_err(|_| "library pack unavailable")?;
        if !m.is_dir() || (m.uid() != 0 && m.uid() != uid) || m.mode() & 0o022 != 0 {
            return Err("library directory must be owner/root managed");
        }
        let f = File::options()
            .read(true)
            .custom_flags(
                (rustix::fs::OFlags::NOFOLLOW
                    | rustix::fs::OFlags::NONBLOCK
                    | rustix::fs::OFlags::CLOEXEC)
                    .bits() as i32,
            )
            .open(root.join("library.toml"))
            .map_err(|_| "library manifest unavailable")?;
        let m = f.metadata().map_err(|_| "library metadata unavailable")?;
        if !m.is_file()
            || m.nlink() != 1
            || (m.uid() != 0 && m.uid() != uid)
            || m.mode() & 0o022 != 0
            || m.len() > BYTES as u64
        {
            return Err("unsafe/oversized library manifest");
        }
        let mut bytes = vec![];
        f.take(BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "library read failed")?;
        if bytes.len() > BYTES {
            return Err("library manifest grew past budget");
        }
        Self::parse(std::str::from_utf8(&bytes).map_err(|_| "library UTF-8 required")?)
    }
    pub(crate) fn parse(raw: &str) -> Result<Self> {
        if raw.len() > BYTES || raw.bytes().filter(|b| *b == b'[').count() > 256 {
            return Err("library manifest budget");
        }
        let mut t: Table = raw.parse().map_err(|_| "invalid library TOML")?;
        if t.remove("version") != Some(Value::Integer(1)) {
            return Err("unsupported library version");
        }
        let enabled = boolean(&mut t, "enabled")?;
        let name = text(&mut t, "name", 24, false)?;
        let compact = boolean(&mut t, "compact")?;
        let requests = match t.remove("requests") {
            None => false,
            Some(Value::Boolean(v)) => v,
            _ => return Err("invalid requests setting"),
        };
        let requests_name = if t.contains_key("requests_name") {
            text(&mut t, "requests_name", 24, false)?
        } else {
            "File request desk".into()
        };
        let mut shelves: Vec<Shelf> = vec![];
        for mut row in tables(&mut t, "shelf", SHELVES)? {
            let id = text(&mut row, "id", 32, false)?;
            slug(&id)?;
            let title = text(&mut row, "title", 32, false)?;
            if !row.is_empty() || shelves.iter().any(|s| s.id == id) {
                return Err("unknown/duplicate library shelf");
            }
            shelves.push(Shelf { id, title });
        }
        let mut books: Vec<Book> = vec![];
        for mut row in tables(&mut t, "book", BOOKS)? {
            let id = text(&mut row, "id", 32, false)?;
            slug(&id)?;
            let shelf = text(&mut row, "shelf", 32, false)?;
            if books.iter().any(|b| b.id == id) || !shelves.iter().any(|s| s.id == shelf) {
                return Err("duplicate book or unknown shelf");
            }
            let reference = text(&mut row, "reference", 78, false)?;
            check_reference(&reference)?;
            let note = text(&mut row, "note", 240, true)?;
            let rights = text(&mut row, "rights", 96, false)?;
            let provenance = text(&mut row, "provenance", 160, false)?;
            let room = match row.remove("room") {
                None => None,
                Some(Value::Integer(n)) if n > 0 => Some(n),
                _ => return Err("positive local discussion room required"),
            };
            if !row.is_empty() {
                return Err("unknown library book field");
            }
            books.push(Book {
                id,
                shelf,
                reference,
                note,
                rights,
                provenance,
                room,
            });
        }
        if !t.is_empty() {
            return Err("unknown library setting");
        }
        Ok(Self {
            enabled,
            name,
            compact,
            requests,
            requests_name,
            shelves,
            books,
        })
    }
}
