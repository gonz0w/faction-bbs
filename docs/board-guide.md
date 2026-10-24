# The board guide

Status: approved-member guide preview, with classic/plain and ANSI routes.

At the ANSI main menu press **F1**, or at a plain menu type **help** and Enter.
The menu includes a hint. **? still refreshes the menu** and current badges;
existing hotkeys and custom-grid positions are unchanged.

The guide is a read-only reference. It never runs the displayed commands, posts,
marks rooms read, joins chat, starts a transfer or unlocks a device store. Return
to the main menu before choosing an action. F1 opens it only from an approved main
menu; it does not replace a draft, reader, game or capability probe.

## Find your way

Arrows/Tab and Enter select a topic; displayed topic letters work too. **N/P** page
through a reference, **L** returns to topics, **Q** returns to the main menu.
Classic/plain terminals use those letters followed by Enter. In ANSI mode Ctrl-C
returns to the menu; at a plain prompt the terminal may end the session. Resize returns to a working main menu; reopen the guide there.

| Topic | Reference |
| --- | --- |
| A | Welcome and the sysop's house notes. |
| B | Actual current enabled menu keys, labels and descriptions. |
| C | Chronological rooms, post/reply composition, preview and explicit checkpoints. |
| D | Local chat commands, with Who/Lounge guidance when the social module is enabled. |
| E | Native Files and the separate manual SFTP experience, when Files is enabled. |
| F | Enabled community places such as wall, pages, gatherings, polls and library. |
| G | Independent boards, deliberate profiles and local native travel tools. |
| H | Classic/modern terminal behavior and safe menu recovery. |
| S | Local sysop controls and the separate OS operator console, for a current sysop. |

Topics follow current feature gates. Disabled places and sysop-only entries do not
appear in the current-key reference. If a feature is disabled while its topic is
open, the next action returns to the current topic list. Approval/role checks stay
current. No private room/channel names, content snippets, hidden/online counts or
mail badges are fetched. Guide activity appears broadly as Menu in Who.

The guide explains existing product limits: chat is local command chat, directory
cards grant no remote membership/trust, room checkpoints are sysop-readable,
private-mail keys stay member-side and encrypted attachments remain planned.
A command example is guidance, not an active invitation, grant or transfer.

## Give your board a voice

A reviewed base or seasonal `theme.toml` can add:

```toml
guide_title = "CREW HANDBOOK"
guide_notes = '''Welcome to the night shift.
Ask before sharing somebody else's files.
Try the room browser before starting a new topic.'''
```

Place these root fields before any `[style]`/`[[menu]]` tables. Title is 1–24
printable ASCII bytes. Notes may be empty or up to 1024 bytes and sixteen lines,
using printable ASCII/spaces and LF. Raw ANSI, tabs, controls and non-ASCII reject
configuration. Notes are static approved-member text; keep secrets/credentials
out. The existing palette, theme ownership/validation, startup loading and seasonal
selection rules apply. Changing notes requires the normal reviewed restart/new
visit workflow; it does not become a remote template or member-editable command.

Guide pages fit the supported 40x18 ANSI baseline and remain below the existing
8-KiB frame cap. Large windows use bounded text pages rather than unbounded output.
Modern synchronization/text capabilities retain their existing gates; ordinary
SSH/plain access needs no native companion. Normal resource defaults and server
schema stay unchanged. See [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/182).
