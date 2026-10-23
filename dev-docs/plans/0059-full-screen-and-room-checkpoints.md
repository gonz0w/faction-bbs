# 0059: Full-screen room editing and recoverable checkpoints

Issue #141. Status: implemented and software validated; ready for review.

The owner authorized merging passing #148 and continuing full-screen editing and
saved drafts while preserving classic fallback and deferring hardening. #148
passed its required workflow and merged at a837a0f. This branch began while that
workflow finished; its completed change is based on the checked integration base.

## Scope and decisions

Use the same <=1024-byte / 32-line room body with cell-aware caret arrows,
paragraph split/join/delete, preview/quote and explicit confirmed Send. Preserve
one-line posts and the line desk; full mode is `.screen` on capable ANSI geometry,
with explicit Line and resize recovery. Sysops may name the desk through an inert
bounded `writing_title` theme field. No external editor/path launch.

Choose explicit server-side room checkpoints for ordinary SSH parity; native-only
persistence would exclude ordinary terminal users and require a new device format.
A checkpoint is plaintext board data readable by the sysop, opt-in through Save,
never private mail. Keep native member-side room drafts a future alternative.
Four rooms/member, one checkpoint/room, 30-day recovery with lazy per-member expiry,
revision-checked save/delete and current board/member/epoch/room/reply binding.
Only explicit Save persists; unsaved changes can be lost. Cancel deletes the
current checkpoint, Leave preserves it. Send uses its stable request ID and clears
that revision in the posting transaction; same-key retry cannot duplicate a post.
Other connections' newer checkpoints are not silently overwritten/deleted.

Schema 8 adds the bounded table. Old schema-2..7 exports remain read-only/valid;
new owner exports include room checkpoints, restore rotates private session board
identity as before. Status changes/owner recovery clear affected checkpoints.
Logical expiry is not secure erasure: lazy rows, WAL and older backups may remain.
Require stopped/drained complete backup before upgrade; no in-place downgrade.
All private-message drafts, plaintext, unlock and keys stay device-side. Existing
helper/session/credential/output/drain/default/network budgets remain unchanged.

## Progress

- [x] Merge checked #148; preserve 180 actual inputs and whole result in snapshot
  26, sources/pre141-full-screen / writing-desk-baseline-manifest.json.
- [x] Implement schema/API/export rules, bounded editor, theme name, classic and
  shared routes, explicit checkpoints/resume/send and current authority checks.
- [x] Unit, local/worker, real SSH/native/plain, resize/reconnect/conflict/expiry/
  unavailable reply/approval/retry, schema upgrade and export/restore checks.
- [x] Clean native NetBSD build and relevant browser/reader/menu/operator/art/
  complete backup regressions; pin actual final inputs/results.
- [x] Stop/remove owned VM/cache, keep saved playground off, final offline checks,
  updated docs/status and reviewable PR. No new capacity/hardening claim.

## Validation and discoveries

Final host/native Rust suites: **213 per OS**, host Clippy `-D warnings` and format
pass; Python **118**. Full-screen/saved-draft PTYs **7 per OS**, unsaved line-desk
regressions **13 per OS**, actual shared SSH/native/plain **11**. Browser **10 per
OS**, ordinary local menu/reader per OS, shared reader **22**, native directory
**11**, launcher **7**, operator **8**, art previews **18**, local art **7**, real
SSH/native art **7** pass. Complete backup/check/restore **11 per OS**; native
actual distinct-UID CLI and export/restore per OS pass. Core unit tests include
schema-7 readonly export/upgrade and schema-8 checkpoint export/restore.

A real resize failure inserted `[D` when resetting a partial arrow decoder.
Preserve decoder/scalar state across fitting resizes and gate fallback input at
Enter; final native/host controls and a fragmented-key unit regression pass.
The 30x12 return correctly uses a plain main menu. No runtime limits were relaxed.

The final native workspace was cleaned; **184 actual selected native inputs**
match copied hashes, **17 host harness inputs** are pinned, **25 dependency/access/
credential inputs** remain unchanged and all four installed binaries match that
build. New [evidence](../evidence/full-screen-netbsd/README.md) records exact
outcomes and limits. No new RSS/capacity or physical-device claim.

Saved checkpoints satisfy ordinary SSH recovery with explicit Save; automatic
keystroke persistence, richer editing and member-scoped draft exports remain
future increments. Owner backup/export is the implemented data recovery route.
Private mail stays device-side. Hardening remains later.

All **50 offline policy/evidence/Python commands** pass, including all 40 saved-
evidence guards. Owned VM/cache removed; saved playground has no open handle.
Formatting/diff checks pass; current Cargo manifests and lock remain unchanged.
