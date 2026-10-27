//! Current approved-member guide on the existing bounded shared owner loop.
use super::*;
impl Client {
    fn guide_theme(&self, b: &mut Board, base: &Theme) -> faction_core::Result<(Theme, bool)> {
        self.approved(b)?;
        let mut t = base.clone();
        let s = self.session()?;
        macro_rules! apply {
            ($f:path) => {
                if let Some(next) = $f(b, s, &t)? {
                    t = next;
                }
            };
        }
        apply!(faction_wall::menu_theme);
        apply!(faction_pages::menu_theme);
        if let Some(next) = faction_pages::social::menu_theme(b, s, self.lease.as_ref(), &t)? {
            t = next;
        }
        apply!(faction_gatherings::menu_theme);
        apply!(faction_polls::menu_theme);
        apply!(faction_catchup::menu_theme);
        apply!(faction_find::menu_theme);
        Ok((t, b.whoami(s)?.role == faction_core::Role::Sysop))
    }
    pub(super) fn guide_show(&mut self, b: &mut Board, theme: &Theme) -> faction_core::Result<()> {
        let (t, sysop) = self.guide_theme(b, theme)?;
        let size = self.size();
        let State::Guide(view) = &mut self.state else {
            return Err(Error::Denied);
        };
        let frame = view.render(&t, sysop, size.0, size.1, self.ansi);
        if frame.len() > 8192 {
            return Err(Error::Invalid("guide frame budget"));
        }
        self.queue(&self.capabilities.frame(&frame, self.ansi));
        if !self.ansi {
            self.queue("Guide command: ");
        }
        self.prompt_started = Instant::now();
        Ok(())
    }
    pub(super) fn guide_open(&mut self, b: &mut Board, theme: &Theme) -> faction_core::Result<()> {
        self.approved(b)?;
        self.dimensions = self.size();
        self.state = State::Guide(Default::default());
        self.guide_show(b, theme)
    }
    pub(super) fn guide_key(
        &mut self,
        key: Key,
        b: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<bool> {
        let (t, sysop) = self.guide_theme(b, theme)?;
        let State::Guide(view) = &mut self.state else {
            return Err(Error::Denied);
        };
        if view.feed(key, &t, sysop) {
            self.menu(theme, b)?;
            return Ok(true);
        }
        self.guide_show(b, theme)?;
        Ok(false)
    }
}
