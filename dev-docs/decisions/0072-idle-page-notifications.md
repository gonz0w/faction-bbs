# 0072: Explicit visit consent and safe idle-menu page notifications

Status: accepted, 2026-10-23
Issue: [#137](https://github.com/gonz0w/faction-bbs/issues/137)
Extends [ADR 0071](0071-standalone-social-visits.md).

## Decision

/notify on|off controls idle page indications for this visit; default off on every
login. It is independent of public availability and persistent mute/block/UTC-hour
preferences. It grants no membership, channel access, OS authority or encrypted
mail access. Offer the command in the Lounge and existing local chat. Both use a
common bounded presentation state with current lease/approval checks.

Poll no more often than once per two seconds, only at a main menu with no pending
input/output or partial terminal control sequence, after two seconds without a
key. Keep at most eight current eligible signal ID/state pairs. Query existing
core eligibility afresh; do not cache sender names/text or queue events while
away from the menu. Coalesce unchanged signals, announce incoming pending or
explicit outgoing answers, and forget cleared signals without reviving them.
Database contention consumes the poll interval rather than retrying every sweep.

ANSI menus redraw in place, preserving arrow focus and decoder/typeahead state.
Plain menus print one short ASCII line and restore their empty command prompt.
Only opted-in standalone plain menus use tracked raw input, with the existing
byte/deadline bounds, so a partial canonical line cannot be overwritten. Ordinary
plain input remains unchanged when notifications are off. Never flush input as
part of an automatic notification.

Shared event-loop work remains charged to its original global budget and bounded
output queue. Local/worker input loops use the same existing Board/lease; no
background thread/helper, connection, listener, registry dependency or schema is
added. Schema stays 17 and normal resource defaults remain.

## Protected states and limitations

There are no page notices in composers, chat input, room readers, games/doors,
transfers, settings or terminal probes. They remain untouched even with /notify
on. Returning to the menu reads current signals; requests expired or made
ineligible meanwhile do not become delayed notifications. Existing normal menu
badges and explicit ? refresh work when notifications are off.

No bell, terminal desktop/OS notification escape, private-message body, forced
join or screen switch. An idle indication is ephemeral, not a mail/delivery
receipt. Logout/ordinary cleanup drops presentation consent; abrupt termination
uses existing lease expiry. Hardening remains a later phase. This completes the
ordinary supported shared/local/worker social story after software validation;
expanded notification surfaces would be separate future work.

## Verification

Use actual local/worker ANSI/plain sessions and marked NetBSD SSH/native/Lua
journeys. Verify default off/on/off/reconnect, quiet coalescing, focus, partial
menu/control input, protected draft/game/probe input, current privacy/expiry,
revocation and no schema/unread/transfer changes. Preserve all prior input/result
identities; record exact final source/binary hashes and remove owned fixtures.
No physical hardware, new capacity or new hardening claim.

The [current software evidence](../evidence/idle-notifications-netbsd/README.md)
passes 354 Rust tests per OS, twenty local/worker/plain notification controls per
OS and twelve actual shared SSH/native/Lua controls. Maintained real heartbeat,
terminal/art and complete-recovery regressions pass; final executed inputs and
installed binaries match. The saved owner playground stays off/intact.
