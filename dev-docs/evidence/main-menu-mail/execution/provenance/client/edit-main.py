from pathlib import Path
p=Path('crates/faction-client/src/main.rs');s=p.read_text();s=s.replace('    fn entries(mail:', '''    fn mail_entries() -> [(u8, &'static str); 2] {
        [(b'O', "Open encrypted home mail"), (b'B', "Back to the BBS")]
    }
    fn mail_choice(&mut self) -> io::Result<u8> {
        let entries = Self::mail_entries();
        let mut focus = 1; // A bare Enter returns safely.
        let mut decoder = Decoder::new();
        loop {
            self.print("\\x1b[2J\\x1b[HLOCAL / ENCRYPTED HOME MAIL\\r\\nArrows / Enter or hotkeys.\\r\\n")?;
            for (i, (key, label)) in entries.iter().enumerate() {
                self.print(&format!("{} [{}] {}\\r\\n", if focus == i { ">" } else { " " }, *key as char, label))?;
            }
            loop {
                if !self.ready(100)? {
                    decoder.expire(Instant::now());
                    continue;
                }
                if let Some(key) = decoder.feed(self.byte()?, Instant::now()) {
                    match key {
                        Key::Enter => return Ok(entries[focus].0),
                        Key::Hotkey(key) if entries.iter().any(|(k, _)| *k == key.to_ascii_uppercase()) => return Ok(key.to_ascii_uppercase()),
                        Key::Cancel => return Ok(b'B'),
                        Key::Up | Key::Down | Key::Left | Key::Right | Key::Next => { focus = 1 - focus; break; }
                        _ => (),
                    }
                }
            }
        }
    }
    fn entries(mail:''');start=s.index('    if t.graphics {',s.index('fn parked_mail('));end=s.index('    // Discard unsolicited',start);block=s[start:end];block=block.replace('Enter to return to local Files: ', 'Enter to return to the board: ');s=s[:start]+'''    private_mail_console(t, home, visit)?;
'''+s[end:];insert=s.index('fn run() -> io::Result<()>');s=s[:insert]+'''// This call owns the console lifetime without access to Remote. The vault and
// hidden-input terminal guard are dropped before any caller can pump the board.
#[cfg(feature = "network-mail")]
fn private_mail_console(
    t: &mut Terminal,
    home: &visit_mail::Home<'_>,
    visit: &mut visit_mail::VisitMail,
) -> io::Result<()> {
'''+block+'''    Ok(())
}
fn main_menu_mail(
    t: &mut Terminal,
    r: &mut Remote,
    visit: &mut visit_mail::VisitMail,
    #[cfg(feature = "network-mail")] home: Option<&visit_mail::Home<'_>>,
) -> io::Result<()> {
    use faction_terminal::home_mail::Request;
    r.reset_mail_boundary();
    termios::tcflush(&t.input, termios::QueueSelector::IFlush)?;
    let mut random = [0; 16];
    getrandom::fill(&mut random).map_err(|_| invalid())?;
    let nonce: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
    #[cfg(feature = "network-mail")]
    let configured = home.is_some();
    #[cfg(not(feature = "network-mail"))]
    let configured = false;
    let mut open = false;
    if configured {
        open = t.mail_choice()? == b'O';
        termios::tcflush(&t.input, termios::QueueSelector::IFlush)?;
        if open {
            let answer = t.line("\\r\\nLOCAL: open encrypted home mail while this board stays parked? [y/N]: ", 1);
            termios::tcflush(&t.input, termios::QueueSelector::IFlush)?;
            open = match answer {
                Ok(value) => value.eq_ignore_ascii_case("y"),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => false,
                Err(error) => return Err(error),
            };
        }
    } else {
        t.print("\\x1b[2J\\x1b[HLOCAL / ENCRYPTED HOME MAIL\\r\\n")?;
        t.print(if cfg!(feature = "network-mail") {
            "Home mail is not configured in this local board catalog.\\r\\n"
        } else {
            "Home mail requires a network-mail client build.\\r\\n"
        })?;
        if let Err(error) = t.line("Enter to return to the board: ", 0)
            && error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
    termios::tcflush(&t.input, termios::QueueSelector::IFlush)?;
    if open {
        r.mail_exchange(&Request::Park(nonce.clone()))?;
        #[cfg(feature = "network-mail")]
        private_mail_console(t, home.ok_or_else(invalid)?, visit)?;
    }
    #[cfg(not(feature = "network-mail"))]
    let _ = visit;
    // No console or secret lives across this boundary. Discard typeahead and
    // queued remote text, then validate current board authority before resuming.
    t.print("\\x1b[2J\\x1b[H")?;
    termios::tcflush(&t.input, termios::QueueSelector::IFlush)?;
    r.mail_exchange(&Request::Back(nonce))
}
'''+s[insert:];needle='''            for f in std::mem::take(&mut r.frames) {''';idx=s.index(needle,s.index('fn session('));s=s[:idx]+'''            if !r.mail_frames.is_empty() {
                if r.native || handshake.is_some() || !r.frames.is_empty()
                    || r.mail_frames != [faction_terminal::home_mail::Frame::Offer] {
                    return Err(invalid());
                }
                r.mail_frames.clear();
                local_keys = launcher::Keys::default();
                main_menu_mail(t, &mut r, &mut visit,
                    #[cfg(feature = "network-mail")]
                    home,
                )?;
                // The keyboard readiness predates the local view and is stale.
                continue;
            }
'''+s[idx:];p.write_text(s)
