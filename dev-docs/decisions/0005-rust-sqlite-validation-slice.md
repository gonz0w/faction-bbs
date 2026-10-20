# 0005: Carry Rust, readiness I/O and SQLite into the validation slice

Status: proposed implementation-experiment decision; production adoption not accepted
Date: 2026-10-19. Owner: maintainer review/merge authority.
Issues: [#4](https://github.com/gonz0w/faction-bbs/issues/4), [#9](https://github.com/gonz0w/faction-bbs/issues/9), #11/#18/#20/#23/#31/#35.
Supersedes: none. Evidence: [native comparison](../research/0004-runtime-comparison.md), [storage/recovery](../research/0009-local-storage-comparison.md).

## Proposed decision

Use hosted **Rust with readiness-based I/O and local SQLite WAL** for the next
integrated validation slice. Mio 1.2.4 is the tested reference adapter; the production
event library and Rust SQLite/SSH bindings require separate pinned dependency review.
Keep core domain logic independent of OS/network/session adapters. One local database,
bounded serialized write work and streaming local files avoid extra mandatory services.
Use FULL synchronization, explicit constraints/transactions, request reconciliation,
derived search and versioned record export/verified backups as hypotheses to validate.

Reject millisecond connection scanning as the initial I/O approach because it exceeded
the proposed connected-idle CPU target in the probe. Keep Go as the measured fallback;
it performed well. Retain C/legacy hosts at justified boundaries; no measured hotspot
requires assembly. Zig/Free Pascal/unikernels remain bounded candidates, not equal
first-slice implementation obligations. MINIX is not a mandatory target per owner input.

## Evidence and alternatives

Twenty fresh native processes (five per C/Rust-scan/Go/Rust-readiness configuration)
handled 250 concurrent **local synthetic** connections each. Readiness Rust used about
4.2 MiB sampled RSS with idle CPU below coarse resolution, while scan implementations
used roughly 6–7% of a core idle. Go was comparably responsive with a larger sampled
working set here; it remains viable. No real authentication, TCP/SSH, durable room or
chat fan-out was implemented in those probes.

Ten independent SQLite DBs exercised indexed reads, guarded writes, process-crash
outcomes, contention, snapshots, FTS retention, backup and explicit export/import.
WAL/FULL had lower observed commit time and allowed concurrent reader snapshots on
this developer host. PostgreSQL/custom KV/files have different deployment/query costs;
they were not benchmarked and receive no fabricated score. Generic SQL-dump restoration
failed for our FTS setup; an explicit domain export/search rebuild passed instead.

Use framework priorities with U for full security/OS/durability/portability/extension
costs. No complete numerical ranking is supported. Higher simplicity/stdlib priority
could favor Go; mandatory unsupported OS or hard footprint failure reopens the runtime
choice. See reports for sources, compiler/dependency versions, hashes and tradeoffs.

## Gate screen and acceptance conditions

Scoped probe G1/G9 checks and synthetic local tests support this **experiment choice**;
production G1/G2/G5/G7/G8/G9 remain unresolved. G3/G4/G6 local authority, independence
and domain/transport separation remain mandatory architectural obligations. Scope
excludes any credential, terminal, door or deployment support guarantee. Unknown gates
block production adoption; this ADR does not bypass them.

Before promoting the stack to production acceptance, the next slice must:

1. Integrate audited secure SSH access and local approval checks for read/post/chat,
   including pending/revoked/resumed sessions and negative shell/forwarding tests.
2. Pin/audit selected dependencies and keep app-owned untrusted parsing/state safe;
   justify and isolate any FFI/unsafe boundary, including SQLite/OS/crypto libraries.
3. Meet or explicitly revise #28's **complete application** targets on named SBC/VM
   hardware with 100/250 connections, crypto, DB, slow readers and bounded workers.
4. Demonstrate durable request retry/recovery and export/backup restoration across
   crash/storage-full/read-only/power-loss scenarios appropriate to the actual media;
   validate search/file permissions, quotas and attachment reconciliation.
5. Keep HTTP/overlay/federation/doors disabled without breaking the useful local slice.

Failure triggers a focused repair/recomparison (including Go at equal semantics), not
weaker security or an unsupported performance claim. #4/#9 remain open for this work.

## Confidence, reversal and next step

Moderate confidence in prioritizing this slice; low confidence in complete target
capacity until measured. Low reversal now (disposable experiments), moderate after
application interfaces, high after persisted/deployed formats. Preserve versioned
exports, explicit migrations/restore and small adapter boundaries to make reversal
practical. Review/merge may accept the experiment direction without accepting the
production stack; record any later promotion with the completed gate evidence.
