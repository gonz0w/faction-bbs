# 0028: Local arcade bests under host-owned run authority

Status: implemented candidate for [#75](https://github.com/gonz0w/faction-bbs/issues/75).
Date: 2026-10-21. Builds on [ADR 0027](0027-lua-arcade-boundary.md).

## Question and gates

How can ordinary members compete and replay without making Lua a BBS account or
database authority? Owner requests persistent scores and useful game doors.
Approval, local data, separate game UID, no JS/TS/dependency addition, bounded
output/storage and existing process/credential limits are mandatory. No external
service or money is needed. Physical SBCs and expanded load work remain deferred.

## Alternatives

- Session-only scores: simple and private, but fail the requested persistence.
- Game-owned score files/SQLite: require game UID to hold identities/data and add
  independent backup/permission semantics. Reject for this slice.
- Full run history: useful for auditing but unbounded growth and unnecessary local
  play metadata. Defer; scores grant no real-world entitlements.
- Host-owned personal best/count: choose. One row per member/game, 64 game keys per
  member; indexed top-ten results. Existing SQLite/export boundaries apply.

## Decision and tradeoffs

The core issues a nonserializable, non-cloneable run bound to board/member/epoch.
The connection alone holds it. The host accepts an optional integer score only
from a complete terminal frame, after gameplay input, without live state/prompt
or further input actions. It commits using the current authenticated Session and
marks the run finished only after commit. Busy failures can retry from the result
screen. Repeated browsing cannot count a run twice; process loss abandons unsaved
runs. Lua receives neither run authority nor member identity. No durable run log
is necessary because authority cannot be imported/replayed after restart.

Root-installed scripts are trusted scoring code, so a modified game may invent
points. This design prevents terminal-selected score/member injection; it does
not attest fair play or support funded competition. Only approved members read
scores; revoked players are omitted, with retained bests/counts still local in
private backups. Reapproval can show their old best again.

Schema 3 adds only game_scores and its index. Exact-catalog snapshot validation
accepts schema 2 or 3, includes scores when present, and rejects invalid slugs,
counts, scores, foreign keys or extra SQL. Restore preserves fresh board identity;
opening a schema-2 restored/live store performs the transactional migration.
Reversal cost is moderate: disabling doors preserves scores; old binaries require
a pre-upgrade private snapshot, losing changes since that snapshot if downgraded.

## Primary evidence and validation

Primary implementation sources, inspected 2026-10-21:
[scores.rs](../../crates/faction-core/src/scores.rs),
[schema](../../crates/faction-core/src/scores.sql),
[snapshot validation](../../crates/faction-core/src/snapshot.rs),
[host UI](../../crates/faction-service/src/door_ui.rs) and
[evaluator](../../native/faction-doors/main.c). No external provider/API adopted.
The core tests exercise ordering/ties/top ten, duplicate/foreign/stale authority,
busy retry, personal game limits, reopen, migration rollback and old/new backups;
protocol tests reject active/duplicate/out-of-range scores. Actual Lua and SSH
play/reboot passed (72 Rust tests, 18 evaluator and 17 SSH check groups). See
[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/75) for results and limitations.

Confidence is limited to recorded software tests; no capacity, physical SBC,
all-platform, hostile native-code or production sandbox claim. Reopen for game
moderation/reset/versioned score seasons, resumable runs or native legacy doors
under existing [#10](https://github.com/gonz0w/faction-bbs/issues/10) and
[#34](https://github.com/gonz0w/faction-bbs/issues/34).
