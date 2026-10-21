# Faction BBS

**Independent boards. Private communities. Shared frequencies.**

**An independent, terminal-first BBS with a modern foundation and an underground soul.**

Faction brings together the things that made a BBS feel like somewhere: a distinctive ANSI welcome, a sysop's own menus, rooms full of conversation, file areas, late-night chat, and a door into another game. The spirit comes from WWIV, Citadel, and the colorful, heavily customized BBS scenes its founder remembers from the 1980s and 1990s—Vision-X, Vision/2, Oblivion, and the many personal hacks that made each board its own world.

The goal is private, human-scale communities whose operators own their systems and their data. Ordinary SSH terminal access is the initial secure-access direction, with optional private-network-only deployments. A web portal is a future optional access surface; a sysop must be able to keep a board off the public web.

## Status

**Working hobby preview · updated 2026-10-21 · Rust on NetBSD; no supported release yet.**

Faction already runs an approved-membership board with hidden-password login,
ANSI menus, sysop themes, Citadel-style rooms, local command chat, approval/revoke
controls and private export/restore. Arrows/Enter and displayed hotkeys both work.
No JavaScript/TypeScript or public HTTP service is required.

**[Run your first NetBSD board](docs/quickstart-netbsd.md).** Build on the target,
run the first-install preflight/helper, initialize your sysop and explicitly start
the dedicated SSH service. The normal shared backend starts with **eight sessions**.
The installer leaves services disabled and binds loopback unless you choose a
specific interface. You can also [try the local console](docs/local-development.md).

The immediate priority is a useful small board: straightforward setup, an
individual BBS identity, rooms, chat and recovery. The optional
**[Lua arcade](docs/lua-doors.md)** now adds a sysop-editable Doors menu and two
original games: **Signal Hunt** and **Neon Salvage**, with arrows/hotkeys and a
separate game account/evaluator. Local top tens, persistent personal bests,
completed-run counts and an arcade guide give members a reason to return. Scores
survive reconnect/reboot and travel in private exports. Game failure returns to
the board. Optional **[classic server doors](docs/classic-game-doors.md)** now
connect to sysop-selected Telnet endpoints with destination confirmation,
ANSI/CP437 display and a local return key. The connector passed owned-fixture
NetBSD tests; actual TradeWars/DOS titles remain to be validated. Both original
servers and isolated DOS hosting are accepted reuse paths, ahead of rewriting.
Files, private mail, the other-BBS
directory, federation, payments, mesh and browser access remain backlog work.
Sysops can customize [themes and original/licensed art](docs/theme-packs.md),
including optional seasonal colors and identity screens.

Native software validation uses a **NetBSD 11 ARM64 VM**. Physical SBCs and other
NetBSD ports remain untested. The maintained service uses base OpenSSH, a separate
terminal identity, private SQLite state and a small native terminal frontend;
[service lifecycle and worker rollback](docs/service-lifecycle.md) remain available.
Read the [security boundary](docs/process-confinement.md) before deployment.

The earlier 250-member memory/admission work is preserved as
[engineering evidence](dev-docs/research/README.md), with
[experimental isolation](docs/experimental-isolation.md),
[key admission](docs/experimental-admission.md) and
[global admission](docs/experimental-global-admission.md) kept separate from normal
setup. Combined-pressure/recovery, host timing, long soaks and independent
privileged-code review stay open. **250 is a research target, not supported
capacity.** These experiments and physical hardware tests do not block building
the ordinary small-board experience.

[Project direction](docs/project.md), [the roadmap](dev-docs/roadmap.md) and
[GitHub Issues](https://github.com/gonz0w/faction-bbs/issues) separate the working
core from future features. Rust/primary NetBSD is selected in
[ADR 0006](dev-docs/decisions/0006-rust-and-primary-netbsd.md); local storage and
identity use SQLite WAL under [ADR 0008](dev-docs/decisions/0008-local-core-and-credentials.md).

## Enter the board

```text
connect -> log in / register -> local sysop approval required
        -> this BBS's ANSI welcome -> message / quote -> main menu
        -> rooms | chat | files | mail | doors | other BBSes
```

- **Approved membership.** Registration requests access; each sysop decides who enters. Community content requires authentication and local approval.
- **Citadel-style message rooms.** Find a room, enter, catch up on new messages, post, and reply. Detailed reply presentation is still being designed.
- **IRC-style conversation.** Command-driven chat begins within one BBS. Channels may stay local. Future shared channels let participating systems subscribe and talk together.
- **Local file areas and mail.** Community resources, messages, and file-sharing policy belong to the local sysop. Transfer protocols and limits remain research decisions.
- **Doors and original games.** Investigate reuse of classics such as TradeWars, including isolated compatibility hosting where appropriate. Ports may follow later. The optional Lua arcade includes two original examples; no legacy title is supported or bundled.
- **A scene of independent boards.** A local Contacts / Other BBSes directory lets operators submit their systems for discovery. Visiting another BBS means entering that system's own membership process.

## The boundaries matter

Each BBS has its own identity, members, rooms, mail, files, games, directory, and rules. **Only participating shared chat channels may federate.** Local-only channels and other core data remain local. A separately labeled, deliberate public-publishing room is a new optional research category; it never implies automatic private-room export. A directory listing establishes neither federation nor remote approval.

Privacy, security, lightweight operation, and efficient text interaction are core requirements. Telemetry must be minimal and opt-in. Constrained, intermittent, private, and future mesh networks are research targets; no alternative-network support is claimed today. Public discovery, an HTTP portal, and shared chat must not become prerequisites for operating an independent BBS.

**No JavaScript or TypeScript.** This includes product code, tooling, and browser-terminal dependencies. We will compare fast, lightweight implementation options using evidence and representative prototypes. Browser-console feasibility under this constraint remains an explicit open question; we will not quietly introduce a JavaScript terminal library.

## Find your way

| You want to… | Start here |
| --- | --- |
| Understand the product | [Project direction](docs/project.md) and [session flow](docs/session.md) |
| Understand privacy and access | [Security and privacy requirements](docs/security-privacy.md) |
| Explore the BBS look and feel | [Customization direction](docs/customization.md) |
| Contribute research or implementation | [Contributing](CONTRIBUTING.md) and [development guide](dev-docs/README.md) |
| Work with Codex | [Agent instructions](AGENTS.md) and [workflow](dev-docs/workflow.md) |
| Find work and dependencies | [Issue index](dev-docs/issue-index.md) and [GitHub Issues](https://github.com/gonz0w/faction-bbs/issues) |
| Review architecture decisions | [Decision records](dev-docs/decisions/README.md) |
| Report a vulnerability | [Security policy](SECURITY.md) |

## Build the local core

Use Rust 1.96+ and a native C compiler for the local application. Python 3.10+ and Git support repository checks. The console needs a real Unix terminal; application tests and terminal/recovery workflows also ran natively on NetBSD 11 ARM64. Read the [SSH guide](docs/restricted-ssh.md) before configuring the separate development gateway.

```sh
git clone https://github.com/gonz0w/faction-bbs.git
cd faction-bbs
python3 tools/check_repository.py
cargo build --workspace --locked --release
./target/release/faction --state-dir "$HOME/.faction-dev" init
./target/release/faction --state-dir "$HOME/.faction-dev" login --ansi --theme themes/base
```

See [local development](docs/local-development.md) for member registration, sysop approval, posting, recovery and tests. This owner-capable console opens no listener and must never be used as an SSH forced command. It supports plain text and opt-in ANSI menus; the full BBS experience is still being built. Start contributions with an issue and keep this README and affected docs current.

## License and heritage

Faction BBS is licensed under [BSD-3-Clause](LICENSE). Third-party games, source, fonts, and artwork retain their own licenses; nothing in this repository grants rights to redistribute them. The historical names above describe the founder's inspiration, not affiliation or a claim that their code is included.

Earlier planning used the working name Citadel IRC. Its records are preserved in the [historical archive](dev-docs/archive/README.md); they do not govern current work.
