---
id: BG-016
title: Research mesh and store-and-forward operation
status: captured
created: 2026-07-15
source_type: edd
source_path: docs/transport-erd.md
source_title: Multi-Network and Transport-Agnostic Architecture
area: transports
appetite_hint: medium
depends_on: [BG-003, BG-005, BG-007, BG-009]
external_dependencies: [representative mesh test environment]
impact: high
impact_rationale: mesh constraints are central to resilience but easily overcomplicated
target_outcomes: [DO-03, DO-06]
---

## Problem

Mesh and delay-tolerant links have different ordering, availability, payload, and routing characteristics from interactive IP networks.

## Why Now

The project needs explicit criteria for mesh support rather than treating it as an adapter checkbox.

## Desired Outcome

Documented store-and-forward semantics, delivery expectations, and a minimal experiment determine what mesh support should mean.

## Source Notes

Transport ERD names mesh networks and graceful degradation.
