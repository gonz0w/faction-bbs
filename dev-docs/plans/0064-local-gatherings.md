# 0064: Local gatherings, chat nights and RSVPs

Issue: [#136](https://github.com/gonz0w/faction-bbs/issues/136)
Status: implemented and validated; ready for PR review

## Goal and constraints

Build the next social vertical slice: optional named local gatherings, create/edit/
cancel, counts-only RSVPs, explicit room/channel jumps and optional approved-login
reminders in local/worker/shared terminals. Preserve classic ANSI/plain fallback,
current approval/epoch/role/grant checks and eight-session/two-credential/three-
helper/output budgets. No external calendar, federation, private-mail decrypt,
network listener, new registry dependency or hardening adoption. Keep the owner
playground off; use an owned software clone for native validation.

## Progress

- [x] Read requirements/current core and preserve the prior executed inputs/result.
- [x] Add bounded schema-11 gathering core and old-export/current recovery support.
- [x] Add one shared inert workflow, service/local adapters, menus and login reminder.
- [x] Validate create/edit/cancel/RSVP/privacy/UTC/revocation, classic and native paths.
- [x] Update docs/README/ADR/changelog; prepare the checked focused PR.

## Decisions

One-off explicit UTC starts, 1–1440 minute duration, up to one year ahead; no
recurrence or automatic DST conversion. Event edits have revision checks. Config
default disabled; sysop organizers initially, optionally approved members. At most
64 retained events, eight future events per ordinary organizer, 250 responses per
event, 30-day lazy ended retention. Titles 64 ASCII bytes, descriptions 480 bytes/
eight <=72-column lines. Counts show going/maybe and own response, never attendee
names; a sysop may restrict aggregate counts to organizers/sysops. Responses are
plaintext owner data, not confidential from the host operator.

Room/channel destination selection never grants access. Only a deliberate join
rechecks the current event and existing room/channel grants. Cancelled/ended/hidden
or revoked-organizer events cannot accept responses or jumps. Hidden organizer
edits cannot clear sysop moderation. Optional reminders contain only a currently
visible event title/time, no private destination or attendee information.

Snapshot 31 freezes the preceding client-vault integration source/harness set and
whole record before any runtime edit. All forty-four historical identities and
forty-four saved guards remain intact; new results use their own source pins.

## Validation and outcomes

Core/shared UI implemented; 256 host/native Rust tests and eight local/worker
PTY controls per OS pass. Eleven actual restricted SSH/native/plain controls and fourteen complete-recovery
controls per OS pass, including actual distinct non-root NetBSD owner/file CLI.
Final clean-built native sources match 220 selected inputs; seventeen host harness
inputs and fifteen unchanged access/credential inputs are pinned. Existing native
room/editor/social/menu/art and local encrypted vault/picker regressions pass.
The first contention test exposed a redraw that requested a write transaction;
read paths now tolerate deferred cleanup under a competing WAL writer, while
explicit saves return Busy and retain the same unsaved edit for retry. Past-event
visibility/expiry, cascading RSVP cleanup and quotas have direct core controls.
Schema upgrades require stopped/drained complete backup;
old schema-2..10 exports stay read-only verifiable/restorable. Fresh-path rollback
uses matching older binaries and a complete pre-upgrade bundle.


No new registry package, thread/helper, listener, relaxed default or capacity claim.
The dedicated saved-result guard raises the historical/current suite to 45 guards;
all 31 frozen snapshots/44 old record identities stay intact. The disposable clone,
reviewed temporary vendor and test credentials are removed, and the owner playground
is stopped with disk/login/keys preserved. PR links belong in the final handover.
