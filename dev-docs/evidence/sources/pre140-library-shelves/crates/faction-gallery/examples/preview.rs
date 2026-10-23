//! Offline owner authoring preview. No member transport invokes this executable.
use faction_gallery::Catalog;
fn main() {
    let a: Vec<_> = std::env::args().skip(1).collect();
    if a.len() != 3 {
        eprintln!("Usage: preview PACK WORK ZERO_BASED_PAGE");
        std::process::exit(2);
    }
    let result = Catalog::load(std::path::Path::new(&a[0]))
        .and_then(|c| c.preview(&a[1], a[2].parse().map_err(|_| "invalid page")?));
    match result {
        Ok(s) => print!("\x1b[2J\x1b[H{s}"),
        Err(e) => {
            eprintln!("Gallery: {e}");
            std::process::exit(1);
        }
    }
}
