//! ANSI named room picker; the normal numeric/plain workflows stay available.
use crate::{Result, board_room_rows, terminal};
use faction_core::{Board, Session};
use faction_terminal::{
    Decoder, Key, Theme,
    capabilities::Capabilities,
    rooms::{Choice, View},
};
use std::{
    io::{self, Write},
    time::Instant,
};
pub(super) fn choose(
    board: &mut Board,
    session: &Session,
    theme: &Theme,
    caps: Capabilities,
) -> Result<Choice> {
    let dimensions = terminal::size().unwrap_or((80, 24));
    let raw = terminal::Raw::open()?;
    let mut decoder = Decoder::new();
    let mut view = View::default();
    let mut redraw = true;
    loop {
        if redraw {
            let rows = board.rooms(session)?;
            let items = board_room_rows(&rows);
            io::stdout().write_all(
                caps.frame(
                    &view.render(&items, dimensions.0, true, caps.unicode, theme),
                    true,
                )
                .as_bytes(),
            )?;
            io::stdout().flush()?;
            redraw = false;
        }
        if terminal::size() != Some(dimensions) {
            println!("\nTerminal resized; returning to main menu.");
            return Ok(Choice::Main);
        }
        decoder.expire(Instant::now());
        if !terminal::ready()? {
            continue;
        }
        let Some(key) = decoder.feed(raw.read_byte()?, Instant::now()) else {
            continue;
        };
        if key == Key::Cancel {
            return Err(io::Error::from(io::ErrorKind::Interrupted).into());
        }
        let rows = board.rooms(session)?;
        let items = board_room_rows(&rows);
        match view.feed(key, &items) {
            Choice::Stay => redraw = true,
            choice => return Ok(choice),
        }
    }
}
