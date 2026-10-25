# Menus, sysop themes and seasonal identity

Status: owner direction with menu polish implemented, 2026-10-22. List/grid menus and bounded identity import
are implemented; [the pack guide](theme-packs.md) states exact supported behavior.
The broader design below includes future requirements. Annual next-login scheduling is now available; historical
art packs and a supported network BBS release remain separate.
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
supported. The bounded SAUCE/iCE profile is documented in the pack guide; broader
format/font compatibility remains research.
Do not reinterpret an artist's file as arbitrary Unicode or raw trusted escape bytes.
Plain text is an alternative mode, not the visual quality target for capable sessions.
Long scrolling identity art and a compact interactive menu can be different assets.

## Seasonal themes

Support the direction of sysop-defined seasonal colors and login/identity screens.
Manual pack/override selection and annual next-login calendars are implemented.
The calendar supports named date windows, explicit UTC/system-local rules,
overlap priorities and a manual override; see the [pack guide](theme-packs.md).
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

## Live-playground follow-up stories

The owner confirmed that the current functional screens need a dedicated polish
pass: clear/replace distinct views, consistent contextual headers/help/back paths,
formatted chronological room messages, width-aware wrapping, reply/quote context
and bounded pagination (#21/#25). Preserve drafts and deliberate live-chat scrolling.
[Modern terminal story #96](modern-sysop-and-communications.md) adds capability-based
colors/text graphs/graphics research with safe fallback. The local sysop dashboard
#95 uses the same board identity where appropriate; richer rendering never gives
an in-BBS role host authority or lets a sysop theme replace trusted local mail UI.

The [shared room reader](room-reading.md) now implements the first presentation
polish: themed ANSI section clears, room/author/reply headers, separators and
wrapped message pages. Plain text follows the same journey. File-reference tokens
remain intact for native shortcuts. Optional native Unicode/graphics and seasonal calendars now extend this
foundation; further composition and final artwork remain #21/#25/#96 work.

## Current lightbar presentation (#25)

The local/worker and shared-service main menu now uses the same compact/wide
framed lightbar with sysop title and optional wide action descriptions. See the
[pack guide and actual-renderer previews](theme-packs.md). Themes retain 16-color
ASCII menu chrome, clear section changes, explicit unavailable markers and bounded
focus updates; classic/plain recovery needs no graphics or Unicode negotiation.
Manual autumn/winter overrides stay independent of command mappings. The new
local preview example and stdlib SVG renderer let sysops validate layout without
opening a live board. These are original interface elements, not final approved
identity art. Commissioned/authorized art, visual font review and
broader formats remain open; don't infer those from the previews.

The [calendar and login guide](theme-packs.md#seasonal-calendar-and-login-screens)
now implements UTC/system-local annual windows, explicit overlap priority,
manual/disabled recovery and next-login selection. A shared preloaded catalog
keeps each approved session coherent. Original seasonal ANSI/compact ASCII
examples exercise the common bounded presenter; the shipped calendar is disabled.
Opt-in [custom grids](theme-packs.md#custom-grid-layouts-and-cp437-art) now add
explicit two-column/five-row positions and spatial arrows while keeping ordered
Tab/hotkeys and compact/plain recovery. The importer accepts bounded SAUCE00
Character/ANSi metadata and non-blink iCE backgrounds, never raw escape streams.
An original CP437 sample demonstrates the path. Final scene identity-art
acceptance, animation and larger formats remain #25 work. Optional long-form
login paging is now implemented in the bounded profile below.

The [long-form login guide](theme-packs.md#long-form-login-art-and-paging) now
supports explicit 80x256 / 64 KiB assets, sixteen-row pages, Enter/skip and
classic/compact recovery. This retains the short-art importer contract and does
not imply unlimited scrolling, animation or compatibility with every scene pack.

## Original artwork candidate

The [Nightwire set](artwork-nightwire.md) is original CP437 artwork with a custom
angular FACTION wordmark, split-wing sigil and abstract signal cuts. Charcoal/steel
and restrained red, amber or deep-blue accents replace the earlier pastel skyline.
Short cover, three-page identity, seasonal colors and compact ASCII cards now
form one coordinated set. It is opt-in with exact source/rights and reproducible
canonical previews; the default installer and all existing packs stay unchanged.
Owner visual acceptance and actual terminal/font review remain distinct from
software checks. No archival artwork or generic font logo is bundled.
