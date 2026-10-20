# 0013: Managed gateway lifecycle and bounded load/failure evidence

Issues: [#18](https://github.com/gonz0w/faction-bbs/issues/18), [#20](https://github.com/gonz0w/faction-bbs/issues/20), [#23](https://github.com/gonz0w/faction-bbs/issues/23), [#35](https://github.com/gonz0w/faction-bbs/issues/35).
Status: implementation/native validation complete; [PR #53](https://github.com/gonz0w/faction-bbs/pull/53) open for review. #52 merged at `08c50bb`.

## Goal and constraints

Manage the existing restricted gateway on NetBSD: start/status/stop/restart,
single-instance ownership, TERM/INT/HUP draining, worker reaping and crash recovery.
Retain eight workers and launch throttling. Exercise bounded admission/load,
slow/failed clients and service/broker/worker failure with real processes. No
physical SBC, production deployment or 100–250-user capacity claim.

## Progress

- [x] Read requirements and #18/#20; merge verified passing #52.
- [x] Implement lifecycle and operator scripts without new registry dependencies.
- [x] Run host checks and fresh native NetBSD software evidence; Linux CI required on the current PR head.
- [x] Publish documentation, measured results and focused PR #53.

## Steps and commands

Use Rust 1.96 release builds, workspace tests and four existing PTY workflows.
Add a disposable process harness to CI. Boot a fresh official signed/hash-checked
NetBSD 11 ARM64 2-vCPU/1-GiB guest with loopback host forwards; native build/tests,
real restricted SSH and lifecycle/load/failure probes. Remove guest, forwards,
keys and private inputs afterward. Save only sanitized measurements/source hashes.

## Discoveries and decisions

2026-10-20: #52 broker lacked signal handling, socket recovery and orphan-worker
handling. Keep native signal handling confined to the existing audited FFI crate;
the handler only writes a lock-free C11 atomic. Worker parent monitoring bounds
broker-loss lifetime, including blocked terminal output; forced exit can leave
chat presence until the existing 60-second lease expiry.

## Validation and outcomes

Native NetBSD 11 ARM64: 54 Rust tests, four PTY workflows, seven local lifecycle
oracles, 20 actual application SSH oracles and 13 service/load/failure oracles
passed. Current-source evidence is in [report 0026](../research/0026-service-lifecycle-load.md)
and saved 024/026 JSON. Seven active members drained/reaped in 5.481 s; broker-loss
member exit observed in 0.167 s. Eight-member chat and rejected 100/250 local
offers passed without descriptor growth. These offers are not 100/250 members.
43 source/config/lock/harness/checker hashes match. Guest/keys/raw logs/disks and
forwards were removed; host SSH and other VMs untouched. Host release Clippy/Rust,
four PTYs, seven lifecycle, 23 Python and dependency/repository checks passed.

Remaining: whole-host USS/PSS/authentication peaks and 100/250 actual connected
members before raising cap; storage/power-loss, upgrade rollback, packaging and
physical-target claims remain separate gates. No schema/dependency version change.

## Review handoff

Branch `codex/18-service-lifecycle`, PR #53. Native evidence/source identity and
cleanup checks pass; CI must pass on the current PR head before merge. No broad
issue is closed. After merge, measure whole-system private/shared costs and
concurrent authentication before changing the eight-worker gate. Existing native
ABI/CMSG regression remains required. Do not rerun obsolete temporary builder
scripts: they can overwrite newer code. Near-term work stays software/VM-first.
