# 0053: Remaining host statistics and sysop-console polish

Issue: [#95](https://github.com/gonz0w/faction-bbs/issues/95); related #18/#98.
Status: implemented and validated; PR #129 open

## Goal and constraints

After checked #128, batch live CPU utilization, RAM used/free, network counters/
rates and bounded graphs with recent-history, clean exit and compact/resize polish.
Keep read-only OS-operator authority, classic/plain fallback, existing BBS owner
limits and optional private snapshots. No host applies, login-profile changes,
public listener, root shell field or hardening work. Peer/backup telemetry cannot
be fabricated for adapters/receipts that do not exist.

## Progress

- [x] Merge exact head 899bb45 after required check SUCCESS; merge 03872fd.
- [x] Preserve all 162 original operator/build inputs and complete result before
  edits in twentieth snapshot `sources/pre95-hoststats`.
- [x] Add bounded native read-only host counter bridge and delta/rate semantics.
- [x] Polish themed console panels/history/exit and compact/plain behavior.
- [x] Validate on host and fresh software NetBSD VM, including real console if available.
- [x] Measure dashboard separately, retain immutable evidence and clean owned clone.
- [x] Update docs, related issue status and focused PR.

## Design and execution

Extend the existing native bridge with fixed scalar outputs and bounded interface
sampling; no external registry dependency or resident helper. Counter deltas need
an earlier valid sample: first/reset/topology-change observations warm up rather
than report false zero/spikes. RAM used includes cache/kernel reservations and is
physical total minus genuinely free pages, not a claim of reclaimable availability.
Network counters describe non-loopback interface bytes, not unique application
traffic; limits/failures must be explicit. Local sampling pauses independently of
owner publication. Keep both histories and rendered frames bounded.

Use primary NetBSD sysctl(7), installed UVM/network headers and actual native
SDK/build/runtime observations; Mac support uses existing OS APIs. Rust application
code stays safe; FFI remains in faction-unix. Restore a clean shell cursor and
termios on exits without adding ANSI to plain mode. Never edit measured sources
or refresh old hashes after recording a candidate; preserve bytes first.

## Validation and outcomes

- Native NetBSD 11 ARM64 release build/tests: 183 tests; host Rust 183 and Python
  115, Clippy with warnings denied and formatting pass. Eight themed UI controls
  per OS; ten real shared operator/SSH/lifecycle/resource controls; seven existing
  shared SSH/native artwork regressions, eighteen previews/seven native logins.
- Four actual VM serial/getty controls: ordinary owner password login, manual
  launch/panels, reserved-row shell return/termios equality and logout to getty.
  Repeated after harness cleanup. Only a synthetic password in the disposable
  clone; no saved image/profile/getty change or physical console claim.
- Twelve half-second samples: dashboard 2,440 KiB RSS / one LWP, owner 5,144 KiB /
  four LWPs; minimum VM free 332.66 MiB. No per-refresh collector helper or new
  registry dependency. Short one-member software observation, not 250 capacity.
- macOS has more than sixteen interfaces; network explicitly unavailable under
  the fixed cap. Native NetBSD has one non-loopback interface and real interval
  CPU/network readings. Initial/resumed/reset/changed-scope semantics tested.
- 155 selected native source hashes match copied execution tree; nine host
  harnesses are pinned separately. Earlier original result/162 inputs preserved.

Owned VM stopped and temporary clone/cache removed; saved image has no open handle.
All 34 saved-evidence guards, twenty-snapshot/thirty-three-historical-identity
checks and repository policy pass. Four locked experiment target checks and
selected-target no-JS inventories pass. PR #129 is attached against the existing article branch; #95/#18/#98 status
updated. #95 stays open for peer/backup adapter telemetry and broader visual
acceptance. Clean committed archive passes source/policy/all 34 evidence guards.

The first GitHub check caught Python 3.12's twenty-static-block limit in the
historical-source test's growing mock list (the newer host Python accepted it).
Refactor only that unmeasured test setup to ExitStack, retaining all assertions
and old result/source identities. Native runtime/results stay unchanged. Updated
head checks pending; saved playground remains off.
