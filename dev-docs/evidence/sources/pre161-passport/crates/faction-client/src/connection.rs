//! Deliberate device-local diagnostics. No sockets, SSH login or remote setters.
use crate::{
    Terminal,
    profile::{self, Settings},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use faction_files::invalid;
use rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const PIN_BYTES: usize = 32768;
const HELPER_SECONDS: u64 = 2;
#[derive(Default, Debug)]
struct Pins {
    keys: Vec<String>,
    special: bool,
    extra: bool,
}
impl Pins {
    fn parse(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() > PIN_BYTES {
            return Err(invalid());
        }
        let text = std::str::from_utf8(bytes).map_err(|_| invalid())?;
        let mut out = Self::default();
        for line in text
            .lines()
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
        {
            let mut fields = line.split_ascii_whitespace();
            let host = fields.next().ok_or_else(invalid)?;
            if host.starts_with('@') {
                // CA/revoked policy is OpenSSH's decision, never a positive trust badge.
                out.special = true;
                continue;
            }
            let kind = fields.next().ok_or_else(invalid)?;
            if kind.len() > 64
                || !kind
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"@.-_".contains(&b))
            {
                return Err(invalid());
            }
            let encoded = fields.next().ok_or_else(invalid)?;
            let blob = STANDARD.decode(encoded).map_err(|_| invalid())?;
            if blob.len() < 4 || blob.len() > 16384 {
                return Err(invalid());
            }
            let n = u32::from_be_bytes(blob[..4].try_into().map_err(|_| invalid())?) as usize;
            if n != kind.len()
                || blob.get(4..4 + n) != Some(kind.as_bytes())
                || STANDARD.encode(&blob) != encoded
            {
                return Err(invalid());
            }
            let fingerprint = format!(
                "SHA256:{}",
                STANDARD.encode(Sha256::digest(&blob)).trim_end_matches('=')
            );
            if !out.keys.contains(&fingerprint) {
                if out.keys.len() < 4 {
                    out.keys.push(fingerprint);
                } else {
                    out.extra = true;
                }
            }
        }
        Ok(out)
    }
    fn lines(&self) -> Vec<String> {
        let mut out = vec![if self.keys.is_empty() {
            "No ordinary matching host pins.".into()
        } else {
            "Stored matching key fingerprints:".into()
        }];
        out.extend(self.keys.iter().cloned());
        if self.extra {
            out.push("More matching keys omitted (limit 4).".into());
        }
        if self.special {
            out.push("CA/revoked policy entries present.".into());
        }
        out.push("Stored pins are not live verification.".into());
        out
    }
}

/// Fixed local OpenSSH lookup, using an already checked file as stdin. Never ssh.
fn pins(
    settings: &Settings,
    files: bool,
    mut cancel: impl FnMut() -> io::Result<bool>,
) -> io::Result<Pins> {
    let f = profile::regular(&settings.known_hosts, 1024 * 1024, false)?;
    let (host, port) = if files {
        (&settings.file_host, settings.file_port)
    } else {
        (&settings.host, settings.port)
    };
    let query = if port == 22 {
        host.clone()
    } else {
        format!("[{host}]:{port}")
    };
    let mut c = Command::new("/usr/bin/ssh-keygen")
        .args(["-F", &query, "-f", "/dev/stdin"])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LC_ALL", "C")
        .stdin(Stdio::from(f))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let result = (|| {
        let mut output = c.stdout.take().ok_or_else(invalid)?;
        fcntl_setfl(&output, fcntl_getfl(&output)? | OFlags::NONBLOCK)?;
        let deadline = Instant::now() + Duration::from_secs(HELPER_SECONDS);
        let mut bytes = Vec::new();
        let mut eof = false;
        loop {
            if cancel()? {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelled"));
            }
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "local pin lookup timed out",
                ));
            }
            let mut b = [0; 4096];
            match output.read(&mut b) {
                Ok(0) => eof = true,
                Ok(n) => {
                    if bytes.len() + n > PIN_BYTES {
                        return Err(invalid());
                    }
                    bytes.extend_from_slice(&b[..n]);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => (),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
            if eof && let Some(status) = c.try_wait()? {
                // OpenSSH returns 1 for an empty match. Errors/diagnostics never become pins.
                if !(status.success() || (status.code() == Some(1) && bytes.is_empty())) {
                    return Err(invalid());
                }
                return Pins::parse(&bytes);
            }
            let mut fds = [rustix::event::PollFd::new(
                &output,
                rustix::event::PollFlags::IN,
            )];
            rustix::event::poll(
                &mut fds,
                Some(&rustix::event::Timespec {
                    tv_sec: 0,
                    tv_nsec: 20_000_000,
                }),
            )?;
        }
    })();
    // Reap on every outcome, including cancellation, malformed output and deadlines.
    let _ = c.kill();
    let _ = c.wait();
    result
}

pub(super) struct Report {
    settings: Option<Settings>,
    identity: bool,
    public: bool,
    known: bool,
    terminal_pins: Option<Vec<String>>,
    file_pins: Option<Vec<String>>,
}
impl Report {
    pub fn load(path: &Path) -> Self {
        let settings = Settings::load(path).ok();
        let identity = settings
            .as_ref()
            .is_some_and(|s| profile::regular(&s.identity, 65536, false).is_ok());
        let public = settings.as_ref().is_some_and(|s| s.public_key().is_ok());
        let known = settings
            .as_ref()
            .is_some_and(|s| profile::regular(&s.known_hosts, 1024 * 1024, false).is_ok());
        Self {
            settings,
            identity,
            public,
            known,
            terminal_pins: None,
            file_pins: None,
        }
    }
    pub fn ready(&self) -> bool {
        self.settings.is_some() && self.identity && self.public && self.known
    }
    /// Deliberate copy/share preview; no path, address, label, key, fingerprint or timestamps.
    pub fn share_summary(&self) -> String {
        format!(
            "Faction local connection check v1\nProfile structure: {}\nAdmission key file: {}\nAdmission public key: {}\nHost-pin file: {}\nPrivate-key unlock: not tested\nRemote host identity: not tested\nSSH admission: not tested\nBBS login/approval: not tested\nTor/invitation/route verification: not implemented\nLocal-only check; no network connection.\nAddresses, paths and key material omitted.\n",
            if self.settings.is_some() {
                "parsed"
            } else {
                "needs attention"
            },
            if self.identity {
                "readable"
            } else {
                "needs attention"
            },
            if self.public {
                "valid Ed25519 format"
            } else {
                "needs attention"
            },
            if self.known {
                "readable"
            } else {
                "needs attention"
            }
        )
    }
    fn lines(&self, label: &str) -> Vec<String> {
        let mut out = vec![
            format!("Board: {label}"),
            "DEVICE CHECK / no connection".into(),
        ];
        if let Some(s) = &self.settings {
            out.extend([
                format!("Terminal: {} port {}", s.host, s.port),
                format!("Files: {} port {}", s.file_host, s.file_port),
            ]);
        } else {
            out.push("Profile cannot be parsed/read safely.".into());
        }
        out.extend(
            self.share_summary()
                .lines()
                .skip(1)
                .take(8)
                .map(str::to_owned),
        );
        out.extend([
            "".into(),
            "LOCAL HOST-PIN LOOKUP".into(),
            "K: inspect stored pins (no network).".into(),
            "Terminal pins:".into(),
        ]);
        out.extend(
            self.terminal_pins
                .clone()
                .unwrap_or_else(|| vec!["Not checked this view.".into()]),
        );
        out.push("Files pins:".into());
        out.extend(
            self.file_pins
                .clone()
                .unwrap_or_else(|| vec!["Not checked this view.".into()]),
        );
        out.extend([
            "".into(),
            "RECOVERY / independent stages".into(),
            "Obtain profile/host keys out of band.".into(),
            "Fix file ownership/modes manually.".into(),
            "A readable key may still be locked.".into(),
            "Check SSH agent/unlock separately.".into(),
            "Verify changed host keys with sysop.".into(),
            "Never accept a key just to get in.".into(),
            "SSH admission is not BBS approval.".into(),
            "Ask destination sysop about approval.".into(),
            "Contacts have no verified age here.".into(),
            "Tor/invitation checks unavailable.".into(),
            "No routes changed or probed.".into(),
            "S previews a redacted share summary.".into(),
            "No report/history is saved.".into(),
        ]);
        out
    }
}
fn screen(lines: &[String], columns: usize, rows: usize, selected: &mut usize) -> String {
    if !(40..=240).contains(&columns) || !(18..=100).contains(&rows) {
        return "\x1b[2J\x1b[HResize to 40x18.\r\nQ: back\r\n".into();
    }
    let width = columns.clamp(40, 240) - 1;
    let height = rows.clamp(18, 100) - 7;
    let mut wrapped = Vec::new();
    for line in lines {
        if line.is_empty() {
            wrapped.push(String::new());
        } else {
            wrapped.extend(
                line.as_bytes()
                    .chunks(width)
                    .map(|b| String::from_utf8_lossy(b).into_owned()),
            );
        }
    }
    let pages = wrapped.len().div_ceil(height).max(1);
    *selected = (*selected).min(pages - 1);
    format!(
        "\x1b[2J\x1b[HLOCAL CONNECTION DESK\r\nPage {} / {}\r\n\r\n{}\r\n\r\nN/P:page R:reload K:pins S:share Q:back\r\n",
        *selected + 1,
        pages,
        wrapped
            .iter()
            .skip(*selected * height)
            .take(height)
            .cloned()
            .collect::<Vec<_>>()
            .join("\r\n")
    )
}
pub(super) fn panel(t: &mut Terminal, label: &str, path: &Path) -> io::Result<()> {
    let mut report = Report::load(path);
    let mut page = 0usize;
    let mut share = false;
    rustix::termios::tcflush(&t.input, rustix::termios::QueueSelector::IFlush)?;
    loop {
        let size = rustix::termios::tcgetwinsize(&t.input)?;
        let lines = if share {
            report.share_summary().lines().map(str::to_owned).collect()
        } else {
            report.lines(label)
        };
        t.print(&screen(
            &lines,
            size.ws_col as usize,
            size.ws_row as usize,
            &mut page,
        ))?;
        match t.line("Desk> ", 1)?.to_ascii_lowercase().as_str() {
            "q" => return Ok(()),
            "n" => page = page.saturating_add(1),
            "p" => page = page.saturating_sub(1),
            "r" => {
                report = Report::load(path);
                page = 0;
                share = false;
            }
            "s" => {
                share = !share;
                page = 0;
            }
            "k" if !share => {
                if let Some(s) = &report.settings {
                    t.print("Checking local pins; Ctrl-C cancels.\r\n")?;
                    let mut lookup = |files| -> io::Result<Vec<String>> {
                        match pins(s, files, || Ok(t.ready(0)? && matches!(t.byte()?, 3 | 4))) {
                            Ok(v) => Ok(v.lines()),
                            Err(e) if e.kind() == io::ErrorKind::Interrupted => Err(e),
                            Err(_) => Ok(vec![
                                "Lookup unavailable; inspect locally.".into(),
                                "No live identity conclusion.".into(),
                            ]),
                        }
                    };
                    report.terminal_pins = Some(lookup(false)?);
                    report.file_pins = Some(lookup(true)?);
                    // Move to the pin section, with normal paging for narrow screens.
                    let before = report
                        .lines(label)
                        .iter()
                        .position(|l| l == "LOCAL HOST-PIN LOOKUP")
                        .unwrap_or(0);
                    page = before / (usize::from(size.ws_row).clamp(18, 100) - 7);
                }
            }
            _ => (),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pins_render_only_deduplicated_fingerprints_and_never_host_fields_or_comments() {
        let key = "AAAAC3NzaC1lZDI1NTE5AAAAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let line = format!("secret.example ssh-ed25519 {key} evil\\x1b comment\n");
        let pins = Pins::parse(
            format!("# header\n{line}{line}@revoked hidden ssh-ed25519 {key}\n").as_bytes(),
        )
        .unwrap();
        assert_eq!(pins.keys.len(), 1);
        assert!(pins.special);
        let text = pins.lines().join("\n");
        assert!(text.contains("SHA256:"));
        for secret in ["secret.example", "hidden", "evil", key] {
            assert!(!text.contains(secret));
        }
        for bad in [
            b"bad".as_slice(),
            b"host ssh-ed25519 !!!!",
            b"host fake AAAAC3NzaC1lZDI1NTE5AAAAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            &[255],
        ] {
            assert!(Pins::parse(bad).is_err());
        }
        assert!(Pins::parse(&vec![b'x'; PIN_BYTES + 1]).is_err());
    }
    #[test]
    fn plain_share_omits_contact_details_and_separates_unknown_remote_authority() {
        let r = Report {
            settings: Some(Settings {
                host: "secret.invalid".into(),
                port: 2222,
                file_host: "files.secret".into(),
                file_port: 2223,
                identity: "/secret/key".into(),
                public: "/secret/key.pub".into(),
                known_hosts: "/secret/hosts".into(),
            }),
            identity: true,
            public: false,
            known: true,
            terminal_pins: Some(vec!["SHA256:secret".into()]),
            file_pins: None,
        };
        assert!(!r.ready());
        let text = r.share_summary();
        assert!(text.contains("BBS login/approval: not tested"));
        for s in ["secret", "2222", "2223", "SHA256:"] {
            assert!(!text.contains(s));
        }
        let mut page = usize::MAX;
        let view = screen(&r.lines("Board"), 40, 18, &mut page);
        let plain = view.replace("\x1b[2J\x1b[H", "");
        assert!(plain.lines().count() < 18);
        assert!(plain.lines().all(|s| s.len() < 40));
    }
}
