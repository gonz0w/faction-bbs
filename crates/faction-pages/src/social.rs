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
