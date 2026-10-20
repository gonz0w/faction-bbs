# Citadel IRC

Citadel IRC is the working name; the final product name is undecided.

## What This Is

Citadel IRC is a self-hosted, local-first, terminal-based community platform with a WWIV-inspired session and menu experience. Users log in or register and must be approved by the local sysop before entering the BBS. After successful approved login, the BBS displays sysop-defined ANSI identity art, then a main menu with a sysop-defined message, quote, or announcement.

Members find and enter Citadel-style rooms to read and post messages, use IRC-style commands for live chat, and navigate BBS-style menus to reach community features and ANSI door games. A local Contacts / Other BBSes directory lets operators register their systems for discovery and visits. Each sysop hosts the content and features they care about. Only explicitly shared chat channels may federate; boards, files, mail, games, directories, accounts/approval, and other BBS content remain local. Chat channels may also remain confined to one BBS. The terminal experience must be available through a network terminal app and an embedded console on a webpage.

It must operate efficiently across conventional IP networks and constrained, alternative, or intermittent networks without making any one connection protocol, terminal implementation, or deployment topology fundamental to the product.

## Core Value

Give independent communities a durable, low-bandwidth, sysop-controlled home that remains useful across the networks they can actually reach.

## Requirements

### Validated

- Self-hosting, local ownership, exportability, and minimal opt-in telemetry are product foundations.
- The product combines asynchronous discussion with persistent chat and classic BBS concepts.
- Citadel-style rooms are the organizing model for message boards: users find a room, enter it, read new messages, and post or reply there. Favor the Citadel room/message experience; detailed reply presentation remains to be specified.
- Live chat uses IRC-style commands; shared/global chat initially spans one installation. Future optional federation must allow independent systems to subscribe to shared chat channels. Compatibility with existing IRC clients remains open.
- The primary experience is terminal-based, with BBS-style menus and doors to ANSI games. A terminal app over a network and a webpage embedding the console are required access surfaces.
- The WWIV-inspired session requires login/registration and local sysop approval before community access. Approved login leads through BBS identity ANSI art to the main menu and sysop-defined welcome/broadcast content.
- A locally hosted Contacts / Other BBSes directory accepts registrations of other systems so members can discover and visit independently operated BBSes. Listing a system does not grant access to it or link its content.
- Federation is limited to participating shared chat channels. All other BBS data and features remain local to each sysop; local-only chat channels must remain possible.
- Prefer using existing classic door games, including TradeWars, unchanged where available and compatible; investigate integration or compatibility hosting before considering future ports. No individual title is guaranteed supported.
- Transport-specific addressing, discovery, and routing must not leak into core community and content models.
- Constrained and intermittent links require graceful degradation and a useful asynchronous experience.
- Sysops need a safe, durable path to customize and modernize deployments.

### Active

- Research and compare architecture, core-language, storage, client, transport, topology, and extension approaches before committing implementation.
- Define a low-bandwidth interaction budget and the terminal/ASCII/ANSI compatibility contract, including the same BBS experience through direct terminal access and a browser console.
- Evaluate independent-node operation and optional chat-channel interconnection topologies, including constrained/mesh operation, within the chat-only federation boundary.
- Establish a minimal core and explicit extension boundaries that avoid premature platform complexity.
- Identify small, falsifiable prototypes that retire the highest-risk decisions.

### Out of Scope

- Selecting a production language, framework, protocol, or transport before comparative research and prototypes. This would prematurely constrain the design.
- Treating every candidate network as an initial implementation target. Different transport families have materially different constraints.
- Building product features during the research milestones. The immediate goal is evidence-backed architecture decisions.
- Federation of boards, files, mail, games, directories, accounts, or other BBS content. The confirmed cross-system feature is shared chat channels; visiting another BBS is a separate local access experience.

## Context

The workspace contains planning artifacts and two historical draft inputs, `docs/initial-seed-prd.md` and `docs/transport-erd.md`; no implementation stack has been selected. The seed documents mention IP, Reticulum, mesh networks, anonymity networks, overlays, and packet radio as candidates, but these are research inputs rather than commitments. The user has directed the first milestones toward deep architecture research while avoiding needless complexity or premature narrowing.

The user's 2026-10-19 clarification establishes the product experience below and supersedes the seed PRD's web-first framing and optional-terminal language. It does not select a production stack or start a milestone.

## Product Direction Record — 2026-10-19

### Question and Decision

What community interaction and access experience must architecture research preserve?

Adopt Citadel-style message-board rooms, IRC-style command-driven live chat, BBS-style menus, doors to ANSI games, and a terminal experience accessible through a network terminal app and within a webpage. Reuse existing games where feasible; ports are a possible future response to unavailable or incompatible titles, not current implementation scope.

Follow-up clarification on the same date: favor Citadel's room/message experience over Reddit-style interaction; global chat initially spans one installation; future independent systems should be able to subscribe to federated chat channels. TTY access means a terminal app over a network. Physical serial-terminal support is not a current requirement.

Further clarification on the same date: the session/menu reference is WWIV BBS. Login or registration is required, with no community access before sysop approval. Approved login displays BBS-specific ANSI art before the main menu; the sysop can define a broadcast message, quote, or other welcome content. A local contacts directory lets other operators register their BBSes for discovery and visits. Only participating shared chat channels federate; everything else stays local, including BBS-local chat channels where configured. Citadel IRC is a working name.

**Authority and confidence:** Explicit user direction in this project's chat on 2026-10-19 ("lets lock that in"). High confidence in the product requirements; technical feasibility and individual game compatibility remain unvalidated. The project owner is the decision authority.

### Constraints, Alternatives, and Tradeoffs

- The adopted room navigation replaces an unspecified generic forum organization. The user clarified "more like citadel, less like reddit": use room-oriented reading/posting and unread catch-up as the baseline to specify. Detailed reply presentation remains open; voting, ranking, and Reddit-style nested threads are not established requirements.
- A browser terminal is a required access surface for the BBS experience. The seed draft's web-first interface with an optional terminal is superseded. A shared terminal experience introduces keyboard, terminal-size, character-encoding, and accessibility questions that must be tested.
- IRC-style interaction is required; an actual IRC server/protocol or external-client compatibility is a separate research decision. Global chat initially operates within one installation. Subscription-based shared-channel federation is a future requirement captured in BG-017; it must remain optional for standalone nodes, with no protocol or delivery model selected yet.
- Access through a terminal app over a network is required; Telnet was suggested as an example, not selected. SSH, Telnet, and browser session transport need comparison against the existing network/privacy constraints. Physical serial-terminal support is not a current requirement.
- Existing game binaries or servers, isolated compatibility runtimes, and future source ports are alternatives for doors. Reuse can preserve original behavior but adds runtime, deployment, session handoff, and maintenance constraints. Keep these optional and outside core community behavior; do not make one game's runtime mandatory for all nodes.
- Mandatory approved membership replaces public/guest community browsing. Pre-approval access is limited to the entry and registration/status experience; the approval authority remains local. Approval timing and notification/recovery mechanics still need design.
- WWIV-style session/menu behavior, Citadel-style message-board rooms, and IRC-style chat are distinct interaction references. No source-code reuse, exact command set, or network protocol follows from those references alone.
- A local directory supports discovery without synchronizing BBS content or granting remote membership. Submission review, stale listings, and the mechanics of visiting another system remain open. A central directory and automatic cross-system login are not requirements.
- Chat-only federation narrows the earlier broad topology research. It shares participating channel messages and the necessary delivery/author context, without replicating local account databases, approvals, boards, directories, or other BBS content. Local-only chat must stay outside federation. Local backups and client session recovery remain compatible with this boundary.

### Confirmed Session Shape

1. Connect through a network terminal app or the webpage console.
2. Log in, or submit a registration request. Registration does not automatically grant community access; pending or unapproved users remain at the entry/approval-status boundary.
3. After successful login and local approval, display sysop-defined ANSI art identifying this BBS.
4. Show the main menu with a sysop-defined broadcast, quote, welcome message, or equivalent content.
5. Navigate the BBS's locally available features; chat may include local-only channels and, in a future milestone, subscribed shared channels.
6. Use Contacts / Other BBSes to discover other independent systems. A visit is subject to the destination sysop's own login and approval boundary; connection handoff is not yet designed.

Candidate menu areas for research are message rooms, local/shared chat, mail, files, games/doors, Contacts / Other BBSes, account/preferences, help, sysop contact, and logout. These are a menu inventory to evaluate, not an accepted first-release feature list; sysop tools require the appropriate role.

### Primary Evidence

External sources retrieved 2026-10-19; these establish research leads, not Citadel IRC support claims:

- User product clarification in this project's chat, 2026-10-19: authoritative source for the requirements above.
- User follow-up answers in this project's chat, 2026-10-19: "more like citadel, less like reddit"; one-installation global chat initially with future federation for systems subscribing to channels; "Terminal app over a network" for TTY access.
- User further clarification in this project's chat, 2026-10-19: "session is more like WWIV BBS"; "no access unless you are approvewd"; ANSI art before the menu; sysop-defined welcome content; a contacts page for other BBS registrations; "the only thing that is federate is the global chat groups"; all other content local, with local-only chat possible; Citadel IRC is a working name. This is product authority, not a claim about WWIV's implementation.
- [Citadel room and message commands](https://citadel.org/room_and_message_commands.html): room entry, room-scoped message retrieval/posting, and unread state provide a reference for studying the requested room experience, without adopting Citadel's protocol.
- [EIS TradeWars product page](https://eisonline.classictw.com/): describes the standalone TradeWars Game Server and links to downloads/registration. A candidate for reuse evaluation; artifacts, operating-system requirements, and availability/permission for the intended use still require per-title verification.
- [Synchronet DOS door integration documentation](https://wiki.synchro.net/howto:dosemu): documents an existing compatibility approach, including communication drivers, dropfiles, and multiuser considerations. It is precedent to evaluate, not a selected Citadel runtime.
- [xterm.js project](https://xtermjs.org/): an example of an existing browser terminal implementation. Selection and ANSI/legacy encoding suitability remain unevaluated.

### Open Questions

- What exact room reading order, unread navigation, and reply presentation best preserve the requested Citadel experience?
- For future federated channels, who owns a channel, approves subscriptions, and applies moderation across installations? What history and reconnect behavior is required?
- Is external IRC-client interoperability required, or is IRC-style interaction inside the BBS sufficient?
- Which original games and terminal/encoding capabilities should define the first compatibility tests?
- What registration fields, approval/status feedback, rejection, and access-revocation behavior are required?
- Should other-BBS directory submissions require local sysop review, and how should stale or removed listings behave?
- How should visiting another BBS work from a network terminal and a browser console while preserving the destination's access boundary?
- What minimum menu inventory, ANSI fallback/skip behavior, and welcome-content controls are appropriate for constrained links and differing terminal capabilities?

### Falsifiable Validation Plan

These are proposed research experiments, not accepted roadmap work:

1. Approved session and room/chat navigation: through both a network terminal and browser console, demonstrate registration without community access, denial for an unapproved account, local approval, successful login, ANSI identity art before the main menu, and sysop-defined welcome content. Then demonstrate finding a room, reading new messages, posting, entering live chat, and returning. Pass only if both access surfaces enforce approval before all community actions and preserve posting/read state without missing or duplicating posts across a tested disconnect.
2. Legacy door handoff: select one available title after checking its distribution and runtime requirements. Demonstrate entry from a BBS menu, correct character/key handling, gameplay, exit, and return to the BBS. Where the title supports concurrent users, test session separation and shared game state. A failed or interrupted game must not terminate the core BBS or expose another user's session.
3. Constrained terminal baseline: agree on terminal capabilities and a measurable link budget before testing room reading/posting and chat degradation. Report which optional game capabilities exceed that budget rather than making every door a baseline requirement.
4. Local ownership and future channel federation: when federation enters a milestone, demonstrate that only a subscribed shared channel exchanges messages between two BBSes. Local-only chat and other BBS data must remain local. Adding a remote system to a contacts directory must neither start federation nor grant access to that system. Confirm both BBSes continue local operation when their link is unavailable.

Existing captured candidates cover this direction without new duplicate items: BG-006 (approval/local identity), BG-007/BG-009/BG-017 (chat-only interconnection and local data), BG-008/BG-013/BG-014 (approved terminal/browser experience), BG-010 (doors), BG-012 (rooms, directory, local/shared channel model), and BG-018/BG-019 (sysop controls and access boundaries). No candidate is promoted and no milestone is activated by this record.
