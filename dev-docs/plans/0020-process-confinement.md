# 0020: Native process envelope and terminal peer authentication

Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55), related #19/#20/#23/#28.
Status: implementation/native validation complete; focused PR prepared for review.
Production acceptance remains deferred. #62 merged at 627a824.

## Goal and boundary

Tighten the maintained service/frontend without adding per-member Rust workers,
new registry dependencies, weaker SSH/Argon2 or automatic host tuning. Preserve
the approved terminal flow, eight-session defaults and explicit experimental 250
profile. This slice adds kernel peer identity and irreversible resource ceilings;
it does not claim filesystem/network syscall isolation or same-UID hostile-code
isolation. Compare chroot/other native mechanisms and record the remaining scope.

## Steps

- [x] Verify/merge #62; preserve all measured #62 bytes before editing code.
- [x] Probe native limits/peer semantics and define exact threats/denial tests.
- [x] Implement peer checks, descriptor/TTY/environment hygiene and process limits.
- [x] Exercise actual frontend/service boundary attacks and existing UI/SSH/lifecycle.
- [x] Repeat fresh-boot 250-member native runs with full counters and unchanged guard.
- [x] Restore host settings, check integrity, remove private VM material.
- [x] Prepare README/operator/security docs, evidence/ADR and focused PR/backlog handoff.

## Provenance and evidence

The 90-file snapshot `dev-docs/evidence/sources/627a824` preserves current executed
inputs plus LICENSE, anchored to the actual #62 merge. Existing 09b506e snapshot
and all nine result files remain unchanged. Old validators resolve only matching
recorded bytes; new confinement evidence must check its current source directly.

Native probes must verify that hard limits deny fork/additional threads, locking
memory, excessive descriptors and core dumps without breaking fixed credential
threads, owner tools, SQLite WAL, full UI or 250 accepted connections. Do not assume
RLIMIT_RSS enforces RAM (NetBSD's manual says it does not). Keep actual soft/hard
readbacks, OS/build identities, public primary sources and outcomes.

Both frontend and backend must verify kernel-reported peer credentials, not just
socket pathname metadata. Reject elevated/mismatched identities, malformed or
mixed TTY descriptors, inherited descriptor authority and unwanted control data.
Direct probes must actually run under the intended identities; a root-created
mock socket with chowned metadata is not a positive peer-authentication oracle.

Software VM only: signed NetBSD 11 ARM64 / 2 vCPUs / 1 GiB, explicit compact
allocator/pageable-NTP research profile; no physical SBC gate. Preserve current
Argon2/two-slot lifetime, authorization, all SSH/frontends/base services and the
128-MiB/zero-swap/process/file reserve predicate. Mark stronger confinement,
active/soak/authenticated-transport admission and appliance policy as remaining.

## Findings so far

Native tests: 63 Rust; 31 frontend; 29 shared UI/peer/failure; 11 shared lifecycle;
20 worker SSH; 13 worker lifecycle; five unsafe-startup/clean-start checks. Native
C resource probes and frontend compiler/ELF hardening pass. The first complete
250-member run passes with 137.61 MiB lowest stress / 183.25 MiB final headroom,
exact limits in all nine readbacks and 1.338-second loaded stop. The two additional
fresh-boot repetitions also pass; the complete table is in research 0034.

The first shared lifecycle attempt encountered a stopped dedicated SSH listener
left by worker teardown. Its direct backend probes passed. Restore the maintained
frontend after temporary diagnostics and start the listener with its required
backend before the next campaign. The final counted 11/29 checks pass. No failed
setup or diagnostic binary is counted as measured service success.

The 627a824 snapshot independently builds with a separate target directory.
Both snapshots/all nine old records pass unchanged. Current-source validation
continues to reject drift even when a matching historical copy exists.

## Completion

All three fresh boots complete at 250 under the unchanged guard and exact live
ceilings. Original host settings are restored/read back after reboot, integrity
and counts checked, and private VM/keys/logs/data/forwards removed. Results 035
pin 94 current/native inputs and installed/build identities. Full repository,
dependency/evidence, 42 Python and Rust fmt/Clippy/release checks pass. The focused
PR advances #55/#19; filesystem/network/exec isolation, sustained active mixes,
transport admission and maintained host/timekeeping policy remain open.
