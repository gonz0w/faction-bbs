//! Read-only current-menu reference. No domain reads, commands or member content.
use crate::{Action, Key, Theme};
#[derive(Default)]
pub struct View {
    selected: usize,
    topic: Option<u8>,
    page: usize,
}
fn enabled(t: &Theme, sysop: bool, a: Action) -> bool {
    t.entries()
        .iter()
        .any(|e| e.action() == a && t.enabled(a, sysop))
}
fn hint(t: &Theme, sysop: bool, a: Action) -> Option<String> {
    t.entries()
        .iter()
        .find(|e| e.action() == a && t.enabled(a, sysop))
        .map(|e| format!("[{}] {}", e.key() as char, e.label()))
}
fn topics(t: &Theme, s: bool) -> Vec<(u8, &'static str)> {
    let mut out = vec![
        (b'A', "Welcome / house notes"),
        (b'B', "Your current menu keys"),
    ];
    if [Action::Rooms, Action::New, Action::History, Action::Post]
        .iter()
        .any(|a| enabled(t, s, *a))
    {
        out.push((b'C', "Rooms and the writing desk"));
    }
    if enabled(t, s, Action::Chat) {
        out.push((b'D', "Chat, Who and the Lounge"));
    }
    if enabled(t, s, Action::Files) {
        out.push((b'E', "Files and transfers"));
    }
    if [
        Action::Wall,
        Action::MemberPages,
        Action::Gatherings,
        Action::Polls,
        Action::Catchup,
        Action::Find,
        Action::Gallery,
        Action::Library,
    ]
    .iter()
    .any(|a| enabled(t, s, *a))
    {
        out.push((b'F', "Community places"));
    }
    if enabled(t, s, Action::Contacts) {
        out.push((b'G', "Visiting other boards"));
    }
    out.push((b'H', "Terminal and menu tips"));
    if enabled(t, s, Action::Sysop) {
        out.push((b'S', "Local sysop controls"));
    }
    out
}
fn lines(t: &Theme, s: bool, key: u8) -> Vec<String> {
    let mut out = Vec::new();
    let mut add = |text: &str| out.extend(text.lines().map(str::to_owned));
    match key {
        b'A' => {
            add(
                "Welcome to this sysop's world.\nLocal approval is required to use it.\nRooms are chronological discussion.\nChat is live; doors are optional games.\nCommunity content is local by default.\nPrivate mail stays on member devices.\n\nHOUSE NOTES",
            );
            if t.guide_notes().is_empty() {
                add("No extra house notes configured.");
            } else {
                add(t.guide_notes());
            }
        }
        b'B' => {
            add(
                "These are your current enabled keys.\nLabels and keys may vary by board.\nSelect actions from the main menu.\n",
            );
            for e in t.entries().iter().filter(|e| t.enabled(e.action(), s)) {
                out.push(format!("[{}] {}", e.key() as char, e.label()));
                if !e.description().is_empty() {
                    out.push(format!("  {}", e.description()));
                }
            }
        }
        b'C' => {
            add(
                "ROOMS / chronological conversation\nChoose a room, then New or History.\nRead complete pages to mark posts read.\nPost authors a subject and optional reply.\nOne-line Message input sends at Enter.\nUse /edit there for the writing desk.\n\nWRITING / before you send\n.preview: read a paged inert preview\n.quote: copy an allowed reply excerpt\n.save: explicit board checkpoint\n.leave: keep the last saved checkpoint\n.send: post from the classic line desk\n.cancel: discard this saved revision\n.screen: use the full-screen editor\n.line: return to classic editing\n\nFull screen: arrows move the caret.\nCtrl-P previews; Ctrl-S saves.\nCtrl-T asks for SEND confirmation.\nCtrl-Q returns to classic line editing.\nResize recovery keeps completed text.\nRoom checkpoints are sysop-readable.\nUnsaved edits can be lost on disconnect.\nNo private-mail draft is stored here.",
            );
        }
        b'D' => {
            add(
                "LOCAL CHAT / slash commands\n/list: currently accessible channels\n/join #name: choose a channel\n/part: leave the selected channel\n/names: joined channel participants\n/history: eligible retained messages\n/poll or blank: read new chat\n/me text: emote; //text: literal slash\n/quit: return to the BBS menu\n/help: the current chat command list\nChat is local; this is not IRC wire.\nPrivate channels are sysop-readable.",
            );
            if enabled(t, s, Action::Who) || enabled(t, s, Action::Lounge) {
                add(
                    "\nWHO / availability is separate\nMain Who shows broad current activity.\nGroup /who shows the same caller list.\nThe Lounge has invitations and controls.\nVisibility/availability apply to all three.\n/available text: explicitly share status\n/busy text or /away text: no invitations\n/hide: leave the shared caller roster\n/page handle: bodyless invitation\n/pages: inspect current page signals\n/answer ID accept or decline\n/notify on|off: quiet idle-menu notices\nStatus is public to approved viewers.\nPages never switch channels for you.\nNew visits start hidden; no auto-resume.",
                );
            }
            if s {
                add(
                    "\nSYSOP CHAT / local controls\n/create #name board|private\n/allow #name handle /deny #name handle\n/kick #name handle: parts current joins\n/social on|off: opt-in social module",
                );
            }
        }
        b'E' => {
            add(
                "FILES / deliberate separate transfers\nOpen Files from the main menu.\nThe native client negotiates there.\nIt provides a local Files menu and paths.\nOrdinary SSH uses manual SFTP steps.\nSeparate transfer admission still applies.\nReview destination and bytes yourself.\nRoom/chat faction-file links are references,\nnot public download URLs or permissions.\nCurrent file grants are checked again.\nA canceled upload is not publication.\nEncrypted-mail attachments are planned;\ncommunity files are not private mail.",
            );
        }
        b'F' => {
            add("YOUR BOARD / enabled community places\nNo feature below is a public web service.");
            for a in [
                Action::Wall,
                Action::MemberPages,
                Action::Gatherings,
                Action::Polls,
                Action::Catchup,
                Action::Find,
                Action::Gallery,
                Action::Library,
            ] {
                if let Some(h) = hint(t, s, a) {
                    out.push(h);
                    out.push(
                        match a {
                            Action::Wall => "  Short notes/art; review before sharing.",
                            Action::MemberPages => "  Bios, projects and explicit .plan updates.",
                            Action::Gatherings => "  UTC chat nights and deliberate RSVPs.",
                            Action::Polls => "  Local questions; review/change ballots.",
                            Action::Catchup => "  Return view; visits save explicitly.",
                            Action::Find => "  Search, bookmarks and exact sources.",
                            Action::Gallery => "  Curated original/licensed art editions.",
                            _ => "  Curated files and request fulfillment.",
                        }
                        .into(),
                    );
                }
            }
        }
        b'G' => {
            add(
                "OTHER BOARDS / independent communities\nDirectory cards are local editorial hints.\nCheck host keys out of band.\nEach board has its own login/approval.\nA listing grants no admission or trust.\nNo keys/accounts are forwarded for you.\n\nThe optional native Boards picker keeps\nyour own local profiles between visits.\nYou choose and confirm each connection.\nI checks local setup; T sets travel limits.\nOptional J holds explicit local souvenirs.\nIts notes are plaintext on your device.\nNo automatic travel log or federation.",
            );
        }
        b'H' => {
            add(
                "CLASSIC AND MODERN TERMINALS\nArrows/Tab select; Enter/hotkeys open.\nF1 opens this guide at an ANSI main menu.\nUse help then Enter at a plain menu.\n? refreshes the menu and current badges.\nQ leaves this guide without other actions.\nMenu keys are inactive inside typed chat.\nUnknown escapes need Enter to recover.\nCtrl-C exits the current interaction.\n\nTerminal settings select text/graphics.\nModern features need actual capability\nand explicit local approval where needed.\nPlain/classic access stays available.\nNo guide key posts, joins or transfers.",
            );
        }
        b'S' => {
            add(
                "LOCAL SYSOP / BBS ROLE\nSysop controls manage local approval,\nroom creation and enabled community desks.\nChat /help lists local channel controls.\nReview publication and privacy choices.\nYour BBS role grants no host shell.\n\nThe separate VM/OS operator console\nfollows OS login and shows host health.\nBackups/host setup stay operator tasks.\nThis guide changes no policy or files.",
            );
        }
        _ => (),
    }
    out
}
fn wrap(lines: &[String], width: usize) -> Vec<String> {
    let mut out = Vec::new();
    for line in lines {
        if line.is_empty() {
            out.push(String::new());
        } else {
            out.extend(
                line.as_bytes()
                    .chunks(width)
                    .map(|b| String::from_utf8_lossy(b).into_owned()),
            );
        }
    }
    out
}
impl View {
    pub fn feed(&mut self, key: Key, t: &Theme, s: bool) -> bool {
        let items = topics(t, s);
        self.selected = self.selected.min(items.len() - 1);
        match key {
            Key::Cancel | Key::Hotkey(b'Q') => return true,
            Key::Guide | Key::Hotkey(b'L') => {
                self.topic = None;
                self.page = 0;
            }
            Key::Up if self.topic.is_none() => {
                self.selected = (self.selected + items.len() - 1) % items.len()
            }
            Key::Down | Key::Next if self.topic.is_none() => {
                self.selected = (self.selected + 1) % items.len()
            }
            Key::Enter if self.topic.is_none() => {
                self.topic = Some(items[self.selected].0);
                self.page = 0;
            }
            Key::Right | Key::Hotkey(b'N') => self.page = self.page.saturating_add(1),
            Key::Left | Key::Hotkey(b'P') => self.page = self.page.saturating_sub(1),
            Key::Hotkey(k) if items.iter().any(|v| v.0 == k) => {
                self.topic = Some(k);
                self.page = 0;
            }
            _ => (),
        }
        false
    }
    pub fn render(&mut self, t: &Theme, s: bool, columns: u16, rows: u16, ansi: bool) -> String {
        let width = usize::from(columns.clamp(40, 120)) - 1;
        let height = usize::from(rows.clamp(18, 40)) - 9;
        let items = topics(t, s);
        if self.topic.is_some_and(|k| !items.iter().any(|i| i.0 == k)) {
            self.topic = None;
            self.page = 0;
        }
        self.selected = self.selected.min(items.len() - 1);
        let (title, body) = if let Some(k) = self.topic {
            (
                items.iter().find(|i| i.0 == k).unwrap().1.to_owned(),
                lines(t, s, k),
            )
        } else {
            (
                "Choose a topic".into(),
                items
                    .iter()
                    .enumerate()
                    .map(|(n, (k, label))| {
                        format!(
                            "{} [{}] {label}",
                            if self.selected == n { ">" } else { " " },
                            *k as char
                        )
                    })
                    .collect(),
            )
        };
        let body = wrap(&body, width);
        let pages = body.len().div_ceil(height).max(1);
        self.page = self.page.min(pages - 1);
        let mut out = if ansi {
            format!("{}\x1b[2J\x1b[H", t.menu_styles().0.sgr())
        } else {
            "\n".into()
        };
        out.push_str(&format!(
            "{}\n{}\n{}\nPage {} / {}\n\n",
            t.guide_title(),
            t.board(),
            title,
            self.page + 1,
            pages
        ));
        for line in body.iter().skip(self.page * height).take(height) {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str("\nArrows/Enter or topic key; N/P:page\nL:topics Q:main menu\n");
        if ansi { out.replace("\n", "\r\n") } else { out }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guide_is_current_menu_only_and_has_no_command_side_effects() {
        let mut t = Theme::default();
        let mut v = View::default();
        let text = v.render(&t, false, 40, 18, false);
        assert!(text.contains("Rooms and the writing desk"));
        assert!(!text.contains("Files and transfers"));
        assert!(!text.contains("Local sysop controls"));
        assert!(!v.feed(Key::Hotkey(b'E'), &t, false));
        assert!(v.topic.is_none());
        t.set_files_enabled(true).unwrap();
        v.feed(Key::Hotkey(b'E'), &t, false);
        assert!(
            v.render(&t, false, 40, 18, false)
                .contains("FILES / deliberate")
        );
        t.set_files_enabled(false).unwrap();
        assert!(
            !v.render(&t, false, 40, 18, false)
                .contains("FILES / deliberate")
        );
        assert!(v.topic.is_none());
        assert!(v.feed(Key::Hotkey(b'Q'), &t, false));
    }
    #[test]
    fn reference_pages_are_bounded_on_compact_and_large_terminals() {
        let t = Theme::default();
        let mut v = View::default();
        v.feed(Key::Hotkey(b'C'), &t, false);
        for size in [(40, 18), (120, 100)] {
            for _ in 0..8 {
                let out = v.render(&t, false, size.0, size.1, false);
                assert!(out.len() < 8192);
                assert!(out.lines().all(|l| l.len() < usize::from(size.0)));
                v.feed(Key::Hotkey(b'N'), &t, false);
            }
        }
    }
}
