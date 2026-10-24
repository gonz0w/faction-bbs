# 0002: Faction BBS identity, BSD license, and no-JS constraints

Status: accepted product constraints; technical choices unvalidated
Date: 2026-10-19
Owner: project founder
Issue: BBS-004, BBS-024, and BBS-032 in the [GitHub issue](https://github.com/gonz0w/faction-bbs/issues)
Supersedes: Citadel IRC working name and older optional-terminal/web-first draft framing

## Question and decision

Which constraints govern the public foundation? The founder requested a name in the direction of “Faction BBS,” selected BSD 3-Clause, and explicitly prohibited JavaScript/TypeScript everywhere, including browser-terminal dependencies. Adopt Faction BBS / `faction-bbs` and [BSD-3-Clause](../../LICENSE). Preserve terminal-first operation, private/optional HTTP access, privacy/security, and fast/lightweight system requirements.

## Alternatives and tradeoffs

Other names were proposed but the founder preferred the Faction direction. Keep earlier names only in migration/history. No browser JS exception is authorized; the prior xterm.js research lead is superseded. An HTTP portal/browser console remains a goal but must be evaluated without JS dependencies. If feasible behavior cannot satisfy these constraints, record the conflict and alternatives rather than introducing an exception silently.

At this foundation decision no production language, transport, federation protocol, database, or extension runtime was selected. [ADR 0006](0006-rust-and-primary-netbsd.md) later selects Rust and primary NetBSD; the remaining choices still require validation. Repository checks use Python's standard library only; this is not a product runtime selection. CI uses shell/Python and no JavaScript-based marketplace actions.

## Evidence and confidence

Primary authority: founder statements and clarification answers in this project's chat on 2026-10-19. High confidence in the explicit license/no-JS requirements; naming adopts the requested Faction direction. [BSD-3-Clause license template](https://api.github.com/licenses/bsd-3-clause) retrieved 2026-10-19; license text is included in the repository. Basic repository-name checks are availability checks, not a broader identity clearance.

## Validation and unknowns

Check consistent active project naming, actual BSD-3-Clause text, absence of JS/TS sources/assets/tooling, and accurate unimplemented status. Compare fast/lightweight runtime options with prototypes. Research optional no-JS browser access as a separate falsifiable question; no browser runtime support claim is made today.
