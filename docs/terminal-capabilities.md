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
- **G** optionally probes and shows the [trusted Kitty image preview](kitty-graphics.md).
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

The native client's default filter passes text/RGB and filters terminal queries,
sync and graphics. Optional local `--kitty` enables only the separately bounded
[graphics gate](kitty-graphics.md); sync remains filtered. Ordinary SSH may carry
emulator replies; proxies/multiplexers can suppress them. Actual GNU screen
4.00.03 fixtures returned no sync or Kitty reply. Real Ghostty/Kitty GUI/font
and tmux certification stays #118.

## Scope and budgets

Mouse, extended keyboard, Sixel, terminal-side image files/shared memory, network
fetching and clipboard remain disabled. Block/braille are text cells. The optional
[Kitty preview](kitty-graphics.md) has separate explicit gates and bounded image
budgets; general theme/member/remote graphics remain filtered. There is no new
JavaScript, GPU dependency or owner helper. The settings panel is below 2 KiB;
sync adds 16 bytes per bounded frame. Existing output/helper/credential/default
limits remain unchanged.

See [research and comparison](../dev-docs/research/0049-modern-terminal-capabilities.md),
[ADR 0041](../dev-docs/decisions/0041-modern-terminals.md) and
[software evidence](../dev-docs/evidence/modern-terminals-netbsd/README.md).
The optional graphics renderer/client gate is documented separately. Hardening #106–#111
stays later; this work changes presentation, not host/firewall policy.

Follow-ups: [bounded inline bitmap renderer #117](https://github.com/gonz0w/faction-bbs/issues/117)
and [real emulator/font/multiplexer validation #118](https://github.com/gonz0w/faction-bbs/issues/118).
