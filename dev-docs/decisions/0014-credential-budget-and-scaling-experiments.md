# 0014: Bound credential memory before scaling SSH sessions

Status: implemented development choice, pending review; experimental capacity only
Date: 2026-10-20
Owner: maintainer; owner authorized memory profiling/scaling toward 100–250 users
Issues: [#28](https://github.com/gonz0w/faction-bbs/issues/28), [#20](https://github.com/gonz0w/faction-bbs/issues/20), [#23](https://github.com/gonz0w/faction-bbs/issues/23)
Extends: [ADR 0013](0013-managed-gateway-lifecycle.md)
Evidence: [memory/scaling report](../research/0027-memory-scaling.md)

## Question and mandatory gates

How can we profile higher session counts without making the memory-hard password
step an unbounded admission cost? Preserve approved local membership, owner-only
operations, Argon2id v19 m=19456 KiB/t=2/p=1, native NetBSD, local data and no-JS.
Production throughput, memory budgets, fairness and hostile-worker confinement
remain gates, not assumptions that an experiment may waive.

| Alternative | Gate outcome | Cost/uncertainty |
| --- | --- | --- |
| Raise worker count without credential budgeting | Fails bounded-work gate | Up to 250 simultaneous 19-MiB derivations exceeds a 1-GiB hobby host. |
| Lower Argon2 memory/work | Fails equivalent-security gate | Rejected; small BBS footprint does not justify weaker credential storage. |
| Budget only in the SSH broker | Incomplete gate coverage | Local registration/bootstrap/recovery still derive passwords outside broker control. |
| Two kernel file locks per board around every credential KDF | Passes tested locality/dependency/bounded-work gates | Simple cross-process reuse; advisory under trusted owner, no FIFO fairness or hostile-code sandbox. |
| Central credential/session service | Unknown integrated/resource gates | May improve future scaling; adds IPC, identity mapping and lifecycle work before comparison. |

No weighted score is justified. Choose the small bounded step and profile the
existing stack before undertaking a session-service rewrite.

## Decision, tradeoffs and reversal

Each board permits at most two simultaneous password derivations across cooperating
owner/member processes. Acquisition tries two protected `0600` owner-only regular
single-link files with CLOEXEC/NOFOLLOW and nonblocking exclusive flock. No waiting
queue or database transaction is held during KDF work. Excess work returns
`credential work busy; reconnect later`. Drop/process exit releases the kernel
lock; files remain on disk and must never be unlinked while the board is live.
This bounds the intended Argon2 work arrays to about 38 MiB per board, not total
process/host peak. Multiple independent boards have separate budgets.

Authentication rejects noncanonical stored algorithm/version/cost/salt/output
parameters before attempting the gate or verifier. This aligns login with existing
snapshot policy and prevents a corrupted/imported cost from requesting arbitrary
KDF memory or silently weaker work. Local owner/root and arbitrary same-UID code
remain trusted; these locks are not an OS enforcement boundary against them.

Retain eight normal workers, launch burst eight/refill one per five seconds, and
existing session maximum. Add **owner-only experimental** `--experimental-workers
1..250` and `--experimental-launch-ms 100..5000` for a marked controlled profiling
fixture; burst remains at most eight and KDF slots stay at two. Operators must
account for process/file/TTY limits and whole SSH/OS memory before using higher
limits. No production adoption or automatic sysctl tuning is included.

Reversal cost low/moderate: no schema or registry dependency change. Stop all board
processes before deleting obsolete lock files or replacing the budget mechanism;
snapshot data excludes those coordination files. Restored boards create fresh
locks. A central service would require new IPC/security tests, not just swapping
this gate. Higher limits can be removed from operator config without data migration.

## Sources and confidence

Primary references retrieved 2026-10-20:
[NetBSD flock(2)](https://man.netbsd.org/flock.2),
[kinfo_getvmmap(3)](https://man.netbsd.org/kinfo_getvmmap.3),
[pmap(1)](https://man.netbsd.org/pmap.1),
[vmstat(1)](https://man.netbsd.org/vmstat.1),
[NetBSD 11 sysctl interfaces](https://github.com/NetBSD/src/blob/netbsd-11/sys/sys/sysctl.h),
[NetBSD 11 ps unit conversion](https://github.com/NetBSD/src/blob/netbsd-11/bin/ps/print.c).
Pinned Argon2 0.6.0 source confirms verification reads encoded PHC parameters;
existing snapshot policy and new positive/negative tests guide the strict check.
Kernel headers shipped in the actual guest and native comparison with ps verify
collector layout/units. File counts use KERN_FILE2, not nonexistent `kern.nfiles`.

NetBSD VM mapping refcounts/virtual sizes do not provide per-page resident sharing.
Do not manufacture USS/PSS. Use separately labeled process RSS sums, resident high
water, dedicated SSH process ancestry and whole-VM page/CPU deltas; acknowledge
collector/admin/cache effects. Mapping counts and reference counts cannot turn
RSS into an exact private-memory metric.

Confidence is high for owner requirements/unchanged KDF parameters, moderate for
tested software paths and low for complete application budgets/production capacity.
The report records actual native results; interfaces alone are not runtime proof.

## Falsification and remaining work

Require held slots to block real cross-process login, released slots to permit
registration, invalid paths/costs to fail closed, independent board budgets, and
native full application/lifecycle regression. Scaling stages must prove distinct
approved accounts, live workers/frontends, message delivery and fresh login after
load. Stop on memory/process/file headroom or functional failure, and keep failed
stages visible. No rejected offers count as connected users.

Reopen on gate bypass, stuck slots after exit, unsupported native locking, transient
memory above its measured allocation ledger, or session/SSH costs exceeding targets.
Remaining: representative action mix, repeat/long soak, exact shared-page/private
attribution, login fairness, output policy, deployment/upgrade/physical-SBC gates.
