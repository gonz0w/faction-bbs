// Private synthetic fixture copying only; no production target includes this file.
fn capture_owned_test_fixture(
    source: &std::path::Path,
    sink: &std::path::Path,
) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    assert!(
        !sink.starts_with(source),
        "private fixture retention must be outside the live fixture"
    );
    assert_eq!(
        std::fs::metadata(sink).unwrap().permissions().mode() & 0o7777,
        0o700
    );
    let target = sink.join(source.file_name().unwrap());
    fn copy(source: &std::path::Path, target: &std::path::Path) {
        use std::os::unix::{
            ffi::OsStrExt,
            fs::{DirBuilderExt, PermissionsExt},
        };
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(target)
            .unwrap();
        for item in std::fs::read_dir(source).unwrap() {
            let item = item.unwrap();
            let kind = item.file_type().unwrap();
            let destination = target.join(item.file_name());
            if kind.is_dir() {
                copy(&item.path(), &destination);
            } else if kind.is_file() {
                std::fs::copy(item.path(), &destination).unwrap();
                std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(0o600))
                    .unwrap();
            } else if kind.is_symlink() {
                let mut name = item.file_name();
                name.push(".symlink-target");
                let destination = target.join(name);
                std::fs::write(
                    &destination,
                    std::fs::read_link(item.path())
                        .unwrap()
                        .as_os_str()
                        .as_bytes(),
                )
                .unwrap();
                std::fs::set_permissions(destination, std::fs::Permissions::from_mode(0o600))
                    .unwrap();
            } else {
                panic!("unsupported synthetic fixture input type");
            }
        }
    }
    copy(source, &target);
    target
}
