use crate::{Result, edited_line, terminal};
use faction_core::{Board, Session};
use faction_terminal::Theme;
pub(super) fn run(
    b: &mut Board,
    s: &Session,
    theme: &Theme,
    ansi: bool,
    settings: bool,
) -> Result<faction_gatherings::Outcome> {
    let desk = if settings {
        faction_gatherings::Desk::settings()
    } else {
        faction_gatherings::Desk::default()
    };
    run_desk(b, s, theme, ansi, desk)
}
pub(super) fn run_event(
    b: &mut Board,
    s: &Session,
    theme: &Theme,
    ansi: bool,
    id: i64,
) -> Result<faction_gatherings::Outcome> {
    run_desk(b, s, theme, ansi, faction_gatherings::Desk::open(id))
}
fn run_desk(
    b: &mut Board,
    s: &Session,
    theme: &Theme,
    ansi: bool,
    mut desk: faction_gatherings::Desk,
) -> Result<faction_gatherings::Outcome> {
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
            Ok(faction_gatherings::Outcome::Stay) => (),
            Ok(outcome) => return Ok(outcome),
            Err(e) => {
                if !desk.recover(&e) {
                    return Err(e.into());
                }
            }
        }
    }
}
