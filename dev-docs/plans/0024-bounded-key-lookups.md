# 0024: Bound SSH helper lookup work

Issue: [#66](https://github.com/gonz0w/faction-bbs/issues/66), related #55/#23/#28.
Status: complete bounded implementation/validation, 2026-10-21; PR review pending.

## Goal and constraints

Bound repeated AuthorizedKeysCommand work on the measured NetBSD 11 ARM64/base
OpenSSH 10.2 path without replacing SSH parsing/crypto, changing local approval,
Argon2, member ceilings, eight-session defaults or the 128 MiB/zero-swap guard.
Keep native source penalties; this is not a total TCP/pre-fork limit. No physical
SBC dependency. Never share factionauth with extensions.

## Progress

- [x] Merge #67 after exact-head passing CI and no unresolved review threads.
- [x] Preserve 137 actual measured sources and unchanged 037/038 result hashes in
  a third buildable snapshot at merged commit 71491cc. Older snapshots unchanged.
- [x] Implement fixed four-lookup allowance per monitor birth identity and explicit
  quiescent v1 migration, with no key reads/writes/cohort scans after exhaustion.
- [x] Compare upstream PK_OK accounting and scoped admission alternatives.
- [x] Native ordinary, unsigned, invalid/signed-failure, options, migration,
  restart/state and process-birth checks; count actual helper/syscall work.
- [x] Native 250-member full host controls with useful chat and bounded abuse;
  retain failures and full accounting, verify cleanup/rollback.
- [x] Update operator docs/status/decision/research and open a focused PR.

## Steps and commands

Use native disposable release-image VM with serial-verified host keys, marked
synthetic fixture, pinned offline Rust dependencies and base OpenSSH. New source
and result files carry the new lookup campaign; never overwrite result 038 or
reuse old capture scripts without adapting output names. Check preserved records
with tools/check_evidence_sources.py, check_admission_evidence.py and
check_sustained_evidence.py. Run repository policy, Python regression tests and
CI; capture actual native identities and limits before deleting private inputs.

## Discoveries and decisions

The original helper emits a key file indefinitely for known unsigned offers.
Candidate charges four durable returns per monitor, allowing normal query plus
signed verification (measured clients previously needed two or three calls).
Exhaustion denies; on the selected base release denied requests count against
MaxAuthTries 3. Repeated recorded monitors skip cohort liveness scans. New
monitors still scan at most 250 slots. A v2 registry adds 250 uint64 counters
(2000 bytes); no daemon or per-member resident worker is added. Root migration
requires the member listener stopped, no surviving terminal/orphan sessions and
no live recorded monitor; no silent reset or live format conversion.

## Validation and outcomes

Initial preservation checks passed for eleven frozen records/three snapshots.
Native/whole-host validation passed: four PK_OK per monitor, 56 helpers/32
reads+commits for eight known-key adversaries, 24 helpers each unknown/signed-failure
cohort, positive ordinary clients/options, six state and one exhausted-restart
controls. Two fresh 250-member matrices pass every original guard/control and
three-minute 250/249/250 useful-member abuse intervals. Lowest active 155.14
MiB, lowest stress 130.36 MiB, zero swap. Same-source denial recovers at
50.48/50.55 seconds without reset; this is a limitation,
not a fairness pass. Source/result preservation, exact host rollback, ordinary
regressions, disposable data reset and private VM cleanup are verified. See
research 0038/result 039 for all counters and the corrected key-file-mode fixture
failure. #55/#23/#18/#20/#28 retain broader adoption and resource/policy gates.
