# Citadel IRC Agent Guide

## Project Direction

Citadel IRC is a self-hosted, local-first BBS/forum/chat platform for independent communities. It must remain useful over conventional IP and constrained, intermittent, mesh, private, or future networks without making any one transport, client, topology, or runtime mandatory.

The near-term objective is research and architecture validation, not feature implementation. Preserve optionality until comparative evidence and small prototypes justify a decision.

The user-confirmed product experience is terminal-based: Citadel-style rooms for message boards, IRC-style command-driven live chat, BBS-style menus, and doors to ANSI games. Direct terminal access and an embedded webpage console are required. Prefer reuse of existing classic games where feasible; ports are possible future work. These are interaction requirements, not commitments to Telnet, the IRC protocol, any terminal library, or any legacy runtime. See `.planning/PROJECT.md` for the 2026-10-19 direction record and unresolved questions; this supersedes web-first/optional-terminal language in the seed drafts.

The direct-terminal target is a terminal app over a network. Room interaction should follow Citadel's room/message model. Global chat initially serves one installation; future optional federation should allow independent systems to subscribe to shared chat channels (BG-017), without making federation mandatory for ordinary operation.

Citadel IRC is a working name. The session/menu model is WWIV-inspired: login or registration, local sysop approval before community access, BBS-specific ANSI art after approved login, then a main menu with sysop-defined welcome/broadcast content. Each sysop hosts local content and a Contacts / Other BBSes directory for registering and discovering other systems. Only participating shared chat channels federate; all other BBS data remains local, and local-only chat channels must be possible. Directory listings do not grant remote access or establish federation.

## Non-Negotiable Product Constraints

- Favor independent operation, data ownership, exportability, privacy, and minimal opt-in telemetry.
- Preserve a useful low-bandwidth baseline. Treat text, ASCII, and ANSI as active design inputs; do not assume that richer clients are the only clients.
- Keep community/domain behavior independent of transport-specific addressing, routing, discovery, and connection mechanics.
- Prefer simple local deployments and graceful degradation over distributed complexity without a demonstrated use case.
- Sysop customization must have explicit permissions, lifecycle, upgrade, and failure-isolation boundaries.

## Research-First Workflow

- Read `.planning/PROJECT.md`, `.planning/INTENT.md`, and relevant backlog items before proposing work.
- Treat `docs/` as draft input, not approved technical design.
- For each material decision, record the question, constraints, alternatives, primary sources with retrieval dates, tradeoffs, confidence, and a falsifiable prototype or validation plan.
- Do not select a language, framework, protocol, federation model, scripting runtime, or transport solely because it is familiar or currently popular.
- Keep research bounded to a named decision. Surface unresolved conflicts and unknowns rather than concealing them with a recommendation.

## Backlog Discipline

- `.planning/backlog/captured/` contains durable future-milestone candidates; it is not an active roadmap.
- Add future work through `/pd-new-backlog` when available, or use the `BG-###--short-kebab-title.md` schema in `.planning/backlog/captured/` with required provenance and problem/outcome sections.
- Before adding an item, check `.planning/BACKLOG.md`, captured items, and active roadmap state for overlap. Cross-reference rather than duplicate.
- Keep open questions and dependencies explicit. Only milestone intake may promote a candidate into accepted roadmap work.
- Update `.planning/BACKLOG.md` when adding, merging, promoting, or retiring an item.

## Engineering and Safety

- There is no implementation stack or VCS repository yet. Do not invent one in plans or code.
- Keep core, adapters, client surfaces, persistence, and extensions conceptually separable until research validates their boundaries.
- Never add credentials, network endpoints, or generated secrets to the repository.
- Validate artifacts and experiments with the smallest relevant check; report unavailable checks plainly.
