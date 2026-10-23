//! Local keyboard and member-owned profile catalog; never fed by remote output.
use crate::{Terminal, profile};
use faction_files::invalid;
use faction_terminal::{Decoder, Key};
use std::{
    collections::BTreeSet,
    io::{self, Read},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
const BYTES: usize = 16384;
const LIMIT: usize = 16;
const PAGE: usize = 4;
pub(super) struct Board {
    pub label: String,
    id: String,
    pub profile: PathBuf,
}
pub(super) struct Catalog {
    pub boards: Vec<Board>,
    pub mail_store: Option<PathBuf>,
}
impl Catalog {
    pub fn load(path: &Path) -> io::Result<Self> {
        let mut text = String::new();
        profile::regular(path, BYTES as u64, true)?
            .take(BYTES as u64 + 1)
            .read_to_string(&mut text)?;
        Self::parse(&text)
    }
    fn parse(text: &str) -> io::Result<Self> {
        if text.len() > BYTES || text.bytes().filter(|b| *b == b'[').count() > 128 {
            return Err(invalid());
        }
        let mut t = text.parse::<toml::Table>().map_err(|_| invalid())?;
        if t.remove("version").and_then(|v| v.as_integer()) != Some(1) {
            return Err(invalid());
        }
        let mail_store = match t.remove("mail_store") {
            None => None,
            Some(toml::Value::String(path)) => Some(profile::path(path)?),
            _ => return Err(invalid()),
        };
        let Some(toml::Value::Array(entries)) = t.remove("boards") else {
            return Err(invalid());
        };
        if !t.is_empty() || entries.is_empty() || entries.len() > LIMIT {
            return Err(invalid());
        }
        let (mut ids, mut labels, mut paths) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
        let mut boards = Vec::new();
        for entry in entries {
            let toml::Value::Table(mut e) = entry else {
                return Err(invalid());
            };
            let mut string = |key: &str| {
                e.remove(key)
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .ok_or_else(invalid)
            };
            let id = string("id")?;
            let label = string("label")?;
            let profile = profile::path(string("profile")?)?;
            if !e.is_empty()
                || id.is_empty()
                || id.len() > 32
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                || label.is_empty()
                || label.len() > 26
                || !label.bytes().all(|b| b.is_ascii_graphic() || b == b' ')
                || !ids.insert(id.clone())
                || !labels.insert(label.clone())
                || !paths.insert(profile.clone())
            {
                return Err(invalid());
            }
            boards.push(Board { id, label, profile });
        }
        Ok(Self { boards, mail_store })
    }
}
#[derive(Default)]
struct View {
    selected: usize,
}
enum Choice {
    Stay,
    Open(usize),
    Quit,
    Mail,
}
pub(super) enum Destination {
    Board(usize),
    Mail(usize),
    Quit,
}
impl View {
    fn feed(&mut self, key: Key, count: usize, mail: bool) -> Choice {
        match key {
            Key::Cancel | Key::Hotkey(b'Q') => Choice::Quit,
            Key::Enter => Choice::Open(self.selected),
            Key::Hotkey(b'M') if mail => Choice::Mail,
            Key::Hotkey(k @ b'A'..=b'D') => {
                let i = self.selected / PAGE * PAGE + usize::from(k - b'A');
                if i < count {
                    Choice::Open(i)
                } else {
                    Choice::Stay
                }
            }
            Key::Up => {
                self.selected = (self.selected + count - 1) % count;
                Choice::Stay
            }
            Key::Down | Key::Next => {
                self.selected = (self.selected + 1) % count;
                Choice::Stay
            }
            Key::Right | Key::Hotkey(b'N') => {
                self.selected = if self.selected / PAGE + 1 < count.div_ceil(PAGE) {
                    (self.selected / PAGE + 1) * PAGE
                } else {
                    0
                };
                Choice::Stay
            }
            Key::Left | Key::Hotkey(b'P') => {
                self.selected = if self.selected >= PAGE {
                    (self.selected / PAGE - 1) * PAGE
                } else {
                    (count - 1) / PAGE * PAGE
                };
                Choice::Stay
            }
            _ => Choice::Stay,
        }
    }
    fn render(&self, catalog: &Catalog) -> String {
        let start = self.selected / PAGE * PAGE;
        let mut out = format!(
            "\x1b[2J\x1b[H\x1b[1;36mFaction // Boards on your device\x1b[0m\r\nMember-owned profiles; no auto-connect.\r\nPage {} / {}\r\n\r\n",
            start / PAGE + 1,
            catalog.boards.len().div_ceil(PAGE)
        );
        for (i, b) in catalog.boards.iter().enumerate().skip(start).take(PAGE) {
            out.push_str(&format!(
                "{} [{}] {}\r\n    {}\r\n",
                if i == self.selected { ">" } else { " " },
                char::from(b'A' + (i - start) as u8),
                b.label,
                b.id
            ));
        }
        out.push_str("\r\nArrows/Tab: select  N/P: page\r\nA-D/Enter: choose  Q: quit\r\n");
        out.push_str(if catalog.mail_store.is_some() {
            "M: local offline vault (no SSH)\r\n"
        } else {
            "Each board needs its own approval.\r\n"
        });
        out
    }
}
pub(super) fn choose(t: &mut Terminal, catalog: &Catalog, last: usize) -> io::Result<Destination> {
    let mut view = View {
        selected: last.min(catalog.boards.len() - 1),
    };
    let mut decoder = Decoder::new();
    // Drop old session typeahead; no remote frame or queued key selects a board.
    rustix::termios::tcflush(&t.input, rustix::termios::QueueSelector::IFlush)?;
    loop {
        t.print(&view.render(catalog))?;
        loop {
            decoder.expire(Instant::now());
            if !t.ready(100)? {
                continue;
            }
            let Some(key) = decoder.feed(t.byte()?, Instant::now()) else {
                continue;
            };
            match view.feed(key, catalog.boards.len(), catalog.mail_store.is_some()) {
                Choice::Quit => return Ok(Destination::Quit),
                Choice::Open(i) => return Ok(Destination::Board(i)),
                Choice::Mail => return Ok(Destination::Mail(view.selected)),
                Choice::Stay => break,
            }
        }
    }
}
/// Only terminal keyboard bytes pass here. Server stream/filter has no reference.
#[derive(Default)]
pub(super) struct Keys {
    pending: Option<Instant>,
}
impl Keys {
    pub fn expired(&mut self, now: Instant) -> Option<u8> {
        if self
            .pending
            .is_some_and(|at| now.saturating_duration_since(at) >= Duration::from_secs(1))
        {
            self.pending = None;
            Some(29)
        } else {
            None
        }
    }
    /// Forward bytes plus a local-boards request. Trailing typeahead is discarded.
    pub fn feed(&mut self, bytes: &[u8], now: Instant) -> (Vec<u8>, bool) {
        let mut forward = Vec::new();
        if let Some(b) = self.expired(now) {
            forward.push(b);
        }
        for &b in bytes {
            if self.pending.take().is_some() {
                match b {
                    b'b' | b'B' => return (forward, true),
                    29 => forward.push(29),
                    _ => forward.extend([29, b]),
                }
            } else if b == 29 {
                self.pending = Some(now);
            } else {
                forward.push(b);
            }
        }
        (forward, false)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const ONE: &str =
        "version=1\n[[boards]]\nid='radio'\nlabel='Night Radio'\nprofile='/private/radio.toml'\n";
    #[test]
    fn catalog_is_private_bounded_and_never_contains_remote_endpoint_options() {
        assert_eq!(Catalog::parse(ONE).unwrap().boards.len(), 1);
        for bad in [
            ONE.replace("/private/radio.toml", "../radio.toml"),
            ONE.replace("Night Radio", "Bad\u{1b}"),
            format!("{ONE}host='remote.example'\n"),
            format!("{ONE}[[boards]]{}", ONE.split_once("[[boards]]").unwrap().1),
            "version=1\nboards=[]".into(),
        ] {
            assert!(Catalog::parse(&bad).is_err());
        }
        let p = std::env::temp_dir().join(format!("faction-launcher-{}", std::process::id()));
        std::fs::write(&p, ONE).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(Catalog::load(&p).is_ok());
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(Catalog::load(&p).is_err());
        std::fs::remove_file(p).unwrap();
    }
    #[test]
    fn maximum_catalog_keyboard_pages_and_frame_fit_compact_geometry() {
        let body = ONE.split_once("[[boards]]").unwrap().1;
        let mut source = "version=1\n".to_owned();
        for i in 0..16 {
            source.push_str(&format!(
                "[[boards]]{}",
                body.replace("radio", &format!("radio{i}"))
                    .replace("Night Radio", &format!("Night Radio {i}"))
            ));
        }
        let c = Catalog::parse(&source).unwrap();
        let mut v = View::default();
        assert!(matches!(v.feed(Key::Right, 16, false), Choice::Stay));
        assert!(v.render(&c).contains("Page 2 / 4"));
        assert!(matches!(v.feed(Key::Enter, 16, false), Choice::Open(4)));
        assert!(matches!(
            v.feed(Key::Hotkey(b'D'), 16, false),
            Choice::Open(7)
        ));
        assert!(v.render(&c).lines().count() <= 18);
        assert!(Catalog::parse(&(source + "[[boards]]" + body)).is_err());
    }
    #[test]
    fn optional_local_store_is_explicit_bounded_and_not_a_remote_profile() {
        let source = ONE.replace("version=1\n", "version=1\nmail_store='/private/mail'\n");
        let c = Catalog::parse(&source).unwrap();
        assert_eq!(c.mail_store.as_deref(), Some(Path::new("/private/mail")));
        let mut view = View::default();
        assert!(matches!(
            view.feed(Key::Hotkey(b'M'), 1, false),
            Choice::Stay
        ));
        assert!(matches!(
            view.feed(Key::Hotkey(b'M'), 1, true),
            Choice::Mail
        ));
        assert!(view.render(&c).contains("M: local offline vault (no SSH)"));
        assert!(view.render(&c).lines().count() <= 18);
        for value in ["'../mail'", "12", "[]", "'/a/../mail'"] {
            assert!(Catalog::parse(&source.replace("'/private/mail'", value)).is_err());
        }
    }
    #[test]
    fn keyboard_escape_is_fragmented_bounded_and_preserves_door_passthrough() {
        let now = Instant::now();
        let mut k = Keys::default();
        assert_eq!(k.feed(b"hello\x1d", now), (b"hello".to_vec(), false));
        assert_eq!(k.feed(b"Byes\r", now), (vec![], true));
        assert_eq!(k.feed(b"\x1d\x1d", now), (vec![29], false));
        assert_eq!(k.feed(b"\x1dx", now), (vec![29, b'x'], false));
        assert_eq!(k.feed(b"\x1d", now), (vec![], false));
        assert_eq!(k.expired(now + Duration::from_secs(1)), Some(29));
        assert_eq!(
            k.feed(b"b", now + Duration::from_secs(2)),
            (vec![b'b'], false)
        );
    }
}
