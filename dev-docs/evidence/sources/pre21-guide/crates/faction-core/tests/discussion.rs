use faction_core::{
    Board, DiscussionField as Field, DiscussionQuery as Query, Error, PostDraft, Session, Status,
    export_local, request_id, restore_local, verify_export,
};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-poll-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("faction-discussion-{}", request_id().unwrap()));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        Self(p)
    }
    fn path(&self) -> PathBuf {
        self.0.join("board")
    }
    fn db(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(self.path().join("faction.sqlite3")).unwrap()
    }
    fn board(&self) -> (Board, Session) {
        let mut b = Board::open(&self.path()).unwrap();
        b.bootstrap("sysop", PASS).unwrap();
        let s = b.authenticate("sysop", PASS).unwrap();
        (b, s)
    }
    fn enable(&self, b: &mut Board, s: &Session, members: bool) {
        let mut c = b.discussion_config(s).unwrap();
        c.enabled = true;
        let _ = members;
        b.configure_discussion(s, &c).unwrap();
    }
    fn member(&self, b: &mut Board, s: &Session, name: &str) -> Session {
        b.register(name, PASS).unwrap();
        b.set_status(s, name, Status::Approved).unwrap();
        b.authenticate(name, PASS).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn post(b: &mut Board, s: &Session, body: &str, reply: Option<i64>) -> i64 {
    b.post(s, 1, &request_id().unwrap(), body, reply)
        .unwrap()
        .id
}
fn file(f: &Fixture, area: i64, id: &str) {
    f.db()
        .execute(
            "INSERT INTO file_objects VALUES(?1,?2,1,?3,'Synthetic file',1,?4,'published')",
            rusqlite::params![id, area, format!("{}.txt", &id[..4]), "a".repeat(64)],
        )
        .unwrap();
}
fn query(text: &str, field: Field) -> Query {
    Query {
        text: text.into(),
        field,
        room: None,
        author: String::new(),
    }
}
fn article(b: &mut Board, s: &Session, room: i64, subject: &str, body: &str) -> i64 {
    b.post_article(
        s,
        room,
        &request_id().unwrap(),
        PostDraft {
            subject,
            body,
            reply_to: None,
        },
    )
    .unwrap()
    .id
}
#[test]
fn default_optional_policy_current_roles_sessions_and_unicode_query_validation() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    assert!(b.discussion_start(&s).is_err());
    f.enable(&mut b, &s, false);
    let a = f.member(&mut b, &s, "alice");
    let c = b.discussion_config(&s).unwrap();
    assert!(b.configure_discussion(&a, &c).is_err());
    b.register("pending", PASS).unwrap();
    let p = b.authenticate("pending", PASS).unwrap();
    assert!(b.discussion_start(&p).is_err());
    let other = Fixture::new();
    let (mut ob, _) = other.board();
    assert!(ob.discussion_bookmark(&s, 1, true).is_err());
    for text in ["", " ", "bad\ntext", "\x1b[2J"] {
        assert!(
            b.discussion_search(&s, &query(text, Field::Both), 99)
                .is_err()
        );
    }
    assert!(
        b.discussion_search(&s, &query(&"x".repeat(65), Field::Both), 99)
            .is_err()
    );
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(b.discussion_start(&a).is_err());
}
#[test]
fn literal_subject_body_room_and_exact_author_filters_are_useful() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let a = f.member(&mut b, &s, "alice");
    let r = b.create_room(&s, "code", "Coding").unwrap();
    article(&mut b, &s, 1, "Café ANSI", "SQLite tips");
    article(&mut b, &a, r, "Archive guide", "Find the ANSI collection");
    article(&mut b, &s, 1, "Literal patterns", "%_ OR 1=1");
    assert_eq!(
        b.discussion_search(&s, &query("ANSI", Field::Subject), 99)
            .unwrap()
            .hits
            .len(),
        1
    );
    assert_eq!(
        b.discussion_search(&s, &query("ansi", Field::Body), 99)
            .unwrap()
            .hits[0]
            .author,
        "alice"
    );
    assert_eq!(
        b.discussion_search(&s, &query("Café", Field::Subject), 99)
            .unwrap()
            .hits
            .len(),
        1
    );
    assert_eq!(
        b.discussion_search(&s, &query("%_ OR 1=1", Field::Body), 99)
            .unwrap()
            .hits
            .len(),
        1
    );
    let mut q = query("ANSI", Field::Both);
    q.room = Some(r);
    q.author = "ALICE".into();
    assert_eq!(b.discussion_search(&s, &q, 99).unwrap().hits.len(), 1);
    q.author = "sysop".into();
    assert!(b.discussion_search(&s, &q, 99).unwrap().hits.is_empty());
    q.room = None;
    q.author = "nobody".into();
    assert!(b.discussion_search(&s, &q, 99).unwrap().hits.is_empty());
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
}
#[test]
fn bounded_sparse_search_pages_do_not_skip_older_hits_or_new_publications() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let first = article(&mut b, &s, 1, "Needle", "Old useful conversation");
    for _ in 0..130 {
        post(&mut b, &s, "ordinary traffic", None);
    }
    let start = b.discussion_start(&s).unwrap();
    let new = article(&mut b, &s, 1, "Needle", "New arrival");
    let q = query("Needle", Field::Subject);
    let mut at = start;
    let mut found = vec![];
    for _ in 0..200 {
        let p = b.discussion_search(&s, &q, at).unwrap();
        assert!(p.scanned <= 100 && p.hits.len() <= 4);
        found.extend(p.hits.into_iter().map(|h| h.id));
        match p.next {
            Some(next) => {
                assert!(next < at);
                at = next;
            }
            None => break,
        }
    }
    assert_eq!(found, vec![first]);
    let refreshed = b.discussion_start(&s).unwrap();
    assert_eq!(
        b.discussion_search(&s, &q, refreshed).unwrap().hits[0].id,
        new
    );
    assert!(b.discussion_search(&s, &q, -1).is_err());
    assert!(b.discussion_bookmarks(&s, -1).is_err());
}
#[test]
fn file_projection_precedes_matching_snippets_and_exact_source_views() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let a = f.member(&mut b, &s, "alice");
    let area = b
        .file_create_area(&s, "private", "Private files", true, 10000)
        .unwrap();
    file(&f, area, &"a".repeat(32));
    let link = b.file_reference(&s, &"a".repeat(32)).unwrap();
    let id = article(&mut b, &s, 1, "Library", &link);
    let q = query("aaaa.txt", Field::Body);
    assert!(b.discussion_search(&a, &q, 99).unwrap().hits.is_empty());
    assert!(!b.discussion_item(&a, id).unwrap().body.contains(&link));
    b.file_allow(&s, area, "alice", true).unwrap();
    assert_eq!(b.discussion_search(&a, &q, 99).unwrap().hits.len(), 1);
    b.discussion_bookmark(&a, id, true).unwrap();
    b.file_allow(&s, area, "alice", false).unwrap();
    assert!(b.discussion_search(&a, &q, 99).unwrap().hits.is_empty());
    assert!(
        !b.discussion_bookmarks(&a, 99).unwrap().hits[0]
            .snippet
            .contains("aaaa.txt")
    );
    assert!(
        !b.discussion_search(&a, &query(&"a".repeat(32), Field::Body), 99)
            .unwrap()
            .hits
            .iter()
            .any(|h| h.id == id)
    );
    assert_eq!(b.unread_cursor(&a, 1).unwrap(), 0);
}
#[test]
fn bookmarks_are_member_owned_idempotent_bounded_and_newest_first() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let a = f.member(&mut b, &s, "alice");
    for _ in 0..129 {
        let p = post(&mut b, &s, "Bookmark candidate", None);
        if p <= 128 {
            b.discussion_bookmark(&s, p, true).unwrap();
        }
    }
    b.discussion_bookmark(&s, 128, true).unwrap();
    assert!(b.discussion_bookmark(&s, 129, true).is_err());
    assert!(b.discussion_bookmarks(&a, 999).unwrap().hits.is_empty());
    let p = b.discussion_bookmarks(&s, 999).unwrap();
    assert_eq!(p.hits.len(), 4);
    assert_eq!(p.hits[0].id, 128);
    assert_eq!(
        b.discussion_bookmarks(&s, p.next.unwrap()).unwrap().hits[0].id,
        124
    );
    b.discussion_bookmark(&s, 128, false).unwrap();
    b.discussion_bookmark(&s, 128, false).unwrap();
    b.discussion_bookmark(&s, 129, true).unwrap();
    assert!(b.discussion_bookmark(&s, 999, true).is_err());
}
#[test]
fn removed_posts_never_leave_stale_titles_sources_or_export_rows() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let id = article(&mut b, &s, 1, "Removed title", "Gone body");
    b.discussion_bookmark(&s, id, true).unwrap();
    let db = f.db();
    db.execute_batch("PRAGMA foreign_keys=ON;DELETE FROM post_metadata;DELETE FROM posts")
        .unwrap();
    assert!(b.discussion_item(&s, id).is_err());
    assert!(b.discussion_bookmarks(&s, 999).unwrap().hits.is_empty());
    assert!(
        !b.discussion_bookmark_export(&s, 999)
            .unwrap()
            .0
            .contains("Removed")
    );
    assert!(
        b.discussion_search(&s, &query("Gone", Field::Both), 999)
            .unwrap()
            .hits
            .is_empty()
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM discussion_bookmarks", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn explicit_bookmark_reference_export_is_lossless_scoped_and_not_an_unread_ack() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let id = article(&mut b, &s, 1, "Café 界", "Useful body");
    b.discussion_bookmark(&s, id, true).unwrap();
    let (out, next) = b.discussion_bookmark_export(&s, 99).unwrap();
    assert!(next.is_none());
    assert!(out.starts_with(&format!(
        "FACTION-BOOKMARKS/1 board={}\n",
        b.file_namespace(&s).unwrap()
    )));
    let fields = out.lines().nth(1).unwrap().split('\t').collect::<Vec<_>>();
    assert_eq!(fields.len(), 4);
    assert_eq!(fields[1], id.to_string());
    assert_eq!(fields[2], b.post_metadata(&s, id).unwrap().article_id);
    let bytes = (0..fields[3].len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&fields[3][i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(String::from_utf8(bytes).unwrap(), "Café 界");
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
}
#[test]
fn busy_readers_continue_but_bookmark_writes_retry_and_disable_hides_every_path() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let id = post(&mut b, &s, "Useful", None);
    let lock = f.db();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    assert!(
        b.discussion_search(&s, &query("Useful", Field::Body), 99)
            .is_ok()
    );
    assert!(matches!(
        b.discussion_bookmark(&s, id, true),
        Err(Error::Busy)
    ));
    lock.execute_batch("ROLLBACK").unwrap();
    b.discussion_bookmark(&s, id, true).unwrap();
    let mut c = b.discussion_config(&s).unwrap();
    c.enabled = false;
    b.configure_discussion(&s, &c).unwrap();
    assert!(b.discussion_item(&s, id).is_err());
    assert!(b.discussion_bookmark_export(&s, 99).is_err());
    assert!(b.discussion_bookmark(&s, id, false).is_err());
}
#[test]
fn complete_snapshot_and_exact_schema_thirteen_exports_remain_readonly() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let id = post(&mut b, &s, "Retained reference", None);
    b.discussion_bookmark(&s, id, true).unwrap();
    drop(b);
    let out = f.0.join("export");
    export_local(&f.path(), &out).unwrap();
    verify_export(&out).unwrap();
    let target = f.0.join("restored");
    restore_local(&out, &target).unwrap();
    let mut b = Board::open(&target).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert_eq!(b.discussion_bookmarks(&s, 99).unwrap().hits[0].id, id);
    drop(b);
    f.db().execute_batch("DROP TABLE visit_activity;DROP TABLE social_pages;DROP TABLE social_presence;DROP TABLE social_blocks;DROP TABLE social_prefs;DROP TABLE social_cooldowns;DROP TABLE social_settings;DROP TABLE magazine_entries;DROP TABLE magazine_issues;DROP TABLE magazine_submissions;DROP TABLE file_requests;DROP INDEX discussion_author_posts;DROP TABLE discussion_bookmarks;DROP TABLE discussion_settings;PRAGMA user_version=13").unwrap();
    let old = f.0.join("old-export");
    export_local(&f.path(), &old).unwrap();
    verify_export(&old).unwrap();
    assert_eq!(
        f.db()
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        13
    );
    let restored = f.0.join("old-restored");
    restore_local(&old, &restored).unwrap();
    let mut b = Board::open(&restored).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert!(!b.discussion_config(&s).unwrap().enabled);
}
