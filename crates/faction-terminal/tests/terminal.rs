use faction_terminal::{Action, Art, Decoder, Key, Menu, Theme};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt, symlink},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
struct Pack(PathBuf);
impl Pack {
    fn new(source: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "faction-theme-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        let pack = Self(path);
        pack.source(source);
        pack
    }
    fn source(&self, text: &str) {
        fs::write(self.0.join("theme.toml"), text).unwrap();
    }
}
impl Drop for Pack {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
const BASE: &str = "version=1\nid='test'\nboard='Test board'\n";
fn decode(bytes: &[u8]) -> Vec<Key> {
    let mut d = Decoder::new();
    bytes
        .iter()
        .filter_map(|b| d.feed(*b, Instant::now()))
        .collect()
}
#[test]
fn split_coalesced_and_crlf_keys() {
    assert_eq!(
        decode(b"\x1b[A\x1b[B\x1bOC\x1bODpP\r\n"),
        vec![
            Key::Up,
            Key::Down,
            Key::Down,
            Key::Up,
            Key::Hotkey(b'P'),
            Key::Hotkey(b'P'),
            Key::Enter
        ]
    );
    let now = Instant::now();
    let mut d = Decoder::new();
    assert_eq!(d.feed(27, now), None);
    assert_eq!(d.feed(b'[', now + Duration::from_millis(40)), None);
    assert_eq!(
        d.feed(b'B', now + Duration::from_millis(80)),
        Some(Key::Down)
    );
}
#[test]
fn malicious_sequence_tails_never_activate() {
    for input in [
        b"\x1b[99999999999999999999999999999DP\r\nP".as_slice(),
        b"\x1b[?25lDP\r\nP",
        b"\x1bOPP\r\nP",
        b"\x1b]52;c;P\nDP\x1b\\P",
        b"\x1bPpayload\x07P\nDP\x1b\\P",
        b"\x1b^privateP\x1b\\P",
        b"\x9dtitleP\x9cP",
    ] {
        assert_eq!(decode(input), vec![Key::Hotkey(b'P')]);
    }
    let mut d = Decoder::new();
    let start = Instant::now();
    d.feed(27, start);
    d.expire(start + Duration::from_secs(1));
    assert_eq!(d.feed(b'P', start + Duration::from_secs(1)), None);
    assert_eq!(d.feed(b'\r', start + Duration::from_secs(1)), None);
    assert_eq!(d.feed(b'\n', start + Duration::from_secs(1)), None);
    assert_eq!(
        d.feed(b'P', start + Duration::from_secs(1)),
        Some(Key::Hotkey(b'P'))
    );
    let mut long = b"\x1bP".to_vec();
    long.extend(vec![b'P'; 100_000]);
    long.extend(b"\x1b\\Q");
    assert_eq!(decode(&long), vec![Key::Hotkey(b'Q')]);
    assert_eq!(decode(b"\x1b]unfinishedP\n\x03"), vec![Key::Cancel]);
}
#[test]
fn arrows_enter_and_casefolded_hotkeys_share_actions() {
    let t = Theme::default();
    let mut m = Menu::new(&t, false);
    for _ in 0..3 {
        assert_eq!(m.feed(Key::Down), None)
    }
    assert_eq!(m.feed(Key::Enter), Some(Action::Post));
    assert_eq!(m.feed(Key::Hotkey(b'p')), Some(Action::Post));
    assert_eq!(m.feed(Key::Hotkey(b'P')), Some(Action::Post));
    for _ in 0..100 {
        m.feed(Key::Down);
        assert!(t.entries()[m.focus()].action().enabled(false));
    }
    assert_eq!(m.feed(Key::Hotkey(b'S')), None);
    assert_eq!(m.feed(Key::Hotkey(b'D')), None);
    let mut admin = Menu::new(&t, true);
    assert_eq!(admin.feed(Key::Hotkey(b'S')), Some(Action::Sysop));
    assert_eq!(admin.feed(Key::Hotkey(b'D')), None);
}
#[test]
fn redraw_is_incremental_and_bounded() {
    let t = Theme::default();
    let mut menu = Menu::new(&t, true);
    let before = menu.focus();
    assert!(menu.render().len() < 2048);
    menu.feed(Key::Down);
    let redraw = menu.redraw(before);
    assert!(
        redraw.len() <= 256,
        "focus redraw was {} bytes",
        redraw.len()
    );
    assert!(!redraw.contains("\x1b[2J"));
    assert!(redraw.contains(">"));
    assert!(!redraw.contains(t.board()));
    println!(
        "default full menu={} bytes; focus update={} bytes",
        menu.render().len(),
        redraw.len()
    );
}
#[test]
fn ansi_cp437_is_converted_to_canonical_cells() {
    let art = Art::parse(b"\x1b[31mA\xdb\r\n\x1b[2;3Htest\x1b[s\x1b[1D!\x1b[u\x1b[0m").unwrap();
    let out = art.render();
    assert!(out.contains("A█"));
    assert!(out.contains("tes!"));
    assert!(out.contains("\x1b[2;1H"));
    assert!(!out.contains("\x1b[s"));
    assert!(!out.contains("\x1b]"));
    assert!(
        Art::parse(b"\x1b[44m\x1b[2J")
            .unwrap()
            .render()
            .contains(";44m")
    );
}
#[test]
fn malicious_and_unbounded_art_is_rejected() {
    for bad in [
        b"\x1b]52;c;clipboard\x07".as_slice(),
        b"\x1bPdevice\x1b\\",
        b"\x1b[?25l",
        b"\x1b[999;999H",
        b"\x1b[16;80HAB",
        b"\x1b[1A",
        b"\x1b[5mblink",
        b"\x1b[38;2;1;2;3m",
        b"trailer\x1aSAUCE00",
        b"\x1b[123456789012345678901234567890123456m",
        b"\x1b[",
    ] {
        assert!(Art::parse(bad).is_err());
    }
    assert!(Art::parse(&vec![b'a'; 16385]).is_err());
    assert!(Art::parse(&[b'\n'; 18]).is_err());
}
#[test]
fn strict_theme_and_malicious_labels_fail() {
    let p = Pack::new(BASE);
    assert_eq!(Theme::load(&p.0).unwrap().board(), "Test board");
    for bad in [
        "version=2\nid='bad'",
        "version=1\nid='bad'\nscript='execute'",
        "version=1\nid='bad'\nnormal={fg=16,bg=0}",
        "version=1\nid='bad'\nnormal={fg=0,bg=0}",
        "version=1\nid='bad'\nmenu=[]",
    ] {
        p.source(bad);
        assert!(Theme::load(&p.0).is_err());
    }
    p.source("version=1\nid='bad'\nboard=\"\\u001b]52\"");
    assert!(Theme::load(&p.0).is_err());
}
#[test]
fn asset_paths_provenance_and_symlinks_fail_atomically() {
    let p = Pack::new(BASE);
    let good = Theme::load(&p.0).unwrap();
    p.source(&(BASE.to_owned()+"identity_ansi='../escape.ans'\ncreator='test'\nsource='test fixture'\npermission='test only'\n"));
    assert!(Theme::load(&p.0).is_err());
    p.source(&(BASE.to_owned() + "identity_ansi='screen.ans'\n"));
    fs::write(p.0.join("screen.ans"), b"A").unwrap();
    assert!(Theme::load(&p.0).is_err());
    p.source(&(BASE.to_owned()+"identity_ansi='screen.ans'\ncreator='test'\nsource='test fixture'\npermission='test only'\n"));
    assert!(Theme::load(&p.0).is_ok());
    fs::remove_file(p.0.join("screen.ans")).unwrap();
    symlink("theme.toml", p.0.join("screen.ans")).unwrap();
    assert!(good.with_season(&p.0).is_err());
    assert_eq!(good.board(), "Test board");
    fs::remove_file(p.0.join("screen.ans")).unwrap();
    fs::write(p.0.join("screen.ans"), b"\x1b]52;c;unsafe\x07").unwrap();
    assert!(Theme::load(&p.0).is_err());
    assert_eq!(good.board(), "Test board");
    fs::set_permissions(p.0.join("theme.toml"), fs::Permissions::from_mode(0o666)).unwrap();
    assert!(Theme::load(&p.0).is_err());
}
#[test]
fn seasonal_overrides_cannot_remap_commands() {
    let base = Theme::default();
    let first =
        Pack::new("version=1\nid='autumn'\nnormal={fg=3,bg=0}\nwelcome='Autumn on the board'\n");
    let second = Pack::new("version=1\nid='winter'\nnormal={fg=6,bg=0}\n");
    let a = base.with_season(&first.0).unwrap();
    let b = base.with_season(&second.0).unwrap();
    assert_eq!(a.id(), "autumn");
    assert_eq!(b.id(), "winter");
    let commands = |t: &Theme| {
        t.entries()
            .iter()
            .map(|e| (e.action(), e.key(), e.label().to_owned()))
            .collect::<Vec<_>>()
    };
    assert_eq!(commands(&base), commands(&a));
    assert_eq!(commands(&base), commands(&b));
    assert_ne!(Menu::new(&a, true).render(), Menu::new(&b, true).render());
    first.source("version=1\nid='bad'\nmenu=[]");
    assert!(base.with_season(&first.0).is_err());
    assert_eq!(base.id(), "builtin-text");
}
#[test]
fn custom_layout_collision_and_target_validation() {
    let mut source = BASE.to_owned();
    for (action, key) in [
        ("quit", "X"),
        ("sysop", "O"),
        ("post", "W"),
        ("history", "H"),
        ("new", "N"),
        ("rooms", "R"),
    ] {
        source += &format!("[[menu]]\naction='{action}'\nkey='{key}'\nlabel='{action}'\n");
    }
    let p = Pack::new(&source);
    let t = Theme::load(&p.0).unwrap();
    let mut menu = Menu::new(&t, false);
    assert_eq!(menu.feed(Key::Hotkey(b'W')), Some(Action::Post));
    assert_eq!(menu.feed(Key::Enter), Some(Action::Quit));
    for bad in [
        source.replace("key='X'", "key='R'"),
        source.replace("action='post'", "action='recover'"),
        source.replace("action='post'", "action='rooms'"),
    ] {
        p.source(&bad);
        assert!(Theme::load(&p.0).is_err());
    }
}

#[test]
fn utf8_continuation_bytes_do_not_enter_c1_control_modes() {
    assert_eq!(decode("Ðé中P".as_bytes()), vec![Key::Hotkey(b'P')]);
    assert_eq!(decode(b"\xc3P\r\nQ"), vec![Key::Hotkey(b'Q')]);
}

#[test]
fn checked_in_base_and_season_examples_validate() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
    let base = Theme::load(&root.join("base")).unwrap();
    for name in ["autumn", "winter"] {
        let seasonal = base.with_season(&root.join("seasons").join(name)).unwrap();
        assert_eq!(
            seasonal
                .entries()
                .iter()
                .map(|e| (e.action(), e.key()))
                .collect::<Vec<_>>(),
            base.entries()
                .iter()
                .map(|e| (e.action(), e.key()))
                .collect::<Vec<_>>()
        );
    }
}
