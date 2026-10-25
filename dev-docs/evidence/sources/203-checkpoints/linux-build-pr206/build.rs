use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn checked(command: &mut Command) -> String {
    let output = command.output().expect("run native build tool");
    assert!(
        output.status.success(),
        "native build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("UTF-8 tool output")
}

const CORRECTED: &[(&str, &str)] = &[
    (
        "gen/omemo2.c",
        "ef2be5e5d4676508cb2ab4144bb71f182467e3644864723965403850799530ab",
    ),
    (
        "gen/omemo2.h",
        "1eed053acaa4d73a5d117d358a370331b5c68b70ae30f74d1b1884fb2522dfc3",
    ),
    (
        "hacl.c",
        "c204f117deb68ab30a204c052227384734d0f0424330487dde45f48792dca8e9",
    ),
    (
        "openssl.c",
        "4e4877dca3dd2ebff93fc6a8adbff117b67a1c28d665eea2721831c20c0583ce",
    ),
    (
        "driver.h",
        "4847d58fd3388e73bbe911417753ff153e6fa9f5a99af55f549f2f8df57bc792",
    ),
    (
        "omemo.h",
        "6871e89e0c5f9f0e2e76f4c8b7bc48631b849ace3157704b5d6ad4cb6d6485e2",
    ),
];

fn verify(path: &Path, expected: &str) {
    println!("cargo:rerun-if-changed={}", path.display());
    let mut hash = if env::var("CARGO_CFG_TARGET_OS").unwrap() == "netbsd" {
        let mut cmd = Command::new("sha256");
        cmd.arg("-q");
        cmd
    } else {
        let mut cmd = Command::new("shasum");
        cmd.args(["-a", "256"]);
        cmd
    };
    let actual = checked(hash.arg(path));
    assert_eq!(
        actual.split_whitespace().next(),
        Some(expected),
        "unpinned candidate input {}",
        path.display()
    );
}

fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_NATIVE");
    if env::var_os("CARGO_FEATURE_NATIVE").is_none() {
        return;
    }
    assert_eq!(
        env::var_os("HOST"),
        env::var_os("TARGET"),
        "faction-omemo native feature rejects cross compilation"
    );
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=AR");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-changed=native/adapter.c");
    let source = PathBuf::from("vendor");
    for (file, expected) in CORRECTED {
        verify(&source.join(file), expected);
    }
    println!("cargo:rerun-if-env-changed=OPENSSL_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=OPENSSL_LIB_DIR");
    let explicit_include = env::var_os("OPENSSL_INCLUDE_DIR");
    let explicit_lib = env::var_os("OPENSSL_LIB_DIR");
    assert_eq!(
        explicit_include.is_some(),
        explicit_lib.is_some(),
        "set both explicit OpenSSL directories"
    );
    let (cflags, libs) = if explicit_include.is_some() {
        (String::new(), String::new())
    } else {
        (
            checked(Command::new("pkg-config").args(["--cflags", "libcrypto"])),
            checked(Command::new("pkg-config").args(["--libs", "libcrypto"])),
        )
    };
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut objects = Vec::new();
    for (name, path) in [
        ("omemo2", source.join("gen/omemo2.c")),
        ("hacl", source.join("hacl.c")),
        ("openssl", source.join("openssl.c")),
        ("proof", Path::new("native/adapter.c").to_path_buf()),
    ] {
        let obj = out.join(format!("{name}.o"));
        let mut compiler = Command::new(env::var_os("CC").unwrap_or_else(|| "cc".into()));
        if env::var("CARGO_CFG_TARGET_OS").unwrap() == "netbsd" {
            compiler.arg("-D_NETBSD_SOURCE");
        }
        compiler.arg("-DPROOF_AD_ORDER_V1");
        if let Some(ref include) = explicit_include {
            compiler.arg("-I").arg(include);
        }
        checked(
            compiler
                .args([
                    "-std=c11",
                    "-O2",
                    "-fPIC",
                    "-Wall",
                    "-Wextra",
                    "-Wno-unused-function",
                    "-Wno-pointer-sign",
                ])
                .arg("-I")
                .arg(source.join("gen"))
                .arg("-I")
                .arg(&source)
                .args(cflags.split_whitespace())
                .arg("-c")
                .arg(path)
                .arg("-o")
                .arg(&obj),
        );
        objects.push(obj);
    }
    checked(
        Command::new(env::var_os("AR").unwrap_or_else(|| "ar".into()))
            .arg("rcs")
            .arg(out.join("libproof.a"))
            .args(objects),
    );
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=proof");
    if let Some(lib) = explicit_lib {
        println!(
            "cargo:rustc-link-search=native={}",
            PathBuf::from(lib).display()
        );
        println!("cargo:rustc-link-lib=crypto");
    }
    for flag in libs.split_whitespace() {
        if let Some(path) = flag.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={path}");
        } else if let Some(lib) = flag.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={lib}");
        } else {
            panic!("unsupported pkg-config linker flag: {flag}");
        }
    }
}
