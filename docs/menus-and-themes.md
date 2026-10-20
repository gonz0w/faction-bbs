# Menus, sysop themes and seasonal identity

Status: confirmed owner direction, 2026-10-19. A local vertical-menu/theme baseline
is implemented; [the pack guide](theme-packs.md) states exact supported behavior.
The broader design below includes future requirements. No scheduler, historical
art pack or network BBS release is available.
Issues: #13/#21/#25/#29/#31. [Visual references](../dev-docs/research/0025-menu-and-art-direction.md).

## Interaction that belongs on a BBS

Use Oblivion/2 as the menu experience reference. On capable terminals, arrow keys
move a visible selection/lightbar; Enter activates it. Direct hotkeys also work:
`[D]oors` means pressing `D` goes to Doors without navigating the lightbar first.
Accept upper/lowercase for these displayed ASCII hotkeys. Both paths select the
same action and perform the same current approval/role checks.

- Movement highlights an option; movement alone never launches an action.
- Menus can be vertical or arranged by the sysop. A multidirectional layout needs
  explicit/predictable neighbors; a decorative picture cannot silently redefine keys.
- Focus must remain visible through a marker/reverse/bold treatment as appropriate,
  not color alone. Preserve a readable ASCII/hotkey path on limited terminals.
- Themes may change labels/layout/presentation; keep action IDs and displayed
  hotkeys consistent. Validate collisions and missing destinations before activation.
  Seasonal color/art changes must not unexpectedly remap commands.
- Input is context-specific: typing a letter in an editor/chat prompt is not a menu
  command. Fragmented arrow escape sequences must never leak trailing letters into
  hotkey dispatch. CR/LF handling must not activate twice.
- Unauthorized options cannot be used through either navigation path. Hiding or
  graying an option is presentation, never the permission check.

Exact escape grammar, timeout/recovery, wrapping, resize/focus and grid conventions
are #13/#25 implementation questions. Prefer updating the old/new selection regions
over repainting a large artwork for every arrow key; measure actual encoded bytes.
Provide explicit refresh/help/return controls and safe recovery after interrupted output.

## Theming is foundational

Sysops define the board's visual identity: palettes, normal/focused/hotkey/disabled
styles, prompts, menu layouts, headers/footers, login identity art, welcome/quote,
logout screens and ASCII alternatives. These are first-class configurable surfaces,
not one built-in color scheme with a logo field. The initial core must reserve and
exercise these surfaces; not every later scheduling/editor tool is a first release gate.

Proposed theme packs declare an ID/version, compatible theme format, terminal
dimensions/capabilities, encoding/font assumptions, asset slots and provenance.
Load/validate atomically with a known-good fallback and rollback on invalid/missing
assets. Keep data/actions/ACLs separate from layout and art. Basic themes do not
need Lua; later Lua menus/doors use the bounded extension contract.

ANSI is a primary product surface. Preserve CP437/text-cell composition, palette,
shade/half-block detail, intended aspect/font and safe control semantics where
supported. Research SAUCE metadata and iCE bright-background versus blink behavior;
do not reinterpret an artist's file as arbitrary Unicode or raw trusted escape bytes.
Plain text is an alternative mode, not the visual quality target for capable sessions.
Long scrolling identity art and a compact interactive menu can be different assets.

## Seasonal themes

Support the direction of sysop-defined seasonal colors and login/identity screens.
The initial step may be manual pack/override selection; calendar automation remains
an optional #25 design item. Proposed scheduling supports named date windows,
the sysop's timezone/calendar, explicit priority for overlaps and a manual override.
Do not hardcode Northern Hemisphere seasons or a particular holiday calendar.

Use a base theme with optional seasonal palette/art overrides. Missing overrides
fall back to the base. Choose a coherent version for the entry/art/menu flow;
switch existing sessions only at a safe menu boundary or on next login, never
overwrite a message draft, live chat input or running door at midnight. Test year
boundaries, local dates, conflicts, disabled scheduling and broken assets.

## Art direction and review

Study ACiD and iCE ANSI work and Razor 1911/scene ASCII typography as the owner
requested. The goal is artist-shaped composition, distinctive lettering, deliberate
negative space, controlled shading/color ramps and a recognizable board identity.
No group or era has only one style. A plain boxed masthead, automatic image-to-text
conversion or generic LLM block-font logo is not an approved identity screen.

Use specific attributed references and describe which composition/lettering/palette
features inform a new work. Preserve artist signatures/credits. Favor original or
explicitly authorized human-authored art; AI output is draft material requiring the
same native-cell/terminal review and rights/provenance checks. Do not accept an art
screen merely because it contains box-drawing characters or is labeled "ANSI".

Historical archive access is a study reference, not a redistribution license.
Themes/assets need creator, source, permissions, modifications and encoding/dimension
records before distribution. The reference catalog links examples without bundling
their files, screenshots, group logos or incidental historical text.
