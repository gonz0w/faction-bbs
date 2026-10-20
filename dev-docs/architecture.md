# Architecture: boundaries to validate

Status: no implementation or production stack selected. Product requirements are in [docs/project.md](../docs/project.md).

Separate community semantics from transport-specific addressing, discovery, routing, and connections. Candidate responsibilities are local accounts/approval, message rooms/unread state, chat/channel policy, files/mail/directory data, session/menu presentation, persistence/search, transport adapters, operations, and optional game/extension hosts. This is a research map, not a prescribed code/module layout.

## Non-negotiable boundaries

- The server enforces approved membership for every community action on each access surface.
- A node operates independently with no public HTTP service, shared-channel federation, central discovery, or alternative transport requirement.
- Only explicitly participating shared chat may cross BBS boundaries. Local-only chat and all other BBS data stay local; necessary remote author/delivery metadata does not imply federated accounts.
- Directory listings are local discovery records, not membership grants or federation subscriptions.
- Doors are optional and isolated from the core, with explicit resources, privileges, lifecycle, and return/recovery behavior.
- Text/ASCII remains useful; ANSI capabilities and legacy encodings need a compatibility contract.
- No JS/TS in source or dependencies, including the optional browser console and repository tooling.

## Open decisions

Compare runtime/language, local persistence/search, command/session representation, secure terminal access, optional HTTP exposure, no-JS browser-console feasibility, file transfers, room reply semantics, IRC interoperability, extension mechanisms, and later shared-channel/mesh delivery against measurable constraints. Each material choice needs a source-attributed decision record and a falsifiable validation plan. See [the issue index](issue-index.md).

No database schema, federation protocol, transport adapter contract, runtime, framework, performance target, or supported operating system is adopted by this document. Select fast/lightweight options on measured evidence, including deployment burden and security, rather than language familiarity.
