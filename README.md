# Faction BBS

**Independent boards. Private communities. Shared frequencies.**

**An independent, terminal-first BBS with a modern foundation and an underground soul.**

Faction brings together the things that made a BBS feel like somewhere: a distinctive ANSI welcome, a sysop's own menus, rooms full of conversation, file areas, late-night chat, and a door into another game. The spirit comes from WWIV, Citadel, and the colorful, heavily customized BBS scenes its founder remembers from the 1980s and 1990s—Vision-X, Vision/2, Oblivion, and the many personal hacks that made each board its own world.

The goal is private, human-scale communities whose operators own their systems and their data. Ordinary SSH terminal access is the initial secure-access direction, with optional private-network-only deployments. A web portal is a future optional access surface; a sysop must be able to keep a board off the public web.

## Status

**Early working core · updated 2026-10-21 · local console and restricted SSH development gateway; no supported release.**

The working Rust core provides approved local accounts, hidden-password login,
Citadel-style rooms/replies/unread cursors, bounded local IRC-style chat, sysop
controls and private export/restore. [Try the local console](docs/local-development.md).
[Sysop theme packs](docs/theme-packs.md) support ANSI arrows/Enter and displayed
hotkeys, safe CP437 identity, manual seasons and plain/resize recovery.

The maintained [shared terminal backend](docs/shared-service.md) now runs this full
UI through the actual NetBSD rc.d/SSH path. It uses strict owner configuration,
base OpenSSH, a separate terminal UID and small native terminal frontend. Services
are opt-in, with **eight sessions by default** and tested worker rollback using the
same database. [Setup and lifecycle](docs/service-lifecycle.md).
**Rust is selected; NetBSD is the primary target.** No JavaScript/TypeScript or
public HTTP service is required. Software/VM validation drives development;
physical SBC tests follow a usable product.

The service/native frontend now verify both local socket peers, sanitize inherited
authority and apply irreversible process/thread/descriptor/core/locked-memory
ceilings. [Exact confinement boundary](docs/process-confinement.md).

An opt-in [filesystem/exec and IP-policy profile](docs/experimental-isolation.md)
now runs the same UI inside a minimal noexec root after native privilege drop.
It uses explicit group/user NPF rules and a separately installed **setuid frontend**.
This adds trusted startup code and remains experimental pending independent review.
Ordinary binaries and worker rollback remain available; setup never replaces an
operator firewall or silently tunes the host.

An opt-in [SSH key-admission profile](docs/experimental-admission.md) now caps
reserved/authenticated monitors with a dedicated helper UID and a persistent
process-birth registry. It retains base SSH preauthentication controls and adds
no resident admission daemon. Local BBS login and approval remain separate.

Two fresh-boot **250-distinct-member runs** now include a **15-minute active
interval** each: rotating chat senders, 423,798 confirmed peer deliveries,
room/read/post cycles, fresh logins, busy input and silent SSH pressure. All
sampled guards, post-load controls and recovery checks pass on the 2-vCPU / 1-GiB
NetBSD 11 ARM64 VM. Minimum active headroom is **152.97 MiB**; minimum stress/control
headroom is **133.13 MiB**, with zero swap and full SSH/frontend/base-service costs.
[Native evidence and limits](dev-docs/research/0037-sustained-abuse.md).

The [bounded key-lookup follow-up](dev-docs/research/0038-bounded-key-lookups.md)
now charges at most **four lookups per SSH monitor** and requires explicit,
quiescent registry migration. Eight known-key adversaries produce at most 32
key-file reads/commits across 56 helper executions; exhausted calls avoid repeated
key/state IO and cohort liveness scans. Ordinary SSH and local approval still work.
Two further fresh-boot 250-member matrices pass: minimum active **155.14 MiB**,
minimum stress **130.36 MiB**, zero swap. Three-minute abuse/recovery intervals
retain useful chat (250 members before/after, 249 while testing fresh admission).

The [global SSH admission experiment](dev-docs/research/0039-global-ssh-admission.md)
now bounds accepted sockets before base OpenSSH forks using a listener-only native
library, fixed 120-byte persistent budget and 32 volatile source slots. It adds
no resident daemon or proxy. Native fork/restart/reboot/failure tests pass; the
module disappears at session exec. **OpenSSH penalties remain unchanged.** Two
private tunnel peers sharing one outer address pass separate-source penalties,
fresh approved BBS/chat and forged-source UDP controls. Native NetBSD WireGuard
remains experimental; this does not establish 250 private-peer support.

Two new fresh-boot 250-member matrices pass every unchanged sampled guard:
**132.86 MiB minimum stress headroom**, **141.14 MiB during bounded TCP pressure**,
zero swap and 34,611 confirmed peer chat deliveries. The second includes two
configured private peers in server accounting. Independent privileged-loader
review, broader combined worst-case loads, readiness/host timing and long soaks
remain open; retained preflight/setup failures are documented. See the
[experimental boundary and rollback](docs/experimental-global-admission.md).

The 250-session configuration and compact-allocator/pageable-NTP profile remain
explicit research inputs. **Eight remains the default.** Native source penalties
still temporarily deny legitimate callers sharing a source; finite lookup work
is not a global TCP/fork-rate bound. CPU counters, broader overlaps/long soaks,
complete budgets, independent authority review and maintained host policy remain
gates. These software observations are not supported capacity or physical SBC
claims. [Earlier admission evidence and rejected candidate](dev-docs/research/0036-ssh-admission.md)
and [historical executed sources](dev-docs/evidence/README.md) are preserved.
There is no supported network release.

The [decision framework](dev-docs/research/decision-framework.md) is accepted. Research defines [private/poor/interrupted-link journeys](dev-docs/research/0003-constrained-user-journeys.md), [text/ANSI interaction budgets](dev-docs/research/0008-interaction-budget.md), and [small-host resource targets](dev-docs/research/0028-small-host-budgets.md). The owner target is a small SBC/Pi or VM with 1–2 cores, roughly 1 GiB host RAM and 100–250 connected members, while application memory targets are much smaller.

**Selected direction: Rust on NetBSD; local identity/rooms now use SQLite WAL ([ADR 0008](dev-docs/decisions/0008-local-core-and-credentials.md)).** The [native comparison](dev-docs/research/0004-runtime-comparison.md) and [storage/recovery research](dev-docs/research/0009-local-storage-comparison.md) support this proposed experiment choice. The owner has accepted Rust/NetBSD in [ADR 0006](dev-docs/decisions/0006-rust-and-primary-netbsd.md). [ADR 0005](dev-docs/decisions/0005-rust-sqlite-validation-slice.md) defines the integrated security/hardware tests required before production adoption. [ADR 0007](dev-docs/decisions/0007-software-first-and-extension-direction.md) makes physical SBC testing a later hardware-claim gate, not a prerequisite for software work.

[SSH access with optional private-network-only operation](dev-docs/decisions/0004-ssh-and-optional-private-network.md) is accepted product direction. [Native NetBSD 11 ARM64 checks](dev-docs/research/0023-netbsd-dependencies.md) passed for Mio, bundled/pkgsrc SQLite and nine restricted OpenSSH gateway oracles. A separate frontend/broker/member-worker gateway now passes complete application and real SSH tests on NetBSD 11 ARM64; native CMSG portability required a small explicit C/FFI bridge. Production backend/capacity, minimal appliance and real SBC behavior remain unvalidated. [The roadmap](dev-docs/roadmap.md) separates research from future capabilities; [GitHub Issues](https://github.com/gonz0w/faction-bbs/issues) track the work.

The [first local software slice](dev-docs/first-local-slice.md) gives the core a concrete finish line. **Lua menus and isolated game doors come next.** A [16-oracle Rust/Lua boundary probe](dev-docs/research/0010-extensible-ansi-platform.md) now exercises synthetic approval/capability checks, worker limits and crash/deadline cleanup on the developer host; it is not a released plugin sandbox. [Optional extension research](docs/extensions.md) covers permanent-message rooms, modern networks, ANSI shops, payment models and game economies without requiring them in the core.

[Menu and art direction](docs/menus-and-themes.md): Oblivion/2-inspired arrow-key highlighting plus direct hotkeys, sysop-defined themes and optional seasonal identity screens. ACiD/iCE ANSI and Razor/scene ASCII typography guide the visual reference work; no historical artwork is bundled.

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
