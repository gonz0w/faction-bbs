# Architecture: boundaries to validate

Status: Rust/primary NetBSD direction accepted; early local core implemented, no accepted complete production stack. Product requirements are in [docs/project.md](../docs/project.md).

Separate community semantics from transport-specific addressing, discovery, routing, and connections. Candidate responsibilities are local accounts/approval, message rooms/unread state, chat/channel policy, files/mail/directory data, session/menu presentation, persistence/search, transport adapters, operations, and optional game/extension hosts. This is a research map, not a prescribed code/module layout.

## Non-negotiable boundaries

- The server enforces approved membership for every community action on each access surface.
- A node operates independently with no public HTTP service, shared-channel federation, central discovery, or alternative transport requirement.
- Only explicitly participating shared chat may cross BBS boundaries. Local-only chat and all other BBS data stay local; necessary remote author/delivery metadata does not imply federated accounts.
- Directory listings are local discovery records, not membership grants or federation subscriptions.
- Doors are optional and isolated from the core, with explicit resources, privileges, lifecycle, and return/recovery behavior.
- Text/ASCII remains useful; ANSI capabilities and legacy encodings need a compatibility contract.
- No JS/TS in source or dependencies, including the optional browser console and repository tooling.

## Current direction and proposed validation slice

[SSH with optional private-network-only operation](decisions/0004-ssh-and-optional-private-network.md) is accepted product intent; its production backend and complete enforcement remain unvalidated. A synthetic NetBSD/OpenSSH fixture now has narrow negative/approval evidence. [ADR 0005](decisions/0005-rust-sqlite-validation-slice.md) proposes hosted Rust/readiness I/O and local SQLite WAL for the next integrated validation slice, based on native-session and storage/recovery probes. It rejects millisecond scanning as the initial I/O approach and keeps Go as a measured fallback. [ADR 0006](decisions/0006-rust-and-primary-netbsd.md) accepts Rust and NetBSD as primary target. NetBSD 11 ARM64 native compatibility is measured; a complete production stack, appliance and supported device list remain open.

[ADR 0007](decisions/0007-software-first-and-extension-direction.md) establishes software/VM validation now, local core first and Lua menus/isolated doors next. [Core scope](first-local-slice.md) and [extension research](research/0010-extensible-ansi-platform.md) describe bounded host/helper responsibilities. Deliberate public publication, payment intent/signing/settlement and entitlements are separate proposed capabilities, not automatic private-data federation or selected APIs.

## First implementation boundary

[ADR 0008](decisions/0008-local-core-and-credentials.md) introduces `faction-core`
(local accounts/current authorization and SQLite room actions) and a separate local
`faction` CLI. [ADR 0009](decisions/0009-safe-terminal-menus-and-themes.md) adds
`faction-terminal` for validated presentation/action IDs with no account, store or
network authority; its local TTY adapter maps both navigation routes to core checks. [Runnable instructions and limits](../docs/local-development.md)
distinguish development behavior from product requirements. No transport receives
owner recovery authority; the current CLI is not a remote adapter.

[ADR 0010](decisions/0010-bounded-local-chat.md) adds checked local channels, grants,
lease-scoped joins, bounded history and explicit schema 1 -> 2 migration. The local
TTY adapter polls owned pages without holding a DB transaction during output;
server event notification/output limits remain separate integrated work.

## Open decisions

With Rust and primary NetBSD selected, compare local persistence/search, command/session representation, secure terminal access, optional HTTP exposure, no-JS browser-console feasibility, file transfers, room reply semantics, IRC interoperability, extension mechanisms, and later shared-channel/mesh delivery against measurable constraints. Each material choice needs a source-attributed decision record and a falsifiable validation plan. See [the issue index](issue-index.md).

The development schema is version 2 under ADR 0010, with a transactional migration from the initial ADR 0008 schema; it has no stable on-disk compatibility promise yet. No federation protocol, transport adapter contract, framework, production performance guarantee or supported device list is adopted by this document. Select fast/lightweight options on measured evidence, including deployment burden and security, rather than language familiarity.

[ADR 0011](decisions/0011-private-export-restore.md) adds owner-only current-format
SQLite export/verify and new-directory restore. It preserves durable local data,
discards chat leases/joins/receipts and rotates internal session binding. Private
paths/no-overwrite publication are checked; external theme packaging, member-scoped
interchange, protected archives and native NetBSD/power-loss validation remain work.
These APIs must not be delegated by a future member transport adapter.

[ADR 0012](decisions/0012-restricted-ssh-and-terminal-handoff.md) adds the restricted
OpenSSH frontend under a database-inaccessible OS identity, a protected local
terminal handoff and trusted owner worker. Transport admission stays separate from
BBS identity/password/approval. Owner commands remain in the local console only.
A small native NetBSD CMSG bridge has an explicit C/FFI ownership boundary; all
other application crates forbid unsafe Rust. Default cap is eight workers; complete
host/load/failure and hostile-code confinement remain required before production.

[ADR 0013](decisions/0013-managed-gateway-lifecycle.md) adds opt-in NetBSD rc.d,
locked single-instance ownership, signal draining, member parent monitoring and
bounded native load/failure evidence. The narrow native helper also implements
a lock-free shutdown flag; no new registry dependency or schema is added. Eight
workers remain the development cap. Process/SSH RSS sums are not private memory
or evidence for 100–250 connected members.

[ADR 0014](decisions/0014-credential-budget-and-scaling-experiments.md) bounds all
cooperating per-board credential derivations with two protected kernel locks and
rejects unsupported encoded costs before KDF work. Normal admission remains eight;
owner-only experimental limits support guarded profiling. A fresh NetBSD study
passed 150 distinct-account SSH menu/chat sessions and stopped admission at 168.
Whole-VM anonymous/kernel-pool growth motivates a bounded shared-board/readiness
comparison before 250-user adoption; RSS sums are not exact private memory.

[ADR 0015](decisions/0015-shared-board-readiness-experiment.md) tests a standalone
readiness terminal/shared-board owner against workers, retaining base OpenSSH,
the separate terminal UID, current core sessions/approval/grants and the credential
budget. Connection generations are internal; no member identity/owner RPC exists.
The [native comparison](research/0029-shared-board-and-credential-peaks.md) and
calibrated persistent credential sampler provide the next evidence. Complete
themes/plain/resize/UI parity, fairness, full workloads and adoption remain gates;
the normal broker/member path and eight-worker default are unchanged.

[ADR 0016](decisions/0016-native-allocator-comparison.md) records a native
allocator comparison before another transport/runtime rewrite. Its versioned
shared backend resets chat sequencing for each new lease; both profiles use the
same code. [Two short functional 250-member VM workloads (one final guard pass, one 4.10-MiB shortfall)](research/0030-transport-memory.md)
include all SSH/frontends and high-load real credentials. Guest-wide allocator
tuning is experimental; normal services stay at eight workers. Complete #55
behavior/fairness and #28 scoped deployment/full workloads remain required.

The [repeatable headroom study](research/0031-repeatable-headroom.md) now leaves
measured margin at 250 useful SSH members across three fresh boots by making
time-daemon memory pageable in the marked fixture, with all base services
retained. This is a fixed-host policy result with complete live-limit and
credential accounting. Normal eight-worker gateway and production architecture
adoption remain unchanged; #55/#20/#28 own the remaining build/deployment gates.
