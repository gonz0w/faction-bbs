# 0005: Narrow the runtime and local storage choices

Issues: #4 and #9; related #18/#23. Status: local research complete; publication/review pending.
Base: PR #38 (`f9adfcb`), still open; use a dependent PR, do not merge it implicitly.

## Outcome and scope

Produce concrete, reversible recommendations for the first implementation experiment,
with native-session and SQLite recovery probes. Do not turn a microbenchmark into a
production security, portability or 250-user capacity claim. Preserve no-JS, approved
local membership, optional/private HTTP and local content ownership.

Owner steering 2026-10-19: newer or older technology is welcome; fit and evidence matter.
Small VM appliance and Raspberry Pi are interesting targets; investigate MINIX rather
than assuming Linux-only acceptance. Private-network-first access needs explanation;
it is not selected by the owner's question. SSH/private-overlay combination remains
a recommendation to validate, not a configured service.

## Predeclared experiments

1. Native session probe: C, Rust and Go standard-library/POSIX implementations, 250
   simultaneous local Unix-stream connections, same bounded ASCII command contract.
   Synthetic approval flag is a harness control, not authentication. Compare source
   complexity, compiled bytes, sampled process RSS, idle CPU and command latency.
   Require pending/revoked denial, oversized/fragmented input handling, bounded
   connection count, slow-client independence and identical responses before ranking.
   No TCP/SSH/crypto/storage/artwork/door integration; do not infer their costs.
2. SQLite probe: rollback journal versus WAL, FULL sync, bounded cache, same seeded
   synthetic posts and indexed pages; commit/read timing and on-disk sizes. Verify
   idempotent retry, approval within transaction, locked-writer bounded failure,
   reader snapshots, killed process before/after commit, logical export, online
   backup/restore, FTS deletion and retention. Actual power failure/full disk and
   hostile files remain later tests, not assumed passes.

Use temp directories with synthetic data; no external services or real members.
Compile binaries outside repository, sanitize compiler/OS metadata, save raw results
with source hashes. Repeat fresh native processes and independent DBs five times.
Compiler failures or missing candidates are evidence gaps, not losing scores.

## Progress

- [x] Read product/privacy/workflow, issues and prior budget evidence.
- [x] Capture owner steering and explain private-network access.
- [x] Build/run bounded experiments and record limitations.
- [x] Record comparative research and proposed decision records.
- [ ] Update docs, checks, issues and dependent PR; verify CI.

## Discoveries and owner decisions — 2026-10-19

- Owner clarified MINIX is optional; goal is practical fast/compact SBC/Pi hobby use and VM images, with newer/older technology on merit. Linux/NetBSD/minimal-image options remain research, no OS assumed selected.
- Owner explicitly accepted SSH terminal access with optional private-network mode; ADR 0004 records intent, not backend enforcement.
- Initial native scan approaches missed proposed connected-idle CPU target. Added a pre-measured follow-up comparing readiness-based Rust using pinned Mio, same contract/oracles/250 connections. Both Rust application variants forbid unsafe code; dependency internals are not claimed safe by that attribute.
- Twenty final native processes completed sequentially after the storage probe, five each C/Rust-scan/Go/Rust+Mio. C/Rust scan used ~6–7% idle core; Go/Rust readiness below coarse CPU resolution. RSS is sampled process-only, not full secure application charge. Final source-hash observations supersede preliminary runs.
- SQLite generic `iterdump` restore failed for external-content FTS (`no such table: search`). Preserve that negative result. Explicit versioned domain export with search rebuild passed final ten DB runs; do not generalize to all CLI dumps.
- SQLite WAL and DELETE both passed eleven scoped transaction/recovery/backup/export/FTS oracles. Process-kill recovery is not power-cut proof. File-object/hostile-import/full-disk/actual storage media tests remain later work.
- Recommend Rust/readiness + SQLite WAL for the next integrated validation slice, retaining Go fallback. ADR 0005 remains proposed, with production gate/exit tests; #4/#9 stay open and are not closed by this bounded PR.

## Reproduction and local validation

Commands from repository root: `python3 tools/measure_native_sessions.py --run`,
`python3 experiments/009-local-storage/probe.py --run`; source-provenance checks use
`--check` without compiling or measuring again. Native tool compiles/removes temp
binaries and caches; no IP listeners/real users/secrets. Record final repository/test
and CI outcomes in publication handoff. No SBC, VM image, real SSH, live registration,
real fan-out, power cut, door or provider was tested/configured.

## Local check outcomes

- `python3 tools/check_repository.py`: passed, including current links/policy/archive hashes.
- All prior model/replay/allocation `--check` commands passed unchanged.
- `python3 tools/measure_native_sessions.py --check`: passed saved source/workload evidence.
- `python3 experiments/009-local-storage/probe.py --check`: passed ten saved DB experiments.
- `python3 -m unittest discover -s tools -p 'test_*.py'`: all 23 tests passed.
- `git diff --check`: passed.

CI verifies saved evidence only; no compiler, registry fetch, timing threshold or DB
benchmark rerun is required. Required PR CI and publication/issue handoff remain next.
