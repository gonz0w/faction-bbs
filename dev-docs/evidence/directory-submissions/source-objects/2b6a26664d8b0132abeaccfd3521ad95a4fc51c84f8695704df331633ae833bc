use faction_terminal::{
    Key, Theme,
    directory::{Directory, Overlay, View},
};
const ONE: &str = "version=1\n[[boards]]\nslug='radio'\nname='Radio Board'\nsummary='Night radio.'\nhost='radio.example'\nport=2222\nlogin='bbs'\nreviewed='2026-10-22'\nvalid_until='2027-10-22'\n";
fn overlay(id: i64, revision: &str, slug: &str) -> Overlay {
    Overlay {
        id,
        revision: revision.into(),
        catalog: Directory::parse(&ONE.replace("radio", slug)).unwrap(),
        expired: false,
    }
}
#[test]
fn changed_snapshot_resets_card_before_endpoint_can_swap() {
    let owner = Directory::parse("version=1\nboards=[]").unwrap();
    let first = owner
        .compose(vec![
            overlay(1, "first", "radio"),
            overlay(2, "second", "other"),
        ])
        .unwrap();
    let mut view = View::default();
    view.synchronize(&first);
    view.feed(Key::Enter, &first, 40, 18);
    assert!(view.is_card());
    let replaced = owner.compose(vec![overlay(2, "second", "other")]).unwrap();
    view.feed(Key::Enter, &replaced, 40, 18);
    assert!(!view.is_card());
    assert!(!view.synchronize(&replaced));
    assert!(
        replaced
            .render(&view, 40, 18, false, &Theme::default())
            .contains("Page 1 / 1")
    );
    view.feed(Key::Enter, &replaced, 40, 18);
    let revised = owner
        .compose(vec![overlay(2, "new-revision", "other")])
        .unwrap();
    assert!(view.synchronize(&revised));
    assert!(!view.is_card());
    // Feed also guards callers that have not synchronized a removed card.
    let empty = owner.compose(vec![]).unwrap();
    view.feed(Key::Enter, &empty, 40, 18);
    assert!(!view.is_card());
}
#[test]
fn composition_validates_union_and_labels_owner_management() {
    let owner = Directory::parse(ONE).unwrap();
    assert_eq!(owner.curator_keys()[0].1, "radio.example");
    assert!(owner.compose(vec![overlay(1, "r", "radio")]).is_err());
    let union = owner.compose(vec![overlay(1, "r", "other")]).unwrap();
    assert_eq!(union.len(), 2);
    let mut view = View::default();
    view.feed(Key::Enter, &union, 80, 24);
    let frame = union.render(&view, 80, 24, false, &Theme::default());
    assert!(frame.contains("Owner-managed"));
    let many = (0..32)
        .map(|i| overlay(i, "r", &format!("board{i}")))
        .collect();
    assert!(owner.compose(many).is_err());
    let mut expired = overlay(2, "r", "old");
    expired.expired = true;
    let union = Directory::default().compose(vec![expired]).unwrap();
    let frame = union.render(&View::default(), 40, 18, false, &Theme::default());
    assert!(frame.contains("OLD"));
    assert!(frame.lines().count() <= 18);
    assert!(frame.lines().all(|line| line.len() < 40));
    assert!(!frame.contains('\x1b'));
}

#[test]
fn composition_accepts_exact_capacity_and_rejects_either_conflict_key() {
    let owner = Directory::parse(ONE).unwrap();
    let cards = (1..32)
        .map(|id| overlay(id, "revision", &format!("board{id}")))
        .collect();
    let full = owner.compose(cards).unwrap();
    assert_eq!(full.len(), 32);
    assert_eq!(full.curator_keys().len(), 1);
    let same_endpoint = Overlay {
        id: 1,
        revision: "revision".into(),
        catalog: Directory::parse(&ONE.replace("slug='radio'", "slug='other'")).unwrap(),
        expired: false,
    };
    assert!(owner.compose(vec![same_endpoint]).is_err());
    let same_slug = Overlay {
        id: 1,
        revision: "revision".into(),
        catalog: Directory::parse(&ONE.replace("radio.example", "other.example")).unwrap(),
        expired: false,
    };
    assert!(owner.compose(vec![same_slug]).is_err());
    assert!(
        Directory::default()
            .compose(vec![Overlay {
                id: 1,
                revision: "revision".into(),
                catalog: Directory::default(),
                expired: false
            }])
            .is_err()
    );
}
#[test]
fn unchanged_snapshot_keeps_card_but_reordering_consumes_selection() {
    let owner = Directory::default();
    let first = owner
        .compose(vec![
            overlay(1, "one", "first"),
            overlay(2, "two", "second"),
        ])
        .unwrap();
    let mut view = View::default();
    view.feed(Key::Down, &first, 40, 18);
    view.feed(Key::Enter, &first, 40, 18);
    assert!(view.is_card());
    let same = owner
        .compose(vec![
            overlay(1, "one", "first"),
            overlay(2, "two", "second"),
        ])
        .unwrap();
    assert!(!view.synchronize(&same));
    assert!(view.is_card());
    let reordered = owner
        .compose(vec![
            overlay(2, "two", "second"),
            overlay(1, "one", "first"),
        ])
        .unwrap();
    view.feed(Key::Hotkey(b'A'), &reordered, 40, 18);
    assert!(!view.is_card());
    view.feed(Key::Enter, &reordered, 40, 18);
    assert!(view.is_card());
}
