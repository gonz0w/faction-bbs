use faction_terminal::composer::{Command, Draft, Target};
#[test]
fn explicit_commands_preserve_paragraphs_and_never_implicitly_send() {
    let mut d = Draft::default();
    assert!(d.command(".send").is_err());
    assert_eq!(d.command("hello"), Ok(Command::Changed));
    d.command("").unwrap();
    d.command("remove").unwrap();
    d.command(".undo").unwrap();
    d.command("..send").unwrap();
    assert_eq!(d.body(), "hello\n\n.send");
    assert_eq!(d.command(".preview"), Ok(Command::Preview));
    assert_eq!(d.command(".cancel"), Ok(Command::Cancel));
    assert_eq!(d.command(".send"), Ok(Command::Send));
    assert_eq!(d.body(), "hello\n\n.send");
    d.command(".clear").unwrap();
    assert_eq!(d.lines(), 0);
    assert!(d.command(".send").is_err());
}
#[test]
fn byte_line_and_control_failures_preserve_the_previous_draft() {
    let mut d = Draft::default();
    d.command(&"界".repeat(341)).unwrap();
    let before = d.body().to_owned();
    assert!(d.command("x").is_err());
    assert!(d.command("\x1b[31m").is_err());
    assert_eq!(d.body(), before);
    d.command(".clear").unwrap();
    for _ in 0..32 {
        d.command("").unwrap();
    }
    assert!(d.command("").is_err());
    d.command(".undo").unwrap();
    d.command("ok").unwrap();
    assert_eq!(d.lines(), 32);
    assert!(d.command(".SEND").is_err());
}
#[test]
fn quotes_are_all_or_nothing_and_do_not_choose_a_destination() {
    let mut d = Draft::default();
    d.command("An answer:").unwrap();
    d.quote("one\n\ntwo").unwrap();
    assert_eq!(d.body(), "An answer:\n> one\n> \n> two");
    let before = d.body().to_owned();
    assert!(d.quote(&"x".repeat(1024)).is_err());
    assert!(d.quote("bad\x07").is_err());
    assert_eq!(d.body(), before);
}
#[test]
fn previews_page_without_claiming_a_real_post_and_keep_text_inert() {
    let mut d = Draft::default();
    for _ in 0..24 {
        d.command("界 wave\u{202e} text").unwrap();
    }
    for unicode in [false, true] {
        let mut offset = 0;
        let mut pages = 0;
        loop {
            let p = d
                .preview(
                    &Target {
                        room: 2,
                        reply: Some(1),
                        subject: "Échos 界",
                    },
                    (40, 18),
                    offset,
                    unicode,
                )
                .unwrap();
            assert!(p.text.contains("DRAFT PREVIEW (not sent)"));
            assert!(!p.text.contains("#0 by"));
            assert!(p.text.len() < 8192);
            assert!(!p.text.chars().any(|c| c.is_control() && c != '\n'));
            pages += 1;
            match p.next {
                Some(n) => {
                    assert!(n > offset);
                    offset = n;
                }
                None => break,
            }
        }
        assert!(pages > 1);
    }
    assert_eq!(d.lines(), 24);
}
#[test]
fn extended_input_retains_utf8_and_rejects_controls_and_overlong_lines() {
    use faction_terminal::subject::{Edit, feed_limited};
    let mut b = vec![];
    for v in "界".bytes() {
        feed_limited(&mut b, v, true, 1024).unwrap();
    }
    assert_eq!(
        feed_limited(&mut b, 127, true, 1024),
        Ok(Edit::Echo("\x08 \x08".repeat(2)))
    );
    for v in "Échos".bytes() {
        feed_limited(&mut b, v, false, 1024).unwrap();
    }
    assert_eq!(
        feed_limited(&mut b, b'\n', false, 1024),
        Ok(Edit::Submit("Échos".into()))
    );
    assert!(feed_limited(&mut b, 27, true, 1024).is_err());
    b = vec![b'x'; 1024];
    assert!(feed_limited(&mut b, b'x', false, 1024).is_err());
}
