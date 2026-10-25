# Nightwire terminal compatibility

Updated 2026-10-24. This describes the original software compatibility check of the opt-in
[Nightwire packs](artwork-nightwire.md), not a supported terminal/font list or
approval of the artwork's appearance.

## Keep the intended layout

Use a UTF-8 terminal with a monospace font that covers box drawing and upper/lower
half blocks. The importer reads CP437 assets and emits Unicode cells; do not switch
the SSH terminal to raw CP437. Full login pages need **80 columns and 24 rows of
usable content**, inside any multiplexer. Larger windows work; artwork stays at
its original 80-column width.

A default tmux status bar in a 24-row outer window leaves only 23 content rows.
That is a compact session to Faction. Increase the outer window to at least 25
rows, or choose your own multiplexer status layout. The software fixture disables
only its private tmux status bar to measure full art in a genuine 80x24 viewport;
Faction never changes a member's multiplexer configuration.

The board uses Enter to advance and S+Enter to skip long art. At compact geometry
it keeps a fitting ASCII card and the classic ordered menu. Plain mode needs no
art/Unicode or terminal control sequences. The short pack is a complete one-page
cover for boards that prefer a shorter login. Selecting the earlier/base pack and
restarting after preflight remains the sysop's recovery path.

## What the measurements establish

The recorded private Ghostty, Kitty and Alacritty instances received the compiled
product's canonical long, short and seasonal pages, compact/plain cards and
80/40-column menus. Cursor-position replies are checked at every emitted text
row, including right-edge pending wrap. Repeated runs of every CP437 glyph used
by these frames check cumulative cell width. A real 40x18 window checks compact
cards and the ordered menu separately. Requested Menlo and a deliberately missing
font exercise separate font-selection profiles. Alacritty’s missing-font window
provided only 80x20 cells despite requesting 80x24; that profile was limited to
compact authoring controls rather than full art. Its 40-column menu control is
not a claim about automatic menu layout selection in an 80-column BBS session.
The fixture cannot identify the
actual fallback face from cursor reports.

The [saved evidence](../dev-docs/evidence/nightwire-terminals/README.md) gives
versions, actual viewport geometry, observations, hashes and reproduction steps.
The original NetBSD/SSH/native [login proof](../dev-docs/evidence/nightwire-netbsd/README.md)
remains separate and unchanged. This matrix runs the host authoring preview in
real terminal emulators, not a new NetBSD capacity or SSH-authentication test.

The raw query TTY explicitly emulates the ordinary OPOST/ONLCR output processing
preserved by the BBS adapters. Otherwise an LF-only ASCII card would staircase in
this research TTY even though it displays correctly through the normal BBS path.
An initial probe exposed that fixture mismatch; it was corrected in the probe.
No product/theme byte was changed to compensate for the measurement setup.

## What still needs eyes

Cursor replies establish placement and cell width. They do not read terminal
pixels, certify glyph shape/half-block seams, identify a font face or prove a
terminal's RGB palette. The existing canonical cell checks verify the selected
SGR colors, geometry and visible foreground/background differences; user palettes
can still change contrast. The SVGs use a reference palette and forced reference
cell spacing, so they cannot establish real-font quality.

The current dark artwork replaces the measured skyline/receiver composition.
Its canonical previews and local login/gallery PTYs are checked separately; the
old emulator matrix is not a fresh observation of these new bytes.

Review the cover lettering, split-wing sigil, carrier notes, seasons and menu together
on the intended terminal before accepting a board identity. Prefer a different
local font or palette, the short pack or ASCII fallback when necessary. No desktop
capture/automation permission is requested. Animation, more formats, broader
platforms and owner visual acceptance remain future #13/#25/#29 work. Hardening
stays later and the saved playground stays off.
