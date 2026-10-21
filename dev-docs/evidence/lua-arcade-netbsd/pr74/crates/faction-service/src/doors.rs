//! Bounded nonblocking requests to the separate game UID. No member authority RPC.
use rustix::net::{AddressFamily, SocketAddrUnix, SocketType, connect, socket};
use std::{
    io::{self, Read, Write},
    net::Shutdown,
    os::{fd::AsFd, unix::net::UnixStream},
    path::Path,
    time::{Duration, Instant},
};

pub const SOCKET: &str = "/var/run/faction-doors/door.sock";
const LIMIT: usize = 4096;
fn invalid() -> io::Error {
    io::Error::other("door unavailable or invalid response")
}
fn field(s: &str, max: usize) -> bool {
    s.len() <= max && s.bytes().all(|b| (32..=126).contains(&b) && b != b'|')
}
fn slug(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
#[derive(Clone, Debug)]
pub enum Target {
    Back,
    Lobby,
    Game(String),
    Input(String),
}
#[derive(Clone, Debug)]
pub struct Choice {
    pub key: u8,
    pub label: String,
    pub target: Target,
}
#[derive(Clone, Debug)]
pub struct Screen {
    pub title: String,
    pub text: Vec<String>,
    pub prompt: Option<String>,
    pub choices: Vec<Choice>,
    pub state: String,
}
impl Screen {
    fn parse(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() > LIMIT {
            return Err(invalid());
        }
        let text = std::str::from_utf8(bytes).map_err(|_| invalid())?;
        let text = text
            .strip_prefix("FDS1\n")
            .and_then(|s| s.strip_suffix("END\n"))
            .ok_or_else(invalid)?;
        let (mut title, mut state, mut prompt) = (None, None, None);
        let (mut lines, mut choices) = (Vec::new(), Vec::<Choice>::new());
        for line in text.lines() {
            let (kind, body) = line.split_once('|').ok_or_else(invalid)?;
            match kind {
                "TITLE" if title.is_none() && !body.is_empty() && field(body, 64) => {
                    title = Some(body.into())
                }
                "STATE" if state.is_none() && field(body, 1024) => state = Some(body.into()),
                "TEXT" if lines.len() < 16 && field(body, 120) => lines.push(body.into()),
                "PROMPT" if prompt.is_none() && !body.is_empty() && field(body, 120) => {
                    prompt = Some(body.into())
                }
                "OPTION" if choices.len() < 8 => {
                    let fields: Vec<_> = body.split('|').collect();
                    if fields.len() != 3
                        || fields[0].len() != 1
                        || !fields[0].as_bytes()[0].is_ascii_uppercase()
                        || fields[1].is_empty()
                        || !field(fields[1], 48)
                        || choices.iter().any(|c| c.key == fields[0].as_bytes()[0])
                    {
                        return Err(invalid());
                    }
                    let target = match fields[2] {
                        "back" => Target::Back,
                        "lobby" => Target::Lobby,
                        s if s.strip_prefix("game:").is_some_and(slug) => {
                            Target::Game(s[5..].into())
                        }
                        s if s.starts_with("input:") && field(&s[6..], 64) => {
                            Target::Input(s[6..].into())
                        }
                        _ => return Err(invalid()),
                    };
                    choices.push(Choice {
                        key: fields[0].as_bytes()[0],
                        label: fields[1].into(),
                        target,
                    });
                }
                _ => return Err(invalid()),
            }
        }
        if (prompt.is_some() && !choices.is_empty()) || (prompt.is_none() && choices.is_empty()) {
            return Err(invalid());
        }
        Ok(Self {
            title: title.ok_or_else(invalid)?,
            state: state.ok_or_else(invalid)?,
            prompt,
            text: lines,
            choices,
        })
    }
}
pub struct Request {
    stream: UnixStream,
    request: Vec<u8>,
    sent: usize,
    response: Vec<u8>,
    deadline: Instant,
    uid: u32,
    verified: bool,
}
impl Request {
    pub fn start(uid: u32, game: &str, state: &str, input: &str) -> io::Result<Self> {
        Self::at(Path::new(SOCKET), uid, game, state, input)
    }
    fn at(path: &Path, uid: u32, game: &str, state: &str, input: &str) -> io::Result<Self> {
        if uid == 0 || !slug(game) || !field(state, 1024) || !field(input, 128) {
            return Err(invalid());
        }
        let fd = socket(AddressFamily::UNIX, SocketType::STREAM, None)?;
        rustix::io::fcntl_setfd(&fd, rustix::io::FdFlags::CLOEXEC)?;
        rustix::fs::fcntl_setfl(&fd, rustix::fs::OFlags::NONBLOCK)?;
        let address = SocketAddrUnix::new(path)?;
        match connect(&fd, &address) {
            Ok(()) | Err(rustix::io::Errno::INPROGRESS) => (),
            Err(e) => return Err(e.into()),
        }
        Ok(Self {
            stream: UnixStream::from(fd),
            request: format!("{game}|{state}|{input}\n").into_bytes(),
            sent: 0,
            response: Vec::with_capacity(1024),
            deadline: Instant::now() + Duration::from_secs(2),
            uid,
            verified: false,
        })
    }
    pub fn poll(&mut self) -> io::Result<Option<Screen>> {
        if Instant::now() >= self.deadline {
            return Err(invalid());
        }
        if !self.verified {
            match faction_unix::peer_uid(self.stream.as_fd()) {
                Ok(uid) if uid == self.uid => self.verified = true,
                Err(e) if e.kind() == io::ErrorKind::NotConnected => return Ok(None),
                _ => return Err(invalid()),
            }
        }
        if self.sent < self.request.len() {
            match self.stream.write(&self.request[self.sent..]) {
                Ok(0) => return Err(invalid()),
                Ok(n) => {
                    self.sent += n;
                    if self.sent == self.request.len() {
                        self.stream.shutdown(Shutdown::Write)?;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
                Err(e) => return Err(e),
            }
            if self.sent < self.request.len() {
                return Ok(None);
            }
        }
        let mut bytes = [0; 1024];
        // One bounded read per loop; door traffic cannot monopolize the BBS.
        match self.stream.read(&mut bytes) {
            Ok(0) => Screen::parse(&self.response).map(Some),
            Ok(n) => {
                if self.response.len() + n > LIMIT {
                    return Err(invalid());
                }
                self.response.extend_from_slice(&bytes[..n]);
                Ok(None)
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }
}
pub struct Door {
    pub game: String,
    pub screen: Option<Screen>,
    pub focus: usize,
    pub pending: Option<Request>,
}
impl Door {
    pub fn open(uid: u32, game: &str) -> io::Result<Self> {
        Ok(Self {
            game: game.into(),
            screen: None,
            focus: 0,
            pending: Some(Request::start(uid, game, "", "")?),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validate_complete_screen_and_only_allowed_actions() {
        let good = b"FDS1\nTITLE|Arcade\nTEXT|Hello\nOPTION|H|Hunt|game:signal-hunt\nSTATE|\nEND\n";
        let screen = Screen::parse(good).unwrap();
        assert_eq!(screen.choices[0].key, b'H');
        for bad in ["game:../../state", "shell:sh", "approve:me", "game:"] {
            let frame = String::from_utf8(good.to_vec())
                .unwrap()
                .replace("game:signal-hunt", bad);
            assert!(Screen::parse(frame.as_bytes()).is_err());
        }
        for frame in [
            b"FDS1\nTITLE|bad\x1b[2J\nPROMPT|Guess\nSTATE|\nEND\n".as_slice(),
            b"FDS1\nTITLE|x\nTITLE|y\nPROMPT|Guess\nSTATE|\nEND\n",
            b"FDS1\nTITLE|x\nOPTION|Q|Quit|back\nOPTION|Q|Again|back\nSTATE|\nEND\n",
            b"FDS1\nTITLE|x\nPROMPT|Guess\nSTATE|\nEND\nEXTRA\n",
        ] {
            assert!(Screen::parse(frame).is_err());
        }
    }
    #[test]
    fn nonblocking_rpc_rejects_wrong_peer_and_truncated_results() {
        use std::{fs, os::unix::net::UnixListener, thread};
        let path = std::env::temp_dir().join(format!("faction-door-{}", std::process::id()));
        let listener = UnixListener::bind(&path).unwrap();
        let uid = rustix::process::geteuid().as_raw();
        let worker = thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let mut request = String::new();
            s.read_to_string(&mut request).unwrap();
            assert_eq!(request, "doors||\n");
            s.write_all(b"FDS1\nTITLE|incomplete\n").unwrap();
        });
        let mut r = Request::at(&path, uid, "doors", "", "").unwrap();
        loop {
            match r.poll() {
                Ok(None) => thread::sleep(Duration::from_millis(1)),
                Err(_) => break,
                Ok(Some(_)) => panic!("partial result accepted"),
            }
        }
        worker.join().unwrap();
        fs::remove_file(&path).unwrap();
        let _listener = UnixListener::bind(&path).unwrap();
        let mut wrong = Request::at(&path, uid + 1, "doors", "", "").unwrap();
        assert!(wrong.poll().is_err());
        fs::remove_file(&path).unwrap();
    }
}
