use super::*;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
};
const PASS: &str = "synthetic-gallery-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "faction-gallery-{}",
            faction_core::request_id().unwrap()
        ));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        for n in [
            "gallery.toml",
            "nightwire.ans",
            "nightwire.txt",
            "autumn.ans",
            "autumn.txt",
            "winter.ans",
            "winter.txt",
            "signal.txt",
        ] {
            fs::copy(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../gallery/nightwire")
                    .join(n),
                p.join(n),
            )
            .unwrap();
        }
        Self(p)
    }
    fn change(&self, a: &str, b: &str) {
        let p = self.0.join("gallery.toml");
        let s = fs::read_to_string(&p).unwrap();
        assert!(s.contains(a));
        fs::write(p, s.replacen(a, b, 1)).unwrap();
    }
    fn load(&self) -> catalog::Result<Catalog> {
        Catalog::load(&self.0)
    }
    fn board(&self) -> (Board, Session) {
        let mut b = Board::open(&self.0.join("board")).unwrap();
        b.bootstrap("sysop", PASS).unwrap();
        let s = b.authenticate("sysop", PASS).unwrap();
        (b, s)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn originals_parse_to_bounded_canonical_pages() {
    let f = Fixture::new();
    let c = f.load().unwrap();
    assert_eq!(c.len(), 4);
    assert!(c.enabled());
    assert!(c.preview("nightwire", 0).unwrap().contains("\x1b["));
    assert!(c.preview("nightwire", 3).is_err());
    assert!(c.preview("unknown", 0).is_err());
    for w in &c.works {
        for p in 0..w.art.page_count() {
            assert!(w.art.render_page(p).unwrap().len() <= ART_FRAME_BYTES);
        }
    }
}
#[test]
fn explicit_rights_provenance_and_known_metadata_are_mandatory() {
    for (a, b) in [
        ("source = \"original\"", "source = \"archive\""),
        ("display_permission = true", "display_permission = false"),
        ("artist = \"Faction contributors\"", "artist = \"\""),
        (
            "license = \"BSD-3-Clause; see LICENSE\"",
            "licence = \"unknown\"",
        ),
        ("id = \"autumn\"", "id = \"nightwire\""),
        (
            "provenance = \"Original Faction repository artwork: themes/nightwire.\"",
            "provenance = \"\"",
        ),
        (
            "formats = [\"ansi\", \"ascii\"]",
            "formats = [\"ansi\", \"ansi\"]",
        ),
    ] {
        let f = Fixture::new();
        f.change(a, b);
        assert!(f.load().is_err(), "{a}");
    }
}
#[test]
fn unsafe_paths_links_permissions_and_file_sizes_are_rejected() {
    for name in ["../nightwire.ans", "/tmp/art.ans", "sub/art.ans"] {
        let f = Fixture::new();
        f.change("asset = \"nightwire.ans\"", &format!("asset = \"{name}\""));
        assert!(f.load().is_err());
    }
    let f = Fixture::new();
    let p = f.0.join("nightwire.ans");
    fs::rename(&p, f.0.join("other.ans")).unwrap();
    symlink("other.ans", &p).unwrap();
    assert!(f.load().is_err());
    fs::remove_file(&p).unwrap();
    fs::hard_link(f.0.join("other.ans"), &p).unwrap();
    assert!(f.load().is_err());
    let f = Fixture::new();
    fs::set_permissions(f.0.join("nightwire.txt"), fs::Permissions::from_mode(0o666)).unwrap();
    assert!(f.load().is_err());
    let f = Fixture::new();
    fs::write(f.0.join("nightwire.ans"), vec![b' '; 65537]).unwrap();
    assert!(f.load().is_err());
    let f = Fixture::new();
    fs::write(f.0.join("gallery.toml"), vec![b' '; CATALOG_BYTES + 1]).unwrap();
    assert!(f.load().is_err());
    let f = Fixture::new();
    fs::set_permissions(&f.0, fs::Permissions::from_mode(0o777)).unwrap();
    assert!(f.load().is_err());
}
#[test]
fn active_controls_and_ascii_escape_sequences_never_enter_cells() {
    for bytes in [
        b"\x1b]52;c;secret\x07".as_slice(),
        b"\x1bPremote\x1b\\",
        b"\x1b_Gquery\x1b\\",
        b"\x1b[6n",
        b"bell\x07",
    ] {
        let f = Fixture::new();
        fs::write(f.0.join("nightwire.ans"), bytes).unwrap();
        assert!(f.load().is_err());
    }
    let f = Fixture::new();
    fs::write(f.0.join("signal.txt"), b"\x1b[31mraw").unwrap();
    assert!(f.load().is_err());
}
#[test]
fn text_fallback_budgets_and_aggregate_page_budget_are_enforced() {
    for text in [
        "a".repeat(73),
        "line\n".repeat(129),
        "x".repeat(8193),
        "bad\tline".into(),
        "é".into(),
        "".into(),
    ] {
        let f = Fixture::new();
        fs::write(f.0.join("nightwire.txt"), text).unwrap();
        assert!(f.load().is_err());
    }
    let f = Fixture::new();
    for n in ["nightwire.ans", "autumn.ans", "winter.ans"] {
        fs::write(f.0.join(n), "x\r\n".repeat(512)).unwrap();
    }
    assert!(f.load().is_err());
}
#[test]
fn disabled_or_filtered_formats_hide_assets_without_reading_them() {
    let f = Fixture::new();
    f.change("enabled = true", "enabled = false");
    fs::remove_file(f.0.join("nightwire.ans")).unwrap();
    assert!(!f.load().unwrap().enabled());
    let f = Fixture::new();
    f.change("formats = [\"ansi\", \"ascii\"]", "formats = [\"ascii\"]");
    fs::remove_file(f.0.join("nightwire.ans")).unwrap();
    let c = f.load().unwrap();
    assert_eq!(c.len(), 1);
    assert!(c.preview("nightwire", 0).is_err());
}
#[test]
fn download_reference_requires_separate_permission_and_canonical_local_id() {
    for extra in [
        "download_ref = \"https://example.invalid/art\"",
        "download_ref = \"faction-file:bad/id\"",
        "download_ref = \"faction-file:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"",
    ] {
        let f = Fixture::new();
        f.change(
            "download_permission = false",
            &format!("download_permission = false\n{extra}"),
        );
        assert!(f.load().is_err());
    }
    let f = Fixture::new();
    f.change("download_permission = false", "download_permission = true");
    assert!(f.load().is_err());
}
#[test]
fn workflow_paging_resizes_plaintext_and_current_approval_are_bounded() {
    let f = Fixture::new();
    let c = f.load().unwrap();
    let (mut b, s) = f.board();
    let t = Theme::default();
    let mut d = Desk::default();
    for cmd in ["e", "seasons", "v", "autumn", "a"] {
        d.render(&mut b, &s, &c, &t, true, (80, 24)).unwrap();
        assert!(!d.feed(cmd, &mut b, &s, &c).unwrap());
    }
    for cmd in ["n", "n", "n", "n", "p", "t", "n", "i"] {
        let frame = d.render(&mut b, &s, &c, &t, true, (80, 24)).unwrap();
        assert!(frame.len() <= 8192);
        d.feed(cmd, &mut b, &s, &c).unwrap();
    }
    let plain = d.render(&mut b, &s, &c, &t, false, (40, 18)).unwrap();
    assert!(!plain.contains('\x1b'));
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
    b.register("pending", PASS).unwrap();
    let p = b.authenticate("pending", PASS).unwrap();
    assert!(matches!(
        d.render(&mut b, &p, &c, &t, false, (80, 24)),
        Err(Error::Denied)
    ));
    assert!(d.feed("\x1b[6n", &mut b, &s, &c).is_err());
    assert!(d.feed(&"x".repeat(65), &mut b, &s, &c).is_err());
    b.set_status(&s, "pending", Status::Approved).unwrap();
    let p = b.authenticate("pending", PASS).unwrap();
    b.set_status(&s, "pending", Status::Revoked).unwrap();
    assert!(d.feed("q", &mut b, &p, &c).is_err());
}
#[test]
fn live_download_resolution_rechecks_grants_and_namespace() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    b.register("reader", PASS).unwrap();
    b.set_status(&s, "reader", Status::Approved).unwrap();
    let reader = b.authenticate("reader", PASS).unwrap();
    let db = rusqlite::Connection::open(f.0.join("board/faction.sqlite3")).unwrap();
    let ns: String = db
        .query_row("select id from file_namespace", [], |r| r.get(0))
        .unwrap();
    let reference = format!("faction-file:{ns}/{}", "c".repeat(32));
    db.execute_batch("INSERT INTO file_areas(slug,title,private,quota) VALUES('private','Private art',1,10000); INSERT INTO file_objects VALUES('cccccccccccccccccccccccccccccccc',1,1,'original.ans','Original licensed display',1,'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa','published');").unwrap();
    f.change(
        "download_permission = false",
        &format!("download_permission = true\ndownload_ref = \"{reference}\""),
    );
    let c = f.load().unwrap();
    let mut d = Desk::default();
    for v in ["v", "nightwire", "l"] {
        d.feed(v, &mut b, &reader, &c).unwrap();
    }
    let mut show = |b: &mut Board| {
        d.render(b, &reader, &c, &Theme::default(), false, (80, 24))
            .unwrap()
    };
    assert!(!show(&mut b).contains(&reference));
    db.execute("INSERT INTO file_area_grants(area_id,member_id) SELECT 1,id FROM members WHERE handle='reader'",[]).unwrap();
    assert!(show(&mut b).contains(&reference));
    db.execute("DELETE FROM file_area_grants", []).unwrap();
    let out = show(&mut b);
    assert!(!out.contains("original.ans") && !out.contains(&reference));
}
