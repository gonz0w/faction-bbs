# Tested terminal profiles

Software observations, 2026-10-22: macOS 27 ARM64 member devices and a NetBSD 11
ARM64 shared BBS. These are named profiles with known limits, not universal
terminal or visual certification. Classic ANSI remains the default everywhere.

| Profile | Native scalar cells | Sync via P | Kitty preview via G | Recommendation |
| --- | --- | --- | --- | --- |
| Ghostty 1.3.1, direct | Matched corpus | Positive | Stored/deleted correctly | Optional U/P/G |
| Kitty 0.49.2, direct | Matched corpus | Positive | Stored/deleted correctly | Optional U/P/G |
| Alacritty 0.17.0, direct | Matched corpus | Positive | No reply; fallback | Optional U/P; classic image fallback |
| All three through tmux 3.8 | Matched corpus | Positive | No reply; fallback | Optional U/P; no passthrough workaround |
| All three through GNU screen 4.00.03 | Some emoji miscounted | No reply | No reply; fallback | Keep C/classic |

The native client intentionally filters sync queries. Without `--kitty`, it also
filters graphics queries/images. With `--kitty`, Ghostty and Kitty pass the local
probe and the reviewed fixed-image gate on direct routes. Alacritty and the tested
multiplexers decline safely. Unicode/color choices remain independent of images.
All 48 local/SSH/native sessions posted/read a synthetic Unicode article and
returned to C/classic successfully.

A deliberately nonexistent font family was also tested in each direct emulator.
Fallback retained the measured cell behavior; Alacritty's resulting grid height
changed. Use actual TTY geometry, not the requested startup dimensions. Ghostty's
font-face helper reports Menlo/PingFang SC/Apple Color Emoji for the regular
profile, JetBrains Mono instead of the missing family, and internal block/braille
sprites. Font names/widths do not certify glyph shapes or a user's installed fonts.

## Use the fallback when the path changes

Choose **T**, then **C**, whenever alignment or colors are wrong. It clears the
optional image and modern preferences. New sessions start classic. In particular,
old screen's emoji-width mismatch is in the multiplexer; a capable outer emulator
cannot fix that path. The tested screen profile explicitly uses `TERM=xterm-256color`
on entry to the old multiplexer. This is a compatibility hint, not a graphics grant.
No DCS passthrough, automatic brand-based enablement or relaxed filter is needed.

Physical keyboard/mouse protocols remain separate: these runs used programmatic
classic application input; extended keyboard and mouse stay disabled. Combining
marks, ZWJ sequences, flags, modifiers and bidi forms retain Faction's existing
inert scalar/escape policy. Raw cluster widths were observations, not a new editor
or grapheme policy.

Two additional actual member-SSH tests resized the owned SSH PTY to 40x18. Menu
input recovered to plain commands; resize with an active image closed conservatively
and the emulator acknowledged that image's deletion. These test SIGWINCH propagation,
not a mouse-driven OS-window resize. The emulators ignored the separate fixture's
CSI window-resize request; the native client's recorded geometry remains authoritative.

## What was measured

Twelve profiles cover direct, tmux, screen and missing-font variants of all three
emulators. There are 336 cursor/cell/cluster observations, 48 real BBS routes,
twelve actual image retention/deletion pairs, and two live SSH resize controls.
Image presence is checked by a fixture-only placement request; after C the same
request gets ENOENT. Those requests are not a new BBS or native-filter capability.
No terminal response is fabricated in this matrix.

GUI copies use private bundle identities/signatures and isolated configuration;
main ARM64 machine-code sections match the source references. Kitty's HTML
documentation/JS is excluded. No emulator, desktop library or multiplexer becomes
a BBS runtime dependency. Tests restore focus only from their own new window,
ignore/discard unrelated keyboard input, and never control existing user terminals.

Desktop capture is unavailable and was not requested. Pixel appearance, glyph
quality, scaling/scrollback, physical key mappings and GPU refresh FPS are not
certified. The measured cursor replies are below 1 ms here; the 300-ms collection
window is not a frame-rate benchmark. Sampled GUI RSS is about 98–137 MiB per
reference process, excluding GPU/child/kernel costs; this is client-side observation,
not a NetBSD server budget or 250-member claim.

See [research 0051](../dev-docs/research/0051-real-terminal-compatibility.md),
[exact evidence](../dev-docs/evidence/terminal-compatibility/README.md),
[terminal settings](terminal-capabilities.md) and [graphics limits](kitty-graphics.md).
Hardening remains later. No public endpoint, host/firewall change or new mail/federation
protocol follows from terminal compatibility testing.
