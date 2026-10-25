//! Executes the real dependency-free build script in an owned throwaway fixture.
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn disabled_has_no_native_tools_cross_rejects_and_source_tampering_fails() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = Fixture(std::env::temp_dir().join(format!(
            "faction203-build-owned-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    fs::create_dir(&fixture.0).unwrap();
    fs::write(
        fixture.0.join("OWNED-FACTION203"),
        b"disposable build-boundary fixture",
    )
    .unwrap();
    let program = fixture.0.join("build-script");
    assert!(
        Command::new("rustc")
            .arg(root.join("build.rs"))
            .args(["--edition=2024", "-o"])
            .arg(&program)
            .status()
            .unwrap()
            .success()
    );
    let off = Command::new(&program)
        .current_dir(&fixture.0)
        .env_clear()
        .env("HOST", "host")
        .env("TARGET", "other")
        .env("CC", "/no/compiler")
        .env("AR", "/no/archiver")
        .env("PATH", "/no/tools")
        .output()
        .unwrap();
    assert!(
        off.status.success(),
        "disabled build unexpectedly invoked native tools"
    );
    let stdout = String::from_utf8(off.stdout).unwrap();
    assert!(!stdout.contains("link-"));
    assert!(!stdout.contains("vendor"));
    let cross = Command::new(&program)
        .current_dir(&fixture.0)
        .env_clear()
        .env("CARGO_FEATURE_NATIVE", "1")
        .env("HOST", "host")
        .env("TARGET", "other")
        .env("PATH", "/no/tools")
        .output()
        .unwrap();
    assert!(!cross.status.success());
    assert!(String::from_utf8_lossy(&cross.stderr).contains("rejects cross compilation"));
    fs::create_dir_all(fixture.0.join("vendor/gen")).unwrap();
    fs::write(
        fixture.0.join("vendor/gen/omemo2.c"),
        b"unreviewed source substitution",
    )
    .unwrap();
    let tampered = Command::new(&program)
        .current_dir(&fixture.0)
        .env("CARGO_FEATURE_NATIVE", "1")
        .env("HOST", "host")
        .env("TARGET", "host")
        .env("CARGO_CFG_TARGET_OS", std::env::consts::OS)
        .env("CC", "/no/compiler")
        .output()
        .unwrap();
    assert!(!tampered.status.success());
    assert!(String::from_utf8_lossy(&tampered.stderr).contains("unpinned candidate input"));
    #[cfg(feature = "native")]
    {
        let headers = fixture.0.join("include/openssl");
        fs::create_dir_all(&headers).unwrap();
        fs::write(
            headers.join("opensslv.h"),
            b"#define OPENSSL_VERSION_MAJOR 2\n",
        )
        .unwrap();
        fs::write(
            headers.join("crypto.h"),
            b"unsigned int OPENSSL_version_major(void);\n",
        )
        .unwrap();
        let compile = Command::new(std::env::var_os("CC").unwrap_or_else(|| "cc".into()))
            .args(["-std=c11", "-DPROOF_AD_ORDER_V1", "-I"])
            .arg(fixture.0.join("include"))
            .arg("-I")
            .arg(root.join("vendor/gen"))
            .arg("-I")
            .arg(root.join("vendor"))
            .arg("-c")
            .arg(root.join("native/adapter.c"))
            .arg("-o")
            .arg(fixture.0.join("incompatible.o"))
            .output()
            .unwrap();
        assert!(!compile.status.success());
        assert!(
            String::from_utf8_lossy(&compile.stderr)
                .contains("requires native system OpenSSL major 3")
        );
    }
}
