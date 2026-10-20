# 0026: NetBSD managed gateway lifecycle, bounded load and failures

Date: 2026-10-20. Issues: [#18](https://github.com/gonz0w/faction-bbs/issues/18),
[#20](https://github.com/gonz0w/faction-bbs/issues/20),
[#23](https://github.com/gonz0w/faction-bbs/issues/23),
[#28](https://github.com/gonz0w/faction-bbs/issues/28).
Decision: [ADR 0013](../decisions/0013-managed-gateway-lifecycle.md).
Status: fresh native campaign passed; sanitized final-source evidence saved and private guest removed.

## Question, fixture and scope

Can the restricted eight-worker gateway start, drain, stop and recover on NetBSD
without leaking workers/descriptors or losing the local approval boundary?
Reuse the [official release provenance and private fixture](0024-restricted-ssh-netbsd.md):
NetBSD 11 ARM64, QEMU/HVF, two vCPUs, about 1 GiB RAM, native Rust 1.96,
bundled SQLite, separate terminal/backend UIDs and real restricted OpenSSH.
Software evidence only; no physical SBC or production deployment.

## Reproduction and pass/fail rules

Build/test the exact source in the marked disposable guest. Run the four existing
PTY workflows and `tools/smoke_broker_lifecycle.py` as the non-root backend owner.
Install root-owned rc.d scripts and dedicated key/configuration from
[the service guide](../../docs/service-lifecycle.md), leaving admin SSH separate.
Run the existing `tools/check_restricted_ssh.py` private fixture arguments with
`--lifecycle-output PRIVATE_RESULT_PATH`. This adds
`tools/check_service_lifecycle.py` after its 20 real application SSH oracles,
reusing generated credentials in memory. Never run this administrative harness
against an existing board: it requires a marker and initially absent database.

Eight real approved SSH sessions must enter chat and observe a delivered message.
A ninth must fail while all eight workers remain observable. Then offer 100 and
250 additional **local control connections**, all rejected at the cap; verify
eight workers, unchanged broker descriptor rows and healthy chat afterward. Those
offers are not authenticated SSH handshakes or 100/250 connected-member tests.

Measure process RSS/VSZ for application and SSH roles, a cumulative `ps` CPU sample,
VM page summary and chat delivery time. RSS sums double-count shared pages and
include the separate admin SSH fixture; this is not USS/PSS or a whole-host
application allowance. CPU sample is not a steady-state utilization benchmark.

Kill one worker, stop seven active sessions, kill a broker with a real member,
restart through its stale socket and authenticate afresh. Hold a SQLite write
transaction and require a failed post plus later fresh login. For output failure,
use a bounded synthetic trusted writer to a terminal-owned PTY that is deliberately
not drained; prove its progress stalls before its shortened two-second absolute
session deadline reaps it. This isolates supervision; real UI output deadlines
and end-to-end slow SSH links remain separate work.

Host/CI tests cover eight stalled handoffs (observe they really remain open),
duplicate instances, signals, stale recovery, non-socket/symlink preservation and
pidfile symlink refusal. They do not fake a different-UID terminal handoff. Native
real SSH supplies positive application coverage. Force-killed workers rely on
chat lease expiry, not a claim of immediate durable presence deletion.

## Observed results

[Saved sanitized evidence](results/026-netbsd-lifecycle.json) records 54 native
Rust tests, four native PTY workflows, seven native local lifecycle oracles and
13 real SSH/service/load/failure oracles. The companion [application evidence](results/024-netbsd-application.json)
was refreshed from the same final source: its 20 SSH oracles and native positive
CMSG transfer passed. All 43 relevant source/config/lock/harness/checker hashes
match the current tree; native binaries/smokes and installed rc.d script hashes
were independently compared with their guest copies. Host-run SSH orchestration
and the non-executing saved-evidence checker are labeled in the capture process.
Guest, loopback forwards, keys, disks and raw transcripts were removed afterward;
developer-host SSH and unrelated VMs were untouched.

| Observation | Native result |
| --- | --- |
| Eight approved SSH members, all in chat | Pass; all observed one arrival within 0.116 s in this local VM fixture |
| Ninth SSH connection at cap | Denied; eight workers remained |
| 100 then 250 local over-cap offers | All rejected; broker descriptor rows 15 before / 15 after; existing chat still worked |
| Service stop with seven surviving active members | 5.481 s; no member processes/socket remained |
| Broker SIGKILL with a live real member | Member exit observed after 0.167 s; stale socket recovered on start; fresh login passed |
| Eight stalled handoffs at shutdown | 6.265 s; no premature handoff closures mistaken for successful waiting |
| SQLite write contention | Bounded busy retry prompt; decline returned to menu, logout and later fresh login passed |
| Blocked synthetic writer | Live writer stopped progressing on its first 8-KiB write; PTY contained output; two-second session limit reaped it; probe completion verified |
| Dedicated SSH service | status/restart/fresh login/ordered stop passed |

One quiescent eight-chat process sample:

| Role | Count | Individual RSS KiB | Sum RSS MiB |
| --- | --- | --- | --- |
| Member | 8 | 4036–4060 | 31.59 |
| Terminal frontend | 8 | 1980–1988 | 15.50 |
| Broker | 1 | 3248 | 3.17 |
| SSH listener roles | 2 | 3784–4752 | 8.34 |
| SSH session roles, including administrative fixture | 18 | 4664–8820 | 113.70 |

Application RSS sums to 50.26 MiB; SSH roles add a 122.03-MiB RSS sum. These sums
include shared pages repeatedly and do not estimate incremental cost/member.
The guest reported 64666 free, 109434 active and 39961 inactive pages; it also
contained build tools/caches and the admin fixture. No minimal runtime-image or
private resident-memory claim follows. The coarse cumulative application `ps`
CPU sample rounded to zero and is not useful as a CPU-capacity result.

Run `python3 tools/check_service_evidence.py` and
`python3 tools/check_netbsd_application_evidence.py` to verify saved oracles,
cleanup and source provenance; neither command reruns NetBSD. CI additionally
runs the seven real unprivileged lifecycle probes on Linux. Host Darwin release
Rust/Clippy/tests, four PTY workflows, seven lifecycle probes, 23 Python unit tests,
dependency-policy and repository checks passed.

The first test pass exposed inherited nonblocking accepted sockets on Darwin;
explicitly setting accepted streams to blocking mode fixed premature receive
failure. An initially incorrect contention oracle expected a fatal SQLite string;
inspection showed the existing bounded retry UI was correct, and the test now
checks that positive recovery path. A writer-progress marker initially followed
its first write and was never reached when that write blocked; moving the marker
before the attempt plus live-process, queued-byte and completion assertions avoids
false success. The final native campaign used clean synthetic state and exact
final code, not the failed intermediate fixtures.

## Limitations and next falsifiable work

Keep eight workers. No 100–250 actual connected members, physical SBC, long-running
session churn, per-key/account fairness, authentication peak attribution, disk-full,
power loss or complete upgrade rollback tested here. Native ELF sizes and RSS do
not measure minimal boot-image size. Retain existing Argon2 security parameters.
Investigate measured whole-system costs before replacing process isolation or
raising concurrency; Lua/game doors remain separate, resource-bounded work.
