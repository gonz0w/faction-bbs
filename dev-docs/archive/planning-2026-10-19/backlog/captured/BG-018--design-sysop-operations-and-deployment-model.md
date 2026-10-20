---
id: BG-018
title: Design sysop operations and deployment model
status: captured
created: 2026-07-15
source_type: prd
source_path: docs/initial-seed-prd.md
source_title: Product Requirements Document
area: operations
appetite_hint: medium
depends_on: [BG-004, BG-007, BG-009, BG-010]
external_dependencies: []
impact: high
impact_rationale: self-hosting must be viable for ordinary sysops
target_outcomes: [DO-01, DO-05]
---

## Problem

Easy deployment, backup, transport configuration, customization, and upgrades are goals without an operating model.

## Why Now

Operational complexity can invalidate otherwise sound technical choices.

## Desired Outcome

A deployment model defines the minimum local installation, configuration layers, transport enablement, upgrade path, backup/restore, and failure recovery.

User direction on 2026-10-19 requires sysop controls for local membership approval, BBS identity/ANSI art, menu and welcome/broadcast/quote content, the features and games hosted locally, other-BBS directory submissions, and local-only versus future shared chat-channel participation. Define permissions and lifecycle for these controls with BG-006/BG-010; assess relevant standard BBS menu functions rather than treating the full menu inventory as initial scope. Each sysop's content remains local except participating shared chat. Provenance: `.planning/PROJECT.md`, Confirmed Session Shape and Product Direction Record; remains captured pending milestone intake.

## Source Notes

PRD self-hosting requirements and transport setup principles.
