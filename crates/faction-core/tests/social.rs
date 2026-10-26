use faction_core::{
    Board, ChatLease, Error, Session, SocialConfig, Status, export_local, request_id,
    restore_local, verify_export,
};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-social-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("faction-social-{}", request_id().unwrap()));
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
    fn board(&self) -> (Board, Session, ChatLease) {
        let mut b = Board::open(&self.path()).unwrap();
        b.bootstrap("sysop", PASS).unwrap();
        let s = b.authenticate("sysop", PASS).unwrap();
        let l = b.chat_open(&s).unwrap();
        (b, s, l)
    }
    fn member(&self, b: &mut Board, s: &Session, name: &str) -> (Session, ChatLease) {
        b.register(name, PASS).unwrap();
        b.set_status(s, name, Status::Approved).unwrap();
        let m = b.authenticate(name, PASS).unwrap();
        let l = b.chat_open(&m).unwrap();
        (m, l)
    }
    fn enable(&self, b: &mut Board, l: &ChatLease) {
        b.social_configure(
            l,
            &SocialConfig {
                enabled: true,
                name: "Original lounge".into(),
            },
        )
        .unwrap();
    }
    fn ready(&self, b: &mut Board, l: &ChatLease) {
        b.social_availability(l, "available", "Original public status")
            .unwrap();
    }
    fn reset_rate(&self) {
        self.db()
            .execute("DELETE FROM social_cooldowns", [])
            .unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn default_hidden_explicit_optin_and_current_board_approval() {
    let f = Fixture::new();
    let (mut b, s, op) = f.board();
    let (_, a) = f.member(&mut b, &s, "alice");
    let (_, bob) = f.member(&mut b, &s, "bob");
    assert!(!b.social_config(&op).unwrap().enabled);
    assert!(b.social_roster(&a, None, false).is_err());
    assert!(
        b.social_configure(
            &a,
            &SocialConfig {
                enabled: true,
                name: "Wrong".into()
            }
        )
        .is_err()
    );
    f.enable(&mut b, &op);
    assert!(
        b.social_roster(&bob, None, false)
            .unwrap()
            .people
            .is_empty()
    );
    f.ready(&mut b, &a);
    let v = b.social_roster(&bob, None, false).unwrap();
    assert_eq!(v.people.len(), 1);
    assert_eq!(v.people[0].handle, "alice");
    assert!(v.next.is_none());
    b.social_availability(&a, "busy", "Working on a draft")
        .unwrap();
    assert_eq!(
        b.social_roster(&bob, None, false).unwrap().people[0].mode,
        "busy"
    );
    b.social_availability(&a, "hidden", "").unwrap();
    assert!(
        b.social_roster(&bob, None, false)
            .unwrap()
            .people
            .is_empty()
    );
    f.ready(&mut b, &a);
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(
        b.social_roster(&bob, None, false)
            .unwrap()
            .people
            .is_empty()
    );
    assert!(b.social_roster(&a, None, false).is_err());
}
#[test]
fn bodyless_page_response_is_recipient_only_and_never_changes_channels() {
    let f = Fixture::new();
    let (mut b, s, op) = f.board();
    let (_, a) = f.member(&mut b, &s, "alice");
    let (_, bob) = f.member(&mut b, &s, "bob");
    f.enable(&mut b, &op);
    f.ready(&mut b, &a);
    f.ready(&mut b, &bob);
    b.chat_create(&s, "#other", false).unwrap();
    b.chat_join(&a, "#lobby").unwrap();
    b.chat_join(&bob, "#other").unwrap();
    let n = request_id().unwrap();
    b.social_request(&a, "bob", &n, false).unwrap();
    b.social_request(&a, "bob", &n, false).unwrap();
    assert!(b.social_request(&a, "sysop", &n, false).is_err());
    let rows = b.social_pages(&bob).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].from, "alice");
    assert_eq!(rows[0].state, "pending");
    let id = rows[0].id;
    assert!(b.social_answer(&a, id, true).is_err());
    assert!(b.social_answer(&op, id, true).is_err());
    b.social_answer(&bob, id, true).unwrap();
    assert_eq!(b.social_pages(&a).unwrap()[0].state, "accepted");
    assert!(b.social_answer(&bob, id, false).is_err());
    assert_eq!(b.chat_names(&a, "#lobby", None).unwrap().handles, ["alice"]);
    assert_eq!(b.chat_names(&bob, "#other", None).unwrap().handles, ["bob"]);
}
#[test]
fn blocked_hidden_muted_quiet_unknown_and_non_sysop_targets_do_not_leak() {
    let f = Fixture::new();
    let (mut b, s, op) = f.board();
    let (_, a) = f.member(&mut b, &s, "alice");
    let (_, bob) = f.member(&mut b, &s, "bob");
    f.enable(&mut b, &op);
    f.ready(&mut b, &a);
    f.ready(&mut b, &bob);
    b.social_block(&bob, "ALICE", true).unwrap();
    assert!(
        b.social_roster(&a, None, false)
            .unwrap()
            .people
            .iter()
            .all(|p| p.handle != "bob")
    );
    b.social_request(&a, "bob", &request_id().unwrap(), false)
        .unwrap();
    assert!(b.social_pages(&a).unwrap().is_empty());
    assert!(b.social_pages(&bob).unwrap().is_empty());
    f.reset_rate();
    b.social_request(&a, "unknown", &request_id().unwrap(), false)
        .unwrap();
    assert!(b.social_pages(&a).unwrap().is_empty());
    f.reset_rate();
    b.social_block(&bob, "alice", false).unwrap();
    b.social_preferences(&bob, true, None).unwrap();
    b.social_request(&a, "bob", &request_id().unwrap(), false)
        .unwrap();
    assert!(b.social_pages(&a).unwrap().is_empty());
    f.reset_rate();
    let minute: i64 = f
        .db()
        .query_row("SELECT (unixepoch()/60)%1440", [], |r| r.get(0))
        .unwrap();
    b.social_preferences(
        &bob,
        false,
        Some((minute as u16, ((minute + 2) % 1440) as u16)),
    )
    .unwrap();
    b.social_request(&a, "bob", &request_id().unwrap(), false)
        .unwrap();
    assert!(b.social_pages(&a).unwrap().is_empty());
    f.reset_rate();
    b.social_preferences(&bob, false, None).unwrap();
    b.social_request(&a, "bob", &request_id().unwrap(), true)
        .unwrap();
    assert!(b.social_pages(&a).unwrap().is_empty());
    f.reset_rate();
    assert!(b.social_roster(&a, None, true).unwrap().people.is_empty());
    f.ready(&mut b, &op);
    assert_eq!(
        b.social_roster(&a, None, true).unwrap().people[0].handle,
        "sysop"
    );
    b.social_request(&a, "sysop", &request_id().unwrap(), true)
        .unwrap();
    let id = b.social_pages(&op).unwrap()[0].id;
    b.social_answer(&op, id, false).unwrap();
    assert_eq!(b.social_pages(&a).unwrap()[0].state, "declined");
}
#[test]
fn cooldown_survives_mode_changes_reconnect_and_busy_transaction_retry() {
    let f = Fixture::new();
    let (mut b, s, op) = f.board();
    let (session, a) = f.member(&mut b, &s, "alice");
    let (_, bob) = f.member(&mut b, &s, "bob");
    f.enable(&mut b, &op);
    f.ready(&mut b, &a);
    f.ready(&mut b, &bob);
    let lock = f.db();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    let n = request_id().unwrap();
    assert!(matches!(
        b.social_request(&a, "bob", &n, false),
        Err(Error::Busy)
    ));
    lock.execute_batch("ROLLBACK").unwrap();
    b.social_request(&a, "bob", &n, false).unwrap();
    b.social_availability(&a, "hidden", "").unwrap();
    f.ready(&mut b, &a);
    assert!(
        b.social_request(&a, "bob", &request_id().unwrap(), false)
            .is_err()
    );
    b.chat_close(&a).unwrap();
    let next = b.chat_open(&session).unwrap();
    f.ready(&mut b, &next);
    assert!(
        b.social_request(&next, "bob", &request_id().unwrap(), false)
            .is_err()
    );
    f.db()
        .execute("UPDATE social_cooldowns SET last_page=unixepoch()-31", [])
        .unwrap();
    b.social_request(&next, "bob", &request_id().unwrap(), false)
        .unwrap();
    b.social_preferences(&bob, true, None).unwrap();
    assert!(b.social_pages(&next).unwrap().is_empty());
    b.social_preferences(&bob, false, None).unwrap();
    assert!(b.social_pages(&bob).unwrap().is_empty());
}
#[test]
fn disconnect_stale_future_and_expired_requests_do_not_revive() {
    let f = Fixture::new();
    let (mut b, s, op) = f.board();
    let (_, a) = f.member(&mut b, &s, "alice");
    let (_, bob) = f.member(&mut b, &s, "bob");
    f.enable(&mut b, &op);
    f.ready(&mut b, &a);
    f.ready(&mut b, &bob);
    b.social_request(&a, "bob", &request_id().unwrap(), false)
        .unwrap();
    f.db()
        .execute("UPDATE social_pages SET created=unixepoch()-61", [])
        .unwrap();
    assert!(b.social_pages(&bob).unwrap().is_empty());
    f.reset_rate();
    b.social_request(&a, "bob", &request_id().unwrap(), false)
        .unwrap();
    b.chat_close(&bob).unwrap();
    assert!(b.social_pages(&a).unwrap().is_empty());
    f.db().execute("UPDATE chat_sessions SET touched=unixepoch()+10 WHERE member=(SELECT id FROM members WHERE handle='alice')",[]).unwrap();
    assert!(b.social_roster(&op, None, false).unwrap().people.is_empty());
    assert!(b.social_pages(&a).is_err());
    f.db().execute("UPDATE chat_sessions SET touched=unixepoch()-61 WHERE member=(SELECT id FROM members WHERE handle='alice')",[]).unwrap();
    assert!(b.social_roster(&op, None, false).unwrap().people.is_empty());
}
#[test]
fn privacy_block_and_request_storage_budgets_are_bounded() {
    let f = Fixture::new();
    let (mut b, s, op) = f.board();
    let (_, a) = f.member(&mut b, &s, "alice");
    let (_, bob) = f.member(&mut b, &s, "bob");
    f.enable(&mut b, &op);
    f.ready(&mut b, &a);
    f.ready(&mut b, &bob);
    for n in 0..16 {
        b.social_block(&a, &format!("unknown{n}"), true).unwrap();
    }
    assert!(b.social_block(&a, "extra", true).is_err());
    assert_eq!(b.social_prefs(&a).unwrap().blocks.len(), 16);
    for _ in 0..8 {
        f.reset_rate();
        b.social_request(&a, "bob", &request_id().unwrap(), false)
            .unwrap();
    }
    assert_eq!(b.social_pages(&bob).unwrap().len(), 8);
    f.reset_rate();
    b.social_request(&a, "bob", &request_id().unwrap(), false)
        .unwrap();
    assert_eq!(b.social_pages(&bob).unwrap().len(), 8);
    b.social_configure(
        &op,
        &SocialConfig {
            enabled: false,
            name: "Closed lounge".into(),
        },
    )
    .unwrap();
    assert!(b.social_pages(&a).is_err());
    f.enable(&mut b, &op);
    assert!(b.social_roster(&op, None, false).unwrap().people.is_empty());
}
#[test]
fn exact_snapshots_keep_preferences_but_remove_live_social_authority_and_schema_sixteen_is_readonly()
 {
    let f = Fixture::new();
    let (mut b, s, op) = f.board();
    let (_, a) = f.member(&mut b, &s, "alice");
    f.enable(&mut b, &op);
    f.ready(&mut b, &a);
    f.ready(&mut b, &op);
    b.social_block(&a, "unknown", true).unwrap();
    b.social_request(&a, "sysop", &request_id().unwrap(), true)
        .unwrap();
    let out = f.0.join("export");
    export_local(&f.path(), &out).unwrap();
    verify_export(&out).unwrap();
    let restored = f.0.join("restored");
    restore_local(&out, &restored).unwrap();
    let mut recovered = Board::open(&restored).unwrap();
    let s = recovered.authenticate("sysop", PASS).unwrap();
    let l = recovered.chat_open(&s).unwrap();
    assert!(
        recovered
            .social_roster(&l, None, false)
            .unwrap()
            .people
            .is_empty()
    );
    let db = rusqlite::Connection::open(restored.join("faction.sqlite3")).unwrap();
    for table in ["social_pages", "social_presence", "social_cooldowns"] {
        assert_eq!(
            db.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    assert_eq!(
        db.query_row("SELECT handle FROM social_blocks", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "unknown"
    );
    drop(b);
    f.db().execute_batch("DROP TABLE roaming_chat;DROP TABLE roaming_transfers;DROP TABLE roaming_bindings;DROP TABLE roaming_issuers;DROP TABLE directory_submissions;DROP TABLE visit_activity;DROP TABLE social_pages;DROP TABLE social_presence;DROP TABLE social_blocks;DROP TABLE social_prefs;DROP TABLE social_cooldowns;DROP TABLE social_settings;PRAGMA user_version=16").unwrap();
    let old = f.0.join("old");
    export_local(&f.path(), &old).unwrap();
    verify_export(&old).unwrap();
    assert_eq!(
        f.db()
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        16
    );
    let restored = f.0.join("old-restored");
    restore_local(&old, &restored).unwrap();
    let mut upgraded = Board::open(&restored).unwrap();
    let session = upgraded.authenticate("sysop", PASS).unwrap();
    let lease = upgraded.chat_open(&session).unwrap();
    assert!(!upgraded.social_config(&lease).unwrap().enabled);
}
#[test]
fn malformed_status_config_quiet_hours_and_forged_lease_snapshot_fail() {
    let f = Fixture::new();
    let (mut b, s, op) = f.board();
    let (_, a) = f.member(&mut b, &s, "alice");
    f.enable(&mut b, &op);
    for text in ["\u{1b}[2J", "bad\nline", "é"] {
        assert!(b.social_availability(&a, "available", text).is_err());
        assert!(
            b.social_configure(
                &op,
                &SocialConfig {
                    enabled: true,
                    name: text.into()
                }
            )
            .is_err()
        );
    }
    assert!(b.social_availability(&a, "offline", "").is_err());
    assert!(b.social_preferences(&a, false, Some((1440, 0))).is_err());
    assert!(b.social_preferences(&a, false, Some((1, 1))).is_err());
    f.ready(&mut b, &a);
    f.db()
        .execute("UPDATE social_presence SET member=1", [])
        .unwrap();
    assert!(export_local(&f.path(), &f.0.join("bad")).is_err());
}

#[test]
fn leaving_chat_keeps_connection_presence_but_clears_joins_receipt_and_history_floor() {
    use faction_core::ChatKind;
    let f = Fixture::new();
    let (mut b, s, op) = f.board();
    let (_, a) = f.member(&mut b, &s, "alice");
    let (_, bob) = f.member(&mut b, &s, "bob");
    f.enable(&mut b, &op);
    f.ready(&mut b, &a);
    f.ready(&mut b, &bob);
    b.chat_create(&s, "#vault", true).unwrap();
    b.chat_allow(&s, "#vault", "alice", true).unwrap();
    b.chat_join(&a, "#lobby").unwrap();
    b.chat_join(&bob, "#lobby").unwrap();
    b.chat_join(&a, "#vault").unwrap();
    b.chat_send(&a, "#vault", 1, "before leaving", ChatKind::Say)
        .unwrap();
    b.chat_leave_all(&a).unwrap();
    assert_eq!(b.chat_names(&bob, "#lobby", None).unwrap().handles, ["bob"]);
    assert!(b.chat_read(&a, "#vault", 0).is_err());
    assert_eq!(
        b.social_advertisement(&a).unwrap().unwrap().mode,
        "available"
    );
    assert!(
        b.social_roster(&bob, None, false)
            .unwrap()
            .people
            .iter()
            .any(|p| p.handle == "alice")
    );
    f.db().execute("UPDATE chat_sessions SET touched=unixepoch()-50 WHERE member=(SELECT id FROM members WHERE handle='alice')",[]).unwrap();
    b.chat_heartbeat(&a).unwrap();
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM chat_sessions", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        3
    );
    b.chat_join(&a, "#vault").unwrap();
    b.chat_send(&a, "#vault", 1, "new explicit entry", ChatKind::Say)
        .unwrap();
    let history = b.chat_read(&a, "#vault", 0).unwrap();
    assert_eq!(history.messages.len(), 1);
    assert_eq!(history.messages[0].body, "new explicit entry");
}
#[test]
fn board_visit_pages_work_without_channel_joins_and_end_on_disconnect() {
    let f = Fixture::new();
    let (mut b, s, op) = f.board();
    let (session, a) = f.member(&mut b, &s, "alice");
    let (_, bob) = f.member(&mut b, &s, "bob");
    f.enable(&mut b, &op);
    assert!(b.social_session_config(&session).unwrap().enabled);
    assert!(b.social_advertisement(&a).unwrap().is_none());
    f.ready(&mut b, &a);
    f.ready(&mut b, &bob);
    b.social_request(&bob, "alice", &request_id().unwrap(), false)
        .unwrap();
    let rows = b.social_pages(&a).unwrap();
    assert_eq!(rows.len(), 1);
    b.social_answer(&a, rows[0].id, true).unwrap();
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM chat_joins", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    b.chat_close(&a).unwrap();
    assert!(b.social_pages(&bob).unwrap().is_empty());
    let next = b.chat_open(&session).unwrap();
    assert!(b.social_advertisement(&next).unwrap().is_none());
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(b.social_session_config(&session).is_err());
    assert!(b.chat_leave_all(&next).is_err());
}

#[test]
fn host_activity_is_current_cross_connection_and_uses_universal_visibility() {
    use faction_core::VisitActivity as A;
    let f = Fixture::new();
    let (mut b, s, op) = f.board();
    let (_, a) = f.member(&mut b, &s, "alice");
    f.enable(&mut b, &op);
    b.visit_activity(&a, A::Doors).unwrap();
    assert!(b.social_roster(&op, None, false).unwrap().people.is_empty());
    f.ready(&mut b, &a);
    let mut reader = Board::open(&f.path()).unwrap();
    let row = reader
        .social_roster(&op, None, false)
        .unwrap()
        .people
        .remove(0);
    assert_eq!(
        (row.activity.as_str(), row.mode.as_str()),
        ("Doors", "available")
    );
    b.social_availability(&a, "busy", "Taking a break").unwrap();
    b.visit_activity(&a, A::Rooms).unwrap();
    let row = reader
        .social_roster(&op, None, false)
        .unwrap()
        .people
        .remove(0);
    assert_eq!(
        (
            row.activity.as_str(),
            row.mode.as_str(),
            row.status.as_str()
        ),
        ("Rooms", "busy", "Taking a break")
    );
    b.social_block(&a, "sysop", true).unwrap();
    assert!(
        reader
            .social_roster(&op, None, false)
            .unwrap()
            .people
            .is_empty()
    );
    b.social_block(&a, "sysop", false).unwrap();
    b.social_availability(&a, "hidden", "").unwrap();
    assert!(
        reader
            .social_roster(&op, None, false)
            .unwrap()
            .people
            .is_empty()
    );
    f.ready(&mut b, &a);
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(b.visit_activity(&a, A::Chat).is_err());
    assert!(
        reader
            .social_roster(&op, None, false)
            .unwrap()
            .people
            .is_empty()
    );
}

#[test]
fn current_activity_cascades_on_close_and_never_survives_export_restore() {
    use faction_core::VisitActivity as A;
    let f = Fixture::new();
    let (mut b, _, op) = f.board();
    f.enable(&mut b, &op);
    f.ready(&mut b, &op);
    b.visit_activity(&op, A::Files).unwrap();
    let output = f.0.join("current-export");
    export_local(&f.path(), &output).unwrap();
    verify_export(&output).unwrap();
    let db = rusqlite::Connection::open(&output).unwrap();
    assert_eq!(
        db.query_row("SELECT count(*) FROM visit_activity", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    let restored = f.0.join("restored");
    restore_local(&output, &restored).unwrap();
    let restored = rusqlite::Connection::open(restored.join("faction.sqlite3")).unwrap();
    assert_eq!(
        restored
            .query_row("SELECT count(*) FROM visit_activity", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    b.chat_close(&op).unwrap();
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM visit_activity", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn schema_seventeen_export_is_read_only_and_owner_open_adds_empty_activity() {
    let f = Fixture::new();
    let (b, _, _) = f.board();
    drop(b);
    f.db()
        .execute_batch(
            "DROP TABLE roaming_chat;DROP TABLE roaming_transfers;DROP TABLE roaming_bindings;DROP TABLE roaming_issuers;DROP TABLE directory_submissions;DROP TABLE visit_activity; PRAGMA user_version=17;",
        )
        .unwrap();
    let output = f.0.join("schema17-export");
    export_local(&f.path(), &output).unwrap();
    verify_export(&output).unwrap();
    assert_eq!(
        f.db()
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        17
    );
    let mut b = Board::open(&f.path()).unwrap();
    assert_eq!(
        f.db()
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        20
    );
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM visit_activity", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    let s = b.authenticate("sysop", PASS).unwrap();
    let lease = b.chat_open(&s).unwrap();
    assert!(b.social_advertisement(&lease).is_err()); // feature remains off
}
