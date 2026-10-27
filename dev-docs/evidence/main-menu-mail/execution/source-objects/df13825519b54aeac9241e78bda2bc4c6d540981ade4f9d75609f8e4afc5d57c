use faction_terminal::home_mail::{self, Frame, Request};
const NONCE: &str = "0123456789abcdef0123456789abcdef";
#[test]
fn exact_mail_codec_is_separate_from_files_and_rejects_extensions() {
    for (frame, payload) in [
        (Frame::Offer, "offer".to_owned()),
        (Frame::Parked(NONCE.into()), format!("parked;{NONCE}")),
        (Frame::Returned(NONCE.into()), format!("returned;{NONCE}")),
    ] {
        assert_eq!(home_mail::parse_frame(&payload).unwrap(), frame);
        assert_eq!(
            home_mail::encode_frame(&frame).unwrap(),
            format!("\x1b]7770;FACTION-MAIL1;{payload}\x07")
        );
    }
    for bad in [
        "",
        "offer;extra",
        " offer",
        "offer ",
        "parked",
        "parked;ABCDEF0123456789abcdef0123456789ab",
        "returned;0123",
        "returned;0123456789abcdef0123456789abcdef;extra",
        "offer\x07",
        "hello;namespace;filekey",
    ] {
        assert!(home_mail::parse_frame(bad).is_err(), "{bad:?}");
    }
    for bad in [
        "",
        "0",
        "0123456789ABCDEF0123456789ABCDEF",
        "0123456789abcdef0123456789abcdeg",
    ] {
        assert!(!home_mail::is_valid_nonce(bad));
        assert!(home_mail::encode_frame(&Frame::Parked(bad.into())).is_err());
        assert!(home_mail::encode_request(&Request::Back(bad.into())).is_err());
    }
    for (request, word) in [
        (Request::Park(NONCE.into()), "park"),
        (Request::Back(NONCE.into()), "back"),
    ] {
        let line = format!("@MAIL1 {word} {NONCE}");
        assert_eq!(home_mail::encode_request(&request).unwrap(), line);
        assert_eq!(
            home_mail::parse_request(&format!("{line}\r\n")).unwrap(),
            request
        );
    }
    for bad in [
        format!("@MAIL1  park {NONCE}"),
        format!("@MAIL1 park {NONCE} extra"),
        format!("@MAIL1 park {NONCE} "),
        format!("@MAIL1 back {NONCE}\nextra"),
        format!("@FACTION1 park {NONCE}"),
    ] {
        assert!(home_mail::parse_request(&bad).is_err());
    }
}

#[test]
fn oversized_mail_requests_are_refused_before_token_allocation() {
    for line in [
        " ".repeat(100_000),
        format!("@MAIL1 park {NONCE}{}", "\r\n".repeat(100)),
    ] {
        assert_eq!(
            home_mail::parse_request(&line),
            Err("mail request too large")
        );
    }
}
