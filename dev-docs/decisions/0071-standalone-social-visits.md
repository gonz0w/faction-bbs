# 0071: Synchronous standalone social visits

Status: accepted for implementation, 2026-10-23
Issue: [#137](https://github.com/gonz0w/faction-bbs/issues/137)
Extends [ADR 0070](0070-connection-scoped-social.md).

## Decision

Local login and the inherited worker own one optional chat/social lease for the
approved BBS visit. A private Visit borrows the existing Board connection; it
allocates the lease only when chat or an enabled Lounge opens. Both reuse it.
Chat exit leaves every channel/reset receipts and destroys its editor while
retaining explicit availability. Logout and ordinary error/EOF paths close the
lease; a drop guard covers unwinding. A failed channel-part cleanup ends the
visit instead of renewing retained joins from the menu. Abrupt process termination
or broker-watch exit still relies on the existing 60-second stale-lease expiry, not secure erasure.

Heartbeat every ten seconds, silently, in existing synchronous input waits. Poll
at bounded intervals with the original 120-second line deadline. Raw menu, room
picker, directory and full-screen editor loops tick the same owner. Authentication
and identity precede lease allocation; short graphics negotiations retain their
existing bounded states. No new thread/helper, listener, SQLite connection,
registry dependency, table or per-member lease is introduced.

Private member adapters accept Visit and coerce its Board for existing current
core/common presenter calls. Public authentication/menu entry signatures remain
unchanged. Explicit context avoids global/thread-local Board ownership, unsafe
pointers or a separate keepalive process/database connection.

Reuse the common current-read Lounge panel, runtime theme gate, public bounded
status/bodyless invitations and safe menu star/header. Question mark refreshes
both ANSI and plain menus without executing an action. The panel uses bounded
ASCII input, re-fetches current eligible rows on action/resize, and returns safely
if disabled. No status query runs on every idle Lounge poll. It cannot expose
hidden counts, private channel names, private-message bodies or OS authority.

## Scope and validation

Classic/plain local sessions and inherited workers receive the same visit scope
as the shared service. Shared-only transfer/game capabilities are unchanged;
this is no new standalone Lua/legacy door adapter. Availability survives supported
local workflows, not a guarantee for a process killed or blocked indefinitely.
No unsolicited page output over input/editors/probes. Optional idle notification
UX remains follow-on #137 work; hardening stays deferred.

Use real registered/approved member PTYs on the host and a marked NetBSD VM.
Measure one simultaneous 66-second wait at the local menu, worker full-screen
editor and plain canonical room prompt; verify heartbeats and original unsent
input. Check privacy/quiet/block/resize, named themes/custom keys, sysop opt-in,
plain worker, current revocation, clean logout, worker EOF, fresh hidden visits,
terminal restoration and schema/unread/transfer invariants. Preserve prior
measured sources/results independently and run maintained recovery/terminal checks.
