use crate::{
    profile::Profile,
    stream::{Event, Filter, Recent},
};
use faction_files::invalid;
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    fs::{OFlags, fcntl_getfl, fcntl_setfl},
    termios,
};
use std::{
    fs::File,
    io::{self, Read, Write},
    os::fd::AsFd,
    process::{Child, Stdio},
    time::{Duration, Instant},
};
pub fn nonblock(f: impl AsFd) -> io::Result<()> {
    fcntl_setfl(&f, fcntl_getfl(&f)? | OFlags::NONBLOCK).map_err(Into::into)
}
pub fn wait(f: impl AsFd, event: PollFlags, deadline: Instant) -> io::Result<()> {
    let d = deadline
        .checked_duration_since(Instant::now())
        .ok_or_else(|| io::Error::from(io::ErrorKind::TimedOut))?;
    let mut p = [PollFd::new(&f, event)];
    let ts = Timespec {
        tv_sec: d.as_secs() as _,
        tv_nsec: i64::from(d.subsec_nanos()),
    };
    if poll(&mut p, Some(&ts))? == 0 || !p[0].revents().intersects(event | PollFlags::HUP) {
        return Err(invalid());
    }
    Ok(())
}
pub fn write_all(f: &mut (impl Write + AsFd), b: &[u8], deadline: Instant) -> io::Result<()> {
    let mut n = 0;
    while n < b.len() {
        wait(&*f, PollFlags::OUT, deadline)?;
        match f.write(&b[n..]) {
            Ok(0) => return Err(invalid()),
            Ok(k) => n += k,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
pub fn read_all(f: &mut (impl Read + AsFd), b: &mut [u8], deadline: Instant) -> io::Result<()> {
    let mut n = 0;
    while n < b.len() {
        wait(&*f, PollFlags::IN, deadline)?;
        match f.read(&mut b[n..]) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(k) => n += k,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
pub struct Remote {
    pub child: Child,
    pub pty: File,
    pub filter: Filter,
    pub nonce: String,
    pub sequence: u32,
    pub namespace: String,
    pub native: bool,
    pub frames: Vec<Vec<String>>,
    pub mail_frames: Vec<faction_terminal::home_mail::Frame>,
    pub output: Vec<u8>,
    pub recent: Recent,
}
impl Drop for Remote {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Remote {
    pub fn start(p: &Profile, rows: u16, cols: u16, compact: bool) -> io::Result<Self> {
        let (master, slave) = faction_client_pty::open(rows, cols)?;
        let mut t = termios::tcgetattr(&slave)?;
        t.make_raw();
        termios::tcsetattr(&slave, termios::OptionalActions::Now, &t)?;
        let slave = File::from(slave);
        let child = p
            .ssh(false)
            .env(
                "TERM",
                if compact {
                    "faction-compact"
                } else {
                    "xterm-256color"
                },
            )
            .stdin(Stdio::from(slave.try_clone()?))
            .stdout(Stdio::from(slave.try_clone()?))
            .stderr(Stdio::from(slave))
            .spawn()?;
        let pty = File::from(master);
        nonblock(&pty)?;
        let mut seed = [0; 16];
        getrandom::fill(&mut seed).map_err(|_| invalid())?;
        let nonce = seed.iter().map(|b| format!("{b:02x}")).collect();
        Ok(Self {
            child,
            pty,
            filter: Filter::default(),
            nonce,
            sequence: 0,
            namespace: String::new(),
            native: false,
            frames: Vec::new(),
            mail_frames: Vec::new(),
            output: Vec::new(),
            recent: Recent::default(),
        })
    }
    pub fn send(&mut self, b: &[u8]) -> io::Result<()> {
        write_all(&mut self.pty, b, Instant::now() + Duration::from_secs(3))
    }
    pub fn pump(&mut self) -> io::Result<()> {
        let mut b = [0; 4096];
        let n = match self.pty.read(&mut b) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(e),
        };
        self.accept(&b[..n])
    }
    fn accept(&mut self, bytes: &[u8]) -> io::Result<()> {
        // Parse the entire read before applying any event: malformed trailing
        // controls cannot be hidden by an otherwise valid acknowledgement.
        let events = self.filter.feed(bytes)?;
        let text_bytes: usize = events
            .iter()
            .map(|event| match event {
                Event::Text(text) => text.len(),
                _ => 0,
            })
            .sum();
        let frames = events
            .iter()
            .filter(|event| !matches!(event, Event::Text(_)))
            .count();
        if self.output.len() + text_bytes > 32768
            || self.frames.len() + self.mail_frames.len() + frames > 24
        {
            return Err(invalid());
        }
        for event in events {
            match event {
                Event::Text(t) => {
                    self.recent.feed(&t);
                    if self.output.len() + t.len() > 32768 {
                        return Err(invalid());
                    }
                    self.output.extend(t);
                }
                Event::Frame(f) => self.frames.push(f),
                Event::Mail(frame) => {
                    // Only output following the acknowledgement may resume.
                    self.output.clear();
                    self.mail_frames.push(frame);
                }
            }
        }
        Ok(())
    }
    pub fn reset_mail_boundary(&mut self) {
        self.output.clear();
        self.filter.reset_boundary();
    }
    pub fn mail_exchange(
        &mut self,
        request: &faction_terminal::home_mail::Request,
    ) -> io::Result<()> {
        use faction_terminal::home_mail::{self, Frame, Request};
        if self.native || !self.frames.is_empty() || !self.mail_frames.is_empty() {
            return Err(invalid());
        }
        self.reset_mail_boundary();
        let mut line = home_mail::encode_request(request).map_err(|_| invalid())?;
        line.push('\r');
        self.send(line.as_bytes())?;
        let expected = match request {
            Request::Park(nonce) => Frame::Parked(nonce.clone()),
            Request::Back(nonce) => Frame::Returned(nonce.clone()),
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            wait(&self.pty, PollFlags::IN, deadline)?;
            self.pump()?;
            if !self.frames.is_empty() {
                return Err(invalid());
            }
            if !self.mail_frames.is_empty() {
                if self.mail_frames != [expected] {
                    return Err(invalid());
                }
                self.mail_frames.clear();
                if matches!(request, Request::Park(_)) {
                    self.reset_mail_boundary();
                }
                return Ok(());
            }
        }
    }
    pub fn rpc(&mut self, fields: &[&str]) -> io::Result<Vec<Vec<String>>> {
        let result = self.rpc_inner(fields);
        if result
            .as_ref()
            .is_err_and(|e| e.kind() != io::ErrorKind::PermissionDenied)
        {
            self.native = false;
        }
        result
    }
    fn rpc_inner(&mut self, fields: &[&str]) -> io::Result<Vec<Vec<String>>> {
        self.frames.clear();
        self.output.clear();
        self.sequence = self.sequence.checked_add(1).ok_or_else(invalid)?;
        let seq = self.sequence.to_string();
        self.send(faction_files::native::command(&self.nonce, self.sequence, fields)?.as_bytes())?;
        let returning = fields == ["back"];
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut rows = Vec::new();
        let mut denied = false;
        loop {
            wait(&self.pty, PollFlags::IN, deadline)?;
            self.pump()?;
            if !self.mail_frames.is_empty() {
                return Err(invalid());
            }
            if !returning {
                self.output.clear();
            }
            for f in std::mem::take(&mut self.frames) {
                if f.len() < 3 || f[1] != self.nonce || f[2] != seq {
                    return Err(invalid());
                }
                match f[0].as_str() {
                    "row" if f.len() >= 4 && rows.len() < 20 => rows.push(f[3..].to_vec()),
                    "error" => denied = true,
                    "end" => {
                        return if denied {
                            Err(io::Error::new(
                                io::ErrorKind::PermissionDenied,
                                "file operation denied",
                            ))
                        } else {
                            Ok(rows)
                        };
                    }
                    "bye" => {
                        self.native = false;
                        return Ok(rows);
                    }
                    _ => return Err(invalid()),
                }
            }
        }
    }
}

#[cfg(test)]
mod home_mail_tests {
    use super::*;
    use faction_terminal::home_mail::{self, Frame, Request};
    fn remote() -> (Remote, File) {
        let (master, slave) = faction_client_pty::open(24, 80).unwrap();
        let mut mode = termios::tcgetattr(&slave).unwrap();
        mode.make_raw();
        termios::tcsetattr(&slave, termios::OptionalActions::Now, &mode).unwrap();
        let pty = File::from(master);
        nonblock(&pty).unwrap();
        (
            Remote {
                child: std::process::Command::new("true").spawn().unwrap(),
                pty,
                filter: Filter::default(),
                nonce: String::new(),
                sequence: 0,
                namespace: String::new(),
                native: false,
                frames: Vec::new(),
                mail_frames: Vec::new(),
                output: Vec::new(),
                recent: Recent::default(),
            },
            File::from(slave),
        )
    }
    fn frame(frame: Frame) -> String {
        home_mail::encode_frame(&frame).unwrap()
    }
    #[test]
    fn mail_return_discards_pre_ack_text_and_keeps_post_ack_fifo() {
        let (mut remote, _slave) = remote();
        remote.accept(b"old buffered text").unwrap();
        let input = format!(
            "hostile pre-ack{}same menu",
            frame(Frame::Returned("a".repeat(32)))
        );
        remote.accept(input.as_bytes()).unwrap();
        assert_eq!(remote.output, b"same menu");
        assert_eq!(remote.mail_frames, [Frame::Returned("a".repeat(32))]);
        assert!(!remote.native);
        assert!(remote.namespace.is_empty());
    }
    #[test]
    fn mail_exchange_rejects_wrong_duplicate_and_trailing_controls() {
        for suffix in [
            frame(Frame::Returned("b".repeat(32))),
            format!(
                "{}{}",
                frame(Frame::Returned("a".repeat(32))),
                frame(Frame::Offer)
            ),
            format!(
                "{}\x1b]7770;FACTION-MAIL1;returned;bad\x07",
                frame(Frame::Returned("a".repeat(32)))
            ),
            format!(
                "{}{}",
                frame(Frame::Returned("a".repeat(32))),
                faction_files::native::frame(&["ready"]).unwrap()
            ),
        ] {
            let (mut remote, mut slave) = remote();
            let peer = std::thread::spawn(move || {
                let mut request = [0; 45];
                slave.read_exact(&mut request).unwrap();
                assert_eq!(
                    request.as_slice(),
                    format!("@MAIL1 back {}\r", "a".repeat(32)).as_bytes()
                );
                slave.write_all(suffix.as_bytes()).unwrap();
                std::thread::sleep(Duration::from_millis(30));
            });
            assert!(
                remote
                    .mail_exchange(&Request::Back("a".repeat(32)))
                    .is_err()
            );
            peer.join().unwrap();
        }
    }
    #[test]
    fn fragmented_mail_ack_returns_on_same_child_without_file_authority() {
        let (mut remote, mut slave) = remote();
        let child = remote.child.id();
        let peer = std::thread::spawn(move || {
            let mut request = [0; 45];
            slave.read_exact(&mut request).unwrap();
            assert_eq!(
                request.as_slice(),
                format!("@MAIL1 back {}\r", "a".repeat(32)).as_bytes()
            );
            let wire = format!("untrusted{}MENU", frame(Frame::Returned("a".repeat(32))));
            for byte in wire.bytes() {
                slave.write_all(&[byte]).unwrap();
            }
            std::thread::sleep(Duration::from_millis(30));
        });
        remote
            .mail_exchange(&Request::Back("a".repeat(32)))
            .unwrap();
        assert_eq!(remote.child.id(), child);
        // Later PTY reads belong to the resumed board as well; the ACK need
        // not share one read with the complete menu.
        let deadline = Instant::now() + Duration::from_secs(1);
        while remote.output.len() < 4 {
            wait(&remote.pty, PollFlags::IN, deadline).unwrap();
            remote.pump().unwrap();
        }
        assert_eq!(remote.output, b"MENU");
        assert!(remote.namespace.is_empty());
        assert_eq!(remote.sequence, 0);
        peer.join().unwrap();
    }
    #[test]
    fn mail_and_files_share_existing_frame_and_output_limits() {
        let (mut remote, _slave) = remote();
        let native = faction_files::native::frame(&["ready"]).unwrap();
        remote.accept(native.repeat(23).as_bytes()).unwrap();
        remote.accept(frame(Frame::Offer).as_bytes()).unwrap();
        assert!(remote.accept(frame(Frame::Offer).as_bytes()).is_err());
        let (mut remote, _slave) = self::remote();
        remote.accept(&vec![b'x'; 32768]).unwrap();
        assert!(
            remote
                .accept(format!("x{}", frame(Frame::Offer)).as_bytes())
                .is_err()
        );
    }
}
