fn main() {
    println!("cargo:rerun-if-changed=src/terminal_fd.c");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("netbsd") {
        cc::Build::new()
            .file("src/terminal_fd.c")
            .warnings(true)
            .extra_warnings(true)
            .warnings_into_errors(true)
            .compile("faction_terminal_fd");
    }
}
