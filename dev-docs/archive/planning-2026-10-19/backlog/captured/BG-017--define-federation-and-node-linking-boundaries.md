---
id: BG-017
title: Define federation and node-linking boundaries
status: captured
created: 2026-07-15
source_type: discussion
source_path: .planning/INTENT.md
source_title: Citadel IRC Project Intent
area: architecture
appetite_hint: medium
depends_on: [BG-006, BG-007, BG-009, BG-012]
external_dependencies: []
impact: high
impact_rationale: prevents federation from becoming an implicit early requirement
target_outcomes: [DO-03]
---

## Problem

The user explicitly wants future federation in which independent systems subscribe to shared live chat channels. Initial global chat is limited to one installation. Federation is restricted to participating shared chat channels; all other BBS content stays local, and BBS-local chat channels must be possible. No policy yet defines shared-channel ownership, subscription approval, author context, moderation, history, or delivery across linked installations.

## Why Now

Cross-node behavior affects identity, moderation, export, abuse handling, and conflict resolution.

## Desired Outcome

A boundary document names the minimum independent node and the conditions that justify each progressively more connected model.

Define optional subscription-based chat-channel federation as a requested future capability. Compare channel ownership/discovery, subscription/unsubscription, publication permissions, cross-node author context, moderation authority, history sharing, duplicate/loop prevention, reconnect behavior, and disconnected operation. Board-room sharing and full community replication are outside the confirmed product scope. Local-only chat, boards, files, mail, games, directories, accounts/approval, and other BBS content must remain local. Independent nodes must remain useful without federation; no central topology or protocol is predetermined.

## Source Notes

Captured from user questions about mothership and node interconnection.

Updated from explicit user follow-up in this project's chat on 2026-10-19: "for now its one installation" and "federation across other systems who subscribe to those chat channels in the roadmap." See `.planning/PROJECT.md`, Product Direction Record, and `.planning/INTENT.md` C-08. This records the requested future scope; it remains captured until milestone intake schedules and promotes it.

Further user clarification on 2026-10-19: "the only thing that is federate is the global chat groups people participate in"; everything else stays local, and some chat channels may be private to the BBS. A contacts-directory listing and a visit to another BBS establish neither federation nor membership. Preserve each destination's local approval authority. See `.planning/INTENT.md` C-09/C-10.

## Open Questions

- Who owns shared channels, and who approves systems subscribing or publishing to them?
- How are user identities and moderation decisions represented across installations?
- What history is shared with a new subscriber, and how are reconnects, duplicates, and gaps handled?
- Which topology and delivery guarantees fit the measured network and operational constraints?

## Proposed Validation

After the local chat baseline is validated and federation research is taken into a milestone, use two installations and one shared channel to test subscription, permitted message exchange, unsubscribe, reconnect without duplication, and enforcement of the selected moderation policy. Confirm each installation's local chat still works while the other is unavailable. Set delivery/history pass criteria before building the experiment.

Also demonstrate that local-only chat and all other BBS data remain local, and that registering another BBS in the directory neither subscribes to its channels nor grants a user access there.
