//! Disposable profiling fixture only. Never installed as a member/owner command.
#![forbid(unsafe_code)]
use faction_core::{Board, Status};
use rusqlite::{Connection, params};
use std::{
    io::{self, Read},
    path::PathBuf,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = std::env::args_os().skip(1).collect();
    if a.len() != 2 || a[0] != "--disposable-state" {
        return Err("disposable-state path required".into());
    }
    let path = PathBuf::from(&a[1]);
    if !path.join(".profile-fixture").is_file() || path.join("faction.sqlite3").exists() {
        return Err("fresh marked disposable fixture required".into());
    }
    let mut secret = String::new();
    io::stdin().take(129).read_to_string(&mut secret)?;
    if secret.len() > 128 {
        return Err("secret limit".into());
    }
    let mut b = Board::open(&path)?;
    b.bootstrap("profile_sysop", &secret)?;
    let root = b.authenticate("profile_sysop", &secret)?;
    // Every account gets a separately salted real Argon2 record via the core.
    for n in 0..1000 {
        let name = format!("profile{n:04}");
        b.register(&name, &secret)?;
        b.set_status(&root, &name, Status::Approved)?;
    }
    drop(b);
    // Bulk authored content is fixture construction, not an online member API.
    let mut db = Connection::open(path.join("faction.sqlite3"))?;
    db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")?;
    let tx = db.transaction()?;
    for n in 2..=50 {
        tx.execute(
            "INSERT INTO rooms(id,slug,title) VALUES(?1,?2,?3)",
            params![n, format!("profile{n}"), format!("Profile room {n}")],
        )?;
    }
    let body = "p".repeat(1024);
    {
        let mut insert =
            tx.prepare("INSERT INTO posts(room_id,author_id,request_id,body) VALUES(?1,?2,?3,?4)")?;
        for n in 0..100_000 {
            insert.execute(params![
                1 + n % 50,
                2 + n % 1000,
                format!("profile-post-{n:016}"),
                body
            ])?;
        }
    }
    tx.commit()?;
    db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    println!("Seeded 1000 independently salted approved accounts, 50 rooms, 100000 1-KiB posts.");
    Ok(())
}
