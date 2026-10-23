use crate::{Result, edited_line, terminal};
use faction_core::{Board, Session};
use faction_terminal::Theme;
pub(super) fn run(
    b: &mut Board,
    s: &Session,
    theme: &Theme,
    ansi: bool,
    settings: bool,
) -> Result<faction_catchup::Outcome> {
    let desk = if settings {
        faction_catchup::Desk::settings()
    } else {
        faction_catchup::Desk::default()
    };
    let mut desk = desk;
    loop {
        let columns = terminal::size().map_or(80, |s| s.0);
        let frame = match desk.render(b, s, theme, ansi && terminal::capable(), columns as usize) {
            Ok(frame) => frame,
            Err(e) => {
                if desk.recover(&e) {
                    continue;
                } else {
                    return Err(e.into());
                }
            }
        };
        // Enter raw input before displaying its prompt; a flush after the prompt
        // can discard a quick member's first keys.
        let input = edited_line(&frame, 128, false)?;
        match desk.feed(&input, b, s) {
            Ok(faction_catchup::Outcome::Stay) => (),
            Ok(outcome) => return Ok(outcome),
            Err(e) => {
                if !desk.recover(&e) {
                    return Err(e.into());
                }
            }
        }
    }
}
