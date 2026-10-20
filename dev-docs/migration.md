# Planning-to-GitHub migration

Date: 2026-10-19. Authorized by the founder's request to initialize a public, Codex-friendly Faction BBS project and retire the earlier planning mode.

## Authority after migration

| Previous surface | Current destination |
| --- | --- |
| `.planning/PROJECT.md`, `INTENT.md` | [Product direction](../docs/project.md), [session](../docs/session.md), [security/privacy](../docs/security-privacy.md), and accepted decisions. |
| `.planning/BACKLOG.md`, captured candidates | GitHub Issues and the [issue index](issue-index.md), preserving BG-001 through BG-020 provenance. |
| `.planning/ROADMAP.md`, `STATE.md` | [Roadmap](roadmap.md), README status, GitHub milestones/issues. |
| `.planning/codebase/`, temporary maps | Read-only historical archive; current [architecture questions](architecture.md) replace stale implementation claims. |
| `.planning/memory/` | Empty historical memory arrays preserved; future durable knowledge belongs in docs/decisions or issue-linked plans. |
| Old agent guide | Current root [AGENTS.md](../AGENTS.md); old guide archived. |
| `docs/initial-seed-prd.md`, `docs/transport-erd.md` | Historical seed archive; current documentation replaces draft requirements. |

The `.planning/` directory and its command-based workflow are retired. There is no dual-write or synchronization process. Historical path strings remain only in the archive, migration explanation, and provenance records.

## Preservation and changes

The [archive manifest](archive/manifest.json) records every non-OS source file and its SHA-256 digest. Original content is preserved unchanged. macOS `.DS_Store` files are not project artifacts and are excluded. Archives may contain obsolete internal links; current-doc link checks exclude their historical bodies while hash checks verify them.

Each old candidate gets one GitHub issue with its legacy ID, archived source, refreshed scope, deliverables, and acceptance criteria. New work is separately scoped and cross-referenced. Migration does not select a production stack or treat every feature as an initial-release commitment.

Later owner direction takes precedence over old examples: Faction BBS naming; BSD-3-Clause licensing; no JS/TS anywhere, including browser assets; WWIV-style approved sessions; Citadel-style local rooms; chat-only federation; private/optional HTTP exposure; privacy/security; lightweight/fast operation; future mesh research. Earlier xterm.js, Node, mandatory public web, or broad content federation ideas are not current options.
