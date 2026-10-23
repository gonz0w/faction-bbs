//! Room writing; only explicit send publishes, only explicit save checkpoints.
use super::*;
use faction_terminal::composer::{Command, Draft, HELP, PROMPT, Screen, ScreenEvent, Target};
fn quote(board: &mut Board, s: &Session, target: &Target<'_>, draft: &mut Draft) -> Result<()> {
    let parent = target
        .reply
        .ok_or_else(|| invalid("select a reply target before quoting"))?;
    board.reply_subject(s, target.room, parent)?;
    let rows = board.posts(s, target.room, parent - 1)?;
    let p = rows
        .first()
        .filter(|p| p.id == parent)
        .ok_or_else(|| invalid("reply target unavailable"))?;
    draft.quote(&p.body).map_err(invalid)
}
fn preview(
    board: &mut Board,
    s: &Session,
    target: &Target<'_>,
    draft: &Draft,
    theme: &Theme,
    ansi: bool,
    caps: faction_terminal::capabilities::Capabilities,
) -> Result<()> {
    let mut offset = 0;
    loop {
        board.unread_cursor(s, target.room)?;
        let page = draft
            .preview(
                target,
                terminal::size().unwrap_or((80, 24)),
                offset,
                caps.unicode,
            )
            .map_err(invalid)?;
        print!(
            "{}{}",
            caps.section(
                theme,
                &format!("{} preview", theme.writing_title()),
                ansi && terminal::capable()
            ),
            page.text
        );
        if let Some(next) = page.next {
            if !line("Preview more? [y/N]: ", 8)?.eq_ignore_ascii_case("y") {
                break;
            }
            offset = next;
        } else {
            break;
        }
    }
    Ok(())
}
struct ScreenContext<'a> {
    target: Target<'a>,
    theme: &'a Theme,
    caps: faction_terminal::capabilities::Capabilities,
    saved: bool,
}
fn screen_input(
    board: &mut Board,
    s: &Session,
    screen: &mut Screen,
    draft: &mut Draft,
    context: ScreenContext<'_>,
) -> Result<Command> {
    let raw = terminal::Raw::open()?;
    let mut redraw = true;
    let mut size = terminal::size().unwrap_or((0, 0));
    let mut at = Instant::now();
    loop {
        if at.elapsed() > Duration::from_secs(120) {
            return Err(invalid(
                "writing timed out; last explicit checkpoint retained",
            ));
        }
        let current = terminal::size().unwrap_or((0, 0));
        if !terminal::capable() {
            println!("Terminal resized; returning to line editing with draft intact.");
            drop(raw);
            line("Press Enter to resume line editing: ", 1024)?;
            return Ok(Command::Line);
        }
        if current != size {
            size = current;
            redraw = true;
        }
        if redraw {
            board.unread_cursor(s, context.target.room)?;
            print!(
                "{}",
                context.caps.frame(
                    &screen.render(
                        draft,
                        &context.target,
                        size,
                        context.caps.unicode,
                        context.theme,
                        (context.saved, "^S checkpoints edits")
                    ),
                    true
                )
            );
            io::stdout().flush()?;
            redraw = false;
        }
        if !terminal::ready()? {
            continue;
        }
        board.unread_cursor(s, context.target.room)?;
        let byte = raw.read_byte()?;
        at = Instant::now();
        match screen.feed(draft, byte, at, size, context.caps.unicode) {
            Ok(ScreenEvent::Command(c)) => return Ok(c),
            Ok(ScreenEvent::Changed) => redraw = true,
            Ok(ScreenEvent::None) => (),
            Err(e) => {
                println!("{e}; returning to line editing with draft intact.");
                drop(raw);
                line("Press Enter to resume line editing: ", 1024)?;
                return Ok(Command::Line);
            }
        }
    }
}
pub(super) fn run(
    board: &mut Board,
    s: &Session,
    target: Target<'_>,
    theme: &Theme,
    ansi: bool,
    caps: faction_terminal::capabilities::Capabilities,
    saved: Option<faction_core::SavedRoomDraft>,
) -> Result<Option<(String, Option<String>)>> {
    let mut draft =
        Draft::from_body(saved.as_ref().map_or("", |d| d.body.as_str())).map_err(invalid)?;
    let mut key = saved.map(|d| d.request_id);
    let mut screen = None;
    print!(
        "{}",
        caps.section(
            theme,
            &format!("{} | room #{}", theme.writing_title(), target.room),
            ansi && terminal::capable()
        )
    );
    println!("Room post only. Save is plaintext on this board, readable by its sysop.");
    print!("{HELP}");
    loop {
        board.unread_cursor(s, target.room)?;
        if let Some(parent) = target.reply {
            board.reply_subject(s, target.room, parent)?;
        }
        let command = if let Some(view) = screen.as_mut() {
            Ok(screen_input(
                board,
                s,
                view,
                &mut draft,
                ScreenContext {
                    target: Target {
                        room: target.room,
                        reply: target.reply,
                        subject: target.subject,
                    },
                    theme,
                    caps,
                    saved: key.is_some(),
                },
            )?)
        } else {
            draft.command(&edited_line(PROMPT, 1024, caps.unicode)?)
        };
        board.unread_cursor(s, target.room)?;
        match command {
            Ok(Command::Send) => {
                if draft.body().trim().is_empty() {
                    println!("empty draft; write before sending");
                    continue;
                }
                if screen.is_some()
                    && line("Type SEND to post, or Enter to keep editing: ", 8)? != "SEND"
                {
                    continue;
                }
                return Ok(Some((draft.body().to_owned(), key)));
            }
            Ok(Command::Save) => match board.save_room_draft(
                s,
                target.room,
                key.as_deref(),
                faction_core::PostDraft {
                    subject: target.subject,
                    body: draft.body(),
                    reply_to: target.reply,
                },
            ) {
                Ok(saved) => {
                    key = Some(saved.request_id);
                    println!("Checkpoint saved on this board for 30 days; edits need Save again.");
                }
                Err(e) => println!("Save failed; previous checkpoint retained: {e}"),
            },
            Ok(Command::Cancel) => {
                if let Some(key) = key.as_deref() {
                    board.delete_room_draft(s, target.room, key)?;
                }
                println!("Draft cancelled; nothing sent.");
                return Ok(None);
            }
            Ok(Command::Leave) => {
                println!("Writing closed; last explicit checkpoint kept.");
                return Ok(None);
            }
            Ok(Command::Screen) => {
                if ansi && terminal::capable() {
                    screen = Some(Screen::at_end(&draft));
                } else {
                    println!("Full-screen unavailable; classic line editing remains active.");
                }
            }
            Ok(Command::Line) => {
                screen = None;
                print!(
                    "{}",
                    caps.section(theme, theme.writing_title(), ansi && terminal::capable())
                );
                println!("Classic line editing; draft kept.");
            }
            Ok(Command::Help) => print!("{HELP}"),
            Ok(Command::Quote) => {
                if let Err(e) = quote(board, s, &target, &mut draft) {
                    println!("{e}");
                }
            }
            Ok(Command::Preview) => preview(board, s, &target, &draft, theme, ansi, caps)?,
            Ok(Command::Changed) => (),
            Err(e) => println!("{e}"),
        }
        if screen.is_none() {
            println!(
                "Draft: {} bytes / {} lines. Nothing sent.",
                draft.body().len(),
                draft.lines()
            );
        }
    }
}
