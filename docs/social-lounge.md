# Opt-in chat lounge and polite pages

The lounge helps approved members find people who chose to be available during
local chat. It works in the shared service, inherited worker and local console,
with the same commands on classic/plain terminals. It is disabled by default.

The primary shared SSH/native service now supports availability outside chat;
see the session-lounge section below. The standalone/worker path retains its
chat-only scope. Check requests explicitly with /pages or the Lounge panel;
there are no asynchronous page popups or bells. Accepting
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

Each new shared BBS connection starts unadvertised; standalone/worker chat entries
start unadvertised. Members explicitly choose:

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
One account advertises at most one live entry (shared visit or standalone chat);
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
Closing standalone chat or disconnecting a shared visit removes its advertisement
and invitations. Expired/crashed entries
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

## Availability outside chat in the shared service

The primary shared SSH/native service now has a main-menu **Social lounge**
action (default L). A sysop enables the existing feature with /social on in chat;
after that, approved members can open the lounge without joining any channel.
Choose /available explicitly, /busy, /away or /hide, and Q to return to the BBS.
The panel shows your own current advertisement, eligible members and current
bodyless signals. Existing page, answer, privacy and sysop controls work there.

This service keeps one optional lease for the complete connection. Availability
survives leaving chat while you read rooms, compose or play a door. Chat exit
parts all channels and discards its unsent chat input; it does not revoke your
explicit visit availability. Re-entering chat explicitly joins again with a new
history floor and send context. Logout/disconnect/service stop ends the lease.
No advertisement resumes automatically on a fresh connection.

A star on the Lounge entry and the generic header “Page waiting: open Social
lounge” indicate a current incoming invitation or an explicit reply to your sent
page. The header fits classic/narrow menus even if the Lounge entry is scrolled
offscreen. It is refreshed only at ordinary safe menu redraws; press ? to refresh
a waiting menu or open the lounge directly. There are no unsolicited popups,
bells or forced switches while editing, reading, running a game or negotiating
terminal capabilities. R refreshes the lounge. Expired signals do not become
permanent notifications, and acceptance never automatically opens a channel.

The panel re-fetches current eligible names, signals and personal controls on
every redraw, including resize. It keeps no old roster/page output in notices.
Current block, mute, quiet-hour, role/epoch and lease checks still apply. No
private-channel name or count of unadvertised members appears.

This outside-chat lifetime is a capability of the maintained shared service.
The legacy standalone console and worker rollback still use their original
chat-only lifetime and do not expose a session-lounge menu action. The earlier
chat-entry command descriptions above continue to apply to those adapters.
Standalone parity and optional idle notifications remain follow-on #137 scope.

Schema stays 17. There is no new listener, helper/thread, database connection or
second lease per member, and all existing resource limits remain. Complete
exports remove these connection leases and live social authority as before.
See [ADR 0070](../dev-docs/decisions/0070-connection-scoped-social.md),
[plan 0075](../dev-docs/plans/0075-boardwide-social.md) and the [current software evidence](../dev-docs/evidence/boardwide-social-netbsd/README.md). Later hardening, encrypted mail and federation remain separate.
