//! Broker-launched member worker. No bootstrap, recovery, export or host commands.
#![forbid(unsafe_code)]
use faction_cli::{Result, invalid, line, menu, new_secret, secret};
use faction_core::Board;
use faction_terminal::Theme;
use std::{
    io::{self, IsTerminal},
    path::PathBuf,
};
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() < 2
        || args[0] != "--state-dir"
        || !io::stdin().is_terminal()
        || !io::stdout().is_terminal()
    {
        return Err(invalid(
            "member worker requires a broker terminal and fixed state directory",
        ));
    }
    let path = PathBuf::from(&args[1]);
    if !path.join("faction.sqlite3").is_file() {
        return Err(invalid("initialized board required"));
    }
    let mut theme = Theme::default();
    let mut index = 2;
    while index < args.len() {
        let value = args
            .get(index + 1)
            .ok_or_else(|| invalid("theme path required"))?;
        theme = match args[index].to_str() {
            Some("--theme") => Theme::load(&PathBuf::from(value)).map_err(invalid)?,
            Some("--season") => theme.with_season(&PathBuf::from(value)).map_err(invalid)?,
            _ => return Err(invalid("unknown member worker option")),
        };
        index += 2;
    }
    let mut board = Board::open(&path)?;
    println!("Faction BBS access. SSH admission does not grant BBS membership.");
    match line("[L]ogin [R]egister [Q]uit: ", 1)?
        .to_ascii_lowercase()
        .as_str()
    {
        "l" => {
            let name = line("Handle: ", 24)?;
            let password = secret("Password (hidden): ")?;
            let s = board.authenticate(&name, &password)?;
            drop(password);
            menu(&mut board, &s, &theme, true)?;
        }
        "r" => {
            let name = line("Handle: ", 24)?;
            let password = new_secret()?;
            board.register(&name, &password)?;
            println!("Registered pending sysop approval. Reconnect after approval.");
        }
        "q" => (),
        _ => return Err(invalid("unknown access choice")),
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Faction: {e}");
        std::process::exit(1)
    }
}
