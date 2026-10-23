use super::*;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
};
const PASS: &str = "synthetic-library-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "faction-library-{}",
            faction_core::request_id().unwrap()
        ));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        Self(p)
    }
    fn board(&self) -> (Board, Session, rusqlite::Connection) {
        let mut b = Board::open(&self.0.join("board")).unwrap();
        b.bootstrap("sysop", PASS).unwrap();
        let s = b.authenticate("sysop", PASS).unwrap();
        b.register("reader", PASS).unwrap();
        b.set_status(&s, "reader", Status::Approved).unwrap();
        let db = rusqlite::Connection::open(self.0.join("board/faction.sqlite3")).unwrap();
        db.execute_batch("INSERT INTO file_areas VALUES(1,'public','Public manuals',0,10000); INSERT INTO file_areas VALUES(2,'private','Private manuals',1,10000);").unwrap();
        for (i, state, area) in [
            (1, "published", 1),
            (2, "published", 2),
            (3, "pending", 1),
            (4, "removed", 1),
        ] {
            db.execute(
                "INSERT INTO file_objects VALUES(?1,?2,1,?3,'Original test document',1,?4,?5)",
                rusqlite::params![
                    format!("{i:032x}"),
                    area,
                    format!("file-{i}.txt"),
                    "a".repeat(64),
                    state
                ],
            )
            .unwrap();
        }
        (b, s, db)
    }
    fn source(&self, db: &rusqlite::Connection) -> String {
        let ns: String = db
            .query_row("SELECT id FROM file_namespace", [], |r| r.get(0))
            .unwrap();
        format!("version=1\nenabled=true\nname='Night library'\ncompact=false\n[[shelf]]\nid='manuals'\ntitle='Public field notes'\n[[shelf]]\nid='hidden'\ntitle='Hidden collection'\n{}",(1..=4).map(|i|format!("[[book]]\nid='book-{i}'\nshelf='{}'\nreference='faction-file:{ns}/{i:032x}'\nnote='{}'\nrights='Original BSD-3-Clause test data'\nprovenance='Synthetic owner-curated test fixture'\nroom=1\n",if i==2{"hidden"}else{"manuals"},if i==2{"PRIVATE-NOTE"}else{"Original useful notes"})).collect::<String>())
    }
    fn load(&self, source: &str) -> catalog::Result<Catalog> {
        fs::write(self.0.join("library.toml"), source).unwrap();
        Catalog::load(&self.0)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn render(d: &mut Desk, b: &mut Board, s: &Session, c: &Catalog) -> String {
    d.render(b, s, c, &Theme::default(), false, 40).unwrap()
}
fn feed(d: &mut Desk, v: &str, b: &mut Board, s: &Session, c: &Catalog) {
    assert!(matches!(d.feed(v, b, s, c).unwrap(), Outcome::Stay));
}
fn select(d: &mut Desk, v: &str, b: &mut Board, s: &Session, c: &Catalog) {
    feed(d, "v", b, s, c);
    feed(d, v, b, s, c);
}
#[test]
fn manifest_requires_version_rights_provenance_and_exact_local_references() {
    let f = Fixture::new();
    let (_, _, db) = f.board();
    let source = f.source(&db);
    assert_eq!(f.load(&source).unwrap().len(), 4);
    for (a, b) in [
        ("version=1", "version=2"),
        ("name='Night library'", "name='bad\u{1b}'"),
        ("rights='Original BSD-3-Clause test data'", "rights=''"),
        (
            "provenance='Synthetic owner-curated test fixture'",
            "provenance=''",
        ),
        ("id='book-2'", "id='book-1'"),
        ("shelf='hidden'", "shelf='absent'"),
        ("room=1", "room=-1"),
        ("compact=false", "compact='yes'"),
    ] {
        assert!(Catalog::parse(&source.replacen(a, b, 1)).is_err(), "{a}");
    }
    assert!(Catalog::parse(&(source.clone() + "unknown=12\n")).is_err());
    let reference = source
        .lines()
        .find(|s| s.starts_with("reference="))
        .unwrap();
    assert!(
        Catalog::parse(&source.replacen(reference, "reference='https://example.invalid/file'", 1))
            .is_err()
    );
}
#[test]
fn pack_load_checks_owned_regular_files_and_input_caps() {
    let f = Fixture::new();
    let (_, _, db) = f.board();
    let source = f.source(&db);
    f.load(&source).unwrap();
    let path = f.0.join("library.toml");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();
    assert!(Catalog::load(&f.0).is_err());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::rename(&path, f.0.join("other.toml")).unwrap();
    symlink("other.toml", &path).unwrap();
    assert!(Catalog::load(&f.0).is_err());
    fs::remove_file(&path).unwrap();
    fs::hard_link(f.0.join("other.toml"), &path).unwrap();
    assert!(Catalog::load(&f.0).is_err());
    fs::remove_file(&path).unwrap();
    fs::write(&path, vec![b' '; BYTES + 1]).unwrap();
    assert!(Catalog::load(&f.0).is_err());
    assert!(Catalog::parse(&"[".repeat(257)).is_err());
}
#[test]
fn catalog_bounds_books_shelves_and_disabled_capability() {
    let f = Fixture::new();
    let (_, _, db) = f.board();
    let source = f.source(&db);
    let head = source.split("[[book]]").next().unwrap();
    let body = source.split("[[book]]").nth(1).unwrap();
    let many = |n| {
        format!(
            "{head}{}",
            (0..n)
                .map(|i| format!(
                    "[[book]]{}",
                    body.replace("id='book-1'", &format!("id='b-{i}'"))
                ))
                .collect::<String>()
        )
    };
    assert_eq!(Catalog::parse(&many(32)).unwrap().len(), 32);
    assert!(Catalog::parse(&many(33)).is_err());
    let shelves = |n| {
        format!(
            "version=1\nenabled=true\nname='Test'\ncompact=false\nbook=[]\n{}",
            (0..n)
                .map(|i| format!("[[shelf]]\nid='s-{i}'\ntitle='Shelf'\n"))
                .collect::<String>()
        )
    };
    assert!(Catalog::parse(&shelves(8)).is_ok());
    assert!(Catalog::parse(&shelves(9)).is_err());
    assert!(
        !Catalog::parse(&source.replace("enabled=true", "enabled=false"))
            .unwrap()
            .enabled()
    );
    assert!(!Catalog::default().enabled());
}
#[test]
fn shelves_and_notes_filter_before_display_and_recheck_grants() {
    let f = Fixture::new();
    let (mut b, _, db) = f.board();
    let s = b.authenticate("reader", PASS).unwrap();
    let c = f.load(&f.source(&db)).unwrap();
    let mut d = Desk::default();
    let out = render(&mut d, &mut b, &s, &c);
    assert!(out.contains("file-1.txt"));
    for hidden in ["book-2", "file-2.txt", "book-3", "book-4", "PRIVATE-NOTE"] {
        assert!(!out.contains(hidden));
    }
    feed(&mut d, "e", &mut b, &s, &c);
    assert!(!render(&mut d, &mut b, &s, &c).contains("Hidden collection"));
    assert!(matches!(
        d.feed("hidden", &mut b, &s, &c),
        Err(Error::NotFound)
    ));
    db.execute("INSERT INTO file_area_grants VALUES(2,2)", [])
        .unwrap();
    feed(&mut d, "hidden", &mut b, &s, &c);
    assert!(render(&mut d, &mut b, &s, &c).contains("PRIVATE-NOTE"));
    select(&mut d, "book-2", &mut b, &s, &c);
    db.execute("DELETE FROM file_area_grants", []).unwrap();
    let err = d
        .render(&mut b, &s, &c, &Theme::default(), false, 40)
        .unwrap_err();
    assert!(d.recover(&err));
    let out = render(&mut d, &mut b, &s, &c);
    assert!(!out.contains("PRIVATE-NOTE") && !out.contains("Hidden collection"));
}
#[test]
fn notes_links_resize_and_previews_are_bounded_and_leave_unread_alone() {
    let f = Fixture::new();
    let (mut b, s, db) = f.board();
    let source = f
        .source(&db)
        .replace("Original useful notes", &"word ".repeat(45));
    let c = f.load(&source).unwrap();
    let mut d = Desk::default();
    select(&mut d, "book-1", &mut b, &s, &c);
    let mut all = String::new();
    for _ in 0..16 {
        let out = render(&mut d, &mut b, &s, &c);
        assert!(out.len() <= 8192 && !out.contains('\x1b'));
        all.push_str(&out);
        feed(&mut d, "n", &mut b, &s, &c);
    }
    assert!(all.contains("Provenance"));
    let out = d
        .render(&mut b, &s, &c, &Theme::default(), true, 80)
        .unwrap();
    assert!(out.contains("file-1.txt / 1 bytes"));
    feed(&mut d, "l", &mut b, &s, &c);
    assert!(render(&mut d, &mut b, &s, &c).contains("faction-file:"));
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
}
#[test]
fn file_and_discussion_jumps_recheck_exact_current_sources() {
    let f = Fixture::new();
    let (mut b, s, db) = f.board();
    let c = f.load(&f.source(&db)).unwrap();
    let mut d = Desk::with_files(true);
    select(&mut d, "book-1", &mut b, &s, &c);
    assert!(
        matches!(d.feed("f",&mut b,&s,&c).unwrap(),Outcome::File(id) if id==format!("{:032x}",1))
    );
    assert!(matches!(
        d.feed("d", &mut b, &s, &c).unwrap(),
        Outcome::Room(1)
    ));
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
    db.execute(
        "UPDATE file_objects SET state='removed' WHERE id=?1",
        [format!("{:032x}", 1)],
    )
    .unwrap();
    assert!(matches!(d.feed("f", &mut b, &s, &c), Err(Error::NotFound)));
    assert!(matches!(d.feed("d", &mut b, &s, &c), Err(Error::NotFound)));
}
#[test]
fn approval_epochs_foreign_sessions_and_disabled_packs_never_grant_access() {
    let f = Fixture::new();
    let (mut b, s, db) = f.board();
    let c = f.load(&f.source(&db)).unwrap();
    let mut d = Desk::default();
    b.register("pending", PASS).unwrap();
    let pending = b.authenticate("pending", PASS).unwrap();
    assert!(matches!(
        d.render(&mut b, &pending, &c, &Theme::default(), false, 80),
        Err(Error::Denied)
    ));
    let reader = b.authenticate("reader", PASS).unwrap();
    b.set_status(&s, "reader", Status::Revoked).unwrap();
    assert!(d.feed("q", &mut b, &reader, &c).is_err());
    let other = Fixture::new();
    let (mut foreign, _, _) = other.board();
    assert!(d.feed("q", &mut foreign, &s, &c).is_err());
    assert!(d.feed("q", &mut b, &s, &Catalog::default()).is_err());
    for v in ["\x1b[6n", &"a".repeat(65)] {
        assert!(d.feed(v, &mut b, &s, &c).is_err());
    }
}
#[test]
fn invisible_selected_source_is_not_replaced_and_unavailable_transfers_stay_disabled() {
    let f = Fixture::new();
    let (mut b, _, db) = f.board();
    let s = b.authenticate("reader", PASS).unwrap();
    let c = f.load(&f.source(&db)).unwrap();
    let mut d = Desk::default();
    select(&mut d, "book-1", &mut b, &s, &c);
    assert!(!render(&mut d, &mut b, &s, &c).contains("F:Files"));
    assert!(matches!(
        d.feed("f", &mut b, &s, &c),
        Err(Error::Invalid(_))
    ));
    db.execute(
        "DELETE FROM file_objects WHERE id=?1",
        [format!("{:032x}", 1)],
    )
    .unwrap();
    let e = d
        .render(&mut b, &s, &c, &Theme::default(), false, 80)
        .unwrap_err();
    assert!(d.recover(&e));
    let out = render(&mut d, &mut b, &s, &c);
    assert!(!out.contains("file-2.txt") && !out.contains("PRIVATE-NOTE"));
    assert!(out.contains("Library>"));
}

#[test]
fn requests_only_pack_preview_cancel_create_fulfill_close_and_classic_frames() {
    let f = Fixture::new();
    let (mut b, s, db) = f.board();
    let c=f.load("version=1\nenabled=true\nname='Community library'\ncompact=false\nrequests=true\nrequests_name='Wanted files'\nshelf=[]\nbook=[]\n").unwrap();
    assert!(c.enabled());
    let mut d = Desk::default();
    feed(&mut d, "r", &mut b, &s, &c);
    assert!(render(&mut d, &mut b, &s, &c).contains("Wanted files"));
    for input in ["c", "A useful guide", "Original licensed documentation"] {
        feed(&mut d, input, &mut b, &s, &c);
        render(&mut d, &mut b, &s, &c);
    }
    assert!(b.file_requests(&s, i64::MAX).unwrap().is_empty());
    feed(&mut d, "q", &mut b, &s, &c);
    feed(&mut d, "c", &mut b, &s, &c);
    feed(&mut d, "A useful guide", &mut b, &s, &c);
    feed(&mut d, "Original licensed documentation", &mut b, &s, &c);
    feed(&mut d, "yes", &mut b, &s, &c);
    let id = b.file_requests(&s, i64::MAX).unwrap()[0].id;
    for input in [
        "u",
        &format!("{:032x}", 1),
        "BSD-3-Clause original fixture",
        "Fixture author",
        "yes",
    ] {
        feed(&mut d, input, &mut b, &s, &c);
        let out = render(&mut d, &mut b, &s, &c);
        assert!(out.len() < 8192 && !out.contains('\x1b'));
    }
    assert!(b.file_request(&s, id).unwrap().offer.is_some());
    feed(&mut d, "v", &mut b, &s, &c);
    feed(&mut d, &id.to_string(), &mut b, &s, &c);
    feed(&mut d, "l", &mut b, &s, &c);
    assert!(render(&mut d, &mut b, &s, &c).contains("faction-file:"));
    feed(&mut d, "q", &mut b, &s, &c);
    feed(&mut d, "x", &mut b, &s, &c);
    feed(&mut d, "yes", &mut b, &s, &c);
    assert_eq!(b.file_request(&s, id).unwrap().state, "closed");
    assert_eq!(
        db.query_row("SELECT count(*) FROM unread", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn request_review_never_silently_adopts_changed_revision_or_withdrawn_file() {
    let f = Fixture::new();
    let (mut b, s, _) = f.board();
    let c = f
        .load(
            &f.source(&rusqlite::Connection::open(f.0.join("board/faction.sqlite3")).unwrap())
                .replace("compact=false", "compact=false\nrequests=true"),
        )
        .unwrap();
    let id = b
        .create_file_request(
            &s,
            &faction_core::request_id().unwrap(),
            &faction_core::FileRequestDraft {
                title: "Guide".into(),
                description: "Original manual".into(),
            },
        )
        .unwrap();
    let mut d = Desk::default();
    for input in ["r", "v", &id.to_string(), "x"] {
        feed(&mut d, input, &mut b, &s, &c);
    }
    let row = b.file_request(&s, id).unwrap();
    b.finish_file_request(&s, id, &row.revision, false).unwrap();
    let e = d
        .render(&mut b, &s, &c, &Theme::default(), false, 40)
        .unwrap_err();
    assert!(d.recover(&e));
    assert!(render(&mut d, &mut b, &s, &c).contains("reopen"));
    assert_eq!(b.file_request(&s, id).unwrap().state, "closed");
}

#[test]
fn long_request_review_pages_and_resize_never_send_unseen_tail() {
    let f = Fixture::new();
    let (mut b, s, _) = f.board();
    let c=f.load("version=1\nenabled=true\nname='Library'\ncompact=false\nrequests=true\nshelf=[]\nbook=[]").unwrap();
    let mut d = Desk::default();
    for input in ["r", "c", &"T".repeat(64), &"D".repeat(128)] {
        feed(&mut d, input, &mut b, &s, &c);
    }
    let out = d
        .render(&mut b, &s, &c, &Theme::default(), false, 20)
        .unwrap();
    assert!(out.contains("Post request?"));
    assert!(d.feed("yes", &mut b, &s, &c).is_err());
    assert!(b.file_requests(&s, i64::MAX).unwrap().is_empty());
    feed(&mut d, "n", &mut b, &s, &c);
    d.render(&mut b, &s, &c, &Theme::default(), false, 20)
        .unwrap();
    d.render(&mut b, &s, &c, &Theme::default(), false, 40)
        .unwrap();
    feed(&mut d, "yes", &mut b, &s, &c);
    assert_eq!(b.file_requests(&s, i64::MAX).unwrap().len(), 1);
}

fn magazine_source(state: &str, consent: &str, articles: usize) -> String {
    let mut out = format!(
        "version=1\nenabled=true\nname='Library'\ncompact=false\nmagazine_name='Nightwire'\nshelf=[]\nbook=[]\n[[magazine]]\nid='signal-1'\nnumber=1\nrevision=1\nstate='{state}'\ntitle='Local edition'\neditor='Fixture editor'\nreleased='Autumn 2026'\nschedule='When ready'\n"
    );
    for i in 0..articles {
        out.push_str(&format!("[[magazine.article]]\nid='article-{i}'\ntitle='Original article {i}'\nsection='Notes'\nauthor='Fixture author'\nrights='BSD-3-Clause original fixture'\nprovenance='Original local editorial fixture'\nconsent='{consent}'\nbody='''{}'''\n",(0..50).map(|i|format!("Line {i} original body\n")).collect::<String>()));
    }
    out
}
#[test]
fn magazine_strict_roles_of_declared_consent_states_and_unknown_fields() {
    let good = magazine_source("published", "board-local", 1);
    let c = Catalog::parse(&good).unwrap();
    assert!(c.enabled());
    assert_eq!(c.published_issues(), 1);
    for (a, b) in [
        ("consent='board-local'", "consent='pending'"),
        ("number=1", "number=0"),
        ("state='published'", "state='public'"),
        ("revision=1", "revision=10000"),
        ("author='Fixture author'", "author='bad\x1b'"),
        ("body='''", "body='''faction-file:unreviewed "),
    ] {
        assert!(Catalog::parse(&good.replace(a, b)).is_err());
    }
    assert!(Catalog::parse(&format!("{good}unknown=true\n")).is_err());
    assert!(Catalog::parse(&magazine_source("published", "board-local", 9)).is_err());
    for state in ["draft", "withdrawn"] {
        let c = Catalog::parse(&magazine_source(state, "pending", 1)).unwrap();
        assert!(!c.enabled());
        assert_eq!(c.published_issues(), 0);
        assert!(c.magazine_text("signal-1", false).is_err());
        assert!(c.magazine_text("signal-1", true).unwrap().contains("["));
    }
}
#[test]
fn magazine_plain_complete_read_credits_toc_pages_resize_and_no_unread_ack() {
    let f = Fixture::new();
    let (mut b, s, _) = f.board();
    let c = Catalog::parse(&magazine_source("published", "board-local", 5)).unwrap();
    let mut d = Desk::default();
    for key in ["m", "v", "signal-1"] {
        feed(&mut d, key, &mut b, &s, &c);
    }
    assert!(render(&mut d, &mut b, &s, &c).contains("article-0"));
    feed(&mut d, "n", &mut b, &s, &c);
    assert!(render(&mut d, &mut b, &s, &c).contains("article-4"));
    for key in ["r", "article-0"] {
        feed(&mut d, key, &mut b, &s, &c);
    }
    let mut all = String::new();
    for _ in 0..20 {
        let frame = render(&mut d, &mut b, &s, &c);
        assert!(frame.len() < 8192 && !frame.contains('\x1b'));
        all.push_str(&frame);
        feed(&mut d, "n", &mut b, &s, &c);
    }
    assert!(all.contains("Line 49 original body"));
    feed(&mut d, "p", &mut b, &s, &c);
    d.render(&mut b, &s, &c, &Theme::default(), false, 20)
        .unwrap();
    feed(&mut d, "c", &mut b, &s, &c);
    let mut credits = String::new();
    for _ in 0..6 {
        credits.push_str(&render(&mut d, &mut b, &s, &c));
        feed(&mut d, "n", &mut b, &s, &c);
    }
    assert!(credits.contains("Original local editorial"));
    feed(&mut d, "t", &mut b, &s, &c);
    assert!(render(&mut d, &mut b, &s, &c).contains("Contents>"));
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
}
#[test]
fn magazine_current_approval_unpublished_selection_and_file_grants_are_independent() {
    let f = Fixture::new();
    let (mut b, s, db) = f.board();
    let reader = b.authenticate("reader", PASS).unwrap();
    let ns = b.file_namespace(&s).unwrap();
    let raw = magazine_source("published", "board-local", 1).replace(
        "schedule='When ready'",
        &format!(
            "schedule='When ready'\ndownload='faction-file:{ns}/{:032x}'\nroom=1",
            2
        ),
    );
    let c = Catalog::parse(&raw).unwrap();
    let mut d = Desk::with_files(true);
    for key in ["m", "v", "signal-1"] {
        feed(&mut d, key, &mut b, &reader, &c);
    }
    assert!(!render(&mut d, &mut b, &reader, &c).contains("F:Files"));
    assert!(d.feed("l", &mut b, &reader, &c).is_err());
    b.file_allow(&s, 2, "reader", true).unwrap();
    assert!(render(&mut d, &mut b, &reader, &c).contains("F:Files"));
    feed(&mut d, "l", &mut b, &reader, &c);
    assert!(render(&mut d, &mut b, &reader, &c).contains("faction-file:"));
    b.file_allow(&s, 2, "reader", false).unwrap();
    let e = d
        .render(&mut b, &reader, &c, &Theme::default(), false, 40)
        .unwrap_err();
    assert!(d.recover(&e));
    assert!(!render(&mut d, &mut b, &reader, &c).contains("faction-file:"));
    for key in ["v", "signal-1"] {
        feed(&mut d, key, &mut b, &reader, &c);
    }
    assert!(matches!(
        d.feed("d", &mut b, &reader, &c).unwrap(),
        Outcome::Room(1)
    ));
    assert_eq!(b.unread_cursor(&reader, 1).unwrap(), 0);
    b.set_status(&s, "reader", Status::Revoked).unwrap();
    assert!(
        d.render(&mut b, &reader, &c, &Theme::default(), false, 40)
            .is_err()
    );
    assert!(d.feed("q", &mut b, &reader, &c).is_err());
    drop(db);
}
#[test]
fn magazine_owner_export_is_bounded_inert_and_starter_edition_is_original() {
    let c =
        Catalog::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../library/nightwire"))
            .unwrap();
    let text = c.magazine_text("signal-001", false).unwrap();
    assert!(text.contains("End of issue 001."));
    assert!(text.len() < 65536 && text.bytes().all(|b| b == b'\n' || (32..=126).contains(&b)));
    assert_eq!(c.published_issues(), 1);
    assert!(c.magazine_text("missing", false).is_err());
}
#[test]
fn magazine_body_pack_and_issue_limits_reject_oversize_duplicate_or_empty_editions() {
    let raw = magazine_source("published", "board-local", 1);
    assert!(Catalog::parse(&raw.replace("Line 0 original body", &"x".repeat(121))).is_err());
    assert!(Catalog::parse(&raw.replace("Line 0 original body", &"x\n".repeat(129))).is_err());
    assert!(Catalog::parse(&magazine_source("published", "board-local", 0)).is_err());
    let issue = raw.split_once("[[magazine]]").unwrap().1;
    assert!(Catalog::parse(&format!("{raw}[[magazine]]{issue}")).is_err());
    assert!(Catalog::parse(&raw.repeat(20)).is_err());
}

#[test]
fn maximum_article_reads_beyond_rewind_capacity_on_narrow_terminals() {
    let f = Fixture::new();
    let (mut b, s, _) = f.board();
    let raw = magazine_source("published", "board-local", 1);
    let old = (0..50)
        .map(|i| format!("Line {i} original body\n"))
        .collect::<String>();
    let body = format!(
        "{}FINAL-ARTICLE-MARKER",
        format!("{}\n", "X".repeat(120)).repeat(67)
    );
    assert!(body.len() <= 8192);
    let c = Catalog::parse(&raw.replace(&old, &body)).unwrap();
    let mut d = Desk::default();
    for key in ["m", "v", "signal-1", "r", "article-0"] {
        feed(&mut d, key, &mut b, &s, &c);
    }
    let mut found = false;
    for _ in 0..150 {
        let out = d
            .render(&mut b, &s, &c, &Theme::default(), false, 20)
            .unwrap();
        assert!(out.len() <= 8192);
        if out.contains("FINAL-ARTICLE-MARKE") {
            found = true;
            break;
        }
        feed(&mut d, "n", &mut b, &s, &c);
    }
    assert!(found);
    for _ in 0..40 {
        feed(&mut d, "p", &mut b, &s, &c);
        d.render(&mut b, &s, &c, &Theme::default(), false, 20)
            .unwrap();
    }
}
