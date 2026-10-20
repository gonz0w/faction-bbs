# Concerns and Research Constraints

## Seed-repository limitation

This is a documentation-only seed with no implementation and no VCS repository. Claims about runtime behavior, security posture, dependencies, data schema, performance, deployment, or code quality cannot be verified. The two documents are draft inputs, not approval or design decisions.

## Product and scope risks

- Combining forums, persistent chat, file areas, terminal modes, doors, search, moderation, and self-host operations creates substantial MVP scope pressure.
- The PRD identifies unifying forum and chat data models, terminal/ANSI compatibility, moderation/spam, and scope creep as risks.
- Anti-algorithm and local-first commitments constrain feed/discovery, telemetry, export, and recommendation choices.

## Transport and ERD constraints for later research

The high-level ERD requires transport-independent core entities: **User**, **Community/Node**, **Post/Thread**, **ChatMessage**, **File**, and **Identity/Address**. Users can belong to multiple communities; communities contain posts and chat channels; posts/messages may carry transport metadata for hybrid routing. Address/identity records must represent transport-specific addresses without polluting core models.

Architecture research must evaluate how a relational data model represents:

- stable user identities across networks and per-transport identity/address bindings;
- membership, roles, moderation, reputation, and private messaging;
- threaded posts versus real-time and store-and-forward chat;
- files and attachments independent of delivery transport;
- transport metadata, routing, deduplication, ordering, persistence, and offline delivery;
- search across posts and chat without coupling queries to a transport; and
- export, backup/restore, encryption, retention, and minimal/opt-in telemetry.

## Adapter and operational risks

- The listed transports have materially different reachability, identity, discovery, latency, bandwidth, and reliability models.
- Hybrid support requires explicit consistency, failure, retry, and security boundaries; the seed provides no contract for them.
- I2P/Tor, mesh, packet radio, and future transports are not equivalent implementation targets and should not be bundled into an initial proof of concept.
- The transport document recommends isolating transport code and starting with IP plus Reticulum, but neither choice has been adopted.

## Evidence

- `docs/initial-seed-prd.md` lines 13-18, 42-94, 103-121
- `docs/transport-erd.md` lines 7-19, 21-39, 40-81, 83-107, 121-136
