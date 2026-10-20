# ADR 0016: Compare the native allocator before changing transports

Status: experiment choice; production adoption deferred. Date: 2026-10-20.
Issue: [#58](https://github.com/gonz0w/faction-bbs/issues/58).

## Question and constraints

Can lower native allocation retention reduce complete per-transport cost enough
for a guarded 250-member short workload on two cores/about 1 GiB? Keep the Rust
core, base OpenSSH, terminal/owner UID separation, actual native descriptor
handoff, current approval/epoch/grant checks, two credential slots and Argon2.
Normal service defaults stay eight workers. The standalone shared candidate
still requires #55 complete UI/fairness/adoption work.

## Alternatives and decision

| Candidate | Reason to compare / defer |
| --- | --- |
| Native allocator arenas/cache/decay | Small deployment-only experiment; changes no app protocol, dependency or SSH implementation; matched profiles use identical application code. Test CPU/page churn and whole-guest effects. Chosen first. |
| Trim per-worker SQLite caches | Cache ceiling is already 2 MiB and is not a private resident charge. Cannot remove the member/frontends/SSH process duplication seen in research 0029. Retain as fallback comparison. |
| Shared RPC with complete per-member UI workers | Preserves UI code but introduces framing/capability/secret-handling boundaries and still duplicates processes. #55 adoption alternative if shared UI completion/fairness fails. |
| Minimal C/no_std frontend | Could reduce runtime cost but duplicates a sensitive handoff/TTY-restore contract. Require measured residual savings to justify a separately reviewed opt-in implementation. Allocator-only stops at 248 justify a standalone fixed-buffer C handoff candidate; Rust core remains. Separate source/binary and denial/TTY/failure evidence required before adoption. |
| Embedded/replacement SSH or weaker credentials | Changes security/update burden or fails equivalent behavior. No authority from a memory result; do not adopt. |

The comparison variant also resets the send sequence when opening a new chat
lease. A fresh-marker baseline exposed that correctness bug in 055; preserving
its frozen history requires the separate 058 copy. Both allocator profiles use
the same corrected variant; no memory change is attributed to this fix.

Use the native libc already shipped in the signed NetBSD image. Verify its
actual allocator and options with a fail-closed readback helper; do not assume
that a generic manual, similarly named source file or environment variable
proves the active implementation. The ARM64 image exposes jemalloc 5.3.0 via
`__je_mallctl`. The legacy `lib/libc/stdlib/jemalloc.c` exists in the same source
tree but is not this build's selected allocator.

Compare fresh-boot default allocation against a marked-disposable-guest-wide
`/etc/malloc.conf` symlink selecting one arena, no tcache and immediate dirty/
muzzy decay, with retain false. Default/candidate readbacks and source/binary
identities belong in the ledger. This is guest-wide: SSH, administration,
collectors and other newly started services can change too. Baselines and
whole-VM counters must include these effects; no claim of exclusive app savings.

## Tradeoffs, validation and reversal

Disabling caches/reducing arenas may add synchronization, syscalls and page
faults; releasing pages sooner may trade CPU/latency for RAM. A short idle/chat
test cannot characterize active file/game/extension workloads or unrelated host
services. No normal rc.d installer or user-facing setup changes are authorized
by this experiment. Per-service scoping and broader workload validation remain
necessary before deployment adoption.

Match dataset, full real SSH transports, explicit limits, unique approved
accounts, warmup and native code. Capture authenticated-no-channel transport,
handed-off prompt, menu and chat states. Record the actual unchanged guard
decisions and between-check admission effects. Repeat fresh-boot trials;
sample calibrated credential operations while a useful cohort remains in chat;
exercise failures and fresh login. Keep every SSH/environment/forwarding denial,
positive descriptor oracle, private data denial and core current-authority check.
RSS sums and virtual mappings are not USS/PSS; observed sampled peaks are lower
bounds. Rejected or never-offered connections never count as BBS members.

Adopt only through a later focused review after complete #55 behavior and #28
active-mix/soak/resource/lifecycle tests, with maintainable scoped configuration
and rollback. Confidence is limited to the published measurements. Reversal is
cheap while isolated: remove the fixture setting and reboot; normal application
sources, database/schema and service defaults have not changed. If repeated
whole-host results fail margin/latency or broader workloads regress, discard the
profile and measure another bounded alternative instead of loosening the guard.

## Primary sources

Retrieved 2026-10-20: [NetBSD libc allocator selection](https://github.com/NetBSD/src/blob/netbsd-11/lib/libc/stdlib/Makefile.inc),
[NetBSD allocator manual](https://man.netbsd.org/jemalloc.3) (configuration sources,
arena/cache/decay tradeoffs; online version can differ from the image),
[OpenSSH configuration](https://man.openbsd.org/sshd_config) (the original access
restrictions remain). Native version/option readbacks and actual execution,
rather than manual defaults, determine the experiment inputs.

## Bounded next comparison after allocator-only limits

Two default repeats stopped at 224, two compact repeats at 248 under the unchanged
predicate, after completed 200-member workloads. The direct 32-transport prompt
phase adds about 13 MiB anon under the compact profile; it includes channel/TTY
and frontend effects, not exclusive private frontend charge. Measure a fixed-
buffer libc-only C frontend before assuming that cost is irreducible. It keeps
the existing native FD packet, UID/endpoint/command/TTY gates and restoration,
with tighter absolute connect/send deadlines. Core/session authority and crypto
remain Rust/base OpenSSH. The loss of Rust's memory-safety checks adds review
and maintenance cost: explicit bounds/lifetime review, GCC analyzer, hardened
native build and positive/negative real terminal/SSH oracles are prerequisites.
This is an opt-in research candidate with cheap rollback to the saved original
frontend, never a normal installer change or production adoption from RAM alone.

## Recorded result

[Research 0030](../research/0030-transport-memory.md) records two functional
250-member minimal-frontend/compact workloads. One passed the final guard at
144.07 MiB headroom; the repeat finished at 123.90 MiB, 4.10 MiB short. Allocator-
only repeats stopped at 248; defaults stopped at 224 after complete 200-member
stages. Unique-marker repeat chat, real credentials under 198/248 live peers,
denial/failure/recovery and provenance checks pass. #58 remains open for reliable
margin; #55/#28 retain complete UI/fairness and scoped deployment/workload gates.
The candidate is not adopted as the production frontend or service profile.
