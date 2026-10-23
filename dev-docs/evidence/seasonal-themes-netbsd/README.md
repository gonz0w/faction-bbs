# Seasonal calendar and login software validation

Executed 2026-10-22 for #25 / [plan 0047](../../plans/0047-seasonal-login-themes.md).
[Results](results.json) pin selected build/run inputs, host harnesses, original
preview assets, installed native binaries and qualified observations.
`tools/check_seasonal_themes_evidence.py` checks this record without booting a VM.

162 Rust tests pass on macOS 27 ARM64 / Rust 1.98.0 and native NetBSD 11 ARM64 /
Rust 1.96.0 in an owned 2-core/1-GiB VM. 110 host Python tests, strict host Clippy/
rustfmt and reviewed root/four-experiment no-JS inventories pass; registry package
versions/features stay unchanged. The active shared-UI experiment's strict Clippy
and credential-permit regression also pass. The new terminal -> Unix dependency
is an internal edge to the existing narrow bridge, not a new registry package.

Nine seasonal controls pass on each OS: approval before identity; wide ANSI,
compact ASCII and plain login; a coherent subject/body draft after source edits;
manual bypass, disabled assets and local-error fallback; inherited worker; and
actual UTC/system-local year/DST conversions. The fixture sets a POSIX TZ only
on new authoring-example processes. Product code never changes TZ or a clock.
Gregorian date/priority/wrap/leap/limit/permission cases are deterministic Rust
controls. The compact ANSI card now waits for Enter before the menu clears it.

Eight separate shared controls use actual restricted member SSH and the host
native client: original calendar-selected ANSI, compact ASCII, immutable preloaded
art through drafts/later logins, pending denial before identity, enabled-art
preflight failure, disabled recovery, manual recovery and system-local startup
under the normal owner envelope. Twenty-two shared frontend reader/Unicode/
graphics/revocation regressions pass on the final candidate. Native local console,
room, five Unicode, five modern, seven Kitty and three native-gate controls also
pass. Admission stays eight / 5000 ms; fixtures pace connections.

The 125 selected build/run inputs match the final guest source tree. Five host
harness sources and the actually executed native calendar driver are pinned
separately. This selection does not claim every file executed on both OSes or
pin all OS/compiler/stdlib/private orchestration code. Prior #122 sources/results
remain unchanged in sixteenth snapshot `sources/pre25-calendar`.

Original full login previews measure 604/657 ANSI bytes for autumn/winter, and
183/187 at 40 columns; plain authoring previews measure 144/148 bytes. These are
specific presenter/welcome examples, not maximum art or network latency budgets.
The catalog holds <=9 themes globally in shared mode and clients retain one index.
Worker mode has its own catalog per process. There is no new owner helper, external
package, schema or changed queue/drain/credential limit. No new RSS, CPU, physical
SBC, 250-member capacity or sandbox certification is claimed.

The original Codex-assisted seasonal templates retain BSD provenance. Their SVGs
regenerate from the real presenter; a locally rasterized cell view was inspected.
They are not final accepted scene identity art or real emulator/font screenshots.
There is no new graphics-control allowance, public listener, clock setting, host
policy, live reload or member-controlled calendar date. Existing profiles from
#118 were not rerun or relabeled as measurements of this candidate.

Reproduce using the [pack/calendar guide](../../../docs/theme-packs.md), root CI,
`smoke_theme_schedule.py` and `smoke_shared_seasons.py`. The shared driver mutates
only a marked owned synthetic guest with existing private fixture profiles; never
use it against a real board. Calendar examples ship disabled; first-install
currently copies the base only. Complete backups must retain external calendar,
pack/art and service configuration. Restore matching old config/flags for rollback;
there is no DB migration. The test clone/cache were removed and the preserved
playground remains off. Hardening #106–#111 stays later.
