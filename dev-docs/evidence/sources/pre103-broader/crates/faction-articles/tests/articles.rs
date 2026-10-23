use faction_articles::{Article, BODY_BYTES, Error, Headers, LINE_BYTES, REFERENCES, WIRE_BYTES};
fn headers() -> Headers {
    Headers {
        message_id: "<post-1@home.example>".into(),
        from: "alice@home.example".into(),
        date: "21 Oct 2026 18:00:00 +0000".into(),
        subject: "After hours".into(),
        newsgroups: vec!["faction.lobby".into()],
        path: "home.example!not-for-mail".into(),
        references: vec!["<parent@peer.example>".into()],
    }
}
fn wire() -> String {
    Article::new(
        headers(),
        "Hello, world.\n> quoted line\nUnicode: café ☃\n\n".into(),
    )
    .unwrap()
    .to_wire()
    .unwrap()
}
#[test]
fn article_roundtrip_preserves_text_identity_replies_and_trailing_blank_lines() {
    let a = Article::parse(wire().as_bytes()).unwrap();
    assert_eq!(a.headers(), &headers());
    assert_eq!(
        a.body(),
        "Hello, world.\n> quoted line\nUnicode: café ☃\n\n"
    );
    assert_eq!(a.to_wire().unwrap(), wire());
    assert!(!wire().contains("Approved:"));
    for text in ["", "\n", "hello", "hello\n", "\n\n"] {
        let a = Article::new(headers(), text.into()).unwrap();
        assert_eq!(Article::parse(a.to_wire().unwrap().as_bytes()).unwrap(), a);
    }
}
#[test]
fn casefolded_headers_and_folded_references_have_one_canonical_encoding() {
    let mut h = headers();
    h.references.push("<second@peer.example>".into());
    let a = Article::new(h, "text".into()).unwrap();
    let text = a
        .to_wire()
        .unwrap()
        .replace("Message-ID:", "mEsSaGe-Id:")
        .replace("\r\n <second", "\r\n\t<second");
    assert_eq!(Article::parse(text.as_bytes()).unwrap(), a);
    assert_eq!(
        Article::parse(a.to_wire().unwrap().as_bytes())
            .unwrap()
            .to_wire()
            .unwrap(),
        a.to_wire().unwrap()
    );
}
#[test]
fn duplicate_missing_unknown_and_authority_headers_fail_closed() {
    let valid = wire();
    for extra in [
        "From: other@peer.example\r\n",
        "fRoM: other@peer.example\r\n",
        "Approved: root@home.example\r\n",
        "Control: cancel <post-1@home.example>\r\n",
        "X-Exec: /bin/sh\r\n",
    ] {
        assert!(Article::parse(format!("{extra}{valid}").as_bytes()).is_err());
    }
    for field in [
        "Date",
        "From",
        "Message-ID",
        "Newsgroups",
        "Path",
        "Subject",
        "MIME-Version",
        "Content-Type",
        "Content-Transfer-Encoding",
    ] {
        let text = valid
            .split_inclusive("\r\n")
            .filter(|line| !line.starts_with(&format!("{field}:")))
            .collect::<String>();
        assert!(Article::parse(text.as_bytes()).is_err(), "{field}");
    }
}
#[test]
fn controls_invalid_utf8_and_line_ending_injection_are_rejected() {
    for body in ["\x1b[2J", "\x00", "\r\nApproved: root", "\u{009b}2J"] {
        assert!(Article::new(headers(), body.into()).is_err());
    }
    let valid = wire();
    for text in [
        valid.replace("\r\n", "\n"),
        valid.replace("Subject: After hours", "Subject: After\nhours"),
        format!("\tstray fold\r\n{valid}"),
        valid.trim_end_matches("\r\n").into(),
    ] {
        assert!(Article::parse(text.as_bytes()).is_err());
    }
    assert!(Article::parse(&[0xff, 0xfe]).is_err());
    let mut h = headers();
    h.subject = "hello\r\nApproved: root".into();
    assert!(Article::new(h, "ok".into()).is_err());
}
#[test]
fn date_profile_checks_calendar_leap_year_time_and_timezone() {
    for value in ["29 Feb 2024 23:59:59 -0600", "01 Jan 2000 00:00:00 +0000"] {
        let mut h = headers();
        h.date = value.into();
        assert!(Article::new(h, "".into()).is_ok());
    }
    for value in [
        "29 Feb 2025 23:00:00 +0000",
        "31 Apr 2026 12:00:00 +0000",
        "00 Oct 2026 18:00:00 +0000",
        "21 Oct 2026 24:00:00 +0000",
        "21 Oct 2026 18:60:00 +0000",
        "21 Oct 2026 18:00:00 +2460",
        "é1 Oct 2026 18:00:00 +0000",
    ] {
        let mut h = headers();
        h.date = value.into();
        assert!(Article::new(h, "".into()).is_err(), "{value}");
    }
}
#[test]
fn malformed_ids_groups_paths_and_reply_cycles_are_not_authority() {
    for id in [
        "post-1@home.example",
        "<a@home.example>\r\n",
        "<a@b@home.example>",
        "<a@../secret>",
        "<.a@home.example>",
    ] {
        let mut h = headers();
        h.message_id = id.into();
        assert!(Article::new(h, "".into()).is_err());
    }
    for group in [
        "",
        "faction..lobby",
        "../../secrets",
        "faction.lobby\r\n",
        "Faction.Lobby",
    ] {
        let mut h = headers();
        h.newsgroups = vec![group.into()];
        assert!(Article::new(h, "".into()).is_err());
    }
    let mut h = headers();
    h.path = "home.example!!not-for-mail".into();
    assert!(Article::new(h, "".into()).is_err());
    let mut h = headers();
    h.references = vec![h.message_id.clone()];
    assert!(Article::new(h, "".into()).is_err());
    let mut h = headers();
    h.references.push(h.references[0].clone());
    assert_eq!(Article::new(h, "".into()), Err(Error::Duplicate));
}
#[test]
fn byte_line_reference_and_group_limits_are_enforced() {
    assert_eq!(
        Article::parse(&vec![b'a'; WIRE_BYTES + 1]),
        Err(Error::Limit)
    );
    assert_eq!(
        Article::new(headers(), "a".repeat(BODY_BYTES + 1)),
        Err(Error::Limit)
    );
    assert!(Article::new(headers(), "a".repeat(LINE_BYTES)).is_ok());
    assert_eq!(
        Article::new(headers(), "a".repeat(LINE_BYTES + 1)),
        Err(Error::Limit)
    );
    let mut h = headers();
    h.references = (0..=REFERENCES)
        .map(|i| format!("<p{i}@peer.example>"))
        .collect();
    assert_eq!(Article::new(h, "".into()), Err(Error::Limit));
    let mut h = headers();
    h.newsgroups = vec!["a".into(); 5];
    assert!(Article::new(h, "".into()).is_err());
    assert!(Article::new(headers(), "\n".repeat(BODY_BYTES)).is_err());
}
#[test]
fn attachments_html_and_unsupported_transfer_encoding_are_rejected() {
    for (old, new) in [
        ("text/plain; charset=utf-8", "text/html; charset=utf-8"),
        ("text/plain; charset=utf-8", "multipart/mixed; boundary=x"),
        ("Encoding: 8bit", "Encoding: base64"),
    ] {
        assert_eq!(
            Article::parse(wire().replace(old, new).as_bytes()),
            Err(Error::Unsupported)
        );
    }
}

#[test]
fn handwritten_article_fixture_and_unsupported_header_encodings() {
    let a = Article::parse(include_bytes!("fixtures/plain.article")).unwrap();
    assert_eq!(a.headers().message_id, "<fixture-1@home.example>");
    assert_eq!(a.body(), "This is an offline article fixture.");
    assert!(a.headers().references.is_empty());
    for text in [
        wire().replace("Subject: After hours", "Subject: =?utf-8?b?Y2Fmw6k=?="),
        wire().replace("References: <parent@peer.example>", "References: "),
        wire().replace("Subject: After hours", "Subject:After hours"),
    ] {
        assert!(Article::parse(text.as_bytes()).is_err());
    }
    let header = format!("Subject: {}\r\n", "a".repeat(5000));
    assert_eq!(
        Article::parse(format!("{header}{}", wire()).as_bytes()),
        Err(Error::Limit)
    );
}

#[test]
fn duplicate_identity_compares_normalized_headers_and_all_content_strictly() {
    use faction_articles::IdentityMatch;
    let a = Article::parse(wire().as_bytes()).unwrap();
    let folded = wire()
        .replace("Message-ID:", "message-id:")
        .replace("References: <parent", "References: \r\n <parent");
    let duplicate = Article::parse(folded.as_bytes()).unwrap();
    assert_eq!(a.compare_identity(&duplicate), IdentityMatch::Duplicate);
    for field in 0..7 {
        let mut h = a.headers().clone();
        let mut body = a.body().to_owned();
        match field {
            0 => h.subject = "changed".into(),
            1 => h.from = "other@peer.example".into(),
            2 => h.date = "22 Oct 2026 18:00:00 +0000".into(),
            3 => h.newsgroups = vec!["other.group".into()],
            4 => h.path = "other.example!not-for-mail".into(),
            5 => h.references.clear(),
            _ => body.push('x'),
        }
        assert_eq!(
            a.compare_identity(&Article::new(h, body).unwrap()),
            IdentityMatch::Conflict
        );
    }
    let mut h = a.headers().clone();
    h.message_id = "<other@peer.example>".into();
    assert_eq!(
        a.compare_identity(&Article::new(h, a.body().into()).unwrap()),
        IdentityMatch::Different
    );
}
