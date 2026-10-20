---
id: BG-012
title: Define core community and content domain model
status: captured
created: 2026-07-15
source_type: edd
source_path: docs/transport-erd.md
source_title: Multi-Network and Transport-Agnostic Architecture
area: architecture
appetite_hint: medium
depends_on: [BG-005, BG-006, BG-009]
external_dependencies: []
impact: high
impact_rationale: establishes durable semantics for posts, chat, files, communities, and moderation
target_outcomes: [DO-01]
---

## Problem

The ERD names entities but does not resolve boundaries between threaded posts, chat, files, identity, membership, moderation, and message delivery.

## Why Now

This model is the contract that transports and clients must not distort.

## Desired Outcome

A domain model distinguishes stable community semantics from delivery, presentation, and storage implementation details.

Treat Citadel-style rooms as the confirmed organizing model for message boards: find/enter a room, read new messages, and post or reply there. The user clarified "more like citadel, less like reddit"; define room reading/unread behavior and reply presentation accordingly, without inferring voting, ranking, or nested Reddit-style threads. Define rooms' relationship to IRC-style live chat channels without assuming they are the same entity. Global chat initially spans one installation; future optional shared-channel subscriptions across installations belong to BG-017. IRC-client interoperability remains open.

Further user clarification on 2026-10-19 establishes local approved membership, sysop-owned BBS content, a local Contacts / Other BBSes directory accepting system registrations, and explicit local-only versus participating shared chat channels. Only the shared chat channels federate. Directory listings support discovery and visits; they neither replicate remote content nor grant remote membership. Define listing ownership/submission/review lifecycle and channel visibility independently of transport addresses. Coordinate approval with BG-006 and federation with BG-017. Menu/session behavior follows WWIV inspiration while board interaction follows Citadel.

## Source Notes

Transport ERD high-level data model section.

Updated from explicit user direction on 2026-10-19; see `.planning/PROJECT.md`, Product Direction Record. This remains a captured candidate, not accepted milestone scope.
