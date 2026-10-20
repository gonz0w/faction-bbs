//! Filesystem-owner console. Never install as an SSH forced command.
#![forbid(unsafe_code)]
use faction_cli::{Result, invalid, line, menu, new_secret, secret};
use faction_core::Board;
use faction_terminal::Theme;
use std::path::PathBuf;
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() || args == ["--help"] {
        println!(
            "Faction local development console\nUsage: faction --state-dir PATH init|register|login|recover\nOwner data tools: faction --state-dir PATH export NEW_FILE | restore EXPORT_FILE\n                 faction verify-export EXPORT_FILE\nLogin options: --ansi | --plain, --theme PACK_DIRECTORY, --season OVERRIDE_DIRECTORY\nPrivate local state only. Never use as an SSH forced command.\nNo password arguments, environment credentials, listener or network service."
        );
        return Ok(());
    }
    if args.first().is_some_and(|v| v == "verify-export") {
        if args.len() != 2 {
            return Err(invalid("verify-export needs one private file"));
        }
        let summary = faction_core::verify_export(&PathBuf::from(&args[1]))?;
        println!(
            "Export verified: {} members, {} rooms, {} posts, {} channels, {} retained chat messages.",
            summary.members, summary.rooms, summary.posts, summary.channels, summary.chat_messages
        );
        return Ok(());
    }
    if args.len() < 3 || args[0] != "--state-dir" {
        return Err(invalid("use --help for syntax"));
    }
    let path = PathBuf::from(&args[1]);
    let command = args[2].to_str().ok_or_else(|| invalid("invalid command"))?;
    if ["export", "restore"].contains(&command) {
        if args.len() != 4 {
            return Err(invalid("export/restore needs exactly one export file path"));
        }
        let file = PathBuf::from(&args[3]);
        let summary = if command == "restore" {
            faction_core::restore_local(&file, &path)?
        } else {
            faction_core::export_local(&path, &file)?
        };
        println!(
            "{} complete: {} members, {} rooms, {} posts, {} channels, {} retained chat messages. Private plaintext data; no live sessions or external theme files.",
            if command == "export" {
                "Export"
            } else {
                "Restore"
            },
            summary.members,
            summary.rooms,
            summary.posts,
            summary.channels,
            summary.chat_messages
        );
        return Ok(());
    }
    if !["init", "register", "login", "recover"].contains(&command) {
        return Err(invalid("unknown command"));
    }
    let (mut ansi, mut base_path, mut season_path) = (false, None, None);
    if command != "login" && args.len() > 3 {
        return Err(invalid("theme options apply only to login"));
    }
    let mut index = 3;
    while index < args.len() {
        match args[index].to_str() {
            Some("--ansi") => ansi = true,
            Some("--plain") => ansi = false,
            Some("--theme") | Some("--season") => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| invalid("theme option needs a directory"))?;
                if args[index] == "--theme" {
                    base_path = Some(PathBuf::from(value));
                } else {
                    season_path = Some(PathBuf::from(value));
                }
                index += 1;
            }
            _ => return Err(invalid("unknown login option")),
        }
        index += 1;
    }
    let mut theme = Theme::default();
    if let Some(directory) = base_path {
        match Theme::load(&directory) {
            Ok(t) => theme = t,
            Err(reason) => eprintln!("Theme rejected: {reason}; using built-in text fallback."),
        }
    }
    if let Some(directory) = season_path {
        match theme.with_season(&directory) {
            Ok(t) => theme = t,
            Err(reason) => eprintln!("Season rejected: {reason}; using the unchanged base theme."),
        }
    }
    let mut board = Board::open(&path)?;
    let name = line("Handle: ", 24)?;
    match command {
        "init" => {
            let value = new_secret()?;
            board.bootstrap(&name, &value)?;
            println!("Sysop initialized. Lobby created.");
        }
        "register" => {
            let value = new_secret()?;
            board.register(&name, &value)?;
            println!("Registered pending sysop approval.");
        }
        "login" => {
            let value = secret("Password (hidden): ")?;
            let s = board.authenticate(&name, &value)?;
            drop(value);
            menu(&mut board, &s, &theme, ansi)?;
        }
        "recover" => {
            if line("Local filesystem-owner recovery. Type RECOVER: ", 8)? != "RECOVER" {
                return Err(invalid("recovery cancelled"));
            }
            let value = new_secret()?;
            board.recover_local_sysop(&name, &value)?;
            println!("Sysop password reset; prior sessions expired.");
        }
        _ => unreachable!(),
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Faction: {e}");
        std::process::exit(1)
    }
}
