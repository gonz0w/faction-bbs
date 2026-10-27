use faction_core::{Board, Session, Status, request_id};
use faction_pages::directory::Desk;
use faction_terminal::Theme;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-directory-ui-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> (Self, Board, Session) {
        let p =
            std::env::temp_dir().join(format!("faction-directory-ui-{}", request_id().unwrap()));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        let mut b = Board::open(&p.join("board")).unwrap();
        b.configure_directory_owner(Some(vec![])).unwrap();
        b.bootstrap("sysop", PASS).unwrap();
        let s = b.authenticate("sysop", PASS).unwrap();
        (Self(p), b, s)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn enter(d: &mut Desk, b: &mut Board, s: &Session, v: &str) {
    assert!(!d.feed(v, b, s).unwrap());
}
fn draft(d: &mut Desk, b: &mut Board, s: &Session) {
    for v in [
        "n",
        "radio",
        "Radio Board",
        "Night radio.",
        "radio.example",
        "2222",
        "bbs",
        "",
    ] {
        enter(d, b, s, v);
    }
}
#[test]
fn preview_cancel_and_save_require_explicit_publication() {
    let (_f, mut b, s) = Fixture::new();
    let mut d = Desk::default();
    draft(&mut d, &mut b, &s);
    let frame = d
        .render(&mut b, &s, &Theme::default(), false, (40, 18))
        .unwrap();
    assert!(frame.contains("Draft preview"));
    assert!(frame.contains("radio.example"));
    assert!(frame.lines().count() <= 18);
    assert!(frame.lines().all(|v| v.len() < 40));
    enter(&mut d, &mut b, &s, "c");
    assert!(b.directory_submissions(&s, 0).unwrap().is_empty());
    draft(&mut d, &mut b, &s);
    enter(&mut d, &mut b, &s, "s");
    let rows = b.directory_submissions(&s, 0).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].state, "draft");
    assert!(b.directory_published(&s).unwrap().is_empty());
    enter(&mut d, &mut b, &s, "s");
    assert_eq!(
        b.directory_submission(&s, rows[0].id).unwrap().state,
        "pending"
    );
    assert!(b.directory_published(&s).unwrap().is_empty());
    enter(&mut d, &mut b, &s, "w");
    assert!(b.directory_submissions(&s, 0).unwrap().is_empty());
}
#[test]
fn current_role_filters_review_and_revocation_denies_draft_input() {
    let (_f, mut b, s) = Fixture::new();
    b.register("alice", PASS).unwrap();
    b.set_status(&s, "alice", Status::Approved).unwrap();
    let a = b.authenticate("alice", PASS).unwrap();
    let mut d = Desk::default();
    let frame = d
        .render(&mut b, &a, &Theme::default(), false, (80, 24))
        .unwrap();
    assert!(!frame.contains("M:review"));
    assert!(d.feed("m", &mut b, &a).is_err());
    enter(&mut d, &mut b, &a, "n");
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(d.feed("radio", &mut b, &a).is_err());
    assert!(
        d.render(&mut b, &a, &Theme::default(), false, (80, 24))
            .is_err()
    );
    assert!(b.directory_submissions(&s, 0).unwrap().is_empty());
}

fn frame(d: &mut Desk, b: &mut Board, s: &Session) -> String {
    let frame = d.render(b, s, &Theme::default(), false, (40, 18)).unwrap();
    assert!(frame.lines().count() <= 18, "{frame}");
    assert!(frame.lines().all(|line| line.len() < 40), "{frame}");
    assert!(!frame.contains('\x1b'));
    frame
}
fn pending(b: &mut Board, s: &Session) -> faction_core::DirectorySubmission {
    let mut d = Desk::default();
    draft(&mut d, b, s);
    enter(&mut d, b, s, "s");
    enter(&mut d, b, s, "s");
    b.directory_submissions(s, 0)
        .unwrap()
        .into_iter()
        .find(|r| r.mine && r.state == "pending")
        .unwrap()
}
fn open_review(d: &mut Desk, b: &mut Board, s: &Session) {
    enter(d, b, s, "m");
    frame(d, b, s);
    enter(d, b, s, "1");
    frame(d, b, s);
}
fn approval(d: &mut Desk, b: &mut Board, s: &Session) {
    for command in ["a", "2026-10-26", "2027-10-26"] {
        enter(d, b, s, command);
        frame(d, b, s);
    }
}
#[test]
fn review_requires_explicit_confirmation_then_can_expire_remove_or_reject() {
    let (_f, mut b, s) = Fixture::new();
    let row = pending(&mut b, &s);
    let mut review = Desk::default();
    open_review(&mut review, &mut b, &s);
    approval(&mut review, &mut b, &s);
    enter(&mut review, &mut b, &s, "approve"); // Exact uppercase confirmation is mandatory.
    assert!(b.directory_published(&s).unwrap().is_empty());
    approval(&mut review, &mut b, &s);
    enter(&mut review, &mut b, &s, "APPROVE");
    assert_eq!(b.directory_published(&s).unwrap().len(), 1);
    assert_eq!(
        b.directory_submission(&s, row.id).unwrap().state,
        "published"
    );
    let directory = faction_pages::directory::catalog(&mut b, &s, &Default::default()).unwrap();
    let mut view = faction_terminal::directory::View::default();
    view.feed(faction_terminal::Key::Enter, &directory, 80, 24);
    let text = directory.render(&view, 80, 24, false, &Theme::default());
    assert!(text.contains("Locally moderated member submission."));
    assert!(text.contains("radio.example"));
    frame(&mut review, &mut b, &s);
    enter(&mut review, &mut b, &s, "1");
    enter(&mut review, &mut b, &s, "e");
    frame(&mut review, &mut b, &s);
    enter(&mut review, &mut b, &s, "EXPIRE");
    let directory = faction_pages::directory::catalog(&mut b, &s, &Default::default()).unwrap();
    assert!(
        directory
            .render(&Default::default(), 40, 18, false, &Theme::default())
            .contains("OLD")
    );
    frame(&mut review, &mut b, &s);
    enter(&mut review, &mut b, &s, "1");
    enter(&mut review, &mut b, &s, "x");
    frame(&mut review, &mut b, &s);
    enter(&mut review, &mut b, &s, "REMOVE");
    assert!(b.directory_submissions(&s, 0).unwrap().is_empty());
    pending(&mut b, &s);
    frame(&mut review, &mut b, &s);
    enter(&mut review, &mut b, &s, "1");
    enter(&mut review, &mut b, &s, "r");
    frame(&mut review, &mut b, &s);
    enter(&mut review, &mut b, &s, "REJECT");
    assert!(b.directory_submissions(&s, 0).unwrap().is_empty());
}
#[test]
fn review_uses_shown_revision_and_recovery_reopens_current_list() {
    let (_f, mut b, s) = Fixture::new();
    let row = pending(&mut b, &s);
    let mut review = Desk::default();
    open_review(&mut review, &mut b, &s);
    approval(&mut review, &mut b, &s);
    b.directory_review(
        &s,
        row.id,
        &row.revision,
        "approve",
        Some("2026-10-26"),
        Some("2027-10-26"),
    )
    .unwrap();
    let e = review.feed("APPROVE", &mut b, &s).unwrap_err();
    assert!(matches!(e, faction_core::Error::RequestConflict));
    assert!(review.recover(&e));
    let text = frame(&mut review, &mut b, &s);
    assert!(text.contains("Submission changed"));
    assert_eq!(b.directory_published(&s).unwrap().len(), 1);
}
#[test]
fn pending_accounts_and_untrusted_input_never_create_a_saved_row() {
    let (_f, mut b, s) = Fixture::new();
    b.register("pending", PASS).unwrap();
    let p = b.authenticate("pending", PASS).unwrap();
    let mut d = Desk::default();
    assert!(d.feed("n", &mut b, &p).is_err());
    assert!(
        d.render(&mut b, &p, &Theme::default(), false, (40, 18))
            .is_err()
    );
    enter(&mut d, &mut b, &s, "n");
    for input in ["bad\x1b[2J", "non-ASCII \u{202e}"] {
        assert!(d.feed(input, &mut b, &s).is_err());
    }
    enter(&mut d, &mut b, &s, "c");
    for input in [
        "n",
        "bad",
        "Bad Board",
        "A summary",
        "-oProxyCommand=sh",
        "22",
        "bbs",
    ] {
        enter(&mut d, &mut b, &s, input);
    }
    assert!(d.feed("", &mut b, &s).is_err());
    enter(&mut d, &mut b, &s, "c");
    assert!(b.directory_submissions(&s, 0).unwrap().is_empty());
}
#[test]
fn publication_conversion_preserves_literal_quotes_and_hides_revoked_authors() {
    let (_f, mut b, s) = Fixture::new();
    b.register("alice", PASS).unwrap();
    b.set_status(&s, "alice", Status::Approved).unwrap();
    let a = b.authenticate("alice", PASS).unwrap();
    let d = faction_core::DirectoryDraft {
        slug: "quoted".into(),
        name: "Quoted Board".into(),
        summary: "A 'quote', a \"quote\", and \\ path.".into(),
        host: "RADIO.EXAMPLE".into(),
        port: 22,
        login: "bbs".into(),
        fingerprint: Some(format!("SHA256:{}", "a".repeat(43))),
    };
    let id = b.directory_save(&a, &request_id().unwrap(), &d).unwrap();
    let row = b.directory_submission(&a, id).unwrap();
    assert!(b.directory_submissions(&s, 0).unwrap().is_empty()); // Another author's draft stays private.
    b.directory_submit(&a, id, &row.revision).unwrap();
    let row = b.directory_submission(&s, id).unwrap();
    b.directory_review(
        &s,
        id,
        &row.revision,
        "approve",
        Some("2026-10-26"),
        Some("2027-10-26"),
    )
    .unwrap();
    let directory = faction_pages::directory::catalog(&mut b, &s, &Default::default()).unwrap();
    let mut view = faction_terminal::directory::View::default();
    view.feed(faction_terminal::Key::Enter, &directory, 120, 40);
    let text = directory.render(&view, 120, 40, false, &Theme::default());
    assert!(text.contains(&d.summary));
    assert!(text.contains("Host: radio.example"));
    assert!(text.contains(d.fingerprint.as_deref().unwrap()));
    b.set_status(&s, "alice", Status::Revoked).unwrap();
    assert!(
        faction_pages::directory::catalog(&mut b, &s, &Default::default())
            .unwrap()
            .is_empty()
    );
}
