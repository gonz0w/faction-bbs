---
id: BG-005
title: Define transport adapter contract
status: captured
created: 2026-07-15
source_type: edd
source_path: docs/transport-erd.md
source_title: Multi-Network and Transport-Agnostic Architecture
area: architecture
appetite_hint: medium
depends_on: [BG-001, BG-003]
external_dependencies: []
impact: critical
impact_rationale: preserves core-domain independence from transport mechanics
target_outcomes: [DO-01, DO-03]
---

## Problem

The transport document names an adapter pattern but does not define delivery semantics, capability discovery, failure, ordering, or identity boundaries.

## Why Now

Without a contract, candidate transport prototypes will drive incompatible core models.

## Desired Outcome

A capability-oriented adapter contract distinguishes required core semantics from optional transport capabilities and explicit degradation paths.

## Source Notes

Transport design proposes common send, subscribe, discover, and connect operations.

## Open Questions

- Which guarantees belong in the core versus the adapter?
- How are store-and-forward and interactive delivery represented without false uniformity?
