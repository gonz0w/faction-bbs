// Private test-fixture custody only; never compiled into a production target.
#[test]
fn owned_fixture_retention_copies_exact_inputs_without_following_links() {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt, symlink};
    let source = Owned::new();
    let sink = Owned::new();
    let secret = b"synthetic fixture-only private input";
    fs::write(source.0.join("device.key"), secret).unwrap();
    fs::set_permissions(
        source.0.join("device.key"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    let nested = source.0.join("vault");
    fs::DirBuilder::new().mode(0o700).create(&nested).unwrap();
    fs::write(nested.join("checkpoint"), b"synthetic checkpoint input").unwrap();
    fs::set_permissions(nested.join("checkpoint"), fs::Permissions::from_mode(0o600)).unwrap();
    symlink(source.0.join("device.key"), source.0.join("key-link")).unwrap();
    let captured = capture_owned_test_fixture(&source.0, &sink.0);
    assert!(
        fs::read(captured.join("device.key")).unwrap() == secret,
        "retained private fixture bytes differ"
    );
    assert!(fs::read(captured.join("vault/checkpoint")).unwrap() == b"synthetic checkpoint input");
    assert!(
        !captured.join("key-link").exists(),
        "retention must not follow or recreate a live symlink"
    );
    assert!(captured.join("key-link.symlink-target").is_file());
    assert_eq!(
        fs::metadata(&captured).unwrap().permissions().mode() & 0o7777,
        0o700
    );
    assert_eq!(
        fs::metadata(captured.join("device.key"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o600
    );
    assert!(!source.0.join("capture").exists());
}
