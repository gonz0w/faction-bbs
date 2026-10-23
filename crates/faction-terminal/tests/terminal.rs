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
            Key::Right,
            Key::Left,
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
    assert!(Theme::load(&p.0).is_err());
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

#[test]
fn chat_editor_is_a_separate_bounded_context() {
    use faction_terminal::{LINE_BYTES, LineEditor, LineEvent};
    let mut editor = LineEditor::default();
    let now = Instant::now();
    for b in b"P is text\x1b[D!" {
        editor.feed(*b, now);
    }
    assert_eq!(editor.text(), "P is text!");
    assert_eq!(editor.viewport(5), "text!");
    assert_eq!(
        editor.feed(b'\r', now),
        Some(LineEvent::Submit("P is text!".into()))
    );
    assert_eq!(editor.feed(b'\n', now), None);
    for _ in 0..=LINE_BYTES {
        editor.feed(b'P', now);
    }
    assert_eq!(editor.feed(b'\r', now), Some(LineEvent::Discarded));
    assert!(editor.text().is_empty());
    assert_eq!(editor.feed(b'\n', now), None);
    for b in b"/me\x1b]52;c;/quit\n\x1b\\ waves" {
        editor.feed(*b, now);
    }
    assert_eq!(
        editor.feed(b'\n', now),
        Some(LineEvent::Submit("/me waves".into()))
    );
    editor.feed(27, now);
    assert_eq!(
        editor.expire(now + Duration::from_secs(1)),
        Some(LineEvent::Discarded)
    );
    editor.feed(b'P', now + Duration::from_secs(1));
    assert!(editor.text().is_empty());
    assert_eq!(editor.feed(3, now), Some(LineEvent::Cancel));
}

#[test]
fn only_runtime_capability_enables_door_arrows_and_hotkeys() {
    let mut theme = Theme::default();
    assert_eq!(theme.hotkey(b'D', false), None);
    theme.set_doors_enabled(true);
    assert_eq!(theme.hotkey(b'd', false), Some(Action::Doors));
    let mut menu = Menu::new(&theme, false);
    menu.feed(Key::Up); // Quit
    menu.feed(Key::Up); // Terminal settings
    menu.feed(Key::Up); // Doors, enabled by host
    assert_eq!(menu.feed(Key::Enter), Some(Action::Doors));
    theme.set_doors_enabled(false);
    assert_eq!(theme.hotkey(b'D', false), None);
}

#[test]
fn optional_files_preserve_default_theme_and_use_arrows_or_hotkeys() {
    let mut theme = Theme::default();
    assert_eq!(theme.entries().len(), 9);
    assert_eq!(theme.hotkey(b'f', false), None);
    theme.set_files_enabled(true).unwrap();
    assert_eq!(theme.entries().len(), 10);
    assert_eq!(theme.hotkey(b'f', false), Some(Action::Files));
    let mut menu = Menu::new(&theme, false);
    menu.feed(Key::Up); // Quit
    menu.feed(Key::Up); // Files before quit
    assert_eq!(menu.feed(Key::Enter), Some(Action::Files));
    theme.set_files_enabled(false).unwrap();
    assert_eq!(theme.hotkey(b'f', false), None);
    let mut menu = Menu::new(&theme, false);
    menu.feed(Key::Up);
    menu.feed(Key::Up); // Terminal settings, skipping unavailable Files
    menu.feed(Key::Up); // Skip unavailable Doors and Sysop.
    assert_eq!(menu.feed(Key::Enter), Some(Action::Chat));
}

// Small independent cell oracle for this renderer's declared output grammar.
// Reject wrap, scrolling and any terminal controls beyond positioning/SGR/clear.
fn paint_menu(frame: &str, screen: &mut [Vec<u8>]) {
    let (mut row, mut col, mut i) = (0usize, 0usize, 0usize);
    let bytes = frame.as_bytes();
    while i < bytes.len() {
        if bytes[i] == 27 {
            assert_eq!(bytes[i + 1], b'[');
            let start = i + 2;
            i = start;
            while bytes[i].is_ascii_digit() || bytes[i] == b';' {
                i += 1;
            }
            let params = &frame[start..i];
            match bytes[i] {
                b'H' => {
                    let values = if params.is_empty() {
                        vec![1, 1]
                    } else {
                        params
                            .split(';')
                            .map(|n| n.parse::<usize>().unwrap())
                            .collect()
                    };
                    assert_eq!(values.len(), 2);
                    (row, col) = (values[0] - 1, values[1] - 1);
                    assert!(row < screen.len() && col < screen[0].len());
                }
                b'J' => {
                    assert_eq!(params, "2");
                    for line in screen.iter_mut() {
                        line.fill(b' ');
                    }
                }
                b'm' => {
                    assert!(!params.is_empty());
                }
                _ => panic!("unexpected menu terminal control"),
            }
        } else {
            assert!(bytes[i].is_ascii_graphic() || bytes[i] == b' ');
            assert!(
                row < screen.len() && col < screen[0].len(),
                "menu wrapped or scrolled"
            );
            screen[row][col] = bytes[i];
            col += 1;
        }
        i += 1;
    }
}
#[test]
fn compact_wide_and_maximal_menus_fit_and_delta_matches_full_cells() {
    let mut source = format!(
        "{BASE}menu_title='{}'\nwelcome='{}'\n",
        "M".repeat(24),
        "W".repeat(36)
    );
    for (action, key) in [
        ("rooms", "R"),
        ("new", "N"),
        ("history", "H"),
        ("post", "P"),
        ("chat", "C"),
        ("sysop", "S"),
        ("doors", "D"),
        ("files", "F"),
        ("terminal", "T"),
        ("quit", "Q"),
    ] {
        source += &format!(
            "[[menu]]\naction='{action}'\nkey='{key}'\nlabel='{}'\ndescription='{}'\n",
            "L".repeat(26),
            "d".repeat(30)
        );
    }
    let pack = Pack::new(&source);
    let mut theme = Theme::load(&pack.0).unwrap();
    theme.set_files_enabled(true).unwrap();
    theme.set_doors_enabled(true);
    for columns in [40, 79, 80, 132, 240] {
        let mut menu = Menu::with_columns(&theme, false, columns);
        let frame = menu.render();
        assert!(frame.len() < 4096);
        assert_eq!(frame.contains(&"d".repeat(30)), columns >= 80);
        let mut actual = vec![vec![b' '; usize::from(columns)]; 18];
        paint_menu(&frame, &mut actual);
        for _ in 0..theme.entries().len() * 2 {
            let previous = menu.focus();
            assert_eq!(menu.feed(Key::Down), None);
            let delta = menu.redraw(previous);
            assert!(delta.len() <= 512 && !delta.contains("\x1b[2J"));
            paint_menu(&delta, &mut actual);
            let mut expected = vec![vec![b' '; usize::from(columns)]; 18];
            paint_menu(&menu.render(), &mut expected);
            assert_eq!(actual, expected, "stale focus at {columns} columns");
        }
        assert_eq!(menu.feed(Key::Hotkey(b's')), None);
        println!(
            "columns={columns}: full={} bytes, focus={} bytes",
            frame.len(),
            menu.redraw(0).len()
        );
    }
}
#[test]
fn descriptions_and_menu_titles_are_inert_atomic_and_not_seasonal() {
    let good = format!("{BASE}menu_title='NIGHT / MAIN'\n");
    let p = Pack::new(&good);
    let base = Theme::load(&p.0).unwrap();
    assert_eq!(base.menu_title(), "NIGHT / MAIN");
    assert!(!base.plain_menu(false).contains('\x1b'));
    let season = Pack::new("version=1\nid='season'\nmenu_title='REMAPPED'\n");
    assert!(base.with_season(&season.0).is_err());
    for value in ["'ABCDEFGHIJKLMNOPQRSTUVWXYZ'", "\"\\u001b[2J\"", "''", "17"] {
        p.source(&format!("{BASE}menu_title={value}\n"));
        assert!(Theme::load(&p.0).is_err());
        assert_eq!(base.menu_title(), "NIGHT / MAIN");
    }
    let shipped =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes/base/theme.toml");
    let source = fs::read_to_string(shipped).unwrap();
    for value in [
        "'ABCDEFGHIJKLMNOPQRSTUVWXYZ12345'",
        "\"\\u001b]52;c;bad\"",
        "''",
        "false",
    ] {
        let bad = source.replace(
            "description = \"Find a room; follow the talk\"",
            &format!("description = {value}"),
        );
        assert_ne!(bad, source);
        p.source(&bad);
        assert!(Theme::load(&p.0).is_err());
    }
}
#[test]
fn focused_hotkey_keeps_full_lightbar_background_and_unavailable_is_explicit() {
    let p = Pack::new(&format!(
        "{BASE}focus={{fg=15,bg=5,bold=true}}\nhotkey={{fg=14,bg=0}}\n"
    ));
    let theme = Theme::load(&p.0).unwrap();
    let menu = Menu::with_columns(&theme, false, 80);
    let frame = menu.render();
    let row = frame
        .split("\x1b[5;3H")
        .nth(1)
        .unwrap()
        .split("\x1b[0m")
        .next()
        .unwrap();
    assert!(row.contains(";45m"));
    assert!(!row.contains(";40m"), "hotkey split the focus background");
    assert!(frame.contains("(unavailable)"));
    let compact = Menu::new(&theme, false).render();
    assert!(compact.contains("Sysop controls             --"));
}

fn grid_pack() -> Pack {
    let p = Pack::new(include_str!("../../../themes/grid/theme.toml"));
    fs::write(
        p.0.join("login.txt"),
        include_bytes!("../../../themes/grid/login.txt"),
    )
    .unwrap();
    fs::write(
        p.0.join("login.ans"),
        include_bytes!("../../../themes/grid/login.ans"),
    )
    .unwrap();
    p
}
#[test]
fn grid_arrows_are_spatial_tab_and_hotkeys_keep_declared_order() {
    let p = grid_pack();
    let t = Theme::load(&p.0).unwrap();
    let mut m = Menu::with_columns(&t, false, 80);
    assert_eq!(m.feed(Key::Right), None); // Sysop at row 1 unavailable: Terminal is nearest.
    assert_eq!(m.feed(Key::Enter), Some(Action::Terminal));
    m.feed(Key::Left);
    assert_eq!(m.feed(Key::Enter), Some(Action::History)); // same row in left column
    m.feed(Key::Next);
    assert_eq!(m.feed(Key::Enter), Some(Action::Post)); // array order, not spatial order
    assert_eq!(m.feed(Key::Hotkey(b's')), None);
    assert_eq!(m.feed(Key::Hotkey(b'f')), None);
    assert_eq!(m.feed(Key::Hotkey(b'd')), None);
    assert_eq!(m.feed(Key::Hotkey(b'r')), Some(Action::Rooms));
    m.restore_focus(0);
    m.feed(Key::Up);
    assert_eq!(m.feed(Key::Enter), Some(Action::Chat)); // wraps in its column
    let mut admin = Menu::with_columns(&t, true, 80);
    admin.feed(Key::Right);
    assert_eq!(admin.feed(Key::Enter), Some(Action::Sysop));
    assert_eq!(
        decode(b"\x1b[C\x1bOD\t"),
        vec![Key::Right, Key::Left, Key::Next]
    );
}
#[test]
fn grid_focus_restores_by_index_and_classic_layout_keeps_every_action() {
    let p = grid_pack();
    let mut t = Theme::load(&p.0).unwrap();
    t.set_files_enabled(true).unwrap();
    t.set_doors_enabled(true);
    for columns in [40, 79, 80, 132, 240] {
        let mut m = Menu::with_columns(&t, true, columns);
        for i in 0..t.entries().len() {
            m.restore_focus(i);
            assert_eq!(m.focus(), i);
            assert_eq!(m.feed(Key::Enter), Some(t.entries()[i].action()));
            let full = m.render();
            assert!(full.len() < 4096);
            let mut screen = vec![vec![b' '; usize::from(columns)]; 18];
            paint_menu(&full, &mut screen);
            assert_eq!(screen.iter().flatten().filter(|c| **c == b'>').count(), 1);
            let before = m.focus();
            m.feed(Key::Next);
            assert_eq!(m.focus(), (i + 1) % t.entries().len());
            let delta = m.redraw(before);
            assert!(delta.len() <= 512, "{columns}: {}", delta.len());
            assert!(!delta.contains("\x1b[2J"));
            paint_menu(&delta, &mut screen);
            let mut expected = vec![vec![b' '; usize::from(columns)]; 18];
            paint_menu(&m.render(), &mut expected);
            assert_eq!(screen, expected, "incremental grid must match full paint");
        }
    }
    let plain = t.plain_menu(false);
    assert!(!plain.contains('\x1b'));
    for e in t.entries() {
        assert!(plain.contains(e.label()));
    }
    let mut compact = Menu::with_columns(&t, false, 40);
    compact.feed(Key::Right);
    assert_eq!(compact.feed(Key::Enter), Some(Action::New));
    compact.feed(Key::Left);
    assert_eq!(compact.feed(Key::Enter), Some(Action::Rooms));
}
#[test]
fn grid_validation_is_atomic_and_seasons_cannot_move_actions() {
    let p = grid_pack();
    let source = fs::read_to_string(p.0.join("theme.toml")).unwrap();
    for bad in [
        source.replacen("row=1", "row=0", 1),
        source.replacen("column=1", "column=3", 1),
        source.replacen("row=2", "row=1", 1),
        source.replacen("row=1\n", "", 1),
        source.replace("layout='grid'", "layout='list'"),
    ] {
        p.source(&bad);
        assert!(Theme::load(&p.0).is_err());
    }
    p.source(&source);
    let t = Theme::load(&p.0).unwrap();
    let season = Pack::new("version=1\nid='bad'\nlayout='list'\n");
    assert!(t.with_season(&season.0).is_err());
    let mut m = Menu::with_columns(&t, false, 80);
    m.restore_focus(5); // unavailable Sysop cannot be restored
    assert_eq!(m.focus(), 0);
    m.restore_focus(usize::MAX);
    assert_eq!(m.focus(), 0);
    // An omitted Files action is assigned the first vacant cell when enabled.
    let without_files = source.replace("\n[[menu]]\naction='files'\nkey='F'\nlabel='File areas'\ndescription='Browse local uploads and files'\nrow=4\ncolumn=2\n", "");
    assert_ne!(without_files, source);
    p.source(&without_files);
    let mut t = Theme::load(&p.0).unwrap();
    t.set_files_enabled(true).unwrap();
    let mut m = Menu::with_columns(&t, true, 80);
    m.restore_focus(
        t.entries()
            .iter()
            .position(|e| e.action() == Action::Files)
            .unwrap(),
    );
    assert_eq!(m.feed(Key::Enter), Some(Action::Files));
}
fn sauce(body: &[u8], flags: u8, comments: Option<&[u8]>) -> Vec<u8> {
    let mut out = body.to_vec();
    out.push(26);
    if let Some(text) = comments {
        out.extend(b"COMNT");
        out.extend(text.iter().copied().chain(std::iter::repeat(b' ')).take(64));
    }
    let mut record = [0; 128];
    record[..7].copy_from_slice(b"SAUCE00");
    record[94] = 1;
    record[95] = 1;
    record[96] = 80;
    record[98] = 16;
    record[104] = u8::from(comments.is_some());
    record[105] = flags;
    out.extend(record);
    out
}
#[test]
fn sauce_ice_bright_backgrounds_are_canonical_and_metadata_is_inert() {
    let bytes = sauce(
        b"\x1b[5;44mA\x1b[41mB\x1b[25mC\x1b[0m\xdb",
        1,
        Some(b"\x1b]52;c;not-rendered\x07"),
    );
    let rendered = Art::parse(&bytes).unwrap().render();
    assert!(rendered.contains(";104mA"));
    assert!(rendered.contains(";101mB"));
    assert!(rendered.contains(";41mC"));
    assert!(rendered.contains('█'));
    assert!(!rendered.contains("not-rendered"));
    assert!(!rendered.contains("SAUCE"));
    assert!(!rendered.contains("[5m"));
    assert!(Art::parse(&sauce(b"\x1b[5mX", 0, None)).is_err());
    assert!(Art::parse(&sauce(b"\x1b]52;c;bad\x07", 1, None)).is_err());
    // Empty SGR is a full reset, including the imported iCE intensity state.
    let reset = Art::parse(&sauce(b"\x1b[5;44mA\x1b[m\x1b[41mB", 1, None))
        .unwrap()
        .render();
    assert!(reset.contains(";41mB"));
    assert!(!reset.contains(";101mB"));
    let p = grid_pack();
    let t = Theme::load(&p.0).unwrap();
    assert!(t.identity_screen(80, 24, true).contains('╓'));
    assert!(!t.identity_screen(40, 18, true).contains('╓'));
    assert!(!t.identity_screen(80, 24, false).contains('\x1b'));
}
#[test]
fn unsupported_or_truncated_sauce_never_changes_the_canvas_or_output_budget() {
    let valid = sauce(b"OK", 1, None);
    for (offset, value) in [
        (5, b'1'),
        (94, 5),
        (95, 2),
        (96, 132),
        (98, 17),
        (104, 1),
        (105, 128),
    ] {
        let mut bad = valid.clone();
        let at = bad.len() - 128;
        bad[at + offset] = value;
        assert!(Art::parse(&bad).is_err(), "offset {offset}");
    }
    let mut bad = valid.clone();
    bad.remove(2);
    assert!(Art::parse(&bad).is_err());
    assert!(Art::parse(&valid[..valid.len() - 1]).is_err());
    assert!(Art::parse(&sauce(b"\x1b[17;1Hbad", 1, None)).is_err());
    assert!(Art::parse(&sauce(&vec![b'X'; 16384], 1, None)).is_err());
}

#[test]
fn optional_directory_and_files_keep_full_grid_usable_in_both_enable_orders() {
    for directory_first in [false, true] {
        let p = grid_pack();
        let mut theme = Theme::load(&p.0).unwrap();
        if directory_first {
            theme.set_directory_enabled(true).unwrap();
        }
        theme.set_files_enabled(true).unwrap();
        theme.set_directory_enabled(true).unwrap();
        assert_eq!(theme.entries().len(), 11);
        assert_eq!(theme.hotkey(b'o', false), Some(Action::Contacts));
        for width in [40, 80] {
            let mut menu = Menu::with_columns(&theme, false, width);
            assert_eq!(menu.feed(Key::Hotkey(b'O')), Some(Action::Contacts));
            let frame = menu.render();
            assert!(frame.len() < 8192);
            assert!(frame.contains("Other BBSes"));
        }
    }
}
