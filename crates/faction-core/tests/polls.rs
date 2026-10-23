use faction_core::{
    Board, Error, PollDraft, PollResults, PollState, Session, Status, export_local, request_id,
    restore_local, verify_export,
};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-poll-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("faction-polls-{}", request_id().unwrap()));
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
        let mut c = b.poll_config(s).unwrap();
        c.enabled = true;
        c.member_creators = members;
        b.configure_polls(s, &c).unwrap();
    }
    fn member(&self, b: &mut Board, s: &Session, name: &str) -> Session {
        b.register(name, PASS).unwrap();
        b.set_status(s, name, Status::Approved).unwrap();
        b.authenticate(name, PASS).unwrap()
    }
    fn draft(&self) -> PollDraft {
        PollDraft {
            question: "Which night for ANSI club?".into(),
            description: "One choice. Local members only.".into(),
            choices: vec!["Friday".into(), "Saturday".into(), "Sunday".into()],
            results: PollResults::AfterVote,
        }
    }
    fn add(&self, b: &mut Board, s: &Session) -> i64 {
        b.save_poll(s, None, &request_id().unwrap(), &self.draft())
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn optional_policy_current_approval_board_and_epoch_gate_every_action() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    assert!(!b.poll_config(&s).unwrap().enabled);
    assert!(b.polls(&s, false, 0).is_err());
    f.enable(&mut b, &s, false);
    let a = f.member(&mut b, &s, "alice");
    assert!(matches!(
        b.save_poll(&a, None, &request_id().unwrap(), &f.draft()),
        Err(Error::Denied)
    ));
    f.enable(&mut b, &s, true);
    let id = f.add(&mut b, &a);
    b.register("pending", PASS).unwrap();
    let p = b.authenticate("pending", PASS).unwrap();
    assert!(b.poll(&p, id).is_err());
    let config = b.poll_config(&s).unwrap();
    assert!(b.configure_polls(&a, &config).is_err());
    let other = Fixture::new();
    let (mut ob, _) = other.board();
    assert!(ob.poll_config(&s).is_err());
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(b.poll(&a, id).is_err());
    assert!(!b.poll(&s, id).unwrap().creator_approved);
    assert!(b.polls(&p, false, 0).is_err());
}
#[test]
fn bounded_distinct_inert_questions_choices_and_descriptions() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    for choices in [
        vec!["one".into()],
        vec!["same".into(), " SAME ".into()],
        vec!["x".repeat(49), "ok".into()],
        vec!["\x1b[2J".into(), "ok".into()],
        (0..9).map(|n| n.to_string()).collect(),
    ] {
        let mut d = f.draft();
        d.choices = choices;
        assert!(b.save_poll(&s, None, &request_id().unwrap(), &d).is_err());
    }
    let mut d = f.draft();
    d.question = "x".repeat(97);
    assert!(d.validate().is_err());
    d = f.draft();
    d.description = "a\nb\nc\nd\ne".into();
    assert!(d.validate().is_err());
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM polls", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn creation_retry_editor_conflicts_and_first_ballot_permanently_lock_wording() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let key = request_id().unwrap();
    let mut d = f.draft();
    let id = b.save_poll(&s, None, &key, &d).unwrap();
    assert_eq!(b.save_poll(&s, None, &key, &d).unwrap(), id);
    d.question = "New question before voting".into();
    assert!(matches!(
        b.save_poll(&s, None, &key, &d),
        Err(Error::RequestConflict)
    ));
    let old = b.poll(&s, id).unwrap();
    b.save_poll(&s, Some(id), &old.revision, &d).unwrap();
    assert!(matches!(
        b.save_poll(&s, Some(id), &old.revision, &d),
        Err(Error::RequestConflict)
    ));
    let p = b.poll(&s, id).unwrap();
    b.vote_poll(&s, id, &p.revision, 2).unwrap();
    b.vote_poll(&s, id, &p.revision, 0).unwrap();
    assert!(b.poll(&s, id).unwrap().ever_voted);
    assert!(b.save_poll(&s, Some(id), &p.revision, &d).is_err());
    assert_eq!(b.poll(&s, id).unwrap().draft.question, d.question);
}
#[test]
fn result_visibility_is_counts_only_and_applies_to_authors_and_sysops() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, true);
    let a = f.member(&mut b, &s, "alice");
    let z = f.member(&mut b, &s, "bob");
    let id = f.add(&mut b, &s);
    let p = b.poll(&s, id).unwrap();
    assert_eq!(p.counts, None);
    assert_eq!(b.poll(&a, id).unwrap().counts, None);
    b.vote_poll(&a, id, &p.revision, 1).unwrap();
    assert_eq!(b.poll(&a, id).unwrap().counts, Some(vec![1, 0, 0]));
    assert_eq!(b.poll(&s, id).unwrap().counts, None);
    b.vote_poll(&a, id, &p.revision, 1).unwrap();
    b.vote_poll(&a, id, &p.revision, 2).unwrap();
    assert_eq!(b.poll(&a, id).unwrap().counts, Some(vec![0, 1, 0]));
    b.vote_poll(&a, id, &p.revision, 0).unwrap();
    assert_eq!(b.poll(&a, id).unwrap().counts, None);
    let mut d = f.draft();
    d.results = PollResults::AfterClose;
    let closed = b.save_poll(&s, None, &request_id().unwrap(), &d).unwrap();
    let p = b.poll(&s, closed).unwrap();
    b.vote_poll(&a, closed, &p.revision, 3).unwrap();
    assert_eq!(b.poll(&a, closed).unwrap().counts, None);
    b.set_poll_state(&s, closed, &p.revision, PollState::Closed)
        .unwrap();
    assert_eq!(b.poll(&z, closed).unwrap().counts, Some(vec![0, 0, 1]));
    d.results = PollResults::Always;
    let always = b.save_poll(&s, None, &request_id().unwrap(), &d).unwrap();
    let p = b.poll(&a, always).unwrap();
    assert_eq!(p.counts, Some(vec![0, 0, 0]));
    b.vote_poll(&a, always, &p.revision, 1).unwrap();
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert_eq!(b.poll(&z, always).unwrap().counts, Some(vec![0, 0, 0]));
    assert!(b.poll(&a, always).is_err());
}
#[test]
fn closure_cancellation_and_reviewed_ballots_are_transactional_and_final() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let id = f.add(&mut b, &s);
    let p = b.poll(&s, id).unwrap();
    b.vote_poll(&s, id, &p.revision, 1).unwrap();
    b.set_poll_state(&s, id, &p.revision, PollState::Closed)
        .unwrap();
    assert!(matches!(
        b.vote_poll(&s, id, &p.revision, 2),
        Err(Error::RequestConflict)
    ));
    let c = b.poll(&s, id).unwrap();
    assert_eq!(c.own_vote, 1);
    assert!(b.vote_poll(&s, id, &c.revision, 0).is_err());
    assert!(
        b.set_poll_state(&s, id, &c.revision, PollState::Open)
            .is_err()
    );
    assert_eq!(b.polls(&s, true, 0).unwrap()[0].id, id);
    let id = f.add(&mut b, &s);
    let p = b.poll(&s, id).unwrap();
    b.set_poll_state(&s, id, &p.revision, PollState::Cancelled)
        .unwrap();
    let c = b.poll(&s, id).unwrap();
    assert_eq!(c.counts, None);
    assert!(b.vote_poll(&s, id, &c.revision, 1).is_err());
}
#[test]
fn moderation_preserves_authorship_and_hide_survives_own_edits() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, true);
    let a = f.member(&mut b, &s, "alice");
    let z = f.member(&mut b, &s, "bob");
    let id = f.add(&mut b, &a);
    let p = b.poll(&s, id).unwrap();
    assert!(matches!(
        b.save_poll(&s, Some(id), &p.revision, &p.draft),
        Err(Error::Denied)
    ));
    assert!(b.moderate_poll(&z, id, &p.revision, "hide").is_err());
    b.moderate_poll(&s, id, &p.revision, "hide").unwrap();
    assert!(matches!(b.poll(&z, id), Err(Error::NotFound)));
    let p = b.poll(&a, id).unwrap();
    let mut d = p.draft;
    d.question = "Own edit while hidden".into();
    b.save_poll(&a, Some(id), &p.revision, &d).unwrap();
    let p = b.poll(&a, id).unwrap();
    assert!(p.hidden);
    assert!(b.vote_poll(&a, id, &p.revision, 1).is_err());
    b.moderate_poll(&s, id, &p.revision, "show").unwrap();
    let p = b.poll(&z, id).unwrap();
    b.vote_poll(&z, id, &p.revision, 1).unwrap();
    b.moderate_poll(&s, id, &p.revision, "remove").unwrap();
    assert!(b.poll(&s, id).is_err());
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM poll_ballots", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn quotas_and_choice_bounds_limit_storage_without_a_concurrency_claim() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, true);
    let a = f.member(&mut b, &s, "alice");
    for _ in 0..4 {
        f.add(&mut b, &a);
    }
    assert!(
        b.save_poll(&a, None, &request_id().unwrap(), &f.draft())
            .is_err()
    );
    let id = f.add(&mut b, &s);
    let p = b.poll(&s, id).unwrap();
    assert!(b.vote_poll(&s, id, &p.revision, 4).is_err());
    let mut db = f.db();
    let tx = db.transaction().unwrap();
    for n in 0..250 {
        tx.execute("INSERT INTO members(handle,password_hash,role,status,epoch) SELECT ?1,password_hash,'member','approved',1 FROM members WHERE handle='sysop'",[format!("voter{n}")]).unwrap();
        let m = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO poll_ballots VALUES(?1,?2,1)",
            rusqlite::params![id, m],
        )
        .unwrap();
    }
    tx.execute("UPDATE polls SET ever_voted=1 WHERE id=?1", [id])
        .unwrap();
    tx.commit().unwrap();
    assert!(b.vote_poll(&s, id, &p.revision, 1).is_err());
    for _ in 5..32 {
        f.add(&mut b, &s);
    }
    assert!(
        b.save_poll(&s, None, &request_id().unwrap(), &f.draft())
            .is_err()
    );
}
#[test]
fn busy_readers_remain_available_while_votes_refuse_without_unread_changes() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let id = f.add(&mut b, &s);
    let p = b.poll(&s, id).unwrap();
    let lock = f.db();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    assert_eq!(b.poll(&s, id).unwrap().id, id);
    assert_eq!(b.polls(&s, false, 0).unwrap().len(), 1);
    assert!(matches!(
        b.vote_poll(&s, id, &p.revision, 1),
        Err(Error::Busy)
    ));
    lock.execute_batch("ROLLBACK").unwrap();
    b.vote_poll(&s, id, &p.revision, 1).unwrap();
    assert_eq!(b.poll(&s, id).unwrap().counts, Some(vec![1, 0, 0]));
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM unread", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn current_export_ballots_and_exact_readonly_schema_eleven_recovery() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let id = f.add(&mut b, &s);
    let p = b.poll(&s, id).unwrap();
    b.vote_poll(&s, id, &p.revision, 2).unwrap();
    drop(b);
    let exported = f.0.join("export");
    export_local(&f.path(), &exported).unwrap();
    verify_export(&exported).unwrap();
    let restored = f.0.join("restored");
    restore_local(&exported, &restored).unwrap();
    let mut b = Board::open(&restored).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert_eq!(b.poll(&s, id).unwrap().own_vote, 2);
    drop(b);
    let db = f.db();
    db.execute_batch("PRAGMA foreign_keys=OFF; DROP TRIGGER catchup_file_insert; DROP TRIGGER catchup_file_publish; DROP TABLE catchup_favorites; DROP TABLE catchup_members; DROP TABLE catchup_settings; DROP TABLE catchup_publications; DROP TABLE poll_ballots; DROP TABLE poll_choices; DROP TABLE polls; DROP TABLE poll_settings; PRAGMA user_version=11;").unwrap();
    drop(db);
    let old = f.0.join("old");
    export_local(&f.path(), &old).unwrap();
    verify_export(&old).unwrap();
    assert_eq!(
        f.db()
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        11
    );
    let restored = f.0.join("old-restored");
    restore_local(&old, &restored).unwrap();
    let mut b = Board::open(&restored).unwrap();
    let s = b.authenticate("sysop", PASS).unwrap();
    assert!(!b.poll_config(&s).unwrap().enabled);
}
