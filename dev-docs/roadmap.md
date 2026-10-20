# Roadmap and stage

Updated 2026-10-19. GitHub issue and milestone state is authoritative; see the [issue index](issue-index.md).

## Current stage: research foundation

The repository now has a product baseline, BSD-3-Clause licensing, contribution/security guidance, a Codex-friendly workflow, documentation checks, and an issue backlog. Rust and NetBSD as primary target are selected; no BBS runtime or complete production stack exists.

The [decision framework](research/decision-framework.md) was accepted through merged PR #36, closing issue #1. [Issue #3](https://github.com/gonz0w/faction-bbs/issues/3), merged through PR #37, supplies [proposed journeys and synthetic link/workload controls](research/0003-constrained-user-journeys.md) for modern private networks with occasional degradation. No runtime experiment or production budget is validated by that analysis. [#8 interaction research](research/0008-interaction-budget.md) now proposes small text pages, capabilities and recovery requirements with an application-paced ASCII/ANSI comparison. [#28 resource research](research/0028-small-host-budgets.md) proposes tiny application budgets and a full measurement contract for 100–250 connected members on a 1–2 core / roughly 1 GiB host, plus a bounded C allocation-ownership experiment. Neither establishes secure server performance. The [#4 native comparison](research/0004-runtime-comparison.md) and [#9 SQLite recovery/export evidence](research/0009-local-storage-comparison.md) now support [a proposed Rust/readiness + SQLite validation slice](decisions/0005-rust-sqlite-validation-slice.md), with Go retained in the comparative evidence. The owner subsequently accepted Rust/primary NetBSD in [ADR 0006](decisions/0006-rust-and-primary-netbsd.md). SSH/private-network operation is [accepted product direction](decisions/0004-ssh-and-optional-private-network.md); the production backend and minimal appliance remain open. [Native NetBSD 11 ARM64 evidence](research/0023-netbsd-dependencies.md) now verifies pinned Mio/rusqlite with bundled and explicitly linked pkgsrc SQLite, plus nine synthetic restricted OpenSSH gateway oracles; physical SBCs, real identity and complete session costs remain unvalidated. Next #11/#20/#35 should bound the integrated secure read/post/chat slice, including #23 SSH enforcement and #18 complete software/VM accounting. Physical SBC checks are deferred per the later owner instruction below. #4/#9 remain open for integrated evidence; broad surveys must not block local core building. Bounded optional-extension research continues alongside it. Browser-console feasibility under the strict no-JS policy remains an explicit decision gate.

## Software-first build path and extension research

The owner has no physical SBC test hardware. Developer-host and NetBSD VM tests
drive software building now; physical SBC testing follows a usable complete
product and gates hardware-specific claims only. See [ADR 0007](decisions/0007-software-first-and-extension-direction.md).

Build the [first useful local software slice](first-local-slice.md) under #35:
real local identity/approval, safe terminal login/menu, durable Citadel rooms and
local chat, then SSH/VM operation and bounded load/recovery tests. Do not wait for
every optional network/storage/game comparison before building this slice.

Lua menus/isolated doors are first after that core. The [extension report](research/0010-extensible-ansi-platform.md) records a 16-oracle Rust/Lua boundary probe and
optional modern storage/network/payment directions. The new future issues capture
permanent rooms (#42), payment models (#43), ANSI shops (#44) and game economies/
wagers (#45); existing #5/#10/#15/#16/#17 own generic adapters, Lua and networks.
Compare member external wallets and sysop-run payment services before choosing.
No live provider, publication, financial operation or production sandbox is adopted.

## Proposed build gates

1. **Requirements and budgets:** enough specificity to compare options fairly; core journeys, privacy/access boundaries, terminal capabilities, bandwidth, hardware/resource targets, and minimal deployment needs.
2. **Evidence-backed architecture:** primary-source comparisons and recorded decisions for runtime, persistence, terminal access, domain boundaries, and extension isolation. No JS/TS, no mandatory public web service, no central dependency.
3. **Representative experiments:** one named uncertainty and pass/fail observations per experiment, including approval enforcement, safe ANSI/session behavior, bounded resource use, persistence/reconnect, and optional game handoff.
4. **First useful local BBS:** select a small vertical slice only after the earlier gates. Define its exact features, tests, support targets, and deferred work in a milestone. Do not interpret the menu inventory or all open issues as its scope.

## Later, separately scoped work

Original-game compatibility, broader file/mail features, customization, imports, operational tooling, optional/private HTTP access, subscribed shared-chat federation, Reticulum, and mesh/store-and-forward experiments have different dependencies. Research leads may begin early; product support requires later evidence and explicit acceptance.

Only shared chat may federate. Boards, files, mail, games, directories, local-only channels, and local accounts/approval remain local. A separate optional public-publishing module is under research; it is never an implicit export of private core data. Alternative-network research must not make every deployment run every adapter. No release date or production support commitment is set.
