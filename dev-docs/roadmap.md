# Roadmap and stage

Updated 2026-10-19. GitHub issue and milestone state is authoritative; see the [issue index](issue-index.md).

## Current stage: research foundation

The repository now has a product baseline, BSD-3-Clause licensing, contribution/security guidance, a Codex-friendly workflow, documentation checks, and an issue backlog. No BBS runtime or production stack exists.

The next research milestone is Architecture & validation: define measurable constraints and use small experiments to retire the riskiest decisions before feature implementation. Start with decision criteria, a complete approved-login/room/chat journey, privacy and threat boundaries, terminal/link/resource budgets, and language/runtime comparison. Browser-console feasibility under the strict no-JS policy is an explicit decision gate.

## Proposed build gates

1. **Requirements and budgets:** enough specificity to compare options fairly; core journeys, privacy/access boundaries, terminal capabilities, bandwidth, hardware/resource targets, and minimal deployment needs.
2. **Evidence-backed architecture:** primary-source comparisons and recorded decisions for runtime, persistence, terminal access, domain boundaries, and extension isolation. No JS/TS, no mandatory public web service, no central dependency.
3. **Representative experiments:** one named uncertainty and pass/fail observations per experiment, including approval enforcement, safe ANSI/session behavior, bounded resource use, persistence/reconnect, and optional game handoff.
4. **First useful local BBS:** select a small vertical slice only after the earlier gates. Define its exact features, tests, support targets, and deferred work in a milestone. Do not interpret the menu inventory or all open issues as its scope.

## Later, separately scoped work

Original-game compatibility, broader file/mail features, customization, imports, operational tooling, optional/private HTTP access, subscribed shared-chat federation, Reticulum, and mesh/store-and-forward experiments have different dependencies. Research leads may begin early; product support requires later evidence and explicit acceptance.

Only shared chat may federate. Boards, files, mail, games, directories, local-only channels, and local accounts/approval remain local. Alternative-network research must not make every deployment run every adapter. No release date or production support commitment is set.
