//! Filesystem-owner console. Never install as an SSH forced command.
#![forbid(unsafe_code)]
use faction_cli::{Result, invalid, line, new_secret, secret};
use faction_core::Board;
use faction_terminal::Theme;
use std::path::PathBuf;
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() || args == ["--help"] {
        println!(
            "Faction local development console\nUsage: faction --state-dir PATH init|register|login|recover\nOwner magazine export: faction --state-dir PATH magazine-edition ISSUE_ID NEW_FILE\nOwner data tools: faction --state-dir PATH export NEW_FILE | restore EXPORT_FILE\n                 faction verify-export EXPORT_FILE\nLogin options: --ansi | --plain, --theme PACK_DIRECTORY, --season OVERRIDE_DIRECTORY, --theme-schedule SCHEDULE_DIRECTORY, --directory PACK, --gallery PACK, --library PACK\nOffline library check: faction library-check PACK\nOwner magazine tools: faction library-preview PACK ISSUE | library-edition PACK ISSUE NEW_FILE\nOffline gallery check: faction gallery-check PACK\nOffline directory check: faction directory-check PACK\nPrivate local state only. Never use as an SSH forced command.\nNo password arguments, environment credentials, listener or network service."
        );
        return Ok(());
    }
    if args
        .first()
        .is_some_and(|v| v == "library-preview" || v == "library-edition")
    {
        let preview = args[0] == "library-preview";
        if args.len() != if preview { 3 } else { 4 } {
            return Err(invalid(
                "library-preview PACK ISSUE or library-edition PACK ISSUE NEW_FILE",
            ));
        }
        let c = faction_library::Catalog::load(&PathBuf::from(&args[1])).map_err(invalid)?;
        let text = c
            .magazine_text(
                args[2]
                    .to_str()
                    .ok_or_else(|| invalid("issue ID needs ASCII"))?,
                preview,
            )
            .map_err(invalid)?;
        if preview {
            print!("{text}");
        } else {
            use std::{io::Write, os::unix::fs::OpenOptionsExt};
            let path = PathBuf::from(&args[3]);
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)?;
            if let Err(e) = f.write_all(text.as_bytes()).and_then(|_| f.sync_all()) {
                let _ = std::fs::remove_file(&path);
                return Err(e.into());
            }
            println!(
                "Wrote local text edition; publish permitted bytes through existing Files review."
            );
        }
        return Ok(());
    }
    if args.first().is_some_and(|v| v == "library-check") {
        if args.len() != 2 {
            return Err(invalid("library-check needs one pack directory"));
        }
        let c = faction_library::Catalog::load(&PathBuf::from(&args[1])).map_err(invalid)?;
        println!(
            "Library verified: {} curated entries; enabled {}. File permissions are checked per member at use.",
            c.len(),
            c.enabled()
        );
        print!("{}", c.magazine_review());
        return Ok(());
    }
    if args.first().is_some_and(|v| v == "gallery-check") {
        if args.len() != 2 {
            return Err(invalid("gallery-check needs one pack directory"));
        }
        let c = faction_gallery::Catalog::load(&PathBuf::from(&args[1])).map_err(invalid)?;
        println!(
            "Gallery verified: {} curated works; enabled {}. Rights remain the curator's responsibility.",
            c.len(),
            c.enabled()
        );
        return Ok(());
    }
    if args.first().is_some_and(|v| v == "directory-check") {
        if args.len() != 2 {
            return Err(invalid("directory-check needs one pack directory"));
        }
        let d = faction_terminal::directory::Directory::load(&PathBuf::from(&args[1]))
            .map_err(invalid)?;
        println!(
            "Directory verified: {} curated listings. No network connection made.",
            d.len()
        );
        return Ok(());
    }
    if args.first().is_some_and(|v| v == "verify-export") {
        if args.len() != 2 {
            return Err(invalid("verify-export needs one private file"));
        }
        let summary = faction_core::verify_export(&PathBuf::from(&args[1]))?;
        println!(
            "Export verified: {} members, {} rooms, {} posts, {} channels, {} retained chat messages, {} game score entries.",
            summary.members,
            summary.rooms,
            summary.posts,
            summary.channels,
            summary.chat_messages,
            summary.game_scores
        );
        return Ok(());
    }
    if args.len() < 3 || args[0] != "--state-dir" {
        return Err(invalid("use --help for syntax"));
    }
    let path = PathBuf::from(&args[1]);
    let command = args[2].to_str().ok_or_else(|| invalid("invalid command"))?;
    if command == "magazine-edition" {
        if args.len() != 5 {
            return Err(invalid(
                "magazine-edition needs issue ID and new private text file",
            ));
        }
        let id = args[3]
            .to_str()
            .and_then(|s| s.parse::<i64>().ok())
            .filter(|id| *id > 0)
            .ok_or_else(|| invalid("positive issue ID required"))?;
        faction_core::export_magazine_local(&path, id, &PathBuf::from(&args[4]))?;
        println!(
            "Wrote currently approved local edition; existing file review/publication still applies."
        );
        return Ok(());
    }
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
            "{} complete: {} members, {} rooms, {} posts, {} channels, {} retained chat messages, {} game score entries. Private plaintext data; no live sessions or external theme files.",
            if command == "export" {
                "Export"
            } else {
                "Restore"
            },
            summary.members,
            summary.rooms,
            summary.posts,
            summary.channels,
            summary.chat_messages,
            summary.game_scores
        );
        return Ok(());
    }
    if !["init", "register", "login", "recover"].contains(&command) {
        return Err(invalid("unknown command"));
    }
    let (
        mut ansi,
        mut base_path,
        mut season_path,
        mut schedule_path,
        mut directory_path,
        mut gallery_path,
        mut library_path,
    ) = (false, None, None, None, None, None, None);
    if command != "login" && args.len() > 3 {
        return Err(invalid("theme options apply only to login"));
    }
    let mut index = 3;
    while index < args.len() {
        match args[index].to_str() {
            Some("--ansi") => ansi = true,
            Some("--plain") => ansi = false,
            Some("--theme")
            | Some("--season")
            | Some("--theme-schedule")
            | Some("--directory")
            | Some("--gallery")
            | Some("--library") => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| invalid("theme option needs a directory"))?;
                if args[index] == "--theme" {
                    base_path = Some(PathBuf::from(value));
                } else if args[index] == "--season" {
                    season_path = Some(PathBuf::from(value));
                } else if args[index] == "--library" {
                    if library_path.is_some() {
                        return Err(invalid("duplicate library option"));
                    }
                    library_path = Some(PathBuf::from(value));
                } else if args[index] == "--gallery" {
                    if gallery_path.is_some() {
                        return Err(invalid("duplicate gallery option"));
                    }
                    gallery_path = Some(PathBuf::from(value));
                } else if args[index] == "--directory" {
                    directory_path = Some(PathBuf::from(value));
                } else {
                    schedule_path = Some(PathBuf::from(value));
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
    let mut themes = match faction_terminal::seasons::Themes::load(
        theme.clone(),
        season_path.as_deref(),
        schedule_path.as_deref(),
    ) {
        Ok(themes) => themes,
        Err(reason) => {
            eprintln!("Season/schedule rejected: {reason}; using the unchanged base theme.");
            faction_terminal::seasons::Themes::fixed(theme)
        }
    };
    let directory = directory_path
        .map(|p| faction_terminal::directory::Directory::load(&p))
        .transpose()
        .map_err(invalid)?;
    themes
        .set_directory_enabled(directory.is_some())
        .map_err(invalid)?;
    let gallery = gallery_path
        .map(|p| faction_gallery::Catalog::load(&p))
        .transpose()
        .map_err(invalid)?
        .unwrap_or_default();
    themes
        .set_gallery_enabled(gallery.enabled())
        .map_err(invalid)?;
    let library = library_path
        .map(|p| faction_library::Catalog::load(&p))
        .transpose()
        .map_err(invalid)?
        .unwrap_or_default();
    themes
        .set_library_enabled(library.enabled())
        .map_err(invalid)?;
    let mut board = Board::open(&path)?;
    board
        .configure_directory_owner(directory.as_ref().map(faction_pages::directory::owner_keys))?;
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
            faction_cli::scheduled_menu_with_library(
                &mut board,
                &s,
                &themes,
                &directory.unwrap_or_default(),
                &gallery,
                &library,
                ansi,
            )?;
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
