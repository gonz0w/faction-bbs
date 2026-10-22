//! Explicit local OS-operator command. Never an SSH forced command or setuid.
#![forbid(unsafe_code)]
fn main() {
    if let Err(reason) = faction_cli::operator::run() {
        eprintln!("Operator console: {reason}");
        std::process::exit(1);
    }
}
