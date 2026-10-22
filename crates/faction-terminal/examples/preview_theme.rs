//! Local authoring preview only: no board, login, listener or member authority.
use faction_terminal::{
    Menu, Theme,
    seasons::{Date, Themes},
};
use std::{
    io::{self, IsTerminal, Write},
    path::PathBuf,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let (
        mut root,
        mut season,
        mut schedule,
        mut columns,
        mut plain,
        mut login,
        mut date,
        mut timestamp,
    ) = (None, None, None, 80u16, false, false, None, None);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--theme" => root = Some(PathBuf::from(args.next().ok_or("missing theme path")?)),
            "--season" => season = Some(PathBuf::from(args.next().ok_or("missing season path")?)),
            "--theme-schedule" => schedule = Some(PathBuf::from(args.next().ok_or("missing schedule path")?)),
            "--date" => date = Some(Date::parse(&args.next().ok_or("missing date")?)?),
            "--timestamp" => timestamp = Some(args.next().ok_or("missing timestamp")?.parse::<i64>()?),
            "--columns" => columns = args.next().ok_or("missing columns")?.parse()?,
            "--plain" => plain = true,
            "--login" => login = true,
            _ => return Err("use --theme PATH [--season PATH|--theme-schedule PATH] [--date YYYY-MM-DD|--timestamp SECONDS] [--columns 40..240] [--plain] [--login]".into()),
        }
    }
    if !(40..=240).contains(&columns) || (date.is_some() && timestamp.is_some()) {
        return Err("invalid columns or conflicting preview dates".into());
    }
    let base = Theme::load(&root.ok_or("--theme required")?)?;
    let mut themes = Themes::load(base, season.as_deref(), schedule.as_deref())?;
    themes.set_services(true, true)?;
    let index = if let Some(date) = date {
        themes.select(Some(date))
    } else if let Some(stamp) = timestamp {
        themes.select(themes.date_at(stamp))
    } else {
        themes.select_now()
    };
    let theme = themes.get(index);
    let frame = if login {
        let mut text = theme.identity_screen(columns, if columns >= 80 { 24 } else { 18 }, !plain);
        // Canonical positioning keeps the authoring SVG grammar bounded; the
        // product supplies its own approved handle/welcome/input pause.
        if !plain {
            text.push_str(if columns >= 80 {
                "\x1b[18;1H"
            } else {
                "\x1b[13;1H"
            });
        } else {
            text.push('\n');
        }
        text.push_str(theme.welcome());
        text
    } else if plain {
        theme.plain_menu(true)
    } else {
        Menu::with_columns(theme, true, columns).render()
    };
    let mut out = io::stdout().lock();
    out.write_all(frame.as_bytes())?;
    if !plain && io::stdout().is_terminal() {
        out.write_all(b"\x1b[20;1H\x1b[0m\n")?;
    }
    out.flush()?;
    eprintln!(
        "Authoring preview only; theme={}, index={}, {} encoded bytes. Optional services shown enabled.",
        theme.id(),
        index,
        frame.len()
    );
    Ok(())
}
