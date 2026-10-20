# Faction BBS: product direction

Status: confirmed product intent, early local core implemented; complete technical design remains under validation. Updated 2026-10-19. Authority: the founder's project conversations on 2026-10-19; preserved earlier context is in the [archive](../dev-docs/archive/README.md).

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
| Terminal | Ordinary SSH terminal clients are the initial secure-access direction, with optional invitation-only private-network operation (owner acceptance 2026-10-19). Overlay admission, SSH identity and local BBS approval remain distinct; no member host shell is implied. Physical serial terminals are not a current target. Backend/enforcement unvalidated. |
| Menus/themes | Oblivion/2-inspired arrow-key highlight + Enter and direct displayed hotkeys (D/d for `[D]oors`). Sysop-defined palettes/layouts/art are foundational; seasonal colors/login screens are an optional direction. ACiD/iCE and Razor/scene typography are visual references; see [menu/theme direction](menus-and-themes.md). |
| Web | HTTP portal/browser-console access is a goal, optional and under sysop control. It may be private, disabled, or absent from an initial deployment. No public HTTP dependency. |
| Networks | Initial focus: modern private networks, with occasional poor/interrupted connections (owner clarification 2026-10-19). Constrained links remain research inputs; future mesh/non-IP networks are separately evaluated and no specific alternative transport is promised. |
| Operations | Software/developer-host and NetBSD VM validation now; physical SBC testing deferred until a complete product can be loaded by the owner. No physical hardware gate on software work. Approachable hobby operation on small SBCs/Raspberry Pi or a small VM appliance; MINIX is a candidate, not mandatory (owner clarification 2026-10-19). Compare whole runtime images and update/recovery burden, not full distribution size. Small, fast, lightweight self-hosting; local ownership, export, backup/recovery, and graceful degradation. Owner research envelope: small SBC/VM, 1–2 cores, about 1 GiB host RAM, 100–250 connected members (2026-10-19). This is a measurement target, not supported capacity or an application memory allowance. |
| Extensions | Core first, then Lua menus/isolated game doors. Research optional public permanent rooms, ANSI commerce/payment models, blockchain-linked game economies and modern network adapters; no provider/custody/runtime adopted. See [extension direction](extensions.md). |
| Privacy | Minimal opt-in telemetry, deliberate exposure and retention policies, no mandatory public directory or central service. |
| Technology | Rust selected; NetBSD primary target, initially validating NetBSD 11 ARM64. No broad NetBSD-port or physical SBC support claim. No JavaScript or TypeScript anywhere in project source/tooling/dependencies, including browser-terminal assets. Select fast/lightweight implementation options through comparison and prototypes. |

## Boundaries

Each BBS controls its identity, members, local roles, rooms, mail, files, games, directory, and policies. Shared chat needs only the author/delivery context necessary for participating channels; it does not imply replicated account databases, shared approvals, or access to another board.

A directory listing is not federation, trust, or membership. Visiting another BBS crosses into that sysop's access boundary. A local-only chat channel must never become shared implicitly. The proposed public-publishing room is a separate explicit outbound category under research; it is not automatic BBS federation or authority to mirror private local rooms. Exact preview, author intent and separate publish permissions remain design requirements.

Transport-specific addresses, discovery, routing, and connection behavior must stay separate from community semantics. Core behavior, persistence, client/session surfaces, adapters, and extensions are boundaries to validate, not a finalized module layout.

## Research before production implementation

The owner selected **Rust** and **NetBSD as primary target** on 2026-10-19 ([ADR 0006](../dev-docs/decisions/0006-rust-and-primary-netbsd.md)). The first development core uses local SQLite WAL and Argon2id credentials ([ADR 0008](../dev-docs/decisions/0008-local-core-and-credentials.md)); the production backend, SSH integration, federation protocol, extension runtime and appliance configuration still need validation. SSH with optional private-network mode is accepted access direction. Native NetBSD ARM64 probes verified Mio readiness, rusqlite with bundled/pkgsrc SQLite, and a synthetic restricted OpenSSH gateway; full security, capacity and hardware validation remain open. The next integrated validation slice uses Rust/readiness I/O and local SQLite WAL; [ADR 0005](../dev-docs/decisions/0005-rust-sqlite-validation-slice.md) defines the experiment and required evidence before production adoption. Familiarity, nostalgia, and popularity alone are insufficient evidence. JavaScript/TypeScript is excluded by explicit owner instruction.

The founder welcomes newer and older technologies on merit and explicitly prioritizes tiny working sets and scalable low-memory native options, including selective assembly if measurements justify it. Historical 640K BBSes are an efficiency inspiration, not a literal modern security/host ceiling. Unusual runtimes, specialized operating systems, blockchain and optional public helpers may be investigated, but must satisfy the same privacy, local-authority and independent-operation constraints. No external service is selected. Proposed numeric application budgets and bounded experiments are in the [small-host research](../dev-docs/research/0028-small-host-budgets.md).

Security, privacy, terminal compatibility, resource use, and local deployment simplicity must be measured. A private/underground atmosphere is not an anonymity guarantee; public source availability says nothing about whether an operator exposes their deployment.

Original game availability and compatibility are per-title questions. This project includes no original game executables or third-party artwork. Future ports are separate work, not a prerequisite for the first useful BBS.

Current sequencing is [software-first local core, then Lua](../dev-docs/decisions/0007-software-first-and-extension-direction.md). The [first software slice](../dev-docs/first-local-slice.md) defines the build scope; physical SBC hardware is not a near-term dependency. Both external-member-wallet and sysop-payment-service models remain open comparison candidates.

## Unresolved questions

- Full account lifecycle, registration fields and remote-safe recovery; local bootstrap, pending/approved/revoked status and owner-only recovery exist in the [development console](local-development.md).
- Room reading order, unread state, quoting/reply behavior, and retention/moderation.
- Whether existing IRC clients must interoperate, beyond IRC-style commands within the BBS.
- SSH backend, identity/bootstrap mapping, confinement and private-only binding validation; any legacy Telnet compatibility remains separate from the accepted secure-access direction. Disabled/private portal operation remains required.
- Whether a useful browser console is achievable with no JavaScript dependencies; permitted alternatives and explicit tradeoffs need research.
- Character encodings, ANSI capabilities, safe parsing, plain-text fallback, and link budgets.
- File transfer protocols, quotas, untrusted upload handling, mail semantics, and search scope.
- Legacy game runtimes, distribution permissions, session handoff, and isolation.
- Directory submission review, stale listings, and connection handoff.
- Shared-channel ownership, subscriptions, author identity, moderation, history, reconnect, and disconnected operation.

See the [roadmap](../dev-docs/roadmap.md) and [issue index](../dev-docs/issue-index.md) for bounded work and dependencies.
