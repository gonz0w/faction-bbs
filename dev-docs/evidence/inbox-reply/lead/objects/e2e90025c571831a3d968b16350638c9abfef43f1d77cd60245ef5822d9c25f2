use faction_core::{
    Board, Error, GatheringDraft, GatheringPlace, Session, Status, export_local, request_id,
    restore_local, verify_export,
};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-gathering-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("faction-gatherings-{}", request_id().unwrap()));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        Self(p)
    }
    fn path(&self) -> PathBuf {
        self.0.join("board")
    }
    fn board(&self) -> (Board, Session) {
        let mut b = Board::open(&self.path()).unwrap();
        b.bootstrap("sysop", PASS).unwrap();
        let s = b.authenticate("sysop", PASS).unwrap();
        (b, s)
    }
    fn db(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(self.path().join("faction.sqlite3")).unwrap()
    }
    fn draft(&self) -> GatheringDraft {
        let starts = self
            .db()
            .query_row(
                "SELECT strftime('%Y-%m-%d %H:%M','now','+1 hour')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        GatheringDraft {
            title: "Afterhours terminal night".into(),
            body: "Bring your ANSI art.\nLocal members only.".into(),
            starts_utc: starts,
            minutes: 120,
            place: "room:1".into(),
        }
    }
    fn enable(&self, b: &mut Board, s: &Session, members: bool) {
        let mut c = b.gathering_config(s).unwrap();
        c.enabled = true;
        c.member_organizers = members;
        b.configure_gatherings(s, &c).unwrap()
    }
    fn member(&self, b: &mut Board, s: &Session, name: &str) -> Session {
        b.register(name, PASS).unwrap();
        b.set_status(s, name, Status::Approved).unwrap();
        b.authenticate(name, PASS).unwrap()
    }
    fn add(&self, b: &mut Board, s: &Session) -> i64 {
        b.save_gathering(s, None, &request_id().unwrap(), &self.draft())
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn default_disabled_current_local_approval_and_epoch_are_required() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    assert!(!b.gathering_config(&s).unwrap().enabled);
    assert!(b.gatherings(&s, false, 0).is_err());
    assert_eq!(b.gathering_reminder(&s).unwrap(), None);
    f.enable(&mut b, &s, true);
    let a = f.member(&mut b, &s, "alice");
    let id = f.add(&mut b, &a);
    b.register("pending", PASS).unwrap();
    let p = b.authenticate("pending", PASS).unwrap();
    assert!(b.gathering(&p, id).is_err());
    let other = Fixture::new();
    let (mut ob, _) = other.board();
    assert!(ob.gathering_config(&a).is_err());
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(b.gathering(&a, id).is_err());
    assert!(!b.gatherings(&s, false, 0).unwrap()[0].organizer_approved);
    let g = b.gathering(&s, id).unwrap();
    assert!(b.gathering_rsvp(&s, id, &g.revision, 1).is_err());
    assert!(b.gathering_destination(&s, id).is_err());
}
#[test]
fn calendar_inputs_are_explicit_utc_and_round_trip_invalid_dates() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, true);
    for value in [
        "2026-02-30 12:00",
        "2026-13-01 12:00",
        "2026-11-01 24:00",
        "2026-11-01 01:30-0600",
        "2026-11-01T01:30",
        "2026-1-01 10:00",
        "2100-01-01 00:00",
    ] {
        let mut d = f.draft();
        d.starts_utc = value.into();
        assert!(
            b.save_gathering(&s, None, &request_id().unwrap(), &d)
                .is_err(),
            "{value}"
        );
    }
    let mut d = f.draft();
    d.starts_utc = f
        .db()
        .query_row(
            "SELECT strftime('%Y-%m-%d %H:%M','now','+1 day','start of day')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let id = b
        .save_gathering(&s, None, &request_id().unwrap(), &d)
        .unwrap();
    let g = b.gathering(&s, id).unwrap();
    assert_eq!(g.draft.starts_utc, d.starts_utc);
    assert!(g.ends_utc.ends_with("02:00"));
    d.minutes = 0;
    assert!(
        b.save_gathering(&s, None, &request_id().unwrap(), &d)
            .is_err()
    );
    d.minutes = 1441;
    assert!(
        b.save_gathering(&s, None, &request_id().unwrap(), &d)
            .is_err()
    );
}
#[test]
fn creation_retry_is_exact_and_bounded_by_retained_events() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let d = f.draft();
    let key = request_id().unwrap();
    let id = b.save_gathering(&s, None, &key, &d).unwrap();
    assert_eq!(b.save_gathering(&s, None, &key, &d).unwrap(), id);
    let mut changed = d.clone();
    changed.title = "Different".into();
    assert!(matches!(
        b.save_gathering(&s, None, &key, &changed),
        Err(Error::RequestConflict)
    ));
    for _ in 1..64 {
        f.add(&mut b, &s);
    }
    assert!(
        b.save_gathering(&s, None, &request_id().unwrap(), &d)
            .is_err()
    );
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM gatherings", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        64
    );
    // Simulate elapsed time in the owned fixture: ended details stay in past,
    // while >30-day events and their responses disappear lazily and free quota.
    let old = b.gathering(&s, 2).unwrap();
    b.gathering_rsvp(&s, 2, &old.revision, 1).unwrap();
    f.db().execute("UPDATE gatherings SET starts_at=(unixepoch()/60-120)*60,ends_at=(unixepoch()/60-60)*60 WHERE id=1", []).unwrap();
    let ended = b.gathering(&s, 1).unwrap();
    assert!(ended.ended);
    assert!(b.gathering_rsvp(&s, 1, &ended.revision, 1).is_err());
    assert!(b.gathering_destination(&s, 1).is_err());
    assert_eq!(b.gatherings(&s, true, 0).unwrap()[0].id, 1);
    f.db().execute("UPDATE gatherings SET starts_at=(unixepoch()/60-46080)*60,ends_at=(unixepoch()/60-46020)*60 WHERE id=2", []).unwrap();
    assert!(matches!(b.gathering(&s, 2), Err(Error::NotFound)));
    // NotFound rolls its transaction back; the next successful list commits maintenance.
    b.gatherings(&s, true, 0).unwrap();
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM gatherings", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        63
    );
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM gathering_rsvps", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    f.add(&mut b, &s);
    let lock = f.db();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    assert!(!b.gatherings(&s, false, 0).unwrap().is_empty());
    assert_eq!(b.gathering(&s, 1).unwrap().id, 1);
    assert!(matches!(
        b.save_gathering(&s, None, &request_id().unwrap(), &d),
        Err(Error::Busy)
    ));
    lock.execute_batch("ROLLBACK").unwrap();
}

#[test]
fn organizer_policy_and_edit_revision_cannot_rewrite_other_authors() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    let a = f.member(&mut b, &s, "alice");
    f.enable(&mut b, &s, false);
    assert!(matches!(
        b.save_gathering(&a, None, &request_id().unwrap(), &f.draft()),
        Err(Error::Denied)
    ));
    f.enable(&mut b, &s, true);
    let id = f.add(&mut b, &a);
    let g = b.gathering(&a, id).unwrap();
    assert!(matches!(
        b.save_gathering(&s, Some(id), &g.revision, &g.draft),
        Err(Error::Denied)
    ));
    let mut d = g.draft.clone();
    d.title = "Revised".into();
    b.save_gathering(&a, Some(id), &g.revision, &d).unwrap();
    assert!(matches!(
        b.save_gathering(&a, Some(id), &g.revision, &d),
        Err(Error::RequestConflict)
    ));
    let g = b.gathering(&a, id).unwrap();
    f.enable(&mut b, &s, false);
    assert!(b.save_gathering(&a, Some(id), &g.revision, &d).is_err());
    b.cancel_gathering(&a, id, &g.revision).unwrap();
}
#[test]
fn rsvp_upsert_change_clear_and_private_counts_never_list_attendees() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let a = f.member(&mut b, &s, "alice");
    let id = f.add(&mut b, &s);
    let g = b.gathering(&a, id).unwrap();
    for _ in 0..2 {
        b.gathering_rsvp(&a, id, &g.revision, 1).unwrap();
    }
    let g = b.gathering(&a, id).unwrap();
    assert_eq!(g.counts, Some((1, 0)));
    assert_eq!(g.response, 1);
    assert_eq!(b.gathering(&s, id).unwrap().response, 0);
    b.gathering_rsvp(&a, id, &g.revision, 2).unwrap();
    assert_eq!(b.gathering(&a, id).unwrap().counts, Some((0, 1)));
    let mut c = b.gathering_config(&s).unwrap();
    c.public_counts = false;
    b.configure_gatherings(&s, &c).unwrap();
    assert_eq!(b.gathering(&a, id).unwrap().counts, None);
    assert_eq!(b.gathering(&s, id).unwrap().counts, Some((0, 1)));
    b.gathering_rsvp(&a, id, &g.revision, 0).unwrap();
    assert_eq!(b.gathering(&a, id).unwrap().response, 0);
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM gathering_rsvps", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn changing_schedule_or_destination_clears_responses_and_stale_votes_fail() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let a = f.member(&mut b, &s, "alice");
    let id = f.add(&mut b, &s);
    let g = b.gathering(&a, id).unwrap();
    b.gathering_rsvp(&a, id, &g.revision, 1).unwrap();
    let mut d = g.draft.clone();
    d.minutes += 1;
    b.save_gathering(&s, Some(id), &g.revision, &d).unwrap();
    assert!(matches!(
        b.gathering_rsvp(&a, id, &g.revision, 1),
        Err(Error::RequestConflict)
    ));
    assert_eq!(b.gathering(&a, id).unwrap().response, 0);
    let g = b.gathering(&s, id).unwrap();
    b.cancel_gathering(&s, id, &g.revision).unwrap();
    let g = b.gathering(&a, id).unwrap();
    assert!(g.cancelled);
    assert!(b.gathering_rsvp(&a, id, &g.revision, 1).is_err());
    assert!(b.gathering_destination(&a, id).is_err());
}
#[test]
fn private_channel_metadata_and_jump_recheck_current_grants() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let a = f.member(&mut b, &s, "alice");
    b.chat_create(&s, "#private", true).unwrap();
    let mut d = f.draft();
    d.place = "#private".into();
    let id = b
        .save_gathering(&s, None, &request_id().unwrap(), &d)
        .unwrap();
    assert_eq!(b.gathering(&a, id).unwrap().draft.place, "");
    assert!(b.gathering_destination(&a, id).is_err());
    b.chat_allow(&s, "#private", "alice", true).unwrap();
    assert_eq!(
        b.gathering_destination(&a, id).unwrap(),
        GatheringPlace::Channel("#private".into())
    );
    b.chat_allow(&s, "#private", "alice", false).unwrap();
    assert!(b.gathering_destination(&a, id).is_err());
    assert_eq!(b.gathering(&a, id).unwrap().draft.place, "");
}
#[test]
fn moderation_is_independent_of_author_edits_and_uses_revision_and_stable_id() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, true);
    let a = f.member(&mut b, &s, "alice");
    let bob = f.member(&mut b, &s, "bobby");
    let id = f.add(&mut b, &a);
    let g = b.gathering(&s, id).unwrap();
    b.moderate_gathering(&s, id, &g.revision, "hide").unwrap();
    assert!(b.gathering(&bob, id).is_err());
    let g = b.gathering(&a, id).unwrap();
    let mut d = g.draft.clone();
    d.title = "Still hidden".into();
    b.save_gathering(&a, Some(id), &g.revision, &d).unwrap();
    assert!(b.gathering(&a, id).unwrap().hidden);
    assert!(b.gathering(&bob, id).is_err());
    let g = b.gathering(&s, id).unwrap();
    b.moderate_gathering(&s, id, &g.revision, "show").unwrap();
    assert!(b.gathering(&bob, id).is_ok());
    let g = b.gathering(&s, id).unwrap();
    b.moderate_gathering(&s, id, &g.revision, "remove").unwrap();
    assert!(b.gathering(&a, id).is_err());
}
#[test]
fn member_and_response_quotas_are_bounded_without_exercising_capacity_claims() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, true);
    let a = f.member(&mut b, &s, "alice");
    for _ in 0..8 {
        f.add(&mut b, &a);
    }
    assert!(
        b.save_gathering(&a, None, &request_id().unwrap(), &f.draft())
            .is_err()
    );
    let id = f.add(&mut b, &s);
    let mut db = f.db();
    let tx = db.transaction().unwrap();
    for n in 0..250 {
        tx.execute("INSERT INTO members(handle,password_hash,role,status,epoch) SELECT ?1,password_hash,'member','approved',1 FROM members WHERE handle='sysop'",[format!("fixture{n}")]).unwrap();
        let member = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO gathering_rsvps VALUES(?1,?2,1)",
            rusqlite::params![id, member],
        )
        .unwrap();
    }
    tx.commit().unwrap();
    let g = b.gathering(&a, id).unwrap();
    assert_eq!(g.counts, Some((250, 0)));
    assert!(b.gathering_rsvp(&a, id, &g.revision, 1).is_err());
    assert!(
        f.db()
            .query_row("SELECT count(*) FROM gathering_rsvps", [], |r| r
                .get::<_, i64>(0))
            .unwrap()
            == 250
    );
}
#[test]
fn reminders_are_opt_in_visible_only_and_do_not_change_unread() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, false);
    let id = f.add(&mut b, &s);
    assert_eq!(b.gathering_reminder(&s).unwrap(), None);
    let mut c = b.gathering_config(&s).unwrap();
    c.reminders = true;
    b.configure_gatherings(&s, &c).unwrap();
    assert!(b.gathering_reminder(&s).unwrap().unwrap().contains("UTC"));
    let g = b.gathering(&s, id).unwrap();
    b.moderate_gathering(&s, id, &g.revision, "hide").unwrap();
    assert_eq!(b.gathering_reminder(&s).unwrap(), None);
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM unread", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn export_restore_preserves_gathering_policy_and_responses_and_upgrades_old_ten() {
    let f = Fixture::new();
    let (mut b, s) = f.board();
    f.enable(&mut b, &s, true);
    let a = f.member(&mut b, &s, "alice");
    let id = f.add(&mut b, &s);
    let g = b.gathering(&a, id).unwrap();
    b.gathering_rsvp(&a, id, &g.revision, 2).unwrap();
    let exported = f.0.join("export");
    export_local(&f.path(), &exported).unwrap();
    verify_export(&exported).unwrap();
    let restored = f.0.join("restored");
    restore_local(&exported, &restored).unwrap();
    let mut rb = Board::open(&restored).unwrap();
    let rs = rb.authenticate("alice", PASS).unwrap();
    assert_eq!(rb.gathering(&rs, id).unwrap().response, 2);
    let legacy = f.0.join("legacy");
    fs::copy(&exported, &legacy).unwrap();
    let db = rusqlite::Connection::open(&legacy).unwrap();
    db.execute_batch("DROP TABLE roaming_chat;DROP TABLE roaming_transfers;DROP TABLE roaming_bindings;DROP TABLE roaming_issuers;DROP TABLE directory_submissions;DROP TABLE visit_activity;DROP TABLE social_pages;DROP TABLE social_presence;DROP TABLE social_blocks;DROP TABLE social_prefs;DROP TABLE social_cooldowns;DROP TABLE social_settings;DROP TABLE magazine_entries;DROP TABLE magazine_issues;DROP TABLE magazine_submissions;DROP TABLE file_requests;DROP INDEX discussion_author_posts;DROP TABLE discussion_bookmarks;DROP TABLE discussion_settings;DROP TRIGGER catchup_file_insert; DROP TRIGGER catchup_file_publish; DROP TABLE catchup_favorites; DROP TABLE catchup_members; DROP TABLE catchup_settings; DROP TABLE catchup_publications; DROP TABLE poll_ballots; DROP TABLE poll_choices; DROP TABLE polls; DROP TABLE poll_settings; DROP TABLE gathering_rsvps; DROP TABLE gatherings; DROP TABLE gathering_settings; PRAGMA user_version=10;").unwrap();
    drop(db);
    let bytes = fs::read(&legacy).unwrap();
    verify_export(&legacy).unwrap();
    assert_eq!(fs::read(&legacy).unwrap(), bytes);
    let dest = f.0.join("legacy-restored");
    restore_local(&legacy, &dest).unwrap();
    let mut lb = Board::open(&dest).unwrap();
    let ls = lb.authenticate("alice", PASS).unwrap();
    assert!(!lb.gathering_config(&ls).unwrap().enabled);
}
