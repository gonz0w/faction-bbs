//! Same inert cards/navigation as the shared owner; no connector or subprocess.
use crate::{Result, invalid, terminal};
use faction_core::{Session, Status};
use faction_terminal::{
    Decoder, Key, Theme,
    directory::{Directory, View},
};
use std::{
    io::{self, Write},
    time::Instant,
};
pub(super) fn run(
    board: &mut crate::Visit<'_>,
    session: &Session,
    theme: &Theme,
    directory: &Directory,
    ansi: bool,
) -> Result<()> {
    let dimensions = terminal::size().unwrap_or((80, 24));
    let ansi = ansi && terminal::capable();
    let raw = if ansi {
        Some(terminal::Raw::open()?)
    } else {
        None
    };
    let mut decoder = Decoder::new();
    let mut view = View::default();
    let mut redraw = true;
    loop {
        board.tick()?;
        if redraw {
            if board.whoami(session)?.status != Status::Approved {
                return Err(invalid("current local approval required"));
            }
            io::stdout().write_all(
                directory
                    .render(&view, dimensions.0, dimensions.1, ansi, theme)
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
            let Some(key) = decoder.feed(raw.read_byte()?, Instant::now()) else {
                continue;
            };
            key
        } else {
            let input = board.line("Directory command: ", 1)?;
            if input.is_empty() {
                Key::Enter
            } else {
                Key::Hotkey(input.as_bytes()[0].to_ascii_uppercase())
            }
        };
        if key == Key::Cancel {
            return Err(std::io::Error::from(std::io::ErrorKind::Interrupted).into());
        }
        if board.whoami(session)?.status != Status::Approved {
            return Err(invalid("current local approval required"));
        }
        if view.feed(key, directory, dimensions.0, dimensions.1) {
            break;
        }
        redraw = true;
    }
    Ok(())
}
