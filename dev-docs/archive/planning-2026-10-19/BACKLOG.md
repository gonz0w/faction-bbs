# Citadel IRC Backlog

> Durable future-milestone candidates only. This index is a convenience projection; individual files in `backlog/captured/` are the candidate records. No item here is accepted roadmap scope.

## Operating Model

1. Capture a specific future candidate with `/pd-new-backlog <description>` or the shared `BG-###--short-kebab-title.md` schema.
2. Check this index, captured items, and active roadmap context for overlap before adding it.
3. Keep candidates draft-safe: problem, why now, desired outcome, provenance, dependencies, and open questions are mandatory evidence, not commitments.
4. Use `/pd-check-backlog` to triage and refresh priority, then `/pd-promote-backlog <item-ref>` or `/pd-new-milestone backlog <item-ref>` only when ready.
5. Move promoted or discarded items to `backlog/archive/`; retain their identifier and provenance.

## Current Candidates

### Research Foundations

- BG-001: Define architecture decision framework
- BG-002: Research modern BBS and community-platform landscape
- BG-003: Research constrained-network user journeys
- BG-004: Evaluate core language and runtime candidates
- BG-005: Define transport adapter contract
- BG-006: Research identity, trust, and authorization models
- BG-007: Compare node topology and interconnection models
- BG-008: Research low-bandwidth client protocol and interaction budget
- BG-009: Evaluate persistence, search, and replication options
- BG-010: Evaluate customization and extension models

### Validation and Product Capabilities

- BG-011: Build architecture decision prototype program
- BG-012: Define core community and content domain model
- BG-013: Design terminal and ANSI client baseline
- BG-014: Design web and richer client layering
- BG-015: Research Reticulum integration feasibility
- BG-016: Research mesh and store-and-forward operation
- BG-017: Define federation and node-linking boundaries
- BG-018: Design sysop operations and deployment model
- BG-019: Design security, privacy, and data portability baseline
- BG-020: Define test and interoperability strategy

## Intake Notes

Items BG-001 through BG-010 are likely prerequisites for the first research milestone. BG-011 through BG-020 are dependent research or later design/prototype candidates and must not be scheduled merely because they appear in this index.

2026-10-19: Refined existing candidates BG-008, BG-010, BG-012, BG-013, and BG-014 to reflect the user-confirmed terminal/BBS experience, Citadel-style message-board rooms, IRC-style live chat, embedded browser console, and reuse-first legacy game doors. See `.planning/PROJECT.md`, Product Direction Record, for provenance, open questions, research leads, and proposed validation. No candidates added or promoted; no milestone activated.

Same-day follow-up: BG-007 and BG-017 now explicitly capture future optional federation for installations subscribing to shared chat channels. Initial global chat remains within one installation; network terminal apps are the direct-access target, and room interaction follows Citadel. This is requested future scope pending milestone intake, not an activated federation milestone.

Further 2026-10-19 clarification: aligned BG-006, BG-007, BG-009, BG-012, BG-013, BG-014, BG-017, BG-018, and BG-019 with the WWIV-inspired approved-login/ANSI-art/menu sequence, local sysop controls, a local Contacts / Other BBSes directory, and federation restricted to shared chat channels. Local-only chat and all other BBS data remain local. Directory work fits BG-012/BG-018 rather than a duplicate candidate. Citadel IRC remains a working name. No candidates added or promoted and no milestone activated.
