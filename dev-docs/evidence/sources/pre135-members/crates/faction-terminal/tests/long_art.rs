use faction_terminal::{Art, Theme};
use std::{
    fs,
    os::unix::fs::DirBuilderExt,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
struct Pack(PathBuf);
impl Pack {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "faction-long-art-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        fs::write(
            p.join("theme.toml"),
            include_bytes!("../../../themes/longform/theme.toml"),
        )
        .unwrap();
        fs::write(
            p.join("login.txt"),
            include_bytes!("../../../themes/longform/login.txt"),
        )
        .unwrap();
        fs::write(
            p.join("login.ans"),
            include_bytes!("../../../themes/longform/login.ans"),
        )
        .unwrap();
        Self(p)
    }
}
impl Drop for Pack {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn long_art_pages_are_local_bounded_and_keep_inherited_styles() {
    let mut bytes = b"\x1b[36m".to_vec();
    for row in 1..=206 {
        bytes.extend(format!("ROW {row:03}\r\n").bytes());
    }
    assert!(Art::parse(&bytes).is_err());
    let art = Art::parse_long(&bytes).unwrap();
    assert_eq!(art.page_count(), 13);
    assert_eq!(art.render(), art.render_page(0).unwrap());
    for page in 0..13 {
        let out = art.render_page(page).unwrap();
        assert!(out.len() <= 16384);
        assert!(out.contains("\x1b[1;1H"));
        assert!(out.contains(";36;40m")); // style inherited across the page boundary
        assert!(out.contains(&format!("ROW {:03}", page * 16 + 1)));
        assert!(!out.contains(&format!("ROW {:03}", (page + 1) * 16 + 1)));
        assert!(!out.contains("\x1b[17;"));
    }
    assert!(art.render_page(13).is_none());
    assert!(art.render_page(usize::MAX).is_none());
}
#[test]
fn page_validation_rejects_later_controls_dimensions_and_output_amplification() {
    for suffix in [
        b"\x1b]52;c;secret\x07".as_slice(),
        b"\x1bPbad\x1b\\",
        b"\x1b[257;1Hbad",
        b"\x1b[5mblink",
    ] {
        let mut data = b"FIRST\x1b[17;1HSECOND".to_vec();
        data.extend(suffix);
        assert!(Art::parse_long(&data).is_err());
    }
    let max = Art::parse_long(b"\x1b[256;80HZ").unwrap();
    assert_eq!(max.page_count(), 16);
    assert!(max.render_page(15).unwrap().contains('Z'));
    assert!(Art::parse_long(&vec![b' '; 65537]).is_err());
    // Small input can expand into a page of alternating per-cell style changes.
    // Validate every page at load, not just the harmless first page.
    let mut data = b"FIRST\x1b[17;1H".to_vec();
    for i in 0..1280 {
        data.extend(if i % 2 == 0 {
            b"\x1b[31mX"
        } else {
            b"\x1b[32mX"
        });
    }
    assert!(data.len() < 65536);
    assert!(Art::parse_long(&data).is_err());
}
#[test]
fn long_identity_is_explicit_and_short_classic_behavior_is_preserved() {
    let p = Pack::new();
    let source = fs::read_to_string(p.0.join("theme.toml")).unwrap();
    let theme = Theme::load(&p.0).unwrap();
    assert_eq!(theme.identity_page_count(80, 24, true), 3);
    for (cols, rows, ansi) in [
        (40, 18, true),
        (79, 24, true),
        (80, 23, true),
        (80, 24, false),
    ] {
        assert_eq!(theme.identity_page_count(cols, rows, ansi), 1);
        assert!(theme.identity_page_screen(cols, rows, ansi, 1).is_none());
        assert!(theme.identity_page_prompt(0, cols, rows, ansi).is_none());
        assert!(
            theme
                .identity_screen(cols, rows, ansi)
                .contains("PRIVATE FREQUENCIES")
        );
    }
    assert!(
        theme
            .identity_page_screen(80, 24, true, 1)
            .unwrap()
            .contains("02 / THROUGH THE RELAY")
    );
    assert!(
        !theme
            .identity_page_screen(80, 24, true, 1)
            .unwrap()
            .contains("01 / INCOMING SIGNALS")
    );
    assert!(theme.identity_page_prompt(2, 80, 24, true).is_none());
    assert!(
        theme
            .identity_page_prompt(usize::MAX, 80, 24, true)
            .is_none()
    );
    assert!(!theme.identity_fallback_screen(80, 24, true).contains('╓'));
    fs::write(
        p.0.join("theme.toml"),
        source.replace("identity_ansi_long=", "identity_ansi="),
    )
    .unwrap();
    assert!(Theme::load(&p.0).is_err());
    fs::write(
        p.0.join("theme.toml"),
        source.replace(
            "identity_ansi_long='login.ans'",
            "identity_ansi_long='login.ans'\nidentity_ansi='login.ans'",
        ),
    )
    .unwrap();
    assert!(Theme::load(&p.0).is_err());
}
#[test]
fn season_can_replace_long_identity_without_moving_grid_actions() {
    let p = Pack::new();
    let base = Theme::load(&p.0).unwrap();
    let source = "version=1\nid='long-season'\nidentity_ansi_long='login.ans'\ncreator='Faction test'\nsource='Original fixture'\npermission='BSD-3-Clause'\n";
    fs::write(p.0.join("theme.toml"), source).unwrap();
    let season = base.with_season(&p.0).unwrap();
    assert_eq!(season.identity_page_count(80, 24, true), 3);
    assert_eq!(season.plain_menu(false), base.plain_menu(false));
    let mut bytes = fs::read(p.0.join("login.ans")).unwrap();
    let at = bytes.len() - 128;
    bytes[at + 98..at + 100].copy_from_slice(&257u16.to_le_bytes());
    fs::write(p.0.join("login.ans"), bytes).unwrap();
    assert!(base.with_season(&p.0).is_err());
    assert_eq!(base.identity_page_count(80, 24, true), 3);
}
