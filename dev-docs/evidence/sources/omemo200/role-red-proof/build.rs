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

fn main() {
    println!("cargo:rerun-if-env-changed=PICOMEMO_SOURCE");
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=AR");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-changed=native/proof.c");
    let source = PathBuf::from(
        env::var_os("PICOMEMO_SOURCE")
            .expect("PICOMEMO_SOURCE must name verified Picomemo 1.3.1 source"),
    );
    // Verify every upstream compilation input. Sources remain outside this repo.
    for (file, expected) in [
        (
            "gen/omemo2.c",
            "45e0ce3589cc779e47b73aee937b9324c23d5de68d4ca5184e846e3511593a9e",
        ),
        (
            "gen/omemo2.h",
            "893fa655fe5f19d50cfa4813eaa52d1c94edead3fe33944e93cc5e28ab099f03",
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
            "9eef3409554f1653493fa90d1a7c4ba80ce24701f4d452a31e50de7930757b98",
        ),
    ] {
        let path = source.join(file);
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
        let actual = checked(hash.arg(&path));
        assert_eq!(
            actual.split_whitespace().next(),
            Some(expected),
            "unpinned upstream input {file}"
        );
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
        ("proof", Path::new("native/proof.c").to_path_buf()),
    ] {
        let obj = out.join(format!("{name}.o"));
        let mut compiler = Command::new(env::var_os("CC").unwrap_or_else(|| "cc".into()));
        if env::var("CARGO_CFG_TARGET_OS").unwrap() == "netbsd" {
            compiler.arg("-D_NETBSD_SOURCE");
        }
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
