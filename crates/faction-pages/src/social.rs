//! One bounded command presenter shared by the local/worker and service chat UIs.
use faction_core::{Board, ChatLease, Error, Result, SocialConfig, request_id};
use std::fmt::Write;
fn minute(v: &str) -> Result<u16> {
    let (h, m) = v
        .split_once(':')
        .ok_or(Error::Invalid("UTC HH:MM required"))?;
    if h.len() != 2 || m.len() != 2 || !h.bytes().chain(m.bytes()).all(|b| b.is_ascii_digit()) {
        return Err(Error::Invalid("UTC HH:MM required"));
    }
    let h: u16 = h
        .parse()
        .map_err(|_| Error::Invalid("UTC HH:MM required"))?;
    let m: u16 = m
        .parse()
        .map_err(|_| Error::Invalid("UTC HH:MM required"))?;
    if h > 23 || m > 59 {
        return Err(Error::Invalid("UTC HH:MM required"));
    }
    Ok(h * 60 + m)
}
const HELP: &str = "/available [public status] /busy [public status] /away [public status] /hide\n/who [after-handle] /pages /page handle /page-sysop [handle]\n/answer ID accept|decline /mute on|off /block handle /unblock handle\n/quiet HH:MM HH:MM (UTC) or /quiet off; /privacy shows your controls.\nPages are bodyless invitations, not private messages. Accepting never switches channels.\nAvailability lasts only during this chat entry; /pages explicitly checks requests.\nSysop: /social on|off|name LABEL. No OS privileges.\n";
pub fn command(b: &mut Board, l: &ChatLease, input: &str) -> Result<Option<String>> {
    command_scoped(b, l, input, false)
}
pub fn visit_command(b: &mut Board, l: &ChatLease, input: &str) -> Result<Option<String>> {
    command_scoped(b, l, input, true)
}
fn command_scoped(
    b: &mut Board,
    l: &ChatLease,
    input: &str,
    visit: bool,
) -> Result<Option<String>> {
    let (verb, args) = input.split_once(' ').unwrap_or((input, ""));
    let args = args.trim();
    let verb = verb.to_ascii_lowercase();
    if ![
        "/social",
        "/available",
        "/busy",
        "/away",
        "/hide",
        "/who",
        "/pages",
        "/page",
        "/page-sysop",
        "/answer",
        "/mute",
        "/quiet",
        "/block",
        "/unblock",
        "/privacy",
    ]
    .contains(&verb.as_str())
    {
        return Ok(None);
    }
    if input.len() > 128 || !input.bytes().all(|v| (32..=126).contains(&v)) {
        return Err(Error::Invalid("bounded ASCII lounge command required"));
    }
    let mut out = String::new();
    match verb.as_str() {
        "/social" => {
            let mut c = b.social_config(l)?;
            if args.is_empty() || args == "help" {
                writeln!(
                    out,
                    "{} [{}]",
                    c.name,
                    if c.enabled { "enabled" } else { "disabled" }
                )
                .unwrap();
                out.push_str(HELP);
            } else {
                match args {
                    "on" => c.enabled = true,
                    "off" => c.enabled = false,
                    _ => {
                        c.name = args
                            .strip_prefix("name ")
                            .ok_or(Error::Invalid("/social on|off|name LABEL"))?
                            .into()
                    }
                }
                b.social_configure(
                    l,
                    &SocialConfig {
                        enabled: c.enabled,
                        name: c.name,
                    },
                )?;
                out.push_str("Social configuration updated.\n");
            }
        }
        "/available" | "/busy" | "/away" | "/hide" => {
            if verb == "/hide" && !args.is_empty() {
                return Err(Error::Invalid("/hide takes no status"));
            }
            let mode = if verb == "/hide" {
                "hidden"
            } else {
                &verb[1..]
            };
            b.social_availability(l, mode, args)?;
            out.push_str(if mode=="hidden"{"Lounge availability hidden; existing invitations removed.\n"}else{"Availability shared with local lounge viewers for this chat entry.\nStatus is public to approved viewers; use /hide to leave the roster.\n"});
        }
        "/who" | "/page-sysop" if verb == "/who" || args.is_empty() => {
            let c = b.social_config(l)?;
            let rows = b.social_roster(
                l,
                if args.is_empty() { None } else { Some(args) },
                verb == "/page-sysop",
            )?;
            writeln!(out, "{} / opted-in chat availability", c.name).unwrap();
            for p in rows.people {
                writeln!(
                    out,
                    "{} [{}{}] {}",
                    p.handle,
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
                "No hidden-member counts or last-seen history. Not a delivery guarantee.\n",
            );
        }
        "/pages" => {
            if !args.is_empty() {
                return Err(Error::Invalid("/pages takes no arguments"));
            }
            for p in b.social_pages(l)? {
                writeln!(
                    out,
                    "Page {} {} {}: {} [{}]",
                    p.id,
                    if p.outgoing { "to" } else { "from" },
                    if p.outgoing { &p.to } else { &p.from },
                    p.state,
                    if p.outgoing { "sent" } else { "received" }
                )
                .unwrap();
            }
            out.push_str("Requests/replies expire after 60 seconds; /answer ID accept|decline.\n");
        }
        "/page" | "/page-sysop" => {
            b.social_request(l, args, &request_id()?, verb == "/page-sysop")?;
            out.push_str(
                "Page attempted if the recipient currently accepts invitations. Check /pages.\n",
            );
        }
        "/answer" => {
            let (id, answer) = args
                .split_once(' ')
                .ok_or(Error::Invalid("/answer ID accept|decline"))?;
            let id = id
                .parse::<i64>()
                .ok()
                .filter(|n| *n > 0)
                .ok_or(Error::Invalid("positive page ID required"))?;
            if !["accept", "decline"].contains(&answer) {
                return Err(Error::Invalid("accept or decline required"));
            }
            b.social_answer(l, id, answer == "accept")?;
            out.push_str("Page answered. Choose a shared channel yourselves; nothing switched.\n");
        }
        "/mute" => {
            if !["on", "off"].contains(&args) {
                return Err(Error::Invalid("/mute on|off"));
            }
            let p = b.social_prefs(l)?;
            b.social_preferences(l, args == "on", p.quiet)?;
            out.push_str("Page preference updated; old invitations removed.\n");
        }
        "/quiet" => {
            let p = b.social_prefs(l)?;
            let quiet = if args == "off" {
                None
            } else {
                let (a, z) = args
                    .split_once(' ')
                    .ok_or(Error::Invalid("/quiet HH:MM HH:MM or off"))?;
                Some((minute(a)?, minute(z)?))
            };
            b.social_preferences(l, p.muted, quiet)?;
            out.push_str("UTC quiet hours updated; old invitations removed.\n");
        }
        "/block" | "/unblock" => {
            b.social_block(l, args, verb == "/block")?;
            out.push_str(
                "Local block preference updated; no account existence/status disclosed.\n",
            );
        }
        "/privacy" => {
            if !args.is_empty() {
                return Err(Error::Invalid("/privacy takes no arguments"));
            }
            let p = b.social_prefs(l)?;
            writeln!(out, "Pages muted: {}", p.muted).unwrap();
            if let Some((a, z)) = p.quiet {
                writeln!(
                    out,
                    "Quiet UTC: {:02}:{:02} to {:02}:{:02}",
                    a / 60,
                    a % 60,
                    z / 60,
                    z % 60
                )
                .unwrap();
            } else {
                out.push_str("Quiet UTC: off\n");
            }
            writeln!(out, "Your blocked handles: {}", p.blocks.join(" ")).unwrap();
        }
        _ => unreachable!(),
    }
    if out.len() > 4096 {
        return Err(Error::Invalid("lounge frame budget"));
    }
    if visit {
        out = out
            .replace("during this chat entry", "during this BBS visit")
            .replace("for this chat entry", "for this BBS visit")
            .replace(
                "Availability lasts only during this chat entry",
                "Availability lasts during this BBS visit",
            );
    }
    Ok(Some(out))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quiet_time_parser_rejects_extra_fields_and_invalid_clock_values() {
        assert_eq!(minute("23:59").unwrap(), 1439);
        for s in [
            "+1:00",
            "24:00",
            "23:60",
            "0:30",
            "12:3",
            "12:30 extra",
            "12:30:00",
            "-1:00",
        ] {
            assert!(minute(s).is_err(), "{s}");
        }
    }
}

/// Enable only in hosts which maintain a connection-scoped lease outside chat.
pub fn menu_theme(
    b: &mut Board,
    s: &faction_core::Session,
    lease: Option<&ChatLease>,
    theme: &faction_terminal::Theme,
) -> Result<Option<faction_terminal::Theme>> {
    if !b.social_session_config(s)?.enabled {
        return Ok(None);
    }
    let mut t = theme.clone();
    t.set_lounge_enabled(true).map_err(Error::Invalid)?;
    if let Some(l) = lease {
        let waiting = b
            .social_pages(l)?
            .iter()
            .any(|p| (!p.outgoing && p.state == "pending") || (p.outgoing && p.state != "pending"));
        t.set_lounge_notice(waiting);
    }
    Ok(Some(t))
}
#[derive(Default)]
pub struct Desk {
    notice: String,
    after: Option<String>,
    view: u8,
}
impl Desk {
    pub fn render(
        &mut self,
        b: &mut Board,
        l: &ChatLease,
        theme: &faction_terminal::Theme,
        ansi: bool,
    ) -> Result<String> {
        let c = b.social_config(l)?;
        if !c.enabled {
            return Err(Error::Invalid("social lounge disabled by sysop"));
        }
        let mut out = faction_terminal::section(theme, &c.name, ansi);
        out.push_str("Board-session lounge / approved local members\nAvailability lasts across menus, rooms and doors until disconnect.\n");
        out.push_str("Q:main R:refresh N:more H:help; /social help for controls.\nMain menu ?:refresh page badge; no popup interruptions.\n");
        writeln!(
            out,
            "Your advertisement: {}",
            b.social_advertisement(l)?
                .map(|p| format!("{} / {}", p.mode, p.status))
                .unwrap_or_else(|| "hidden".into())
        )
        .unwrap();
        if !self.notice.is_empty() {
            writeln!(out, "{}", self.notice).unwrap();
        }
        if self.view == 1 {
            out.push_str(&visit_command(b, l, "/privacy")?.ok_or(Error::NotFound)?);
        }
        if self.view == 2 {
            out.push_str(&HELP.replace("during this chat entry", "during this BBS visit"));
        }
        let roster = b.social_roster(l, self.after.as_deref(), self.view == 3)?;
        for p in &roster.people {
            writeln!(
                out,
                "{} [{}{}] {}",
                p.handle,
                p.mode,
                if p.sysop { ", sysop" } else { "" },
                p.status
            )
            .unwrap();
        }
        out.push_str("Current requests/replies (expire; no delivery guarantee):\n");
        for p in b.social_pages(l)? {
            writeln!(
                out,
                "Page {} {} {}: {}",
                p.id,
                if p.outgoing { "to" } else { "from" },
                if p.outgoing { p.to } else { p.from },
                p.state
            )
            .unwrap();
        }
        out.push_str(
            "No private message bodies. Accepting does not change your location.\nLounge> ",
        );
        if out.len() > 8192 {
            return Err(Error::Invalid("lounge frame budget"));
        }
        Ok(out)
    }
    pub fn feed(&mut self, input: &str, b: &mut Board, l: &ChatLease) -> Result<bool> {
        self.notice.clear();
        let cmd = input.to_ascii_lowercase();
        match cmd.as_str() {
            "q" | "/quit" => return Ok(true),
            "" | "r" => {
                self.view = 0;
                self.after = None;
            }
            "h" => self.view = 2,
            "n" => {
                let rows = b.social_roster(l, self.after.as_deref(), self.view == 3)?;
                self.after = rows.next;
            }
            _ => {
                let out = visit_command(b, l, input)?
                    .ok_or(Error::Invalid("use displayed lounge commands"))?;
                let (verb, args) = cmd.split_once(' ').unwrap_or((&cmd, ""));
                match verb {
                    "/privacy" => self.view = 1,
                    "/social" if args.is_empty() || args == "help" => self.view = 2,
                    "/page-sysop" if args.is_empty() => {
                        self.view = 3;
                        self.after = None;
                    }
                    "/who" => {
                        self.view = 0;
                        self.after = if args.trim().is_empty() {
                            None
                        } else {
                            Some(args.trim().into())
                        };
                    }
                    "/pages" => self.view = 0,
                    _ => {
                        self.view = 0;
                        self.notice = out;
                    }
                }
            }
        }
        Ok(false)
    }
    pub fn recover(&mut self, e: &Error) -> bool {
        if matches!(e, Error::Invalid(_) | Error::NotFound | Error::Busy) {
            self.notice = format!("Lounge: {e}");
            true
        } else {
            false
        }
    }
}
