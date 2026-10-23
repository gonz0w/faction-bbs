use crate::{Result, edited_line, terminal};
use faction_core::{Board, Session};
use faction_terminal::Theme;
pub(super) fn run(
    b: &mut Board,
    s: &Session,
    t: &Theme,
    c: &faction_library::Catalog,
    ansi: bool,
) -> Result<faction_library::Outcome> {
    let mut desk = faction_library::Desk::default();
    loop {
        let frame = match desk.render(
            b,
            s,
            c,
            t,
            ansi && terminal::capable(),
            terminal::size().map_or(80, |s| s.0),
        ) {
            Ok(f) => f,
            Err(e) => {
                if desk.recover(&e) {
                    continue;
                }
                return Err(e.into());
            }
        };
        let input = edited_line(&frame, 64, false)?;
        match desk.feed(&input, b, s, c) {
            Ok(faction_library::Outcome::Stay) => (),
            Ok(out) => return Ok(out),
            Err(e) => {
                if !desk.recover(&e) {
                    return Err(e.into());
                }
            }
        }
    }
}
