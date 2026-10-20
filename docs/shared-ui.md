# Shared UI candidate

Updated 2026-10-20. This is an opt-in experiment; normal services remain the
eight-worker development gateway. There is no supported network release.

The [shared UI candidate](../experiments/055-shared-ui/README.md) now brings the
local room/chat/sysop behavior and [sysop theme packs](theme-packs.md) into one
bounded Rust owner. Approved members see the configured identity and welcome,
then arrows/Enter or displayed hotkeys. Plain mode, safe CP437 import, manual
seasons, custom keys/order, selected-room pages/replies, live draft-preserving
chat and current core permissions remain available.

Packs load once for the owner; an explicit invalid pack fails startup. A restart
selects a different manual season. This candidate has no automatic calendar or
hot reload. Doors remain disabled until isolated extensions exist. Test identity
assets demonstrate behavior and have original provenance; commissioned scene art
is still separate work.

See [native evidence](../dev-docs/research/0032-shared-ui-themes.md) for exactly
what was measured with 250 connected members on the NetBSD software VM, including
the experimental allocator/time-daemon policy and remaining deployment limits.
Do not change system-wide allocator/NTP settings from these observations alone.
