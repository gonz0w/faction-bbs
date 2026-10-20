# 0021: Experimental minimal filesystem and IP boundary

Status: proposed opt-in development profile; production sandbox deferred.
Date: 2026-10-20. Authority: owner requested merging #63 and continuing #55
filesystem/network/exec isolation while preserving 250-member NetBSD headroom.
Related #19/#20/#23/#28.

## Question and constraints

Narrow host filesystem/exec/IP authority without new Rust dependencies, schema,
per-session workers, resident supervisors or weaker credentials/approval. Keep
the maintained UI, default eight, fixed helpers, resource ceilings and two kernel
credential slots shared with owner CLI. Do not silently replace host policy.

## Alternatives and evidence

Base facilities keep the dependency/update surface small. NetBSD 11 NPF documents
user/group TCP/UDP matching; mount flags provide read-only/noexec/nosuid/nodev.
Chroot alone lacks exec/IP policy and privileged callers can escape. An outside
descriptor fexecve entry probe failed; separate nullfs socket aliases also failed.
The resulting pre-main initializer uses one shared view and drops all privileged
IDs before application parsing. A UID-only packet candidate failed a real native
TCP gate; the selected combined dedicated-group/user candidate passed the narrow
native tests. These findings are recorded in [research 0035](../research/0035-os-isolation.md).

PF's NetBSD 11 manual calls it obsolete; do not adopt it for a new profile.
Cells is an interesting actively developed NetBSD 11 isolation system requiring
a patched kernel/control stack; the current secmodel_sandbox project introduces
an external kernel/Lua module. Neither gets silently made a mandatory dependency.
A future native syscall/kauth boundary could cover more address families and
same-UID/process threats; no such coverage is inferred from this profile.

## Decision and tradeoffs

Offer a separately installed **experimental** profile with a small pre-main C
initializer, root-owned identity/config sources, minimal read-only noexec root,
exact writable noexec run/state mounts and explicit NPF group/user candidate.
Keep the normal binaries and service selector available. Profile frontend alone
uses setuid root; root loader/runtime libraries/initializer therefore enter the trusted base.
No Rust main/config/theme/database parsing executes before full identity drop.
The ordinary kernel envelopes and authenticated terminal handoff remain intact.

[The operator contract](../../docs/experimental-isolation.md) describes setup,
root policy, privilege scope, host firewall review and recovery. This is not a
network/process namespace, arbitrary-code sandbox, total memory cap or protection
of board data from the owner. It cannot launch Lua/doors. Root can alter mounts,
policy and processes; live host-policy changes are outside startup guarantees.

## Confidence, validation and reversal

Native software evidence is limited to NetBSD 11 ARM64/2-vCPU/1-GiB with unchanged
full SSH/base-service costs and research host memory settings. Require valid-ELF
exec denials, host/config/DB access separation, privilege/group regain denial,
real live endpoint positive/negative controls, all frontend/UI/lifecycle checks,
cross-view credential/pid locks, every live kernel ceiling and three fresh-boot
250-member guard workloads. Preserve actual executed source bytes and binaries.

Rollback is explicit stopped root selection of the existing normal binaries;
remove profile privileges/mounts and only its selected host settings. No schema
or dependency reversal is needed. Independent privileged-code review, broader
protocol/interface/same-UID/tracing isolation, long/adversarial workloads and
maintained host policy remain required before production adoption.
