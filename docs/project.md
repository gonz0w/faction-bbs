# Faction BBS: product direction

Status: confirmed product intent, early local core implemented; complete technical design remains under validation. Updated 2026-10-23. Authority: the founder's project conversations on 2026-10-19; preserved earlier context is in the [archive](../dev-docs/archive/README.md).

## Purpose and character

Give independent communities a durable, low-bandwidth home that their sysops own and shape. Faction BBS is the selected project name, replacing the working name Citadel IRC.

The founder wants the underground, hax0r/1337, ANSI-rich feel of the BBS scenes they remember: Vision-X, Vision/2, Oblivion, and personal Pascal-era BBS hacks. These are aesthetic and experiential references, not adopted implementations or verified historical lineage. The system should feel like a place operated by people, with distinctive identity, messaging, file sharing, and games.

The optional [gatherings agenda](gatherings.md) now provides one-off UTC chat nights,
explicit counts-only RSVPs, organizer/sysop controls and approved-login reminders.
Local plaintext event data remains separate from device-encrypted private mail.

Optional [community polls](polls.md) now provide local single-choice questions,
explicit ballot changes and configurable result visibility with plain text charts.
They are ordinary local operator-readable data, separate from private mail.

Optional [personal catch-up](catchup.md) now adds a configurable local return view
with room/reply previews, own favorites, eligible files, gatherings and notices.
Only explicit saves record a visit; previews leave room unread markers unchanged.
No server-side private-mail content or count is included.

Optional [discussion discovery](discussion-search.md) adds bounded literal
subject/body/room/author search, current-permission snippets, exact read-only source
views, own bookmarks and deliberate paged reference exports. No stored query
history, private-mail indexing or extra unread ledger is introduced.

Optional [curated art galleries](scene-gallery.md) give each board original/licensed
ANSI/ASCII exhibits, artist notes, text editions and optional current-permission
file links, using owner-reviewed startup packs and the canonical cell importer.

The [native travel panel](travel-modes.md) now provides deliberate compact visits
and per-file payload ceilings without changing routes, accounts or board content.
Offline exchange and link-profile validation remain follow-on work.

Optional [library shelves](library-shelves.md) curate existing eligible files with
notes/rights/provenance and deliberate current-permission transfer/discussion jumps.
The optional [file request desk](file-requests.md) now adds persistent requests,
explicit published-file fulfillment and author/sysop close/delete moderation.

## Confirmed requirements

| Area | Requirement |
| --- | --- |
| Membership | Login/registration and local sysop approval before community access. |
| Session | WWIV-inspired flow: approved login, BBS-specific ANSI identity art, sysop-defined welcome/broadcast/quote, main menu. |
| Boards | Citadel-style chronological rooms: discover/enter, read new messages, post and reply, with formatted headers/wrapping/pagination and clean section transitions. Local by default; Shared wrapped/paged room presentation and an offline Netnews text codec exist; schema-6 metadata now adds local article IDs, authored/default subjects and new-post UTC times; local proto-article projection exists, while bounded v2 MIME/UTF-8 wire support and offline original INN checks exist (#103); live NNTP peering/providers (#101/#104) remain planned; schema-7 Unicode composing and optional [native text rendering](terminal-capabilities.md) now exist. |
| Chat | Local IRC-style commands/channels and bounded history exist ([guide](local-chat.md)). #100 now compares actual IRC/IRCv3 and invited server links for public/shared chat; private XMPP conversations/unread UX are #99. No IRC wire support is claimed. |
| Files/mail | [Local file areas](file-areas.md) with separate restricted SFTP first; later classic transfer adapters. [Private mail](private-mail.md) requires member-side encryption and a native local terminal client; no BBS/home server reads plaintext. File areas/SFTP and the [native transfer client](native-client.md) are optional working previews, including permission-checked room/chat file links; encrypted mail/attachments and roaming identity remain outside the product, with a [two-home protocol proof](../dev-docs/research/0045-encrypted-mail-and-roaming-proof.md) and native integration tracked in #90/#91. |
| Doors | Optional ANSI game doors. Prefer original games/binaries/servers where feasible; evaluate isolated compatibility hosts before future ports. |
| Other systems | Local Contacts / Other BBSes directory with submitted registrations, discovery, and visits subject to destination approval. |
| Federation | Optional shared chat plus explicit device-encrypted private mail and scoped home-identity proofs. Independent invited home servers; local approval, local rooms/community files, directories and local-only chat remain local. #101 prioritizes NNTP-first opt-in discussion rooms; article format #103 and optional providers #104 remain planned. See [home identities/mail](private-mail.md); protocols remain gated. |
| Terminal | Ordinary SSH terminal clients are the initial secure-access direction, with optional invitation-only private-network operation (owner acceptance 2026-10-19). Overlay admission, SSH identity and local BBS approval remain distinct; no member host shell is implied. Physical serial terminals are not a current target. A [restricted development gateway](restricted-ssh.md) now passes native NetBSD application/member oracles; complete production enforcement/capacity remains unvalidated. |
| Menus/themes | Oblivion/2-inspired arrow-key highlight + Enter and direct displayed hotkeys (D/d for `[D]oors`). Sysop-defined palettes/layouts/art are foundational; optional bounded custom grids, SAUCE/iCE import and long-art login paging now extend the classic list. Optional manual/annual seasonal palettes and login templates now work in the preview, with a disabled example calendar and next-login selection. ACiD/iCE and Razor/scene typography are visual references; see [menu/theme direction](menus-and-themes.md). |
| Web | HTTP portal/browser-console access is a goal, optional and under sysop control. It may be private, disabled, or absent from an initial deployment. No public HTTP dependency. |
| Networks | Initial focus: modern private networks, with occasional poor/interrupted connections (owner clarification 2026-10-19). Constrained links remain research inputs; future mesh/non-IP networks are separately evaluated and no specific alternative transport is promised. |
| Operations | Software/developer-host and NetBSD VM validation now; physical SBC testing deferred until a complete product can be loaded by the owner. No physical hardware gate on software work. Approachable hobby operation on small SBCs/Raspberry Pi or a small VM appliance; MINIX is a candidate, not mandatory (owner clarification 2026-10-19). Compare whole runtime images and update/recovery burden, not full distribution size. Small, fast, lightweight self-hosting; local ownership, export, backup/recovery, and graceful degradation. [Current private snapshot tools](export-restore.md) cover schema-2 through schema-15 databases, including arcade bests, file catalog/public references, explicit local-room draft checkpoints and local wall notes/settings/receipts and member pages/dated updates, gatherings, polls and catch-up preferences/checkpoints and personal discussion bookmarks and file requests/fulfillment; complete file-area bundles additionally preserve blobs. Full installation/member-scoped exports remain work. Owner research envelope: small SBC/VM, 1–2 cores, about 1 GiB host RAM, 100–250 connected members (2026-10-19). This is a measurement target, not supported capacity or an application memory allowance. |
| Extensions | Core first, then Lua menus/isolated game doors. Research optional public permanent rooms, ANSI commerce/payment models, blockchain-linked game economies and modern network adapters; no provider/custody/runtime adopted. See [extension direction](extensions.md). |
| Privacy | Minimal opt-in telemetry, deliberate exposure and retention policies, no mandatory public directory or central service. |
| Technology | Rust selected; NetBSD primary target, initially validating NetBSD 11 ARM64. No broad NetBSD-port or physical SBC support claim. No JavaScript or TypeScript anywhere in project source/tooling/dependencies, including browser-terminal assets. Select fast/lightweight implementation options through comparison and prototypes. |

## Boundaries

Each BBS controls its identity, members, local roles, rooms, mail, files, games, directory, and policies. Shared chat, addressed encrypted mail and scoped home identity carry only their necessary context; they do not imply replicated account databases, shared approvals, or access to another board.

A directory listing is not federation, trust, or membership. Visiting another BBS crosses into that sysop's access boundary. A local-only chat channel must never become shared implicitly. The proposed public-publishing room is a separate explicit outbound category under research; it is not automatic BBS federation or authority to mirror private local rooms. Exact preview, author intent and separate publish permissions remain design requirements.

Transport-specific addresses, discovery, routing, and connection behavior must stay separate from community semantics. Core behavior, persistence, client/session surfaces, adapters, and extensions are boundaries to validate, not a finalized module layout.

## Research before production implementation

The owner selected **Rust** and **NetBSD as primary target** on 2026-10-19 ([ADR 0006](../dev-docs/decisions/0006-rust-and-primary-netbsd.md)). The first development core uses local SQLite WAL and Argon2id credentials ([ADR 0008](../dev-docs/decisions/0008-local-core-and-credentials.md)); the production backend, full SSH hardening, federation protocol, extension runtime and appliance configuration still need validation. The restricted development gateway now has [native application evidence](../dev-docs/research/0024-restricted-ssh-netbsd.md). SSH with optional private-network mode is accepted access direction. Native NetBSD ARM64 probes verified Mio readiness, rusqlite with bundled/pkgsrc SQLite, and a synthetic restricted OpenSSH gateway; full security, capacity and hardware validation remain open. The next integrated validation slice uses Rust/readiness I/O and local SQLite WAL; [ADR 0005](../dev-docs/decisions/0005-rust-sqlite-validation-slice.md) defines the experiment and required evidence before production adoption. Familiarity, nostalgia, and popularity alone are insufficient evidence. JavaScript/TypeScript is excluded by explicit owner instruction.

The founder welcomes newer and older technologies on merit and explicitly prioritizes tiny working sets and scalable low-memory native options, including selective assembly if measurements justify it. Historical 640K BBSes are an efficiency inspiration, not a literal modern security/host ceiling. Unusual runtimes, specialized operating systems, blockchain and optional public helpers may be investigated, but must satisfy the same privacy, local-authority and independent-operation constraints. No external service is selected. Proposed numeric application budgets and bounded experiments are in the [small-host research](../dev-docs/research/0028-small-host-budgets.md).

Security, privacy, terminal compatibility, resource use, and local deployment simplicity must be measured. A private/underground atmosphere is not an anonymity guarantee; public source availability says nothing about whether an operator exposes their deployment.

The owner accepted both unchanged game-server reuse and isolated original DOS
binaries on 2026-10-21. The [classic connector](classic-game-doors.md) has bounded
owned-fixture NetBSD evidence; [the per-title inventory](../dev-docs/research/0041-classic-door-reuse.md)
records actual acquisition/interface unknowns. Original game availability and compatibility are per-title questions. This project includes no original game executables or third-party artwork. Future ports are separate work, not a prerequisite for the first useful BBS.

Current sequencing is [software-first local core, then Lua](../dev-docs/decisions/0007-software-first-and-extension-direction.md). The [first software slice](../dev-docs/first-local-slice.md) defines the build scope; physical SBC hardware is not a near-term dependency. Both external-member-wallet and sysop-payment-service models remain open comparison candidates.

## Unresolved questions

- Full account lifecycle, registration fields and remote-safe recovery; local bootstrap, pending/approved/revoked status and owner-only recovery exist in the [development console](local-development.md).
- Room reading order, unread state, quoting/reply behavior, and retention/moderation.
- Which real IRC service/adapter/client interoperability and invited linking profile to adopt (#100); implemented slash commands still provide no IRC wire server.
- SSH backend, identity/bootstrap mapping, confinement and private-only binding validation; any legacy Telnet compatibility remains separate from the accepted secure-access direction. Disabled/private portal operation remains required.
- Whether a useful browser console is achievable with no JavaScript dependencies; permitted alternatives and explicit tradeoffs need research.
- Character encodings, ANSI capabilities, safe parsing, plain-text fallback, and link budgets.
- File transfer protocols, quotas, untrusted upload handling, mail semantics, and search scope.
- Legacy game runtimes, distribution permissions, session handoff, and isolation.
- Directory submission review, stale listings, and connection handoff.
- Shared-channel ownership, subscriptions, author identity, moderation, history, reconnect, and disconnected operation.

See the [roadmap](../dev-docs/roadmap.md) and [issue index](../dev-docs/issue-index.md) for bounded work and dependencies.

## Latest direction: file areas and independent home identities

Owner decisions on 2026-10-21: use a separate restricted SFTP service first; keep
native DOS/Amiga/other door-host adapters extensible and defer expansion. Register
identities at independent invited home authorities, such as `alice@home.example`.
A native local terminal client holds device keys and reads the home inbox while
visiting another board. Each visited sysop still approves local access. No server
may decrypt/render the requested private mail; login unlocks local client keys,
not a server-side mailbox key. New devices and recovery require explicit key trust.
XMPP/OMEMO and cross-BBS auth are under research #84, not working integrations.
This supersedes earlier blanket chat-only/local-mail constraints only for these
explicit capabilities. See ADRs [0030](../dev-docs/decisions/0030-file-areas-and-transfer-adapters.md)
and [0031](../dev-docs/decisions/0031-home-identities-and-client-encrypted-mail.md).

## Latest direction: local sysop console and modern communications

After using the live SSH playground, the owner requested a local OS-console
sysop dashboard (#95), progressive terminal capability/graphics support (#96),
NAT-aware abuse rules (#97), NetBSD maintenance/setup assistance (#98), tightly
integrated native XMPP unread/private chat/mail/attachments (#99), a real IRC
reuse comparison (#100) and optional discussion-room federation research (#101).
The [current stories](modern-sysop-and-communications.md) define acceptance and
trust boundaries. The operator console follows OS authentication rather than the
member SSH path; an in-BBS sysop role grants no host privileges. Capability probes
and guest firewall rules must reflect actual terminal/network topology.

The owner clarified that discussion posts should use Netnews-format articles,
with optional NNTP/news-provider connections; IRC and XMPP retain live/private
formats. See [article/news requirements](discussion-articles-and-news.md) and
#103/#104. Formatting does not publish content or replace SQLite.

The new room item supersedes an absolute ban on **researching** shared rooms;
local-by-default storage and consent remain. No automatic room replication,
shared approvals, public listener, host change or new wire
runtime is adopted. Keep ordinary board operation useful with every adapter off.

The optional [Kitty preview](kitty-graphics.md) adds one bounded trusted RGB image
and a separate native local opt-in gate, preserving classic ANSI and sysop themes.
Real emulator/font/multiplexer certification and richer licensed assets stay future
work; no new host/hardening or capacity claim follows.

[Named terminal compatibility profiles](terminal-compatibility.md) now have real
software evidence, including native client/member SSH and conservative old-screen
fallback. This does not certify pixel/glyph quality or add a terminal runtime.

## Native local vault

The optional [local native mail vault](local-mail-vault.md) now stores encrypted
offline drafts and device pins with member-held backup/recovery. It does not yet
provide network inbox/send, OMEMO private state or authenticated roaming. Local
BBS approval and current rooms/chat/files remain independent.
