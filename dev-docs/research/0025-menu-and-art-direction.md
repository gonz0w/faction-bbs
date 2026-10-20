# 0025: Oblivion/2 menus and a specific artscene reference base

Date/retrieval: 2026-10-19. Status: requirements/reference research completed;
menu/parser/theme implementation untested. Issues: #13/#21/#25/#29/#31/#35.
Authority: owner specifies arrow-key highlights plus direct hotkeys, sysop themes,
possible seasonal colors/login screens, and ACiD/iCE/Razor 1911 style references.
Outcome: [current interaction/theme direction](../../docs/menus-and-themes.md).

## What was actually examined

- [Oblivion/2 XRM author documentation](https://oblivion2-xrm.readthedocs.io/en/latest/)
  and [repository](https://github.com/M-griffin/Oblivion2-XRM), revision
  `75c5fb6c8d8a49b5f4f2f7eeb477dff12a8636a6`: menu configuration/implementation
  source, not an executed original DOS Oblivion/2 installation. The remake's
  `build/MENU/main.yaml` contains separate menu option keys, command fields,
  access strings and presentation/pulldown fields; `src/menu_base.cpp` includes
  hotkey and lightbar/pulldown handling. This is a useful separation to study,
  not source we imported or proof of every historical key's behavior.
- Two archive-rendered ANSI images below were visually inspected. Allanon's
  published ASCII collection was read at its Razor logo section. The observations
  are our visual interpretation of these particular works, not group-wide rules.
- The XRM docs' externally hosted menu screenshots could not be retrieved. Do not
  claim visual verification of those screenshots or an original Oblivion/2 run.
  Original menu sets remain a follow-up artifact to examine.

## Attributed reference catalog

| Reference | Evidence / observation | Use for Faction study |
| --- | --- | --- |
| [Neo Tokyo — Somms / ACiD, acdu1094, SO-NEO1.ANS](https://16colo.rs/pack/acdu1094/SO-NEO1.ANS) | Native-cell archive rendering examined; SAUCE records Somms/ACiD, 1994-10-01, width 80 / height 206. A large grayscale portrait uses textured value ramps and negative space above custom gothic lettering. | Identity-screen composition, depth from restrained tones, distinct illustration and title hierarchy; long-form art need not be squeezed into the menu viewport. |
| [The Citadel — Tempus Thales / iCE, ice-9404, TT-CIT6.ICE](https://16colo.rs/pack/ice-9404/TT-CIT6.ICE) | Archive rendering examined; contrasting vivid planes, detailed patterned shading and curved cyan/violet title lettering, with a distinct artist-credit footer. | Saturated but deliberate palettes, authored curves/letterforms and art/board/credit hierarchy. It is an example from a different board, not Faction's Citadel implementation or a bundled asset. |
| [First ASCII-Colly — Allanon, ALN01ASC.TXT, 1995](https://asciiarena.se/release/ALN01ASC.TXT) | The published Razor 1911 logo section was read. It uses sloped/interlocking ASCII letterforms and a compact artist signature. The archive credits Allanon as Independent. | Scene ASCII/NFO typography and compact headers. This is an independent artist's Razor-labeled example, not proof of an official Razor commission or permission to use their name/logo. |
| [Original Oblivion/2 menu-set archive listing](https://www.synchro.net/files/BBS.Archives/Oblivion-2-Groups-Hidden/) | The listing identifies `nm_2dmenus.zip` as a menu set by Necro [iCE]. Search-indexed listing examined; full archive/ZIP art/binary behavior not examined. | Follow-up for practical screen/menu compositions; do not infer working keys, content quality or reuse rights from the listing alone. |

Group entry points: [ACiD official site](https://www.acid.org/),
[iCE official gallery](https://www.ice.org/),
[Razor 1911 official site](https://www.razor1911.com/),
[ACiD archive](https://16colo.rs/group/acid), [iCE archive](https://16colo.rs/group/ice).
The Razor landing page did not expose usable text/art through this lookup; it is
a group reference, not a work we claim to have inspected. Archived images were
used only as temporary analysis references outside the repository and removed.
No archived ANSI/ASCII, images, contact details or historical incidental copy is
reproduced in the project. Archive metadata/artist attribution is retained as links.

## Interaction and presentation contract to validate

The owner's confirmed behavior is movement -> highlight, Enter -> activate,
and `[D]oors` -> immediate D/d activation. Both routes resolve the same action ID
and current authorization. Do not make the user traverse a lightbar to use a
hotkey, and do not launch a door by merely moving focus over its label.

Separate action/permission definitions, input context, layout/navigation, semantic
theme styles and artwork. A seasonal palette or screen override should not remap
commands or alter permissions. Sysop layout customization is first-class; its
validation must reject conflicting keys, missing targets and invalid focus maps.
No script is needed to change a palette or choose a different entry screen.

Keep a capability-aware native text-cell pipeline: original bytes/encoding,
safe ANSI parse, intended dimensions/font/aspect and rendering. SAUCE/iCE-color
semantics are research inputs, not already implemented support. Don't accept a
raster mockup as proof that the actual TTY/SSH stream is correct. No generic logo
generator or all-purpose frame is selected as the product's art style. Primary
format/input references: [SAUCE revision 5](https://www.acid.org/info/sauce/sauce.htm)
and [XTerm control sequences, patch 411](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html),
retrieved 2026-10-19. Their documentation is not a Faction implementation result.

The best first experiment is a **small deterministic menu/theme fixture**, not
another broad historical survey or a complete art editor. Predeclare these cases:

1. Same action once via arrows/Enter or case-insensitive hotkey; movement does not
   execute; disabled/unauthorized entries cannot activate through either path.
2. Split/coalesced CSI and SS3 arrow input, repeated keys, bare Escape and malformed/
   oversized sequences; no trailing sequence bytes become D/other menu actions.
   CRLF must not cause a second activation; editor/chat letters remain text.
3. Focus stays recognizable without color alone; layout resize and ASCII fallback
   preserve valid actions. Measure incremental old/new focus redraw bytes against
   #8's proposed interaction budget; no full large-art repaint per arrow.
4. Base pack, two seasonal overrides, sysop timezone, manual override, overlaps and
   year boundaries; missing/broken art falls back, invalid pack rolls back atomically.
   Switch only at safe boundaries; active drafts/doors stay intact.
5. Private approval and room/file/door ACLs remain effective with malicious labels,
   assets, menu targets and control sequences; no clipboard/title/device-control
   passthrough just because an asset is sysop-selected.
6. Visually review an original/authorized identity screen and working menu at the
   intended terminal geometry/encoding. Record source/artist/rights/changes before
   adding a distribution asset. Keep art quality review separate from parser tests.

These are proposed software oracles, **not executed tests**. Current checks validate
documentation/links/archive integrity only. No new source code/dependency/artwork
was added, and no old BBS was contacted or run. No physical hardware is required.

## Confidence, alternatives and follow-up

High confidence in owner requirements and the two images actually observed;
moderate confidence in the proposed separation and reference interpretations;
low confidence in original menu exactness, terminal correctness and unbuilt theme
scheduling. Remake sources and artist examples are bounded evidence, not a claim
that all group art or original menus worked alike. No numerical winner is assigned.

Simple declarative menus/theme packs are the core baseline. Later Lua can extend
logic behind the same host actions. Hardcoding a single frame/palette would fail
the sysop requirement; making theming depend on arbitrary scripts increases risk
without helping basic color/art selection. Reversal is cheap while the schema is
proposed; keep versioned theme packs/action IDs and permission checks explicit.

#25 owns menu/theme/seasonal design and the proposed fixture; #13 owns input,
encoding/capabilities and fallback; #29 owns hostile terminal cases; #31 owns
art/dependency provenance; #21/#35 incorporate navigation and theme surfaces into
the first local software flow. No duplicate issues or extra first-release feature
commitments are created by this note. Theme scheduling/editor tooling can follow
after core theming; ANSI/TTY art quality remains a primary acceptance concern.
