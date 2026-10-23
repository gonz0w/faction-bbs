use faction_terminal::text;
fn hex(s: &str) -> String {
    s.bytes().map(|b| format!("{b:02x}")).collect()
}
fn main() {
    for (name, raw) in [
        ("ascii", "Faction"),
        ("accent", "café"),
        ("cjk", "界"),
        ("emoji", "🦀"),
        ("ambiguous", "·"),
        ("blocks", "▁▂█"),
        ("braille", "⡀⣷⣿"),
        ("combining", "e\u{301}"),
        ("zwj", "👩\u{200d}💻"),
        ("flag", "🇨🇦"),
        ("modifier", "👍🏽"),
        ("bidi", "a\u{202e}b"),
    ] {
        let native = text::render(raw, true);
        let classic = text::render(raw, false);
        println!(
            "{name}\t{}\t{}\t{}\t{}\t{}",
            hex(raw),
            hex(&native),
            text::width(&native),
            hex(&classic),
            text::width(&classic)
        );
    }
}
