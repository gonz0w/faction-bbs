use faction_xmpp_probe::probe::{bounded, Config, Status};
use std::time::Duration;

fn config() -> String { format!("5223\n/private/tmp/fixture-ca.pem\n{}\nfixture-secret\nYWJj\n", "ab".repeat(32)) }
#[test]
fn bounded_private_config_requires_exact_pin_and_ciphertext() {
    assert!(Config::parse(&config()).is_ok());
    for bad in [config().replace(&"ab".repeat(32), ""), config().replace("YWJj", "<body>plain</body>"), config().replace("5223", "0"), format!("{}extra\n", config()), "x".repeat(8193)] { assert!(Config::parse(&bad).is_err()); }
}
#[test]
fn library_wire_logging_is_compile_disabled() { assert_eq!(log::STATIC_MAX_LEVEL, log::LevelFilter::Off); }
#[tokio::test(flavor = "current_thread")]
async fn absolute_lifetime_and_cancel_drop_pending_operation() {
    struct Dropped(std::rc::Rc<std::cell::Cell<bool>>);
    impl Drop for Dropped { fn drop(&mut self) { self.0.set(true); } }
    for cancel in [None, Some(Duration::from_millis(1))] {
        let dropped = std::rc::Rc::new(std::cell::Cell::new(false));
        let owned = Dropped(dropped.clone());
        let work = async move { let _owned = owned; std::future::pending::<Status>().await };
        let result = bounded(work, Duration::from_millis(20), cancel).await;
        assert_eq!(result, if cancel.is_some() { Status::Cancelled } else { Status::Timeout });
        assert!(dropped.get());
    }
}
