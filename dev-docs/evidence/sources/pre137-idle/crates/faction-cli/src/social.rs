//! Current-read local/worker Lounge; only safe menu redraws show page notices.
use crate::{Result, Session, Theme, Visit, terminal};
use faction_pages::social::Desk;
use faction_terminal::subject::{Edit, feed_limited};
use std::io::{self, Write};

pub(super) fn run(
    board: &mut Visit<'_>,
    session: &Session,
    theme: &Theme,
    ansi: bool,
) -> Result<()> {
    if !board.social_session_config(session)?.enabled {
        return Err(faction_core::Error::Denied.into());
    }
    board.ensure(session)?;
    let _raw = terminal::Raw::input_only()?;
    let mut desk = Desk::default();
    let mut input = Vec::with_capacity(128);
    let mut size = terminal::size();
    let mut redraw = true;
    loop {
        board.tick()?;
        if redraw && !board.social_session_config(session)?.enabled {
            println!("Social lounge disabled; returning to main menu.");
            return Ok(());
        }
        let current = terminal::size();
        if current != size {
            size = current;
            input.clear();
            rustix::termios::tcflush(&crate::tty()?, rustix::termios::QueueSelector::IFlush)?;
            redraw = true;
        }
        if redraw {
            let (board, lease) = board.parts()?;
            print!(
                "{}",
                desk.render(board, lease, theme, ansi && terminal::capable())?
            );
            io::stdout().flush()?;
            redraw = false;
        }
        if !terminal::ready()? {
            continue;
        }
        let byte = _raw.read_byte()?;
        if byte == 3 || byte == 4 {
            return Ok(());
        }
        match feed_limited(&mut input, byte, false, 128).map_err(crate::invalid)? {
            Edit::Echo(text) => {
                print!("{text}");
                io::stdout().flush()?;
            }
            Edit::Submit(text) => {
                println!();
                let (core, lease) = board.parts()?;
                match desk.feed(&text, core, lease) {
                    Ok(true) => return Ok(()),
                    Ok(false) => (),
                    Err(e) => {
                        if !desk.recover(&e) {
                            return Err(e.into());
                        }
                    }
                }
                rustix::termios::tcflush(&crate::tty()?, rustix::termios::QueueSelector::IFlush)?;
                redraw = true;
            }
        }
    }
}
