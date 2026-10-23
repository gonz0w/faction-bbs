use faction_core::{
    Board, Error, SCORE_GAMES, SCORE_MAX, Session, Status, request_id, restore_local, verify_export,
};
use std::{fs, os::unix::fs::DirBuilderExt, path::PathBuf};
struct Fixture {
    parent: PathBuf,
    board: Board,
    root: Session,
    secret: String,
}
impl Fixture {
    fn new() -> Self {
        let parent = std::env::temp_dir().join(format!("faction-scores-{}", request_id().unwrap()));
        fs::DirBuilder::new().mode(0o700).create(&parent).unwrap();
        let mut board = Board::open(&parent.join("board")).unwrap();
        let secret = request_id().unwrap();
        board.bootstrap("sysop", &secret).unwrap();
        let root = board.authenticate("sysop", &secret).unwrap();
        Self {
            parent,
            board,
            root,
            secret,
        }
    }
    fn member(&mut self, handle: &str, approved: bool) -> Session {
        self.board.register(handle, &self.secret).unwrap();
        if approved {
            self.board
                .set_status(&self.root, handle, Status::Approved)
                .unwrap();
        }
        self.board.authenticate(handle, &self.secret).unwrap()
    }
    fn finish(&mut self, s: &Session, game: &str, score: u32) {
        let mut run = self.board.game_start(s, game).unwrap();
        self.board.game_finish(s, &mut run, score).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.parent);
    }
}
#[test]
fn bests_order_ties_personal_counts_and_top_ten_survive_reopen() {
    let mut f = Fixture::new();
    for i in 0..12 {
        let s = f.member(&format!("player{i:02}"), true);
        f.finish(&s, "neon-salvage", 100 + i);
        if i == 0 {
            f.finish(&s, "neon-salvage", 111);
            f.finish(&s, "neon-salvage", 2);
        }
    }
    let mut reopened = Board::open(&f.parent.join("board")).unwrap();
    let scores = reopened.game_scores(&f.root, "neon-salvage").unwrap();
    assert_eq!(scores.len(), 10);
    assert_eq!(scores[0].handle, "player00");
    assert_eq!((scores[0].best, scores[0].plays), (111, 3));
    assert_eq!(scores[1].handle, "player11");
    assert_eq!(scores[9].best, 103);
    f.board
        .set_status(&f.root, "player00", Status::Revoked)
        .unwrap();
    assert_eq!(
        reopened.game_scores(&f.root, "neon-salvage").unwrap()[0].handle,
        "player11"
    );
    assert!(
        reopened
            .game_personal_best(&f.root, "neon-salvage")
            .unwrap()
            .is_none()
    );
}
#[test]
fn runs_are_one_shot_member_board_and_epoch_bound_with_current_approval() {
    let mut f = Fixture::new();
    let a = f.member("alice", true);
    let b = f.member("bobby", true);
    let pending = f.member("pending", false);
    assert!(matches!(
        f.board.game_start(&pending, "hunt"),
        Err(Error::Denied)
    ));
    assert!(matches!(
        f.board.game_scores(&pending, "hunt"),
        Err(Error::Denied)
    ));
    assert!(matches!(
        f.board.game_personal_best(&pending, "hunt"),
        Err(Error::Denied)
    ));
    let mut run = f.board.game_start(&a, "hunt").unwrap();
    assert!(matches!(
        f.board.game_finish(&b, &mut run, 900),
        Err(Error::Denied)
    ));
    let mut other = Fixture::new();
    assert!(matches!(
        other.board.game_finish(&other.root, &mut run, 900),
        Err(Error::Denied)
    ));
    assert!(matches!(
        f.board.game_finish(&a, &mut run, SCORE_MAX + 1),
        Err(Error::Invalid(_))
    ));
    let accepted = f.board.game_finish(&a, &mut run, 700).unwrap();
    assert!(accepted.improved);
    assert!(matches!(
        f.board.game_finish(&a, &mut run, 900),
        Err(Error::RequestConflict)
    ));
    assert_eq!(
        f.board
            .game_personal_best(&a, "hunt")
            .unwrap()
            .unwrap()
            .plays,
        1
    );
    let mut stale = f.board.game_start(&a, "hunt").unwrap();
    f.board
        .set_status(&f.root, "alice", Status::Revoked)
        .unwrap();
    assert!(matches!(
        f.board.game_finish(&a, &mut stale, 900),
        Err(Error::StaleSession)
    ));
    f.board
        .set_status(&f.root, "alice", Status::Approved)
        .unwrap();
    let fresh = f.board.authenticate("alice", &f.secret).unwrap();
    assert!(matches!(
        f.board.game_finish(&fresh, &mut stale, 900),
        Err(Error::Denied)
    ));
    assert!(matches!(
        f.board.game_scores(&a, "hunt"),
        Err(Error::StaleSession)
    ));
    assert_eq!(
        f.board
            .game_personal_best(&fresh, "hunt")
            .unwrap()
            .unwrap()
            .best,
        700
    );
    // Dropping an incomplete run records nothing.
    drop(f.board.game_start(&fresh, "unfinished").unwrap());
    assert!(
        f.board
            .game_scores(&fresh, "unfinished")
            .unwrap()
            .is_empty()
    );
}
#[test]
fn failed_commit_can_retry_and_personal_game_keys_are_bounded() {
    let mut f = Fixture::new();
    let a = f.member("alice", true);
    let db = rusqlite::Connection::open(f.parent.join("board/faction.sqlite3")).unwrap();
    let mut run = f.board.game_start(&a, "hunt").unwrap();
    db.execute_batch("BEGIN IMMEDIATE").unwrap();
    assert!(matches!(
        f.board.game_finish(&a, &mut run, 7),
        Err(Error::Busy)
    ));
    db.execute_batch("ROLLBACK").unwrap();
    assert_eq!(f.board.game_finish(&a, &mut run, 7).unwrap().plays, 1);
    for i in 1..SCORE_GAMES {
        f.finish(&a, &format!("game-{i}"), 0);
    }
    let mut extra = f.board.game_start(&a, "too-many").unwrap();
    assert!(matches!(
        f.board.game_finish(&a, &mut extra, 1),
        Err(Error::Invalid(_))
    ));
    f.finish(&a, "hunt", 6);
    assert_eq!(
        f.board
            .game_personal_best(&a, "hunt")
            .unwrap()
            .unwrap()
            .plays,
        2
    );
    for bad in ["../db", "", "BAD", "bad\x1b"] {
        assert!(f.board.game_start(&a, bad).is_err());
    }
}
#[test]
fn scores_export_restore_and_schema_two_exports_stay_readable() {
    let mut f = Fixture::new();
    let a = f.member("alice", true);
    f.finish(&a, "signal-hunt", 500);
    let export = f.parent.join("export.sqlite3");
    f.board.export_local(&export).unwrap();
    assert_eq!(verify_export(&export).unwrap().game_scores, 1);
    let restored = f.parent.join("restored");
    restore_local(&export, &restored).unwrap();
    let mut board = Board::open(&restored).unwrap();
    let s = board.authenticate("alice", &f.secret).unwrap();
    assert_eq!(
        board
            .game_personal_best(&s, "signal-hunt")
            .unwrap()
            .unwrap()
            .best,
        500
    );
    assert!(matches!(
        board.game_scores(&a, "signal-hunt"),
        Err(Error::Denied)
    ));
    let db = rusqlite::Connection::open(&export).unwrap();
    db.execute_batch("DROP TABLE poll_ballots; DROP TABLE poll_choices; DROP TABLE polls; DROP TABLE poll_settings; DROP TABLE gathering_rsvps; DROP TABLE gatherings; DROP TABLE gathering_settings; DROP TABLE member_plans; DROP TABLE member_pages; DROP TABLE page_settings; DROP TABLE wall_receipts; DROP TABLE wall_notes; DROP TABLE wall_settings; DROP TABLE room_drafts; DROP TABLE post_metadata; DROP TABLE file_namespace; DROP TABLE file_transfers; DROP TABLE file_objects; DROP TABLE file_area_grants; DROP TABLE file_areas; DROP TABLE game_scores; PRAGMA user_version=2")
        .unwrap();
    drop(db);
    let before = fs::read(&export).unwrap();
    assert_eq!(verify_export(&export).unwrap().game_scores, 0);
    let old = f.parent.join("old-restored");
    restore_local(&export, &old).unwrap();
    let mut old_board = Board::open(&old).unwrap();
    let s = old_board.authenticate("alice", &f.secret).unwrap();
    assert!(old_board.game_scores(&s, "signal-hunt").unwrap().is_empty());
    assert_eq!(fs::read(&export).unwrap(), before);
    let db = rusqlite::Connection::open(old.join("faction.sqlite3")).unwrap();
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        12
    );
}
#[test]
fn migration_failure_rolls_back_and_exports_reject_invalid_score_data() {
    let mut f = Fixture::new();
    let db = rusqlite::Connection::open(f.parent.join("board/faction.sqlite3")).unwrap();
    db.execute_batch(
        "DROP TABLE poll_ballots; DROP TABLE poll_choices; DROP TABLE polls; DROP TABLE poll_settings; DROP TABLE gathering_rsvps; DROP TABLE gatherings; DROP TABLE gathering_settings; DROP TABLE member_plans; DROP TABLE member_pages; DROP TABLE page_settings; DROP TABLE wall_receipts; DROP TABLE wall_notes; DROP TABLE wall_settings; DROP TABLE room_drafts; DROP TABLE post_metadata; DROP TABLE file_namespace; DROP TABLE file_transfers; DROP TABLE file_objects; DROP TABLE file_area_grants; DROP TABLE file_areas; DROP TABLE game_scores; PRAGMA user_version=2; CREATE TABLE game_scores(unrelated TEXT)",
    )
    .unwrap();
    assert!(Board::open(&f.parent.join("board")).is_err());
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        2
    );
    db.execute_batch("DROP TABLE game_scores").unwrap();
    f.board = Board::open(&f.parent.join("board")).unwrap();
    let mut run = f.board.game_start(&f.root, "hunt").unwrap();
    f.board.game_finish(&f.root, &mut run, 9).unwrap();
    let export = f.parent.join("bad.sqlite3");
    f.board.export_local(&export).unwrap();
    let imported = rusqlite::Connection::open(&export).unwrap();
    imported
        .execute_batch("UPDATE game_scores SET game='../state'")
        .unwrap();
    assert!(verify_export(&export).is_err());
    assert!(restore_local(&export, &f.parent.join("denied")).is_err());
    assert!(!f.parent.join("denied").exists());
    imported
        .execute_batch(
            "PRAGMA ignore_check_constraints=ON; UPDATE game_scores SET game='hunt', best=-1",
        )
        .unwrap();
    assert!(verify_export(&export).is_err());
}
