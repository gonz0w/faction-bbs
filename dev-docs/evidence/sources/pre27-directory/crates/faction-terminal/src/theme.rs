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
    description: String,
    position: Option<(usize, usize)>,
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
    pub fn description(&self) -> &str {
        &self.description
    }
}
#[derive(Clone)]
pub struct Theme {
    id: String,
    board: String,
    menu_title: String,
    welcome: String,
    logout: String,
    normal: Style,
    focus: Style,
    hotkey: Style,
    disabled: Style,
    doors_enabled: bool,
    files_enabled: bool,
    entries: Vec<Entry>,
    grid: bool,
    identity_ascii: Option<String>,
    identity_ansi: Option<Art>,
}
impl Default for Theme {
    fn default() -> Self {
        Self {
            id: "builtin-text".into(),
            board: "Faction BBS".into(),
            menu_title: "Main menu".into(),
            welcome: "Independent boards. Private rooms.".into(),
            logout: "Until next time.".into(),
            normal: Style::new(7, 0, false),
            focus: Style::new(15, 4, true),
            hotkey: Style::new(11, 0, true),
            disabled: Style::new(8, 0, false),
            doors_enabled: false,
            files_enabled: false,
            grid: false,
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
                description: String::new(),
                position: None,
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
    "menu_title",
    "welcome",
    "logout",
    "normal",
    "focus",
    "hotkey",
    "disabled",
    "menu",
    "layout",
    "identity_ascii",
    "identity_ansi",
    "identity_ansi_long",
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
    "identity_ansi_long",
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
pub(crate) fn read_file(root: &Path, name: &str, cap: usize) -> Result<Vec<u8>> {
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
                    description: "Browse and transfer local files".into(),
                    position: if self.grid {
                        (0..5)
                            .flat_map(|row| (0..2).map(move |col| (row, col)))
                            .find(|p| !self.entries.iter().any(|e| e.position == Some(*p)))
                    } else {
                        None
                    },
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
    pub fn menu_title(&self) -> &str {
        &self.menu_title
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
            ("menu_title", &mut self.menu_title, 24),
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
        if t.contains_key("identity_ansi") && t.contains_key("identity_ansi_long") {
            return Err("choose short or long ANSI identity, not both");
        }
        if t.contains_key("identity_ascii")
            || t.contains_key("identity_ansi")
            || t.contains_key("identity_ansi_long")
        {
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
        if let Some(value) = t.get("identity_ansi_long") {
            self.identity_ansi = Some(Art::parse_long(&read_file(root, text(value, 64)?, 65536)?)?);
        }
        Ok(())
    }
    pub fn load(root: &Path) -> Result<Self> {
        let t = load_table(root, BASE_KEYS)?;
        let mut theme = Self::default();
        theme.apply(root, &t)?;
        theme.grid = match t.get("layout").and_then(Value::as_str) {
            None if !t.contains_key("layout") => false,
            Some("list") => false,
            Some("grid") => true,
            _ => return Err("layout must be list or grid"),
        };
        if theme.grid && !t.contains_key("menu") {
            return Err("grid requires explicit menu entries and coordinates");
        }
        let mut positions = BTreeSet::new();
        if let Some(value) = t.get("menu") {
            let values = value.as_array().ok_or("menu must be an array")?;
            if !(6..=10).contains(&values.len()) {
                return Err("menu needs 6-10 entries");
            }
            let (mut keys, mut actions, mut entries) =
                (BTreeSet::new(), BTreeSet::new(), Vec::new());
            for value in values {
                let item = value.as_table().ok_or("menu entry must be a table")?;
                unknown(
                    item,
                    &["action", "key", "label", "description", "row", "column"],
                )?;
                let position = if theme.grid {
                    let row = item
                        .get("row")
                        .and_then(Value::as_integer)
                        .filter(|n| (1..=5).contains(n))
                        .ok_or("grid row must be 1-5")? as usize
                        - 1;
                    let col = item
                        .get("column")
                        .and_then(Value::as_integer)
                        .filter(|n| (1..=2).contains(n))
                        .ok_or("grid column must be 1-2")? as usize
                        - 1;
                    if !positions.insert((row, col)) {
                        return Err("duplicate grid position");
                    }
                    Some((row, col))
                } else {
                    if item.contains_key("row") || item.contains_key("column") {
                        return Err("entry coordinates require grid layout");
                    }
                    None
                };
                let name = text(item.get("action").ok_or("action required")?, 16)?;
                let action = Action::parse(name)?;
                let key = text(item.get("key").ok_or("hotkey required")?, 1)?.as_bytes()[0]
                    .to_ascii_uppercase();
                if !key.is_ascii_alphabetic() || !keys.insert(key) || !actions.insert(name) {
                    return Err("duplicate/invalid action or hotkey");
                }
                let label = text(item.get("label").ok_or("label required")?, 26)?.into();
                let description = item
                    .get("description")
                    .map(|v| text(v, 30).map(str::to_owned))
                    .transpose()?
                    .unwrap_or_default();
                entries.push(Entry {
                    action,
                    key,
                    label,
                    description,
                    position,
                });
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
        let mut out = format!("\n{}\n{}\n{}\n", self.board, self.welcome, self.menu_title);
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
    columns: usize,
}
impl<'a> Menu<'a> {
    pub fn new(theme: &'a Theme, sysop: bool) -> Self {
        Self::with_columns(theme, sysop, 40)
    }
    /// The caller still owns geometry validation and plain-mode fallback.
    pub fn with_columns(theme: &'a Theme, sysop: bool, columns: u16) -> Self {
        Self {
            columns: usize::from(columns.clamp(40, 240)),
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
            Key::Up | Key::Down | Key::Left | Key::Right if self.grid() => {
                self.move_grid(key);
                None
            }
            Key::Up | Key::Down | Key::Left | Key::Right | Key::Next => {
                let n = self.theme.entries.len();
                loop {
                    self.focus = if matches!(key, Key::Up | Key::Left) {
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
    /// Restore an entry index after reconstructing the borrowed menu. Grid
    /// arrows are spatial, so replaying Down would not restore array order.
    pub fn restore_focus(&mut self, index: usize) {
        if self
            .theme
            .entries
            .get(index)
            .is_some_and(|e| self.theme.enabled(e.action, self.sysop))
        {
            self.focus = index;
        }
    }
    fn grid(&self) -> bool {
        self.theme.grid && self.columns >= 80
    }
    fn move_grid(&mut self, key: Key) {
        let (row, col) = self.theme.entries[self.focus]
            .position
            .expect("validated grid");
        let candidate = self
            .theme
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| self.theme.enabled(e.action, self.sysop))
            .filter_map(|(i, e)| {
                let (r, c) = e.position.expect("validated grid");
                let rank = match key {
                    Key::Up if c == col && r != row => ((row + 5 - r) % 5, 0),
                    Key::Down if c == col && r != row => ((r + 5 - row) % 5, 0),
                    // Across columns, prefer the same row, then nearest row;
                    // ties choose the upper row. No disabled cell gets focus.
                    Key::Left | Key::Right if c != col => (row.abs_diff(r), r),
                    _ => return None,
                };
                Some((rank, i))
            })
            .min();
        if let Some((_, i)) = candidate {
            self.focus = i;
        }
    }
    fn width(&self) -> usize {
        if self.columns >= 80 { 76 } else { 38 }
    }
    fn position(&self, row: usize) -> String {
        format!("\x1b[{};{}H", row, (self.columns - self.width()) / 2 + 1)
    }
    fn line(&self, row: usize, text: &str, style: Style) -> String {
        format!(
            "{}{}{:<width$}\x1b[0m",
            self.position(row),
            style.sgr(),
            text,
            width = self.width()
        )
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
        // Keep the hotkey on the lightbar's background instead of punching a
        // different-colored hole into the selected row.
        let key_style = if !enabled || focused {
            style
        } else {
            self.theme.hotkey
        };
        if self.grid() {
            let (row, col) = e.position.expect("validated grid");
            let left = (self.columns - self.width()) / 2 + 2 + col * 37;
            return format!(
                "\x1b[{};{}H{}{} {}[{}]{} {:<26}{}\x1b[0m",
                row + 5,
                left,
                style.sgr(),
                if focused { ">" } else { " " },
                key_style.sgr(),
                e.key as char,
                style.sgr(),
                e.label,
                if enabled { "    " } else { " -- " }
            );
        }
        let tail = if self.width() >= 76 {
            format!(
                " | {:<30}",
                if enabled {
                    &e.description
                } else {
                    "(unavailable)"
                }
            )
        } else if !enabled {
            " --".into()
        } else {
            String::new()
        };
        let used = 32 + tail.len(); // ASCII marker, hotkey, label and optional detail.
        format!(
            "{}{}|{} {}[{}]{} {:<26}{}{}|\x1b[0m",
            self.position(index + 5),
            style.sgr(),
            if focused { ">" } else { " " },
            key_style.sgr(),
            e.key as char,
            style.sgr(),
            e.label,
            tail,
            " ".repeat(self.width() - 2 - used)
        )
    }
    pub fn render(&self) -> String {
        let mut out = format!("{}\x1b[2J\x1b[H", self.theme.normal.sgr());
        out.push_str(&self.line(1, &self.theme.board, self.theme.hotkey));
        out.push_str(&self.line(2, &self.theme.welcome, self.theme.normal));
        let rule = format!(
            "+-- {} {}+",
            self.theme.menu_title,
            "-".repeat(self.width() - self.theme.menu_title.len() - 6)
        );
        out.push_str(&self.line(3, &rule, self.theme.hotkey));
        out.push_str(&self.line(
            4,
            &format!(
                "|{:<inner$}|",
                "Arrows select; Enter/hotkey opens.",
                inner = self.width() - 2
            ),
            self.theme.normal,
        ));
        if self.grid() {
            for row in 5..10 {
                out.push_str(&self.line(
                    row,
                    &format!("|{}|", " ".repeat(self.width() - 2)),
                    self.theme.normal,
                ));
            }
        }
        for i in 0..self.theme.entries.len() {
            out.push_str(&self.row(i));
        }
        let bottom = if self.grid() {
            10
        } else {
            self.theme.entries.len() + 5
        };
        out.push_str(&self.line(
            bottom,
            &format!("+{}+", "-".repeat(self.width() - 2)),
            self.theme.hotkey,
        ));
        out.push_str(&self.line(
            bottom + 1,
            "Tab advances; -- unavailable.",
            self.theme.normal,
        ));
        // Stable recovery instruction in every theme; never operator-overridden.
        out.push_str(&self.line(
            bottom + 2,
            "Ctrl-C exits; Esc: Enter to recover.",
            self.theme.normal,
        ));
        if self.grid() {
            out.push_str(&self.detail());
        }
        out
    }
    fn detail(&self) -> String {
        self.line(
            13,
            &self.theme.entries[self.focus].description,
            self.theme.normal,
        )
    }
    pub fn redraw(&self, previous: usize) -> String {
        if previous == self.focus {
            String::new()
        } else {
            format!(
                "{}{}{}",
                self.row(previous),
                self.row(self.focus),
                if self.grid() {
                    self.detail()
                } else {
                    String::new()
                }
            )
        }
    }
}
