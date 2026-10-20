# ADR 0017: Test extent retention against complete host headroom

Status: native experiment choice validated for the bounded #58 workload; production adoption deferred. Date: 2026-10-20.
Issue: [#58](https://github.com/gonz0w/faction-bbs/issues/58).

## Question and decision

Can a bounded native allocator change leave repeatable useful headroom for 250
members on the existing 2-core/1-GiB NetBSD 11 ARM64 software fixture? Research
0030 completed two short functional workloads, but only one passed the final
128-MiB guard. Keep the exact corrected Rust/backend and hardened handoff-only
frontend, base OpenSSH, owner/terminal UID separation, actual descriptor packet,
approval/epoch/private grants, hidden input and two-slot unchanged Argon2.

Compare compact `retain:false` against `retain:true`. Both retain one arena,
disabled tcache, zero dirty/muzzy decay and all normal OS services. The hypothesis
is less mapping/allocator bookkeeping churn; retained virtual ranges do not by
themselves imply a smaller physical footprint. NetBSD page advice/reclamation
can differ from other platforms, and actual whole-host results decide whether
the candidate is useful. Record effective native options after each fresh boot.
Do not infer an improvement or private charge from virtual maps or summed RSS.

## Alternatives and tradeoffs

| Alternative | Assessment |
| --- | --- |
| Extent retention with existing compact allocator | Small, reversible, dependency-free comparison. Could reduce mapping fragmentation or increase virtual/physical retention. Measure CPU, latency, pool/wired/anon/free and credential peaks. |
| Minimal appliance service profile | Legitimate later deployment comparison, but changes the host's responsibilities. Keep base image services matched initially; never silently drop required services from the budget. |
| Per-process allocator/library replacement | More build, update and sensitive-process compatibility burden; defer until native options are measured. |
| Smaller kernel or custom SSH | Broad portability/security/update work. Keep base OpenSSH and existing signed kernel for this comparison. |
| Smaller guard, weaker Argon2, larger VM or omitted helper costs | Fails equivalent behavior or the stated resource question. Excluded. |

Guest-wide `/etc/malloc.conf` remains a marked private fixture input. It affects
SSH, collectors, owner tools and background services too; no exclusive app
savings claim. Per-service scoping, rollback and full UI/fairness remain #55
adoption work. Normal installers and their eight-worker default stay unchanged.

## Validation and reversal

Use separate driver/helper/results and preserve all historical frozen sources.
Match synthetic 1000 independently salted profiles, 50 rooms, 100000 1-KiB posts,
admission cadence, state warmup and workloads. Capture actual guard samples and
batch effects, every live SSH/frontend/owner soft/hard resource limit through
read-only CTL_PROC, all mandatory process costs and native binary/source hashes.
Repeat fresh boots; real member bad/good replacement credentials at 198/248 live
peers, two fresh owner-console credentials with all 250 remaining in chat, plus
known touched 0/19/38-MiB controls at idle and 250. These synthetic controls and
sampled maxima do not prove an instantaneous cold credential upper bound.

Keep actual SSH denials, native positive descriptor/TTY restoration, current
authorization/private-grant checks, slow-reader cleanup and crash/recovery
oracles. Verify graceful high-load stop/reap, fresh login, SQLite integrity and
removal of private VM inputs/keys/logs/forwards. All whole-VM counters include
administration/instrumentation even when role RSS sums omit administration.
Short workloads cannot pass #28 active mix, soak or authenticated-transport flood
gates; software VM results cannot establish physical SBC/all-NetBSD support.

Reversal while isolated: stop the marked fixture, restore the recorded symlink
setting and reboot. No normal source, schema, service or dependency graph change.
Reject the profile if it does not improve repeated margin or compromises any
behavior. Production adoption needs its own later review and scoped setup.

## Primary evidence

Retrieved 2026-10-20:
[NetBSD allocator manual](https://man.netbsd.org/NetBSD-11.0/jemalloc.3),
[native allocator configuration](https://github.com/NetBSD/src/blob/netbsd-11/external/bsd/jemalloc/dist/src/jemalloc.c),
[native page advice implementation](https://github.com/NetBSD/src/blob/netbsd-11/external/bsd/jemalloc/dist/src/pages.c),
[NetBSD per-process sysctl interface](https://man.netbsd.org/NetBSD-11.0/sysctl.7),
[live resource-limit implementation](https://github.com/NetBSD/src/blob/netbsd-11/sys/kern/kern_resource.c).
These explain interfaces and tradeoffs; effective version/options and executed
VM evidence determine the result. Confidence and measured outcomes follow in
the linked research report when the experiment completes.

## Bounded follow-up: slab geometry

Extent retention shows no material improvement through the first 100-member
observations; finish its matched trial before accepting/rejecting it. The native
allocator also supports `slab_sizes`. Its source chooses default slab lengths
as page/object least common multiples; the update interface can request fewer
pages and clamps to enough pages for one object. Compare
`slab_sizes:1-16384:1` with compact retain-false allocation if the retained trial
does not solve the working-margin problem. Record all effective bin geometries.
This leaves requested allocation sizes and security semantics intact but may
increase fragmentation/allocation churn under larger active workloads. No
production choice follows without repeated whole-host/latency/credential checks.

Additional primary sources, retrieved 2026-10-20:
[native slab sizing/clamping](https://github.com/NetBSD/src/blob/netbsd-11/external/bsd/jemalloc/dist/src/sc.c),
[effective bin controls](https://github.com/NetBSD/src/blob/netbsd-11/external/bsd/jemalloc/dist/src/ctl.c).

## Bounded follow-up: native DSS extent topology

Minimum-slab observations through 100 members show little physical-memory benefit;
smaller virtual slab sizes cannot be counted as resident savings. If that full
trial fails working margin, compare the native allocator's supported `dss:primary`
preference with compact retain-false/default slab allocation. This requests
data-segment extents before mmap extents; native jemalloc supports fallback.
The hypothesis is a smaller extent/mapping/metadata footprint for many small
processes. No object size, base SSH binary, cryptography, UID/access gate or guard
changes. DSS may retain memory or behave poorly with other sbrk users; no general
allocator recommendation follows. Verify the effective preference, default bin
geometry, native denial/failure tests and repeated whole-host credential margin.
Keep this guest-wide fixture-only with the same cheap restore-and-reboot reversal.

Primary source, retrieved 2026-10-20:
[NetBSD's native DSS extent implementation](https://github.com/NetBSD/src/blob/netbsd-11/external/bsd/jemalloc/dist/src/extent_dss.c).

## Host attribution and the pageable time-daemon comparison

The allocator-only alternatives do not solve working margin. A new read-only
native service/map audit found two ntpd processes; one has about 25.6 MiB of
locked virtual mappings, matching the guest's wired counter. Locked virtual
mapping bytes and RSS are still not private physical charges. Test the supported
`rlimit memlock -1` time-daemon configuration with compact/default slabs and
retain false, keeping NTP running and every existing mail/discovery/inetd/logging/
DHCP/cron service present. This is a separate host policy comparison, not an
exclusive per-SSH allocator saving. Record before/after actual locked mappings,
whole-VM counters, config hashes, live limits and all high-load work.

The option explicitly calls munlockall and prevents subsequent default mlockall
in the shipped implementation. It allows the OS to page the time daemon's memory;
it does not disable timekeeping or change SSH/credential memory handling. Measure
zero swap and functional timing now; clock precision under sustained pressure
and maintained appliance policy remain #20/#28 deployment gates. Use only the
marked VM, save the original file, append this one option, reboot and verify
actual state. Restore the original file/hash and allocator profile before final
cleanup. No normal installation or host service configuration is changed.

Primary sources, retrieved 2026-10-20:
[NetBSD NTP configuration implementation](https://github.com/NetBSD/src/blob/netbsd-11/external/bsd/ntp/dist/ntpd/ntp_config.c),
[NetBSD NTP startup locking](https://github.com/NetBSD/src/blob/netbsd-11/external/bsd/ntp/dist/ntpd/ntpd.c),
[NTP memory-lock guidance](https://support.ntp.org/Support/LockingNtpdInMemory),
[NetBSD VM map readback](https://github.com/NetBSD/src/blob/netbsd-11/sys/uvm/uvm_map.c).

## Credential handoff correction before final repetitions

The third preliminary pageable-time run had sufficient memory but one of two
valid replacement logins received `credential work busy`. The zero-length job
channel could reject that second submission before another fixed worker began
waiting. Treat this as a functional failure, preserve it, and do not count it
among completed repetitions or rerun until it disappears.

A separately versioned Rust backend in this experiment replaces that rendezvous
handoff with two owned permits covering queued jobs, active workers and
unconsumed results together. Queue capacity is two and aggregate outstanding
work is also two; disconnecting a client never frees its still-running job's
permit. A delivered/discarded result or failed handoff releases the permit by
ownership. Two fixed workers and the unchanged two cross-process kernel Argon2
slots remain; excess work still gets the same bounded busy response. The short handoff queue is bounded by the same two aggregate permits; member
input buffers retain their existing bounds and no authority-import API is added. The deterministic lifetime test
accepts two jobs before any receiver waits, rejects a third through work/results,
and verifies release and disconnected/shutdown paths.

Run a new matched original-NTP baseline and three fresh pageable-time repetitions
with this exact fixed backend, then all native regressions. Preserve all eight
preliminary records and their old backend identity, including the failed third
attempt. The new source/dependency inventory and isolated target add no registry
package or production/default change. Full UI/fairness adoption remains #55.

## Recorded decision and confidence

[Research 0031](../research/0031-repeatable-headroom.md) / results 032 retain
eight exploratory and four final matched workloads. With the corrected two-permit
Rust dispatcher, the original locked-time baseline completes 250 but reaches
98.22 MiB sampled stress headroom. Three fresh-boot pageable-time repetitions
complete every functional and recorded guard check: 180.68–182.69 MiB final
headroom, 137.23 MiB lowest sampled stress headroom, unchanged 128-MiB predicate.
All base services remain running; no swap devices were configured and no OOM
occurred. Original time config/default allocator/kernel limits and full process/
data integrity are verified before removal of the private VM and credentials.

Continue compact allocation, the bounded dispatcher and the pageable-time policy
as an experimental software candidate for #55. Do not select retain/minimum-slab/
DSS changes from these unsuccessful working-margin comparisons. Normal eight-
worker services, schemas and dependency graph remain unchanged. Native 58
workspace tests plus the dispatch lifetime regression, terminal/SSH/current-
authority/private-grant/descriptor/lifecycle/fault suites pass with the winning
profile. Confidence is limited to the recorded short VM workloads and sampled
observations; peaks remain lower bounds, not continuous reserve guarantees.

#55 owns full UI/fairness, frontend review and scoped setup; #20 clock precision/
updates/rollback and #28 active mixes/soak/budgets/authenticated-transport limits
remain adoption gates. Keep Lua/isolated doors next after the useful core.
