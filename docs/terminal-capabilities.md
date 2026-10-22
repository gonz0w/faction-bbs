# Terminal settings and native Unicode

Working software preview, 2026-10-22, #96. Classic ANSI/plain presentation remains
the default. Approved members can choose **[T]erminal settings** for this session
in the built-in menu and current base theme. Custom packs may add the `terminal`
action with a free hotkey; existing custom layouts are not silently rewritten.

## Make the board fit your terminal

- **U** toggles native UTF-8 for post subjects, reply previews and room messages.
  Precomposed accents, CJK and ordinary emoji render with one/two-cell widths.
- **1 / 2 / 3** choose 16, 256 or RGB color. Trusted section headers retain the
  sysop's palette; the settings panel shows a sample. Plain mode sends no SGR.
- **B / A** independently toggle block/braille samples when UTF-8 is selected.
  These are font samples, not live system statistics or bitmap graphics.
- **S** explicitly toggles synchronized redraw; **P** optionally probes that one
  feature. Probing requires an ANSI path and discards no data into a command flow.
- **C** immediately resets modern preferences to classic. **D** leaves settings.
  Reconnecting also resets preferences. No account/database migration is needed.

A terminal's name/TERM is not proof of its features. The admitted TTY supplies
geometry; existing operator/local ANSI choices still apply. A NetBSD VM console
can have different capabilities from a terminal connecting over SSH. Use classic
mode whenever alignment, glyphs or colors are wrong; no font is mandatory.

The width policy uses pinned unicode-width 0.2.2 with ambiguous characters one
cell. Zero-width/formatting characters and cluster-dependent emoji modifiers or
regional flags remain escaped. This is a bounded scalar renderer, not a full
grapheme editor or promise about every font. For example `café 界 🦀` displays
natively; a decomposed accent is retained in storage but displayed as an escape.
Clipping/wrapping count cells and preserve UTF-8 boundaries. Classic mode retains
the ASCII-escaped presentation. Existing file-link and unread rules stay.

Local consoles and inherited workers share the same settings/presenter as the
shared service. Post-body echo in the shared service also follows the selected
cell policy. Native chat editing, Unicode theme labels/art and wider rich-asset
formats remain separate. Sysop ANSI/CP437 theme art still follows its existing
strict import rules; member content cannot supply terminal commands.

## Probes, native client and fallback

P sends only a DECRQM query for mode 2026. One exact response is consumed in a
separate state, capped at 32 bytes and 300 ms after query output drains. A positive
controllable-mode reply enables sync; unknown/no response turns it off. Malformed,
extra or incomplete responses close the session; late CSI cannot become menu
keys or posting input. Stay in settings during the short probe. It never runs in
login/password input and never changes access, roles, host configuration or data.

The native transfer client's existing output filter passes native text/RGB but
filters terminal queries and synchronized-output controls. Select U/color manually
there; sync probing times out safely. No native-client bitmap or sync pass-through
is enabled. Ordinary SSH can carry a supported emulator's own response; proxies
and multiplexers may suppress or change it. A private actual GNU screen 4.00.03
fixture returned no sync reply; tmux and real Ghostty/Kitty GUI/font combinations
are not certified by these software fixtures.

## Scope and budgets

Current mouse, extended keyboard, Kitty/Sixel bitmaps, file/shared-memory image
loading, network image fetching and clipboard paths stay disabled. Block/braille
are text cells. There is no new GPU, JavaScript, telemetry, image cache or owner
helper. Current image pixel/byte/cache budgets are zero. The fixed settings panel
is below 2 KiB; sync adds 16 bytes to a frame. Output is action-driven under the
existing 32-KiB shared output cap and five-second drain deadline. Defaults remain
eight sessions with the same credential and helper limits.

See [research and comparison](../dev-docs/research/0049-modern-terminal-capabilities.md),
[ADR 0041](../dev-docs/decisions/0041-modern-terminals.md) and
[software evidence](../dev-docs/evidence/modern-terminals-netbsd/README.md).
Later graphics need a separately bounded renderer/client gate. Hardening #106–#111
stays later; this work changes presentation, not host/firewall policy.

Follow-ups: [bounded inline bitmap renderer #117](https://github.com/gonz0w/faction-bbs/issues/117)
and [real emulator/font/multiplexer validation #118](https://github.com/gonz0w/faction-bbs/issues/118).
