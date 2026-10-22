//! Bounded, startup-validated annual calendar. No member input, filesystem reload
//! or mid-session mutation; callers pin the selected immutable theme at login.
use crate::{Result, Theme};
use std::{
    collections::BTreeSet,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use toml::{Table, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}
impl Date {
    pub fn new(year: u16, month: u8, day: u8) -> Option<Self> {
        if year == 0 || year > 9999 || !(1..=12).contains(&month) {
            return None;
        }
        let leap =
            year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
        let max = [
            31,
            if leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ][usize::from(month) - 1];
        (day > 0 && day <= max).then_some(Self { year, month, day })
    }
    pub fn parse(s: &str) -> Result<Self> {
        let bytes = s.as_bytes();
        if bytes.len() != 10
            || bytes[4] != b'-'
            || bytes[7] != b'-'
            || !bytes
                .iter()
                .enumerate()
                .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
        {
            return Err("date must be YYYY-MM-DD");
        }
        Self::new(
            s[..4].parse().map_err(|_| "invalid year")?,
            s[5..7].parse().map_err(|_| "invalid month")?,
            s[8..].parse().map_err(|_| "invalid day")?,
        )
        .ok_or("invalid calendar date")
    }
    pub fn fields(self) -> (u16, u8, u8) {
        (self.year, self.month, self.day)
    }
    fn key(self) -> u16 {
        u16::from(self.month) * 100 + u16::from(self.day)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timezone {
    Utc,
    System,
}
struct Window {
    start: u16,
    end: u16,
    priority: u8,
}
impl Window {
    fn matches(&self, key: u16) -> bool {
        if self.start < self.end {
            (self.start..self.end).contains(&key)
        } else {
            key >= self.start || key < self.end
        }
    }
    fn overlaps(&self, other: &Self) -> bool {
        (1..=12).any(|month| {
            (1..=31).any(|day| {
                Date::new(2000, month, day)
                    .is_some_and(|d| self.matches(d.key()) && other.matches(d.key()))
            })
        })
    }
}
pub struct Themes {
    themes: Vec<Theme>,
    windows: Vec<Window>,
    timezone: Option<Timezone>,
}
impl Themes {
    pub fn fixed(theme: Theme) -> Self {
        Self {
            themes: vec![theme],
            windows: Vec::new(),
            timezone: None,
        }
    }
    pub fn load(base: Theme, manual: Option<&Path>, schedule: Option<&Path>) -> Result<Self> {
        // An explicit manual override is also the recovery path for a disabled or
        // broken calendar; it deliberately bypasses schedule loading entirely.
        if let Some(path) = manual {
            return Ok(Self::fixed(base.with_season(path)?));
        }
        let Some(root) = schedule else {
            return Ok(Self::fixed(base));
        };
        // Reuse the pack directory/file ownership, no-symlink and size checks.
        let metadata =
            std::fs::symlink_metadata(root).map_err(|_| "schedule directory unavailable")?;
        use std::os::unix::fs::MetadataExt;
        if !metadata.is_dir()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o022 != 0
        {
            return Err("schedule directory must be owned and not group/world writable");
        }
        let bytes = crate::theme::read_file(root, "schedule.toml", 8192)?;
        let source = std::str::from_utf8(&bytes).map_err(|_| "schedule must be UTF-8")?;
        if source.bytes().filter(|b| *b == b'[').count() > 64 {
            return Err("schedule nesting exceeds limit");
        }
        let mut t = source
            .parse::<Table>()
            .map_err(|_| "invalid schedule TOML")?;
        if t.remove("version").and_then(|v| v.as_integer()) != Some(1) {
            return Err("unsupported schedule version");
        }
        let enabled = t
            .remove("enabled")
            .and_then(|v| v.as_bool())
            .ok_or("schedule enabled boolean required")?;
        let timezone = match t
            .remove("timezone")
            .and_then(|v| v.as_str().map(str::to_owned))
            .as_deref()
        {
            Some("UTC") => Timezone::Utc,
            Some("system") => Timezone::System,
            _ => return Err("timezone must be UTC or system"),
        };
        let entries = match t.remove("season") {
            Some(Value::Array(v)) => v,
            None => Vec::new(),
            _ => return Err("season must be an array"),
        };
        if !t.is_empty() || entries.len() > 8 {
            return Err("unknown schedule field or more than eight seasons");
        }
        let mut windows = Vec::<Window>::new();
        let mut packs = Vec::new();
        let mut seen = BTreeSet::new();
        for value in entries {
            let mut entry = value.as_table().ok_or("season must be a table")?.clone();
            let pack = entry
                .remove("pack")
                .and_then(|v| v.as_str().map(str::to_owned))
                .ok_or("season pack required")?;
            if pack.is_empty()
                || pack.len() > 32
                || !pack
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                || !seen.insert(pack.clone())
            {
                return Err("season packs must be unique simple IDs");
            }
            let mut boundary = |name| -> Result<u16> {
                let s = entry
                    .remove(name)
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .ok_or("season boundary required")?;
                // A leap reference year permits Feb 29 without inventing it in
                // non-leap years. Matching always uses an actual valid date.
                if s.len() != 5 {
                    return Err("season boundary must be MM-DD");
                }
                Ok(Date::parse(&format!("2000-{s}"))?.key())
            };
            let start = boundary("start")?;
            let end = boundary("end")?;
            let priority = entry
                .remove("priority")
                .and_then(|v| v.as_integer())
                .and_then(|v| u8::try_from(v).ok())
                .ok_or("season priority must be 0-255")?;
            if !entry.is_empty() || start == end {
                return Err("unknown season field or empty annual window");
            }
            let window = Window {
                start,
                end,
                priority,
            };
            if windows
                .iter()
                .any(|w| w.priority == priority && w.overlaps(&window))
            {
                return Err("overlapping seasons need distinct priorities");
            }
            windows.push(window);
            packs.push(pack);
        }
        if !enabled {
            return Ok(Self::fixed(base));
        }
        let mut themes = vec![base];
        for pack in packs {
            themes.push(themes[0].with_season(&root.join(pack))?);
        }
        if timezone == Timezone::System {
            faction_unix::init_calendar();
        }
        Ok(Self {
            themes,
            windows,
            timezone: Some(timezone),
        })
    }
    pub fn timezone(&self) -> Option<Timezone> {
        self.timezone
    }
    pub fn len(&self) -> usize {
        self.themes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.themes.is_empty()
    }
    pub fn get(&self, index: usize) -> &Theme {
        self.themes.get(index).unwrap_or(&self.themes[0])
    }
    pub fn select(&self, date: Option<Date>) -> usize {
        let Some(date) = date else {
            return 0;
        };
        self.windows
            .iter()
            .enumerate()
            .filter(|(_, w)| w.matches(date.key()))
            .max_by_key(|(_, w)| w.priority)
            .map_or(0, |(i, _)| i + 1)
    }
    pub fn date_at(&self, seconds: i64) -> Option<Date> {
        let timezone = self.timezone?;
        let (y, m, d) = faction_unix::calendar_date(seconds, timezone == Timezone::System)?;
        Date::new(y, m, d)
    }
    pub fn select_now(&self) -> usize {
        let date = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|d| i64::try_from(d.as_secs()).ok())
            .and_then(|s| self.date_at(s));
        self.select(date)
    }
    pub fn set_directory_enabled(&mut self, enabled: bool) -> Result<()> {
        for theme in &mut self.themes {
            theme.set_directory_enabled(enabled)?;
        }
        Ok(())
    }
    /// Startup-only host capabilities, never a member-controlled selection.
    pub fn set_services(&mut self, files: bool, doors: bool) -> Result<()> {
        for theme in &mut self.themes {
            theme.set_files_enabled(files)?;
            theme.set_doors_enabled(doors);
        }
        Ok(())
    }
}
