# Roadmap and stage

Updated 2026-10-19. GitHub issue and milestone state is authoritative; see the [issue index](issue-index.md).

## Current stage: research foundation

The repository now has a product baseline, BSD-3-Clause licensing, contribution/security guidance, a Codex-friendly workflow, documentation checks, and an issue backlog. No BBS runtime or production stack exists.

The [decision framework](research/decision-framework.md) was accepted through merged PR #36, closing issue #1. [Issue #3](https://github.com/gonz0w/faction-bbs/issues/3), merged through PR #37, supplies [proposed journeys and synthetic link/workload controls](research/0003-constrained-user-journeys.md) for modern private networks with occasional degradation. No runtime experiment or production budget is validated by that analysis. [#8 interaction research](research/0008-interaction-budget.md) now proposes small text pages, capabilities and recovery requirements with an application-paced ASCII/ANSI comparison. [#28 resource research](research/0028-small-host-budgets.md) proposes tiny application budgets and a full measurement contract for 100–250 connected members on a 1–2 core / roughly 1 GiB host, plus a bounded C allocation-ownership experiment. Neither establishes secure server performance. The [#4 native comparison](research/0004-runtime-comparison.md) and [#9 SQLite recovery/export evidence](research/0009-local-storage-comparison.md) now support [a proposed Rust/readiness + SQLite validation slice](decisions/0005-rust-sqlite-validation-slice.md), with Go retained as a measured fallback. SSH/private-network operation is [accepted product direction](decisions/0004-ssh-and-optional-private-network.md); no backend or appliance OS is selected. Next #11/#20/#35 should bound the integrated secure read/post/chat slice, including #23 SSH enforcement and #18 whole-SBC/VM accounting. #4/#9 remain open for integrated evidence; broad technology surveys are no longer the next step. Browser-console feasibility under the strict no-JS policy remains an explicit decision gate.

## Proposed build gates

1. **Requirements and budgets:** enough specificity to compare options fairly; core journeys, privacy/access boundaries, terminal capabilities, bandwidth, hardware/resource targets, and minimal deployment needs.
2. **Evidence-backed architecture:** primary-source comparisons and recorded decisions for runtime, persistence, terminal access, domain boundaries, and extension isolation. No JS/TS, no mandatory public web service, no central dependency.
3. **Representative experiments:** one named uncertainty and pass/fail observations per experiment, including approval enforcement, safe ANSI/session behavior, bounded resource use, persistence/reconnect, and optional game handoff.
4. **First useful local BBS:** select a small vertical slice only after the earlier gates. Define its exact features, tests, support targets, and deferred work in a milestone. Do not interpret the menu inventory or all open issues as its scope.

## Later, separately scoped work

Original-game compatibility, broader file/mail features, customization, imports, operational tooling, optional/private HTTP access, subscribed shared-chat federation, Reticulum, and mesh/store-and-forward experiments have different dependencies. Research leads may begin early; product support requires later evidence and explicit acceptance.

Only shared chat may federate. Boards, files, mail, games, directories, local-only channels, and local accounts/approval remain local. Alternative-network research must not make every deployment run every adapter. No release date or production support commitment is set.
