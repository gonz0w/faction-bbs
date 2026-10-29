//! Explicit private operator executable. Selecting another build starts nothing.
mod operator;
fn main() {
    if operator::run(std::env::args_os().skip(1)).is_err() {
        eprintln!("private provider command refused");
        std::process::exit(1);
    }
}
