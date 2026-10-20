# Architecture Map

## Current State

No application architecture is implemented. The repository contains draft product and transport inputs only.

## Architectural Direction to Evaluate

The transport document proposes a possible separation of a transport-independent application core, transport adapters, client surfaces, and local storage. This is useful as a research hypothesis, not an approved architecture.

Research must test whether it can preserve common domain semantics for users, communities, posts, chat messages, files, moderation, and identity while isolating addressing, discovery, routing, ordering, offline delivery, and connection behavior. It must also compare standalone, connected, hub-and-spoke, and mesh-oriented topologies rather than presupposing federation or a mothership.
