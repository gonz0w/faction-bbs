#![cfg(feature = "native")]
use faction_omemo::sce;

const SENT: u64 = 1_709_164_800; // 2024-02-29T00:00:00Z
fn raw(subject: &str, body: &str, stamp: &str, pad: &str) -> String {
    format!("<envelope xmlns='urn:xmpp:sce:1'><content><subject xmlns='jabber:client'>{subject}</subject><body xmlns='jabber:client'>{body}</body></content><from jid='alice@example'/><to jid='bob@example'/><time stamp='{stamp}'/><rpad>{pad}</rpad></envelope>")
}
fn accepts(text: &str, sent: u64) -> bool {
    sce::decode(text.as_bytes(), "alice@example", "bob@example", sent).is_ok()
}

#[test]
fn accepts_independently_written_sce_and_inert_escaped_unicode() {
    let text = raw("Résumé &amp; &lt;test&gt;", "雪 &#13;\n&lt;script&gt;", "2024-02-29T00:00:00Z", "pad");
    let content = sce::decode(text.as_bytes(), "alice@example", "bob@example", SENT).unwrap();
    assert_eq!(content.subject.as_str(), "Résumé & <test>");
    assert_eq!(content.body.as_str(), "雪 \r\n<script>");
    let encoded = sce::encode("alice@example", "bob@example", SENT, &content.subject, &content.body).unwrap();
    assert!(accepts(std::str::from_utf8(&encoded).unwrap(), SENT));
    assert_ne!(encoded, sce::encode("alice@example", "bob@example", SENT, &content.subject, &content.body).unwrap());
}

#[test]
fn rejects_wrong_binding_shape_namespace_and_active_xml() {
    let valid = raw("subject", "body", "2024-02-29T00:00:00Z", "p");
    assert!(accepts(&valid, SENT));
    for bad in [
        valid.replace("alice@example", "mallory@example"),
        valid.replace("bob@example", "mallory@example"),
        valid.replace("urn:xmpp:sce:1", "urn:xmpp:sce:0"),
        valid.replace("jabber:client", "other"),
        valid.replace("<from jid", "<from unknown='1' jid"),
        valid.replace("<rpad>p</rpad>", "<rpad/>"),
        valid.replace("<rpad>p</rpad>", "<rpad><nested/></rpad>"),
        valid.replace("<rpad>p</rpad>", "<rpad>p</rpad><rpad>p</rpad>"),
        valid.replace("<rpad>p</rpad>", "<unknown>p</unknown>"),
        valid.replace("<content>", "<content>stray"),
        valid.replace("<from jid='alice@example'/>", "<from jid='alice@example'>stray</from>"),
        valid.replace("<body xmlns='jabber:client'>body</body>", "<body xmlns='jabber:client'><b>body</b></body>"),
        valid.replace("<body xmlns='jabber:client'>body</body>", "<body xmlns='jabber:client' xml:lang='en'>body</body>"),
        valid.replace("<content>", "<content><!--comment-->"),
        valid.replace("<content>", "<content><?pi value?>"),
        valid.replace("body</body>", "<![CDATA[body]]></body>"),
        format!("<!DOCTYPE envelope []>{valid}"),
        format!("<!DOCTYPE envelope [<!ENTITY x 'secret'>]>{}", valid.replace("body</body>", "&x;</body>")),
        format!("<?xml version='1.0'?>{valid}"),
        format!("{valid}<extra/>"),
    ] { assert!(!accepts(&bad, SENT), "accepted invalid SCE shape"); }
    assert!(sce::decode(&[0xff], "alice@example", "bob@example", SENT).is_err());
}

#[test]
fn timestamp_subset_calendar_range_and_enclosing_time_are_checked() {
    let valid = raw("s", "b", "2024-02-29T00:00:00Z", "p");
    for sent in [SENT-300, SENT, SENT+300] { assert!(accepts(&valid, sent)); }
    for sent in [SENT-301, SENT+301, 253_402_300_800, u64::MAX] { assert!(!accepts(&valid, sent)); }
    for stamp in ["2023-02-29T00:00:00Z", "2100-02-29T00:00:00Z", "2024-04-31T00:00:00Z", "2024-00-01T00:00:00Z", "2024-01-00T00:00:00Z", "2024-13-01T00:00:00Z", "2024-02-29T24:00:00Z", "2024-02-29T00:60:00Z", "2024-02-29T00:00:60Z", "2024-02-29T00:00:00+00:00", "2024-02-29T00:00:00.0Z", "2024-02-29t00:00:00z", "1969-12-31T23:59:59Z", "10000-01-01T00:00:00Z"] {
        assert!(!accepts(&raw("s", "b", stamp, "p"), SENT));
    }
    for (epoch, stamp) in [(0,"1970-01-01T00:00:00Z"), (951_782_400,"2000-02-29T00:00:00Z"), (253_402_300_799,"9999-12-31T23:59:59Z")] {
        assert!(accepts(&raw("s", "b", stamp, "p"), epoch));
        let encoded = sce::encode("alice@example", "bob@example", epoch, "s", "b").unwrap();
        assert!(std::str::from_utf8(&encoded).unwrap().contains(stamp));
    }
}

#[test]
fn exact_utf8_and_envelope_bounds_and_large_received_padding() {
    let subject = "雪".repeat(42)+"ab";
    let body = "é".repeat(512);
    let encoded = sce::encode("alice@example", "bob@example", SENT, &subject, &body).unwrap();
    assert!(sce::decode(&encoded, "alice@example", "bob@example", SENT).is_ok());
    for (s,b) in [("", "body"), ("subject", ""), (&"s".repeat(129), "body"), ("subject", &"b".repeat(1025)), ("subject", "bad\0"), ("bad\u{1b}", "body")] {
        assert!(sce::encode("alice@example", "bob@example", SENT, s,b).is_err());
    }
    assert!(sce::encode("", "bob@example", SENT, "s", "b").is_err());
    assert!(sce::encode("alice@example", "bob@example", u64::MAX, "s", "b").is_err());
    let base = raw("s", "b", "2024-02-29T00:00:00Z", "");
    let exact = raw("s", "b", "2024-02-29T00:00:00Z", &"x".repeat(8192-base.len()));
    assert_eq!(exact.len(),8192);
    assert!(accepts(&exact,SENT));
    assert!(!accepts(&(exact+" "),SENT));
    assert!(!accepts(&raw(&"s".repeat(129),"b","2024-02-29T00:00:00Z","p"),SENT));
    assert!(!accepts(&raw("s",&"é".repeat(513),"2024-02-29T00:00:00Z","p"),SENT));
    assert!(!accepts(&raw("s","b","2024-02-29T00:00:00Z",&"<n/>".repeat(130)),SENT));
}
