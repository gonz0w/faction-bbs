# ADR 0041: Explicit terminal preferences and bounded native text

Status: accepted for the software prototype, 2026-10-22. #96 follows merged #116.
See [research 0049](../research/0049-modern-terminal-capabilities.md) for primary
sources, alternatives, active/future budgets and confidence limits.

## Decision

Keep classic presentation by default. Add a session-only Terminal action in the
built-in/current example menus, with independent UTF-8, 16/256/RGB, block/braille
and synchronized-redraw preferences. No schema change or persistent member
profile. Existing packs opt in via a free hotkey; no remapping collision.

Use unicode-width 0.2.2 without default features, no active transitive packages,
no build script and actual archive/license/no-JS review. Render native width-one/
two scalars for stored room text and subjects, clipping/wrapping by cells and
keeping UTF-8 boundaries. Escape invisible/formatting and cluster-dependent
forms; ambiguous width is one. Retain classic recovery and current file-reference,
body/retry/approval/unread bounds. This is not full grapheme/font negotiation.

Probe only controllable sync mode with one exact DECRQM/DECRPM contract, after
explicit approved-member selection. Bound bytes/time and isolate all input;
no response becomes a menu action or authority. A lying positive can affect only
a display preference. Wrap complete bounded menu/message redraws and reset on
cleanup, retaining the five-second drain timeout. Never hold a frame across work
or input. Member/operator overrides and conservative timeouts handle unknown paths.

Keep mouse/extended keys/bitmap policy disabled independently. Preserve existing
native-client OSC/DCS/APC/query/sync filtering; native text/color works there by
explicit preference, not false probe success. No image file/shared-memory/network/
clipboard path or arbitrary artwork pass-through. No new owner helpers, crypto,
session defaults, host/firewall applies or hardening adoption.

## Alternatives and reversal

TERM/brand auto-enablement lacks active-path proof. Locale wcwidth/FFI introduces
host-locale coupling. A full terminal/graphics runtime and grapheme editor cost
more code/dependencies than this useful text slice needs. Static width tables
cost read-only code/data and cannot guarantee a font; classic mode is immediate
recovery. Rich bitmap streaming remains separately scoped because its proposed
encoded payload exceeds the existing queue budget.

145 host/native NetBSD Rust tests plus real modern/classic local/worker/shared
PTYs validate the bounded implementation. Original GNU screen returns no reply;
actual Ghostty/Kitty/tmux GUI/font matrices remain future work. No physical SBC,
250-user, image-renderer or complete production-support claim. Source snapshots
and all earlier result hashes remain fixed. C or reconnect reverses preferences;
no data rollback is necessary beyond the already documented schema-7 upgrade.
