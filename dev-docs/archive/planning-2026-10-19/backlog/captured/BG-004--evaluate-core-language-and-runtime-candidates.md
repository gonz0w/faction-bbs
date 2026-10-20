---
id: BG-004
title: Evaluate core language and runtime candidates
status: captured
created: 2026-07-15
source_type: prd
source_path: docs/initial-seed-prd.md
source_title: Product Requirements Document
area: architecture
appetite_hint: medium
depends_on: [BG-001, BG-003]
external_dependencies: []
impact: critical
impact_rationale: the core runtime affects deployment, concurrency, portability, and extension boundaries
target_outcomes: [DO-02]
---

## Problem

The PRD lists Elixir, Go, and Node as examples but provides no decision criteria or evidence.

## Why Now

Language assumptions leak into every prototype and can make later comparisons unfair.

## Desired Outcome

A comparative, criteria-driven evaluation includes at least operational simplicity, constrained-link behavior, concurrency, library maturity, portability, security, and extension isolation.

## Source Notes

PRD technology guidelines are explicitly non-binding examples.

## No-Gos

- Picking a winner without a small representative prototype.
