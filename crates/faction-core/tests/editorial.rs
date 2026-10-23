use faction_core::{
    Board, Error, MagazineIssueDraft, MagazineText, Session, Status, export_local, request_id,
    restore_local, verify_export,
};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-poll-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("faction-editorial-{}", request_id().unwrap()));
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

fn text() -> MagazineText {
    MagazineText {
        title: "Original tutorial".into(),
        section: "Field notes".into(),
        rights: "BSD-3-Clause original work".into(),
        provenance: "Written by the authenticated fixture author".into(),
        body: "ORIGINAL-CONTRIBUTION\nFinal manuscript line.".into(),
    }
}
fn header(n: u16) -> MagazineIssueDraft {
    MagazineIssueDraft {
        number: n,
        edition: 1,
        name: "Nightwire".into(),
        title: "Community edition".into(),
        released: "Autumn 2026".into(),
        schedule: "When ready".into(),
    }
}
fn create(b: &mut Board, s: &Session) -> i64 {
    b.magazine_save_submission(s, None, &request_id().unwrap(), &text(), true)
        .unwrap()
}
fn issue(b: &mut Board, s: &Session, n: u16) -> i64 {
    b.magazine_save_issue(s, None, &request_id().unwrap(), &header(n))
        .unwrap()
}
fn token(b: &mut Board, s: &Session, id: i64) -> String {
    b.magazine_issue(s, id).unwrap().token
}
fn select(b: &mut Board, s: &Session, id: i64, slot: u8, sub: i64) {
    let t = token(b, s, id);
    b.magazine_select(s, id, &t, slot, sub).unwrap();
}
fn approve(b: &mut Board, s: &Session, id: i64) {
    let t = token(b, s, id);
    b.magazine_approve(s, id, &t, true).unwrap();
}
fn publish(b: &mut Board, s: &Session, id: i64) {
    let t = token(b, s, id);
    b.magazine_publish(s, id, &t).unwrap();
}
#[test]
fn private_drafts_submit_retry_owner_editor_and_selection_boundaries() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    let a = f.member(&mut b, &s, "alice");
    let bob = f.member(&mut b, &s, "bob");
    let nonce = request_id().unwrap();
    let sub = b
        .magazine_save_submission(&a, None, &nonce, &text(), false)
        .unwrap();
    assert_eq!(
        b.magazine_save_submission(&a, None, &nonce, &text(), false)
            .unwrap(),
        sub
    );
    assert!(b.magazine_submission(&bob, sub).is_err());
    assert!(b.magazine_submissions(&bob, true, i64::MAX).is_err());
    assert!(b.magazine_save_issue(&a, None, &nonce, &header(1)).is_err());
    let id = issue(&mut b, &s, 1);
    let t = token(&mut b, &s, id);
    assert!(b.magazine_select(&s, id, &t, 1, sub).is_err());
    let rev = b.magazine_submission(&a, sub).unwrap().revision;
    b.magazine_save_submission(&a, Some(sub), &rev, &text(), true)
        .unwrap();
    select(&mut b, &s, id, 1, sub);
    assert!(b.magazine_issue(&bob, id).is_err());
    assert!(b.magazine_issues(&bob, i64::MAX).unwrap().is_empty());
    assert_eq!(b.magazine_issue(&a, id).unwrap().entries[0].author, "alice");
    let token = token(&mut b, &s, id);
    assert!(b.magazine_approve(&s, id, &token, true).is_err());
    let rev = b.magazine_submission(&s, sub).unwrap().revision;
    assert!(
        b.magazine_save_submission(&s, Some(sub), &rev, &text(), true)
            .is_err()
    );
}
#[test]
fn exact_issue_and_proposal_edits_invalidate_all_authors_until_reapproval() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    let a = f.member(&mut b, &s, "alice");
    let bob = f.member(&mut b, &s, "bob");
    let outsider = f.member(&mut b, &s, "reader");
    let sa = create(&mut b, &a);
    let sb = create(&mut b, &bob);
    let id = issue(&mut b, &s, 1);
    select(&mut b, &s, id, 1, sa);
    select(&mut b, &s, id, 2, sb);
    let stale = token(&mut b, &s, id);
    approve(&mut b, &a, id);
    assert!(b.magazine_publish(&s, id, &stale).is_err());
    approve(&mut b, &bob, id);
    let mut t = text();
    t.body = "EDITOR-REVISED-PROPOSAL".into();
    b.magazine_edit_entry(&s, id, &stale, 1, &t).unwrap();
    assert!(b.magazine_approve(&a, id, &stale, true).is_err());
    let v = b.magazine_issue(&a, id).unwrap();
    assert!(v.entries.iter().all(|e| !e.approved));
    assert_eq!(v.entries[0].author, "alice");
    approve(&mut b, &a, id);
    approve(&mut b, &bob, id);
    let t = token(&mut b, &s, id);
    let mut h = header(1);
    h.title = "Reviewed new title".into();
    b.magazine_save_issue(&s, Some(id), &t, &h).unwrap();
    assert!(
        b.magazine_issue(&s, id)
            .unwrap()
            .entries
            .iter()
            .all(|e| !e.approved)
    );
    approve(&mut b, &a, id);
    approve(&mut b, &bob, id);
    publish(&mut b, &s, id);
    let v = b.magazine_issue(&outsider, id).unwrap();
    assert!(v.text().contains("EDITOR-REVISED-PROPOSAL"));
    assert!(v.text().contains("Reviewed new title"));
    assert!(b.magazine_edit_entry(&s, id, &v.token, 1, &text()).is_err());
    assert!(
        b.magazine_save_issue(&s, Some(id), &v.token, &header(1))
            .is_err()
    );
    assert!(b.magazine_select(&s, id, &v.token, 1, sa).is_err());
    assert_eq!(b.unread_cursor(&outsider, 1).unwrap(), 0);
}
#[test]
fn source_changes_drafts_withdrawal_and_published_snapshot_retention() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    let a = f.member(&mut b, &s, "alice");
    let reader = f.member(&mut b, &s, "reader");
    let sub = create(&mut b, &a);
    let id = issue(&mut b, &s, 1);
    select(&mut b, &s, id, 1, sub);
    let rev = b.magazine_submission(&a, sub).unwrap().revision;
    let mut changed = text();
    changed.body = "New source revision".into();
    b.magazine_save_submission(&a, Some(sub), &rev, &changed, true)
        .unwrap();
    let t = token(&mut b, &s, id);
    assert!(b.magazine_approve(&a, id, &t, true).is_err());
    select(&mut b, &s, id, 1, sub);
    approve(&mut b, &a, id);
    publish(&mut b, &s, id);
    let old = b.magazine_issue(&reader, id).unwrap().text();
    let rev = b.magazine_submission(&a, sub).unwrap().revision;
    b.magazine_save_submission(&a, Some(sub), &rev, &text(), false)
        .unwrap();
    assert_eq!(b.magazine_issue(&reader, id).unwrap().text(), old);
    let rev = b.magazine_submission(&a, sub).unwrap().revision;
    b.magazine_withdraw_submission(&a, sub, &rev, false)
        .unwrap();
    assert!(b.magazine_issue(&reader, id).is_err());
    assert_eq!(b.magazine_issue(&s, id).unwrap().state, "withdrawn");
    let rev = b.magazine_submission(&a, sub).unwrap().revision;
    assert!(b.magazine_withdraw_submission(&a, sub, &rev, true).is_err());
}
#[test]
fn revocation_role_sessions_foreign_boards_and_approval_withdrawal_are_current() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    let a = f.member(&mut b, &s, "alice");
    let reader = f.member(&mut b, &s, "reader");
    let sub = create(&mut b, &a);
    let id = issue(&mut b, &s, 1);
    select(&mut b, &s, id, 1, sub);
    approve(&mut b, &a, id);
    publish(&mut b, &s, id);
    let t = token(&mut b, &a, id);
    b.magazine_approve(&a, id, &t, false).unwrap();
    assert!(b.magazine_issue(&reader, id).is_err());
    assert_eq!(b.magazine_issue(&s, id).unwrap().state, "withdrawn");
    assert!(b.magazine_approve(&a, id, &t, true).is_err());
    let id2 = issue(&mut b, &s, 2);
    select(&mut b, &s, id2, 1, sub);
    approve(&mut b, &a, id2);
    publish(&mut b, &s, id2);
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(b.magazine_issue(&reader, id2).is_err());
    assert!(b.magazine_issue(&a, id2).is_err());
    assert!(b.magazine_submission(&a, sub).is_err());
    let f2 = Fixture::new();
    let (mut other, _) = f2.board();
    assert!(other.magazine_issues(&s, i64::MAX).is_err());
}
#[test]
fn busy_retry_bounds_quota_paging_and_explicit_live_retention() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    let lock = f.db();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    let n = request_id().unwrap();
    assert!(matches!(
        b.magazine_save_submission(&s, None, &n, &text(), true),
        Err(Error::Busy)
    ));
    lock.execute_batch("ROLLBACK").unwrap();
    let sub = b
        .magazine_save_submission(&s, None, &n, &text(), true)
        .unwrap();
    assert_eq!(
        b.magazine_save_submission(&s, None, &n, &text(), true)
            .unwrap(),
        sub
    );
    for _ in 0..7 {
        create(&mut b, &s);
    }
    assert!(
        b.magazine_save_submission(&s, None, &request_id().unwrap(), &text(), true)
            .is_err()
    );
    let rows = b.magazine_submissions(&s, false, i64::MAX).unwrap();
    assert_eq!(rows.len(), 4);
    assert_eq!(
        b.magazine_submissions(&s, false, rows[3].id - 1)
            .unwrap()
            .len(),
        4
    );
    let v = b.magazine_submission(&s, sub).unwrap();
    b.magazine_withdraw_submission(&s, sub, &v.revision, true)
        .unwrap();
    create(&mut b, &s);
    for n in 1..=32 {
        issue(&mut b, &s, n);
    }
    assert!(
        b.magazine_save_issue(&s, None, &request_id().unwrap(), &header(33))
            .is_err()
    );
    let rows = b.magazine_issues(&s, i64::MAX).unwrap();
    assert_eq!(rows.len(), 4);
    let v = b.magazine_issue(&s, rows[0].id).unwrap();
    b.magazine_withdraw_issue(&s, v.id, &v.token, true).unwrap();
    issue(&mut b, &s, 33);
    assert!(b.magazine_issues(&s, -1).is_err());
}
#[test]
fn snapshot_export_owner_edition_and_readonly_schema_fifteen_recovery() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    let sub = create(&mut b, &s);
    let id = issue(&mut b, &s, 1);
    select(&mut b, &s, id, 1, sub);
    approve(&mut b, &s, id);
    publish(&mut b, &s, id);
    let payload = b.magazine_issue(&s, id).unwrap().text();
    drop(b);
    let edition = f.0.join("edition");
    faction_core::export_magazine_local(&f.path(), id, &edition).unwrap();
    assert_eq!(fs::read_to_string(&edition).unwrap(), payload);
    assert_eq!(
        fs::metadata(&edition).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(faction_core::export_magazine_local(&f.path(), id, &edition).is_err());
    let out = f.0.join("export");
    export_local(&f.path(), &out).unwrap();
    verify_export(&out).unwrap();
    let restored = f.0.join("restored");
    restore_local(&out, &restored).unwrap();
    let mut b = Board::open(&restored).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert_eq!(b.magazine_issue(&s, id).unwrap().text(), payload);
    drop(b);
    f.db().execute_batch("DROP TABLE magazine_entries;DROP TABLE magazine_issues;DROP TABLE magazine_submissions;PRAGMA user_version=15").unwrap();
    let out = f.0.join("old");
    export_local(&f.path(), &out).unwrap();
    verify_export(&out).unwrap();
    assert_eq!(
        f.db()
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        15
    );
    let dest = f.0.join("old-restored");
    restore_local(&out, &dest).unwrap();
    let mut b = Board::open(&dest).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert!(b.magazine_issues(&s, i64::MAX).unwrap().is_empty());
}
#[test]
fn export_rejects_forged_publication_approval_and_inert_text_failures() {
    for sql in [
        "UPDATE magazine_entries SET approved=NULL",
        "UPDATE magazine_entries SET body=char(27)||'[2J'",
        "UPDATE magazine_entries SET source_revision='ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ'",
    ] {
        let f = Fixture::new();
        let (mut b, s) = f.board();
        let sub = create(&mut b, &s);
        let id = issue(&mut b, &s, 1);
        select(&mut b, &s, id, 1, sub);
        approve(&mut b, &s, id);
        publish(&mut b, &s, id);
        drop(b);
        f.db().execute_batch(sql).unwrap();
        assert!(export_local(&f.path(), &f.0.join("bad")).is_err());
    }
    for body in ["", "\x1b[2J", "faction-file:private", "bad\rline"] {
        let mut t = text();
        t.body = body.into();
        assert!(t.validate().is_err());
    }
    let mut t = text();
    t.body = "X".repeat(121);
    assert!(t.validate().is_err());
}
