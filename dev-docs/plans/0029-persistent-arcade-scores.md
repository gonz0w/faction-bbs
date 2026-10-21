# Persistent arcade scores

Issue: [#75](https://github.com/gonz0w/faction-bbs/issues/75). Follows #74.

## Outcome

Make the optional arcade worth revisiting: local personal bests, run counts and
per-game top tens, clear rules and navigation. Preserve approval, host-rendered
ANSI/plain screens, separate game UID and ordinary resource defaults.

## Steps

1. Add a schema-3 personal-best table and opaque host-owned run capability; enforce
   current member/epoch/board binding and one completion per run. Retain at most
   one score row per member/game, at most 64 game keys per member, top-ten output.
2. Include scores in exact-catalog snapshot validation; continue reading schema-2
   exports and migrate only trusted newly opened live/restored stores.
3. Add bounded optional score output to Lua frames. Host accepts completed results
   only after gameplay, commits under the owning member and offers score browsing,
   help, replay and lobby navigation. Scripts never receive identity or SQL.
4. Test score ordering, ties, lower scores, duplicates, stale/foreign authority,
   rollback/restart, old backups and new export/restore, malformed result frames
   and both actual Lua games. Rehearse SSH play/persistence on a disposable NetBSD
   VM; record exact code and limitations without editing historical measurements.
5. Update current docs/README/changelog, open a focused PR and pass CI.

## Validation / discoveries

Implementation complete. Native NetBSD: 72 Rust tests, 18 evaluator checks,
17 real SSH play/setup/recovery/reboot groups and strict C analysis passed. Scores
persist across disconnect/reboot and owner export/verify/restore; old schema-2
exports remain readable and immutable. Developer Clippy/fmt, workspace/release,
existing terminal regressions, 100 Python tests and 23 model/evidence validators
passed. See [the full record](../validation/netbsd-arcade-scores.md). The task VM
and private inputs were removed; unrelated VMs and paused research were untouched.

No new capacity or physical SBC claim. Historical #74
native record and its C-source override remain immutable; changed current code
will have a separate validation record.
