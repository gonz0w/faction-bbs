//! Executes the real dependency-free build script in an owned throwaway fixture.
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn executable(path: &Path, source: &str) {
    fs::write(path, source).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

// This executes the real build script and real source hashing while capturing
// compiler/archive arguments. It is not a foreign-platform compile or link test.
fn compiler_feature_contract(root: &Path, fixture: &Path, program: &Path) {
    for file in [
        "gen/omemo2.c",
        "gen/omemo2.h",
        "hacl.c",
        "openssl.c",
        "driver.h",
        "omemo.h",
    ] {
        fs::copy(root.join("vendor").join(file), fixture.join("vendor").join(file)).unwrap();
    }
    fs::create_dir(fixture.join("native")).unwrap();
    fs::copy(root.join("native/adapter.c"), fixture.join("native/adapter.c")).unwrap();
    let tools = fixture.join("capture-tools");
    fs::create_dir(&tools).unwrap();
    let compiler = tools.join("capture-cc");
    executable(
        &compiler,
        "#!/bin/sh\nset -eu\nprintf 'BEGIN\\n' >> \"$FACTION203_CC_LOG\"\nprintf '%s\\n' \"$@\" >> \"$FACTION203_CC_LOG\"\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = '-o' ]; then\n    shift\n    : > \"$1\"\n    exit 0\n  fi\n  shift\ndone\nexit 1\n",
    );
    let archiver = tools.join("capture-ar");
    executable(
        &archiver,
        "#!/bin/sh\nset -eu\nprintf '%s\\n' \"$@\" > \"$FACTION203_AR_LOG\"\n",
    );
    // Simulated target OS names choose different hash command spellings. Both
    // aliases delegate to the actual host hashing utility on the copied bytes.
    let host_path = std::env::var_os("PATH").unwrap();
    let hash_name = if std::env::consts::OS == "netbsd" { "sha256" } else { "shasum" };
    let hash = std::env::split_paths(&host_path)
        .map(|path| path.join(hash_name))
        .find(|path| path.is_file())
        .unwrap()
        .canonicalize()
        .unwrap();
    let hash_flag = if hash_name == "sha256" { "-q" } else { "-a 256" };
    // Pass the executable via an environment argument, never shell interpolation.
    let hash_alias = format!("#!/bin/sh\nset -eu\nfor source do :; done\nexec \"$FACTION203_HOST_HASH\" {hash_flag} \"$source\"\n");
    executable(&tools.join("sha256"), &hash_alias);
    executable(&tools.join("shasum"), &hash_alias);
    let path = std::env::join_paths(std::iter::once(tools.clone()).chain(std::env::split_paths(&host_path))).unwrap();
    for (target_os, expected_macro) in [
        ("linux", Some("-D_DEFAULT_SOURCE")),
        ("netbsd", Some("-D_NETBSD_SOURCE")),
        ("macos", None),
    ] {
        let out = fixture.join(format!("out-{target_os}"));
        fs::create_dir(&out).unwrap();
        let cc_log = out.join("compiler-args");
        let ar_log = out.join("archiver-args");
        let result = Command::new(program)
            .current_dir(fixture)
            .env_clear()
            .env("PATH", &path)
            .env("CARGO_FEATURE_NATIVE", "1")
            .env("HOST", "same-native-host")
            .env("TARGET", "same-native-host")
            .env("CARGO_CFG_TARGET_OS", target_os)
            .env("CC", &compiler)
            .env("AR", &archiver)
            .env("OUT_DIR", &out)
            .env("OPENSSL_INCLUDE_DIR", fixture.join("dummy-openssl-include"))
            .env("OPENSSL_LIB_DIR", fixture.join("dummy-openssl-lib"))
            .env("FACTION203_CC_LOG", &cc_log)
            .env("FACTION203_AR_LOG", &ar_log)
            .env("FACTION203_HOST_HASH", &hash)
            .output()
            .unwrap();
        assert!(result.status.success(), "captured {target_os} build command failed: {}", String::from_utf8_lossy(&result.stderr));
        let log = fs::read_to_string(cc_log).unwrap();
        let calls: Vec<_> = log.split("BEGIN\n").skip(1).collect();
        assert_eq!(calls.len(), 4, "{target_os} must compile all four native units");
        for call in calls {
            let args: Vec<_> = call.lines().collect();
            let definitions: Vec<_> = args.iter().copied().filter(|arg| arg.starts_with("-D")).collect();
            let expected: Vec<_> = expected_macro.into_iter().chain(["-DPROOF_AD_ORDER_V1"]).collect();
            assert_eq!(definitions, expected, "wrong compiler feature macros for {target_os}");
            assert!(args.contains(&"-std=c11"));
            assert!(args.contains(&"-c"));
        }
        let archive_args = fs::read_to_string(ar_log).unwrap();
        let archive_args: Vec<_> = archive_args.lines().collect();
        assert_eq!(archive_args.len(), 6);
        assert_eq!(archive_args[0], "rcs");
        assert_eq!(archive_args[1], out.join("libproof.a").to_str().unwrap());
        for name in ["omemo2", "hacl", "openssl", "proof"] {
            assert!(out.join(format!("{name}.o")).is_file());
        }
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
    compiler_feature_contract(&root, &fixture.0, &program);
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
