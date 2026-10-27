//! Synchronous guide, on the existing visit/input heartbeat and Board connection.
use crate::{Result, Visit, terminal};
use faction_core::{Error, Role, Session, Status};
use faction_terminal::{Decoder, Key, Theme, guide::View};
use std::{
    io::{self, Write},
    time::Instant,
};
fn current(b: &mut Visit<'_>, s: &Session, base: &Theme) -> Result<(Theme, bool)> {
    let m = b.whoami(s)?;
    if m.status != Status::Approved {
        return Err(Error::Denied.into());
    }
    let mut t = base.clone();
    macro_rules! apply {
        ($f:path) => {
            if let Some(next) = $f(b, s, &t)? {
                t = next;
            }
        };
    }
    apply!(faction_wall::menu_theme);
    apply!(faction_pages::menu_theme);
    if let Some(next) = b.menu_theme(s, &t)? {
        t = next;
    }
    apply!(faction_gatherings::menu_theme);
    apply!(faction_polls::menu_theme);
    apply!(faction_catchup::menu_theme);
    apply!(faction_find::menu_theme);
    Ok((t, m.role == Role::Sysop))
}
pub(super) fn run(b: &mut Visit<'_>, s: &Session, base: &Theme, ansi: bool) -> Result<()> {
    let dimensions = terminal::size().unwrap_or((80, 24));
    let ansi = ansi && terminal::capable();
    let raw = if ansi {
        Some(terminal::Raw::open()?)
    } else {
        None
    };
    let mut view = View::default();
    let mut decoder = Decoder::new();
    let mut redraw = true;
    loop {
        b.tick()?;
        if redraw {
            let (t, sysop) = current(b, s, base)?;
            io::stdout().write_all(
                view.render(&t, sysop, dimensions.0, dimensions.1, ansi)
                    .as_bytes(),
            )?;
            io::stdout().flush()?;
            redraw = false;
        }
        let key = if let Some(raw) = &raw {
            if terminal::size() != Some(dimensions) {
                println!("\nTerminal resized; returning to main menu.");
                break;
            }
            decoder.expire(Instant::now());
            if !terminal::ready()? {
                continue;
            }
            let Some(k) = decoder.feed(raw.read_byte()?, Instant::now()) else {
                continue;
            };
            k
        } else {
            let v = b.line("Guide command: ", 1)?;
            if v.is_empty() {
                Key::Enter
            } else {
                Key::Hotkey(v.as_bytes()[0].to_ascii_uppercase())
            }
        };
        let (t, sysop) = current(b, s, base)?;
        if view.feed(key, &t, sysop) {
            break;
        }
        redraw = true;
    }
    Ok(())
}
