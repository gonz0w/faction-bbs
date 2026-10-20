# Local ANSI menus and sysop theme packs

Updated 2026-10-19. Implemented in the local development console; no network BBS
release. This is a vertical menu/identity baseline, not a complete art editor or
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
placeholder; selecting or typing D cannot launch anything. Theme labels/keys/order
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
the next menu may use ANSI again if geometry is suitable. No device query or remote
capability negotiation occurs. UTF-8 output and a monospaced font with single-cell
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
| `normal`, `focus`, `hotkey`, `disabled` | Optional inline/style tables: required `fg`/`bg` integers 0–15, optional `bold` boolean. Foreground and background must differ. Numbers use ANSI ordering, including bright 8–15. No raw escape strings. |
| `[[menu]]` | Optional 6–8 entries, in display order. Each has `action`, one ASCII alphabetic `key` (unique ignoring case), printable ASCII `label` up to 26 bytes. |
| Core actions | Exactly one each of `rooms`, `new`, `history`, `post`, `sysop`, `quit`; optional one `chat` entry enables local chat; optional one `doors` entry is always disabled. Unknown/duplicate actions fail. Permissions remain compiled/core-checked. |
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

The built-in eight-entry menu emits 736 UTF-8 bytes for the full initial render
and 137 bytes for the tested first focus move, updating old/new rows without
replaying identity art. These are deterministic renderer byte counts, not network
latency or server capacity results. Regression caps are 2 KiB full menu / 256 bytes
for that focus update. Sysop labels/colors can change byte counts within schema
bounds; application memory and 100/250-member accounting remain unmeasured.
