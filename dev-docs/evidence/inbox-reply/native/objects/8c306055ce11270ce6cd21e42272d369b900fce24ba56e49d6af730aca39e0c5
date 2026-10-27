use faction_xmpp::{Error, Home, MAX_OUTPUT_BYTES, validate_output};
use std::net::SocketAddr;

#[test]
fn invalid_home_never_becomes_transport_authority() {
    assert!(matches!(
        Home::new(
            "127.0.0.1:5223".parse().unwrap(),
            "home.test",
            "alice@home.test/device",
            b"invalid CA",
            [1; 32]
        ),
        Err(Error::Config)
    ));
    let endpoint: SocketAddr = "127.0.0.1:5223".parse().unwrap();
    assert!(
        Home::new(
            endpoint,
            "home.test",
            "alice@home.test/device",
            include_bytes!("fixtures/local-ca.pem"),
            [1; 32]
        )
        .is_ok()
    );
    for (domain, jid) in [
        ("home.test:5223", "alice@home.test/device"),
        ("home.test", "alice@away.test/device"),
        ("home.test", "alice@home.test"),
        ("home.test", "home.test/device"),
        ("home.test", "alice@home.test/dev\u{001b}ice"),
        ("home.test", "Alice@home.test/device"),
        ("-home.test", "alice@-home.test/device"),
    ] {
        assert!(matches!(
            Home::new(
                endpoint,
                domain,
                jid,
                include_bytes!("fixtures/local-ca.pem"),
                [1; 32]
            ),
            Err(Error::Config)
        ));
    }
}

#[test]
fn escaped_encoded_output_is_bounded_before_any_write() {
    use faction_xmpp::Element;
    let ordinary = Element::builder("message", "jabber:client")
        .append("safe")
        .build();
    assert_eq!(validate_output(&ordinary), Ok(()));
    let too_large = Element::builder("message", "jabber:client")
        .append("&".repeat(MAX_OUTPUT_BYTES / 2))
        .build();
    assert_eq!(validate_output(&too_large), Err(Error::OutputLimit));
    let mut nested = ordinary;
    for _ in 0..33 {
        nested = Element::builder("x", "jabber:client")
            .append(nested)
            .build();
    }
    assert_eq!(validate_output(&nested), Err(Error::OutputLimit));
}

#[test]
fn protocol_logging_stays_compile_disabled() {
    assert_eq!(log::STATIC_MAX_LEVEL, log::LevelFilter::Off);
}

#[test]
fn outgoing_encoding_accepts_exact_limit_and_rejects_one_more_byte() {
    use faction_xmpp::Element;
    let one = Element::builder("message", "jabber:client")
        .append("x")
        .build();
    let mut encoded = Vec::new();
    one.write_to(&mut encoded).unwrap();
    let overhead = encoded.len() - 1;
    let fitting = Element::builder("message", "jabber:client")
        .append("x".repeat(MAX_OUTPUT_BYTES - overhead))
        .build();
    let excessive = Element::builder("message", "jabber:client")
        .append("x".repeat(MAX_OUTPUT_BYTES + 1 - overhead))
        .build();
    assert_eq!(validate_output(&fitting), Ok(()));
    assert_eq!(validate_output(&excessive), Err(Error::OutputLimit));
}

#[test]
fn keylog_child() {
    if std::env::var_os("FACTION209_TRANSPORT_KEYLOG_TEST").is_none() {
        return;
    }
    let home = Home::new(
        "127.0.0.1:9".parse().unwrap(),
        "home.test",
        "alice@home.test/device",
        include_bytes!("fixtures/local-ca.pem"),
        [1; 32],
    )
    .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    assert!(matches!(
        runtime.block_on(faction_xmpp::connect(&home, "synthetic-unused")),
        Err(Error::KeylogRefused)
    ));
}

#[test]
fn inherited_key_logging_is_refused_before_connect() {
    for key in ["SSLKEYLOGFILE", "TLSKEYLOGFILE"] {
        for value in ["", "/private/tmp/faction209-transport-must-not-create"] {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "keylog_child"])
                .env_remove("SSLKEYLOGFILE")
                .env_remove("TLSKEYLOGFILE")
                .env("FACTION209_TRANSPORT_KEYLOG_TEST", "1")
                .env(key, value)
                .output()
                .unwrap();
            assert!(output.status.success());
            assert!(output.stderr.is_empty());
        }
    }
}

#[test]
fn read_errors_preserve_loss_without_remote_error_text() {
    use faction_xmpp::classify_read_error;
    use tokio_xmpp::xmlstream::ReadError;
    for kind in [
        std::io::ErrorKind::UnexpectedEof,
        std::io::ErrorKind::ConnectionReset,
        std::io::ErrorKind::ConnectionAborted,
        std::io::ErrorKind::BrokenPipe,
    ] {
        assert_eq!(
            classify_read_error(ReadError::HardError(std::io::Error::from(kind))),
            Error::Lost
        );
    }
    assert_eq!(
        classify_read_error(ReadError::HardError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            rxml::Error::InvalidEof(None)
        ))),
        Error::Lost
    );
    assert_eq!(
        classify_read_error(ReadError::HardError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            rxml::Error::ElementMismatch
        ))),
        Error::Protocol
    );
    assert_eq!(
        classify_read_error(ReadError::StreamFooterReceived),
        Error::Lost
    );
}

#[test]
fn installed_trace_logger_receives_no_records() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Recorder;
    static RECORDS: AtomicUsize = AtomicUsize::new(0);
    static LOGGER: Recorder = Recorder;
    impl log::Log for Recorder {
        fn enabled(&self, _: &log::Metadata<'_>) -> bool {
            true
        }
        fn log(&self, _: &log::Record<'_>) {
            RECORDS.fetch_add(1, Ordering::SeqCst);
        }
        fn flush(&self) {}
    }
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    log::error!("synthetic credential sentinel");
    log::trace!("synthetic XML sentinel");
    assert_eq!(RECORDS.load(Ordering::SeqCst), 0);
}
