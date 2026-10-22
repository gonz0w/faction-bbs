# BBS menu and theme polish (#25)

Owner request 2026-10-22: merge checked #121, then improve BBS menus and ANSI themes;
classic fallback stays, hardening #106–#111 remains later.

## Outcome and scope

A framed lightbar main menu with a deliberate title, full-row focus, readable
unavailable markers, and optional sysop action descriptions on wide terminals.
Use actual TTY columns for compact/wide composition. Keep arrow/Enter/hotkeys,
auth checks, existing identity import controls, ASCII/plain recovery and bounded
old/new-row updates. New optional declarative fields must reject invalid packs
atomically and seasonal overrides must never remap commands or descriptions.
No generic logo, redistributed scene art, new graphics passthrough, dependencies,
listener, session cache, helper, schema change or changed defaults.

## Steps

- [x] Merge #121 after its required check succeeds into its existing feature base.
- [x] Preserve all 115 application, seven harness and two reproduction inputs and
  the unchanged real-terminal record before editing runtime/theme sources.
- [x] Implement shared renderer/theme contract and wire actual widths in both UIs.
- [x] Coordinate shipped base/autumn/winter palettes and document customization.
- [x] Verify cell layout, navigation/delta output, invalid-pack rollback and plain
  fallback; run host and native NetBSD software checks as available.
- [x] Update README/AGENTS/changelog and prepare a focused issue-linked PR.
- [x] Publish [PR #122](https://github.com/gonz0w/faction-bbs/pull/122) and record its link on #25.

## Validation and limits

Use deterministic renderer/cell assertions plus real local/worker and shared
NetBSD PTYs. Measure encoded full-frame and focus updates at minimum/normal widths,
including longest allowed text and enabled Files. Keep each frame below 4 KiB,
focus updates below 512 bytes, and the existing shared 32-KiB/five-second queue.
GUI pixel/glyph/artwork acceptance is not implied by PTY/cell validation; existing
#118 profiles are historical observations, not new measurements of this change.
No physical SBC requirement or fresh 250-user claim. Saved playground stays off.

## Measured outcomes and remaining work

154 Rust tests pass on host and native NetBSD ARM64; 109 host Python tests,
strict host Clippy/rustfmt and selected dependency checks pass. Native build used
Rust 1.96.0; host used 1.98.0. Local/worker PTYs pass on both OSes. Shared native
frontend passes 22 controls; three additional actual restricted member SSH/native
routes pass. See evidence/menu-themes-netbsd/results.json and its README for exact
source/binary identities, output tables, commands and limits. New preview SVGs
regenerate deterministically and were checked as original cell-layout artifacts.

The first compile caught the local adapter's optional TTY-size return type;
handle missing/out-of-range geometry with plain fallback before opening raw mode.
The fixed candidate then passed both compiler/toolchains and native controls.
No runtime behavior was inferred from the rejected compile. Maximal fixture full
frames are 1311–1974 bytes; focus updates 170–248 bytes. Existing theme packs keep
defaults; older binaries reject new fields and need matching packs for rollback.

Original commissioned/licensed scene identity artwork, actual visual glyph/pixel
acceptance, calendar/timezone/overlap scheduling, grid neighbors and long-art
import remain open in #25. Do not close that issue or relabel the old emulator
matrix. Hardening stays later. The saved playground stays off; only the owned
disposable clone and temporary vendor cache are removed after validation.
