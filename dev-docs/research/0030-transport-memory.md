# 0030: Native allocator and remaining SSH transport memory

Date: 2026-10-20. Issue [#58](https://github.com/gonz0w/faction-bbs/issues/58),
[plan 0016](../plans/0016-transport-memory.md),
[ADR 0016](../decisions/0016-native-allocator-comparison.md).

Follow-up: [research 0031](0031-repeatable-headroom.md) records the later
repeatable-margin study. The observations and source pins below remain historical.

## Question and result

Can a smaller native allocation footprint close the remaining guarded 250-member
gap while keeping Rust, base OpenSSH and equivalent approval/terminal behavior?
Two fresh-boot minimal-frontend plus compact-allocator trials completed the short 32/100/200/250-member
menu/chat stages, unique-marker delivery, failed-login cleanup and two fresh real
credential checks with the remaining 198/248 approved members staying in chat.
Two allocator-only compact trials stopped at 248 after passing 200; both
matched default trials passed 200 and stopped at the original headroom
guard before 250. No normal service/default/schema/credential cost changed.

This advances the experiment question; #58 stays open for reliable margin.
One minimal-frontend run passed the final guard; the second completed the full
250-member functional workload but stopped after its final sample fell 4.10 MiB
below the 128-MiB threshold. These are not two capacity/guard passes. The experiment does not establish production
capacity, the full proposed application budgets, an appliance, physical Pi/SBC
behavior or #55 complete shared UI/fairness parity. The profile is guest-wide;
per-service scoping and active-mix/soak/host-service tests remain adoption gates.

## Reproducible scope

Signed official NetBSD 11.0 evbarm/aarch64 image/kernel, QEMU/HVF, two vCPUs and
1073741824 configured bytes; actual hardware/usable-page counters and versions
are in [the complete sanitized ledger](results/031-netbsd-transport-memory.json).
Rust 1.96.0 and the existing 39-NetBSD/41-Linux reviewed registry graph remain.
No new allocator library, frontend runtime, SSH or crypto dependency is added.
Physical hardware is unavailable and not a prerequisite for this software work.

Each trial resets the marked offline synthetic fixture (1000 distinct approved
profiles plus sysop; independently salted hashes, 50 rooms, 100000 posts with
1-KiB bodies) and reboots. Trial 1 for each profile also compares 32 authenticated
no-channel SSH transports and then handed-off password prompts. These observations
are not approved BBS member counts. Trial 2 omits attribution for both profiles.
All six boot identities are distinct. Same corrected Rust backend/data/workload
and mandatory SSH/frontend/crypto processes; defaults first, allocator-only
compact second, then minimal frontend plus compact,
not randomized. Baseline/kernel/cache variation limits precision.

Warmup is five seconds seeded idle, two seconds after attribution state changes,
native touched-page calibrations, one second before each menu/chat observation,
and about ten seconds per measured state. Admissions are sequential, 110-ms
explicit staggering plus actual protocol time. Experimental cap 250, burst eight,
100-ms refill; normal default stays eight with its original cadence. Temporary
guest-only maxproc/maxfiles 2048/16384 restore to 1044/3404 after each trial;
owner soft/hard launch limits after restoration are saved. Live per-process
RLIMIT readbacks during each cohort were not captured; these post-test limits
must not be presented as a complete live-process limit ledger. No host sysctls or other VMs change.

The effective native libc is jemalloc 5.3.0. A fail-closed `__je_mallctl` probe
reads version, arenas, cache, decay, retain and issetugid. Default readback:
eight arenas, tcache enabled, dirty decay 10000 ms, muzzy decay zero, retain false.
Compact readback: one arena, tcache disabled, dirty/muzzy decay zero, retain false.
The setting is a disposable guest `/etc/malloc.conf` symlink, active after reboot;
SSH, administration and collectors also participate. No process-exclusive savings
or portable allocator API is claimed. The generic online manual and legacy
jemalloc source alone initially suggested the wrong implementation; native symbol
and option checks corrected that before any tuning decision.

## Complete transport accounting

All figures below are whole-VM MiB, not private process charges. RSS sums repeat
shared pages and virtual maps are not USS/PSS. Kernel pools are reported directly,
not added to overlapping VM categories as a manufactured total. The collector
includes the dedicated listener and all its SSH descendants plus every frontend,
member worker if present, and shared owner. Administration is excluded from role
sums but remains in whole-VM counters and CPU.

| Profile / state (first trial) | Transports | Anon MiB | File MiB | Pool MiB | Free MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| default / authenticated_transport_no_channel | 32 | 97.86 | 47.45 | 45.15 | 742.66 |
| default / handed_off_password_prompt | 32 | 115.34 | 47.65 | 50.45 | 717.20 |
| default / approved chat | 32 | 134.10 | 48.81 | 52.45 | 695.16 |
| default / approved chat | 200 | 571.02 | 52.23 | 128.28 | 140.14 |
| compact / authenticated_transport_no_channel | 32 | 85.31 | 47.46 | 45.07 | 755.38 |
| compact / handed_off_password_prompt | 32 | 98.30 | 47.66 | 50.32 | 734.60 |
| compact / approved chat | 32 | 109.01 | 48.66 | 51.91 | 721.14 |
| compact / approved chat | 200 | 497.09 | 52.14 | 127.71 | 215.26 |
| minimal / authenticated_transport_no_channel | 32 | 85.34 | 47.50 | 45.12 | 755.24 |
| minimal / handed_off_password_prompt | 32 | 95.83 | 47.70 | 49.92 | 737.91 |
| minimal / approved chat | 32 | 106.53 | 48.95 | 51.32 | 724.40 |
| minimal / approved chat | 200 | 482.02 | 52.42 | 124.83 | 234.18 |
| minimal / approved chat | 250 | 593.66 | 52.54 | 146.54 | 89.29 |

The authenticated-transport cost is already substantial before a BBS channel.
The prompt adds the selected frontend/TTY handoff; approval/menu/chat adds
core work and state. Sequential phases include allocator/cache/background
effects: their difference is not exclusive per-process attribution. Allocator-only improvement
still stops short of 250, so the measured remaining phase cost justifies the
separately hashed fixed-buffer handoff C candidate before considering a new SSH
implementation. The core/session authority remains Rust and base OpenSSH stays. Trimmed worker caches and shared RPC/UI workers
remain alternatives described in ADR 0016 and prior measured research 0029.

| Profile / trial | Completed menu/chat stage | Highest approved menus | Last guard headroom MiB | Stop/reap seconds |
| --- | ---: | ---: | ---: | ---: |
| default / 1 | 200 | 224 | 108.37 | 0.893 |
| default / 2 | 200 | 224 | 109.91 | 1.109 |
| compact / 1 | 200 | 248 | 127.12 | 1.178 |
| compact / 2 | 200 | 248 | 106.06 | 1.097 |
| minimal / 1 | 250 | 250 | 144.07 | 1.312 |
| minimal / 2 | 250 | 250 | 123.90 | 2.012 |

At each completed 250-member minimal+compact stage, role counts prove 250 frontends,
at least 501 dedicated SSH processes and one shared owner; no member-worker
process is hidden. These remain 250 distinct approved local accounts, not
cumulative reconnections, shared identities or rejected offers.

## CPU, delivery and guard behavior

| Profile / trial / chat cohort | Whole VM CPU, % of one core | Unique-marker delivery seconds |
| --- | ---: | ---: |
| default / 1 / 200 | 8.455 | 0.23 |
| default / 2 / 200 | 8.263 | 0.251 |
| compact / 1 / 200 | 5.738 | 0.252 |
| compact / 2 / 200 | 6.168 | 0.254 |
| minimal / 1 / 200 | 6.796 | 0.257 |
| minimal / 1 / 250 | 7.427 | 0.231 |
| minimal / 2 / 200 | 7.423 | 0.248 |
| minimal / 2 / 250 | 5.846 | 0.248 |

These are short local-NAT VM workloads, not latency/CPU promises on a Raspberry
Pi or real poor network. Allocation synchronization/page churn under active
files, games, scripting and unrelated host services still needs measurement.

The predicate remains free + max(0, clean-file - wired) >= 128 MiB, zero swap,
processes + 64 < maxproc and open files + 128 < maxfiles. Every actual admission/
stage/replacement decision saves its sample, pass/fail, available bytes and
shortfall. The previous experiment only saved a later pre-stop snapshot.

Default trial 1 stopped at 224 approved menus: actual headroom 108.37 MiB, shortfall 19.63 MiB. The last 8 admissions reduced the between-check measure by 26.20 MiB.

Default trial 2 stopped at 224 approved menus: actual headroom 109.91 MiB, shortfall 18.09 MiB. The last 8 admissions reduced the between-check measure by 26.21 MiB.

Between-batch checks cannot promise a continuously maintained 128-MiB reserve.
The minimal+compact functional completions do not justify loosening the predicate or production
admission limits. Predictive/per-admission bounds and authenticated-transport
flood control remain #28/#55 deployment work. Rejected/never-offered sessions
are never users. No observed swap use, swapout/OOM event or functional failure
occurred in the final six trials; fresh login passed after each shutdown.

## Real credentials under a useful cohort

At 200/250, two approved transports leave chat and log out; the other 198/248
distinct approved members remain live in chat. Two replacement attempts use bad
passwords, exit with restored terminal echo and are reaped. Two fresh prompts
then perform unchanged real Argon2id v19, m=19456 KiB, t=2, p=1. Hidden input,
successful identity/menu/chat and fresh unique delivery to the whole cohort pass.
Two kernel slots and normal credential-busy behavior remain unchanged; cumulative
success counts never prove instantaneous concurrency.

The persistent native sampler requests 10-ms cadence for 150 samples per stream;
all actual timestamps/query costs and 0/19/38-MiB touched controls are retained.
The following rises use whole-VM anonymous memory from the first sample and
are lower bounds, not instantaneous upper bounds or private heap attribution.

| Profile / trial / cohort left in chat | Successful / rejected credentials | Sampled anon rise MiB | Median / max gap ms |
| --- | ---: | ---: | ---: |
| default / 1 / 198 | 2 / 2 | 38.87 | 20.494 / 24.354 |
| default / 2 / 198 | 2 / 2 | 19.85 | 20.415 / 23.432 |
| compact / 1 / 198 | 2 / 2 | 33.03 | 20.763 / 24.602 |
| compact / 2 / 198 | 2 / 2 | 19.10 | 20.705 / 32.114 |
| minimal / 1 / 198 | 2 / 2 | 34.33 | 20.665 / 22.512 |
| minimal / 1 / 248 | 2 / 2 | 0.09 | 20.645 / 30.276 |
| minimal / 2 / 198 | 2 / 2 | 37.98 | 20.589 / 22.501 |
| minimal / 2 / 248 | 2 / 2 | 38.11 | 20.578 / 22.350 |

The zero-work and known touched allocation controls qualify observation; they do
not turn sampled authentication into an upper-bound proof. The 250-member operations
follow 200-member credential work and many earlier logins; allocation can be
reused/already charged. A near-zero sampled rise never means Argon2 needs near-
zero RAM. Cold concurrent credential/owner-tool mixes remain broader #28 work. See research 0029 for
the separately calibrated eight-prompt saturation ledger. No KDF was weakened,
artificially slowed or offloaded to a mandatory service.

## Correctness finding, regressions and provenance

A first pilot used 055's frozen backend with unique markers at 32 and 100.
At the second chat entry, its request counter continued from the previous lease;
the core correctly rejected that new lease's non-one first request. The pilot
stopped after 100 menus/32 fully completed stages, reaped sessions and recovered.
Reusing message text can hide delivery failure; the new driver always uses fresh
markers. This does not invalidate 0029's first completed 100-member stage, but
does expose a missing repeat-entry gate. Its source/evidence stay unchanged.

The separately versioned 058 backend copies 055 with exactly one line: reset the
request sequence to one when opening a new chat lease. Both allocator profiles
use the same corrected code. CI enforces this exact difference and identical
standalone manifests/lockfiles. New native multi-stage delivery is the regression
proof, not a test that merely mirrors the assignment.

With the Rust frontend/default allocator and with the minimal frontend/compact
allocator, native checks pass 58 Rust tests, five terminal
workflows, seven local lifecycle checks, 20 actual SSH application/escape denials,
13 service/load/failure oracles and all 16 shared approval/private-grant/forged-
control/positive-descriptor/slow-reader/crash/restart/recovery oracles. The minimal frontend also passes 20 direct contract oracles (actual TTY/packet,
success/error/EOF/SIGPIPE restoration and unsafe argument/TERM/endpoint denials).
Its native build passes GCC analyzer plus PIE/RELRO/immediate-binding/separate
RX-RW LOAD readbacks; it links base libc only. Rust/compact has the load comparison
but no separate duplicate full denial-suite run. Complete
theme/season/plain/resize/editor/room/sysop UI and shared-loop fairness remain
open. Isolated native nonreading-PTY cleanup complements high-load bad-login
cleanup; it is not a mixed slow-reader/250-member soak campaign.

57 frozen execution-input hashes match; 49 native source copies and 11 binary
file SHA-256/size identities are saved (the active C frontend and its saved
candidate file share one binary identity; the original Rust binary is separate). Full compact sample matrices retain every sanitized timing/counter row,
with no keys, passwords, transcripts, deployment addresses or member content.
Initial fixture-only listener/readiness failures are recorded rather than
counted as successful application checks. Final SQLite integrity, zero remaining
frontend/backend processes and SSH cleanup are checked. This task's VM, disks,
keys, generated credentials, raw logs and loopback forwards are removed; other
VMs and host SSH are untouched.

## Decision and continuation

Keep #58 open for repeatable headroom and live limit attribution. Retain the
Rust core and base OpenSSH. Continue the measured minimal handoff
frontend plus compact allocation profile as an experimental candidate. Manual
C bounds/lifetime review and a duplicated sensitive frontend contract are real
maintenance costs; a memory result does not select it as the production frontend. Keep guest-wide settings out of normal installers. Complete
#55 UI/fairness parity, scoped service configuration/rollback, and #28 active-mix,
repeat/soak/resource/transport-admission tests before production adoption. Lua
menus and isolated doors remain the first extension priority after the core.

Recheck saved evidence with `python3 tools/check_transport_evidence.py`; it never
starts a VM. Run native tests only in the marked disposable fixture following
[the experiment](../../experiments/058-transport-memory/README.md). Host Python,
format/Clippy, dependency and repository checks plus Linux CI validate source and
saved ledgers; Linux CI does not rerun NetBSD. Primary sources and reversal gates
are in ADR 0016. All measurements are synthetic software evidence with limits.
