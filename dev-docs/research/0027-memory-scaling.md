# 0027: Native memory profiling and distinct-account SSH scaling

Date: 2026-10-20. Issues: [#28](https://github.com/gonz0w/faction-bbs/issues/28),
[#20](https://github.com/gonz0w/faction-bbs/issues/20),
[#18](https://github.com/gonz0w/faction-bbs/issues/18),
[#23](https://github.com/gonz0w/faction-bbs/issues/23).
Status: guarded native campaign complete; current-source evidence captured, private guest removed. Decision: [ADR 0014](../decisions/0014-credential-budget-and-scaling-experiments.md).

## Question and controls

Profile full Faction + dedicated OpenSSH on a fresh official NetBSD 11 ARM64 VM:
two vCPUs/about 1 GiB RAM, native Rust 1.96/bundled SQLite, same local approval,
separate terminal UID and member-only entry point. Owner requests 100–250 connected
users; those are research targets, not a release/support promise. No physical SBC.

Use 1,000 approved fixture accounts, each registered through real core APIs with
an independently salted unchanged Argon2 hash. A private generated fixture password
is reused across these synthetic accounts; it is passed over stdin, never committed
or printed. Seed 50 rooms and 100,000 1-KiB authored posts. Chat retains the actual
product's bounded history rather than inventing a million-record retention feature.
Fixture bulk SQL is isolated in a developer example, never an installed member API.

Run stages 0/8/32/64/100/150/200/250 with independent SSH transports and **distinct
approved accounts**, not one account or channels in a shared transport. Each
nonzero stage holds menus, enters chat, measures short idle intervals, proves all
recipients receive one authored message, and returns to menus. The SSH policy,
encryption and MaxSessions=1 remain in force. Normal broker cap remains eight;
the experimental owner flags alone enable staged admission.

The fixture explicitly raises NetBSD's process/file ceilings from recorded defaults
to 2048/16384 before its experimental baseline, then restores them during cleanup.
No developer-host setting changes, automatic production tuning or additional RAM.
Guard further admission when free pages plus clean file pages, conservatively
discounting all wired pages, fall below 128 MiB, swap appears,
or fewer than 64 process/128 file entries remain. This is a conservative test guard, not an OS available-memory metric. Inactive
anonymous pages cannot be counted as reclaimable when there is no swap. Stop rather than crash the VM.

## Metrics and interpretation

The small read-only native C helper uses shipped sysctl headers/UVM/KERN_PROC2 and
KERN_FILE2. It serializes only role counts, RSS/virtual/high-water/CPU numbers and
whole-system counters; no PIDs, commands, addresses, member identities or paths.
Dedicated SSH ancestry includes its listener and helper descendants, excludes the
separate administrative SSH instance. Whole-VM counters still include the admin
collector and build/cache environment; report baseline and interval deltas.

RSS sums repeat shared pages and are not USS/PSS. NetBSD's mapping interfaces do
not expose resident shared-page attribution; map refcounts are not a substitute.
Virtual bytes use ps-compatible stack-adjusted pages, never resident memory.
Kernel high-water RSS is lifetime information, not synchronized total peak.
Interval CPU uses cumulative kernel/process counters; report percent of one core
(two-vCPU whole-VM ceiling 200%). Profiling connections add overhead.

Two cross-process slots bound simultaneous KDF arrays to about 38 MiB/board while
preserving 19 MiB per Argon2 operation. Synchronized password submission to eight
already admitted prompts observes successful/busy outcomes and recovery. Cumulative
successful logins need not be <=2 if scheduling makes them finish sequentially;
the two independent kernel locks enforce concurrency. No password weakening.

These short exploratory samples do **not** implement the five-repeat/ten-minute
warm-up/representative action mix/24-hour soak contract in research 0028. Idle/chat
delivery evidence is narrower than busy read/post/search/file/door service capacity.

## Reproduction

See [experiment guide](../../experiments/027-memory-scaling/README.md). Build and
run the actual application/PTY/SSH/lifecycle checks first in the private marked
guest. Compile the read-only collector natively, then run the host-side guarded
driver with temporary pinned keys and `--disposable-fixture`. Fresh state/marker
are required; never point it at an existing board. Save only sanitized results,
verify executed source/installed binary identity, then remove guest/keys/disks/
raw transcripts and loopback forwards. Saved evidence checks do not rerun a VM.

## Actual outcomes

The final [sanitized evidence](results/027-netbsd-memory-scaling.json) matches 41
frozen execution inputs, independently checked against the actual guest's native
source copies and native collector/seed binary hashes. A fresh guest boot separated
the final run from exploratory kernel/cache history. The final corrected guard
recorded **no kernel OOM event or swap use**. Fixture integrity, 1,001 unique account
hashes (sysop plus 1,000 approved accounts), 50 rooms and 100,000 posts passed.
Guest limits were restored, and the guest/keys/disks/raw logs/forwards removed.

| Distinct approved SSH accounts/sessions | Menu + chat sample | Delivery to every chat recipient | Whole-VM anonymous memory at end of chat | Kernel pool pages expressed as MiB |
| --- | --- | --- | --- | --- |
| 0 baseline | Five-second seeded idle sample | — | 35.72 MiB | 37.26 MiB |
| 8 | Ten seconds per phase, passed | 0.321 s | See raw observations | See raw observations |
| 32 | Passed | 0.327 s | See raw observations | See raw observations |
| 64 | Passed | 0.288 s | See raw observations | See raw observations |
| 100 | Passed | 0.272 s | 382.78 MiB | 101.87 MiB |
| 150 | Passed | 0.331 s | 558.36 MiB | 134.61 MiB |

The next offered stage was 200; the corrected guard stopped further admission at
**168 approved menu sessions**. That partial cohort did not complete a 200-session
menu/chat stage. Stopping those 168 active workers through the service path took
**6.504 seconds** and reaped member workers/frontends. A fresh authenticated login
after load passed. Neither 200 nor 250 sessions is a success or support claim.

At 100 chat sessions the guest anonymous-page counter increased about 347.06 MiB
from baseline; at 150 it increased 522.63 MiB. Kernel pool counters also increased
about 64.61/97.36 MiB. These are whole-VM counters/deltas, not exact application
USS/PSS and not quantities to blindly add to RSS or each other. At 150, summed RSS
across mandatory roles exceeded physical RAM because it repeatedly counted shared
pages: member 595.34 MiB, frontend 290.64 MiB, dedicated SSH 1866.56 MiB, broker
10.75 MiB. This is why the earlier RSS projection could not establish capacity.
The whole-VM cumulative CPU estimate was 5.759% / 5.112% of one core in the 100/150
idle-chat intervals; ten-second VM/statclock/process estimates are noisy and not
steady active-workload or latency percentiles. No numeric resource target accepted.

The final synchronized eight-password trial produced **four successes and four
explicit budget rejections**. These are cumulative results: completed derivations
free a slot for later arrivals, so success count need not equal simultaneous work.
The two cross-process kernel slots enforce concurrency; four new Rust security
tests and a real Python-held-lock/PTY test exercise saturation, release, invalid
lock paths, independent boards and stored-cost refusal. Successful workers' summed
lifetime high-water RSS is not simultaneous peak. The remote sampling window missed
the short KDF allocation spike (sampled anonymous memory did not rise above its
pre-submit baseline); do not call it measured authentication peak or use it as an
upper bound. A persistent in-guest high-frequency sampler remains follow-up work.
The intended KDF arrays are bounded at two × 19 MiB; total transient process/SSH/OS
costs remain separately measured work.

Fresh native regression also passed **58 Rust tests, five PTY workflows, seven
local lifecycle oracles, 20 application SSH oracles and 13 service/load/fault
oracles**. Host release fmt/Clippy/build/Rust, five PTY workflows, seven lifecycle,
27 Python tests and dependency/repository/provenance checks passed. Existing 024/
026 evidence was refreshed from the same code. CI checks saved provenance and runs
local Linux regressions; it does not rerun native VM scaling.

### Exploratory failures retained in the record

The first pilot completed 150 and reached 183 approved sessions, then the guest
kernel killed a member worker with `out of swap`. The original guard incorrectly
treated all inactive pages as reclaimable; inactive anonymous pages are not
reclaimable without swap. That was a test-guard defect, not a successful capacity
result. A repeat after that pilot stopped at 112 using the old guard, with only
the 100-session stage complete. Kernel pool/cache history changed headroom after
load. These unversioned exploratory pilots are summarized as discoveries, not
current-source validation. The final version adds pool/clean-file counters,
discounts wired pages conservatively, reboots the guest, freezes source hashes,
and stops at 168 without an OOM. Do not average the pilot results into a supported
member count or compare their different cache histories as controlled candidates.

Run `python3 tools/check_memory_evidence.py` for saved source/oracle/integrity/
cleanup verification; it starts no guest. See [ADR 0014](../decisions/0014-credential-budget-and-scaling-experiments.md)
for the gate/experiment decision and [plan 0014](../plans/0014-memory-scaling.md)
for continuation context.

## Next gates

Keep normal cap eight. Choose an architectural change only after identifying
measured fixed/session/SSH/auth/SQL costs. Compare bounded shared readiness/session
state against process workers if per-user costs or polling scale poorly; keep
authorization, output/failure boundaries and equivalent SSH security in comparison.
No mandatory public helper/blockchain service is justified by this local workload.
