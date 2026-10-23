//! Offline stdin validator; never opens a network connection or a database.
use std::io::{self, Read};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    io::stdin()
        .take((faction_articles::WIRE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    let article = faction_articles::Article::parse(&bytes)?;
    println!(
        "Valid Faction text profile: {} body bytes, {} groups, {} references.",
        article.body().len(),
        article.headers().newsgroups.len(),
        article.headers().references.len()
    );
    Ok(())
}
