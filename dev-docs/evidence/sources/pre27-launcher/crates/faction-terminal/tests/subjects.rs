use faction_terminal::subject::{self, Edit};
#[test]
fn split_utf8_echo_backspace_and_byte_boundary() {
    let mut input = Vec::new();
    for (byte, echo) in [(b'A', "A"), (0xc3, ""), (0xa9, "\\u{e9}")] {
        assert_eq!(subject::feed(&mut input, byte), Ok(Edit::Echo(echo.into())));
    }
    assert_eq!(
        subject::feed(&mut input, 127),
        Ok(Edit::Echo("\x08 \x08".repeat(6)))
    );
    // Erase a pending scalar without deleting the preceding complete character.
    subject::feed(&mut input, 0xf0).unwrap();
    assert_eq!(subject::feed(&mut input, 8), Ok(Edit::Echo(String::new())));
    assert_eq!(
        subject::feed(&mut input, b'\n'),
        Ok(Edit::Submit("A".into()))
    );
    for byte in "é".repeat(80).bytes() {
        subject::feed(&mut input, byte).unwrap();
    }
    assert_eq!(input.len(), 160);
    assert!(subject::feed(&mut input, b'x').is_err());
}
#[test]
fn malformed_controls_and_incomplete_submit_fail() {
    for bad in [
        vec![0xff],
        vec![0xc0, 0xaf],
        vec![0xed, 0xa0, 0x80],
        vec![0xf4, 0x90, 0x80, 0x80],
        vec![0x1b],
        vec![0xc2, 0x85],
        vec![0xe2, b'x'],
        vec![0xe2, b'\n'],
    ] {
        let mut input = Vec::new();
        assert!(
            bad.into_iter()
                .any(|b| subject::feed(&mut input, b).is_err())
        );
    }
}
#[test]
fn preview_is_inert_and_bounded_in_display_cells() {
    for width in 0..=140 {
        let text = subject::preview("Subject: café 界 🦀 e\u{301}\u{202e}\x1b[2J\n", width);
        assert!(text.is_ascii());
        assert!(text.bytes().all(|b| (32..=126).contains(&b)));
        assert!(text.len() <= width.min(119));
        if let Some(start) = text.rfind("\\u{") {
            assert!(text[start..].contains('}'), "{text}");
        }
    }
    assert_eq!(subject::preview("café", 20), "caf\\u{e9}");
}
