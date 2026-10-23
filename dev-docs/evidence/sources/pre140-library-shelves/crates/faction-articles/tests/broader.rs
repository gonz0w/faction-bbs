use faction_articles::{Article, Error, Headers, PROFILE_VERSION};
fn headers() -> Headers {
    Headers {
        message_id: "<synthetic@fixture.invalid>".into(),
        from: "author@fixture.invalid".into(),
        date: "21 Oct 2026 18:00:00 +0000".into(),
        subject: "Midnight signal".into(),
        newsgroups: vec!["faction.local.r1".into()],
        path: "not-for-mail".into(),
        references: vec![],
    }
}
fn wire() -> String {
    Article::new(headers(), "exact body".into())
        .unwrap()
        .to_wire()
        .unwrap()
}
#[test]
fn utf8_encoded_subjects_roundtrip_and_adjacent_words_preserve_spaces() {
    assert_eq!(PROFILE_VERSION, 2);
    for subject in [
        "café ☃",
        &"界".repeat(53),
        "literal =? marker",
        "spaces  stay  here",
    ] {
        let mut h = headers();
        h.subject = subject.into();
        let a = Article::new(h, "body".into()).unwrap();
        let wire = a.to_wire().unwrap();
        assert!(wire.is_ascii());
        for word in wire
            .split_ascii_whitespace()
            .filter(|s| s.starts_with("=?"))
        {
            assert!(word.len() <= 75);
        }
        assert_eq!(Article::parse(wire.as_bytes()).unwrap(), a);
    }
    for encoded in ["=?UTF-8?B?Y2Fmw6k=?=", "=?utf-8?Q?caf=C3=A9?="] {
        let text = wire().replace("Subject: Midnight signal", &format!("Subject: {encoded}"));
        assert_eq!(
            Article::parse(text.as_bytes()).unwrap().headers().subject,
            "café"
        );
    }
    let text = wire().replace(
        "Subject: Midnight signal",
        "Subject: =?UTF-8?Q?one_?=\r\n =?UTF-8?Q?two?=",
    );
    assert_eq!(
        Article::parse(text.as_bytes()).unwrap().headers().subject,
        "one two"
    );
}
#[test]
fn decoded_subject_controls_unknown_charsets_and_malformed_words_fail() {
    for encoded in [
        "=?UTF-8?Q?evil=0D=0AApproved:_root?=",
        "=?UTF-8?B?Gw==?=",
        "=?UTF-8?B?/w==?=",
        "=?UTF-16?B?YQ==?=",
        "=?UTF-8?Q?broken=GG?=",
        "=?UTF-8?B?YQ?=",
        "=?UTF-8?Q?word?=suffix",
    ] {
        assert!(
            Article::parse(
                wire()
                    .replace("Subject: Midnight signal", &format!("Subject: {encoded}"))
                    .as_bytes()
            )
            .is_err(),
            "{encoded}"
        );
    }
}
#[test]
fn quoted_printable_preserves_long_lines_whitespace_and_every_final_newline() {
    for body in [
        "",
        "no newline",
        "one\n",
        "two\n\n",
        "space \t\n",
        "Unicode café ☃",
        &"a".repeat(1024),
    ] {
        let a = Article::new(headers(), body.into()).unwrap();
        let wire = a.to_wire().unwrap();
        assert!(wire.contains("Encoding: quoted-printable"));
        let encoded = wire.split_once("\r\n\r\n").unwrap().1;
        assert!(encoded.split("\r\n").all(|line| line.len() <= 76));
        assert_eq!(Article::parse(wire.as_bytes()).unwrap(), a);
    }
}
#[test]
fn base64_7bit_8bit_defaults_and_quoted_charset_are_bounded_text_only() {
    let prefix = "Date: Wed, 21 Oct 2026 18:00 +0000\r\nFrom: author@fixture.invalid\r\nMessage-ID: <x@fixture.invalid>\r\nNewsgroups: faction.local.r1\r\nPath: not-for-mail\r\nSubject: plain\r\n";
    let plain = Article::parse(format!("{prefix}\r\nASCII body\r\n").as_bytes()).unwrap();
    assert_eq!(plain.body(), "ASCII body\n");
    assert_eq!(plain.headers().date, "21 Oct 2026 18:00:00 +0000");
    for (transfer, body, expected) in [
        ("base64", "Y2Fmw6k=\r\n", "café"),
        ("8bit", "café\r\n", "café\n"),
        ("7bit", "ascii\r\n", "ascii\n"),
        ("quoted-printable", "caf=C3=A9=\r\n", "café"),
    ] {
        let wire = format!(
            "{prefix}MIME-Version: 1.0\r\nContent-Type: Text/Plain; Charset=\"UTF-8\"\r\nContent-Transfer-Encoding: {transfer}\r\n\r\n{body}"
        );
        let a = Article::parse(wire.as_bytes()).unwrap();
        assert_eq!(a.body(), expected);
        assert_eq!(Article::parse(a.to_wire().unwrap().as_bytes()).unwrap(), a);
    }
}
#[test]
fn malformed_mime_decoded_controls_overflow_and_bad_weekdays_fail() {
    for (transfer, body) in [
        ("base64", "!!!!\r\n"),
        ("base64", "Gw==\r\n"),
        ("quoted-printable", "=1B[2J\r\n"),
        ("quoted-printable", "broken=GG\r\n"),
        ("7bit", "é\r\n"),
        ("binary", "body\r\n"),
    ] {
        let s = wire().replace(
            "Content-Transfer-Encoding: quoted-printable",
            &format!("Content-Transfer-Encoding: {transfer}"),
        );
        let head = s.split_once("\r\n\r\n").unwrap().0;
        assert!(Article::parse(format!("{head}\r\n\r\n{body}").as_bytes()).is_err());
    }
    for date in [
        "Thu, 21 Oct 2026 18:00:00 +0000",
        "Wed, 32 Oct 2026 18:00 +0000",
        "21 Oct 2026 18:00 GMT",
    ] {
        let mut h = headers();
        h.date = date.into();
        assert!(Article::new(h, "x".into()).is_err());
    }
    let mut h = headers();
    h.subject = "界".repeat(54);
    assert!(Article::new(h, "x".into()).is_err());
    assert_eq!(Article::new(headers(), "é".repeat(8192)), Err(Error::Limit));
}
