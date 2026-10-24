fn main() {
    println!("cargo:rerun-if-changed=src/pty.c");
    cc::Build::new()
        .file("src/pty.c")
        .std("c11")
        .warnings(true)
        .extra_warnings(true)
        .warnings_into_errors(true)
        .compile("faction_client_pty");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        println!("cargo:rustc-link-lib=util");
    }
}
