use faction_xmpp_probe::probe::{bounded, Config, Status};
use std::time::Duration;

fn config() -> String {
    format!(
        "5223\n/private/tmp/fixture-ca.pem\n{}\nfixture-secret\nYWJj\n",
        "ab".repeat(32)
    )
}
#[test]
fn bounded_private_config_requires_exact_pin_and_ciphertext() {
    assert!(Config::parse(&config()).is_ok());
    for bad in [
        config().replace(&"ab".repeat(32), ""),
        config().replace("YWJj", "<body>plain</body>"),
        config().replace("5223", "0"),
        format!("{}extra\n", config()),
        "x".repeat(8193),
    ] {
        assert!(Config::parse(&bad).is_err());
    }
}
#[test]
fn library_wire_logging_is_compile_disabled() {
    assert_eq!(log::STATIC_MAX_LEVEL, log::LevelFilter::Off);
}
#[tokio::test(flavor = "current_thread")]
async fn absolute_lifetime_and_cancel_drop_pending_operation() {
    struct Dropped(std::rc::Rc<std::cell::Cell<bool>>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }
    for cancel in [None, Some(Duration::from_millis(1))] {
        let dropped = std::rc::Rc::new(std::cell::Cell::new(false));
        let owned = Dropped(dropped.clone());
        let work = async move {
            let _owned = owned;
            std::future::pending::<Status>().await
        };
        let result = bounded(work, Duration::from_millis(20), cancel).await;
        assert_eq!(
            result,
            if cancel.is_some() {
                Status::Cancelled
            } else {
                Status::Timeout
            }
        );
        assert!(dropped.get());
    }
}

#[test]
fn installed_logger_never_receives_wire_or_credential_records() {
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
    log::trace!("synthetic wire sentinel");
    assert_eq!(RECORDS.load(Ordering::SeqCst), 0);
}

#[test]
fn inherited_tls_key_logging_is_refused_before_attempt_or_stdin() {
    for key in ["SSLKEYLOGFILE", "TLSKEYLOGFILE"] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_faction-xmpp-probe"))
            .arg("probe")
            .env_remove("SSLKEYLOGFILE")
            .env_remove("TLSKEYLOGFILE")
            .env(key, "/private/tmp/faction208-must-not-be-created")
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            output.stdout,
            b"{\"status\":\"keylog_refused\",\"attempts\":0,\"received_bytes\":0,\"events\":0}\n"
        );
        assert!(output.stderr.is_empty());
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_argument_is_sanitized_config_rejection() {
    use std::os::unix::ffi::OsStringExt;
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_faction-xmpp-probe"))
        .arg(std::ffi::OsString::from_vec(vec![0xff]))
        .env_remove("SSLKEYLOGFILE")
        .env_remove("TLSKEYLOGFILE")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        output.stdout,
        b"{\"status\":\"config\",\"attempts\":0,\"received_bytes\":0,\"events\":0}\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn connection_loss_uses_typed_errors_without_swallowing_protocol_errors() {
    use faction_xmpp_probe::probe::classify_read_error;
    use tokio_xmpp::xmlstream::ReadError;
    for kind in [
        std::io::ErrorKind::UnexpectedEof,
        std::io::ErrorKind::ConnectionReset,
        std::io::ErrorKind::ConnectionAborted,
        std::io::ErrorKind::BrokenPipe,
    ] {
        assert_eq!(
            classify_read_error(ReadError::HardError(std::io::Error::from(kind))),
            Status::Lost
        );
    }
    assert_eq!(
        classify_read_error(ReadError::HardError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            rxml::Error::InvalidEof(None)
        ))),
        Status::Lost
    );
    assert_eq!(
        classify_read_error(ReadError::HardError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            rxml::Error::ElementMismatch
        ))),
        Status::Protocol
    );
    assert_eq!(
        classify_read_error(ReadError::StreamFooterReceived),
        Status::Lost
    );
}
