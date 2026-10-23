//! Local OS-authorized read-only console, separate from member/session UIs.
use crate::{Result, invalid, terminal};
use faction_core::operator::{self, Snapshot};
use faction_terminal::{Decoder, Key, Theme};
use std::{
    collections::VecDeque,
    fs,
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
struct Host {
    cpus: Option<u32>,
    ram: Option<u64>,
    load: Option<String>,
    disk: Option<(u64, u64)>,
}
impl Host {
    fn sample(directory: &Path) -> Self {
        let mut host = Self::default();
        if let Ok(v) = rustix::fs::statvfs(directory) {
            host.disk = u64::try_from(u128::from(v.f_bavail) * u128::from(v.f_frsize))
                .ok()
                .zip(u64::try_from(u128::from(v.f_blocks) * u128::from(v.f_frsize)).ok());
        }
        let command = if cfg!(target_os = "netbsd") {
            Some(("/sbin/sysctl", "hw.physmem64"))
        } else if cfg!(target_os = "macos") {
            Some(("/usr/sbin/sysctl", "hw.memsize"))
        } else {
            None
        };
        if let Some((binary, memory)) = command
            && let Some(text) = query_host(binary, memory)
        {
            let lines: Vec<_> = text.lines().collect();
            if lines.len() == 3 {
                host.cpus = lines[0].trim().parse().ok();
                host.ram = lines[1].trim().parse().ok();
                let load = lines[2].trim().trim_matches(['{', '}']).trim();
                if load.len() <= 48
                    && load
                        .bytes()
                        .all(|b| b.is_ascii_digit() || b". ".contains(&b))
                {
                    host.load = Some(load.to_owned());
                }
            }
        }
        host
    }
}
fn query_host(binary: &str, memory: &str) -> Option<String> {
    // Source-defined absolute command/keys only. No PATH search, shell or -w.
    let mut child = Command::new(binary)
        .args(["-n", "hw.ncpu", memory, "vm.loadavg"])
        .env_clear()
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + Duration::from_millis(250);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                let mut text = String::new();
                child
                    .stdout
                    .take()?
                    .take(1025)
                    .read_to_string(&mut text)
                    .ok()?;
                return (text.len() <= 1024 && text.is_ascii()).then_some(text);
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(5)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
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
            if c.is_ascii() && !c.is_ascii_control() {
                c
            } else {
                '?'
            }
        })
        .collect()
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
    snapshot: Option<&Snapshot>,
    host: &Host,
    available: bool,
    state: (usize, bool),
    history: &VecDeque<Option<u8>>,
) -> Vec<String> {
    let (view, paused) = state;
    let mut out = vec![
        format!("{} / LOCAL SYSOP", theme.board()),
        format!(
            "[B]oard [H]ost [I]nfo | {}",
            if paused { "PAUSED" } else { "READ ONLY" }
        ),
        freshness(snapshot, available, now()),
        String::new(),
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
            "Active doors: {}",
            number(snapshot.map(|s| u64::from(s.doors)))
        ));
        let trace: String = history
            .iter()
            .map(|v| match v {
                None => ' ',
                Some(0) => '.',
                Some(1..=25) => ':',
                Some(26..=50) => '*',
                Some(51..=75) => '#',
                _ => '@',
            })
            .collect();
        out.push(format!("Connected history: {trace}"));
        out.push("Counts are board-wide, not identities.".into());
    } else if view == 1 {
        out.push(format!("Host CPUs: {}", number(host.cpus.map(u64::from))));
        out.push(format!(
            "CPU load 1/5/15m: {}",
            host.load.as_deref().unwrap_or("unavailable")
        ));
        out.push(format!("Physical RAM: {}", mib(host.ram)));
        out.push("RAM used/free: unavailable".into());
        out.push(format!("Board disk free: {}", mib(host.disk.map(|v| v.0))));
        out.push(format!("Board disk size: {}", mib(host.disk.map(|v| v.1))));
        out.push("Network/peers/backup: unavailable".into());
        out.push(format!(
            "Service PID: {}",
            number(snapshot.map(|s| u64::from(s.pid)))
        ));
        out.push(format!(
            "Service uptime: {} s",
            number(snapshot.map(|s| s.uptime))
        ));
        out.push("Fresh sample does not prove liveness.".into());
    } else {
        out.extend(
            [
                "OS owner/root access; not a BBS role.",
                "Arrows/Tab or B/H/I: change panel.",
                "P: pause/resume local sampling.",
                "R: redraw; sampling stays rate-limited.",
                "Q or Ctrl-C/D: return to OS shell.",
                "Esc: Enter to recover keyboard input.",
                "No writes, actions, message bodies or keys.",
                "No getty/autologin or member-SSH change.",
                "Host changes require separate tools.",
            ]
            .map(str::to_owned),
        );
    }
    out.push(String::new());
    out.push("Q:quit P:pause R:redraw B/H/I:views".into());
    out
}

pub fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() || args == ["--help"] {
        println!(
            "Faction local sysop console (read-only, after OS login)\nUsage: faction-sysop --state-dir PATH [--theme PACK] [--ansi|--plain] [--once] [--view board|host|help] [--refresh 2..60]\nUnix state-owner/root access required. Never use as a member SSH forced command."
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
                        Some("help") => 2,
                        _ => return Err(invalid("view must be board, host or help")),
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
            Some("--once") => once = true,
            _ => return Err(invalid("unknown operator option")),
        }
        i += 1;
    }
    let directory = directory.ok_or_else(|| invalid("--state-dir required"))?;
    let mut snapshot = match operator::read(&directory) {
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
    let mut available = snapshot.is_some();
    let interactive = !once && io::stdin().is_terminal() && io::stdout().is_terminal();
    let raw = if interactive {
        Some(terminal::Raw::input_only()?)
    } else {
        None
    };
    let mut host = Host::sample(&directory);
    let mut history = VecDeque::with_capacity(60);
    let mut decoder = Decoder::new();
    let mut paused = false;
    let mut previous = Vec::<String>::new();
    let mut geometry = None;
    let mut next = Instant::now() + Duration::from_secs(refresh);
    let mut redraw = true;
    loop {
        if !paused && Instant::now() >= next {
            next = Instant::now() + Duration::from_secs(refresh);
            match operator::read(&directory) {
                Ok(s) => {
                    snapshot = Some(s);
                    available = true;
                }
                Err(_) => available = false,
            }
            host = Host::sample(&directory);
            if history.len() == 60 {
                history.pop_front();
            }
            history.push_back(
                snapshot
                    .as_ref()
                    .filter(|s| {
                        available
                            && s.running
                            && now().checked_sub(s.sampled).is_some_and(|age| age <= 15)
                    })
                    .map(|s| ((u32::from(s.connected) * 100) / u32::from(s.capacity)) as u8),
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
                snapshot.as_ref(),
                &host,
                available,
                (view, paused),
                &history,
            );
            content[2].push_str(&format!(" | {columns}x{rows}"));
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
                    Key::Left | Key::Up => view = (view + 2) % 3,
                    Key::Right | Key::Down | Key::Next => view = (view + 1) % 3,
                    Key::Hotkey(b'B') => view = 0,
                    Key::Hotkey(b'H') => view = 1,
                    Key::Hotkey(b'I') => view = 2,
                    Key::Hotkey(b'P') => paused = !paused,
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
    fn compact_lines_are_inert_and_leave_right_margin() {
        assert_eq!(safe_line("ab\x1bcd", 5), "ab?c");
        for view in 0..3 {
            let rows = lines(
                &Theme::default(),
                None,
                &Host::default(),
                false,
                (view, false),
                &VecDeque::new(),
            );
            assert!(rows.len() <= 17);
            assert!(rows.iter().all(|s| safe_line(s, 40).len() < 40));
        }
    }
}
