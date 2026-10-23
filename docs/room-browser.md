# Browse rooms by name

In an approved ANSI session, **[R]ooms** opens a themed room browser. Choose a
room with arrows/hotkeys instead of memorizing numeric IDs. The same presenter
works in the local console, inherited member worker and shared SSH/native client.

## Find a conversation

The browser shows four rooms per page, with a stable room number, slug/title and
current unread count. Arrows/Tab select a room; Enter or A–D opens its card.
N/P or Left/Right changes pages. **U** moves to the next room with unread messages
in the listed set, wrapping at the end. If none is unread it leaves the selection.
**Q** returns directly to the main menu. Unicode follows your existing terminal
settings; classic mode keeps escaped, clipped text. Four rows fit a 40x18 terminal.

The current core lists the first 100 rooms, ordered by ID. New posts/rooms and
unread counts are refreshed on commands, not streamed in the background. Selecting
or opening a card never marks messages read. Names are presentation; the selected
room ID is checked again for each action. When a selected ID disappears, the
browser returns to list selection rather than silently posting into another room.
Titles too wide for the frame show an ellipsis; the room card shows its complete ID.

## Read or post from a room card

- **N / Enter** reads new messages in this room.
- **H** opens its chronological history.
- **P** clears the card controls and opens a composer labeled with the room ID.
  Enter the optional parent ID, subject and body using the existing composer.
- **L** returns to the list; **U** selects the next unread room; **Q** returns to main.

These paths skip the numeric **Room ID** question. Existing reply-subject defaults,
Unicode subject composing, retry IDs, chronological order and file-reference
permissions stay the same. After reading/posting, the normal Enter pause returns
to the main menu. This increment does not suspend a reader/draft or remember a
current room across main-menu visits.

The [reader](room-reading.md) marks only complete messages read. Shared sessions
also wait for the final screen to drain; leaving a partial page keeps it unread.
Current membership/approval and file visibility are rechecked on each page/action.
Revocation ends access before another room card/composer/page opens.

A resize while browsing returns to a working main menu; reopen R for the new
geometry. Ctrl-C/Ctrl-D closes the BBS session, consistent with the existing menu.
It does not silently publish a draft or start a room action.

## Plain terminals and existing shortcuts

Plain R keeps the original room-ID listing and line commands. Main-menu **N**,
**H** and **P** still accept numeric room IDs, including rooms beyond the first
100 when their ID is known and the current core permits access. Normal ANSI theme
hotkeys and their displayed labels remain configurable by the sysop.

The feature adds no database/schema migration, network protocol, room federation,
helper/thread, member-side room cache or dependency. Only an ID/card cursor stays
on each shared connection; at most 100 rows exist transiently per action and a
browser frame is at most 8 KiB. The existing queue/drain/default limits remain.
[Plan 0057](../dev-docs/plans/0057-room-browser.md) records exact software validation.

At the body prompt, `/edit` opens the optional [room writing desk](writing-desk.md)
for multiline paragraphs, quotes, preview and explicit send/cancel. Unsent drafts
remain in the current connection only.
