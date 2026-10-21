# 0023: Sustained 250-member workload and bounded abuse

Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55), related #28/#19/#20/#23.
Status: implementation/native/host validation complete; review and current CI state are recorded on GitHub. #65 merged at `b672abf`.

## Goal and declared experiment

Exercise the unchanged selected service, isolation and key-admission binaries on
NetBSD 11 ARM64, 2 vCPU / configured 1 GiB, through real SSH and distinct approved
accounts. Preserve all existing measured source/result bytes. Normal defaults
remain eight; explicit research settings, two credential permits/kernel slots,
Argon2, base SSH and process ceilings stay unchanged.

Run the complete prior 32/100/200/250 menu/chat/credential/UI/control matrix, with
an additional **15-minute active interval at 250** before its final controls.
Target one rotating-sender chat broadcast per second, checking delivery to every
connected chat peer; every 30 seconds exercise rooms/read/post/menu recovery and
every 60 seconds replace one member through a fresh real login. Record actual
rates, latency distribution, CPU, full SSH/frontend/base-service counts and
whole-host memory, never inferred throughput. Clear private client transcripts
between rounds to avoid unbounded driver memory.

During the interval, exercise a bounded silent-preauth cohort (up to eight plus
16 excess offers; 30-second grace), continued chat under that pressure and native
source-penalty effects on a fresh login from the same synthetic source. Do not
reset penalty history to manufacture recovery. Separately test repeated unsigned
known-key queries with a research-only client and a fixed request/time ceiling.
A capacity reservation is not signature authentication or BBS approval.

Use the unchanged sampled predicate: free + max(0, clean_file - wired) >=128 MiB,
zero swap, processes+64<maxproc, open_files+128<maxfiles. Stop/record on functional
or guard failure; preserve failed matrices. Sampling is not a continuous reserve.
A 15-minute software interval is not a long-duration soak or supported capacity.
Prefer two fresh boots for replication; record all outcomes, never select only
successful boots. Calibrated controls remain 0/19/38 MiB, including after sustained
activity. No additional resident server/dependency or automatic host tuning.

## Steps

- [x] Verify exact-head checks/reviews and merge #65.
- [x] Add bounded active/abuse drivers and evidence validation/mutation tests.
- [x] Provision signed disposable NetBSD VM; native build/tests and primary-source review.
- [x] Execute and preserve complete workload/abuse trials with original settings.
- [x] Restore/read back host settings, integrity-check fixture, remove private VM material.
- [x] Update README/operator/research docs and canonical backlog; checks and focused PR.

## Discoveries

The prior UI oracles intentionally leave synthetic rows: first-trial pre-load
context is 1,002 accounts, 52 rooms and 100,004 non-load posts. Keep both trials
and retained posts across reboot; report the actual fixture context instead of
pretending the seed description is an identical per-trial snapshot. A new bounded
reset variant recognizes only the additional synthetic workload posts, because
the older immutable reset correctly refuses their unknown prefix/greater count.

The first active interval completed 851 all-member broadcasts, 23 UI/post cycles
and 11 real fresh logins; all sampled guards passed (154.10 MiB active minimum,
134.33 MiB post-active stress minimum, zero swap). SQL integrity/counts were
recorded before the second fresh boot, retaining 23 new workload posts. The
second replication completed with every original guard and recovery check intact.
Aggregate statistical CPU and process runtime accounting differ; a separate fixed CPU diagnostic runs only after both campaigns.
Keep CPU-budget acceptance open, independently of memory/functional passes.
Final count audit separates 425,500 console-marker observations from 423,798
confirmed non-sender receipts: a sender match could be its own draft echo. Preserve
the actual native fields and record this interpretation rather than rewriting them.

## Continuation and scope

Only synthetic fixtures and sanitized counters/hashes are public. Real keys,
passwords, addresses and transcripts stay private and are removed. Any amplified
lookup, denial/fairness issue, memory failure or stale driver assumption must be
reported with its actual scope. Existing historical evidence stays authoritative.
Independent privileged-code review, complete resource budgets, wider protocols,
maintained host/image policy and multi-hour/day soak remain separate gates.
