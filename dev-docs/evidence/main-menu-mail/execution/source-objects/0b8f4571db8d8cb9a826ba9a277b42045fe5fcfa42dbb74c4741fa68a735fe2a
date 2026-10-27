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
    run_panel(board, session, theme, ansi, false)
}

pub(super) fn who(
    board: &mut Visit<'_>,
    session: &Session,
    theme: &Theme,
    ansi: bool,
) -> Result<()> {
    run_panel(board, session, theme, ansi, true)
}
fn run_panel(
    board: &mut Visit<'_>,
    session: &Session,
    theme: &Theme,
    ansi: bool,
    who: bool,
) -> Result<()> {
    if !board.social_session_config(session)?.enabled {
        return Err(faction_core::Error::Denied.into());
    }
    board.ensure(session)?;
    let _raw = terminal::Raw::input_only()?;
    let mut desk = Desk::default();
    let mut who_desk = faction_pages::who::Desk::default();
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
            let (board, lease, notifications) = board.parts()?;
            let frame = if who {
                who_desk.render(board, lease, theme, ansi && terminal::capable())?
            } else {
                desk.render_with_notifications(
                    board,
                    lease,
                    theme,
                    ansi && terminal::capable(),
                    notifications,
                )?
            };
            print!("{frame}");
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
                let (core, lease, notifications) = board.parts()?;
                let outcome = if who {
                    who_desk.feed(&text, core, lease)
                } else {
                    desk.feed_with_notifications(&text, core, lease, notifications)
                };
                match outcome {
                    Ok(true) => return Ok(()),
                    Ok(false) => (),
                    Err(e) => {
                        if !(if who {
                            who_desk.recover(&e)
                        } else {
                            desk.recover(&e)
                        }) {
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
