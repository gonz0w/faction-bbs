//! Local authoring preview only: no board, login, listener or member authority.
use faction_terminal::{Menu, Theme};
use std::{
    io::{self, IsTerminal, Write},
    path::PathBuf,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let (mut root, mut season, mut columns, mut plain) = (None, None, 80u16, false);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--theme" => root = Some(PathBuf::from(args.next().ok_or("missing theme path")?)),
            "--season" => season = Some(PathBuf::from(args.next().ok_or("missing season path")?)),
            "--columns" => columns = args.next().ok_or("missing columns")?.parse()?,
            "--plain" => plain = true,
            _ => {
                return Err(
                    "use --theme PATH [--season PATH] [--columns 40..240] [--plain]".into(),
                );
            }
        }
    }
    if !(40..=240).contains(&columns) {
        return Err("columns must be 40..240".into());
    }
    let mut theme = Theme::load(&root.ok_or("--theme required")?)?;
    if let Some(path) = season {
        theme = theme.with_season(&path)?;
    }
    // Preview optional services as available; this never enables a real service.
    theme.set_doors_enabled(true);
    theme.set_files_enabled(true)?;
    let frame = if plain {
        theme.plain_menu(true)
    } else {
        Menu::with_columns(&theme, true, columns).render()
    };
    let mut out = io::stdout().lock();
    out.write_all(frame.as_bytes())?;
    if !plain && io::stdout().is_terminal() {
        out.write_all(b"\x1b[18;1H\x1b[0m\n")?;
    }
    out.flush()?;
    eprintln!(
        "Authoring preview only; {} encoded bytes. Optional services shown enabled.",
        frame.len()
    );
    Ok(())
}
