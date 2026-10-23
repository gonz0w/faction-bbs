# 0045: Custom grids and bounded SAUCE/iCE import

Date: 2026-10-22. Status: accepted implementation direction for #25; extends 0043.

## Context and alternatives

The owner requests sysop-defined layouts and richer artscene-style ANSI support,
with the classic fallback retained. Keep the existing list as the default;
restrict custom composition to a two-column/five-row grid before arbitrary
coordinate canvases, script-defined neighbors or long-art pagination. This gives
operators control without adding another layout engine or per-member asset copy.

Raw pass-through would allow terminal operations outside the presentation contract.
An external viewer would introduce an executable/dependency boundary. Instead,
extend the existing fixed cell importer for one useful historical metadata/color
profile. Do not bundle archival art or confuse technical import with permission.

## Decision

Base-only `layout="grid"` requires coordinates on every menu entry, unique within
five rows/two columns. Array order remains Tab/plain/compact order. Wide arrows
wrap vertically within a column; horizontal movement chooses the same/nearest
enabled row across columns, ties upward. Missing destinations retain focus.
Disabled cells never acquire focus. Narrow ANSI renders the existing list. Plain
hotkeys and core role/host checks remain. Shared UI reconstructs a borrowed menu
by entry index; replaying Down would incorrectly depend on spatial layout.

Import SAUCE00 Character/ANSi only, with 0/80 width, height <=16 and existing
16 KiB total input/output limits. Locate optional comments from the tail and
require EOF; ignore optional FileSize. Strip all metadata, including controls in
comments, without rendering or trusting it. Record flag bit 0 allows SGR5/25 to
set/clear background intensity; emit canonical bright SGR backgrounds rather than
blink. Other metadata cannot enlarge the canvas or assert a terminal/font feature.
Preserve strict body-control filtering, ownership and declared art provenance.

A new opt-in grid/login example is original Codex-assisted BSD template linework.
It is functional sample material, not approved final scene identity. The existing
seasonal calendar and default installer remain unchanged. No new registry
packages, worker/helper, database schema or per-member art buffer is introduced.

## Evidence, confidence and reversal

[Primary SAUCE revision 00.5 specification](https://www.acid.org/info/sauce/sauce.htm),
retrieved 2026-10-22, defines the 128-byte record, comment-block layout, optional
FileSize and non-blink flag. Confidence is high in this narrow format interpretation;
compatibility with arbitrary archived art, terminal font metrics and final scene
quality is not claimed. The deliberate EOF/dimension restrictions reject some
otherwise valid broader-format assets. Reversal: choose `layout="list"` or base
pack, remove the optional art and restart after preflight; no data migration.

See [plan 0048](../plans/0048-custom-menu-layouts.md) for executed validation and
[theme guide](../../docs/theme-packs.md) for exact authoring and recovery behavior.
Long art/animation and final original/licensed artistic acceptance stay open.
