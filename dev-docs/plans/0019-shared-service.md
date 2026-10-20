# 0019: Integrate the shared backend into NetBSD services

Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55), related #20/#28.
Status: implemented and natively validated; PR review/merge pending. PR #61 merged at 09b506e.

## Goal and constraints

Promote the measured full shared UI to a maintained workspace binary, with strict
owner configuration and actual rc.d start/stop/status/restart. Keep base OpenSSH,
terminal UID separation, current core/epochs/grants, two-operation credential
budget and private owner tools. Default admission remains eight / five seconds;
250 requires an explicit experimental owner configuration. Retain worker rollback.
Measure the installed service path with all SSH costs and the unchanged guard.

## Progress

- [x] Read requirements, merge #61 and preserve executed source provenance.
- [x] Implement maintained binary/configuration/terminal entry point and rc.d selection.
- [x] Validate configuration, default admission, lifecycle/rollback/negative cases.
- [x] Repeat full-UI 250-member native service workloads and preserve all samples.
- [x] Update operator docs, evidence and decisions; prepare focused PR/backlog handoff.

## Discoveries and steps

Historical source pins include workspace manifests and deployment scripts. A
self-contained 103-file snapshot at `dev-docs/evidence/sources/09b506e` preserves
actual executed inputs and original dependency/build graph; all eight result files
stay byte-identical. Resolver changes never refresh old digests. New evidence checks
current service sources directly.

The service owns a single validated theme snapshot and bounded clients. Configure
absolute state/socket/theme paths, terminal UID, session cap/cadence/deadline in a
small strict root/owner-controlled TOML file. Integrate the existing reviewed C
handoff entry point as NetBSD-only native build input; Rust remains the core/server.
No new registry dependency, JS/TS, schema change or automatic host tuning.

Native tests use signed disposable NetBSD 11 ARM64 2-vCPU/1-GiB VM. The prior compact
allocator/pageable-NTP image profile is an explicit research input, not an rc.d side
effect or supported capacity promise. Restore/read back original settings and remove
private fixture inputs after preserving complete sanitized source/binary/counter data.
Physical SBCs remain deferred; active mix/soak/admission and maintained host-policy
scope/clock precision remain #28/#20 gates.

## Validation

Pinned Rust 1.96 fmt/Clippy/release workspace tests/build pass, including four
service tests (62 total). Native full UI 27, service lifecycle/configuration 11,
SSH 20, worker lifecycle 13 and frontend 20 checks pass. Host Python tests total 40.
Three distinct fresh-boot actual-service 250-member runs pass every unchanged
sampled guard: final headroom 180.67–181.12 MiB, lowest stress 135.23 MiB,
owner-only final-menu RSS 7.24–7.25 MiB, loaded stops 1.609–1.751 s. Complete
source/binary/counter/live-limit matrices are in results 034/research 0033.
Original host settings are restored/read back after reboot; synthetic fixture
integrity/counts are checked. Private VM/keys/data/logs/forwards are removed before
publication. Historical snapshot builds independently; all eight old records pass.

## Integration findings

NetBSD rc.subr status registration requires command to be set. NetBSD pgrep -x
uses the full executable path here. Both were fixed and native checks repeated.
The hostile-peer test initially blocked in host os.write, then a useful command
waited behind flooded transport input. Final harness uses nonblocking attack
writes, validates independent-peer service and discards attack backlog before
fresh useful login. These attempts are not counted as application passes; the
application loop stayed unchanged. Embedded SQL inputs were added from the
original merge revision when build-checking the frozen snapshot.

Repository/policy/evidence checks and 40 Python tests pass. Publish the focused
PR and evidence-linked issue handoff; require the exact-head GitHub check before
merge. #55 remains open for stronger confinement/frontend review and #18/#20/#28
for maintained host policy and wider workloads.
