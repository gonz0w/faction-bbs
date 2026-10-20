# 0020: Native process envelope and authenticated terminal peers

Status: proposed for development-service merge; production confinement deferred.
Date: 2026-10-20. Authority: owner requested #62 merge and continued #55 confinement,
frontend security and 250-member NetBSD headroom work. Related #19/#20/#23/#28.

## Question and constraints

Reduce authority and resource growth at the maintained shared-service/native
frontend boundary without per-member Rust workers, new registry dependencies,
weaker SSH/Argon2, schema changes or implicit host tuning. Keep default eight,
current core authorization, bounded UI and two credential permits/slots. This is
an incremental boundary, not adoption of a complete sandbox or supported capacity.

## Alternatives and evidence

Application-only caps leave fork, extra threads, descriptor growth and core dumps
available after compromise. Kernel soft/hard ceilings plus peer authentication
reduce those specific avenues at small fixed cost. The native probes and actual
installed-service attacks are primary executed evidence in [research 0034](../research/0034-process-confinement.md)
and [results 035](../research/results/035-netbsd-confinement.json).

NetBSD [setrlimit(2)](https://man.netbsd.org/NetBSD-11.0/setrlimit.2), retrieved
2026-10-20, documents the hard-limit rule, per-user process/LWP accounting and
unimplemented RSS enforcement. [chroot(2)](https://man.netbsd.org/chroot.2) and
[chroot(8)](https://man.netbsd.org/chroot.8), retrieved 2026-10-20, describe a
possible future filesystem boundary requiring a privileged setup step and a
maintained runtime/device layout. Chroot alone does not supply networking/exec
policy or replace the current UID split. A global curtain/PaX change would alter
host policy; no such change is installed. A third-party sandbox module without
the same release/port/maintenance evidence is not adopted here.

An arbitrary address-space cap risks breaking native allocator/SQLite/Argon2
behavior without establishing a measured total RAM budget. Keep sampled complete
host accounting and explicit headroom controls; do not claim RLIMIT_RSS is a
memory sandbox. Broker CPU/file-size lifetimes remain uncapped because persistent
service CPU and legitimate board growth need a separate maintained policy.

## Decision and tradeoffs

Use the narrow existing C/FFI boundary for portable kernel peer UID lookup and
irreversible process ceilings. The Rust service checks inherited identity/FD
authority before board open, launches its fixed three helper threads, then sets
limits before serving terminals. The pure C frontend owns and sanitizes its FD
table/environment before applying its smaller envelope. The rc.d launcher passes
a minimal environment before Rust starts. Safe Rust APIs never close Rust-owned
descriptors or mutate process environment behind concurrent safe readers.

Authenticate both local socket peers before descriptor handoff, in addition to
pathname protections. Require matching owned stdin/stdout/stderr terminals and
an exact bounded completion protocol. Keep current approval/session/epoch/grant
checks; kernel UID admission is not BBS identity or approval.

[The operator contract](../../docs/process-confinement.md) names every ceiling and
limitation. No filesystem/network/exec filtering, same-UID adversary isolation,
setuid loader defense or chroot is claimed. Existing workers remain rollback;
the native frontend retains its envelope in worker mode, while the old worker
backend has its original process behavior. Shared owner failure still affects all
members. Future extension launch is a separate boundary, never a reason to weaken
member-process NPROC/NTHR limits.

## Reversal, confidence and validation

Rollback requires root-installed prior binaries and deliberate backend selection,
not raising hard limits in a live process. No database migration or dependency
rollback is needed. Kernel interfaces remain in the reviewed native bridge;
non-NetBSD builds check peers on their host but do not establish NetBSD port
coverage. Confidence applies to the recorded NetBSD 11 ARM64 software fixture.

Required falsifiable evidence: actual startup authority denials before DB/listen;
wrong kernel peers with otherwise valid path/TTY; malformed control/TTY cases;
native fork/thread/mlock/file/core/raise-limit denials; unchanged UI/SSH/rollback;
live soft/hard readbacks on every measured owner/frontend; three fresh boots at
250 with unchanged headroom and complete SSH costs; restored host settings,
fixture integrity and private material cleanup. A current source edit must
invalidate new evidence, even when a historical copy exists. Stronger confinement,
hostile authenticated transport admission, soak and appliance policy stay open.
