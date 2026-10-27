#![cfg(feature = "native")]
use faction_omemo::{attachment, sce};
use zeroize::Zeroizing;
const SENT: u64 = 1_709_164_800;
// Independently written XML, fixed public laboratory key/IV/hash; no file claim.
const XML: &str = "<envelope xmlns='urn:xmpp:sce:1'><content><subject xmlns='jabber:client'>subject &amp; snow</subject><body xmlns='jabber:client'>body 雪</body><file-sharing xmlns='urn:xmpp:sfs:0' disposition='attachment'><file xmlns='urn:xmpp:file:metadata:0'><name>résumé.txt</name><media-type>text/plain</media-type><size>0</size><hash xmlns='urn:xmpp:hashes:2' algo='sha-256'>AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=</hash></file><sources><encrypted xmlns='urn:xmpp:esfs:0' cipher='urn:xmpp:ciphers:aes-256-gcm-nopadding:0'><key>AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=</key><iv>AgICAgICAgICAgIC</iv><sources xmlns='urn:xmpp:sfs:0'><url-data xmlns='http://jabber.org/protocol/url-data' target='https://blob.test/opaque'/></sources></encrypted></sources></file-sharing></content><from jid='alice@example'/><to jid='bob@example'/><time stamp='2024-02-29T00:00:00Z'/><rpad>p</rpad></envelope>";
fn decode(xml: &str) -> faction_omemo::Result<attachment::Content> {
    attachment::decode(xml.as_bytes(), "alice@example", "bob@example", SENT)
}
fn descriptor() -> attachment::Descriptor {
    attachment::Descriptor {
        name: Zeroizing::new("résumé.txt".into()),
        media_type: Zeroizing::new("text/plain".into()),
        size: 0,
        hash: Zeroizing::new([0; 32]),
        key: Zeroizing::new([1; 32]),
        iv: Zeroizing::new([2; 12]),
        locator: Zeroizing::new("https://blob.test/opaque".into()),
    }
}
#[test]
fn independent_xml_and_roundtrip_preserve_typed_private_metadata() {
    let content = decode(XML).unwrap();
    assert_eq!(content.text.subject.as_str(), "subject & snow");
    assert_eq!(content.text.body.as_str(), "body 雪");
    let d = content.attachment;
    assert_eq!(d.name.as_str(), "résumé.txt");
    assert_eq!(d.media_type.as_str(), "text/plain");
    assert_eq!(d.size, 0);
    assert!(*d.hash == [0; 32]);
    assert!(*d.key == [1; 32]);
    assert!(*d.iv == [2; 12]);
    assert_eq!(d.locator.as_str(), "https://blob.test/opaque");
    let encoded = attachment::encode("alice@example", "bob@example", SENT, "subject & snow", "body 雪", &d).unwrap();
    let again = attachment::decode(&encoded, "alice@example", "bob@example", SENT).unwrap();
    assert_eq!(again.attachment.name.as_str(), d.name.as_str());
    assert!(*again.attachment.key == *d.key);
    assert!(sce::decode(&encoded, "alice@example", "bob@example", SENT).is_err());
    assert!(sce::decode_network(&encoded, "alice@example", "bob@example").is_err());
    let plain = sce::encode("alice@example", "bob@example", SENT, "s", "b").unwrap();
    assert!(attachment::decode(&plain, "alice@example", "bob@example", SENT).is_err());
    let (network, stamp) = attachment::decode_network(XML.as_bytes(), "alice@example", "bob@example").unwrap();
    assert_eq!(stamp, SENT);
    assert_eq!(network.attachment.size, 0);
}
#[test]
fn exact_metadata_limits_and_checked_size_without_blob_allocation() {
    let mut d = descriptor();
    *d.name = "雪".repeat(85);
    *d.media_type = format!("{}/b", "a".repeat(125));
    *d.locator = format!("https://blob.test/{}", "x".repeat(512 - "https://blob.test/".len()));
    d.size = u64::MAX - 16;
    let xml = attachment::encode("alice@example", "bob@example", SENT, "s", "b", &d).unwrap();
    let got = attachment::decode(&xml, "alice@example", "bob@example", SENT).unwrap();
    assert_eq!(got.attachment.name.len(), 255);
    assert_eq!(got.attachment.media_type.len(), 127);
    assert_eq!(got.attachment.locator.len(), 512);
    assert_eq!(got.attachment.size, u64::MAX - 16);
    for size in [u64::MAX - 15, u64::MAX] {
        d.size = size;
        assert!(attachment::encode("alice@example", "bob@example", SENT, "s", "b", &d).is_err());
    }
    let base = XML.replace("<rpad>p</rpad>", "<rpad></rpad>");
    let exact = base.replace("<rpad></rpad>", &format!("<rpad>{}</rpad>", "x".repeat(8192 - base.len())));
    assert_eq!(exact.len(), 8192);
    assert!(decode(&exact).is_ok());
    assert!(decode(&(exact + " ")).is_err());
}
#[test]
fn filenames_media_types_and_inert_locators_reject_invalid_values() {
    for name in ["", ".", "..", "../file", "a/b", "a\\b", "bad\n", "bad\u{1b}", "bad\u{85}", &"x".repeat(256)] {
        let mut d = descriptor(); *d.name = name.into();
        assert!(attachment::encode("alice@example", "bob@example", SENT, "s", "b", &d).is_err());
        let xml = XML.replace("résumé.txt", name);
        assert!(decode(&xml).is_err());
    }
    for media in ["", "text", "/plain", "text/", "text/plain/extra", "text/plain;charset=utf8", "text/ plain", "téxt/plain", "text/\nplain", &format!("a/{}", "b".repeat(126))] {
        let mut d = descriptor(); *d.media_type = media.into();
        assert!(attachment::encode("alice@example", "bob@example", SENT, "s", "b", &d).is_err());
        assert!(decode(&XML.replace("text/plain", media)).is_err());
    }
    for locator in ["", "http://blob.test/a", "https:///a", "https://user:pass@blob.test/a", "https://blob.test/a?q=1", "https://blob.test/a#key", "https://blob.test/a b", "https://blob.test/雪", "https://blob.test/\n", &format!("https://blob.test/{}", "x".repeat(496))] {
        let mut d = descriptor(); *d.locator = locator.into();
        assert!(attachment::encode("alice@example", "bob@example", SENT, "s", "b", &d).is_err());
        // Newline in an attribute is XML-normalized into a rejected space.
        assert!(decode(&XML.replace("https://blob.test/opaque", locator)).is_err());
    }
}
#[test]
fn strict_binary_decimal_algorithm_and_shape_rejections() {
    let hash = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    for value in ["", "AA==", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB=", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\n", "__________________________________________8=", &"A".repeat(48)] {
        assert!(decode(&XML.replace(hash, value)).is_err());
    }
    for (field, old) in [("key", "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE="), ("iv", "AgICAgICAgICAgIC")] {
        for value in ["", "AA==", "%%%%", &"A".repeat(48)] {
            let xml = XML.replace(&format!("<{field}>{old}</{field}>"), &format!("<{field}>{value}</{field}>"));
            assert!(decode(&xml).is_err());
        }
    }
    for size in ["", "00", "01", "+1", "-1", " 1", "1 ", "1.0", "18446744073709551600", "18446744073709551616"] {
        assert!(decode(&XML.replace("<size>0</size>", &format!("<size>{size}</size>"))).is_err());
    }
    let mut bad = vec![
        XML.replace("disposition='attachment'", "disposition='inline'"),
        XML.replace("disposition='attachment'", ""),
        XML.replace("disposition='attachment'", "disposition='attachment' extra='x'"),
        XML.replace("sha-256", "sha-512"),
        XML.replace("urn:xmpp:ciphers:aes-256-gcm-nopadding:0", "urn:xmpp:ciphers:aes-256-cbc-pkcs7:0"),
        XML.replace("urn:xmpp:esfs:0", "urn:xmpp:esfs:99"),
        XML.replace("urn:xmpp:file:metadata:0", "other"),
        XML.replace("urn:xmpp:hashes:2", "other"),
        XML.replace("http://jabber.org/protocol/url-data", "other"),
        XML.replace("target='https://blob.test/opaque'", "target='https://blob.test/opaque' xml:lang='en'"),
        XML.replace("<file xmlns", "<file extra='x' xmlns"),
        XML.replace("<name>", "<name extra='x'>"),
        XML.replace("<name>résumé.txt</name>", "<name><nested/></name>"),
        XML.replace("<name>résumé.txt</name>", ""),
        XML.replace("<sources><encrypted", "<sources>stray<encrypted"),
        XML.replace("</file>", "<unknown/></file>"),
        XML.replace("<content>", "<content><!-- comment -->"),
        XML.replace("<content>", "<content><?pi x?>"),
        XML.replace("résumé.txt", "<![CDATA[résumé.txt]]>"),
        format!("<!DOCTYPE envelope []>{XML}"),
        format!("<?xml version='1.0'?>{XML}"),
        format!("{XML}<extra/>"),
        XML.replace("<rpad>p</rpad>", "<rpad/>"),
        XML.replace("<rpad>p</rpad>", "<rpad><nested/></rpad>"),
        XML.replace("<rpad>p</rpad>", &format!("<rpad>{}</rpad>", "<n/>".repeat(130))),
    ];
    for element in ["<name>résumé.txt</name>", "<media-type>text/plain</media-type>", "<size>0</size>", "<iv>AgICAgICAgICAgIC</iv>"] {
        bad.push(XML.replace(element, &format!("{element}{element}")));
        bad.push(XML.replace(element, ""));
    }
    let sharing = XML.split("<file-sharing").nth(1).unwrap().split("</file-sharing>").next().unwrap();
    bad.push(XML.replace("</file-sharing>", &format!("</file-sharing><file-sharing{sharing}</file-sharing>")));
    for xml in bad { assert!(decode(&xml).is_err(), "accepted invalid candidate XML"); }
    assert!(attachment::decode(&[0xff], "alice@example", "bob@example", SENT).is_err());
}
#[test]
fn exact_inner_addresses_time_and_text_are_preserved() {
    assert!(attachment::decode(XML.as_bytes(), "wrong@example", "bob@example", SENT).is_err());
    assert!(attachment::decode(XML.as_bytes(), "alice@example", "wrong@example", SENT).is_err());
    for time in [SENT - 301, SENT + 301, u64::MAX] { assert!(attachment::decode(XML.as_bytes(), "alice@example", "bob@example", time).is_err()); }
    for time in [SENT - 300, SENT + 300] { assert!(attachment::decode(XML.as_bytes(), "alice@example", "bob@example", time).is_ok()); }
    for stamp in ["2023-02-29T00:00:00Z", "2024-02-29T00:00:00.0Z", "2024-02-29T00:00:00+00:00", "2024-02-29T00:00:60Z"] { assert!(decode(&XML.replace("2024-02-29T00:00:00Z", stamp)).is_err()); }
    for (s,b) in [("", "b"), ("s", ""), (&"s".repeat(129), "b"), ("s", &"b".repeat(1025)), ("s", "bad\0")] { assert!(attachment::encode("alice@example", "bob@example", SENT, s, b, &descriptor()).is_err()); }
    let xml = attachment::encode("alice@example", "bob@example", SENT, &"s".repeat(128), &"b".repeat(1024), &descriptor()).unwrap();
    assert!(attachment::decode(&xml, "alice@example", "bob@example", SENT).is_ok());
}
