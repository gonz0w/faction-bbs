---
id: BG-008
title: Research low-bandwidth client protocol and interaction budget
status: captured
created: 2026-07-15
source_type: discussion
source_path: .planning/INTENT.md
source_title: Citadel IRC Project Intent
area: clients
appetite_hint: medium
depends_on: [BG-003]
external_dependencies: []
impact: critical
impact_rationale: establishes a usable baseline across constrained networks
target_outcomes: [DO-04]
---

## Problem

The user confirmed a terminal-based BBS experience on 2026-10-19, but minimum ASCII/ANSI capabilities, character encoding, interaction budgets, and direct-terminal/browser-console compatibility are not yet defined. Optional ANSI games may require capabilities beyond the constrained-link community baseline.

## Why Now

Rendering, protocol, content, attachment, and synchronization choices must follow an explicit bandwidth budget.

## Desired Outcome

A client interaction model specifies minimum text-mode tasks, payload assumptions, caching, pagination, compression, capability negotiation, and richer-client layering.

## Source Notes

Captured from user direction and PRD terminal-mode requirements.

Updated from explicit user direction on 2026-10-19; see `.planning/PROJECT.md`, Product Direction Record. This remains a captured candidate, not accepted milestone scope. Coordinate with BG-013 and BG-014 on terminal parity and with BG-010 on optional game requirements.
