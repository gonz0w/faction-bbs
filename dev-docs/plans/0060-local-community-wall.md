# 0060: Local community wall

Issue #134. Status: implemented and software validated; ready for review.

The owner requested merging passing #149 and building the community wall with
classic fallback; hardening remains deferred. #149 passed run 39880368193 and
merged at 21285b5. This focused branch starts at that integration head.

## Scope

Opt-in local approved-member wall, separate from chronological discussion rooms,
IRC-style chat and device-encrypted private mail. Short inert ASCII notes/art,
explicit composition/review/send, chronological paging, stable-ID own editing/
delete and sysop removal. Bounded named policy, cooldown, retention and optional
UTC-Monday weekly editions with explicitly retained archives. No new connector,
listener, helper, registry dependency or relaxed resource/credential limits.

Schema 9 includes settings, notes and one cooldown/retry receipt per member.
Before upgrade stop/drain and make a verified complete backup; rollback restores
to a fresh path with the earlier binary. Older schema-2..8 exports remain valid
without modification. Logical deletion/expiry is not secure erasure.

A reusable faction-wall workspace crate owns only the bounded interaction and
trusted theme presentation, while faction-core owns authority and transactions.
Local/worker/shared adapters call the same workflow. Classic input is commands
followed by Enter; no external editor or terminal-control input is accepted.
Menu insertion is opt-in; large optional menus use a bounded scrolling window.

## Work

- [x] Merge checked #149; freeze 192 prior measured/harness/compatibility inputs
  and the whole original record in sources/pre134-wall, snapshot 27.
- [x] Storage, rotation, authority, limits, cooldown and owner snapshot validation.
- [x] Reusable interaction, local/worker/shared routes and sysop controls.
- [x] Unit and real local/worker/SSH/native/plain/compact product checks.
- [x] Native NetBSD clean build, backup/export, room/editor/menu/art regressions.
- [x] Record actual inputs/results, remove disposable clone/cache, update status,
  run policy/evidence checks and open reviewable PR.

## Validation notes

Use synthetic private test state and owned disposable NetBSD software clones only.
Keep saved playground off. Never change the host clock to simulate a weekly wall;
move only synthetic edition/timestamp rows in marked fixtures. No new supported
capacity, physical SBC, federation, hardened-host or encrypted-mail claim.

## Results and discoveries

Rust 1.96 workspace 224 per OS, host Clippy/format; Python 118. Wall local/worker
seven per OS and actual restricted SSH/native/plain eleven. Complete wall/file
recovery twelve per OS, native actual CLI with distinct non-root owner/file UIDs;
owner export/verify/restore per OS. Existing full-screen checkpoints seven per OS
and eleven actual shared, unsaved line writer thirteen per OS, room browser ten
per OS, shared reader 22, ordinary local menu/reader and native directory eleven,
launcher seven, operator eight, artwork eighteen previews/seven local/seven
actual SSH/native controls pass. No new resource/capacity claim.

The first local wall PTY exposed a flush-after-prompt first-keystroke race; enter
raw input before displaying the prompt. An actual archive-delete run exposed an
absent-edition redraw after the final archived note was removed; archived-note
deletion now returns to current with success. Final binaries/sources and both
flows were rebuilt and rerun. Native source/binary identity is checked against
the clean build, not just copied mtimes. Weekly test fixtures alter synthetic
editions/timestamps without changing the clock.

The community-wall-netbsd record pins 193 native inputs and nineteen selected
host harness inputs. Snapshot 27 preserves 192 prior sources and the complete
original full-screen result; forty historical identities stay fixed. Keep all
41 saved-evidence guards. Owned clone/cache removed, saved playground off.
