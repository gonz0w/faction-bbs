//! Bounded device-local notebook data. Seals and notes never grant authority.
use crate::profile;
use faction_files::invalid;
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read},
    path::Path,
};
pub const MAX_BYTES: usize = 65536;
pub const LIMIT: usize = 32;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seal {
    pub title: String,
    pub artist: String,
    pub rights: String,
    pub art: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub id: String,
    pub label: String,
    pub binding: String,
    pub notes: String,
    pub favorites: String,
    pub seal: Option<Seal>,
}
#[derive(Clone, Default)]
pub struct Data {
    pub entries: Vec<Entry>,
}
pub fn text(s: &str, max: usize) -> bool {
    s.len() <= max && s.bytes().all(|b| b.is_ascii_graphic() || b == b' ')
}
pub fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn take(t: &mut toml::Table, k: &str) -> io::Result<String> {
    t.remove(k)
        .and_then(|v| v.as_str().map(str::to_owned))
        .ok_or_else(invalid)
}
fn table(s: &str, cap: usize) -> io::Result<toml::Table> {
    if s.len() > cap || s.bytes().filter(|b| *b == b'[').count() > 1024 {
        return Err(invalid());
    }
    s.parse().map_err(|_| invalid())
}
impl Seal {
    fn check(&self) -> io::Result<()> {
        if self.title.is_empty()
            || !text(&self.title, 26)
            || self.artist.is_empty()
            || !text(&self.artist, 48)
            || self.rights.is_empty()
            || !text(&self.rights, 96)
            || self.art.is_empty()
            || self.art.len() > 311
            || self.art.lines().count() > 8
            || !self.art.lines().all(|l| text(l, 38))
            || self
                .art
                .bytes()
                .any(|b| b != b'\n' && !b.is_ascii_graphic() && b != b' ')
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn from(mut t: toml::Table) -> io::Result<Self> {
        let s = Self {
            title: take(&mut t, "title")?,
            artist: take(&mut t, "artist")?,
            rights: take(&mut t, "rights")?,
            art: take(&mut t, "art")?,
        };
        if !t.is_empty() {
            return Err(invalid());
        }
        s.check()?;
        Ok(s)
    }
    pub fn parse(s: &str) -> io::Result<Self> {
        let mut t = table(s, 4096)?;
        if t.remove("version").and_then(|v| v.as_integer()) != Some(1) {
            return Err(invalid());
        }
        Self::from(t)
    }
    pub fn load(path: &Path) -> io::Result<Self> {
        let mut s = String::new();
        profile::regular(path, 4096, false)?
            .take(4097)
            .read_to_string(&mut s)?;
        Self::parse(&s)
    }
    pub fn lines(&self) -> Vec<String> {
        let mut out = vec![format!("Unverified souvenir: {}", self.title)];
        out.extend(self.art.lines().map(str::to_owned));
        out.extend([
            format!("Artist: {}", self.artist),
            format!("Rights: {}", self.rights),
        ]);
        out
    }
}
impl Entry {
    pub fn check(&self) -> io::Result<()> {
        if !id(&self.id)
            || self.label.is_empty()
            || !text(&self.label, 26)
            || self.binding.len() != 64
            || !self
                .binding
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || !text(&self.notes, 512)
            || !text(&self.favorites, 96)
        {
            return Err(invalid());
        }
        if let Some(s) = &self.seal {
            s.check()?;
        }
        Ok(())
    }
    /// Share fields are selected locally. No contact ID/binding/path/key/route is exported.
    pub fn share(&self, seal: bool, notes: bool, favorites: bool) -> Vec<String> {
        let mut out = vec![
            "Faction passport / selected fields".into(),
            "Souvenir only; not verified membership.".into(),
        ];
        if seal {
            if let Some(s) = &self.seal {
                out.extend(s.lines());
            } else {
                out.push("No seal selected.".into());
            }
        }
        if notes {
            out.push(format!("Personal note: {}", self.notes));
        }
        if favorites {
            out.push(format!(
                "Favorite rooms (personal text): {}",
                self.favorites
            ));
        }
        out
    }
}
impl Data {
    pub fn parse(s: &str) -> io::Result<Self> {
        let mut t = table(s, MAX_BYTES)?;
        if t.remove("version").and_then(|v| v.as_integer()) != Some(1) {
            return Err(invalid());
        }
        let Some(toml::Value::Array(entries)) = t.remove("entries") else {
            return Err(invalid());
        };
        if !t.is_empty() || entries.len() > LIMIT {
            return Err(invalid());
        }
        let mut out = Self::default();
        for v in entries {
            let toml::Value::Table(mut e) = v else {
                return Err(invalid());
            };
            let seal = match e.remove("seal") {
                None => None,
                Some(toml::Value::Table(s)) => Some(Seal::from(s)?),
                _ => return Err(invalid()),
            };
            let item = Entry {
                id: take(&mut e, "id")?,
                label: take(&mut e, "label")?,
                binding: take(&mut e, "binding")?,
                notes: take(&mut e, "notes")?,
                favorites: take(&mut e, "favorites")?,
                seal,
            };
            if !e.is_empty() || out.entries.iter().any(|v| v.id == item.id) {
                return Err(invalid());
            }
            item.check()?;
            out.entries.push(item);
        }
        Ok(out)
    }
    pub fn encode(&self) -> io::Result<String> {
        if self.entries.len() > LIMIT {
            return Err(invalid());
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut out = if self.entries.is_empty() {
            "version=1\nentries=[]\n".to_owned()
        } else {
            "version=1\n".to_owned()
        };
        for e in &self.entries {
            e.check()?;
            if !ids.insert(&e.id) {
                return Err(invalid());
            }
            out.push_str("\n[[entries]]\n");
            // Validated ASCII plus the seal's LF: Debug emits only TOML-compatible basic-string escapes.
            for (k, v) in [
                ("id", &e.id),
                ("label", &e.label),
                ("binding", &e.binding),
                ("notes", &e.notes),
                ("favorites", &e.favorites),
            ] {
                out.push_str(&format!("{k}={v:?}\n"));
            }
            if let Some(s) = &e.seal {
                out.push_str("[entries.seal]\n");
                for (k, v) in [
                    ("title", &s.title),
                    ("artist", &s.artist),
                    ("rights", &s.rights),
                    ("art", &s.art),
                ] {
                    out.push_str(&format!("{k}={v:?}\n"));
                }
            }
        }
        if out.len() > MAX_BYTES {
            return Err(invalid());
        }
        Ok(out)
    }
    pub fn put(&mut self, entry: Entry) -> io::Result<()> {
        entry.check()?;
        if let Some(old) = self.entries.iter_mut().find(|e| e.id == entry.id) {
            *old = entry;
        } else {
            if self.entries.len() >= LIMIT {
                return Err(invalid());
            }
            self.entries.push(entry);
        }
        Ok(())
    }
}
/// Exact local configuration snapshot, not verified remote identity or a visit proof.
pub fn binding(path: &Path) -> io::Result<String> {
    let settings = profile::Settings::load(path)?;
    let mut a = Vec::new();
    profile::regular(path, 8192, true)?
        .take(8193)
        .read_to_end(&mut a)?;
    if a.len() > 8192 {
        return Err(invalid());
    }
    let mut b = Vec::new();
    profile::regular(&settings.known_hosts, 1048576, false)?
        .take(1048577)
        .read_to_end(&mut b)?;
    if b.len() > 1048576 {
        return Err(invalid());
    }
    let mut h = Sha256::new();
    h.update(b"Faction local passport snapshot v1\0");
    h.update((a.len() as u64).to_be_bytes());
    h.update(a);
    h.update(b);
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn data_is_bounded_inert_and_selective() {
        let seal=Seal::parse("version=1\ntitle='Nightwire'\nartist='Faction'\nrights='BSD-3-Clause'\nart='[ N / W ]'").unwrap();
        let e = Entry {
            id: "secret-board".into(),
            label: "Private label".into(),
            binding: "a".repeat(64),
            notes: "PRIVATE_NOTE".into(),
            favorites: "SECRET_ROOM".into(),
            seal: Some(seal),
        };
        let d = Data {
            entries: vec![e.clone()],
        };
        assert_eq!(
            Data::parse(&d.encode().unwrap()).unwrap().entries,
            d.entries
        );
        let shared = e.share(true, false, false).join("\n");
        for secret in [
            "PRIVATE_NOTE",
            "SECRET_ROOM",
            "secret-board",
            "Private label",
            &"a".repeat(64),
        ] {
            assert!(!shared.contains(secret));
        }
        assert!(
            e.share(false, true, true)
                .join("\n")
                .contains("PRIVATE_NOTE")
        );
        assert!(
            Data {
                entries: vec![e.clone(); 33]
            }
            .encode()
            .is_err()
        );
        let mut multiple = d.clone();
        let mut another = e.clone();
        another.id = "other".into();
        another.notes = r#"A \ path and "quoted" words"#.into();
        multiple.entries.push(another);
        assert_eq!(
            Data::parse(&multiple.encode().unwrap()).unwrap().entries,
            multiple.entries
        );
        let mut bad = e;
        bad.notes = "\x1b[31m".into();
        assert!(bad.check().is_err());
        for bad in [
            "version=2\nentries=[]",
            "version=1\nentries=[]\nunknown=true",
        ] {
            assert!(Data::parse(bad).is_err());
        }
    }
    #[test]
    fn seal_controls_shapes_and_rights_are_checked() {
        for art in [
            "\x1b[2J".into(),
            "x".repeat(39),
            "a\n".repeat(9),
            "é".into(),
            "a\r\nb".into(),
        ] {
            let s = Seal {
                title: "N".into(),
                artist: "A".into(),
                rights: "R".into(),
                art,
            };
            assert!(s.check().is_err());
        }
        assert!(Seal::parse("version=1\ntitle='X'\nartist='A'\nrights=''\nart='x'").is_err());
    }
}
