# 0045: Real terminal compatibility

Issue #118. Completed software validation, ready for review, 2026-10-22. Merge #120 after exact-head checks, then
measure actual terminal software/routes and safe classic recovery. Keep hardening later.

- [x] Verify/merge green #120 and freeze 112 actual inputs plus original result.
- [x] Pin representative installed Ghostty, temporary Kitty and multiplexer/config/font profiles.
- [x] Run isolated actual terminal TTY measurements and full local/SSH/native routes on synthetic state.
- [x] Exercise independent capability/fallback, width/cluster/font/resize cases, logging real failures without synthetic certification.
- [x] Record exact source/tool/config identities, bounded costs, outcomes and remaining gates; update docs/backlog and open PR.

Classic defaults, current 300-ms/byte/drain/output/credential/helper/admission
limits and reviewed fixed-asset gate stay. No arbitrary DCS/APC workaround,
public endpoint, JS, host/firewall/hardening or new capacity/hardware claim.
Use only new isolated emulator windows and disposable software fixtures; preserve
existing user terminals and the stopped private saved BBS playground.

## Starting observations

Installed Ghostty 1.3.1 on macOS ARM64 uses CoreText/Metal. A separate background
launch is being verified before claiming any actual emulator result. Its CLI
show-face reports Menlo, PingFang SC and Apple Color Emoji, with block/braille
internal sprites; this is metadata, not a visual or full grapheme certification.
Kitty 0.49.2 official DMG metadata publishes a SHA-256 digest. GNU screen 4.00.03
is installed; tmux is not. Temporary tools must remain outside the product graph.

## Measured progress and fixture corrections

All three direct emulators have now produced real matching cell reports for the
selected scalar and escaped-cluster corpus. Kitty 0.49.2 has also passed all four
real BBS routes: local, restricted NetBSD member SSH, native default filtering
and native --kitty. The renderer acknowledges stored product pixels before C
and ENOENT after C; no fake terminal response is injected. Native sync stays
filtered by design. Full pinned font/tmux/screen matrix is running.

Preliminary fixtures exposed AppKit background startup/restoration behavior;
use fresh fixture-only bundle IDs/signatures and a start gate, restoring focus
only when the foreground is still our new instance. References exclude Kitty's
HTML documentation/JS. Source/runtime code sections stay unchanged; these are
isolated packaging derivatives, not installs or global signing/policy changes.

The first observation parser misclassified tmux's valid seven-byte negative
DECRQSS reply; its exact refusal grammar is now handled only by the research
observer. Product DCS filtering is unchanged. Old screen miscounts some emoji;
classify it as classic-only when the escaped ASCII corpus and C recovery pass.
Relay geometry must be captured before PTY fork; local 80x24 ANSI login includes
a welcome pause. Those were fixture assumptions, not product fixes.

A contaminated preliminary keyboard sample is discarded, never used as evidence;
the observer retains only structurally known reports and counts unknown bytes.
No user keyboard input crosses into BBS actions. A clean accepted run requires
zero unclassified input. Capture is unavailable and never requested.

## Final outcome

Twelve profiles: nine native-cell matches, three classic-only screen profiles.
336 real cell/cluster observations, 48 successful local/NetBSD member-SSH/native
BBS routes, twelve actual image retention/deletion pairs and two live SSH PTY
resize controls. All accepted runs have zero unclassified input. 151 Rust tests
per host, 107 Python tests, strict format/clippy, source/dependency guards pass.
115 application inputs byte-match native checkout; references/host harness are
pinned separately. No product runtime/filter/schema/limit change was needed.
The VM/reference apps/mounts/test namespaces are removed; saved playground off.
Visual pixels/glyphs/GPU FPS and physical keyboard conformance remain unclaimed;
#25 retains broader artwork/visual review rather than widening this matrix.
