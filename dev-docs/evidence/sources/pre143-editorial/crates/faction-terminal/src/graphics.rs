//! Trusted original calibration artwork only. No wire-supplied paths or pixels.
use std::time::{Duration, Instant};
pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 80;
pub const RAW_BYTES: usize = WIDTH * HEIGHT * 3;
pub const CHUNK_RAW: usize = 3000;
pub const CHUNKS: usize = RAW_BYTES.div_ceil(CHUNK_RAW);
pub const MAX_COMMAND: usize = 4096;
pub const QUERY: &str = "\x1b_Gi=17392,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\";
pub const DELETE: &str = "\x1b_Ga=d,d=I,i=17391,q=2;\x1b\\";
pub const CLEANUP: &str = "\x1b\\\x1b_Ga=d,d=I,i=17391,q=2;\x1b\\";
pub const TIMEOUT: Duration = Duration::from_millis(300);
pub const INTERVAL: Duration = Duration::from_secs(1);
pub const TRANSFER_TIMEOUT: Duration = Duration::from_secs(5);
const fn artwork() -> [u8; RAW_BYTES] {
    let mut a = [0; RAW_BYTES];
    let mut y = 0;
    while y < HEIGHT {
        let mut x = 0;
        while x < WIDTH {
            // Original frequency sigil: two offset angular rails and scan lines.
            let rail = (x > 14 && x < 145 && (y == 13 || y == 66))
                || (x > 24 && x < 67 && y > 23 && y < 29)
                || (x > 24 && x < 59 && y > 38 && y < 44)
                || (x > 24 && x < 30 && y > 23 && y < 57)
                || (x > 83 && x < 133 && y > 49 && y < 55)
                || (x > 83 && x < 119 && y > 34 && y < 40)
                || (x > 83 && x < 89 && y > 23 && y < 55);
            let i = (y * WIDTH + x) * 3;
            a[i] = if rail { 200 } else { 8 };
            a[i + 1] = if rail { 60 + x as u8 } else { 4 };
            a[i + 2] = if rail {
                255
            } else if y % 4 == 0 {
                35
            } else {
                16
            };
            x += 1;
        }
        y += 1;
    }
    a
}
/// One immutable read-only asset per executable, not an image cache per session.
pub static RGB: [u8; RAW_BYTES] = artwork();
/// One bounded allocation, no full-frame RGB/base64 temporary or decoder.
pub fn chunk(index: usize) -> Option<String> {
    if index >= CHUNKS {
        return None;
    }
    let start = index * CHUNK_RAW;
    let end = (start + CHUNK_RAW).min(RAW_BYTES);
    let mut out = String::with_capacity(MAX_COMMAND);
    if index == 0 {
        out.push_str("\x1b_Ga=T,t=d,f=24,s=160,v=80,i=17391,c=20,r=5,q=2,m=1;");
    } else {
        out.push_str(if index + 1 == CHUNKS {
            "\x1b_Gm=0,q=2;"
        } else {
            "\x1b_Gm=1,q=2;"
        });
    }
    const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for p in RGB[start..end].as_chunks::<3>().0 {
        let n = (u32::from(p[0]) << 16) | (u32::from(p[1]) << 8) | u32::from(p[2]);
        for shift in [18, 12, 6, 0] {
            out.push(B64[((n >> shift) & 63) as usize] as char);
        }
    }
    out.push_str("\x1b\\");
    debug_assert!(out.len() <= MAX_COMMAND);
    Some(out)
}
/// Dedicated probe owns every response byte; none are commands. Bounded error
/// reports are a refusal, not text to render. No DA query or multiplexer wrapping.
#[derive(Default)]
pub struct Probe {
    bytes: Vec<u8>,
    started: Option<Instant>,
}
impl Probe {
    pub fn begin(&mut self, now: Instant) {
        if self.started.is_none() {
            self.started = Some(now);
        }
    }
    pub fn expired(&self, now: Instant) -> bool {
        self.started
            .is_some_and(|t| now.saturating_duration_since(t) >= TIMEOUT)
    }
    pub fn feed(&mut self, byte: u8) -> crate::Result<()> {
        if self.bytes.len() >= 128 {
            return Err("graphics probe overflow");
        }
        let i = self.bytes.len();
        let prefix = b"\x1b_Gi=17392;";
        if i < prefix.len() {
            if byte != prefix[i] {
                return Err("unexpected graphics probe input");
            }
        } else if self.bytes.last() == Some(&27) {
            if byte != b'\\' {
                return Err("unexpected graphics probe terminator");
            }
        } else if self.bytes.ends_with(b"\x1b\\") || !(byte == 27 || (32..=126).contains(&byte)) {
            return Err("unexpected graphics probe input");
        }
        self.bytes.push(byte);
        Ok(())
    }
    pub fn result(&self) -> crate::Result<bool> {
        if self.bytes.is_empty() {
            return Ok(false);
        }
        let payload = self
            .bytes
            .strip_prefix(b"\x1b_Gi=17392;")
            .and_then(|b| b.strip_suffix(b"\x1b\\"))
            .ok_or("incomplete graphics probe")?;
        if payload.is_empty() {
            return Err("empty graphics probe");
        }
        Ok(payload == b"OK")
    }
}
/// Native member-side allowlist, enabled only after local opt-in and a local
/// positive probe. Regenerate exact trusted commands; never pass arbitrary APC.
#[derive(Default)]
pub struct Gate {
    pub enabled: bool,
    index: usize,
    last: Option<Instant>,
    query: Option<Instant>,
    started: Option<Instant>,
}
impl Gate {
    pub fn expired(&self, now: Instant) -> bool {
        self.started
            .is_some_and(|t| now.saturating_duration_since(t) >= TRANSFER_TIMEOUT)
    }
    pub fn accept(&mut self, command: &str, now: Instant) -> crate::Result<Option<String>> {
        if !self.enabled {
            return Ok(None);
        }
        if command == DELETE {
            self.index = 0;
            self.started = None;
            return Ok(Some(DELETE.into()));
        }
        if self.expired(now) {
            return Err("graphics transfer timeout");
        }
        if command == QUERY {
            if self.index != 0
                || self
                    .query
                    .is_some_and(|t| now.saturating_duration_since(t) < INTERVAL)
            {
                return Err("graphics query rate/order");
            }
            self.query = Some(now);
            return Ok(Some(QUERY.into()));
        }
        if self.index == 0
            && self
                .last
                .is_some_and(|t| now.saturating_duration_since(t) < INTERVAL)
        {
            return Err("graphics frame rate");
        }
        let expected = chunk(self.index).ok_or("graphics chunk index")?;
        if command != expected {
            return Err("untrusted graphics command");
        }
        if self.index == 0 {
            self.last = Some(now);
            self.started = Some(now);
        }
        self.index = (self.index + 1) % CHUNKS;
        if self.index == 0 {
            self.started = None;
        }
        Ok(Some(expected))
    }
}

/// Bounded TTY write for standalone/native clients. Restore descriptor flags even
/// on failure. The shared owner already has its own nonblocking output scheduler.
pub fn write_tty(file: &mut std::fs::File, data: &[u8], deadline: Instant) -> std::io::Result<()> {
    use rustix::{
        event::{PollFd, PollFlags, Timespec, poll},
        fs::{OFlags, fcntl_getfl, fcntl_setfl},
    };
    use std::io::{self, Write};
    let flags = fcntl_getfl(&*file)?;
    fcntl_setfl(&*file, flags | OFlags::NONBLOCK)?;
    let result = (|| {
        let mut written = 0;
        while written < data.len() {
            let left = deadline
                .checked_duration_since(Instant::now())
                .ok_or(io::ErrorKind::TimedOut)?;
            let mut fds = [PollFd::new(&*file, PollFlags::OUT)];
            let wait = Timespec {
                tv_sec: left.as_secs() as _,
                tv_nsec: i64::from(left.subsec_nanos()),
            };
            if poll(&mut fds, Some(&wait))? == 0 {
                return Err(io::ErrorKind::TimedOut.into());
            }
            if fds[0]
                .revents()
                .intersects(PollFlags::HUP | PollFlags::ERR | PollFlags::NVAL)
            {
                return Err(io::ErrorKind::BrokenPipe.into());
            }
            match file.write(&data[written..]) {
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(n) => written += n,
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    })();
    let restore = fcntl_setfl(&*file, flags).map_err(io::Error::from);
    result.and(restore)
}
/// Never block terminal restoration behind a stopped emulator. This only deletes
/// our fixed image id; on broken transport the emulator may require a screen reset.
pub fn cleanup(file: &mut std::fs::File) {
    let _ = write_tty(
        file,
        CLEANUP.as_bytes(),
        Instant::now() + Duration::from_millis(20),
    );
}
