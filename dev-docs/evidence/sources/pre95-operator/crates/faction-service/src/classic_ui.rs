//! Local menu/confirmation and escape around a separately hosted original game.
use super::*;
impl Client {
    fn classic_screen(
        &mut self,
        title: &str,
        text: Vec<String>,
        choices: Vec<doors::Choice>,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.door = Some(doors::Door {
            game: "doors".into(),
            screen: Some(doors::Screen {
                title: title.into(),
                text,
                prompt: None,
                choices,
                state: String::new(),
                score: None,
            }),
            focus: 0,
            pending: None,
            run: None,
            turns: 0,
            score_saved: false,
            resume: None,
            notice: None,
        });
        self.door_show(theme)
    }
    pub(super) fn classic_list(
        &mut self,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        let games = self.classic_games.as_ref().ok_or(Error::Denied)?;
        let mut choices = Vec::new();
        for (i, g) in games.iter().enumerate() {
            choices.push(doors::Choice {
                key: b'A' + i as u8,
                label: g.title.clone(),
                target: doors::Target::Server(g.slug.clone()),
            });
        }
        if self.doors_uid.is_some() {
            choices.push(doors::Choice {
                key: b'L',
                label: "Local Lua arcade".into(),
                target: doors::Target::Lobby,
            });
        }
        choices.push(doors::Choice {
            key: b'Q',
            label: "Back to the BBS".into(),
            target: doors::Target::Back,
        });
        self.classic_screen(
            "Classic game servers",
            vec![
                "Original games on sysop-selected servers. Choose one to review.".into(),
                "No BBS credentials or account details are sent automatically.".into(),
            ],
            choices,
            theme,
        )
    }
    fn classic_game(&self, slug: &str) -> faction_core::Result<classic::Game> {
        self.classic_games
            .as_ref()
            .and_then(|g| g.iter().find(|g| g.slug == slug))
            .cloned()
            .ok_or(Error::Denied)
    }
    pub(super) fn classic_confirm(
        &mut self,
        slug: &str,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        let game = self.classic_game(slug)?;
        self.classic_screen(
            &game.title,
            vec![
                format!("Game server: {}", game.address),
                "You are leaving the local arcade for this selected game server.".into(),
                "Legacy Telnet hop: unencrypted. Use your sysop's protected path.".into(),
                "The game has its own login. Never reuse your BBS password.".into(),
                "Nothing is sent until you connect. Ctrl-] returns to Faction.".into(),
            ],
            vec![
                doors::Choice {
                    key: b'C',
                    label: "Connect to this game server".into(),
                    target: doors::Target::Connect(game.slug),
                },
                doors::Choice {
                    key: b'L',
                    label: "Classic server list".into(),
                    target: doors::Target::Classics,
                },
                doors::Choice {
                    key: b'B',
                    label: "Back to the BBS".into(),
                    target: doors::Target::Back,
                },
            ],
            theme,
        )
    }
    pub(super) fn classic_connect(
        &mut self,
        slug: &str,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(board)?;
        let game = self.classic_game(slug)?;
        match classic::Connection::start(&game, self.ansi, self.size()) {
            Ok(connection) => {
                self.door = None;
                self.classic = Some(connection);
                self.state = State::Classic;
                self.input.zeroize();
                self.input.clear();
                self.decoder = Decoder::new();
                self.skip_lf = false;
                self.prompt_started = Instant::now();
                if self.ansi {
                    self.queue("\x1b[0m\x1b[?25h\x1b[2J\x1b[H");
                }
                self.queue("Connecting to classic game server. Ctrl-] returns to Faction.\n");
            }
            Err(_) => self.classic_return(
                "Classic door unavailable or all eight game slots are busy.",
                theme,
                board,
            )?,
        }
        Ok(())
    }
    pub(super) fn classic_return(
        &mut self,
        message: &str,
        theme: &Theme,
        board: &mut Board,
    ) -> faction_core::Result<()> {
        self.classic = None;
        self.door = None;
        self.input.zeroize();
        self.input.clear();
        self.decoder = Decoder::new();
        self.skip_lf = false;
        let _ = termios::tcflush(&self.tty, termios::QueueSelector::IFlush);
        self.menu(theme, board)?;
        self.queue(&format!("\n{message}\n"));
        Ok(())
    }
    pub(super) fn classic_poll(
        &mut self,
        theme: &Theme,
        board: &mut Board,
    ) -> faction_core::Result<()> {
        // Called only with empty output. Every sweep is bounded; no slow peer
        // retains a transaction, and idle remote sessions still lose approval.
        self.approved(board)?;
        match self.classic.as_mut().ok_or(Error::Denied)?.poll() {
            Ok(progress) => {
                if !progress.text.is_empty() {
                    self.queue(&progress.text);
                }
                if progress.closed {
                    self.classic_return("Game server disconnected. Welcome back.", theme, board)?;
                }
            }
            Err(_) => self.classic_return(
                "Classic door unavailable. The BBS is still here.",
                theme,
                board,
            )?,
        }
        Ok(())
    }
}
