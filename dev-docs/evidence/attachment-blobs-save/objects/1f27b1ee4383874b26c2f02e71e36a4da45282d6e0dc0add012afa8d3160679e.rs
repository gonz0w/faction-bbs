#![cfg(feature = "native")]
use std::{path::PathBuf, process::Command};

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
    let compiled = compiler.arg(root.join("tests/blob_guard.c")).arg("-lcrypto").arg("-o").arg(&binary).output().unwrap();
    assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
    for mode in ["positive", "core-refused", "runtime4", "capacity"] {
        let observed = Command::new(&binary).arg(mode).output().unwrap();
        assert!(observed.status.success(), "{mode}: {}", String::from_utf8_lossy(&observed.stderr));
    }
    std::fs::remove_file(binary).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
