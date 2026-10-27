#![cfg(feature = "native")]
use std::{path::PathBuf, process::Command};

#[test]
#[ignore = "isolated blob-only subprocess, invoked by blob_only_public_entry_establishes_core_guard"]
fn blob_only_guard_child() {
    unsafe extern "C" {
        fn proof_core_dumps_disabled() -> i32;
    }
    let before = unsafe { proof_core_dumps_disabled() };
    let encrypted = faction_omemo::blob::encrypt(b"public synthetic bytes").unwrap();
    assert_eq!(encrypted.size, 22);
    assert_eq!(unsafe { proof_core_dumps_disabled() }, 1);
    println!("blob-only process without Device creation: core_disabled_before={before}, after=1");
}

#[test]
fn blob_only_public_entry_establishes_core_guard() {
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "blob_only_guard_child",
            "--nocapture",
            "--test-threads=1",
        ])
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    assert!(String::from_utf8_lossy(&child.stdout).contains("after=1"));
    print!("{}", String::from_utf8_lossy(&child.stdout));
}

#[test]
fn native_guards_and_capacity_refusals_precede_context_allocation() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let directory = std::env::temp_dir().join(format!("faction254-guard-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let binary = directory.join("guard");
    let mut compiler = Command::new(std::env::var_os("CC").unwrap_or_else(|| "cc".into()));
    compiler.args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-O2"]);
    if let Some(include) = std::env::var_os("OPENSSL_INCLUDE_DIR") {
        compiler.arg("-I").arg(include);
    }
    if let Some(lib) = std::env::var_os("OPENSSL_LIB_DIR") {
        compiler.arg("-L").arg(lib);
    }
    let compiled = compiler
        .arg(root.join("tests/blob_guard.c"))
        .arg("-lcrypto")
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    for mode in ["positive", "core-refused", "runtime4", "capacity"] {
        let observed = Command::new(&binary).arg(mode).output().unwrap();
        assert!(
            observed.status.success(),
            "{mode}: {}",
            String::from_utf8_lossy(&observed.stderr)
        );
    }
    std::fs::remove_file(binary).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
