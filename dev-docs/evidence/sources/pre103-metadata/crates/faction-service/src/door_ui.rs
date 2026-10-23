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
            if self.classic_games.as_ref().is_some_and(|g| !g.is_empty()) {
                return self.classic_list(board, theme);
            }
            self.queue("Doors are not installed.\n");
            self.pause();
            return Ok(());
        };
        match doors::Door::open(uid, game) {
            Ok(mut door) => {
                if game != "doors" {
                    door.run = Some(board.game_start(self.session()?, game)?);
                }
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
                if screen.score.is_some()
                    && self
                        .door
                        .as_ref()
                        .is_none_or(|d| d.turns == 0 || d.run.is_none() || d.score_saved)
                {
                    return self.door_failed(theme, board);
                }
                let d = self.door.as_mut().ok_or(Error::Denied)?;
                d.pending = None;
                let mut screen = screen;
                if self.classic_games.as_ref().is_none_or(|g| g.is_empty()) {
                    screen.choices.retain(|c| {
                        !matches!(c.target, doors::Target::Classics | doors::Target::Server(_))
                    });
                }
                d.screen = Some(screen);
                d.focus = 0;
                self.door_save_score(board)?;
                self.door_show(theme)?;
            }
            Some(Err(_)) => self.door_failed(theme, board)?,
            _ => (),
        }
        Ok(())
    }
    fn door_save_score(&mut self, board: &mut Board) -> faction_core::Result<()> {
        let session = self.session.as_ref().ok_or(Error::Denied)?;
        let d = self.door.as_mut().ok_or(Error::Denied)?;
        if d.score_saved {
            return Ok(());
        }
        if let Some(score) = d.screen.as_ref().and_then(|s| s.score) {
            let run = d.run.as_mut().ok_or(Error::Denied)?;
            match board.game_finish(session, run, score) {
                Ok(result) => {
                    d.score_saved = true;
                    d.notice = Some(format!(
                        "{} Saved {} points. Best: {}. Completed runs: {}.",
                        if result.improved {
                            "PERSONAL BEST!"
                        } else {
                            "RUN COMPLETE."
                        },
                        score,
                        result.best,
                        result.plays
                    ));
                }
                Err(Error::Busy) => d.notice = Some(
                    "Score not saved: board busy. Open scores to retry before leaving this run."
                        .into(),
                ),
                Err(Error::Invalid(_)) => {
                    d.notice = Some("Score not saved: this board's score limit was reached.".into())
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
    fn door_scores(
        &mut self,
        game: &str,
        board: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.door_save_score(board)?;
        let leaders = board.game_scores(self.session()?, game)?;
        let personal = board.game_personal_best(self.session()?, game)?;
        let mut text = vec!["Rank  Member                    Best       Runs".into()];
        for (i, score) in leaders.iter().enumerate() {
            text.push(format!(
                "{:>2}.   {:<24} {:>10} {:>10}",
                i + 1,
                score.handle,
                score.best,
                score.plays
            ));
        }
        if leaders.is_empty() {
            text.push("No completed runs yet. Set the first score!".into());
        }
        text.push(personal.map_or_else(
            || "Your best: no completed runs yet.".into(),
            |s| {
                format!(
                    "Your best: {} points from {} completed runs.",
                    s.best, s.plays
                )
            },
        ));
        text.push("Higher wins. Ties keep member registration order. BBS-local only.".into());
        let d = self.door.as_mut().ok_or(Error::Denied)?;
        d.resume = d.screen.take();
        d.screen = Some(doors::Screen {
            title: format!("High scores // {game}"),
            text,
            prompt: None,
            state: String::new(),
            score: None,
            choices: vec![
                doors::Choice {
                    key: b'R',
                    label: "Return to the arcade screen".into(),
                    target: doors::Target::Resume,
                },
                doors::Choice {
                    key: b'P',
                    label: "Play this game".into(),
                    target: doors::Target::Game(game.into()),
                },
                doors::Choice {
                    key: b'L',
                    label: "Arcade lobby".into(),
                    target: doors::Target::Lobby,
                },
                doors::Choice {
                    key: b'B',
                    label: "Back to the BBS".into(),
                    target: doors::Target::Back,
                },
            ],
        });
        d.focus = 0;
        self.door_show(theme)
    }
    pub(super) fn door_show(&mut self, theme: &Theme) -> faction_core::Result<()> {
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
        if let Some(notice) = &d.notice {
            out.push_str(notice);
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
                doors::Target::Scores(game) => self.door_scores(&game, board, theme)?,
                doors::Target::Classics => self.classic_list(board, theme)?,
                doors::Target::Server(slug) => self.classic_confirm(&slug, board, theme)?,
                doors::Target::Connect(slug) => self.classic_connect(&slug, board, theme)?,
                doors::Target::Resume => {
                    let d = self.door.as_mut().ok_or(Error::Denied)?;
                    d.screen = Some(d.resume.take().ok_or(Error::Denied)?);
                    d.focus = 0;
                    self.door_show(theme)?;
                }
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
        let screen = d.screen.as_ref().ok_or(Error::Denied)?;
        if screen.score.is_some() || d.score_saved || d.resume.is_some() {
            return Err(Error::Denied);
        }
        let state = &screen.state;
        match doors::Request::start(uid, &d.game, state, input) {
            Ok(request) => {
                d.pending = Some(request);
                d.turns = d.turns.saturating_add(1);
                self.state = State::DoorPending;
                self.prompt_started = Instant::now();
                self.queue("\n");
            }
            Err(_) => self.door_failed(theme, board)?,
        }
        Ok(())
    }
}
