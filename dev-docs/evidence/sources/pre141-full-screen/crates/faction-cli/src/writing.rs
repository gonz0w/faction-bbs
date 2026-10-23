//! Explicit opt-in multiline room writing; only .send creates a submission.
use super::*;
use faction_terminal::composer::{Command, Draft, HELP, PROMPT};
pub(super) fn run(
    board: &mut Board,
    session: &Session,
    target: faction_terminal::composer::Target<'_>,
    theme: &Theme,
    ansi: bool,
    caps: faction_terminal::capabilities::Capabilities,
) -> Result<Option<String>> {
    let faction_terminal::composer::Target { room, reply, .. } = target;
    let mut draft = Draft::default();
    print!(
        "{}",
        caps.section(
            theme,
            &format!("Writing desk | room #{room}"),
            ansi && terminal::capable()
        )
    );
    println!("Room post only; visible to this board. Your draft is not saved.");
    print!("{HELP}");
    loop {
        board.unread_cursor(session, room)?;
        if let Some(parent) = reply {
            board.reply_subject(session, room, parent)?;
        }
        let input = edited_line(PROMPT, faction_terminal::composer::BYTES, caps.unicode)?;
        board.unread_cursor(session, room)?;
        match draft.command(&input) {
            Ok(Command::Send) => return Ok(Some(draft.body().to_owned())),
            Ok(Command::Cancel) => {
                println!("Draft cancelled; nothing sent.");
                return Ok(None);
            }
            Ok(Command::Help) => print!("{HELP}"),
            Ok(Command::Quote) => {
                let quoted = (|| -> Result<()> {
                    let parent =
                        reply.ok_or_else(|| invalid("select a reply target before quoting"))?;
                    board.reply_subject(session, room, parent)?;
                    let rows = board.posts(session, room, parent - 1)?;
                    let post = rows
                        .first()
                        .filter(|p| p.id == parent)
                        .ok_or_else(|| invalid("reply target unavailable"))?;
                    draft.quote(&post.body).map_err(invalid)?;
                    Ok(())
                })();
                if let Err(e) = quoted {
                    println!("{e}");
                }
            }
            Ok(Command::Preview) => {
                let mut offset = 0;
                loop {
                    board.unread_cursor(session, room)?;
                    let (cols, rows) = terminal::size().unwrap_or((80, 24));
                    let page = draft
                        .preview(&target, (cols, rows), offset, caps.unicode)
                        .map_err(invalid)?;
                    print!(
                        "{}{}",
                        caps.section(theme, "Writing desk preview", ansi && terminal::capable()),
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
            }
            Ok(Command::Changed) => (),
            Err(e) => println!("{e}"),
        }
        println!(
            "Draft: {} bytes / {} lines. Nothing sent.",
            draft.body().len(),
            draft.lines()
        );
    }
}
