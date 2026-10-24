# 0059: Local one-off gatherings and counts-only RSVPs

Status: accepted for the hobby preview, 2026-10-22. Issue: #136.

## Context

Members need a reason and a time to meet in their own board. The existing local
room/chat authority and shared presentation should support an agenda without an
external calendar, new listener, owner helper or federation dependency. Ordinary
SSH/classic terminals must work and private mail must remain device-encrypted.

## Decision

Add optional default-disabled local gathering policy, events and responses in
schema 11. One small `faction-gatherings` workflow depends only on core/terminal;
local/worker and shared service adapters handle I/O. It has no host/network/SQL
extension authority. Existing typed actions open local room/chat workflows after
current authorization. Trusted themes may place an optional action but cannot
enable policy. Existing eight-session/5000-ms admission/two-credential/three-helper
and 32-KiB queue/five-second drain budgets remain.

Use explicit one-off UTC minute precision rather than adopting recurrence/timezone
libraries before users need them. Duration <=24 hours, starts within 366 days,
64 retained events/eight future per ordinary organizer, 250 responses/event,
30-day lazy ended retention. Titles/notes are bounded inert ASCII. No new registry
package, background scheduler, per-connection event history or attendee list.

Only an organizer edits/cancels; a sysop separately hides/shows/removes. Revisions
protect edits, moderation, cancellation and reviewed responses. Time/duration/place
changes atomically clear RSVPs. Foreign private-channel metadata is suppressed;
jump checks the current event and destination grants. Revoked organizers cannot
supply reminders/foreign cards/attendance/jump. Counts exclude revoked respondents.
No RSVP identity list API; optional organizer-only totals. These are display rules:
all event/response data remains local plaintext readable by the owner/backups.

Only explicit Save persists; an event create key survives a Busy retry while its
workflow is open. Retained-key same-content retry returns the same ID; changed
content conflicts. There is no durable receipt after removal/expiry and no secure
erasure promise. Unsaved connection-local drafts are not checkpoints.

The optional approved-login notice is title/time only, ongoing or within 24 hours.
It rechecks current policy/visibility after login art and forces a readable welcome
pause. It does not update room unread state or publish chat/mail notifications.

## Alternatives and consequences

An external calendar/recurrence service adds time/network/privacy and dependency
cost for a local hobby agenda; defer it until a specific interoperability story.
Attendee lists disclose more community metadata than needed; counts and own response
are sufficient now. A separately implemented local/shared agenda risks drift; the
small common workflow follows the existing wall/pages pattern.

Schema 11 needs a stopped/drained complete backup and fresh-path rollback. Old
schema-2..10 exports are validated against exact old schemas without mutating them.
New local policy/event/response rows travel in private owner snapshots/bundles.
Independent invited mail/home federation, NNTP room publication and IRC adapters
remain separate. No new capacity, physical-hardware or hardening claim.

## Evidence and reversal

See [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/136) and the dedicated
[NetBSD software record](../evidence/gatherings-netbsd/README.md). Snapshot 31 freezes
the preceding actual inputs and whole result; historical records stay fixed.
Remove the optional UI without touching old data, or restore the stopped pre-upgrade
complete bundle with its matching binary in fresh paths. Never downgrade in place.
