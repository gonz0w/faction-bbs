# Architecture Map

## Current state

No application architecture is implemented. The repository is a documentation-only seed: `docs/initial-seed-prd.md` is a draft product requirements document and `docs/transport-erd.md` is a draft multi-network design input. There is no product code or VCS repository from which component boundaries or runtime behavior can be verified.

## Proposed architectural direction (not approved)

The transport design describes a layered direction for later research:

1. **Transport-agnostic core application** — users, communities, posts, chat messages, files, reputation, and moderation share business logic and a unified data model.
2. **Transport abstraction** — adapters handle addressing, discovery, routing, persistence, and connection-specific behavior.
3. **Client/front-end layer** — web UI, terminal/ANSI mode, and transport-specific native clients or bridges present a consistent community experience.
4. **Local storage/services** — relational or lightweight local storage and transport-independent file storage support self-hosted operation.

## Constraints to preserve during later architecture research

- The MVP combines asynchronous threaded discussion and persistent real-time chat within communities/nodes.
- The system targets local-first ownership, self-hosting, data export, low/opt-in telemetry, authentication, and applicable encryption.
- Chronological or user-curated feeds are primary; opaque engagement optimization conflicts with the PRD philosophy.
- Hybrid operation must allow more than one enabled transport while avoiding transport-specific pollution of core models.
- Constrained and intermittent networks require graceful degradation, including prioritizing asynchronous use over live chat where necessary.
- IP and Reticulum are stated initial prototype priorities; other transport families are staged, not guaranteed.

## Unresolved work

No adapter contracts, authorization model, event model, storage schema, deployment topology, or client protocol has been selected. The documents explicitly call for research, prototypes, detailed interfaces, and a hybrid-operation proof of concept. This map records inputs only; it is not approval or a design decision.

## Evidence

- `docs/initial-seed-prd.md` lines 7-18, 42-94, 103-121
- `docs/transport-erd.md` lines 7-39, 40-81, 100-136
