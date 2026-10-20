---
id: BG-009
title: Evaluate persistence search and replication options
status: captured
created: 2026-07-15
source_type: edd
source_path: docs/transport-erd.md
source_title: Multi-Network and Transport-Agnostic Architecture
area: data
appetite_hint: medium
depends_on: [BG-001, BG-003, BG-007]
external_dependencies: []
impact: high
impact_rationale: data storage and sync choices shape offline operation and node interconnection
target_outcomes: [DO-01, DO-03]
---

## Problem

The documents name relational or embedded storage but do not address search, attachments, replication, conflicts, retention, or backup under intermittent links.

## Why Now

Topology and transport research needs realistic persistence assumptions.

## Desired Outcome

An evidence-backed data strategy separates local source of truth, indexes, file storage, export, backup, and any future replication mechanism.

User direction on 2026-10-19 restricts cross-BBS federation to participating shared chat channels and necessary delivery context. Boards, files, mail, games, directories, local accounts/approval, and local-only chat stay local. Evaluate local persistence, backups, client recovery, and future shared-chat history within that boundary; do not infer federation of all stored data from replication research. Coordinate BG-017. Provenance: `.planning/PROJECT.md`, Product Direction Record; this remains a captured candidate.

## Source Notes

Derived from PRD storage guidance and transport ERD core entities.
