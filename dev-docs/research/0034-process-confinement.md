# 0034: Native process ceilings and authenticated terminal handoff

Date: 2026-10-20. Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55),
related #19/#20/#23/#28. Status: implementation and native software validation;
complete production confinement and capacity remain unaccepted.

## Question and implementation

Can the maintained shared service narrow frontend/process authority without
spending the headroom retained for 250 approved NetBSD members? The implementation
adds kernel peer UID checks on both ends of the terminal socket, inherited
identity/FD/environment hygiene and irreversible resource ceilings. The Rust
core/UI, database schema, credential costs, two permits/slots, rotating work,
encoded output deadline and default eight-session admission remain unchanged.
No new registry dependency, per-member Rust process or thread is introduced.

The [operator contract](../../docs/process-confinement.md) gives exact values:
zero core/locked-memory/process limits; eight frontend files/no additional LWPs;
service `2 * max_sessions + 48` files/three additional LWPs. Limits are applied to
both soft/hard values and read back, never raised. The service creates its fixed
handoff and two credential threads first. The private owner CLI remains a separate
invocation and is not launched by members. NetBSD's process/LWP accounting is
per UID; limits are inherited process settings. The native deny probes test the
actual result rather than assuming a numerical cap has the intended effect.

The native C bridge returns kernel credentials from real Unix sockets. The frontend
verifies the bound service UID before sending the FD. The service verifies the
accepted terminal UID before receiving it. Path ownership remains checked;
correctly chowned metadata cannot disguise the wrong kernel peer. Credentials
describe bind/connect time, not executable identity or future UID changes.
Core BBS authentication/approval remains required after terminal admission.

The frontend requires the same member-owned real terminal on stdin/stdout/stderr,
closes inherited descriptors, copies only the validated bounded TERM hint, then
clears environment and moves cwd to `/`. Completion is one status byte followed
by EOF; malformed/trailing bytes and missing EOF fail with terminal restoration.
The shared rc.d launch supplies a minimal environment before Rust starts. Its
startup refuses elevated/mismatched identities, extra supplementary groups and
inherited descriptors above stderr before opening board state. No safe Rust API
closes Rust-owned descriptors or unsafely mutates global environment storage.

## Alternatives, scope and primary sources

This bounds resource growth and authenticates the local endpoint; it does not
filter filesystem, networking or exec syscalls, erase environment memory, undo
pre-main dynamic-loader effects, confine same-UID hostile code or sandbox the
trusted database owner. SSH root configuration and the UID split remain trusted.
An exec can still replace a process despite fork being denied. Existing worker
rollback keeps its original backend behavior while using the hardened frontend.

NetBSD base chroot is a viable future filesystem candidate, but needs privileged
setup plus a maintained shell/runtime/library/device layout for SSH/frontend and
separate owner state. It does not solve all syscall/network policy. Global
curtain/PaX changes affect the host; no such policy is silently installed. A
third-party sandbox kernel module needs release/port/update validation before
adoption. RLIMIT_RSS is not an enforced UVM RAM budget; an arbitrary virtual-memory
cap could break the current allocator/SQLite/Argon2 behavior. Neither substitutes
for complete measured accounting or the unchanged host guard. CPU/file-size
policies for a long-lived board require separate workload/growth decisions.

Primary references, retrieved 2026-10-20:

- [NetBSD 11 setrlimit(2)](https://man.netbsd.org/NetBSD-11.0/setrlimit.2): limits,
  hard-limit increases, process/additional-LWP accounting and RSS limitations.
- [NetBSD unix(4)](https://man.netbsd.org/NetBSD-5.0/unix.4): kernel local peer IDs;
  older manual semantics were checked against actual NetBSD 11 native calls and
  cross-UID endpoint probes rather than taken as current-port validation.
- [NetBSD chroot(2)](https://man.netbsd.org/chroot.2) and
  [chroot(8)](https://man.netbsd.org/chroot.8): privileged root setup and identity
  changes; the fetched current manual is not a new NetBSD 11 application test.
- [NetBSD 11 sysctl(7)](https://man.netbsd.org/NetBSD-11.0/sysctl.7) and
  [ktrace(2)](https://man.netbsd.org/NetBSD-11.0/ktrace.2): host visibility/tracing
  policy has a broader scope than this process envelope.
- Maintained [native bridge](../../crates/faction-unix/src/process_security.c),
  [frontend](../../native/faction-tty/main.c),
  [oracles](../../experiments/055-confinement/README.md) and
  [results 035](results/035-netbsd-confinement.json): actual source and execution.

## Native checks and provenance

Merged #62 at `627a8241bf736db2371938641d8bef88785d374a`. Before changing its
measured bytes, preserved the 89 executed build/probe inputs plus license (90 files) in
[the second source snapshot](../evidence/README.md). The first 103-file snapshot
and all nine historical JSON result files remain byte-identical. Multi-revision
resolution selects the requested path/digest, detects snapshot tampering and
cannot substitute historical bytes for current source validation. The new result
pins current transferred sources and installed/built binaries directly.

The signed official NetBSD 11 ARM64 image/kernel and SHA-512 manifest were
verified afresh using the official signing fingerprint. HTTPS key retrieval is
not independent web-of-trust authentication. Fixture: two vCPUs/1 GiB configured,
about 1006 MiB physical/972 MiB UVM, no swap. This is a software VM, not a physical
SBC or all-port compatibility claim. Native compiler/Rust/Python/OpenSSH identities
are recorded. The root workspace passes 63 native Rust tests. The frontend passes
native GCC analyzer and PIE/RELRO/immediate-binding/no-RWE LOAD checks.

Thirty-one real frontend oracles pass, including wrong kernel peer before any FD
send, inherited FD closure, elevated IDs/extra groups, mixed/foreign TTYs and strict
completion. Five actual installed-service startup checks pass, requiring denied
authority before DB/listen with unchanged board digest and a positive clean rc.d
launch. The C process probe passes under dropped identities, verifies exact hard
and soft caps, denies fork/extra thread/mlock/FD growth/raising limits and crashes
with SIGABRT without a core file in a writable directory.

Twenty-nine shared UI/authority/failure checks pass through the real service,
including wrong root/owner peers sending otherwise valid terminal-owned FDs.
Eleven installed lifecycle/configuration/default-eight/rollback checks pass;
existing 20 worker SSH and 13 lifecycle/load/failure checks also pass. Console,
chat, export/restore, member, credential-budget and broker PTY checks pass.
These checks used the stock host profile before scaling. A worker regression
teardown stopped the dedicated SSH listener; restarting that required dependency
resolved the first shared lifecycle attempt. A temporary diagnostic frontend was
restored before all counted final checks. Neither interrupted setup nor a different
binary is counted as a successful campaign.

## Three fresh-boot 250-member trials

All three actual installed-service runs complete at 250 distinct approved members
on separate boots. Every stage/stress sample passes the unchanged guard, all nine
live-limit readbacks per run preserve the exact owner/frontend ceilings, loaded
stop reaps all application frontends/workers and a fresh login succeeds. No old
measurement is substituted for a new observation.

| Fresh boot | Final headroom (MiB) | Lowest stress headroom (MiB) | Owner final-menu RSS (MiB) | Loaded stop (s) |
| --- | ---: | ---: | ---: | ---: |
| 1 | 183.25 | 137.61 | 5.39 | 1.338 |
| 2 | 184.23 | 139.45 | 5.39 | 1.603 |
| 3 | 184.33 | 139.63 | 5.39 | 1.625 |

The lowest stress margin is 137.61 MiB, only 9.61 MiB above the guard.
This retains experimental headroom, not a statistically established improvement
or production guarantee. Full SSH/frontend costs and base services remain in
whole-VM accounting; owner-only RSS is not the complete application budget.

Each run uses the explicit compact allocator/pageable-NTP research profile from
[0031](0031-repeatable-headroom.md), retaining NTP/syslog/DHCP/cron/inetd/Postfix/mDNS.
Ordinary setup never changes these policies. The synthetic fixture has 1000
independently salted approved accounts plus sysop, 50 rooms and 100000 1-KiB posts.
Real distinct SSH members progress through 32/100/200/250 menu/chat stages, member
and owner credentials, selected room/post/chat work at 250, 0/19/38-MiB controls,
loaded stop and fresh-login recovery. Each stage records full native owner/frontend/
SSH roles and all twelve resource soft/hard limits. New validation checks every
measured owner/frontend retains its exact five confinement ceilings.

The guard remains `free + max(0, clean_file - wired) >= 128 MiB`, zero used swap,
64-process reserve and 128-file reserve. Samples/batched admission are observations,
not a continuous production enforcement guarantee. Summed process RSS repeats
shared mappings; it is not USS/PSS. Complete small-memory budget acceptance,
worst-case active/theme/output mixes, soak, authenticated transport admission,
maintained host/timekeeping policy and stronger isolation remain open. Lua and
isolated doors require a separate constrained launcher and explicit capabilities.

## Restoration, validation and handoff

After the campaigns, restored the original NTP configuration byte-for-byte,
removed only the known experimental allocator symlink and rebooted. Readbacks
confirm stock allocator (eight arenas/tcache on), locked NTP again, original
sysctls and retained base services. Offline synthetic reset/integrity confirms
1001 unique account hashes, 50 rooms and 100000 posts. Persistent credential
locks are never unlinked live. Installed native frontend/startup/resource probes
were repeated before capture to resolve the temporary diagnostic-build concern.
The private VM, disks, generated credentials, keys, raw logs and forwarded ports
are removed; only sanitized complete counters/source/binary identities remain.

Both historical source snapshots build independently; nine old record hashes
remain unchanged. Rust fmt/Clippy/workspace tests, dependency policies, repository
checks, historical/new evidence checks and 42 Python tests pass. The new ceiling
corruption test checks every role/resource/soft-hard pair; current-source drift
cannot be masked by a historical snapshot. CI reruns saved evidence validation,
not native timing. #55/#19/#20/#23/#28 retain the broader adoption/security gates
listed above. [ADR 0020](../decisions/0020-native-process-envelope.md) records the
decision and reversal cost.
