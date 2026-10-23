//! Bounded offline canonicalizer; no database, listener, path or URL input.
use std::io::{self, Read};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut wire = Vec::new();
    io::stdin()
        .take((faction_articles::WIRE_BYTES + 1) as u64)
        .read_to_end(&mut wire)?;
    let article = faction_articles::Article::parse(&wire)?;
    print!("{}", article.to_wire()?);
    Ok(())
}
