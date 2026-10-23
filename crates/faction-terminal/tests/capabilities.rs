use faction_terminal::{
    Message, MessageDetails,
    capabilities::{Capabilities, Color, PROBE_TIME, Probe},
    subject, text,
};
use std::time::Instant;
#[test]
fn independent_preferences_and_classic_recovery() {
    let mut c = Capabilities::default();
    assert!(!c.unicode && !c.synchronized);
    for cmd in ["u", "3", "b", "a", "s"] {
        assert!(c.choose(cmd));
    }
    assert!(c.unicode && c.blocks && c.braille && c.synchronized);
    assert_eq!(c.color, Color::Rgb);
    let panel = c.screen(80, 24, true);
    assert!(panel.len() < 2048);
    assert_eq!(c.frame("safe", true).len(), 20);
    assert!(panel.contains("▁▂") && panel.contains("⡀⡄") && panel.contains("38;2;"));
    assert!(c.frame("safe", true).ends_with("\x1b[?2026l"));
    assert!(!c.screen(80, 24, false).contains('\x1b'));
    assert!(!c.choose("kitty") && !c.choose("\x1b]52;"));
    c.choose("c");
    assert_eq!(c, Capabilities::default());
}
#[test]
fn probe_fragments_timeout_unknown_lying_and_malformed_never_return_keys() {
    let t = Instant::now();
    let mut p = Probe::default();
    assert!(!p.expired(t + PROBE_TIME));
    p.begin(t);
    assert!(!p.expired(t + PROBE_TIME / 2));
    assert!(p.expired(t + PROBE_TIME));
    assert!(!p.result().unwrap());
    for state in 0..=4 {
        let mut p = Probe::default();
        for b in format!("\x1b[?2026;{state}$y").bytes() {
            p.feed(b).unwrap();
        }
        assert_eq!(p.result().unwrap(), state == 1 || state == 2);
        assert!(p.feed(b'P').is_err());
    }
    for bad in [
        b"P\n".as_slice(),
        b"\x1b[?2026;9$y",
        b"\x1b]52;c;secret\x07",
        b"\x1b[?2026;1$yQ\n",
    ] {
        let mut p = Probe::default();
        assert!(bad.iter().any(|b| p.feed(*b).is_err()));
    }
    let mut p = Probe::default();
    p.feed(27).unwrap();
    assert!(p.result().is_err());
    // A lying positive response can affect only presentation, never bitmap/authority.
    let mut c = Capabilities::default();
    let mut p = Probe::default();
    for b in b"\x1b[?2026;1$y" {
        p.feed(*b).unwrap();
    }
    c.synchronized = p.result().unwrap();
    assert!(!c.unicode);
    assert!(c.screen(80, 24, true).contains("bitmap: disabled"));
}
#[test]
fn native_scalars_have_bounded_cells_and_escape_invisible_controls() {
    assert_eq!(text::render("café 界 🦀", true), "café 界 🦀");
    assert_eq!(text::width("café 界 🦀"), 10);
    assert_eq!(
        text::render("🦀\u{1f3fb}\u{1f1e6}", true),
        "🦀\\u{1f3fb}\\u{1f1e6}"
    );
    assert_eq!(
        text::render("e\u{301}\u{202e}\x1b", true),
        "e\\u{301}\\u{202e}\\u{1b}"
    );
    for n in 0..140 {
        let label = subject::preview_with("Subject: café 界 🦀 e\u{301}\u{202e}", n, true);
        assert!(text::width(&label) <= n.min(119));
        assert!(!label.contains('\u{202e}'));
    }
    let mut input = Vec::new();
    for b in "界".bytes() {
        subject::feed_with(&mut input, b, true).unwrap();
    }
    assert_eq!(
        subject::feed_with(&mut input, 127, true).unwrap(),
        subject::Edit::Echo("\x08 \x08".repeat(2))
    );
}
#[test]
fn unicode_pages_wrap_by_cells_preserve_tokens_and_validate_offsets() {
    let body = format!(
        "{} faction-file:{}/{} end",
        "界🦀 café ".repeat(80),
        "a".repeat(32),
        "b".repeat(32)
    );
    let m = Message {
        room: 1,
        id: 1,
        author: "member",
        reply_to: None,
        body: &body,
    };
    let mut pos = 0;
    let mut pages = 0;
    let mut seen = String::new();
    loop {
        let p = m
            .page_with_capabilities(
                40,
                18,
                pos,
                MessageDetails {
                    subject: "café 界",
                    posted: "now",
                },
                true,
            )
            .unwrap();
        assert!(p.text.contains("Subject: café 界"));
        for line in p.text.lines() {
            assert!(text::width(line) <= 39 || line.starts_with("faction-file:"));
        }
        seen.push_str(&p.text);
        pages += 1;
        match p.next {
            Some(n) => {
                assert!(n > pos);
                pos = n
            }
            None => break,
        };
        assert!(pages < 100);
    }
    assert_eq!(seen.matches("界🦀").count(), 80);
    assert!(seen.contains(&format!(
        "faction-file:{}/{}",
        "a".repeat(32),
        "b".repeat(32)
    )));
    assert!(
        m.page_with_capabilities(
            40,
            18,
            1,
            MessageDetails {
                subject: "x",
                posted: "y"
            },
            true
        )
        .is_err()
    );
}
