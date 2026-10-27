//! Caller-list presentation, separate from the Lounge's invitation desk.
use faction_core::{Board, ChatLease, Error, Result};
use faction_terminal::Theme;
use std::fmt::Write;

pub fn roster(board: &mut Board, lease: &ChatLease, after: Option<&str>) -> Result<String> {
    Ok(format_roster(board.social_roster(lease, after, false)?))
}
fn format_roster(rows: faction_core::SocialRoster) -> String {
    let mut out =
        String::from("Who / visible board visits\nHandle | Where | Availability | Public status\n");
    for p in rows.people {
        writeln!(
            out,
            "{} | {} | {}{} | {}",
            p.handle,
            p.activity,
            p.mode,
            if p.sysop { ", sysop" } else { "" },
            p.status
        )
        .unwrap();
    }
    if let Some(next) = rows.next {
        writeln!(out, "More: /who {next}").unwrap();
    }
    out.push_str(
        "Same visibility and availability as the Lounge. No hidden counts or private locations. Not a delivery guarantee.\n",
    );
    out
}

#[derive(Default)]
pub struct Desk {
    after: Option<String>,
    next: Option<String>,
    notice: String,
}
impl Desk {
    pub fn render(
        &mut self,
        board: &mut Board,
        lease: &ChatLease,
        theme: &Theme,
        ansi: bool,
    ) -> Result<String> {
        let mut out = faction_terminal::section(theme, "Who is here", ansi);
        let me = board.social_advertisement(lease)?;
        writeln!(
            out,
            "Your visibility: {}",
            if me.is_some() { "visible" } else { "hidden" }
        )
        .unwrap();
        if let Some(me) = me {
            writeln!(out, "Your availability: {} / {}", me.mode, me.status).unwrap();
        }
        let rows = board.social_roster(lease, self.after.as_deref(), false)?;
        self.next = rows.next.clone();
        out.push_str(&format_roster(rows));
        out.push_str(&self.notice);
        out.push_str("R:refresh N:more Q:main; /available /busy /away /hide share settings everywhere.\nUse the Lounge for invitations and paging.\nWho> ");
        if out.len() > 8192 {
            return Err(Error::Invalid("who frame bound"));
        }
        Ok(out)
    }
    pub fn feed(&mut self, input: &str, board: &mut Board, lease: &ChatLease) -> Result<bool> {
        // Every action rechecks the current actor and board feature gate.
        if !board.social_config(lease)?.enabled {
            return Err(Error::Denied);
        }
        self.notice.clear();
        match input.to_ascii_lowercase().as_str() {
            "q" => return Ok(true),
            "" | "r" => self.after = None,
            "n" => self.after = self.next.clone(),
            _ => {
                let verb = input.split(' ').next().unwrap_or("");
                if ![
                    "/available",
                    "/busy",
                    "/away",
                    "/hide",
                    "/privacy",
                    "/mute",
                    "/quiet",
                    "/block",
                    "/unblock",
                ]
                .contains(&verb)
                {
                    return Err(Error::Invalid("Who shows callers; use Lounge for paging"));
                }
                self.notice =
                    crate::social::visit_command(board, lease, input)?.ok_or(Error::NotFound)?;
                self.after = None;
            }
        }
        Ok(false)
    }
    pub fn recover(&mut self, error: &Error) -> bool {
        if matches!(error, Error::Invalid(_) | Error::NotFound | Error::Busy) {
            self.notice = format!("{error}\n");
            true
        } else {
            false
        }
    }
}
