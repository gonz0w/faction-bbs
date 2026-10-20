# Roadmap and stage

Updated 2026-10-20. GitHub issue and milestone state is authoritative; see the [issue index](issue-index.md).

## Current stage: first local implementation

The repository now has a product baseline, BSD-3-Clause licensing, contribution/security guidance, a Codex-friendly workflow, documentation checks, and an issue backlog. Rust and NetBSD as primary target are selected. The first [local core/console](../docs/local-development.md) now exercises real accounts, approval/revocation and durable chronological rooms. A [local ANSI/theme/menu baseline](../docs/theme-packs.md) now supports both navigation routes, bounded identity import and manual seasons. [Local command chat](../docs/local-chat.md) now exchanges messages between simultaneous consoles with private-channel grants and bounded history. [Private snapshot/restore](../docs/export-restore.md) now covers current durable state with no-overwrite publication and fresh sessions. [Restricted SSH](../docs/restricted-ssh.md) and [native NetBSD application validation](research/0024-restricted-ssh-netbsd.md) now pass a bounded real-member slice. Managed [NetBSD lifecycle and eight-session load/failure checks](research/0026-service-lifecycle-load.md) now pass. Complete resource/capacity, durable failure and maintained appliance work remain next; no complete production stack or network release exists.

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

Memory/scaling now proceeds under [plan 0014](plans/0014-memory-scaling.md): bounded
credential work, native whole-VM/process/SSH metrics, guarded distinct-account
experiments and measured architecture follow-up. Default service limits and the
full #28 workload/soak/production gates remain separate from short profiling runs.

The next small-host increment compares shared board/readiness ownership (#55)
and calibrated persistent credential peaks (#56); see [ADR 0015](decisions/0015-shared-board-readiness-experiment.md)
and the [comparison](research/0029-shared-board-and-credential-peaks.md). Keep the
full member UI and base OpenSSH approval/isolation behavior while deciding how
to reduce remaining per-transport cost. An isolated prototype is not adoption;
250-user capacity and the broader #28 workload/soak gates remain unvalidated.

[#58](https://github.com/gonz0w/faction-bbs/issues/58) isolates remaining
per-transport costs after the shared-owner comparison; it preserves base OpenSSH,
current approval/credential/terminal denials and all mandatory process accounting.

The [native allocator/transport follow-up](research/0030-transport-memory.md)
now completes functional 250-member SSH menu/chat and high-load credential
checks in two fresh-boot 1-GiB NetBSD VM trials; only one retained the final guard
margin. #58 remains open for repeatable margin. Rust/base OpenSSH remain; a versioned
standalone backend fixes new-chat-lease sequencing. This is not adopted capacity.
Next: #55 complete shared UI/fairness and scoped allocator configuration, with
#28 active-mix/soak/host-service/admission budgets before deployment adoption.
Repeatable 250-member headroom remains #58. Normal gateway defaults remain eight; Lua/isolated doors follow the useful core.
