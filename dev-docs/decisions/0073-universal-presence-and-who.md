# 0073: One presence model, distinct Who and Lounge interfaces

Status: accepted, 2026-10-23
Issue: [#176](https://github.com/gonz0w/faction-bbs/issues/176)

## Decision

The owner explicitly separates the main-menu Who caller list, group `/who` and
the Lounge's social/paging experience. Who and group `/who` show the same visible
board-wide callers, current broad activity and availability/status. Lounge keeps
its roster, invitations, answers, privacy and page-the-sysop attributes. Visibility
and available/busy/away state are universal, with the existing explicit opt-in and
fresh hidden login. Current block/approval/epoch/lease checks apply everywhere.

Derive broad location from trusted host UI transitions, using a closed enum. Never
accept a member-supplied location or disclose private room/channel identifiers,
draft/game contents, addresses or native encrypted-mail work. Keep only one current
row per live lease; schema 18 adds a bounded transient relation with cascading
cleanup and read-only older-snapshot verification. Restore/export never resumes
activity or presence. A stopped complete backup precedes upgrade; no mixed owners
or in-place downgrade.

Reuse existing loops, Board and connection leases. Charge shared transitions to
existing work and back off on contention; no per-member helper/thread or listener.
No dependency/resource-default increase. Short broad-location samples may lag
under contention; this is not a history or delivery guarantee. The existing social
feature gate governs all three, while owner Lounge naming remains Lounge-only.
Themes can name/map Who independently; select a free hotkey if W is used by Wall,
and allow the explicit word `who` in plain menus. Classic fallback remains.

## Alternatives and verification

Separate rosters/preferences would contradict universal privacy. Inferring names
from private room/channel joins would leak resource participation. An in-memory
shared-only map would omit standalone/worker parity; a custom sidecar store would
introduce a second storage format. A small SQLite transient relation fits existing
lease, transaction, backup and bounded-session controls at one schema migration.

Verify actual local/worker/shared SSH/native/plain visits, transitions, universal
hide/busy/away/status, block/revocation/stale/disconnect/disable, pagination/custom
keys, protected draft/game input and complete recovery. Preserve previous evidence
before edits. No capacity, physical-SBC, Ghostty-pixel or hardening claim; saved
owner playground stays stopped/intact. #175 is a prerequisite, not merged by this
owner clarification.
