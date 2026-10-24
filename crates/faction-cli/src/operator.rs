//! Local OS-authorized read-only console, separate from member/session UIs.
use crate::{Result, invalid, terminal};
use faction_core::operator::{self, Snapshot};
use faction_terminal::{Decoder, Key, Theme};
use std::{
    collections::VecDeque,
    fs,
    io::{self, IsTerminal, Write},
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

mod host;
use host::{Host, Reading, Sampler};

struct Service {
    snapshot: Option<Snapshot>,
    available: bool,
    owner: operator::lease::State,
}
impl Service {
    fn refresh(&mut self, directory: &std::path::Path) {
        match operator::read(directory) {
            Ok(snapshot) => {
                self.owner = operator::lease::observe(directory, &snapshot);
                self.snapshot = Some(snapshot);
                self.available = true;
            }
            Err(_) => {
                self.available = false;
                self.owner = operator::lease::State::Unavailable;
            }
        }
    }
}
fn owner_text(state: operator::lease::State) -> &'static str {
    use operator::lease::State;
    match state {
        State::Held => "LEASE HELD / current instance",
        State::Released => "NO LEASE / publisher inactive",
        State::Different => "DIFFERENT / last-known sample",
        State::Unverified => "UNVERIFIED / legacy sample",
        State::Unavailable => "UNAVAILABLE",
    }
}

#[derive(Default)]
enum Backup {
    #[default]
    Missing,
    Unavailable,
    Recorded(operator::backup::Receipt),
}
impl Backup {
    fn read(directory: &std::path::Path) -> Self {
        match operator::backup::read(directory) {
            Ok(value) => Self::Recorded(value),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Self::Missing,
            Err(_) => Self::Unavailable,
        }
    }
    fn lines(&self, clock: u64) -> Vec<String> {
        let age = |stamp| {
            if stamp == 0 {
                "never".to_owned()
            } else {
                clock
                    .checked_sub(stamp)
                    .map_or_else(|| "CLOCK UNKNOWN".to_owned(), |v| format!("{v}s ago"))
            }
        };
        let mut out = vec!["RECOVERY / historical bundle check".to_owned()];
        match self {
            Self::Missing => out.push("Backup check: UNRECORDED".into()),
            Self::Unavailable => out.push("Backup check: UNAVAILABLE".into()),
            Self::Recorded(r) => {
                out.push(format!(
                    "Last attempt: {}",
                    if r.successful { "VERIFIED" } else { "FAILED" }
                ));
                out.push(format!("Attempt: {}", age(r.attempted)));
                out.push(format!("Last verified: {}", age(r.verified)));
                out.push(format!(
                    "Catalog: {}",
                    mib((r.verified != 0).then_some(r.catalog_bytes))
                ));
                out.push(format!(
                    "Objects: {} / {}",
                    r.objects,
                    mib((r.verified != 0).then_some(r.object_bytes))
                ));
            }
        }
        out.extend(
            [
                "Operator-selected local bundle.",
                "As of check; no live revalidation.",
                "No off-host or restore guarantee.",
                "Themes/OS/device keys excluded.",
                "Federation: not configured.",
            ]
            .map(str::to_owned),
        );
        out
    }
}

#[derive(Default)]
struct History {
    connected: VecDeque<Option<u8>>,
    cpu: VecDeque<Option<u8>>,
    ram: VecDeque<Option<u8>>,
    network: VecDeque<Option<u64>>,
}
impl History {
    fn push(&mut self, snapshot: Option<&Snapshot>, available: bool, host: &Host) {
        fn keep<T>(v: &mut VecDeque<T>, n: T) {
            if v.len() == 60 {
                v.pop_front();
            }
            v.push_back(n);
        }
        keep(
            &mut self.connected,
            snapshot
                .filter(|s| {
                    available && s.running && now().checked_sub(s.sampled).is_some_and(|a| a <= 15)
                })
                .map(|s| (u32::from(s.connected) * 100 / u32::from(s.capacity)) as u8),
        );
        keep(
            &mut self.cpu,
            match host.cpu {
                Reading::Value(v) => Some(v),
                _ => None,
            },
        );
        keep(
            &mut self.ram,
            host.used
                .zip(host.ram)
                .filter(|(_, n)| *n > 0)
                .map(|(used, n)| (u128::from(used) * 100 / u128::from(n)) as u8),
        );
        keep(
            &mut self.network,
            match host.network {
                Reading::Value((rx, tx)) => rx.checked_add(tx),
                _ => None,
            },
        );
    }
}
fn trace(label: &str, values: &VecDeque<Option<u8>>, columns: usize, unicode: bool) -> String {
    let width = columns.saturating_sub(label.len() + 1).min(60);
    let start = values.len().saturating_sub(width);
    let glyph = |value: Option<u8>| match value {
        None => ' ',
        Some(v) if unicode => {
            ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'][usize::from(v.min(100)) * 7 / 100]
        }
        Some(0) => '.',
        Some(1..=25) => ':',
        Some(26..=50) => '*',
        Some(51..=75) => '#',
        _ => '@',
    };
    format!(
        "{label}{}",
        values
            .iter()
            .skip(start)
            .map(|v| glyph(*v))
            .collect::<String>()
    )
}
fn percent(value: Reading<u8>) -> String {
    match value {
        Reading::Value(v) => format!("{v}%"),
        Reading::Warming => "warming up".into(),
        Reading::Unavailable => "unavailable".into(),
    }
}
fn bar(value: Option<u8>, width: usize, unicode: bool) -> String {
    let Some(v) = value else {
        return String::new();
    };
    let n = usize::from(v.min(100)) * width / 100;
    format!(
        " [{}{}]",
        if unicode { "█" } else { "#" }.repeat(n),
        "-".repeat(width - n)
    )
}
struct ExitScreen {
    ansi: bool,
}
impl Drop for ExitScreen {
    fn drop(&mut self) {
        let text = if self.ansi {
            match terminal::size() {
                Some((40..=240, rows @ 18..=100)) => format!("\x1b[{rows};1H\x1b[0m\n"),
                _ => "\n".into(),
            }
        } else {
            "\n".into()
        };
        let _ = io::stdout().write_all(text.as_bytes());
        let _ = io::stdout().flush();
    }
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}
fn number(value: Option<u64>) -> String {
    value.map_or_else(|| "unavailable".into(), |v| v.to_string())
}
fn mib(value: Option<u64>) -> String {
    value.map_or_else(|| "unavailable".into(), |v| format!("{} MiB", v / 1048576))
}
fn safe_line(text: &str, columns: usize) -> String {
    text.chars()
        .take(columns.saturating_sub(1))
        .map(|c| {
            if (c.is_ascii() && !c.is_ascii_control()) || ('▁'..='█').contains(&c) {
                c
            } else {
                '?'
            }
        })
        .collect()
}
fn geometry_line(header: &str, columns: u16, rows: u16) -> String {
    let suffix = format!(" | {columns}x{rows}");
    let prefix = usize::from(columns).saturating_sub(suffix.len() + 1);
    safe_line(header, prefix + 1) + &suffix
}
fn freshness(snapshot: Option<&Snapshot>, available: bool, clock: u64) -> String {
    if !available {
        return "UNAVAILABLE / last-known only".into();
    }
    let Some(s) = snapshot else {
        return "UNAVAILABLE / enable operator_status".into();
    };
    if !s.running {
        return "STOPPED / final snapshot".into();
    }
    match clock.checked_sub(s.sampled) {
        Some(age) if age <= 15 => format!("FRESH sample / {age}s old"),
        Some(age) => format!("STALE / {age}s old"),
        None => "CLOCK UNKNOWN / future sample".into(),
    }
}
fn lines(
    theme: &Theme,
    service: &Service,
    host: &Host,
    state: (usize, bool, usize, bool),
    history: &History,
    backup: &Backup,
) -> Vec<String> {
    let (view, paused, columns, unicode) = state;
    let snapshot = service.snapshot.as_ref();
    let available = service.available;
    let mut out = vec![
        format!("{} / LOCAL SYSOP", theme.board()),
        format!(
            "[B]oard [H]ost [D]ata [I]nfo | {}",
            if paused { "PAUSED" } else { "READ ONLY" }
        ),
        format!("Board: {}", freshness(snapshot, available, now())),
        format!("Owner: {}", owner_text(service.owner)),
    ];
    if view == 0 {
        let counts = snapshot.and_then(|s| s.counts);
        out.push(format!(
            "Connected: {} / {}",
            number(snapshot.map(|s| u64::from(s.connected))),
            number(snapshot.map(|s| u64::from(s.capacity)))
        ));
        out.push(format!(
            "Credentials: {} / 2 | queued: {} B",
            number(snapshot.map(|s| u64::from(s.credentials))),
            number(snapshot.map(|s| s.output_bytes))
        ));
        for (label, a, b) in [
            ("Members / pending", 0, 1),
            ("Rooms / posts", 2, 3),
            ("Channels / retained chat", 4, 5),
            ("Published / pending files", 6, 7),
            ("Transfers / completed games", 8, 9),
        ] {
            out.push(format!(
                "{label}: {} / {}",
                number(counts.map(|v| v[a])),
                number(counts.map(|v| v[b]))
            ));
        }
        out.push(format!(
            "Active doors: {} | output: {}",
            number(snapshot.map(|s| u64::from(s.doors))),
            snapshot.map_or("unknown", |s| if s.output_bytes == 0 {
                "idle"
            } else {
                "buffered"
            })
        ));
        out.push(trace(
            "Connected history: ",
            &history.connected,
            columns,
            unicode,
        ));
        out.push(format!(
            "PID {} / uptime {}s",
            number(snapshot.map(|s| u64::from(s.pid))),
            number(snapshot.map(|s| s.uptime))
        ));
        out.push("Counts are board-wide, not identities.".into());
    } else if view == 1 {
        let width = if columns >= 80 { 20 } else { 8 };
        let cpu = match host.cpu {
            Reading::Value(v) => Some(v),
            _ => None,
        };
        out.push(format!(
            "CPU {} busy: {}{}",
            number(host.cpus.map(u64::from)),
            percent(host.cpu),
            bar(cpu, width, unicode)
        ));
        out.push(format!(
            "Load 1/5/15m: {}",
            host.load.as_deref().unwrap_or("unavailable")
        ));
        let used_percent = host
            .used
            .zip(host.ram)
            .filter(|(_, n)| *n > 0)
            .map(|(u, n)| (u128::from(u) * 100 / u128::from(n)) as u8);
        out.push(format!(
            "RAM used: {}/{}{}",
            mib(host.used),
            mib(host.ram),
            bar(used_percent, width, unicode)
        ));
        out.push(format!(
            "RAM free: {} (used includes cache)",
            mib(host.free)
        ));
        out.push(format!(
            "Disk free: {}/{}",
            mib(host.disk.map(|v| v.0)),
            mib(host.disk.map(|v| v.1))
        ));
        let network = match host.network {
            Reading::Value((rx, tx)) => format!("{}/{} KiB/s", rx / 1024, tx / 1024),
            Reading::Warming => "warming up".into(),
            Reading::Unavailable => "unavailable".into(),
        };
        out.push(format!("Net RX/TX: {network}"));
        out.push(format!(
            "Non-loopback interfaces: {}",
            number(host.interfaces.map(|n| n as u64))
        ));
        out.push(trace("CPU hist: ", &history.cpu, columns, unicode));
        out.push(trace("RAM hist: ", &history.ram, columns, unicode));
        let peak = history.network.iter().flatten().copied().max().unwrap_or(0);
        let net = history
            .network
            .iter()
            .map(|v| {
                v.map(|n| {
                    (u128::from(n) * 100)
                        .checked_div(u128::from(peak))
                        .unwrap_or(0) as u8
                })
            })
            .collect();
        out.push(trace("Net hist*: ", &net, columns, unicode));
        out.push("*window peak; D:backup check".into());
    } else if view == 2 {
        out.extend(backup.lines(now()));
    } else {
        out.extend(
            [
                "OS owner/root access; not a BBS role.",
                "Arrows/Tab or B/H/D/I: change panel.",
                "P: pause/resume local sampling.",
                "R: redraw; sampling stays rate-limited.",
                "Q or Ctrl-C/D: return to OS shell.",
                "Esc: Enter to recover keyboard input.",
                "No writes, actions, message bodies or keys.",
                "Lease held is not responsiveness proof.",
                "Federation: not configured.",
                "No getty/autologin or member-SSH change.",
                "Host changes require separate tools.",
            ]
            .map(str::to_owned),
        );
    }
    out.push(String::new());
    out.push("Q:quit P:pause R:redraw B/H/D/I:views".into());
    out
}

pub fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() || args == ["--help"] {
        println!(
            "Faction local sysop console (read-only, after OS login)\nUsage: faction-sysop --state-dir PATH [--theme PACK] [--ansi|--plain] [--unicode] [--once] [--view board|host|data|help] [--refresh 2..60]\nUnix state-owner/root access required. Never use as a member SSH forced command."
        );
        return Ok(());
    }
    if rustix::process::getuid() != rustix::process::geteuid()
        || rustix::process::getgid() != rustix::process::getegid()
    {
        return Err(invalid("setuid/setgid operator launch refused"));
    }
    let mut directory = None;
    let mut theme = Theme::default();
    let mut ansi = false;
    let mut unicode = false;
    let mut once = false;
    let mut refresh = 5;
    let mut view = 0;
    let mut i = 0;
    while i < args.len() {
        match args[i].to_str() {
            Some("--state-dir") | Some("--theme") | Some("--refresh") | Some("--view") => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| invalid("missing option value"))?;
                if args[i] == "--state-dir" {
                    directory = Some(PathBuf::from(v));
                } else if args[i] == "--theme" {
                    theme = Theme::load(&PathBuf::from(v)).map_err(invalid)?;
                } else if args[i] == "--view" {
                    view = match v.to_str() {
                        Some("board") => 0,
                        Some("host") => 1,
                        Some("data") => 2,
                        Some("help") => 3,
                        _ => return Err(invalid("view must be board, host, data or help")),
                    };
                } else {
                    refresh = v
                        .to_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .filter(|v| (2..=60).contains(v))
                        .ok_or_else(|| invalid("refresh must be 2..60 seconds"))?;
                }
                i += 1;
            }
            Some("--ansi") => ansi = true,
            Some("--plain") => ansi = false,
            Some("--unicode") => unicode = true,
            Some("--once") => once = true,
            _ => return Err(invalid("unknown operator option")),
        }
        i += 1;
    }
    let directory = directory.ok_or_else(|| invalid("--state-dir required"))?;
    let snapshot = match operator::read(&directory) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            return Err(invalid(
                "OS operator permission required; unsafe/private snapshot denied",
            ));
        }
        Err(_) => None,
    };
    // A missing directory is not an authorized operator context.
    let m = fs::symlink_metadata(&directory)?;
    if !m.is_dir() {
        return Err(invalid("private owner directory required"));
    }
    let mut service = Service {
        available: snapshot.is_some(),
        owner: snapshot
            .as_ref()
            .map_or(operator::lease::State::Unavailable, |s| {
                operator::lease::observe(&directory, s)
            }),
        snapshot,
    };
    let interactive = !once && io::stdin().is_terminal() && io::stdout().is_terminal();
    let raw = if interactive {
        Some(terminal::Raw::input_only()?)
    } else {
        None
    };
    let _exit =
        (interactive || (ansi && io::stdout().is_terminal())).then_some(ExitScreen { ansi });
    let mut sampler = Sampler::default();
    let mut host = sampler.sample(&directory);
    let mut backup = Backup::read(&directory);
    let mut history = History::default();
    history.push(
        service.snapshot.as_ref(),
        service.available && service.owner == operator::lease::State::Held,
        &host,
    );
    let mut decoder = Decoder::new();
    let mut paused = false;
    let mut previous = Vec::<String>::new();
    let mut geometry = None;
    let mut next = Instant::now() + Duration::from_secs(refresh);
    let mut redraw = true;
    loop {
        if !paused && Instant::now() >= next {
            next = Instant::now() + Duration::from_secs(refresh);
            service.refresh(&directory);
            host = sampler.sample(&directory);
            backup = Backup::read(&directory);
            history.push(
                service.snapshot.as_ref(),
                service.available && service.owner == operator::lease::State::Held,
                &host,
            );
            redraw = true;
        }
        let size = terminal::size();
        if size != geometry {
            previous.clear();
            geometry = size;
            redraw = true;
        }
        if redraw {
            let use_ansi = ansi && matches!(size, Some((40..=240, 18..=100)));
            let (columns, rows) = size.unwrap_or((80, 24));
            let mut content = lines(
                &theme,
                &service,
                &host,
                (view, paused, usize::from(columns), unicode && use_ansi),
                &history,
                &backup,
            );
            content[2] = geometry_line(&content[2], columns, rows);
            let rendered: Vec<_> = content
                .into_iter()
                .take(usize::from(rows.saturating_sub(1)))
                .map(|s| safe_line(&s, usize::from(columns)))
                .collect();
            let mut frame = String::new();
            if use_ansi {
                if previous.is_empty() {
                    frame.push_str("\x1b[2J\x1b[H");
                }
                for n in 0..rendered.len().max(previous.len()) {
                    if rendered.get(n) != previous.get(n) {
                        frame.push_str(&format!(
                            "\x1b[{};1H{}{}\x1b[K",
                            n + 1,
                            if n == 0 {
                                theme.menu_styles().1.sgr()
                            } else {
                                theme.menu_styles().0.sgr()
                            },
                            rendered.get(n).map_or("", String::as_str)
                        ));
                    }
                }
                frame.push_str("\x1b[0m");
            } else {
                frame = rendered.join("\n") + "\n";
            }
            if frame.len() > 4096 {
                return Err(invalid("operator frame budget"));
            }
            io::stdout().write_all(frame.as_bytes())?;
            io::stdout().flush()?;
            previous = rendered;
            redraw = false;
        }
        if !interactive {
            break;
        }
        if terminal::ready()? {
            let byte = raw.as_ref().expect("interactive raw guard").read_byte()?;
            if let Some(key) = decoder.feed(byte, Instant::now()) {
                match key {
                    Key::Cancel | Key::Hotkey(b'Q') => break,
                    Key::Left | Key::Up => view = (view + 3) % 4,
                    Key::Right | Key::Down | Key::Next => view = (view + 1) % 4,
                    Key::Hotkey(b'B') => view = 0,
                    Key::Hotkey(b'H') => view = 1,
                    Key::Hotkey(b'D') => view = 2,
                    Key::Hotkey(b'I') => view = 3,
                    Key::Hotkey(b'P') => {
                        paused = !paused;
                        sampler.reset();
                    }
                    _ => (),
                }
                redraw = true;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_future_and_unavailable_never_look_fresh() {
        let s = Snapshot {
            sampled: 100,
            pid: 1,
            instance: None,
            uptime: 1,
            running: true,
            connected: 0,
            capacity: 8,
            credentials: 0,
            output_bytes: 0,
            doors: 0,
            counts: None,
        };
        assert!(freshness(Some(&s), true, 116).starts_with("STALE"));
        assert!(freshness(Some(&s), true, 99).starts_with("CLOCK UNKNOWN"));
        assert!(freshness(Some(&s), false, 101).starts_with("UNAVAILABLE"));
    }
    #[test]
    fn owner_lifetime_is_separate_from_sample_age_and_resets_on_failed_read() {
        use operator::lease::State;
        assert!(owner_text(State::Held).contains("LEASE HELD"));
        assert!(owner_text(State::Released).contains("NO LEASE"));
        assert!(owner_text(State::Different).contains("DIFFERENT"));
        assert!(owner_text(State::Unverified).contains("UNVERIFIED"));
        assert!(owner_text(State::Unavailable).contains("UNAVAILABLE"));
        let path = std::env::temp_dir().join(faction_core::request_id().unwrap());
        let mut service = Service {
            snapshot: None,
            available: true,
            owner: State::Held,
        };
        service.refresh(&path);
        assert!(!service.available);
        assert_eq!(service.owner, State::Unavailable);
    }

    #[test]
    fn histories_keep_recent_tail_and_fixed_bounds() {
        let mut history = History::default();
        let host = Host {
            cpu: Reading::Value(100),
            ..Host::default()
        };
        for _ in 0..80 {
            history.push(None, false, &host);
        }
        assert_eq!(history.cpu.len(), 60);
        assert_eq!(history.network.len(), 60);
        history.cpu[59] = Some(0);
        let tail = trace("CPU: ", &history.cpu, 10, false);
        assert_eq!(tail, "CPU: @@@.");
        assert!(trace("CPU: ", &history.cpu, 10, true).ends_with('▁'));
    }
    #[test]
    fn compact_lines_are_inert_and_leave_right_margin() {
        assert_eq!(safe_line("ab\x1bcd", 5), "ab?c");
        let header = geometry_line("Board: UNAVAILABLE / last-known only", 40, 18);
        assert_eq!(header.len(), 39);
        assert!(header.ends_with(" | 40x18"));
        let receipt = Backup::Recorded(operator::backup::Receipt {
            attempted: 10,
            verified: 10,
            successful: true,
            catalog_bytes: 4096,
            object_bytes: 100,
            objects: 1,
        });
        assert!(receipt.lines(20).len() <= 11);
        assert!(receipt.lines(5).iter().any(|s| s.contains("CLOCK UNKNOWN")));
        for view in 0..4 {
            let rows = lines(
                &Theme::default(),
                &Service {
                    snapshot: None,
                    available: false,
                    owner: operator::lease::State::Unavailable,
                },
                &Host::default(),
                (view, false, 40, false),
                &History::default(),
                &receipt,
            );
            assert!(rows.len() <= 17);
            assert!(rows.iter().all(|s| safe_line(s, 40).len() < 40));
        }
    }
}
