use faction_terminal::{MESSAGE_BYTES, Message, Theme, section};
#[test]
fn wrapped_pages_preserve_source_order_and_never_emit_terminal_controls() {
    let body = format!(
        "> Quote from another member\n\n{}\n\x1b[2J\u{202e}é",
        "long-word-".repeat(100)
    );
    let msg = Message {
        room: 7,
        id: 42,
        author: "alice\x1b]52;secret",
        reply_to: Some(12),
        body: &body,
    };
    let mut offset = 0;
    let mut pages = 0;
    loop {
        let p = msg.page(40, 18, offset).unwrap();
        assert!(p.text.contains("Room #7"));
        assert!(p.text.contains("#42 by alice"));
        assert!(p.text.contains("Reply to #12"));
        assert!(!p.text.contains('\x1b'));
        assert!(p.text.lines().all(|l| l.len() <= 39));
        assert!(p.text.lines().count() + 4 <= 18);
        pages += 1;
        if let Some(next) = p.next {
            assert!(next > offset);
            offset = next;
        } else {
            break;
        }
        assert!(pages < 100);
    }
    assert!(pages > 1);
}
#[test]
fn page_boundaries_do_not_discard_words_blank_lines_or_escape_tails() {
    for body in [
        "abcdef".repeat(100),
        "\n".repeat(100),
        "a b c d ".repeat(100),
        "\x1b".repeat(100),
    ] {
        let msg = Message {
            room: 1,
            id: 1,
            author: "sysop",
            reply_to: None,
            body: &body,
        };
        let mut offset = 0;
        let mut count = 0;
        loop {
            let page = msg.page(80, 24, offset).unwrap();
            count += 1;
            if let Some(next) = page.next {
                assert!(next > offset);
                offset = next;
            } else {
                break;
            }
            assert!(count < 100);
        }
        assert!(count >= 1);
    }
    let msg = Message {
        room: 1,
        id: 1,
        author: "a",
        reply_to: None,
        body: "hello",
    };
    assert!(msg.page(80, 24, 99).is_err());
    let oversized = "a".repeat(MESSAGE_BYTES + 1);
    assert!(
        Message {
            body: &oversized,
            ..msg
        }
        .page(80, 24, 0)
        .is_err()
    );
}
#[test]
fn section_transitions_follow_the_theme_and_plain_text_has_no_escapes() {
    let theme = Theme::default();
    let ansi = section(&theme, "Rooms", true);
    assert!(ansi.contains("\x1b[2J\x1b[H"));
    assert!(ansi.contains(&theme.menu_styles().0.sgr()));
    let plain = section(&theme, "Read messages", false);
    assert!(plain.contains(theme.board()));
    assert!(!plain.contains('\x1b'));
}

#[test]
fn every_wrapped_word_is_present_once_in_chronological_order() {
    let words: Vec<_> = (0..150).map(|i| format!("word{i:03}")).collect();
    let body = words.join(" ");
    let msg = Message {
        room: 1,
        id: 2,
        author: "alice",
        reply_to: None,
        body: &body,
    };
    let mut offset = 0;
    let mut rendered = String::new();
    loop {
        let page = msg.page(40, 18, offset).unwrap();
        rendered.push_str(&page.text);
        if let Some(next) = page.next {
            offset = next;
        } else {
            break;
        }
    }
    let mut previous = 0;
    for word in words {
        assert_eq!(rendered.matches(&word).count(), 1);
        let index = rendered.find(&word).unwrap();
        assert!(index >= previous);
        previous = index;
    }
}

#[test]
fn file_references_survive_wrapping_as_one_native_client_token() {
    let link = format!("faction-file:{}/{}", "a".repeat(32), "b".repeat(32));
    for prefix in ["", "See this ", "a".repeat(36).as_str()] {
        let body = format!("{prefix}{link} [File: demo.txt (7 bytes)]");
        let msg = Message {
            room: 1,
            id: 1,
            author: "alice",
            reply_to: None,
            body: &body,
        };
        let mut offset = 0;
        let mut rendered = String::new();
        loop {
            let page = msg.page(40, 18, offset).unwrap();
            rendered.push_str(&page.text);
            if let Some(next) = page.next {
                offset = next;
            } else {
                break;
            }
        }
        assert_eq!(rendered.matches(&link).count(), 1);
        assert!(rendered.contains("demo.txt"));
    }
}
