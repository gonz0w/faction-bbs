# 0013: Managed gateway lifecycle with bounded worker supervision

Status: implemented development choice, pending PR review; no production adoption
Date: 2026-10-20
Owner: project maintainer; owner authorized service lifecycle and load/failure work
Issues: [#18](https://github.com/gonz0w/faction-bbs/issues/18), [#20](https://github.com/gonz0w/faction-bbs/issues/20), [#23](https://github.com/gonz0w/faction-bbs/issues/23)
Extends: [ADR 0012](0012-restricted-ssh-and-terminal-handoff.md)
Evidence: [lifecycle report](../research/0026-service-lifecycle-load.md)

## Question and mandatory gates

How should the eight-worker development gateway stop and recover without leaving
active member processes or making a stale socket an unsafe deletion instruction?
Preserve separate OS identities, fixed worker configuration, local approval,
private state, no content logging, no JS/TS, and a small dependency surface.
Production resource, hostile-process isolation, durable failure and deployment
gates remain unknown. This choice does not waive those gates.

| Alternative | Applicable gate result | Tradeoff |
| --- | --- | --- |
| Unmanaged foreground process | Fails ordinary lifecycle requirement | Existing default TERM exits without orderly worker cleanup. |
| Base NetBSD rc.d plus in-process supervisor | Experiment passes access/locality/no-JS gates; production capacity unknown | Reuses native operations; requires a small signal bridge and real process fault tests. |
| New service-manager dependency | Unknown deployment/resource gates | More portable supervision could help later, but adds a mandatory runtime to a hobby host. |
| Raise cap to 100–250 now | Unknown resource/failure gates | Exceeds available evidence; rejected for this increment. |

Qualitative comparison: correctness and bounded resource ownership precede package
count. No weighted score or all-platform winner follows from eight-worker tests.

## Decision and boundary

Use two opt-in root-managed rc.d scripts for the existing broker and dedicated
OpenSSH instance. Broker remains non-root, foreground and fixed-configuration.
Its owned `0600` pidfile also holds an exclusive nonblocking kernel file lock.
Never unlink that lock inode during ordinary service operation. Under its lock,
recover only a backend-owned socket whose connect fails with connection-refused;
refuse other files, symlinks and live sockets. Ordinary removal checks socket
device/inode identity. Parent directories and same-UID owner remain trusted.

TERM/INT/HUP stop accepting, allow five seconds to drain, then request supervisor
teardown and join/reap children before releasing socket/lock. HUP means stop,
not reload. A narrow native C handler writes only a C11 always-lock-free atomic;
Rust polls it. The existing `faction-unix` unsafe exception now includes this
bridge on all build hosts. No new registry packages are introduced. All other
application crates still forbid unsafe Rust. Do not call this project unsafe-free.

Actual member workers receive the expected broker PID through broker-controlled
environment. A 64-KiB-stack monitor compares the kernel parent every 200 ms and
exits on reparenting, including when terminal output blocks the UI thread. This
is failure cleanup, not authentication or confinement. Forced exit can leave a
chat lease until existing expiry; ordinary close remains preferable. Direct
local PTY workers without this environment retain the development path.

Keep the eight-worker/burst limit and five-second token refill. Owner-only
`--session-seconds 1..1800` may shorten the existing 1800-second maximum, including
fault tests; it cannot increase it. No general configuration reload or automatic
crash-loop restart policy is added. Supervisor teardown is cooperative around
kernel waits; uninterruptible I/O/scheduler failure is not a hard realtime promise.

## Evidence, confidence and falsification

Primary references retrieved 2026-10-20:
[rc.subr(8)](https://man.netbsd.org/rc.subr.8),
[rc.d(8)](https://man.netbsd.org/rc.d.8),
[sigaction(2)](https://man.netbsd.org/sigaction.2),
[flock(2)](https://man.netbsd.org/flock.2), and
[getpid(2)](https://man.netbsd.org/getpid.2).
These define interfaces, not evidence that our service works. C's lock-free
atomic property is a compile-time assertion; positive signal/process/SSH tests
provide runtime evidence. A host test caught inherited nonblocking accepted
sockets on Darwin; explicitly restore blocking mode before receive timeouts.

Require positive native login/chat after recovery, denied ninth admission while
eight live workers are observed, stable descriptor counts under overload,
observable worker exit on broker death, and actual blocked-output progress before
testing its deadline. A matching compile or immediately failed connection alone
cannot pass these oracles. See the report for actual outcomes and limits.

Confidence is moderate for tested NetBSD 11 ARM64/software fixtures and low for
unmeasured physical devices, arbitrary ports and whole-host capacity. Reopen if
signals fail to drain, a broker-loss worker survives, a second instance can acquire
the same live endpoint, output stalls retain database writes, or measured overhead
misses the small-host target. Reversal cost is low/moderate: no schema change;
stop services, retain/export state, replace supervisor/scripts and retest. Existing
sessions are lost on restart. Private data need not leave the sysop's machine.

## Remaining work

Whole-host shared/private memory and authentication peaks, 100–250 actual connected
members, long-duration traffic, per-key/account fairness, disk-full/power-loss and
upgrade rollback remain open under #18/#20/#23/#28/#35. User authorization is for
software research/builds; no physical hardware or live public deployment is needed.
