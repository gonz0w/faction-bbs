# Architecture: boundaries to validate

Status: no production implementation or accepted production stack. Product requirements are in [docs/project.md](../docs/project.md).

Separate community semantics from transport-specific addressing, discovery, routing, and connections. Candidate responsibilities are local accounts/approval, message rooms/unread state, chat/channel policy, files/mail/directory data, session/menu presentation, persistence/search, transport adapters, operations, and optional game/extension hosts. This is a research map, not a prescribed code/module layout.

## Non-negotiable boundaries

- The server enforces approved membership for every community action on each access surface.
- A node operates independently with no public HTTP service, shared-channel federation, central discovery, or alternative transport requirement.
- Only explicitly participating shared chat may cross BBS boundaries. Local-only chat and all other BBS data stay local; necessary remote author/delivery metadata does not imply federated accounts.
- Directory listings are local discovery records, not membership grants or federation subscriptions.
- Doors are optional and isolated from the core, with explicit resources, privileges, lifecycle, and return/recovery behavior.
- Text/ASCII remains useful; ANSI capabilities and legacy encodings need a compatibility contract.
- No JS/TS in source or dependencies, including the optional browser console and repository tooling.

## Current direction and proposed validation slice

[SSH with optional private-network-only operation](decisions/0004-ssh-and-optional-private-network.md) is accepted product intent; its backend and enforcement remain unvalidated. [ADR 0005](decisions/0005-rust-sqlite-validation-slice.md) proposes hosted Rust/readiness I/O and local SQLite WAL for the next integrated validation slice, based on native-session and storage/recovery probes. It rejects millisecond scanning as the initial I/O approach and keeps Go as a measured fallback. No production stack or appliance OS has been accepted.

## Open decisions

Compare runtime/language, local persistence/search, command/session representation, secure terminal access, optional HTTP exposure, no-JS browser-console feasibility, file transfers, room reply semantics, IRC interoperability, extension mechanisms, and later shared-channel/mesh delivery against measurable constraints. Each material choice needs a source-attributed decision record and a falsifiable validation plan. See [the issue index](issue-index.md).

No database schema, federation protocol, transport adapter contract, runtime, framework, performance target, or supported operating system is adopted by this document. Select fast/lightweight options on measured evidence, including deployment burden and security, rather than language familiarity.
