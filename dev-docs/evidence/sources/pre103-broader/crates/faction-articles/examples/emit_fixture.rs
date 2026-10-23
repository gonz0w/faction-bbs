use faction_articles::{Article, Headers};
fn main() {
    let article = Article::new(
        Headers {
            message_id: "<synthetic@fixture.invalid>".into(),
            from: "author@fixture.invalid".into(),
            date: "21 Oct 2026 18:00:00 +0000".into(),
            subject: "Midnight signal".into(),
            newsgroups: vec!["faction.local.r1".into()],
            path: "not-for-mail".into(),
            references: vec![
                "<root@fixture.invalid>".into(),
                "<parent@fixture.invalid>".into(),
            ],
        },
        "Unicode: café ☃\n> reply\n".into(),
    )
    .unwrap();
    print!("{}", article.to_wire().unwrap());
}
