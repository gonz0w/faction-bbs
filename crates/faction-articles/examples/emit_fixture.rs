use faction_articles::{Article, Headers};
fn main() {
    let case = std::env::args().nth(1).unwrap_or_else(|| "default".into());
    let (subject, body) = match case.as_str() {
        "default" => ("Midnight signal", "Unicode: café ☃\n> reply\n".into()),
        "unicode" => ("café ☃ frequencies", "Unicode reply\n".into()),
        "long" => ("Long line", "L".repeat(1024)),
        "no-newline" => ("Final byte", "last byte".into()),
        "whitespace" => ("Whitespace", "tail \t\n\n".into()),
        _ => panic!("unknown synthetic fixture"),
    };
    let article = Article::new(
        Headers {
            message_id: "<synthetic@fixture.invalid>".into(),
            from: "author@fixture.invalid".into(),
            date: "21 Oct 2026 18:00:00 +0000".into(),
            subject: subject.into(),
            newsgroups: vec!["faction.local.r1".into()],
            path: "not-for-mail".into(),
            references: vec![
                "<root@fixture.invalid>".into(),
                "<parent@fixture.invalid>".into(),
            ],
        },
        body,
    )
    .unwrap();
    print!("{}", article.to_wire().unwrap());
}
