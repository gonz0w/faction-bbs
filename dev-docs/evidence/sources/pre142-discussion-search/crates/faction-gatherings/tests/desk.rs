use faction_core::{Board, Error, Session, request_id};
use faction_gatherings::{Desk, Outcome};
use faction_terminal::Theme;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
const PASS: &str = "synthetic-gathering-desk-password";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> (Self, Board, Session) {
        let p =
            std::env::temp_dir().join(format!("faction-gathering-desk-{}", request_id().unwrap()));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        let mut b = Board::open(&p.join("board")).unwrap();
        b.bootstrap("sysop", PASS).unwrap();
        let s = b.authenticate("sysop", PASS).unwrap();
        (Self(p), b, s)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn render(d: &mut Desk, b: &mut Board, s: &Session) -> String {
    d.render(b, s, &Theme::default(), false, 40).unwrap()
}
fn feed(d: &mut Desk, input: &str, b: &mut Board, s: &Session) {
    assert!(matches!(d.feed(input, b, s).unwrap(), Outcome::Stay));
}
#[test]
fn opt_in_settings_creation_preview_ascii_and_explicit_save() {
    let (_f, mut b, s) = Fixture::new();
    let mut desk = Desk::settings();
    feed(&mut desk, "e", &mut b, &s);
    feed(&mut desk, "n", &mut b, &s);
    feed(&mut desk, "Afterhours agenda", &mut b, &s);
    assert!(render(&mut desk, &mut b, &s).contains("Afterhours agenda"));
    let mut desk = Desk::default();
    feed(&mut desk, "c", &mut b, &s);
    assert!(desk.feed("Bad\x1b[2J", &mut b, &s).is_err());
    feed(&mut desk, "A future gathering", &mut b, &s);
    feed(&mut desk, "2099-01-01 12:00", &mut b, &s);
    feed(&mut desk, "120", &mut b, &s);
    feed(&mut desk, "room:1", &mut b, &s);
    feed(&mut desk, "Original local notes", &mut b, &s);
    feed(&mut desk, ".", &mut b, &s);
    assert!(render(&mut desk, &mut b, &s).contains("PREVIEW ONLY"));
    assert!(b.gatherings(&s, false, 0).unwrap().is_empty());
    let error = desk.feed("s", &mut b, &s).err().unwrap();
    assert!(desk.recover(&error));
    assert!(render(&mut desk, &mut b, &s).contains("PREVIEW ONLY"));
    feed(&mut desk, "c", &mut b, &s);
    assert!(b.gatherings(&s, false, 0).unwrap().is_empty());
}
#[test]
fn disabling_an_open_desk_hides_its_content_and_returns_to_main() {
    let (_f, mut b, s) = Fixture::new();
    let mut config = b.gathering_config(&s).unwrap();
    config.enabled = true;
    b.configure_gatherings(&s, &config).unwrap();
    let mut desk = Desk::default();
    feed(&mut desk, "c", &mut b, &s);
    feed(&mut desk, "Unsaved title", &mut b, &s);
    config.enabled = false;
    b.configure_gatherings(&s, &config).unwrap();
    let frame = render(&mut desk, &mut b, &s);
    assert!(frame.contains("Gatherings are disabled"));
    assert!(!frame.contains("Unsaved title"));
    assert!(matches!(desk.feed("q", &mut b, &s).unwrap(), Outcome::Main));
}
#[test]
fn a_frame_error_is_recoverable_but_revocation_is_not() {
    let (_f, mut b, s) = Fixture::new();
    let mut desk = Desk::default();
    assert!(desk.recover(&Error::Invalid("fixture")));
    assert!(!desk.recover(&Error::Denied));
    assert!(!desk.recover(&Error::StaleSession));
    assert!(
        desk.render(&mut b, &s, &Theme::default(), true, 40)
            .unwrap()
            .len()
            < 8192
    );
}
