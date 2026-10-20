---
id: BG-019
title: Design security privacy and data portability baseline
status: captured
created: 2026-07-15
source_type: prd
source_path: docs/initial-seed-prd.md
source_title: Product Requirements Document
area: security
appetite_hint: medium
depends_on: [BG-006, BG-009, BG-018]
external_dependencies: []
impact: critical
impact_rationale: local ownership requires credible data protection and recovery guarantees
target_outcomes: [DO-01]
---

## Problem

The PRD requires authentication, encryption where applicable, export, backups, and minimal telemetry but lacks threat boundaries and operational guarantees.

## Why Now

Security and portability must influence identity, storage, extension, and deployment decisions from the start.

## Desired Outcome

A baseline defines data classes, trust boundaries, encryption expectations, logs and telemetry policy, retention, export, backup/restore, and incident-recovery assumptions.

User-confirmed on 2026-10-19: enforce authentication and local sysop approval before community access on every terminal/browser entry path. Define pending/rejected/revoked access and recovery with BG-006. Directory submissions and visits must not grant access or bypass the destination's local approval. Future federation may exchange participating shared-channel chat and necessary author/delivery context only; local-only chat, account/approval data, and all other BBS content remain local. Include these boundaries in future validation with BG-017/BG-020. Provenance: `.planning/PROJECT.md`, Product Direction Record; remains captured pending milestone intake.

## Source Notes

PRD security, privacy, operations, and data-ownership goals.
