---
id: BG-007
title: Compare node topology and interconnection models
status: captured
created: 2026-07-15
source_type: discussion
source_path: .planning/INTENT.md
source_title: Citadel IRC Project Intent
area: architecture
appetite_hint: medium
depends_on: [BG-001, BG-003, BG-005, BG-006]
external_dependencies: []
impact: critical
impact_rationale: prevents accidental dependency on a mothership or premature federation
target_outcomes: [DO-03]
---

## Problem

Standalone, peer-connected, hub-and-spoke, and mesh-capable deployments have different consistency, moderation, discovery, and operations costs.

## Why Now

The user explicitly wants to keep all topology options open while avoiding unnecessary complexity.

## Desired Outcome

A comparison recommends a minimum standalone deployment and clear triggers for node links, relay services, synchronization, or federation.

The user's 2026-10-19 clarification sets shared/global chat within one installation as the initial scope and optional federation for systems subscribing to shared chat channels as future work. Compare topology options for that future requirement with BG-017 while preserving independent operation and avoiding a mandatory central service. No federation protocol or first-milestone implementation is selected.

The user's further 2026-10-19 clarification restricts federation to participating shared chat channels. Other BBS data remains local, including the contacts directory; local-only chat channels are required. Evaluate topology alternatives within that scope. Cross-BBS directory visits are separate connections subject to each destination's approval, not content federation. See `.planning/PROJECT.md`, Confirmed Session Shape.

## Source Notes

Captured from project bootstrap discussion.

Updated from user follow-up answers on 2026-10-19; see `.planning/PROJECT.md`, Product Direction Record. Remains captured pending milestone intake.

## No-Gos

- Requiring a central service for ordinary independent operation.
