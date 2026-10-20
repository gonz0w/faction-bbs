---
id: BG-014
title: Design web and richer client layering
status: captured
created: 2026-07-15
source_type: prd
source_path: docs/initial-seed-prd.md
source_title: Product Requirements Document
area: clients
appetite_hint: medium
depends_on: [BG-008, BG-012, BG-013]
external_dependencies: []
impact: medium
impact_rationale: keeps modern clients additive rather than mandatory
target_outcomes: [DO-04]
---

## Problem

The user requires a webpage with an embedded console exposing the terminal-based BBS experience, superseding the seed PRD's web-first framing. Session transport, terminal rendering, input handling, accessibility, and behavior shared with direct terminal access remain unvalidated. Additional richer clients remain optional research candidates.

## Why Now

Client divergence can make compatibility and operations unmanageable.

## Desired Outcome

A layered client strategy identifies shared APIs and capabilities while allowing richer presentation without changing core semantics.

First establish browser-console access to the same room, chat, menu, and game-door actions as a direct terminal. Compare terminal emulation and browser session mechanisms without selecting a library or production stack prematurely. Evaluate character/ANSI rendering, keyboard handling, resize behavior, connection loss, and access on narrow screens, alongside BG-008's bandwidth budget and BG-013's terminal contract.

Further user direction on 2026-10-19 requires the same approved-login gate, BBS identity ANSI art before the menu, sysop-defined welcome content, and local Contacts / Other BBSes navigation in the browser console. Loading the webpage must not bypass approval or expose community content. Compare remote-visit mechanisms while preserving each destination's login/approval boundary; automatic account transfer is not a requirement. See `.planning/PROJECT.md`, Confirmed Session Shape.

## Source Notes

PRD client and usability requirements.

Updated from explicit user direction on 2026-10-19; see `.planning/PROJECT.md`, Product Direction Record. Remains captured pending milestone intake; this update does not schedule browser implementation.
