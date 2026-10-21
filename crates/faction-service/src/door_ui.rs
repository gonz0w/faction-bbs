//! Host-owned rendering/navigation. Lua supplies validated data, never ANSI bytes.
use super::*;
impl Client {
    pub(super) fn door_open(
        &mut self,
        game: &str,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        let Some(uid) = self.doors_uid else {
            self.queue("Doors are not installed.\n");
            self.pause();
            return Ok(());
        };
        match doors::Door::open(uid, game) {
            Ok(door) => {
                self.door = Some(door);
                self.state = State::DoorPending;
                self.prompt_started = Instant::now();
                self.queue("Opening door...\n");
            }
            Err(_) => self.door_failed(theme, board)?,
        }
        Ok(())
    }
    fn door_failed(&mut self, theme: &Theme, board: &mut Board) -> faction_core::Result<()> {
        self.door = None;
        self.menu(theme, board)?;
        self.queue("\nThat door is unavailable. The BBS is still here; try another time.\n");
        Ok(())
    }
    pub(super) fn door_poll(
        &mut self,
        theme: &Theme,
        board: &mut Board,
    ) -> faction_core::Result<()> {
        let result = self
            .door
            .as_mut()
            .and_then(|d| d.pending.as_mut())
            .map(doors::Request::poll);
        match result {
            Some(Ok(Some(screen))) => {
                self.approved(board)?;
                let d = self.door.as_mut().ok_or(Error::Denied)?;
                d.pending = None;
                d.screen = Some(screen);
                d.focus = 0;
                self.door_show(theme)?;
            }
            Some(Err(_)) => self.door_failed(theme, board)?,
            _ => (),
        }
        Ok(())
    }
    fn door_show(&mut self, theme: &Theme) -> faction_core::Result<()> {
        let d = self.door.as_ref().ok_or(Error::Denied)?;
        let s = d.screen.as_ref().ok_or(Error::Denied)?;
        let (normal, focus, hotkey) = theme.menu_styles();
        let mut out = String::new();
        if self.ansi {
            out.push_str(&format!("{}\x1b[?25h\x1b[2J\x1b[H", normal.sgr()));
        }
        out.push_str(&format!("{}\n\n", s.title));
        for line in &s.text {
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
        for (i, c) in s.choices.iter().enumerate() {
            if self.ansi {
                out.push_str(&if i == d.focus { focus } else { normal }.sgr());
            }
            out.push_str(if i == d.focus && self.ansi {
                "> "
            } else {
                "  "
            });
            if self.ansi {
                out.push_str(&hotkey.sgr());
            }
            out.push_str(&format!("[{}] ", c.key as char));
            if self.ansi {
                out.push_str(&if i == d.focus { focus } else { normal }.sgr());
            }
            out.push_str(&c.label);
            out.push('\n');
        }
        let prompt = s.prompt.clone();
        if self.ansi {
            out.push_str("\x1b[0m");
        }
        if let Some(label) = &prompt {
            out.push_str(label);
        } else if self.ansi {
            out.push_str("Arrows / Enter or hotkey. Ctrl-C returns to the BBS.\n");
        } else {
            out.push_str("Choice (then Enter): ");
        }
        self.state = if prompt.is_some() {
            State::DoorInput
        } else {
            State::DoorMenu
        };
        self.input.zeroize();
        self.input.clear();
        self.decoder = Decoder::new();
        self.prompt_started = Instant::now();
        self.queue(&out);
        Ok(())
    }
    pub(super) fn door_key(
        &mut self,
        key: Key,
        theme: &Theme,
        board: &mut Board,
    ) -> faction_core::Result<bool> {
        self.approved(board)?;
        let d = self.door.as_mut().ok_or(Error::Denied)?;
        let screen = d.screen.as_ref().ok_or(Error::Denied)?;
        let n = screen.choices.len();
        if n == 0 {
            return Err(Error::Denied);
        }
        let target = match key {
            Key::Up => {
                d.focus = (d.focus + n - 1) % n;
                None
            }
            Key::Down => {
                d.focus = (d.focus + 1) % n;
                None
            }
            Key::Enter => Some(screen.choices[d.focus].target.clone()),
            Key::Hotkey(key) => screen
                .choices
                .iter()
                .find(|c| c.key == key)
                .map(|c| c.target.clone()),
            Key::Cancel => Some(doors::Target::Back),
        };
        if let Some(target) = target {
            match target {
                doors::Target::Back => {
                    self.door = None;
                    self.menu(theme, board)?;
                }
                doors::Target::Lobby => self.door_open("doors", board, theme)?,
                doors::Target::Game(game) => self.door_open(&game, board, theme)?,
                doors::Target::Input(input) => self.door_submit(&input, board, theme)?,
            }
            Ok(true)
        } else {
            self.door_show(theme)?;
            Ok(false)
        }
    }
    pub(super) fn door_submit(
        &mut self,
        input: &str,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        let uid = self.doors_uid.ok_or(Error::Denied)?;
        let d = self.door.as_mut().ok_or(Error::Denied)?;
        let state = &d.screen.as_ref().ok_or(Error::Denied)?.state;
        match doors::Request::start(uid, &d.game, state, input) {
            Ok(request) => {
                d.pending = Some(request);
                self.state = State::DoorPending;
                self.prompt_started = Instant::now();
                self.queue("\n");
            }
            Err(_) => self.door_failed(theme, board)?,
        }
        Ok(())
    }
}
