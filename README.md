# Faction BBS

```text
+------------------------------------------------------------+
|  F A C T I O N   /   B B S                                  |
|  independent boards. private communities. shared frequencies.|
|                                                            |
|  [M]essage rooms   [C]hat   [F]iles   [D]oors                 |
|  your sysop. your scene. your system.                       |
+------------------------------------------------------------+
```

**An independent, terminal-first BBS with a modern foundation and an underground soul.**

Faction brings together the things that made a BBS feel like somewhere: a distinctive ANSI welcome, a sysop's own menus, rooms full of conversation, file areas, late-night chat, and a door into another game. The spirit comes from WWIV, Citadel, and the colorful, heavily customized BBS scenes its founder remembers from the 1980s and 1990s—Vision-X, Vision/2, Oblivion, and the many personal hacks that made each board its own world.

The goal is private, human-scale communities whose operators own their systems and their data. Ordinary SSH terminal access is the initial secure-access direction, with optional private-network-only deployments. A web portal is a future optional access surface; a sysop must be able to keep a board off the public web.

## Status

**Research and repository foundation · updated 2026-10-19 · no release or runnable BBS yet.**

This repository contains the product requirements, development workflow, preserved design history, and a GitHub issue backlog. There is no selected production language, implemented BBS server, verified door-game support, or supported deployment command. Privacy, security, and performance statements below describe requirements to validate, not guarantees of existing software.

The [decision framework](dev-docs/research/decision-framework.md) is accepted. Research defines [private/poor/interrupted-link journeys](dev-docs/research/0003-constrained-user-journeys.md), [text/ANSI interaction budgets](dev-docs/research/0008-interaction-budget.md), and [small-host resource targets](dev-docs/research/0028-small-host-budgets.md). The owner target is a small SBC/Pi or VM with 1–2 cores, roughly 1 GiB host RAM and 100–250 connected members, while application memory targets are much smaller.

**Recommended next validation slice: Rust with readiness-based I/O and SQLite WAL.** The [native comparison](dev-docs/research/0004-runtime-comparison.md) and [storage/recovery research](dev-docs/research/0009-local-storage-comparison.md) support this proposed experiment choice. Go remains a measured fallback. [ADR 0005](dev-docs/decisions/0005-rust-sqlite-validation-slice.md) defines the integrated security/hardware tests required before production adoption.

[SSH access with optional private-network-only operation](dev-docs/decisions/0004-ssh-and-optional-private-network.md) is accepted product direction. No SSH backend or appliance OS is selected. Local probes do not establish complete server performance or network support. [The roadmap](dev-docs/roadmap.md) separates research from future capabilities; [GitHub Issues](https://github.com/gonz0w/faction-bbs/issues) track the work.

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
- **Doors and original games.** Investigate reuse of classics such as TradeWars, including isolated compatibility hosting where appropriate. Ports may follow later; no title is currently supported or bundled.
- **A scene of independent boards.** A local Contacts / Other BBSes directory lets operators submit their systems for discovery. Visiting another BBS means entering that system's own membership process.

## The boundaries matter

Each BBS has its own identity, members, rooms, mail, files, games, directory, and rules. **Only participating shared chat channels may federate.** Local-only channels and everything else remain local. A directory listing establishes neither federation nor remote approval.

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

## Work on the foundation

Python 3.10 or newer and Git are sufficient for the repository checks. Python is repository tooling, not a production language decision.

```sh
git clone https://github.com/gonz0w/faction-bbs.git
cd faction-bbs
python3 tools/check_repository.py
```

There is no application to launch yet. Start with an issue, read its dependencies, and use the research/decision templates for architecture work. Keep this README's status and the relevant documentation current whenever behavior, scope, setup, or support changes.

## License and heritage

Faction BBS is licensed under [BSD-3-Clause](LICENSE). Third-party games, source, fonts, and artwork retain their own licenses; nothing in this repository grants rights to redistribute them. The historical names above describe the founder's inspiration, not affiliation or a claim that their code is included.

Earlier planning used the working name Citadel IRC. Its records are preserved in the [historical archive](dev-docs/archive/README.md); they do not govern current work.
