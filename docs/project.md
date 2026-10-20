# Faction BBS: product direction

Status: confirmed product intent, technical design unvalidated. Updated 2026-10-19. Authority: the founder's project conversations on 2026-10-19; preserved earlier context is in the [archive](../dev-docs/archive/README.md).

## Purpose and character

Give independent communities a durable, low-bandwidth home that their sysops own and shape. Faction BBS is the selected project name, replacing the working name Citadel IRC.

The founder wants the underground, hax0r/1337, ANSI-rich feel of the BBS scenes they remember: Vision-X, Vision/2, Oblivion, and personal Pascal-era BBS hacks. These are aesthetic and experiential references, not adopted implementations or verified historical lineage. The system should feel like a place operated by people, with distinctive identity, messaging, file sharing, and games.

## Confirmed requirements

| Area | Requirement |
| --- | --- |
| Membership | Login/registration and local sysop approval before community access. |
| Session | WWIV-inspired flow: approved login, BBS-specific ANSI identity art, sysop-defined welcome/broadcast/quote, main menu. |
| Boards | Citadel-style rooms: discover/enter a room, read new messages, post, reply. Exact reply presentation remains open. |
| Chat | IRC-style commands and channels. Initial shared chat stays within one installation; channels may be local-only. |
| Files/mail | Local file areas, messaging, and sysop-owned sharing/access policy. Detailed transfer and mail semantics require design. |
| Doors | Optional ANSI game doors. Prefer original games/binaries/servers where feasible; evaluate isolated compatibility hosts before future ports. |
| Other systems | Local Contacts / Other BBSes directory with submitted registrations, discovery, and visits subject to destination approval. |
| Federation | Future optional subscriptions to shared chat channels only. Other BBS data, accounts/approval, directory entries, and local-only chat remain local. |
| Terminal | Network terminal applications are central; physical serial terminals are not a current target. |
| Web | HTTP portal/browser-console access is a goal, optional and under sysop control. It may be private, disabled, or absent from an initial deployment. No public HTTP dependency. |
| Networks | Ordinary IP plus research into constrained, intermittent, private, future mesh/non-IP networks. No specific alternative transport is promised. |
| Operations | Small, fast, lightweight self-hosting; local ownership, export, backup/recovery, and graceful degradation. |
| Privacy | Minimal opt-in telemetry, deliberate exposure and retention policies, no mandatory public directory or central service. |
| Technology | No JavaScript or TypeScript anywhere in project source/tooling/dependencies, including browser-terminal assets. Select fast/lightweight implementation options through comparison and prototypes. |

## Boundaries

Each BBS controls its identity, members, local roles, rooms, mail, files, games, directory, and policies. Shared chat needs only the author/delivery context necessary for participating channels; it does not imply replicated account databases, shared approvals, or access to another board.

A directory listing is not federation, trust, or membership. Visiting another BBS crosses into that sysop's access boundary. A local-only chat channel must never become shared implicitly.

Transport-specific addresses, discovery, routing, and connection behavior must stay separate from community semantics. Core behavior, persistence, client/session surfaces, adapters, and extensions are boundaries to validate, not a finalized module layout.

## Research before production implementation

No language, framework, storage engine, access protocol, federation protocol, extension runtime, or deployment topology has been selected. Familiarity, nostalgia, and popularity alone are insufficient evidence. JavaScript/TypeScript is excluded by explicit owner instruction.

Security, privacy, terminal compatibility, resource use, and local deployment simplicity must be measured. A private/underground atmosphere is not an anonymity guarantee; public source availability says nothing about whether an operator exposes their deployment.

Original game availability and compatibility are per-title questions. This project includes no original game executables or third-party artwork. Future ports are separate work, not a prerequisite for the first useful BBS.

## Unresolved questions

- Registration fields, approval/status/rejection/revocation, account recovery, and operator bootstrap.
- Room reading order, unread state, quoting/reply behavior, and retention/moderation.
- Whether existing IRC clients must interoperate, beyond IRC-style commands within the BBS.
- Secure terminal access, any Telnet compatibility conditions, and disabled/private portal operation.
- Whether a useful browser console is achievable with no JavaScript dependencies; permitted alternatives and explicit tradeoffs need research.
- Character encodings, ANSI capabilities, safe parsing, plain-text fallback, and link budgets.
- File transfer protocols, quotas, untrusted upload handling, mail semantics, and search scope.
- Legacy game runtimes, distribution permissions, session handoff, and isolation.
- Directory submission review, stale listings, and connection handoff.
- Shared-channel ownership, subscriptions, author identity, moderation, history, reconnect, and disconnected operation.

See the [roadmap](../dev-docs/roadmap.md) and [issue index](../dev-docs/issue-index.md) for bounded work and dependencies.
