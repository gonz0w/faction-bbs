# 0006: Rust with NetBSD as the primary target

Status: **accepted owner direction**; no production readiness/support guarantee.
Date: 2026-10-19. Authority: explicit owner instruction in the project conversation.
Issues: #4/#18/#23/#31; related #9/#11/#20/#28/#35.
Supersedes: ADR 0005's provisional language choice and OS-open state only;
its integrated validation gates and unaccepted production-stack status remain.
Evidence: [NetBSD dependency research](../research/0023-netbsd-dependencies.md),
[native comparison](../research/0004-runtime-comparison.md).

## Decision and constraints

Use **Rust** for Faction's application and **NetBSD as the primary OS target**.
Start concrete validation with NetBSD 11.0 on aarch64, covering the SBC/VM interest.
This selects engineering direction, not support for every NetBSD port, Pi model,
Rust tier, old CPU or toolchain. Record compatibility per named target/device.
Other targets may follow when demand and evidence justify their cost.

The owner prefers BSD, hobby hardware, small working sets, privacy and scalable
native systems. Prior comparative evidence favored readiness I/O over idle scans;
the owner explicitly retained Rust despite narrower coverage than NetBSD's C ports.
No JS/TS, mandatory HTTP/central service or implicit local-data federation is allowed.

Mio 1.2.4 plus rusqlite 0.40.1/libsqlite3-sys 0.38.2 is a **verified compatibility
candidate** in the named NetBSD VM. Prefer bundled SQLite for the next probe to
make runtime selection explicit; compare pkgsrc's update/footprint benefits during
appliance work. Base OpenSSH is the measured restricted-gateway comparator,
not yet the production SSH backend. Keep networking, SSH, persistence and domain
authorization interfaces separable.

## Alternatives, tradeoffs and confidence

C would reach more NetBSD machines and has a small runtime, but increases manual
memory-safety burden in hostile parsers/state. Assembly remains justified only by
an observed hotspot; it would fragment portability and does not solve SSH/DB costs.
Go performed well in earlier local probes but does not meet the owner's selected
Rust direction. Minimal Linux/FreeBSD or a dedicated appliance remain comparative
deployment evidence, not equal first-target obligations. MINIX was considered,
never required. Unverified legacy architectures must not weaken access controls.

Native Rust/Mio, bundled and explicitly linked pkgsrc SQLite, and nine restricted
SSH gateway oracles passed on NetBSD 11 ARM64, two vCPUs/1 GiB QEMU VM. The probe
also caught a hypervisor boot issue and SQLite runtime-path ambiguity. These
findings support this first target while showing why target execution is required.

High confidence in this pinned compatibility path; complete resource/security/
hardware confidence remains low until tested. Base OpenSSH's process model and
account mapping may be too costly for our complete tiny-host budgets. Embedded
SSH remains a source-screened, unbuilt alternative. Neither a successful key login
nor optional private-network admission substitutes for current local BBS approval.

## Acceptance gates and reversal

Retain ADR 0005's integrated security, authorization, dependency, capacity,
durability and disabled-optional-component gates. Before a support statement,
build/test on named SBC hardware and a supported toolchain, measure the complete
application at 100/250 members, validate real member/bootstrap/revocation and
listener/forwarding/terminal/file boundaries, and exercise update/backup/restore.
Do not infer application allowance from a 1-GiB host or today's stock disk image.

Reversal is inexpensive now (research probes); adapter boundaries, explicit
runtime dependencies and versioned exports keep it manageable later. Reopen a
named choice if a required device cannot sustain a maintained toolchain, safe
dependencies, whole-application budget or practical sysop updates. Narrow a target
or compare an alternative explicitly; never silently drop safety to claim a port.

Primary sources, retrieval dates, exact versions, limitations and the falsifiable
next experiments are in the linked research report. User acceptance selects the
direction; passing evidence determines what can truthfully be called supported.
