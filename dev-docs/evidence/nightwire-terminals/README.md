# Nightwire in real terminal emulators

Issue [#25](https://github.com/gonz0w/faction-bbs/issues/25), 2026-10-22.
[Results](results.json) are sanitized observations of actual owned macOS terminal
TTYs, not a mocked terminal responder or screenshots. No desktop capture or
accessibility/automation permission was requested.

## Scope and interpretation

The matrix uses private, signed fixture copies of Ghostty 1.3.1, Kitty 0.49.2 and
Alacritty 0.17.0. Modified bundle identities isolate restoration/configuration;
ARM64 machine-code sections match the selected reference executables. The
preparation tool verifies downloaded artifact sizes/hashes and excludes Kitty's
HTML/JS documentation from selected reference files. These are optional validation
tools, not new product runtime dependencies or stock-package certification.

Each emulator runs direct with requested Menlo, a deliberately nonexistent font,
private tmux 3.8 and installed GNU screen 4.00.03, then a real 40x18 direct window.
Private tmux uses status off for an actual 80x24 content viewport. Requested font
and actual TTY geometry are recorded independently; no actual fallback face is
inferred. No personal application/multiplexer settings are changed.

Eleven usable wide profiles run twenty frames each; four smaller viewports run
five compact controls each: **240 frames, 2,476 cursor checkpoints and 187
repeated-glyph observations pass across 15 profiles**. Alacritty’s missing-font
profile returned 80x20 rather than the requested 80x24; its five compact authoring
controls do not certify automatic BBS layout selection at that geometry.

Twenty wide-window frames cover ten long/short/seasonal ANSI pages, four compact
cards, four plain identities and two menu layouts. Five frames in each real
compact window cover four compact cards plus the ordered menu. Every text segment
is sent unchanged except explicit OPOST/ONLCR emulation in the raw response TTY.
Real cursor reports are compared at product cursor-move boundaries. Every CP437
glyph present in the frames is repeated 32 times to expose cumulative width drift.
The record includes frame hashes, expected/observed positions and glyph results.

This is host authoring-output validation. The earlier [native NetBSD proof](../nightwire-netbsd/README.md)
provides separate local/worker and actual SSH/native login results. Its full record
and all 167 distinct source/art/harness/preview inputs remain unchanged. No new
native VM run, 250-user capacity, physical device, SSH security or hardening claim.
No product runtime, artwork asset, installed default or dependency changed.

## Findings and limits

The initial two direct Ghostty runs revealed a **probe** mismatch: a raw query TTY
had disabled normal newline output processing, producing staircased compact cards.
The probe now explicitly models the product adapters' preserved OPOST/ONLCR path.
The first default-status tmux fixture also left only 23 rows and exited before an
art result. Final runs use an explicit private status-off layout. Failed fixture
observations are retained in the record; no failed product rendering was hidden
by rehashing earlier observations or altering art to suit the probe.

Cursor reports certify neither pixels nor the selected font/palette. Canonical
cell/style checks and reference SVGs are separate. User palette contrast, glyph
shape/half-block seams, broader terminal/platform coverage and final owner visual
acceptance remain open. See [operator guidance](../../../docs/artwork-terminal-compatibility.md).

## Reproduce

1. Build the existing `preview_theme` release example with the locked workspace.
2. In a **new private directory**, prepare the three fixture applications using
   `tools/prepare_terminal_reference.py`. Ghostty needs an explicitly selected
   local app; Kitty/Alacritty use the tool's pinned release artifacts.
3. Optionally build private tmux with `tools/build_terminal_multiplexer.py`; its
   C/libevent/ncurses build is outside product runtime. Compile
   `tools/native_terminal_window.m` with Cocoa into the same private directory.
4. Run `tools/run_artwork_terminals.py --help`, supplying absolute private app,
   controller, binary, output, directory and optional tmux paths. It serializes
   cases, restores focus only from owned instances and stops each owned instance.
   Result files/logs stay private for diagnosis. It never installs applications.
5. Verify frame oracles with `python3 -m unittest discover -s tools -p test_artwork_terminal.py`
   and the saved record with `python3 tools/check_artwork_terminal_evidence.py`.
6. Stop only processes whose executable/command paths belong to this newly owned
   fixture. Remove its generated fixture bundle preferences/caches/restoration
   namespaces and private files after collecting a sanitized record. Never run an
   old result collector/cleanup script that would rewrite historical records.

Historical observations must remain immutable. Preserve all pinned measured
bytes and the complete original result before future edits. The CI guard verifies
source identities and measured positions without launching GUI software.
