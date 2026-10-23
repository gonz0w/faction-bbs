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
        for event in self.filter.feed(&b[..n])? {
            match event {
                Event::Text(t) => {
                    self.recent.feed(&t);
                    if self.output.len() + t.len() > 32768 {
                        return Err(invalid());
                    }
                    self.output.extend(t);
                }
                Event::Frame(f) => {
                    if self.frames.len() >= 24 {
                        return Err(invalid());
                    }
                    self.frames.push(f);
                }
            }
        }
        Ok(())
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
