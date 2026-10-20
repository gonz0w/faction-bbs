---
id: BG-013
title: Design terminal and ANSI client baseline
status: captured
created: 2026-07-15
source_type: prd
source_path: docs/initial-seed-prd.md
source_title: Product Requirements Document
area: clients
appetite_hint: medium
depends_on: [BG-008, BG-012]
external_dependencies: []
impact: high
impact_rationale: validates classic BBS accessibility and constrained-link value
target_outcomes: [DO-04]
---

## Problem

The user confirmed the terminal-based BBS as the primary experience on 2026-10-19, superseding the seed PRD's optional terminal framing. Interaction, accessibility, session, terminal capabilities/encoding, and access protocols still require research.

## Why Now

A text-mode baseline makes bandwidth constraints concrete and protects against web-only assumptions.

## Desired Outcome

A terminal baseline covers navigation, reading, posting, chat degradation, capability levels, escape handling, and compatibility targets.

Cover Citadel-style room discovery/entry/posting, IRC-style chat commands, BBS menus, and game-door entry/return. The user clarified that the direct-access target is a terminal app over a network; physical serial terminals are not a current requirement. Compare access options without equating terminal interaction with a Telnet requirement, and coordinate browser-console equivalence with BG-014 and legacy games with BG-010.

Further user direction on 2026-10-19 requires a WWIV-inspired session: login/registration and local approval before any community access, then BBS-specific ANSI art, then the main menu with sysop-defined broadcast/quote/welcome content. Study relevant standard BBS menu areas, including the local Contacts / Other BBSes directory. Define approval-status screens, ANSI capability fallback/skip behavior, and remote-visit navigation without bypassing destination access controls. See `.planning/PROJECT.md`, Confirmed Session Shape; inventory items are not automatically first-release scope.

## Source Notes

PRD calls for Telnet/SSH/ANSI terminal mode.

Updated from explicit user direction and follow-up answers on 2026-10-19; see `.planning/PROJECT.md`, Product Direction Record. Network terminal access is required; the network access protocol remains unselected. Remains captured pending milestone intake.
