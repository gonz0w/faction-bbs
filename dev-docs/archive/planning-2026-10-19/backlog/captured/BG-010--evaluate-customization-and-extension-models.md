---
id: BG-010
title: Evaluate customization and extension models
status: captured
created: 2026-07-15
source_type: discussion
source_path: .planning/INTENT.md
source_title: Citadel IRC Project Intent
area: extensibility
appetite_hint: medium
depends_on: [BG-001, BG-004, BG-006]
external_dependencies: []
impact: high
impact_rationale: sysop customization is core value but unbounded scripting can compromise safety and upgrades
target_outcomes: [DO-05]
---

## Problem

Configuration, themes, declarative workflows, scripts, plugins, doors, and external integrations have different security, portability, and maintenance tradeoffs.

## Why Now

The extension boundary must shape the core without committing to an embedded scripting runtime prematurely.

## Desired Outcome

A staged model identifies what is configurable first, what requires extension APIs, and what evidence would justify scripting or in-process plugins.

Include a reuse-first door-game evaluation: identify available original titles (TradeWars is the user's named example), check their distribution and runtime requirements, and compare unchanged binaries/servers, isolated compatibility hosting, and possible future ports. Define session handoff, identity/dropfile requirements where applicable, shared game state, disconnect behavior, and return to the BBS. Keep game runtimes optional and isolated from core community behavior; no universal compatibility or initial porting commitment is implied.

## Source Notes

Captured from PRD extensibility goals and user discussion.

Updated from explicit user direction on 2026-10-19; see `.planning/PROJECT.md`, Product Direction Record, for primary research leads and validation questions. Coordinate menu/terminal behavior with BG-013 and browser-console behavior with BG-014. Remains captured pending milestone intake.

## Open Questions

- Which original titles should define representative compatibility tests after TradeWars?
- Which binaries, server versions, source, and permissions are available for each candidate title?
- What hosting requirements, communication interfaces, concurrency, and recovery behavior does each game impose?
- When would an isolated compatibility host be preferable to a future port?

## No-Gos

- Executing untrusted extensions in the core process by default.
