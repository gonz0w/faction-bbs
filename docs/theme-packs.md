# Local ANSI menus and sysop theme packs

Updated 2026-10-22. Implemented in the local console and trusted SSH member worker; a supported
network release remains work. This is a vertical menu/identity baseline, not a complete art editor or
historical terminal emulator. See [build/setup](local-development.md).

```sh
./target/release/faction --state-dir "$HOME/.faction-dev" login --ansi --theme themes/base
./target/release/faction --state-dir "$HOME/.faction-dev" login --ansi --theme themes/base --season themes/seasons/winter
```

Plain text remains the default; use `--plain` explicitly if desired. With `--ansi`,
an approved member sees the configured identity, then the welcome and an Enter
pause, then the menu. Up/Down or Left/Right move the visible `>` focus marker;
Enter opens the selected enabled action. Displayed hotkeys activate directly,
case-insensitively, without Enter. Tab moves forward. Moving focus does not execute.
After a room/sysop action, Enter returns to the menu. Message/editor input is a
separate cooked input context, never dispatched as menu keys.

Sysop controls are disabled for members. Doors are an explicitly unavailable
placeholder in the local console; the shared backend can enable optional Lua
Doors and Files at runtime. A `files` menu entry may choose its own key/label/order.
When installed without an explicit files entry, the host adds `[F]ile areas` before
Quit; a conflicting F key requires a custom files entry. Neither theme action
enables its underlying service. Theme labels/keys/order
do not grant privileges. Every real community action still checks the current
database session/approval/role; revocation is enforced on the next checked action.
This console still has filesystem-owner recovery authority: never expose it as an
SSH forced command, remote shell or member OS account.

## Terminal baseline and recovery

ANSI mode is an explicit assertion that the local terminal understands these
controls. stdin/stdout must be TTYs, `TERM` must be nonempty and not `dumb`, and
geometry must be 40–240 columns by 18–100 rows. Identity ANSI art additionally needs
at least 80x24. Otherwise the console uses the same theme's plain hotkey menu with
Enter. A resize during a menu restores cooked mode and falls back for that action;
the next menu may use ANSI again if geometry is suitable. The main menu itself sends no device queries. Optional Terminal settings have
separate bounded explicit probes; see [terminal capabilities](terminal-capabilities.md). UTF-8 output and a monospaced font with single-cell
CP437-mapped glyphs are assumed for art; ambiguous-width/font/aspect settings can
affect composition. No all-terminal compatibility claim.

Supported arrow input is CSI/SS3 A/B/C/D, optionally `1`; modified/function/mouse
keys are not actions. Sequences have a 16-byte parameter limit and 300-ms completion
deadline. Bare Escape, unknown/malformed/oversized sequences quarantine following
bytes until a line boundary; press Enter to recover. OSC strings drain to BEL/ST,
DCS/APC/PM to ST, even across newlines; Ctrl-C/D exits a stuck menu. CRLF activates
once. Non-ASCII UTF-8 keyboard characters are ignored as whole byte sequences;
their continuation bytes do not become C1 controls. Typeahead is flushed at raw/
cooked transitions, so bytes queued with a selection do not become editor commands.

Normal return/error restores termios, SGR and cursor visibility. Fatal process
termination can still require local terminal recovery (`stty sane`). The local
adapter polls its actual stdin TTY with a 100-ms sleeping timeout to notice resize/
decoder deadlines; it is not the proposed many-member server event loop.

## Format version 1

Copy [the base example](../themes/base/theme.toml) to a directory you own. Packs
are loaded once per login invocation, fully validated in memory, and stay coherent
for that session. Invalid base packs fall back to the built-in text identity/menu;
invalid seasonal overrides retain the entire valid base. No partial activation,
runtime scripts, file writes, shell commands, outbound calls or hot reload.

`theme.toml` is UTF-8 TOML, at most 8 KiB; this narrow schema also limits opening
brackets to 64 before parsing. Unknown fields/types and unsupported versions fail.
The pack directory and files must belong to the effective OS user, be regular
directory/files, not symlinks, and not group/world writable. Parent directories and
same-UID operator processes are trusted; this is not a hostile-filesystem sandbox.

| Field | Contract |
| --- | --- |
| `version`, `id` | Required version 1; ID is 1–32 lowercase ASCII letters/digits/hyphens. |
| `board`, `welcome`, `logout` | Optional printable ASCII, maximum 36 / 36 / 60 bytes. Built-in defaults fill omissions. |
| `menu_title` | Optional printable ASCII section title, 1–24 bytes; defaults to `Main menu`. Base-only, never a seasonal command-map change. |
| `normal`, `focus`, `hotkey`, `disabled` | Optional inline/style tables: required `fg`/`bg` integers 0–15, optional `bold` boolean. Foreground and background must differ. Numbers use ANSI ordering, including bright 8–15. No raw escape strings. |
| `[[menu]]` | Optional 6–10 entries, in display order. Each has `action`, one ASCII alphabetic `key` (unique ignoring case), printable ASCII `label` up to 26 bytes, optional printable ASCII `description` up to 30 bytes. |
| Core actions | Exactly one each of `rooms`, `new`, `history`, `post`, `sysop`, `quit`; optional one `chat` entry enables local chat; optional `doors`, `files` and `terminal` entries expose host capabilities/preferences. Doors/Files availability is decided by the host, never the theme. Unknown/duplicate actions fail. Permissions remain compiled/core-checked. |
| `identity_ascii`, `identity_ansi` | Optional simple local filenames, at most 64 bytes, starting with an alphanumeric and containing only letters/digits/dot/underscore/hyphen. No directories or traversal. |
| `creator`, `source`, `permission` | Required printable provenance text (at most 160 bytes each) when an identity file is selected. Record real authorship/rights; text declarations do not prove licensing. |

Older custom version-1 menus can omit chat; add an entry with `action="chat"`,
`key="C"` and a label to expose it. The shipped base includes it.

Omitted art displays the board name as text. No generic block-font logo or archived
artwork is bundled. ASCII identity files allow printable ASCII/CRLF/LF, at most
2 KiB, 80 columns and 16 lines. ANSI files are CP437 bytes, at most 16 KiB, imported
into an 80x16 cell canvas. Glyph bytes 128–255 map to Unicode; 127 uses the DOS
house glyph. Other low control glyphs are unsupported. CR/LF is normalized to
line starts; this is not a byte-perfect ANSI terminal emulator.

The ANSI subset allows SGR reset/bold/normal intensity, basic/bright foreground and
background colors; bounded CSI cursor H/f/A/B/C/D; save/restore s/u; clear J=2 and
erase-to-end K=0. Out-of-canvas movement, other controls, OSC/DCS/device/clipboard/
window operations, blink and extended/RGB colors fail the whole pack. Original
escape bytes never reach the terminal. Rendering uses only canonical local SGR,
bounded cursor positioning and known glyphs, capped at 16 KiB output.

Long scrolling art, SAUCE metadata, iCE blink-versus-bright-background interpretation,
custom screen grids, per-room styling and automatic terminal/font detection remain
under #13/#25/#29. Do not call the importer compatible with every archived art pack.
The rights/provenance and [art direction](menus-and-themes.md) requirements still
apply to every actual identity screen.

## Seasonal overrides and measured redraw

`--season` selects an override explicitly at next login. [Autumn](../themes/seasons/autumn/theme.toml)
and [winter](../themes/seasons/winter/theme.toml) are examples, not a default calendar
or hemisphere assumption. Overrides may set version/ID, welcome, styles, identity
files and their provenance. They cannot change board name, menu labels, keys,
actions or order. Missing fields retain the base; any invalid field/file rejects
the entire override. No scheduling/timezone/overlap or mid-session switching yet.

The menu uses the actual TTY width: 40–79 columns selects a centered 38-cell
compact panel; 80–240 selects a centered 76-cell panel with action descriptions.
Both use the same vertical action order, Up/Down/Left/Right/Tab navigation,
Enter/direct hotkeys and current authorization. A 10-entry menu fits in 18 rows.
No keyboard action or service is added by a description. Unavailable actions show
`--` in compact mode and `(unavailable)` in wide mode. The `>` marker stays visible
without color; focus covers the whole row, including the hotkey background.
The fixed help/recovery instructions cannot be overridden by a pack.

The built-in nine-entry compact menu emits 1,224 UTF-8 bytes; its first focus
move emits 170 bytes. The maximal-text ten-entry fixture measures 1,311–1,974
bytes for a full screen and 170–248 for a two-row focus update across 40, 79, 80,
132 and 240 columns. Focus movement never replays login art or clears the screen.
Regression ceilings are 4 KiB per full menu and 512 bytes per two-row update;
these are renderer output budgets, not latency, resident-memory or capacity claims.
The shared queue/drain and normal eight-member admission defaults stay unchanged.

New fields are additive to version 1: existing packs load with a default title
and no descriptions. Older binaries reject a pack using new fields rather than
partly applying it. Keep a matching older pack when rolling back a binary. Each
startup/login selects one validated snapshot; there is no mid-session reload.

The [shared UI candidate](shared-ui.md) reuses this validator, safe CP437 importer
and menu renderer for all members through one immutable owner snapshot. Select
the base then manual season at owner startup; a restart changes the shared
snapshot. Invalid explicit packs fail startup before binding. Normal local-console
fallback behavior above is unchanged. Native theme/plain/resize/editor checks and
three short 250-member VM workloads pass; automatic scheduling, hot reload and
commissioned scene artwork remain separate work.

The optional `terminal` menu action opens session-only [terminal settings](terminal-capabilities.md).
Add it with a free hotkey (the current base example uses T). Existing custom
layouts are not remapped automatically. Rich color headers use the sysop's
existing 16-color palette; this does not widen trusted art or theme-text formats.

## Preview and customize without a live board

The shipped base uses a violet accent and blue lightbar; autumn and winter are
manual palette/welcome overrides. Menu chrome and descriptions are original
Faction work under BSD-3-Clause. They are not a commissioned identity screen or
redistributed scene artwork. Identity assets still require their own provenance.

![Base menu reference-palette preview](images/menu-base.svg)

These SVGs use a fixed illustrative 16-color reference palette; a terminal's own
palette/font changes the appearance. [Compact](images/menu-compact.svg) and
[autumn](images/menu-autumn.svg) previews come from the same renderer. They are
cell-layout previews, not emulator screenshots or glyph/pixel certification.

Build the optional local authoring example:

```sh
cargo build -p faction-terminal --example preview_theme --release --locked
./target/release/examples/preview_theme --theme themes/base --columns 80
./target/release/examples/preview_theme --theme themes/base --columns 40 --plain
./target/release/examples/preview_theme --theme themes/base --season themes/seasons/autumn
python3 tools/render_theme_preview.py target/release/examples/preview_theme --theme themes/base --output /tmp/faction-menu.svg
```

The preview validates the whole pack and fails on errors. It uses no board,
credentials, account permissions or listener. For authoring it shows sysop,
Doors and Files as available; it does not enable those services on a live board.
The SVG tool needs only Python's standard library and rejects terminal controls
outside the menu's canonical positioning/16-color grammar. Product builds do not
need it. Use an empty terminal at least as wide/high as the chosen preview.

For a custom layout, copy the base pack, change `menu_title`, labels/descriptions,
colors and order, then validate the copy. Retain the required core actions, unique
keys and fixed action names. Restart the shared owner after checking a stopped
configuration; don't edit an active session's action mapping. A season changes
only its allowed palette/welcome/identity fields; base descriptions and title stay
coherent. Calendar scheduling, custom grids, long art/SAUCE/iCE support and native
visual acceptance of original/authorized scene identity art remain #25 follow-ups.
