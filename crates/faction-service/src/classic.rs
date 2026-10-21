//! Fixed-destination classic-server connection. No account or score authority.
use rustix::net::{AddressFamily, SocketType, connect, socket};
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};
const IAC: u8 = 255;
const QUEUE: usize = 4096;
static ACTIVE: AtomicUsize = AtomicUsize::new(0);
fn invalid() -> io::Error {
    io::Error::other("classic door unavailable")
}
#[derive(Clone, Debug)]
pub struct Game {
    pub slug: String,
    pub title: String,
    pub address: SocketAddr,
}
impl Game {
    pub fn valid(&self) -> bool {
        !self.slug.is_empty()
            && self.slug.len() <= 32
            && self
                .slug
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            && !self.title.is_empty()
            && self.title.len() <= 48
            && self
                .title
                .bytes()
                .all(|b| (32..=126).contains(&b) && b != b'|')
            && self.address.port() != 0
            && !self.address.ip().is_unspecified()
            && !self.address.ip().is_multicast()
            && !matches!(self.address.ip(),std::net::IpAddr::V6(ip) if ip.to_ipv4_mapped().is_some())
    }
}
struct Permit;
impl Permit {
    fn acquire() -> io::Result<Self> {
        ACTIVE
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < 8).then_some(n + 1)
            })
            .map_err(|_| invalid())?;
        Ok(Self)
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        ACTIVE.fetch_sub(1, Ordering::Release);
    }
}
pub struct Connection {
    stream: TcpStream,
    codec: Codec,
    pending: Vec<u8>,
    sent: usize,
    started: Instant,
    connected: bool,
    write_started: Instant,
    window: Instant,
    received: usize,
    total: usize,
    _permit: Permit,
    skip_lf: bool,
}
pub struct Progress {
    pub text: String,
    pub closed: bool,
}
impl Connection {
    pub fn start(game: &Game, ansi: bool, size: (u16, u16)) -> io::Result<Self> {
        if !game.valid() {
            return Err(invalid());
        }
        let permit = Permit::acquire()?;
        let fd = socket(
            if game.address.is_ipv4() {
                AddressFamily::INET
            } else {
                AddressFamily::INET6
            },
            SocketType::STREAM,
            None,
        )?;
        rustix::io::fcntl_setfd(&fd, rustix::io::FdFlags::CLOEXEC)?;
        rustix::fs::fcntl_setfl(&fd, rustix::fs::OFlags::NONBLOCK)?;
        match connect(&fd, &game.address) {
            Ok(()) | Err(rustix::io::Errno::INPROGRESS) => (),
            Err(e) => return Err(e.into()),
        }
        let stream = TcpStream::from(fd);
        stream.set_nodelay(true)?;
        let now = Instant::now();
        Ok(Self {
            stream,
            codec: Codec::new(ansi, size),
            pending: Vec::with_capacity(256),
            sent: 0,
            started: now,
            connected: false,
            write_started: now,
            window: now,
            received: 0,
            total: 0,
            _permit: permit,
            skip_lf: false,
        })
    }
    fn enqueue(&mut self, bytes: &[u8]) -> io::Result<()> {
        if self.pending.len() + bytes.len() > QUEUE {
            return Err(invalid());
        }
        if self.pending.is_empty() {
            self.write_started = Instant::now();
        }
        self.pending.extend_from_slice(bytes);
        Ok(())
    }
    pub fn key(&mut self, byte: u8) -> io::Result<()> {
        // No local echo or automatic credentials. One Enter becomes CR LF;
        // suppress a paired LF from terminals that send CR LF themselves.
        if self.skip_lf {
            self.skip_lf = false;
            if byte == b'\n' {
                return Ok(());
            }
        }
        match byte {
            b'\r' => {
                self.skip_lf = true;
                self.enqueue(b"\r\n")
            }
            b'\n' => self.enqueue(b"\r\n"),
            127 => self.enqueue(&[8]),
            0..=126 => self.enqueue(&[byte]),
            _ => Ok(()),
        }
    }
    pub fn poll(&mut self) -> io::Result<Progress> {
        if !self.connected {
            if self.started.elapsed() > Duration::from_secs(3) {
                return Err(invalid());
            }
            if let Some(e) = self.stream.take_error()? {
                return Err(e);
            }
            if self.stream.peer_addr().is_err() {
                return Ok(Progress {
                    text: String::new(),
                    closed: false,
                });
            }
            self.connected = true;
        }
        if self.sent < self.pending.len() {
            if self.write_started.elapsed() > Duration::from_secs(5) {
                return Err(invalid());
            }
            match self.stream.write(&self.pending[self.sent..]) {
                Ok(0) => return Err(invalid()),
                Ok(n) => self.sent += n,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => (),
                Err(e) => return Err(e),
            }
            if self.sent == self.pending.len() {
                self.pending.fill(0);
                self.pending.clear();
                self.sent = 0;
            }
        }
        let mut bytes = [0; 1024];
        match self.stream.read(&mut bytes) {
            Ok(0) => Ok(Progress {
                text: String::new(),
                closed: true,
            }),
            Ok(n) => {
                if self.window.elapsed() >= Duration::from_secs(1) {
                    self.window = Instant::now();
                    self.received = 0;
                }
                self.received += n;
                self.total += n;
                if self.received > 65536 || self.total > 4 * 1024 * 1024 {
                    return Err(invalid());
                }
                let (text, reply) = self.codec.feed(&bytes[..n])?;
                self.enqueue(&reply)?;
                Ok(Progress {
                    text,
                    closed: false,
                })
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(Progress {
                text: String::new(),
                closed: false,
            }),
            Err(e) => Err(e),
        }
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        self.pending.fill(0);
    }
}
#[derive(Clone, Copy)]
enum Telnet {
    Data,
    Iac,
    Command(u8),
    Sub,
    SubIac,
}
#[derive(Clone, Copy)]
enum Escape {
    Text,
    Esc,
    Csi,
    String,
    StringEsc,
}
struct Codec {
    telnet: Telnet,
    escape: Escape,
    sub: Vec<u8>,
    csi: Vec<u8>,
    discarded: usize,
    local: [bool; 256],
    remote: [bool; 256],
    ansi: bool,
    size: (u16, u16),
}
impl Codec {
    fn new(ansi: bool, size: (u16, u16)) -> Self {
        Self {
            telnet: Telnet::Data,
            escape: Escape::Text,
            sub: Vec::with_capacity(32),
            csi: Vec::with_capacity(32),
            discarded: 0,
            local: [false; 256],
            remote: [false; 256],
            ansi,
            size: (size.0.clamp(40, 240), size.1.clamp(18, 100)),
        }
    }
    fn feed(&mut self, bytes: &[u8]) -> io::Result<(String, Vec<u8>)> {
        let mut out = String::new();
        let mut reply = Vec::new();
        for &byte in bytes {
            match self.telnet {
                Telnet::Data if byte == IAC => self.telnet = Telnet::Iac,
                Telnet::Data => self.display(byte, &mut out)?,
                Telnet::Iac => match byte {
                    251..=254 => self.telnet = Telnet::Command(byte),
                    250 => {
                        self.sub.clear();
                        self.telnet = Telnet::Sub;
                    }
                    IAC => {
                        self.display(IAC, &mut out)?;
                        self.telnet = Telnet::Data;
                    }
                    _ => self.telnet = Telnet::Data,
                },
                Telnet::Command(command) => {
                    let i = usize::from(byte);
                    match command {
                        251 => {
                            let accept = matches!(byte, 1 | 3);
                            if !self.remote[i] || !accept {
                                reply.extend([IAC, if accept { 253 } else { 254 }, byte]);
                            }
                            self.remote[i] = accept;
                        }
                        252 => self.remote[i] = false,
                        253 => {
                            let accept = matches!(byte, 3 | 24 | 31);
                            if !self.local[i] || !accept {
                                reply.extend([IAC, if accept { 251 } else { 252 }, byte]);
                                if byte == 31 {
                                    let (w, h) = self.size;
                                    reply.extend([IAC, 250, 31, 0, w as u8, 0, h as u8, IAC, 240]);
                                }
                            }
                            self.local[i] = accept;
                        }
                        254 => {
                            if self.local[i] {
                                reply.extend([IAC, 252, byte]);
                            }
                            self.local[i] = false;
                        }
                        _ => unreachable!(),
                    }
                    self.telnet = Telnet::Data;
                }
                Telnet::Sub => {
                    if byte == IAC {
                        self.telnet = Telnet::SubIac;
                    } else {
                        self.sub.push(byte);
                    }
                }
                Telnet::SubIac => match byte {
                    240 => {
                        if self.sub == [24, 1] && self.local[24] {
                            reply.extend_from_slice(if self.ansi {
                                b"\xff\xfa\x18\x00ANSI\xff\xf0"
                            } else {
                                b"\xff\xfa\x18\x00DUMB\xff\xf0"
                            });
                        }
                        self.sub.clear();
                        self.telnet = Telnet::Data;
                    }
                    IAC => {
                        self.sub.push(IAC);
                        self.telnet = Telnet::Sub;
                    }
                    _ => return Err(invalid()),
                },
            }
            if self.sub.len() > 64 || reply.len() > QUEUE {
                return Err(invalid());
            }
        }
        Ok((out, reply))
    }
    fn display(&mut self, byte: u8, out: &mut String) -> io::Result<()> {
        match self.escape {
            Escape::Text => match byte {
                27 => self.escape = Escape::Esc,
                b'\r' | b'\n' | b'\t' | 8 => out.push(char::from(byte)),
                32..=126 => out.push(char::from(byte)),
                128..=255 => out.push(
                    CP437
                        .chars()
                        .nth(usize::from(byte) - 128)
                        .expect("complete mapping"),
                ),
                _ => (),
            },
            Escape::Esc => match byte {
                b'[' => {
                    self.csi.clear();
                    self.escape = Escape::Csi;
                }
                b']' | b'P' | b'_' | b'^' | b'X' => {
                    self.discarded = 0;
                    self.escape = Escape::String;
                }
                b'7' | b'8' => {
                    if self.ansi {
                        out.push_str(if byte == b'7' { "\x1b7" } else { "\x1b8" });
                    }
                    self.escape = Escape::Text;
                }
                // Drop the complete unsupported escape, including intermediate bytes.
                0x20..=0x2f => (),
                _ => self.escape = Escape::Text,
            },
            Escape::Csi => {
                if (0x40..=0x7e).contains(&byte) {
                    if self.ansi && safe_csi(&self.csi, byte) {
                        out.push_str("\x1b[");
                        out.push_str(std::str::from_utf8(&self.csi).map_err(|_| invalid())?);
                        out.push(char::from(byte));
                    }
                    self.csi.clear();
                    self.escape = Escape::Text;
                } else if byte == b'\r' && self.csi == b"!" {
                    // Original LORD ends its legacy RIP probe ESC [ ! with CR.
                    // Discard that query; never send it to the member terminal.
                    self.csi.clear();
                    self.escape = Escape::Text;
                    out.push('\r');
                } else if (0x20..=0x3f).contains(&byte) {
                    self.csi.push(byte);
                    if self.csi.len() > 48 {
                        return Err(invalid());
                    }
                } else {
                    return Err(invalid());
                }
            }
            Escape::String | Escape::StringEsc => {
                self.discarded += 1;
                if self.discarded > 1024 {
                    return Err(invalid());
                }
                if byte == 7 || (matches!(self.escape, Escape::StringEsc) && byte == b'\\') {
                    self.escape = Escape::Text;
                } else {
                    self.escape = if byte == 27 {
                        Escape::StringEsc
                    } else {
                        Escape::String
                    };
                }
            }
        }
        Ok(())
    }
}
fn safe_csi(params: &[u8], end: u8) -> bool {
    if params == b"?25" && matches!(end, b'h' | b'l') {
        return true;
    }
    if !params.iter().all(|b| b.is_ascii_digit() || *b == b';') {
        return false;
    }
    let Ok(s) = std::str::from_utf8(params) else {
        return false;
    };
    let numbers: Option<Vec<u16>> = s
        .split(';')
        .map(|v| {
            if v.is_empty() {
                Some(0)
            } else {
                v.parse().ok()
            }
        })
        .collect();
    let Some(n) = numbers else {
        return false;
    };
    if n.len() > 16 {
        return false;
    }
    match end {
        b'm' => n
            .iter()
            .all(|v| matches!(v,0|1|2|4|5|7|22|24|25|27|30..=37|39|40..=47|49|90..=97|100..=107)),
        b'H' | b'f' => n.len() <= 2 && n[0] <= 100 && n.get(1).is_none_or(|v| *v <= 240),
        b'A' | b'B' | b'C' | b'E' | b'F' | b'G' => n.len() == 1 && n[0] <= 240,
        // Classic art uses 255-left to reach the start of an <=240-column line.
        b'D' => n.len() == 1 && n[0] <= 255,
        b'd' => n.len() == 1 && n[0] <= 100,
        b'J' | b'K' => n.len() == 1 && n[0] <= 2,
        b's' | b'u' => params.is_empty(),
        _ => false,
    }
}
const CP437: &str = "ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■ ";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn telnet_fragmentation_negotiates_only_minimal_options_without_identity() {
        let bytes=b"\xff\xfb\x01\xff\xfb\x03\xff\xfd\x18\xff\xfa\x18\x01\xff\xf0\xff\xfd\x1f\xff\xfd\x27\xff\xfb\x2aHi\xff\xff!";
        for split in 0..=bytes.len() {
            let mut codec = Codec::new(true, (80, 24));
            let (a, mut answer) = codec.feed(&bytes[..split]).unwrap();
            let (b, rest) = codec.feed(&bytes[split..]).unwrap();
            answer.extend(rest);
            assert_eq!(a + b.as_str(), "Hi\u{a0}!");
            assert!(
                answer
                    .windows(10)
                    .any(|x| x == b"\xff\xfa\x18\x00ANSI\xff\xf0")
            );
            assert!(
                answer
                    .windows(9)
                    .any(|x| x == b"\xff\xfa\x1f\x00\x50\x00\x18\xff\xf0")
            );
            assert!(answer.windows(3).any(|x| x == b"\xff\xfc\x27")); // NEW-ENVIRON refused
            assert!(answer.windows(3).any(|x| x == b"\xff\xfe\x2a"));
            assert!(!answer.windows(5).any(|x| x == b"sysop"));
        }
        let mut codec = Codec::new(false, (80, 24));
        let (_, answer) = codec.feed(b"\xff\xfd\x18\xff\xfa\x18\x01\xff\xf0").unwrap();
        assert!(
            answer
                .windows(10)
                .any(|x| x == b"\xff\xfa\x18\x00DUMB\xff\xf0")
        );
        let (_, answer) = codec.feed(b"\xff\xfd\x18").unwrap();
        assert!(answer.is_empty());
    }
    #[test]
    fn fragmented_ansi_cp437_and_hidden_controls_are_filtered() {
        let bytes=b"Hello\x1b[1;35m\xb0\xb1\xb2\xdb\x1b[0m\r\n\x1b]52;c;clipboard-secret\x07\x1bPpayload\x1b\\\x1b[6n\x1b[?2004h\x1b[2J\x1b[5;9HOK";
        let expected = "Hello\x1b[1;35m░▒▓█\x1b[0m\r\n\x1b[2J\x1b[5;9HOK";
        for split in 0..=bytes.len() {
            let mut codec = Codec::new(true, (80, 24));
            let (a, _) = codec.feed(&bytes[..split]).unwrap();
            let (b, _) = codec.feed(&bytes[split..]).unwrap();
            assert_eq!(a + b.as_str(), expected);
        }
        let mut codec = Codec::new(false, (80, 24));
        let mut out = String::new();
        for byte in bytes {
            out.push_str(&codec.feed(&[*byte]).unwrap().0);
        }
        assert_eq!(out, "Hello░▒▓█\r\nOK");
        assert_eq!(CP437.chars().count(), 128);
    }
    #[test]
    fn original_lord_rip_probe_is_discarded_and_art_can_return_to_column_one() {
        let bytes = b"\x1b[!\r\n\x1b[255DTitle\x1b[!p\x1b[6n";
        for ansi in [false, true] {
            for split in 0..=bytes.len() {
                let mut codec = Codec::new(ansi, (80, 25));
                let (a, reply) = codec.feed(&bytes[..split]).unwrap();
                let (b, rest) = codec.feed(&bytes[split..]).unwrap();
                assert!(reply.is_empty() && rest.is_empty());
                assert_eq!(
                    a + &b,
                    if ansi {
                        "\r\n\x1b[255DTitle"
                    } else {
                        "\r\nTitle"
                    }
                );
            }
        }
        assert!(!safe_csi(b"256", b'D'));
        assert!(!safe_csi(b"255", b'C'));
        assert!(Codec::new(true, (80, 25)).feed(b"\x1b[12\r").is_err());
    }
    #[test]
    fn protocol_buffers_and_terminal_commands_are_bounded() {
        for prefix in [b"\xff\xfa".as_slice(), b"\x1b[", b"\x1b]"] {
            let mut codec = Codec::new(true, (80, 24));
            codec.feed(prefix).unwrap();
            assert!(codec.feed(&vec![b'0'; 1100]).is_err());
        }
        assert!(!safe_csi(b"99999", b'A'));
        assert!(!safe_csi(b"38;2;1;2;3", b'm'));
        assert!(!safe_csi(b"?1000", b'h'));
        assert!(!safe_csi(b"6", b'n'));
        assert!(safe_csi(b"?25", b'l'));
        assert!(safe_csi(b"0;1;31;40", b'm'));
    }
    #[test]
    fn real_tcp_stream_has_no_automatic_identity_and_releases_all_game_slots() {
        use std::{net::TcpListener, thread};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let game = Game {
            slug: "fixture".into(),
            title: "Owned fixture".into(),
            address: listener.local_addr().unwrap(),
        };
        let worker = thread::spawn(move || {
            let (mut peer, _) = listener.accept().unwrap();
            peer.set_read_timeout(Some(Duration::from_millis(100)))
                .unwrap();
            let mut bytes = [0; 16];
            assert!(
                peer.read(&mut bytes).is_err(),
                "identity was sent before game input"
            );
            peer.write_all(b"\xff\xfd\x18\xff\xfa\x18\x01\xff\xf0\x1b[31mREADY\x1b[0m")
                .unwrap();
            peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            let mut input = Vec::new();
            while !input.ends_with(b"go\r\n") {
                let n = peer.read(&mut bytes).unwrap();
                assert!(n > 0);
                input.extend_from_slice(&bytes[..n]);
                assert!(input.len() < 100);
            }
            assert!(input.windows(4).any(|x| x == b"ANSI"));
            peer.write_all(b"DONE").unwrap();
        });
        let mut c = Connection::start(&game, true, (80, 24)).unwrap();
        let deadline = Instant::now() + Duration::from_secs(4);
        let mut text = String::new();
        let mut sent = false;
        loop {
            assert!(Instant::now() < deadline);
            let progress = c.poll().unwrap();
            text.push_str(&progress.text);
            if text.contains("READY") && !sent {
                for b in b"go\r" {
                    c.key(*b).unwrap();
                }
                sent = true;
            }
            if progress.closed {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        assert!(text.contains("DONE"));
        drop(c);
        worker.join().unwrap();
        let slots: Vec<_> = (0..8).map(|_| Permit::acquire().unwrap()).collect();
        assert!(Permit::acquire().is_err());
        drop(slots);
        assert!(Permit::acquire().is_ok());
    }
}
