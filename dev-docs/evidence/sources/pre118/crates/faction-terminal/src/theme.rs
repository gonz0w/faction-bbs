use crate::{Art, Key, Result};
use std::{
    collections::BTreeSet,
    fmt::Write,
    fs::{self, File},
    io::Read,
    os::unix::fs::MetadataExt,
    path::Path,
};
use toml::{Table, Value};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Rooms,
    New,
    History,
    Post,
    Chat,
    Sysop,
    Doors,
    Files,
    Quit,
    Terminal,
}
impl Action {
    fn parse(s: &str) -> Result<Self> {
        match s {
            "rooms" => Ok(Self::Rooms),
            "new" => Ok(Self::New),
            "history" => Ok(Self::History),
            "post" => Ok(Self::Post),
            "chat" => Ok(Self::Chat),
            "sysop" => Ok(Self::Sysop),
            "doors" => Ok(Self::Doors),
            "files" => Ok(Self::Files),
            "quit" => Ok(Self::Quit),
            "terminal" => Ok(Self::Terminal),
            _ => Err("unknown menu action"),
        }
    }
    pub fn enabled(self, sysop: bool) -> bool {
        self != Self::Doors && self != Self::Files && (self != Self::Sysop || sysop)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub(crate) fg: u8,
    pub(crate) bg: u8,
    pub(crate) bold: bool,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            fg: 7,
            bg: 0,
            bold: false,
        }
    }
}
impl Style {
    fn new(fg: u8, bg: u8, bold: bool) -> Self {
        Self { fg, bg, bold }
    }
    pub fn sgr(self) -> String {
        format!(
            "\x1b[0;{};{};{}m",
            if self.bold { 1 } else { 22 },
            if self.fg < 8 {
                30 + self.fg
            } else {
                90 + self.fg - 8
            },
            if self.bg < 8 {
                40 + self.bg
            } else {
                100 + self.bg - 8
            }
        )
    }
}
#[derive(Debug, Clone)]
pub struct Entry {
    action: Action,
    key: u8,
    label: String,
}
impl Entry {
    pub fn action(&self) -> Action {
        self.action
    }
    pub fn key(&self) -> u8 {
        self.key
    }
    pub fn label(&self) -> &str {
        &self.label
    }
}
#[derive(Clone)]
pub struct Theme {
    id: String,
    board: String,
    welcome: String,
    logout: String,
    normal: Style,
    focus: Style,
    hotkey: Style,
    disabled: Style,
    doors_enabled: bool,
    files_enabled: bool,
    entries: Vec<Entry>,
    identity_ascii: Option<String>,
    identity_ansi: Option<Art>,
}
impl Default for Theme {
    fn default() -> Self {
        Self {
            id: "builtin-text".into(),
            board: "Faction BBS".into(),
            welcome: "Independent boards. Private rooms.".into(),
            logout: "Until next time.".into(),
            normal: Style::new(7, 0, false),
            focus: Style::new(15, 4, true),
            hotkey: Style::new(11, 0, true),
            disabled: Style::new(8, 0, false),
            doors_enabled: false,
            files_enabled: false,
            entries: [
                (Action::Rooms, b'R', "Message rooms"),
                (Action::New, b'N', "New messages"),
                (Action::History, b'H', "Room history"),
                (Action::Post, b'P', "Post a message"),
                (Action::Chat, b'C', "Local chat"),
                (Action::Sysop, b'S', "Sysop controls"),
                (Action::Doors, b'D', "Doors"),
                (Action::Terminal, b'T', "Terminal settings"),
                (Action::Quit, b'Q', "Log out"),
            ]
            .into_iter()
            .map(|(action, key, label)| Entry {
                action,
                key,
                label: label.into(),
            })
            .collect(),
            identity_ascii: None,
            identity_ansi: None,
        }
    }
}
const BASE_KEYS: &[&str] = &[
    "version",
    "id",
    "board",
    "welcome",
    "logout",
    "normal",
    "focus",
    "hotkey",
    "disabled",
    "menu",
    "identity_ascii",
    "identity_ansi",
    "creator",
    "source",
    "permission",
];
const SEASON_KEYS: &[&str] = &[
    "version",
    "id",
    "welcome",
    "normal",
    "focus",
    "hotkey",
    "disabled",
    "identity_ascii",
    "identity_ansi",
    "creator",
    "source",
    "permission",
];
fn unknown(table: &Table, allowed: &[&str]) -> Result<()> {
    if table.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err("unknown theme field");
    }
    Ok(())
}
fn text(value: &Value, max: usize) -> Result<&str> {
    let s = value.as_str().ok_or("theme field must be text")?;
    if s.is_empty() || s.len() > max || !s.bytes().all(|b| b.is_ascii_graphic() || b == b' ') {
        return Err("theme text must be bounded printable ASCII");
    }
    Ok(s)
}
fn read_file(root: &Path, name: &str, cap: usize) -> Result<Vec<u8>> {
    if name.is_empty()
        || name.len() > 64
        || !name.as_bytes()[0].is_ascii_alphanumeric()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err("theme assets must be simple local filenames");
    }
    let path = root.join(name);
    let m = fs::symlink_metadata(&path).map_err(|_| "theme file unavailable")?;
    if !m.is_file()
        || m.uid() != rustix::process::geteuid().as_raw()
        || m.mode() & 0o022 != 0
        || m.len() > cap as u64
    {
        return Err(
            "theme files must be owned regular files, bounded and not group/world writable",
        );
    }
    let mut data = Vec::new();
    File::open(path)
        .map_err(|_| "theme file unavailable")?
        .take((cap + 1) as u64)
        .read_to_end(&mut data)
        .map_err(|_| "theme read failed")?;
    if data.len() > cap {
        return Err("theme file too large");
    }
    Ok(data)
}
fn load_table(root: &Path, allowed: &[&str]) -> Result<Table> {
    let m = fs::symlink_metadata(root).map_err(|_| "theme directory unavailable")?;
    if !m.is_dir() || m.uid() != rustix::process::geteuid().as_raw() || m.mode() & 0o022 != 0 {
        return Err("theme directory must be owned and not group/world writable");
    }
    let data = read_file(root, "theme.toml", 8192)?;
    let source = std::str::from_utf8(&data).map_err(|_| "theme must be UTF-8 TOML")?;
    if source.bytes().filter(|b| *b == b'[').count() > 64 {
        return Err("theme nesting/table count exceeds limit");
    }
    let table = source.parse::<Table>().map_err(|_| "invalid theme TOML")?;
    unknown(&table, allowed)?;
    if table.get("version").and_then(Value::as_integer) != Some(1) {
        return Err("unsupported theme version");
    }
    let id = text(table.get("id").ok_or("theme ID required")?, 32)?;
    if !id
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err("invalid theme ID");
    }
    Ok(table)
}
fn style(value: &Value) -> Result<Style> {
    let t = value.as_table().ok_or("style must be a table")?;
    unknown(t, &["fg", "bg", "bold"])?;
    let color = |key| -> Result<u8> {
        let n = t
            .get(key)
            .and_then(Value::as_integer)
            .ok_or("style color required")?;
        if !(0..=15).contains(&n) {
            return Err("style color must be 0-15");
        }
        Ok(n as u8)
    };
    let bold = match t.get("bold") {
        Some(v) => v.as_bool().ok_or("bold must be boolean")?,
        None => false,
    };
    let (fg, bg) = (color("fg")?, color("bg")?);
    if fg == bg {
        return Err("style foreground and background must differ");
    }
    Ok(Style::new(fg, bg, bold))
}
impl Theme {
    pub fn set_files_enabled(&mut self, enabled: bool) -> Result<()> {
        self.files_enabled = enabled;
        if enabled && !self.entries.iter().any(|e| e.action == Action::Files) {
            if self.entries.iter().any(|e| e.key == b'F') {
                return Err("F is reserved by this pack; add a files action with another key");
            }
            let index = self.entries.len() - 1;
            self.entries.insert(
                index,
                Entry {
                    action: Action::Files,
                    key: b'F',
                    label: "File areas".into(),
                },
            );
        }
        Ok(())
    }
    /// Runtime capability, set only by the host; a theme cannot enable a door.
    pub fn set_doors_enabled(&mut self, enabled: bool) {
        self.doors_enabled = enabled;
    }
    fn enabled(&self, action: Action, sysop: bool) -> bool {
        (action == Action::Files && self.files_enabled)
            || (action == Action::Doors && self.doors_enabled)
            || action.enabled(sysop)
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn board(&self) -> &str {
        &self.board
    }
    pub fn welcome(&self) -> &str {
        &self.welcome
    }
    pub fn logout(&self) -> &str {
        &self.logout
    }
    pub fn menu_styles(&self) -> (Style, Style, Style) {
        (self.normal, self.focus, self.hotkey)
    }
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    pub fn identity_ascii(&self) -> Option<&str> {
        self.identity_ascii.as_deref()
    }
    pub fn identity_ansi(&self) -> Option<&Art> {
        self.identity_ansi.as_ref()
    }
    fn apply(&mut self, root: &Path, t: &Table) -> Result<()> {
        self.id = text(&t["id"], 32)?.into();
        for (name, dest, max) in [
            ("board", &mut self.board, 36),
            ("welcome", &mut self.welcome, 36),
            ("logout", &mut self.logout, 60),
        ] {
            if let Some(value) = t.get(name) {
                *dest = text(value, max)?.into()
            }
        }
        for (name, dest) in [
            ("normal", &mut self.normal),
            ("focus", &mut self.focus),
            ("hotkey", &mut self.hotkey),
            ("disabled", &mut self.disabled),
        ] {
            if let Some(value) = t.get(name) {
                *dest = style(value)?
            }
        }
        if t.contains_key("identity_ascii") || t.contains_key("identity_ansi") {
            for key in ["creator", "source", "permission"] {
                text(t.get(key).ok_or("art provenance required")?, 160)?;
            }
        }
        if let Some(value) = t.get("identity_ascii") {
            let bytes = read_file(root, text(value, 64)?, 2048)?;
            let ascii = std::str::from_utf8(&bytes)
                .map_err(|_| "ASCII identity must be ASCII")?
                .replace("\r\n", "\n");
            if ascii.is_empty()
                || !ascii
                    .bytes()
                    .all(|b| b == b'\n' || b.is_ascii_graphic() || b == b' ')
                || ascii.lines().count() > 16
                || ascii.lines().any(|l| l.len() > 80)
            {
                return Err("ASCII identity exceeds supported screen");
            }
            self.identity_ascii = Some(ascii);
        }
        if let Some(value) = t.get("identity_ansi") {
            self.identity_ansi = Some(Art::parse(&read_file(root, text(value, 64)?, 16384)?)?);
        }
        Ok(())
    }
    pub fn load(root: &Path) -> Result<Self> {
        let t = load_table(root, BASE_KEYS)?;
        let mut theme = Self::default();
        theme.apply(root, &t)?;
        if let Some(value) = t.get("menu") {
            let values = value.as_array().ok_or("menu must be an array")?;
            if !(6..=9).contains(&values.len()) {
                return Err("menu needs 6-9 entries");
            }
            let (mut keys, mut actions, mut entries) =
                (BTreeSet::new(), BTreeSet::new(), Vec::new());
            for value in values {
                let item = value.as_table().ok_or("menu entry must be a table")?;
                unknown(item, &["action", "key", "label"])?;
                let name = text(item.get("action").ok_or("action required")?, 16)?;
                let action = Action::parse(name)?;
                let key = text(item.get("key").ok_or("hotkey required")?, 1)?.as_bytes()[0]
                    .to_ascii_uppercase();
                if !key.is_ascii_alphabetic() || !keys.insert(key) || !actions.insert(name) {
                    return Err("duplicate/invalid action or hotkey");
                }
                let label = text(item.get("label").ok_or("label required")?, 26)?.into();
                entries.push(Entry { action, key, label });
            }
            if ["rooms", "new", "history", "post", "sysop", "quit"]
                .iter()
                .any(|s| !actions.contains(s))
            {
                return Err("required core action missing");
            }
            theme.entries = entries;
        }
        Ok(theme)
    }
    /// Seasonal overrides cannot alter labels, keys, layout or action IDs.
    pub fn with_season(&self, root: &Path) -> Result<Self> {
        let t = load_table(root, SEASON_KEYS)?;
        let mut candidate = self.clone();
        candidate.apply(root, &t)?;
        Ok(candidate)
    }
    pub fn plain_menu(&self, sysop: bool) -> String {
        let mut out = format!("\n{}\n{}\n", self.board, self.welcome);
        for e in &self.entries {
            writeln!(
                out,
                "[{}] {}{}",
                e.key as char,
                e.label,
                if self.enabled(e.action, sysop) {
                    ""
                } else {
                    " (unavailable)"
                }
            )
            .expect("string output");
        }
        out
    }
    pub fn hotkey(&self, key: u8, sysop: bool) -> Option<Action> {
        self.entries
            .iter()
            .find(|e| e.key == key.to_ascii_uppercase() && self.enabled(e.action, sysop))
            .map(|e| e.action)
    }
}
pub struct Menu<'a> {
    theme: &'a Theme,
    sysop: bool,
    focus: usize,
}
impl<'a> Menu<'a> {
    pub fn new(theme: &'a Theme, sysop: bool) -> Self {
        Self {
            theme,
            sysop,
            focus: theme
                .entries
                .iter()
                .position(|e| theme.enabled(e.action, sysop))
                .expect("required core actions"),
        }
    }
    pub fn focus(&self) -> usize {
        self.focus
    }
    pub fn feed(&mut self, key: Key) -> Option<Action> {
        match key {
            Key::Cancel => Some(Action::Quit),
            Key::Enter => Some(self.theme.entries[self.focus].action),
            Key::Hotkey(k) => self.theme.hotkey(k, self.sysop),
            Key::Up | Key::Down => {
                let n = self.theme.entries.len();
                loop {
                    self.focus = if key == Key::Up {
                        (self.focus + n - 1) % n
                    } else {
                        (self.focus + 1) % n
                    };
                    if self
                        .theme
                        .enabled(self.theme.entries[self.focus].action, self.sysop)
                    {
                        break;
                    }
                }
                None
            }
        }
    }
    fn row(&self, index: usize) -> String {
        let e = &self.theme.entries[index];
        let enabled = self.theme.enabled(e.action, self.sysop);
        let focused = index == self.focus;
        let style = if !enabled {
            self.theme.disabled
        } else if focused {
            self.theme.focus
        } else {
            self.theme.normal
        };
        format!(
            "\x1b[{};1H{}{} {}[{}]{} {}\x1b[K\x1b[0m",
            index + 5,
            style.sgr(),
            if focused { ">" } else { " " },
            if enabled {
                self.theme.hotkey.sgr()
            } else {
                self.theme.disabled.sgr()
            },
            e.key as char,
            style.sgr(),
            e.label
        )
    }
    pub fn render(&self) -> String {
        let mut out = format!(
            "{}\x1b[2J\x1b[H{}\r\n{}\r\n\r\nArrows select; Enter or hotkey opens.\x1b[0m",
            self.theme.normal.sgr(),
            self.theme.board,
            self.theme.welcome
        );
        for i in 0..self.theme.entries.len() {
            out.push_str(&self.row(i));
        }
        write!(
            out,
            "{}\x1b[{};1HCtrl-C exits; Esc: Enter to recover.\x1b[K\x1b[0m",
            self.theme.normal.sgr(),
            self.theme.entries.len() + 7
        )
        .expect("string output");
        out
    }
    pub fn redraw(&self, previous: usize) -> String {
        if previous == self.focus {
            String::new()
        } else {
            format!("{}{}", self.row(previous), self.row(self.focus))
        }
    }
}
