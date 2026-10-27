//! Visit-local generic mail notices; never opens a vault or network connection.
#[cfg(feature = "network-mail")]
pub(super) struct Home<'a> {
    pub paths: &'a crate::launcher::HomeMail,
    pub unicode: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Counts {
    pub unread: Option<usize>,
    pub retained: usize,
    pub pending: usize,
}
#[derive(Clone, Copy)]
pub(super) enum NoticeContext {
    MainMenu,
    Files,
}
#[derive(Default)]
pub(super) struct VisitMail {
    configured: bool,
    notices: bool,
    snapshot: Option<Counts>,
}
impl VisitMail {
    pub fn new(configured: bool) -> Self {
        Self {
            configured,
            ..Self::default()
        }
    }
    pub fn configured(&self) -> bool {
        self.configured
    }
    pub fn toggle(&mut self) {
        if self.configured {
            self.notices = !self.notices;
        }
        self.snapshot = None;
    }
    #[cfg(any(feature = "network-mail", test))]
    pub fn record(&mut self, counts: Option<Counts>) {
        self.snapshot = if self.configured && self.notices {
            counts
        } else {
            None
        };
    }
    pub fn notice(&self, context: NoticeContext) -> Option<String> {
        if !self.configured {
            return None;
        }
        let open = match context {
            NoticeContext::MainMenu => 'O',
            NoticeContext::Files => 'E',
        };
        Some(if !self.notices {
            format!("LOCAL MAIL: notices off. {open} opens mail.\r\n")
        } else if let Some(counts) = self.snapshot {
            let status = counts.unread.map_or_else(
                || "tracking disabled; unread unknown".into(),
                |n| format!("unread: {n}"),
            );
            format!(
                "LOCAL MAIL / LOCKED\r\nstale local snapshot; home not polled.\r\n{status}\r\nretained: {}; pending: {}\r\n{open} opens mail; N turns notices off.\r\n",
                counts.retained, counts.pending
            )
        } else {
            format!(
                "LOCAL MAIL / LOCKED: unread unknown.\r\nNotices on; {open} unlocks manually.\r\nN turns notices off.\r\n"
            )
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notices_fit_the_compact_terminal_without_control_payloads() {
        let mut visit = VisitMail::new(true);
        for stage in 0..4 {
            if stage == 1 {
                visit.toggle();
            }
            if stage == 2 {
                visit.record(Some(Counts {
                    unread: Some(64),
                    retained: 64,
                    pending: 8,
                }));
            }
            if stage == 3 {
                visit.record(Some(Counts {
                    unread: None,
                    retained: 64,
                    pending: 8,
                }));
            }
            for context in [NoticeContext::MainMenu, NoticeContext::Files] {
                let text = visit.notice(context).unwrap();
                assert!(text.lines().count() <= 5);
                assert!(text.lines().all(|line| line.len() <= 39), "{text}");
                assert!(text.is_ascii());
                assert!(
                    text.bytes()
                        .all(|b| b.is_ascii_graphic() || b" \r\n".contains(&b))
                );
                assert!(!text.contains('\u{1b}') && !text.contains('\u{7}'));
            }
        }
    }
    #[test]
    fn context_hints_match_the_local_entry_in_every_notice_state() {
        let mut visit = VisitMail::new(true);
        for counts in [
            None,
            Some(Counts {
                unread: Some(64),
                retained: 64,
                pending: 8,
            }),
            Some(Counts {
                unread: None,
                retained: 64,
                pending: 8,
            }),
        ] {
            for enabled in [false, true] {
                if enabled {
                    visit.toggle();
                }
                visit.record(counts);
                let main = visit.notice(NoticeContext::MainMenu).unwrap();
                let files = visit.notice(NoticeContext::Files).unwrap();
                assert!(main.contains("O opens mail") || main.contains("O unlocks manually"));
                assert!(!main.contains("E opens mail") && !main.contains("E unlocks manually"));
                assert!(files.contains("E opens mail") || files.contains("E unlocks manually"));
                assert!(!files.contains("O opens mail") && !files.contains("O unlocks manually"));
                if enabled {
                    visit.toggle();
                }
            }
        }
        assert!(
            VisitMail::default()
                .notice(NoticeContext::MainMenu)
                .is_none()
        );
    }
    #[test]
    fn only_configured_visits_offer_mail_and_notices() {
        assert!(!VisitMail::default().configured());
        assert!(VisitMail::default().notice(NoticeContext::Files).is_none());
        let visit = VisitMail::new(true);
        assert!(visit.configured());
        assert!(
            visit
                .notice(NoticeContext::Files)
                .unwrap()
                .contains("notices off")
        );
    }
    #[test]
    fn explicit_opt_in_retains_only_stale_local_counts_and_opt_out_clears() {
        let mut visit = VisitMail::new(true);
        let counts = Counts {
            unread: Some(2),
            retained: 4,
            pending: 1,
        };
        visit.record(Some(counts));
        assert!(
            !visit
                .notice(NoticeContext::Files)
                .unwrap()
                .contains("unread: 2")
        );
        visit.toggle();
        assert!(
            visit
                .notice(NoticeContext::Files)
                .unwrap()
                .contains("unread unknown")
        );
        visit.record(Some(counts));
        let notice = visit.notice(NoticeContext::Files).unwrap();
        for text in ["LOCKED", "unread: 2", "stale", "retained: 4", "pending: 1"] {
            assert!(notice.contains(text), "{notice}");
        }
        visit.toggle();
        visit.toggle();
        assert!(
            visit
                .notice(NoticeContext::Files)
                .unwrap()
                .contains("unread unknown")
        );
    }
    #[test]
    fn error_or_disabled_tracking_never_claims_a_fresh_inbox_count() {
        let mut visit = VisitMail::new(true);
        visit.toggle();
        visit.record(Some(Counts {
            unread: None,
            retained: 2,
            pending: 1,
        }));
        assert!(
            visit
                .notice(NoticeContext::Files)
                .unwrap()
                .contains("tracking disabled")
        );
        visit.record(None);
        assert!(
            visit
                .notice(NoticeContext::Files)
                .unwrap()
                .contains("unread unknown")
        );
        let mut off = VisitMail::default();
        off.toggle();
        off.record(Some(Counts {
            unread: Some(99),
            retained: 99,
            pending: 9,
        }));
        assert!(off.notice(NoticeContext::Files).is_none());
    }
}
