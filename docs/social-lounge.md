# Social lounge, polite pages and idle notifications

The lounge helps approved members find people who chose to be available during
a BBS visit. It works in the shared service, inherited worker and local console,
with the same commands on classic/plain terminals. It is disabled by default.

Shared SSH/native access, local login and inherited workers support availability
outside chat; see the session-lounge section below. Check requests explicitly with /pages or the Lounge panel;
optional quiet idle-menu notifications are described below; there are no bells
or interruptions in active workflows. Accepting
a page never switches channels, overwrites a draft or launches a private session.

## Turn it on and meet

An in-BBS sysop enters C (local chat), then uses:

~~~text
/social on
/social name Night lounge
/social help
~~~

The name is 1–24 printable ASCII bytes. Existing sysop theme colors and classic
fallback remain; these commands confer no OS-console or host authority. /social
off clears live lounge presence and invitations, retaining personal controls.
No host policy, listener, service config or external account setup is needed.

Each new BBS visit starts unadvertised, including local login and inherited
workers. Members explicitly choose:

| Command | Effect |
| --- | --- |
| /available optional public status | Share available status with approved lounge viewers. |
| /busy optional public status | Stay on the opt-in roster while not accepting invitations. |
| /away optional public status | Advertise away, without invitations. |
| /hide | Leave the lounge roster and remove current invitations. |
| /who optional after-handle | See up to eight eligible opted-in handles/statuses; follow the shown cursor. |
| /page handle | Attempt a bodyless invitation to a currently available member. |
| /pages | Check current sent/received requests and replies. |
| /answer ID accept or /answer ID decline | Explicitly answer a received pending invitation. |
| /page-sysop | List currently opted-in, page-eligible local sysops. |
| /page-sysop handle | Invite a specific available sysop under the same limits. |

Public statuses are at most 48 printable ASCII bytes. Never place private content
in them. A busy/away/muted/quiet status is not a delivery guarantee. Choose a
mutually accessible chat channel yourselves after acceptance; normal private
channel grants and join-history floors still apply. There is no page subject,
body, file or plaintext private-message feature. Private conversations belong to
the member-device encrypted mail/private UI work under #90/#99.

Lounge opt-in differs from existing channel presence. /hide removes lounge
advertising; it does not hide your explicit channel joins from /names, erase
messages or make you anonymous to fellow channel members. /who reveals neither
private-channel participation nor counts of unadvertised/blocked people.
One account advertises at most one live visit;
explicitly opting in from a second entry replaces its previous advertisement and
invitations. A separate
still-live entry can remain advertised until closed, hidden or replaced.

## Personal controls

| Command | Effect |
| --- | --- |
| /mute on or /mute off | Stop/resume incoming and outgoing pages. Does not hide an advertised status. |
| /quiet HH:MM HH:MM | Set a UTC quiet interval; overnight intervals work. |
| /quiet off | Remove quiet hours. Use mute for all-day silence. |
| /block handle / /unblock handle | Maintain up to 16 normalized blocked handles. |
| /privacy | Show only your own mute, UTC hours and block settings. |

Controls survive reconnect and complete backup/restore, but availability does
not automatically resume. Quiet intervals include their start and exclude their
end; identical start/end is rejected. During quiet hours pages cannot be sent,
received or answered. Blocks suppress both parties from each other's lounge
rosters and page view; they disclose no account existence, approval or online
status. An unknown valid handle can be blocked without looking it up.

Changing availability or privacy controls removes old invitations; turning a
control off does not revive them. Current account revocation expires the lease
and hides its advertisement/requests immediately on the next checked action.
Sysops receive no override to inspect hidden members or someone else's pages.

## Limits, expiry and storage

Pages/replies live for at most 60 seconds under a valid clock/lease. Each account
may attempt one page per 30 seconds, including unsuccessful/unknown targets.
Cooldown survives hide/reopt-in and reconnect. Exact core nonce retries while
the signal is retained do not create another invitation; there is no permanent
retry log across expiry or removal. These are ephemeral signals, not a
reliable mail queue or delivery receipt. Unknown, hidden, blocked, quiet, busy,
muted or unavailable recipients produce the same generic attempt outcome.

Keep at most eight records per advertised entry, 128 total, 256 live presence
entries under the existing chat-lease ceiling, and 256 transient cooldown rows.
Roster screens show eight handles, never total online/hidden counts; page views
show at most eight signals. Common command frames stay below 4 KiB. No channel
history or per-member output queue is added.

Existing chat/visit heartbeats run every ten seconds; leases expire after sixty.
Logout and ordinary EOF/error cleanup remove the visit advertisement and
invitations. Abrupt termination relies on the existing stale-lease expiry. Expired/crashed entries
are excluded from reads immediately and reclaimed on later social writes/chat
opens. Clock rollback/future leases fail closed; cooldown guidance reports that
clock changes can delay a retry. Expired rows can remain at rest until cleanup;
this is not secure erasure from SQLite pages/WAL/raw copies/backups.

Schema 17 stores lounge config and private control intent separately from live
presence, signals and short-lived rate timestamps. Complete exports remove live
chat/social authority and rate timestamps; restored members log in and explicitly
opt in again through chat or, in the shared service, the Lounge. There is no
public last-seen log or central travel history.
Stop/drain and verify a [complete backup](export-restore.md) before the forward-only
upgrade; older binaries cannot open schema 17. Supported older snapshots remain
read-only verifiable/restorable and owner startup migrates them forward.

See [ADR 0069](../dev-docs/decisions/0069-opt-in-chat-lounge.md) and
[plan 0074](../dev-docs/plans/0074-social-lounge.md). Normal eight-session,
5000-ms admission, two credential permits, three owner helpers, output and drain
limits stay unchanged. No new registry package, helper/listener, IRC protocol,
federation, encrypted-message transport, capacity, hardware or hardening claim.

## Availability outside chat

Shared SSH/native access, local login and inherited workers have a main-menu **Social lounge**
action (default L). A sysop enables the existing feature with /social on in chat;
after that, approved members can open the lounge without joining any channel.
Choose /available explicitly, /busy, /away or /hide, and Q to return to the BBS.
The panel shows your own current advertisement, eligible members and current
bodyless signals. Existing page, answer, privacy and sysop controls work there.

Each adapter keeps one optional lease for the complete approved visit. Availability
survives leaving chat while you read rooms, compose or play a door. Chat exit
parts all channels and discards its unsent chat input; it does not revoke your
explicit visit availability. Re-entering chat explicitly joins again with a new
history floor and send context. Logout/disconnect/service stop ends the lease.
No advertisement resumes automatically on a fresh connection.

A star on the Lounge entry and the generic header “Page waiting: open Social
lounge” indicate a current incoming invitation or an explicit reply to your sent
page. The header fits classic/narrow menus even if the Lounge entry is scrolled
offscreen. By default it refreshes at ordinary safe menu redraws; press ? to refresh
a waiting menu or open the lounge directly. /notify on also enables safe idle
menu refreshes as described below. There are no unsolicited popups,
bells or forced switches while editing, reading, running a game or negotiating
terminal capabilities. R refreshes the lounge. Expired signals do not become
permanent notifications, and acceptance never automatically opens a channel.

The panel re-fetches current eligible names, signals and personal controls on
every redraw, including resize. It keeps no old roster/page output in notices.
Current block, mute, quiet-hour, role/epoch and lease checks still apply. No
private-channel name or count of unadvertised members appears.

Standalone login and inherited workers use the same panel and menu indicators.
Their existing input loops silently heartbeat the visit, including plain prompts,
room picking and full-screen editing. A new login starts hidden. A clean logout
or ordinary terminal EOF/error closes the lease; abrupt process termination or
broker-watch exit relies on the existing 60-second expiry. No keepalive helper or
second database connection is added. Shared-only file/game adapters remain
separate; this does not add standalone transfer or game support. Optional idle-menu
notifications now follow the same rules across these adapters; see below.

Schema stays 17. There is no new listener, helper/thread, database connection or
second lease per member, and all existing resource limits remain. Complete
exports remove these connection leases and live social authority as before.
See [ADR 0070](../dev-docs/decisions/0070-connection-scoped-social.md),
[plan 0075](../dev-docs/plans/0075-boardwide-social.md) and the [current software evidence](../dev-docs/evidence/boardwide-social-netbsd/README.md). Later hardening, encrypted mail and federation remain separate.

See [ADR 0071](../dev-docs/decisions/0071-standalone-social-visits.md) and
[plan 0076](../dev-docs/plans/0076-standalone-social.md) for standalone parity.

## Optional idle notifications

Use /notify on in the Lounge or chat to enable quiet notifications for this visit.
/notify shows the current setting; /notify off disables it. Every new login starts
off. This does not advertise you or override availability, mute, blocks, quiet
hours, local approval or sysop feature controls. Choose /available separately.
The Lounge displays the current visit setting.

After at least two seconds without input, an idle main menu may show the generic
“Page waiting: open Social lounge” indication. ANSI menus preserve the current
arrow selection; plain terminals receive a short ASCII line and a fresh empty
command prompt. A partial menu line or terminal control sequence defers it.
Repeated polls coalesce the same eligible signals; there is no bell or desktop
notification escape. Incoming pending invitations and explicit replies to your
outgoing pages qualify. No sender name or private content appears in the notice.

Drafts, chat input, readers, games/doors, file transfers, terminal settings and
probes receive no notification output. Returning to the menu checks current
eligibility; an expired, hidden, blocked, muted or quiet signal is not replayed
from a deferred event queue. Requests still expire after 60 seconds. The existing
safe menu badge and ? refresh remain available with notifications off.

This applies to shared SSH/native access, local login and inherited workers,
including classic/plain terminals. Only opted-in standalone plain main menus
track their input directly so partial typing is safe; their existing byte and
120-second line limits remain. No new helper/thread, SQLite connection, listener,
registry dependency or schema is added. At most eight ID/state keys are retained
per visit and optional polling is limited to once per two seconds while safe.

See [ADR 0072](../dev-docs/decisions/0072-idle-page-notifications.md) and
[plan 0077](../dev-docs/plans/0077-idle-notifications.md). Hardening remains later.
