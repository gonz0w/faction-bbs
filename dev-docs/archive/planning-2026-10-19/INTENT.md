# Citadel IRC Project Intent

**Revision:** 3
**Created:** 2026-07-15
**Updated:** 2026-10-19

<objective>
Establish an evidence-backed architecture for a modern, self-hosted BBS/forum/chat platform that can serve independent communities over conventional and alternative networks while remaining bandwidth-efficient, extensible, and operationally simple.

Citadel IRC is a working name. The required experience is terminal-based: a WWIV-inspired approved-login/session flow, Citadel-style rooms for message-board posting, IRC-style command-driven live chat, BBS-style menus, and doors to ANSI games. Login or registration and local sysop approval precede community access. Approved login displays BBS-specific ANSI art, then a menu with sysop-defined welcome/broadcast content. Users access the console through a terminal app over a network and embedded in a webpage.

Each sysop hosts their own content and features, including a local directory where other operators register BBSes for discovery and visits. Global chat initially spans one installation; future optional federation is restricted to shared chat channels that systems participate in. Local-only chat channels remain possible; all other BBS content and account/approval state remain local. Prefer existing classic games unchanged where feasible, with ports considered as future work when necessary.

The first milestones are deliberately research-led. They must convert the current product and transport ideas into explicit constraints, evaluated options, prototype questions, and reversible decisions before implementation begins.
</objective>

<users>
- Sysops and self-hosters operating small-to-medium independent communities.
- Members of technical, hobbyist, local, retro-computing, and open-source communities.
- Operators on constrained, intermittent, private, mesh, or non-IP networks.
- Developers and extension authors who need well-defined customization boundaries.
</users>

<outcomes>
- DO-01 [P0]: A documented architecture decision set distinguishes stable core responsibilities from transport, client, storage, and extension boundaries.
- DO-02 [P0]: A comparative evaluation identifies viable implementation-language and runtime choices against the actual product constraints.
- DO-03 [P0]: A transport and topology model explains what standalone nodes, interconnected nodes, hub-and-spoke deployments, and mesh-capable deployments require, without assuming one is mandatory.
- DO-04 [P0]: A low-bandwidth terminal strategy defines Citadel-style room navigation, IRC-style chat interaction, BBS menus, and equivalent community actions through direct terminal access and a webpage console; additional clients may layer on this baseline.
- DO-05 [P1]: An extension strategy evaluates configuration, declarative customization, scripting, plugins, external integrations, and reuse of legacy door games with explicit safety, compatibility, and maintenance tradeoffs.
- DO-06 [P1]: Small prototypes validate the riskiest selected assumptions before feature development.
</outcomes>

<criteria>
- SC-01: No production stack or network support claim is adopted without cited comparative evidence and a recorded rationale.
- SC-02: The selected core can preserve domain semantics independently of transport-specific addressing, discovery, and delivery behavior.
- SC-03: The proposed baseline experience remains usable under constrained links through measurable payload, latency, and offline/store-and-forward assumptions.
- SC-04: Sysop customization has bounded permissions, lifecycle, upgrade, and failure-isolation expectations.
- SC-05: The initial architecture can begin with a small deployment and grow without requiring every future transport, client, or federation feature in the first release.
- SC-06: Direct-terminal and browser-console validation demonstrates the same room/posting and command-driven chat actions without making a particular access protocol mandatory.
- SC-07: Door research demonstrates a representative existing game's session handoff and failure isolation, or records the specific blocker; it does not assume universal compatibility or require ports in the first milestone.
- SC-08: Both terminal access surfaces deny community access before authentication and local approval; approved login displays BBS identity ANSI art before the menu and sysop-defined welcome content.
- SC-09: Future federation exchanges only participating shared-channel chat and necessary delivery context; local-only channels and other BBS data remain local. Directory listings grant neither membership nor federation.
</criteria>

<constraints>
- C-01: Preserve self-hosting, data ownership, export, privacy, and minimal opt-in telemetry.
- C-02: Do not assume Internet/IP availability, but do not require all alternative networks in the first proof of concept.
- C-03: Prefer simple, local, reversible designs over distributed-system complexity that lacks a demonstrated use case.
- C-04: Treat the current PRD and transport design as draft inputs, not approved technical decisions.
- C-05: Do not select an implementation language, extension mechanism, federation protocol, or deployment topology before research establishes decision criteria and prototype evidence.
- C-06: Preserve the terminal-based BBS experience, including a webpage console; the seed PRD's web-first/optional-terminal framing is superseded by user direction on 2026-10-19.
- C-07: Citadel-style rooms and IRC-style commands define interaction requirements, not protocol adoption. Target terminal apps over a network and the browser console. Detailed room reply presentation and IRC-client interoperability remain open in `PROJECT.md`.
- C-08: Global chat initially serves one installation; subscription-based shared-channel federation is a future requirement recorded in BG-017, with milestone timing and technical design still open.
- C-09: Use the WWIV-inspired login/registration and approval gate, ANSI identity art, and menu/welcome sequence in `PROJECT.md`; community content requires local approval.
- C-10: Federation is chat-only. Boards, files, mail, games, directories, local accounts/approvals, and other content remain local to each sysop. Channels may be confined to one BBS. A directory visit does not transfer approval or establish a shared account system.
- C-11: Citadel IRC is a working name; naming and branding remain undecided.
</constraints>

<health>
- HM-01: Every material architecture choice has an owner, decision criteria, source-attributed alternatives, and an explicit confidence level.
- HM-02: Each prototype has one named uncertainty and a pass/fail observation.
- HM-03: The planned initial core has fewer mandatory runtime components than optional adapters and extensions combined.
- Principle: Optimize for independent operation and graceful degradation, not feature parity across every transport.
- Principle: Preserve options until evidence makes a constraint real.
</health>

<history>
- 2026-10-19: Added WWIV-inspired approved-login/session flow, ANSI BBS identity art and sysop-defined welcome/menu content, a local other-BBS directory, explicit chat-only federation and local-only channels, and working-name status. Aligned existing candidates without activating a milestone.
- 2026-10-19: Recorded explicit user requirements for Citadel-style rooms, IRC-style live chat, terminal/BBS menus, browser console access, and reuse-first legacy game doors. Access protocols, game runtimes, and unresolved interaction details remain research questions; no milestone activated.
- 2026-10-19: Follow-up confirms Citadel-oriented room interaction, one-installation chat initially, future subscription-based chat-channel federation, and network terminal apps as the TTY target. Refined existing candidates including BG-017 without milestone promotion.
- 2026-07-15: Initialized from the draft PRD, transport design, and user direction to prioritize deep architectural research without premature narrowing.
</history>
