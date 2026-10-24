use crate::{Result, edited_line, terminal};
use faction_core::Session;
use faction_terminal::{Theme, capabilities::Capabilities};
pub(super) fn run(
    b: &mut crate::Visit<'_>,
    s: &Session,
    theme: &Theme,
    ansi: bool,
    settings: bool,
    caps: Capabilities,
) -> Result<()> {
    let mut desk = if settings {
        faction_find::Desk::settings()
    } else {
        faction_find::Desk::default()
    };
    loop {
        let frame = match desk.render(
            b,
            s,
            theme,
            ansi && terminal::capable(),
            terminal::size().unwrap_or((80, 24)),
            caps.unicode,
        ) {
            Ok(f) => f,
            Err(e) => {
                if desk.recover(&e) {
                    continue;
                }
                return Err(e.into());
            }
        };
        let input = edited_line(b, &frame, 128, caps.unicode)?;
        match desk.feed(&input, b, s) {
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
