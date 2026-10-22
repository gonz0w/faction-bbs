# 0043: Bounded BBS main-menu composition

Date: 2026-10-22. Status: accepted implementation direction for #25.

## Context and alternatives

The owner requests an Oblivion/2-style lightbar/hotkey BBS with sysop themes and
classic fallback. The old menu already has safe action mapping and delta updates
but is a sparse fixed-column list. Retaining it alone doesn't improve composition.
Arbitrary grids/Lua layouts would add neighbor maps, geometry and privileges
before the ordinary menu is polished. Rich bitmap identity art is separately gated
by protocol capabilities, actual artistic acceptance and redistribution rights.

## Decision

Keep one ordered vertical menu and its existing decoder/authority model. Use
actual TTY width for centered 38/76-cell ASCII panels, never a TERM/brand guess.
Wide rows add bounded inert action descriptions. A base-only 24-byte title and
30-byte descriptions extend the existing strict version-1 schema; all ten known
actions may coexist. New fields have safe defaults; old readers reject new packs.
Manual seasons override only already-allowed palette/identity fields. Keep fixed
help/recovery instructions, a non-color marker, explicit unavailable status and
whole-row lightbar background. Only old/new rows repaint on navigation.

## Costs, validation and reversal

One additional bounded description String per shared theme entry and one title
belong to the immutable theme, not a new per-member cache. Render work remains
bounded by ten rows and terminal geometry, with no new dependencies/helpers or
changes to credentials/admission/queue limits. Validate through an independent
cell oracle, local/worker PTYs and the native shared frontend. SVG authoring
previews are illustrative and cannot certify an actual emulator/font or identity
art. Restore the prior renderer with a matching older pack to reverse this change;
there is no database migration. See plan 0046 for actual measured results.

Primary requirement/contracts examined 2026-10-22: owner #25; docs/theme-packs.md,
docs/menus-and-themes.md; theme.rs, terminal.rs and shared ui.rs at merged #121
(`f6f4588`). Historical external-art research remains research 0025, not newly
executed software or new redistribution permission. Confidence: high for bounded
ASCII layout; actual visual/font/art acceptance and calendar/grid work remain open.
