# 0014: Profile memory and scale connected sessions

Issues: [#28](https://github.com/gonz0w/faction-bbs/issues/28), [#20](https://github.com/gonz0w/faction-bbs/issues/20), [#18](https://github.com/gonz0w/faction-bbs/issues/18), [#23](https://github.com/gonz0w/faction-bbs/issues/23).
Status: implementation/native validation complete; [PR #54](https://github.com/gonz0w/faction-bbs/pull/54) open for review. #53 merged at `c9abca9`.

## Goal and constraints

Measure complete application/SSH and whole-VM memory on NetBSD 11 ARM64,
2 vCPUs/about 1 GiB; bound credential transients and explore staged distinct-account
SSH loads toward 100/250. Default remains eight workers until measured adoption.
Explicit experimental limits are operator-only. No weaker Argon2, privileged
member entry point, mandatory external service, hardware gate or production claim.

## Progress

- [x] Merge verified passing #53; read resource/validation requirements.
- [x] Add per-board cross-process credential budget and meaningful regression tests.
- [x] Add guarded opt-in scaling, profiler and reproducible distinct-account load.
- [x] Measure fresh native baseline, authentication peaks, idle/chat and recovery.
- [x] Record findings/decision, update docs/backlog and publish focused PR #54.

## Method and stopping conditions

Use official signed/hash-verified isolated NetBSD VM, loopback host forwards and
separate temporary admin/member keys. NetBSD pmap/kinfo_vmentry do not expose
resident private/share counts; do not label virtual mappings USS/PSS. Collect
kernel whole-VM anon/file/wired/free/inactive counters, process resident/high-water
and CPU times, and full dedicated-SSHD descendants. Report raw counters/deltas
and limitations. Staged independent SSH sessions use distinct approved accounts;
offer higher stages only with memory/process headroom and abort safely on failures.
No rejected offers are counted as connected users. Preserve lifecycle/approval
oracles, delete private guest/keys/disks/logs and publish only sanitized evidence.

## Discoveries and outcomes

2026-10-20: pmap documents that resident lookup is beyond its scope. Kinfo VM
entries expose mapping/ref counts, not page residency/share multiplicity. Use
whole-system accounting rather than invent per-process USS/PSS.
Final guarded fresh-boot run passed distinct-account stages 8/32/64/100/150;
stopped admission at 168 for headroom, no OOM/swap, 168-worker service stop 6.504 s
and fresh login passed. 150-recipient delivery 0.331 s. Seed/integrity: 1,000 approved
accounts plus sysop, 1,001 unique hashes, 50 rooms, 100,000 posts. Native regressions:
58 Rust, five PTY, seven lifecycle, 20 SSH and 13 service/fault oracles passed.
Host Rust/Clippy/five PTYs/seven lifecycle, 27 Python and dependency checks passed.
41 execution-source hashes match and were checked against native source copies;
guest/keys/disks/raw logs/forwards removed, host SSH/other VMs untouched.

Earlier pilot reached 183 and kernel-killed a worker: the old guard wrongly
counted inactive anonymous pages as reclaimable. Corrected guard uses free plus
clean file pages minus wired; final run did not OOM. Kernel pool/cache history
also changes repeat headroom. Remote auth sampling missed the short KDF spike;
four cumulative successes/four busy rejections do not imply four concurrent KDFs.
Two locks bound KDF arrays, but persistent in-guest peak sampling remains next.
Full #28 workload/repeat/soak/hardware and appliance acceptance remain open.

## Review and continuation

Branch `codex/28-memory-scaling`, PR #54. Require current-head Linux CI before
merge; native evidence is saved, not rerun by CI. Fresh profiling inputs are
frozen in results/027; do not refresh their hashes without a matching execution.
Default service remains eight workers; higher limits are owner-only experiments.

Next bounded comparisons are [#55](https://github.com/gonz0w/faction-bbs/issues/55)
(shared board/session ownership versus per-member workers) and
[#56](https://github.com/gonz0w/faction-bbs/issues/56) (persistent in-guest credential
peak sampling). Both preserve SSH/approval, local data and the two-slot budget.
Do not interpret 168 admitted menu sessions as a passed 200-user stage or raise
production defaults from a short 150-session menu/chat run. Pilot OOM and remote
sampling gaps stay visible. Full active mix/repeat/soak/fairness/storage/appliance
and later physical-device gates remain open. Obsolete temporary builders can
clobber newer work; use current checked-in sources and this plan.
