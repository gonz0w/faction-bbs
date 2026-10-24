# 0031: Repeatable 250-member headroom and the time daemon

Date: 2026-10-20. Issue [#58](https://github.com/gonz0w/faction-bbs/issues/58),
[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/58),
[ADR 0017](../decisions/0017-repeatable-headroom.md).

## Result and scope

Three fresh-boot 2-vCPU/1-GiB NetBSD 11 ARM64 trials completed the same
32/100/200/250-distinct-member real SSH menu/chat, member replacement and fresh
owner-console credential work. Every actual admission/stage decision, sampled
idle observation and sampled credential/control guard passed the unchanged
128-MiB predicate. Final headroom was 180.68–182.69 MiB;
the lowest sampled stress headroom was 137.23 MiB. All original base services,
including NTP, Postfix, mDNS, inetd, logging, DHCP and cron, remain present.

The effective change is the time daemon's supported pageable-memory policy,
combined with the prior experimental compact allocator/minimal handoff frontend
and the fixed bounded-dispatch Rust shared backend. Native service-map attribution identified a
large locked time-daemon footprint. Restoring its original setting reproduced
the credential/control shortfall. Allocator-only retain, minimum-slab and DSS
alternatives did not solve that working-margin problem. This is a fixed-host
policy improvement; no exclusive per-SSH private-memory saving is claimed.

This completes the bounded #58 short-workload research gate. Normal services
still use eight workers. Complete shared UI/theme/plain/resize/editor/sysop
parity, fairness and scoped configuration remain #55. Active file/game/extension
mixes, soak, accepted complete application budgets and authenticated-transport
flood bounds remain #28; maintained appliance/update/timekeeping policy remains
#20. These software VM observations do not establish physical SBC capacity,
all-NetBSD support or production deployment acceptance.

## Four final matched fresh-boot observations

All profiles use the signed official 11.0 image/kernel, two vCPUs and 1 GiB
configured RAM (actual physical 1054990336 bytes), Rust
1.96, base OpenSSH and the same separately versioned two-permit Rust backend/hardened minimal
C handoff frontend. The Rust core remains selected; neither the frontend nor
guest-wide tuning is adopted by normal installers. Reviewed registry/features
remain 39 NetBSD / 41 Linux packages with no JS/TS additions.

Each trial resets the marked offline fixture: 1000 distinct approved profiles
plus sysop, 1001 independently salted unchanged Argon2 hashes, 50 rooms and
100000 posts with 1-KiB bodies. Same SSH admission key/terminal UID intentionally;
BBS identities/password/approval remain separate. Five-second idle warmup,
one-second stage settling, roughly ten-second menu/chat observations, sequential
110-ms admission staggering plus protocol time. Profiling cap 250, burst eight,
100-ms refill and temporary guest maxproc/maxfiles 2048/16384; original 1044/3404
restore after each run. All four final boot identities and the eight exploratory campaign identities are distinct.

Final profile order: fixed-dispatch compact baseline, then three pageable-time
repeats with that exact same backend. Earlier exploratory trials compared the
allocator alternatives and counterbalanced restored time settings, exposing a
credential dispatch race described below. These studies are sequential rather
than randomized. Trial 1 of each profile includes 32 authenticated no-channel
transports and 32 handed-off password prompts; repeats omit attribution.
All necessary SSH/frontend/owner/crypto processes stay in whole-VM accounting.
Roughly 250 frontends, 501 dedicated SSH processes and one shared owner exist at
250; extra fixture administration is also included in VM counters. RSS sums
repeat shared pages; virtual maps and locked mapping ranges are not USS/PSS.

| Profile / trial | Completed stage | Final guard MiB / decision | Minimum sampled stress MiB | All stress guards | Stop/reap s |
| --- | ---: | ---: | ---: | --- | ---: |
| Compact / locked time / 1 | 250 | 143.76 / pass | 98.22 | fail | 1.204 |
| Compact / pageable time / 1 | 250 | 180.68 / pass | 139.66 | pass | 1.691 |
| Compact / pageable time / 2 | 250 | 182.62 / pass | 137.23 | pass | 2.289 |
| Compact / pageable time / 3 | 250 | 182.69 / pass | 141.73 | pass | 1.511 |

"Completed stage" includes menu/chat, fresh unique delivery/re-entry, member
failure/replacement and the additional 250-stage work. A functional completion
alone can still fail the final or sampled stress guard, as the baseline row
shows. The initial stale-binary pilot is excluded from this table and separately
recorded with its failed re-entry and recovery.

## Preserve exploratory failures and fix credential dispatch

The complete record also retains all eight preliminary workloads: two compact
baselines, one retained-extent, one minimum-slab, one DSS-first and three
pageable-time attempts. The first two preliminary pageable-time runs passed
their sampled guards; the third failed one of two valid replacement logins with
`credential work busy` despite substantial memory headroom. It completed the
200-member stage and reached 250 menus/chat before that rejection. It remains
an explicit functional failure and is not counted among final repetitions.

The original zero-length credential handoff could reject the second submission
before the second fixed worker began waiting. The new standalone Rust variant
uses a two-entry handoff with two owned permits spanning queued, active and
unconsumed-result work together. Disconnects keep a permit until work/result
disposal. A failed handoff, delivered/discarded result or shutdown releases it.
Queue capacity and aggregate outstanding work are both two; the two fixed
workers, cross-process kernel slots and Argon2 costs are unchanged. Legitimate
excess work still receives the same busy response. Connection-bound authority
and generation checks stay intact. A deterministic lifetime regression covers
submissions before receivers wait, busy while jobs/results remain, and release/
disconnected/shutdown paths. Final matched baseline and three repetitions use
the fixed binary, whose identity differs from the preserved exploratory backend.
No normal core/service adoption follows from this isolated correction.

## Actual guard and workload

Unchanged: free + max(0, clean file − wired) ≥ 128 MiB, zero swap, processes + 64
< maxproc and open files + 128 < maxfiles. Save every actual admission/stage/
replacement decision and its full snapshot, passed flag and shortfall; batch
admissions remain at most eight. Between-batch checks do not guarantee a
continuously held reserve. The new helper also evaluates this exact predicate
on every saved stress sample; no threshold or reclamation definition was relaxed.

At 200/250, two members leave; 198/248 remain in chat. Two wrong passwords close
and reap their transports, then two fresh real logins restore the full cohort
and unique delivery. At 250, two additional fresh owner-console processes perform
real password checks while all 250 members remain in chat, followed by fresh
delivery. These are private fixture administrator sessions, never a member RPC
or forced-command route to owner tools. Hidden input checks pass. Argon2id v19,
m=19456 KiB, t=2, p=1 and two cross-process kernel credential slots are unchanged.
Cumulative success counts alone do not prove instantaneous KDF concurrency.

Persistent native sampling requests 10-ms cadence and keeps all 150 samples per
stream with actual monotonic start/sample/finish/stop times and query costs.
Every run has touched-page 0/19/38-MiB controls at idle and, after complete 250
work, with that cohort still in chat. The extra 38 MiB is a known synthetic
allocation, not a cold Argon2 upper bound. Real member checks follow prior
logins; fresh owner processes still share a warm host/page cache. Sampled peaks
remain lower bounds. The complete ledgers preserve timing, calibration windows,
whole-VM anonymous/file/clean/wired/pool/free/swap counters and CPU/role costs.

## Identify the host cost before changing access

The controlled idle readback found ntpd reporting 18.04 MiB RSS and
18.04 MiB locked virtual ranges; the guest wired counter was
18.01 MiB. A fresh boot with the single
supported `rlimit memlock -1` option found the time daemon running with zero
locked ranges and 2.83 MiB reported RSS; wired fell to
0.004 MiB. Whole-VM differences include
cache/background effects and overlapping/shared pages; they are not private
daemon charges. The complete paired counters and service lists are saved.

The option allows the OS to page time-daemon memory; it keeps timekeeping and
every base service enabled. The VM has no swap devices configured; recorded swap
use/swapout and kernel OOM events remain zero. Clock precision under sustained pressure is an appliance adoption
gate. Saved wall-clock/host comparison intervals are narrow VM observations,
not external time-source accuracy certification. No time peer, address or config
content is published. Original/modified config hashes and exact restoration
are recorded. Normal host services and normal repository installers are untouched.

Native `__je_mallctl` readbacks qualify one arena, no tcache, zero dirty/muzzy
decay and retain false for the winning profile. Rejected variants record retain
true or native default/minimum slab geometry (all 36 bins), or DSS primary
preference. Requested option strings alone never establish effective settings.
Guest-wide settings also affect SSH/admin/background processes, so no causal
claim of process-exclusive savings. ADR 0017 records tradeoffs and primary sources.

## Live limits, executable identity and regressions

The native read-only CTL_PROC helper captures every live frontend, shared owner,
dedicated SSH listener/privileged/session process's actual soft/hard limits at
menu/chat stages, plus both owner consoles at their password prompts. Kernel
global profiling increases do not imply that inherited process limits increase.
Each complete readback retains the exact multiset of role/RSS/virtual/limit-profile
rows; identical rows use explicit counts. No PIDs, arguments, endpoints or member
content are emitted. Limits are observations rather than confinement promises.

A shared Cargo output directory initially reused a stale same-name standalone
artifact. A public two-member regression reproduced one successful chat cycle
then failure on the second, while an isolated build passed all three fresh-marker
cycles. A forced release rebuild produced the exact isolated-build binary hash.
The failed 100-member pilot was reaped/recovered and excluded from matched trials.
CI now builds experiments 055 and 058 into separate target directories. Historical
029/030/031 executed sources/data remain unchanged; source hashes alone cannot
establish which executable was installed.

With the winning profile, native validation passes 58 workspace Rust tests plus
one bounded-dispatch lifetime regression, five terminal
workflows, seven local lifecycle checks, 20 actual SSH application/escape-denial
oracles, 13 service/load/failure checks, all 16 shared approval/private-grant/
descriptor/slow-reader/crash/restart/recovery oracles, and 20 direct minimal
frontend contract checks. Isolated slow-reader failures complement the 250-member
credential cleanup; this is not a mixed slow-reader/file/game soak campaign.
Installed app/SSH/libc binary identities, 70 execution
input hashes and 58 native source copies are saved.

Final SQLite integrity and 1001 unique profile hashes / 50 rooms / 100000 posts
pass. All backend/frontend/member processes are reaped; only the dedicated SSH
listener remains before shutdown. Original time configuration and default native
allocator are restored. This task's VM, disks, generated credentials/keys, raw
logs and loopback forwards are removed; other VMs and host SSH stay untouched.

## Recheck and next build

[Complete sanitized evidence](results/032-netbsd-headroom.json) retains every
calibrated sample matrix and live-limit multiset. Run
`python3 tools/check_headroom_evidence.py` and
`python3 -m unittest discover -s tools -p 'test_*.py'` for saved evidence/privacy/
boundary checks. Linux CI checks source, dependency graph and saved ledgers;
native execution happened on NetBSD. [The experiment](../../experiments/058-headroom/README.md)
describes marked-fixture reruns. Hardware testing stays deferred.

Next build work is #55 complete shared UI/fairness and scoped frontend/allocator/
host policy adoption, with #20 clock/update/reversal and #28 full workload/soak/
admission budgets. Keep the useful local core first and Lua/isolated doors next.
