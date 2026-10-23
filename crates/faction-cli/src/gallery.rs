use crate::{Result, edited_line, terminal};
use faction_core::{Board, Session};
use faction_terminal::Theme;
pub(super) fn run(
    b: &mut Board,
    s: &Session,
    theme: &Theme,
    catalog: &faction_gallery::Catalog,
    ansi: bool,
) -> Result<()> {
    let mut desk = faction_gallery::Desk::default();
    loop {
        let frame = match desk.render(
            b,
            s,
            catalog,
            theme,
            ansi && terminal::capable(),
            terminal::size().unwrap_or((80, 24)),
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
        match desk.feed(&input, b, s, catalog) {
            Ok(true) => return Ok(()),
            Ok(false) => (),
            Err(e) => {
                if !desk.recover(&e) {
                    return Err(e.into());
                }
            }
        }
    }
}
