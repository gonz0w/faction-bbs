# 0017: Repeatable 250-member memory headroom

Issue: [#58](https://github.com/gonz0w/faction-bbs/issues/58).
Status: bounded native comparison complete; focused PR review pending. PR #59 merged at `b91e869` on 2026-10-20.

## Goal and constraints

Find repeatable useful margin on the 2-vCPU/1-GiB NetBSD 11 ARM64 software VM.
Keep the research 0030 backend, minimal handoff frontend, base OpenSSH, separate
terminal/owner identities, native descriptor checks and unchanged two-slot Argon2.
Keep the 128-MiB clean-file/free guard and process/file reserves. No adoption or
normal eight-worker default change. Preserve all historical execution inputs.

## Progress

- [x] Verify required CI and merge #59; review #58, constraints and old results.
- [x] Attribute repeated whole-host variation and actual live process limits.
- [x] Compare a bounded allocator/deployment alternative on fresh boots.
- [x] Repeat 250-member real SSH/menu/chat, credentials, failure and recovery;
  retain native input/binary identities and exact guard decisions.
- [x] Publish evidence, decision, current docs and backlog/PR handoff; check
  host/native results and remove the task's private VM/credentials/forwards.

## Steps and commands

Use separately versioned `experiments/058-headroom/` and
`tools/check_headroom_scaling.py`; existing 029/030/031 inputs stay immutable.
Start with actual live RLIMIT reads through NetBSD CTL_PROC and full VM counters.
Compare compact allocation with retain false/true: virtual extent retention may
reduce fragmented mappings/metadata while immediate decay releases unused pages.
This is a hypothesis, not a measured improvement or approval to change security.
Use matched guest-wide profiles, fixed synthetic distinct accounts/dataset,
unchanged workload/cadence/guard, fresh boots and saved effective readbacks.
Capture high-load owner-console credential work as well as member credentials;
sampled maxima remain lower bounds. Do not infer private memory from summed RSS.

## Discoveries and decisions

Prior minimal/compact runs completed 250 functional workloads but only one passed
the final guard (144.07 versus 123.90 MiB); the repeat had more anonymous/wired
memory. Native source exposes read-only process RLIMITs; use live values rather
than post-test launch-shell limits. No physical SBC support claim is intended.

## Validation and outcomes

Four final matched trials plus eight preserved exploratory workloads and complete
native regressions are recorded in
[research 0031](../research/0031-repeatable-headroom.md) / results 032. Full shared UI/fairness/deployment adoption remains
#55; broad active mixes, soak and authenticated-transport flood bounds remain #28.

### 2026-10-20 continuation

Completed compact, retained-extent, minimum-slab and DSS-first 250-member short
functional workloads. None leaves 128 MiB throughout the newly sampled member/
fresh-owner/control work; baseline reached about 98 MiB on fresh owner credentials.
Actual live limits captured. A shared Cargo output directory initially reused a
stale same-name standalone executable; excluded that failed pilot, added an
actual three-cycle SSH re-entry regression, and proved clean release and isolated
rebuild hashes match. CI now isolates standalone target directories.

Read-only host mapping attribution found ntpd locking about 25.6 MiB, matching
the guest wired counter. Next compare NTP's supported pageable-memory option,
keeping time synchronization and every base service running, with exact rollback
and repeated fresh-boot 250-member member/owner/control workloads. This host-policy
effect is separate from per-transport allocation; ADR 0017 records the tradeoff.

### Fixed dispatcher repetition gate

Two preliminary pageable-time runs passed all guards (lowest sampled 134.72 MiB
in the first); the third hit a reproducible design race in credential handoff,
returning busy with a worker slot logically available. Preserve that functional
failure and the eight exploratory trials. The separately versioned Rust candidate
now owns two permits over queued/active/result work; one deterministic lifetime
test, host Clippy/dependency checks and native build/test pass. Original Argon2
and cross-process budget stay unchanged. Final acceptance now uses a new matched
locked-NTP baseline plus three fresh-boot pageable-time trials with this fixed
backend, followed by all native regressions and complete private-VM cleanup.

### Final validation

Three pageable-time trials pass all actual and sampled guards at 250, with
180.68–182.69 MiB final headroom and 137.23 MiB lowest
sampled credential/control headroom. Native 58 workspace tests plus the dispatcher lifetime regression, five terminal workflows,
seven local lifecycle, 20 SSH, 13 service/failure, 16 shared and 20 direct frontend
checks pass; exact original time config/default allocator/limits are restored.
Source/binary provenance, all samples/live-limit multisets, SQLite integrity,
client/server reaping and private fixture removal are recorded. Host 36 Python
tests, Rust/dependency and all saved-evidence/repository checks accompany the PR;
exact-head CI/review status is recorded there. No normal service adoption.
