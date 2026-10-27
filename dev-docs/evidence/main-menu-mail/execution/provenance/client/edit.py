from pathlib import Path
p=Path('crates/faction-client/src/stream.rs');s=p.read_text();s=s.replace('    Frame(Vec<String>),','    Frame(Vec<String>),\n    Mail(faction_terminal::home_mail::Frame),');s=s.replace('    fn printable(','''    pub fn reset_boundary(&mut self) {
        self.state = State::Ground;
        self.utf8.clear();
    }
    fn printable(''');needle='''                            out.push(Event::Frame(frame));
                        }
''';s=s.replace(needle,'''                            out.push(Event::Frame(frame));
                        } else if let Some(payload) = v.strip_prefix(faction_terminal::home_mail::PREFIX.as_bytes()) {
                            // MAIL1 requires the frozen BEL terminator, unlike Files OSC.
                            if b != 7 || esc {
                                return Err(invalid());
                            }
                            let payload = std::str::from_utf8(payload).map_err(|_| invalid())?;
                            let frame = faction_terminal::home_mail::parse_frame(payload).map_err(|_| invalid())?;
                            if !text.is_empty() {
                                out.push(Event::Text(std::mem::take(&mut text)));
                            }
                            out.push(Event::Mail(frame));
                        }
''');s=s.replace('                    Event::Frame(_) => panic!("unexpected local action"),','                    Event::Frame(_) | Event::Mail(_) => panic!("unexpected local action"),');s=s.replace('                    Event::Frame(f) => frames.push(f),','                    Event::Frame(f) => frames.push(f),\n                    Event::Mail(_) => panic!("unexpected mail action"),');p.write_text(s)
p=Path('crates/faction-client/src/transport.rs');s=p.read_text();s=s.replace('    pub frames: Vec<Vec<String>>,','    pub frames: Vec<Vec<String>>,\n    pub mail_frames: Vec<faction_terminal::home_mail::Frame>,');s=s.replace('            frames: Vec::new(),','            frames: Vec::new(),\n            mail_frames: Vec::new(),');s=s.replace('''        for event in self.filter.feed(&b[..n])? {
''','''        self.accept(&b[..n])
    }
    fn accept(&mut self, bytes: &[u8]) -> io::Result<()> {
        // Parse the entire read before applying any event: malformed trailing
        // controls cannot be hidden by an otherwise valid acknowledgement.
        let events = self.filter.feed(bytes)?;
        let text_bytes: usize = events.iter().map(|event| match event {
            Event::Text(text) => text.len(),
            _ => 0,
        }).sum();
        let frames = events.iter().filter(|event| !matches!(event, Event::Text(_))).count();
        if self.output.len() + text_bytes > 32768 || self.frames.len() + self.mail_frames.len() + frames > 24 {
            return Err(invalid());
        }
        for event in events {
''');s=s.replace('''                Event::Frame(f) => {
                    if self.frames.len() >= 24 {
                        return Err(invalid());
                    }
                    self.frames.push(f);
                }
''','''                Event::Frame(f) => self.frames.push(f),
                Event::Mail(frame) => {
                    // Only output following the acknowledgement may resume.
                    self.output.clear();
                    self.mail_frames.push(frame);
                }
''');needle='''    pub fn rpc(&mut self, fields: &[&str])''';s=s.replace(needle,'''    pub fn reset_mail_boundary(&mut self) {
        self.output.clear();
        self.filter.reset_boundary();
    }
    pub fn mail_exchange(&mut self, request: &faction_terminal::home_mail::Request) -> io::Result<()> {
        use faction_terminal::home_mail::{self, Frame, Request};
        if self.native || !self.frames.is_empty() || !self.mail_frames.is_empty() {
            return Err(invalid());
        }
        self.reset_mail_boundary();
        let mut line = home_mail::encode_request(request).map_err(|_| invalid())?;
        line.push('\\r');
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
    pub fn rpc(&mut self, fields: &[&str])''');s=s.replace('''            self.pump()?;
            if !returning''','''            self.pump()?;
            if !self.mail_frames.is_empty() {
                return Err(invalid());
            }
            if !returning''');p.write_text(s)
