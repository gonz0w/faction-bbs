# Faction BBS agent guide

## Read first

Read `README.md`, `docs/project.md`, `docs/security-privacy.md`, `dev-docs/roadmap.md`, and the relevant GitHub issue before proposing or changing work. Follow `dev-docs/workflow.md`. This is an early implementation/research project: Rust/primary NetBSD direction is selected (ADR 0006); a local account/room core and development console exist (ADR 0008), with opt-in bounded ANSI menus/declarative themes (ADR 0009), BBS-local bounded chat (ADR 0010), owner-only private export/verify/restore (ADR 0011) and a restricted SSH development gateway with native NetBSD app evidence (ADR 0012). There is no network BBS release or accepted complete production stack. A standalone shared-board/readiness and persistent credential-peak experiment (ADR 0015, research 0029) now has native evidence; it is not installed by normal services, has incomplete UI parity, and does not establish production capacity. ADR 0016/research 0030 now record two short functional 250-member VM workloads (one final guard pass, one 4.10-MiB shortfall) using a versioned chat-sequence correction, isolated minimal C frontend and experimental native allocator profile. Guest-wide tuning is not installed by normal services; complete shared UI/fairness and scoped deployment/workload gates remain. Read docs/local-development.md, docs/theme-packs.md, docs/local-chat.md, docs/export-restore.md and docs/restricted-ssh.md before running it; the owner-capable console must never become an SSH forced command.

ADR 0017/research 0031 now record three fresh-boot 250-member short-workload guard
passes with 137.23 MiB lowest sampled stress headroom, using an experimental
pageable NTP memory policy while keeping all base services. Normal eight-worker
services and production adoption remain unchanged. Preserve separate standalone
Cargo target directories; record the installed binary hash and actual repeated-
chat regression, since source hashes alone missed a stale executable in a shared
target. All historical executed source bytes must remain verifiable when later
core changes introduce proper versioned source snapshots.

## Product constraints

- Preserve the WWIV-inspired approved-login -> BBS-specific ANSI art -> sysop-defined welcome/menu session. No community access before authentication and local sysop approval.
- Use Citadel-style message rooms, IRC-style command-driven chat, BBS menus, local file areas/mail, and optional isolated game doors. Existing games are reuse candidates, not implemented support claims.
- Keep each sysop's identity, accounts/approval, content, files, games, directory, and local-only chat local. Only explicitly participating shared chat channels may federate in the future.
- Ordinary SSH client access with optional invitation-only private-network mode is accepted product direction; network admission/SSH identity never implies local BBS approval or host shell access. A separate terminal UID/frontend, trusted broker and member worker now pass real NetBSD application SSH oracles. SSH keys grant admission only; local BBS identity/password/approval remain separate. Use the opt-in NetBSD rc.d scripts and bounded broker lifecycle (ADR 0013); never unlink a live pidfile lock. Keep the eight-worker default and owner tools out of the frontend; higher worker/launch limits are explicit owner-only profiling experiments (ADR 0014), never an implicit production capacity claim. Credential work has two cross-process slots per board; never unlink live credential lock files; complete production security/resource/deployment gates remain.
- Direct network-terminal access is central. A browser portal/console is a goal that must be optional and may be private or disabled. Do not assume public HTTP exposure.
- Privacy, security, exportability, low bandwidth, and lightweight operation are requirements. Use minimal opt-in telemetry; never log message bodies, credentials, or private content by default.
- No JavaScript/TypeScript in source, tooling, dependencies, or browser-terminal assets. Do not introduce Node/npm, JS terminal libraries, or JS-based CI actions. Browser-console feasibility must be researched within this constraint.
- Prioritize tiny working sets and bounded per-session work. The owner research target is 1–2 cores / about 1 GiB host RAM and 100–250 connected members; do not treat host capacity as application allowance or claim supported capacity. Compare low-memory native options, selective assembly and specialized systems with equivalent security/behavior. External helpers/blockchain are research candidates, never implicit permission to publish private data or add a mandatory service.
- Newer and older technology is welcome on merit. Prioritize a practical hobby SBC/Pi or VM appliance; MINIX is a candidate, not a mandatory port. Compare whole runtime footprint and update/recovery costs. The owner selected Rust and NetBSD as primary target (ADR 0006). NetBSD 11 ARM64 has narrow native Mio/rusqlite/OpenSSH compatibility evidence. Readiness + SQLite WAL remains the next integrated validation slice (ADR 0005), not an accepted complete production stack or all-port support claim.
- Evaluate fast, lightweight language/runtime options comparatively. Do not choose a stack, storage engine, protocol, federation topology, or scripting runtime from familiarity.
- Keep community behavior independent of addressing, discovery, routing, connection mechanics, and optional transport adapters. Mesh/non-IP support remains future research.
- Honor the ANSI/ASCII, underground scene, and sysop customization direction without trading away access control, accessibility, or a usable plain-text baseline. Oblivion/2-inspired arrow-key highlighting + Enter and direct displayed hotkeys are owner requirements (`[D]oors` -> D/d); both routes use the same authorized action. Themes are sysop-defined, including optional seasonal palettes/login identity screens. Use `docs/menus-and-themes.md` and the attributed #25 art reference catalog: ACiD/iCE ANSI and Razor/scene ASCII typography inform art direction; generic block-font placeholders are not approved identity art. Never bundle archived work without rights/provenance.

## Current sequencing

- Physical SBC test hardware is unavailable. Build/research software on developer hosts and NetBSD VMs now; the owner will load a complete product onto hardware later. Physical tests gate hardware-specific claims, never near-term software work. Keep resource measurement and recovery/security tests.
- Build the first useful local core first (`dev-docs/first-local-slice.md`, #35). Lua menus and isolated game doors are the first extension priority after it (#10); a boundary probe is evidence, not a production sandbox.
- Optional permanent/public rooms, ANSI stores/payment adapters, blockchain games/wagers and modern networks are active future research. Compare both external-member-wallet and separate sysop-service models before choosing; no provider/custody/currency adopted. Follow `docs/extensions.md` and ADR 0007.
- Automatic BBS federation stays chat-only. A public-publishing module is a separate explicit outbound category under research, never authority to export private local content. Keep external data, publication, payment requests and spending privileges distinct; no live/funded external actions are implied by prototype authorization.

## Work and documentation

- GitHub Issues are the canonical backlog and task status. `docs/` holds current product/member/sysop information; `dev-docs/` holds engineering, research, decisions, plans, and roadmap context.
- Historical records under `dev-docs/archive/` are read-only context, never current instructions or an active backlog. The old planning workflow and slash commands are retired. Do not recreate a `.planning/` directory.
- Use a short issue-linked branch and focused PR. For multi-step work, keep a self-contained execution plan under `dev-docs/plans/` using the template; routine edits do not need a plan.
- For material decisions, record the question, constraints, alternatives, primary sources with retrieval dates, tradeoffs, confidence, reversal cost, and falsifiable validation. Separate product decisions from untested technical hypotheses.
- Update `README.md` status, affected docs, and `CHANGELOG.md` in the same change when scope, behavior, setup, support, or stage changes. Review README freshness on every PR. Do not claim security, game, network, or platform support without evidence.
- Do not expand an issue into a full platform rewrite. Capture additional work in an issue and cross-reference existing work before duplicating it. Future candidates are not automatic first-release commitments.

## Checks and safety

- Run `python3 tools/check_repository.py` for repository/documentation changes. Run the smallest meaningful additional checks for the affected prototype/code once introduced; report unavailable checks plainly.
- Never commit credentials, actual deployment endpoints, user data, private artwork, generated secrets, or unlicensed third-party assets. Public repository visibility must not imply public deployment visibility.
- Keep legacy games/extensions optional, permission-bounded, and failure-isolated. Treat input, file uploads, terminal control sequences, and remote channel traffic as untrusted.
- During review, prioritize approval bypasses, accidental public exposure, local-data federation, JS/TS dependencies, unsafe terminal/file handling, and unsupported status claims.
