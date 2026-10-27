use faction_terminal::{
    Theme,
    seasons::{Date, Themes, Timezone},
};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt, symlink},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "faction-seasons-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let f = Self(root);
        f.pack("base");
        f.pack("autumn");
        f.pack("winter");
        f
    }
    fn pack(&self, id: &str) {
        let root = self.0.join(id);
        fs::create_dir(&root).unwrap();
        fs::write(root.join("theme.toml"),format!("version=1\nid='{id}'\nwelcome='{id} welcome'\nidentity_ascii='login.txt'\nidentity_ansi='login.ans'\ncreator='Faction test'\nsource='Original synthetic fixture'\npermission='BSD-3-Clause'\n")).unwrap();
        fs::write(root.join("login.txt"), format!("{id} ASCII\n")).unwrap();
        fs::write(root.join("login.ans"), format!("\x1b[31m{id} ANSI\x1b[0m")).unwrap();
    }
    fn base(&self) -> Theme {
        Theme::load(&self.0.join("base")).unwrap()
    }
    fn schedule(&self, text: &str) {
        fs::write(self.0.join("schedule.toml"), text).unwrap();
    }
    fn load(&self) -> Result<Themes, &'static str> {
        Themes::load(self.base(), None, Some(&self.0))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
const HEADER: &str = "version=1\nenabled=true\ntimezone='UTC'\n";
fn window(pack: &str, start: &str, end: &str, priority: u8) -> String {
    format!("[[season]]\npack='{pack}'\nstart='{start}'\nend='{end}'\npriority={priority}\n")
}
fn at(themes: &Themes, date: &str) -> String {
    themes
        .get(themes.select(Some(Date::parse(date).unwrap())))
        .id()
        .into()
}
#[test]
fn annual_boundaries_wrap_leap_and_unknown_clock_use_valid_dates() {
    let f = Fixture::new();
    f.schedule(
        &(HEADER.to_owned()
            + &window("autumn", "09-01", "12-01", 10)
            + &window("winter", "12-01", "03-01", 10)),
    );
    let themes = f.load().unwrap();
    assert_eq!(themes.timezone(), Some(Timezone::Utc));
    for (date, id) in [
        ("2026-08-31", "base"),
        ("2026-09-01", "autumn"),
        ("2026-11-30", "autumn"),
        ("2026-12-01", "winter"),
        ("2027-01-01", "winter"),
        ("2028-02-29", "winter"),
        ("2028-03-01", "base"),
    ] {
        assert_eq!(at(&themes, date), id);
    }
    assert_eq!(themes.select(None), 0);
    assert_eq!(themes.select(themes.date_at(-1)), 0);
    assert!(Date::parse("2027-02-29").is_err());
    assert!(Date::parse("1900-02-29").is_err());
    assert!(Date::parse("2000-02-29").is_ok());
    for bad in [
        "0000-01-01",
        "2026-13-01",
        "2026-01-32",
        "2026-1-01",
        "2026-0é-1",
    ] {
        assert!(Date::parse(bad).is_err());
    }
}
#[test]
fn overlap_priority_is_order_independent_and_ties_are_rejected() {
    let f = Fixture::new();
    let a = window("autumn", "09-01", "12-01", 10);
    let b = window("winter", "10-15", "11-01", 20);
    for rules in [a.clone() + &b, b + &a] {
        f.schedule(&(HEADER.to_owned() + &rules));
        let themes = f.load().unwrap();
        assert_eq!(at(&themes, "2026-10-22"), "winter");
        assert_eq!(at(&themes, "2026-11-01"), "autumn");
    }
    f.schedule(&(HEADER.to_owned() + &a + &window("winter", "10-15", "11-01", 10)));
    assert!(f.load().is_err());
    f.schedule(
        &(HEADER.to_owned()
            + &window("autumn", "02-29", "03-01", 1)
            + &window("winter", "02-28", "03-02", 1)),
    );
    assert!(f.load().is_err());
}
#[test]
fn disabled_calendar_manual_precedence_and_broken_asset_recovery() {
    let f = Fixture::new();
    f.schedule(&(HEADER.replace("true", "false") + &window("missing", "09-01", "12-01", 1)));
    assert_eq!(f.load().unwrap().len(), 1);
    f.schedule("not TOML");
    let manual = Themes::load(f.base(), Some(&f.0.join("winter")), Some(&f.0)).unwrap();
    assert_eq!(manual.get(0).id(), "winter");
    assert!(manual.timezone().is_none());
    f.schedule(&(HEADER.to_owned() + &window("autumn", "09-01", "12-01", 1)));
    fs::write(f.0.join("autumn/login.ans"), b"\x1b]52;c;no\x07").unwrap();
    let base = f.base();
    assert!(Themes::load(base.clone(), None, Some(&f.0)).is_err());
    assert_eq!(base.id(), "base");
    f.schedule(&HEADER.replace("true", "false"));
    assert_eq!(f.load().unwrap().get(0).id(), "base");
}
#[test]
fn schedule_schema_paths_permissions_and_size_are_bounded() {
    let f = Fixture::new();
    for source in [
        HEADER.replace("UTC", "America/Denver"),
        HEADER.replace("version=1", "version=2"),
        HEADER.to_owned() + "shell='no'",
        HEADER.to_owned() + &window("../winter", "09-01", "12-01", 1),
        HEADER.to_owned() + &window("winter", "09-01", "09-01", 1),
        HEADER.to_owned() + &window("winter", "02-30", "03-01", 1),
        HEADER.to_owned()
            + &window("winter", "01-01", "03-01", 1)
            + &window("winter", "06-01", "07-01", 2),
        "#".repeat(8193),
    ] {
        f.schedule(&source);
        assert!(f.load().is_err());
    }
    f.schedule(HEADER);
    fs::set_permissions(f.0.join("schedule.toml"), fs::Permissions::from_mode(0o666)).unwrap();
    assert!(f.load().is_err());
    fs::remove_file(f.0.join("schedule.toml")).unwrap();
    symlink("base/theme.toml", f.0.join("schedule.toml")).unwrap();
    assert!(f.load().is_err());
}
#[test]
fn eight_preloaded_seasons_and_pinned_sessions_do_not_reload_art() {
    let f = Fixture::new();
    let mut source = HEADER.to_owned();
    for i in 0..8 {
        let name = format!("pack{i}");
        f.pack(&name);
        source += &window(&name, "09-01", "12-01", i);
    }
    f.schedule(&source);
    let themes = f.load().unwrap();
    assert_eq!(themes.len(), 9);
    let pinned = themes.select(Some(Date::parse("2026-10-22").unwrap()));
    assert_eq!(themes.get(pinned).id(), "pack7");
    fs::write(
        f.0.join("pack7/login.txt"),
        "changed while member is composing",
    )
    .unwrap();
    f.schedule("broken while online");
    assert_eq!(themes.select(Some(Date::parse("2026-12-01").unwrap())), 0);
    assert_eq!(themes.get(pinned).identity_ascii(), Some("pack7 ASCII\n"));
    assert_eq!(
        themes.select(Some(Date::parse("2027-10-22").unwrap())),
        pinned
    );
    f.pack("pack8");
    f.schedule(&(source + &window("pack8", "09-01", "12-01", 8)));
    assert!(f.load().is_err());
}
#[test]
fn seasonal_identity_classic_fallback_and_action_map_stay_coherent() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
    let base = Theme::load(&root.join("base")).unwrap();
    let commands = |theme: &Theme| {
        theme
            .entries()
            .iter()
            .map(|e| {
                (
                    e.action(),
                    e.key(),
                    e.label().to_owned(),
                    e.description().to_owned(),
                )
            })
            .collect::<Vec<_>>()
    };
    for name in ["autumn", "winter"] {
        let theme = base.with_season(&root.join("seasons").join(name)).unwrap();
        assert_eq!(commands(&theme), commands(&base));
        let full = theme.identity_screen(80, 24, true);
        assert!(full.contains("faction / bbs"));
        assert!(full.contains("\x1b[18;1H"));
        assert!(full.len() < 16384);
        assert!(theme.identity_needs_pause(40, 18, true));
        assert!(!theme.identity_needs_pause(40, 18, false));
        let compact = theme.identity_screen(40, 18, true);
        assert!(compact.contains("FACTION //"));
        assert!(!compact.contains("faction / bbs"));
        let plain = theme.identity_screen(40, 18, false);
        assert!(plain.contains("FACTION //"));
        assert!(!plain.contains('\x1b'));
        assert!(plain.lines().all(|l| l.len() < 40));
        assert_eq!(theme.identity_screen(20, 5, true), theme.board());
        assert_eq!(theme.identity_screen(0, 0, false), theme.board());
        assert_eq!(theme.hotkey(b'S', false), None);
        assert_eq!(theme.hotkey(b'D', false), None);
    }
}
#[test]
fn oversized_ascii_identity_falls_back_instead_of_wrapping_compact_login() {
    let f = Fixture::new();
    fs::write(f.0.join("base/login.txt"), format!("{}\n", "x".repeat(79))).unwrap();
    let theme = f.base();
    assert!(
        !theme
            .identity_screen(40, 18, true)
            .contains(&"x".repeat(79))
    );
    assert!(
        theme
            .identity_screen(80, 24, false)
            .contains(&"x".repeat(79))
    );
}
