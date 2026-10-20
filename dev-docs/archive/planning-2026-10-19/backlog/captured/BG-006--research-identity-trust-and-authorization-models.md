---
id: BG-006
title: Research identity trust and authorization models
status: captured
created: 2026-07-15
source_type: edd
source_path: docs/transport-erd.md
source_title: Multi-Network and Transport-Agnostic Architecture
area: security
appetite_hint: medium
depends_on: [BG-001, BG-005]
external_dependencies: []
impact: critical
impact_rationale: identity and permissions cross every client, node, and transport boundary
target_outcomes: [DO-01, DO-03]
---

## Problem

The draft requires users and transport-specific identities but does not define account binding, trust, recovery, moderation authority, or privacy boundaries.

## Why Now

Identity choices are expensive to retrofit and determine whether interconnected nodes are credible.

## Desired Outcome

Documented alternatives and threat models cover local accounts, cryptographic identities, transport bindings, role delegation, recovery, and cross-node trust.

User-confirmed on 2026-10-19: registration/login and local sysop approval are mandatory before community access, through either network terminal or browser console. Specify pending/approved/rejected/revoked behavior, approval authority, status feedback, and recovery without public/guest community access. Other-BBS directory listings grant no remote membership; visits remain subject to the destination's approval. Future shared-channel federation needs author context, not replication of local account or approval databases. Evidence and open questions: `.planning/PROJECT.md`, Product Direction Record. Coordinate BG-013, BG-017, and BG-019; remains captured pending milestone intake.

## Source Notes

Transport ERD identifies User and Identity/Address as transport-independent core entities.
