# 0049: Native text and terminal capabilities

Status: software prototype validated; full emulator matrix/bitmap runtime deferred.
Date: 2026-10-22. Issue #96; decision ADR 0041. Owner: project maintainer.
Stopping condition: sourced comparison, bounded runtime/fixtures and explicit
fallback/budgets, rather than an emulator feature-list support claim.

## Question, scope and gates

How can members use modern terminal text while retaining a small NetBSD BBS and
safe classic recovery? Gates: current approval/local data, no JS, bounded input/
output/work, no new owner helpers, and no arbitrary OSC/DCS/APC or image access.
Accepted: session-only explicit preferences, cell-aware UTF-8 subject/room reading,
color depth, sample text-cell graphs and a single opt-in sync probe. Native-client
control filtering stays unchanged. Native chat editing and richer theme-asset formats
remain separate; this increment renders stored room text and post subjects. No host configuration or later hardening work.

## Independent capabilities and tiers

| Capability | Evidence/input | Current policy |
| --- | --- | --- |
| Geometry | admitted TTY TIOCGWINSZ; 40–240 columns / 18–100 rows for ANSI | classic/plain fallback outside range; reader clamps its existing page bounds |
| Encoding/cells | explicit U override; pinned width tables, ambiguous=1 | default ASCII escapes; width-one/two native scalars, escape invisible/cluster-dependent forms |
| ANSI | existing explicit local option/sysop setting plus geometry | TERM may veto, never enable richer features by brand |
| Color | independent 16/256/RGB member override | retain sysop palette and use trusted indexed/RGB SGR headers |
| Block / braille | two separate member switches and UTF-8 mode | fixed sample graphs, not live collectors or bitmaps |
| Synchronized redraw | independent explicit override or positive DECRQM 2026 result | bracket complete bounded menu/message frames; no periodic refresh |
| Mouse / extended keyboard | independent future parser contracts | disabled; no mode push or mouse tracking |
| Kitty / Sixel bitmap | independent future renderer/transport gates | disabled; active pixel, image-byte and cache budgets are zero |

Tier 0 is plain/ANSI with ASCII escapes. Tier 1 opts into native text, colors and
text-cell samples. Tier 2 is only a proposal: explicitly negotiated trusted bitmap
assets under a separate renderer. Terminal reports/preferences never grant
membership, roles, host access, paths, network actions or publication authority.

## Primary comparison (retrieved 2026-10-22)

[Ghostty features](https://ghostty.org/docs/features) documents Kitty graphics,
keyboard and synchronized rendering. Its [sync guide](https://ghostty.org/docs/help/synchronized-output)
explains complete-frame boundaries. These are emulator facilities, not automatic
Faction support. No Sixel support is inferred. GPU rendering is member-side;
server text emission does not need a GPU. Actual Ghostty GUI/font versions were
not exercised by these byte/PTY fixtures.

[XTerm control sequences, patch 411](https://invisible-island.net/xterm/ctlseqs/ctlseqs.txt)
documents DECRQM/DECRPM and SGR; the active probe sends only CSI ?2026$p and accepts
one exact CSI ?2026;Ps$y reply. Values 1/2 permit controllable sync; unknown or no
reply disables it. Permanently fixed modes are not enabled as controllable.
Ghostty's [mode implementation](https://raw.githubusercontent.com/ghostty-org/ghostty/main/src/terminal/modes.zig)
is an additional living primary reference; it is not a pinned runtime dependency.

The [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/)
has direct, file and shared-memory transmission and support replies. A future
Faction renderer should use trusted inline raw RGB only; no file/shared-memory,
compressed PNG decoder, network fetch, clipboard or arbitrary member artwork.
The [keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/) is
progressive and changes input framing. Keep it disabled until its parser and mode
restoration are independently scoped; a graphics response cannot enable it.

[btop's configuration](https://github.com/aristocratos/btop) distinguishes braille,
block and TTY graph glyphs and their font/resolution tradeoffs. Faction's samples
borrow that text-cell technique, not btop code, dependency or collector support.
Native glyphs are optional; no required font or bitmap/GPU path.

[NetBSD terminfo(5)](https://man.netbsd.org/terminfo.5) provides capability data
selected by terminal name. A local terminfo database can be a hint but does not
prove the active SSH/multiplexer/font route, and expanding arbitrary stored escape
strings is unnecessary here. Explicit overrides plus one bounded positive query
are the smaller first implementation. No terminfo file/network lookup is added.

[tmux's FAQ](https://github.com/tmux/tmux/wiki/FAQ) discusses terminal/feature
configuration; [GNU screen's character processing](https://www.gnu.org/software/screen/manual/html_node/Character-Processing.html)
separates UTF-8 encoding settings. Their version/configuration and the outer
terminal all affect the effective path. Do not unwrap DCS passthrough to guess
outer capabilities. A private actual macOS GNU screen 4.00.03 -U fixture returned
no sync reply, so fallback is the measured outcome. tmux was unavailable: dropped/
unknown replies are modeled, not a claim of actual tmux interoperability.

## Width dependency and alternatives

[unicode-width 0.2.2](https://docs.rs/unicode-width/0.2.2/unicode_width/) is pinned
with default features disabled, no active transitive dependencies, no build script,
no_std and forbidden unsafe Rust in its source. Inspect actual archive, licenses
and no-JS graph in [the review record](../dependencies/unicode-width.json). The
selected graphs add one MIT/Apache-2.0 package, 41 NetBSD / 43 Linux packages.
Disabling CJK alternate widths avoids extra static tables. Scalar width is not a
font or grapheme guarantee: zero-width marks, bidi formatting, skin-tone modifiers
and regional flags use escapes. Precomposed accents, wide CJK and ordinary emoji
render natively. More complex clustering and ambiguous-width policies remain a
separate follow-up; C immediately restores classic mode.

Alternatives rejected for this increment: TERM-brand auto-enablement (no path
proof), locale-dependent libc wcwidth/FFI (host locale/width mismatch), full UI/
graphics runtimes (unnecessary graph/cost), and new grapheme/editor/protocol
adoption before a useful bounded text slice. No invented numerical winner.

## Budgets and falsifiable controls

Existing shared encoded output cap stays 32 KiB with a five-second drain deadline;
normal defaults stay eight sessions, fixed three owner helpers and two credential
permits/kernel slots. Capability state is a few flags; no asset cache/listener or
per-member helper. The fixed settings panel is below 2 KiB. Sync adds 16 bytes per
frame. Output is action-driven with existing input/work scheduling, not a new
refresh loop. Body/subject and page limits remain unchanged. Clip by cells and
UTF-8 boundaries; retain contiguous file references and partial-unread semantics.

A query is nine output bytes, at most 32 response bytes, with a 300-ms deadline
starting after query output drains. It has a separate input state and returns
only a boolean. Extra/malformed/incomplete replies close the connection; timeout
without bytes stays in settings. Late CSI is rejected by settings, not replayed
as menu/posting commands. Probes are explicit, after local approval, and never run
in login/password entry. A lying positive can affect only display preferences.

The native client's existing filter passes native UTF-8 and RGB SGR but strips
queries/sync/APC/DCS/clipboard. Its probe therefore times out and requires explicit
Unicode/color overrides; do not claim end-to-end sync/graphics through it.

Future Tier-2 proposal, **not enabled**: one trusted inline raw RGB asset, at most
160x80 pixels (38,400 raw bytes), <=51,200 base64 bytes plus bounded headers,
<=4-KiB chunks, one image/cache per session, <=1 frame/second and explicit delete/
cleanup. This exceeds the single shared output queue and requires bounded streaming
and full resource validation before adoption. Current image budgets remain zero.

## Validation, confidence and reversal

145 Rust tests on host/native NetBSD plus real local/worker/shared PTYs cover
wide text, clipping/wrapping/file references, backspace, unknown/lying/malformed/
incomplete/late probes, color/sync framing, graph samples and classic recovery.
Native-client refusal is tested on its actual Rust filter. Software fixtures prove
these contracts, not every emulator/font/multiplexer, native GUI graphics, hardware,
250-member capacity or full production hardening. Existing classic tests remain.
Reversal is cheap: C or reconnect resets preferences; no DB/schema migration.
Record exact current sources separately and preserve all twelve snapshots and
twenty-four earlier result identities. Future rich graphics and extended input
need separately reviewable renderer/parser, packaging and interoperability work.

Follow-ups: [bounded inline bitmap renderer #117](https://github.com/gonz0w/faction-bbs/issues/117)
and [real emulator/font/multiplexer validation #118](https://github.com/gonz0w/faction-bbs/issues/118).
