use faction_articles::{Article, IdentityMatch};
use faction_core::{Board, Error, PostDraft, Status, export_local, request_id, restore_local};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const SECRET: &str = "synthetic-private-article-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("faction-article-{}", request_id().unwrap()));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        Self(p)
    }
    fn board(&self) -> Board {
        let mut b = Board::open(&self.0.join("board")).unwrap();
        b.bootstrap("private-sysop", SECRET).unwrap();
        b
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn draft<'a>(subject: &'a str, body: &'a str, reply_to: Option<i64>) -> PostDraft<'a> {
    PostDraft {
        subject,
        body,
        reply_to,
    }
}
#[test]
fn authored_subjects_reply_defaults_retries_and_restore_are_stable() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("private-sysop", SECRET).unwrap();
    let key = request_id().unwrap();
    let p = b
        .post_article(
            &s,
            1,
            &key,
            draft("Midnight frequencies", "First body", None),
        )
        .unwrap();
    let a = b.room_article(&s, p.id).unwrap();
    assert_eq!(a.headers().subject, "Midnight frequencies");
    assert_eq!(
        b.post_article(
            &s,
            1,
            &key,
            draft("Midnight frequencies", "First body", None)
        )
        .unwrap()
        .id,
        p.id
    );
    assert!(matches!(
        b.post_article(&s, 1, &key, draft("changed", "First body", None)),
        Err(Error::RequestConflict)
    ));
    assert!(matches!(
        b.post(&s, 1, &key, "First body", None),
        Err(Error::RequestConflict)
    ));
    let reply = b
        .post_article(
            &s,
            1,
            &request_id().unwrap(),
            draft("", "reply", Some(p.id)),
        )
        .unwrap();
    assert_eq!(
        b.post_metadata(&s, reply.id).unwrap().subject,
        "Re: Midnight frequencies"
    );
    let r2 = b
        .post(
            &s,
            1,
            &request_id().unwrap(),
            "second reply",
            Some(reply.id),
        )
        .unwrap();
    assert_eq!(
        b.post_metadata(&s, r2.id).unwrap().subject,
        "Re: Midnight frequencies"
    );
    let custom = b
        .post_article(
            &s,
            1,
            &request_id().unwrap(),
            draft("A different subject", "reply", Some(r2.id)),
        )
        .unwrap();
    assert_eq!(
        b.post_metadata(&s, custom.id).unwrap().subject,
        "A different subject"
    );
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
    drop(b);
    let export = f.0.join("export");
    export_local(&f.0.join("board"), &export).unwrap();
    let restore = f.0.join("restore");
    restore_local(&export, &restore).unwrap();
    let mut b = Board::open(&restore).unwrap();
    let s = b.authenticate("private-sysop", SECRET).unwrap();
    assert_eq!(b.room_article(&s, p.id).unwrap(), a);
    assert_eq!(
        b.post_metadata(&s, custom.id).unwrap().subject,
        "A different subject"
    );
}
#[test]
fn subjects_are_bounded_inert_and_current_approval_is_required() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("private-sysop", SECRET).unwrap();
    for subject in [
        " bad",
        "bad ",
        "bad\nApproved: root",
        "bad\x1b[2J",
        "=?utf-8?b?x?=",
        "café",
        &"x".repeat(161),
    ] {
        assert!(
            b.post_article(&s, 1, &request_id().unwrap(), draft(subject, "body", None))
                .is_err()
        );
    }
    let p = b
        .post_article(
            &s,
            1,
            &request_id().unwrap(),
            draft(&"x".repeat(160), "body", None),
        )
        .unwrap();
    let r = b
        .post(&s, 1, &request_id().unwrap(), "reply", Some(p.id))
        .unwrap();
    assert_eq!(b.post_metadata(&s, r.id).unwrap().subject.len(), 160);
    b.register("reader", SECRET).unwrap();
    let pending = b.authenticate("reader", SECRET).unwrap();
    assert!(matches!(b.room_article(&pending, p.id), Err(Error::Denied)));
    b.set_status(&s, "reader", Status::Approved).unwrap();
    let reader = b.authenticate("reader", SECRET).unwrap();
    assert!(b.room_article(&reader, p.id).is_ok());
    b.set_status(&s, "reader", Status::Revoked).unwrap();
    assert!(matches!(
        b.room_article(&reader, p.id),
        Err(Error::StaleSession)
    ));
    assert!(matches!(
        b.reply_subject(&reader, 1, p.id),
        Err(Error::StaleSession)
    ));
    assert!(
        b.post_article(
            &reader,
            1,
            &request_id().unwrap(),
            draft("title", "body", None)
        )
        .is_err()
    );
    assert_eq!(b.posts(&s, 1, 0).unwrap().len(), 2);
}
#[test]
fn projection_roundtrips_without_private_identity_and_keeps_chronological_cursors() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("private-sysop", SECRET).unwrap();
    let p = b
        .post_article(
            &s,
            1,
            &request_id().unwrap(),
            draft("After hours", "Unicode: café ☃\nline two", None),
        )
        .unwrap();
    let r = b
        .post(&s, 1, &request_id().unwrap(), "reply body", Some(p.id))
        .unwrap();
    let a = b.room_article(&s, r.id).unwrap();
    let wire = a.to_wire().unwrap();
    assert_eq!(
        a.headers().references,
        vec![b.post_metadata(&s, p.id).unwrap().article_id]
    );
    assert_eq!(a.headers().newsgroups, vec!["faction.local.r1"]);
    assert_eq!(a.headers().path, "not-for-mail");
    assert!(a.headers().from.ends_with("@authors.faction.invalid"));
    assert!(!wire.contains("private-sysop"));
    assert!(!wire.contains(&b.file_namespace(&s).unwrap()));
    assert!(!wire.contains("127.0.0.1"));
    assert!(!wire.contains(f.0.to_str().unwrap()));
    assert_eq!(Article::parse(wire.as_bytes()).unwrap(), a);
    let original = b.room_article(&s, p.id).unwrap();
    assert_eq!(original.body(), "Unicode: café ☃\nline two");
    assert_eq!(
        Article::parse(original.to_wire().unwrap().as_bytes()).unwrap(),
        original
    );
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
    assert_eq!(
        a.compare_identity(&Article::parse(wire.as_bytes()).unwrap()),
        IdentityMatch::Duplicate
    );
    let mut changed = a.headers().clone();
    changed.subject = "conflicting text".into();
    assert_eq!(
        a.compare_identity(&Article::new(changed, a.body().into()).unwrap()),
        IdentityMatch::Conflict
    );
}
#[test]
fn legacy_unknown_dates_unsupported_lines_and_inaccessible_file_links_fail_closed() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("private-sysop", SECRET).unwrap();
    let p = b
        .post(&s, 1, &request_id().unwrap(), "legacy body", None)
        .unwrap();
    let db = rusqlite::Connection::open(f.0.join("board/faction.sqlite3")).unwrap();
    db.execute(
        "UPDATE post_metadata SET created_at=NULL,origin='legacy' WHERE post_id=?1",
        [p.id],
    )
    .unwrap();
    assert!(matches!(b.room_article(&s, p.id), Err(Error::Invalid(_))));
    assert_eq!(b.posts(&s, 1, 0).unwrap()[0].body, "legacy body");
    let long = b
        .post(&s, 1, &request_id().unwrap(), &"a".repeat(1024), None)
        .unwrap();
    assert!(matches!(
        b.room_article(&s, long.id),
        Err(Error::Invalid(_))
    ));
    let private = b
        .post(&s, 1, &request_id().unwrap(), "file fixture", None)
        .unwrap();
    let reference = format!(
        "faction-file:{}/{}",
        b.file_namespace(&s).unwrap(),
        "a".repeat(32)
    );
    db.execute(
        "UPDATE posts SET body=?1 WHERE id=?2",
        rusqlite::params![reference, private.id],
    )
    .unwrap();
    assert!(matches!(b.room_article(&s, private.id), Err(Error::Denied)));
    assert_eq!(b.posts(&s, 1, 0).unwrap()[2].body, "[file unavailable]");
    assert_eq!(b.unread_cursor(&s, 1).unwrap(), 0);
}
#[test]
fn reply_reference_work_is_bounded_and_cross_room_replies_stay_denied() {
    let f = Fixture::new();
    let mut b = f.board();
    let s = b.authenticate("private-sysop", SECRET).unwrap();
    let mut p = b.post(&s, 1, &request_id().unwrap(), "root", None).unwrap();
    let root = b.post_metadata(&s, p.id).unwrap().article_id;
    for _ in 0..16 {
        p = b
            .post(&s, 1, &request_id().unwrap(), "reply", Some(p.id))
            .unwrap();
    }
    let a = b.room_article(&s, p.id).unwrap();
    assert_eq!(a.headers().references.len(), 16);
    assert_eq!(a.headers().references[0], root);
    let next = b
        .post(
            &s,
            1,
            &request_id().unwrap(),
            "too deep for projection",
            Some(p.id),
        )
        .unwrap();
    assert!(matches!(
        b.room_article(&s, next.id),
        Err(Error::Invalid(_))
    ));
    let other = b.create_room(&s, "other", "Other room").unwrap();
    assert!(
        b.post_article(
            &s,
            other,
            &request_id().unwrap(),
            draft("cross room", "body", Some(p.id))
        )
        .is_err()
    );
    let db = rusqlite::Connection::open(f.0.join("board/faction.sqlite3")).unwrap();
    db.execute("UPDATE posts SET reply_to=id WHERE id=?1", [p.id])
        .unwrap();
    assert!(b.room_article(&s, p.id).is_err());
}
