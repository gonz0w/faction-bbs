//! Broker-launched member worker. No bootstrap, recovery, export or host commands.
#![forbid(unsafe_code)]
use faction_cli::{Result, invalid, line, new_secret, secret};
use faction_core::Board;
use faction_terminal::Theme;
use std::{
    io::{self, IsTerminal},
    path::PathBuf,
};
fn watch_broker() -> Result<()> {
    let Some(expected) = std::env::var_os("FACTION_BROKER_PID") else {
        return Ok(()); // direct local PTY development worker
    };
    let expected: u32 = expected
        .to_str()
        .ok_or_else(|| invalid("broker PID"))?
        .parse()?;
    if expected <= 1 || rustix::process::getppid().map(|p| p.as_raw_pid() as u32) != Some(expected)
    {
        return Err(invalid("broker unavailable"));
    }
    std::thread::Builder::new()
        .name("broker-watch".into())
        .stack_size(64 * 1024)
        .spawn(move || {
            loop {
                if rustix::process::getppid().map(|p| p.as_raw_pid() as u32) != Some(expected) {
                    // Also ends a worker blocked on terminal output or database I/O.
                    // Abrupt exit intentionally relies on existing chat lease expiry.
                    std::process::exit(1);
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        })?;
    Ok(())
}
fn run() -> Result<()> {
    watch_broker()?;
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
    let (mut base, mut season, mut schedule) = (None, None, None);
    let mut index = 2;
    while index < args.len() {
        let value = args
            .get(index + 1)
            .ok_or_else(|| invalid("theme path required"))?;
        let target = match args[index].to_str() {
            Some("--theme") => &mut base,
            Some("--season") => &mut season,
            Some("--theme-schedule") => &mut schedule,
            _ => return Err(invalid("unknown member worker option")),
        };
        if target.is_some() {
            return Err(invalid("duplicate member theme option"));
        }
        *target = Some(PathBuf::from(value));
        index += 2;
    }
    let theme = base
        .map(|p| Theme::load(&p))
        .transpose()
        .map_err(invalid)?
        .unwrap_or_default();
    let themes =
        faction_terminal::seasons::Themes::load(theme, season.as_deref(), schedule.as_deref())
            .map_err(invalid)?;
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
            faction_cli::scheduled_menu(&mut board, &s, &themes, true)?;
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
