# 0051: Nightwire across real terminals

Issue: [#25](https://github.com/gonz0w/faction-bbs/issues/25)
Status: implemented; focused PR handoff pending

## Goal and constraints

After merging checked #126, measure the compiled Nightwire pages, seasonal/short
packs, compact/plain identities and menus in owned fresh Ghostty, Kitty and
Alacritty instances. Include missing-font and multiplexer observations when
available. Preserve classic fallback, existing defaults and immutable evidence.
No pixel/font-shape, physical SBC, capacity or hardening claim. No personal
terminal config, installed application or saved playground changes.

## Progress

- [x] Merge #126 after its exact-head required check passes.
- [x] Read product/theme/workflow requirements and existing terminal fixtures.
- [x] Build a bounded artwork-specific real-emulator cursor probe.
- [x] Run the matrix; correct probe newline/viewport defects, no product defect reproduced.
- [x] Check unchanged prior inputs and classic fallback; record limitations.
- [ ] Update docs, issue and focused PR.

## Steps and commands

Use `prepare_terminal_reference.py` for digest-checked, private no-JS references.
`probe_artwork_terminal.py` sends compiled `preview_theme` output to real TTYs
and checks cursor reports at each text-row boundary plus repeated CP437 glyphs.
Canonical row/style validation and source/frame hashes supplement observations;
DSR cannot establish actual RGB palette, font face or visual artwork acceptance.
Do not rewrite historical records or change pinned bytes without preserving them.

## Discoveries and decisions

2026-10-22: #126 required Documentation and policy succeeded at head 9c2c936.
Squash merge 946819e into the existing article feature base. Artwork/runtime inputs
remain unchanged while adding independent tooling. The raw research TTY uses
explicit CRLF for all authoring output, corresponding to ordinary cooked output.
Initial unprocessed compact cards staircased only in the probe. Default tmux left
23 rows; the final private fixture explicitly disables its status bar. Alacritty
with the missing-font request returned 80x20, so that profile runs only five compact
authoring controls; no automatic layout-selection/font-face claim follows.

## Validation and outcomes

- Actual Ghostty 1.3.1, Kitty 0.49.2, Alacritty 0.17.0; private tmux 3.8 and
  installed GNU screen 4.00.03: all 15 profiles pass. Eleven full profiles and four
  compact-control profiles yield 240 frames, 2,476 cursor checkpoints and 187
  repeated-glyph observations (17 CP437 glyph types in each full profile).
- `cargo build --release --locked -p faction-terminal --example preview_theme`
  succeeds. `check_nightwire.py` passes all 18 canonical profiles;
  `smoke_nightwire.py` passes seven local/worker login routes.
- Python discovery: 115 tests pass. Repository policy/links/provenance and all
  32 saved-evidence guards pass; historical source guard retains 18 snapshots /
  30 historical record identities. All 167 distinct original Nightwire measured
  inputs and its complete original result remain unchanged.
- No fresh native VM/Rust-suite/capacity run: product/runtime/artwork are unchanged.
  Original native NetBSD/SSH evidence stays explicitly separate.
- Owned GUI processes, private references and fixture-only preferences removed.
  Saved playground remains off. No screen-capture permission requested.

Remaining: owner visual/font/palette acceptance and broader platform coverage.
No product defect reproduced, so no speculative runtime/art alteration. Hardening
stays later; #25 stays open. Focused PR and issue handoff follow local validation.
