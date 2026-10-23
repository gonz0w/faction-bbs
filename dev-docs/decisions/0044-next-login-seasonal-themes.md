# 0044: Preloaded next-login seasonal themes

Date: 2026-10-22. Status: accepted bounded #25 implementation direction.

## Question and choices

The owner asks for seasonal scheduling and login screens without losing classic
fallback. Reloading files on each keystroke/login adds filesystem work and partial
activation risk; switching at midnight risks drafts/doors. A generic per-board
IANA timezone engine adds another package/parser. External date helpers violate
the existing owner process envelope. Retaining only manual palettes misses the
requested useful calendar behavior.

## Decision

Preload base plus <=8 validated annual overrides before shared binding. Require
explicit UTC or the trusted process/system timezone; use libc through the existing
narrow native bridge and initialize local rules before credential helpers. Do not
change TZ/clock/host configuration or add a date process. Match actual Gregorian
dates against start-inclusive/end-exclusive MM-DD windows, including year wrap;
highest priority wins, tied overlap rejects. Leap-day windows never invent dates.

Choose only after current approval and pin one borrowed catalog index throughout
each shared session. Local/worker sessions hold the chosen theme borrow. Manual
season bypasses calendar; disabled/no-match/unknown clock uses base. Bad explicit
shared/worker configuration fails; local console warns and retains the valid base.
A disabled file still validates metadata but skips asset reads. Files change only
on restart. Use the common existing art importer and new bounded identity prefix:
full ANSI 80x24, fitting compact/plain ASCII, otherwise board name. Preserve all
input/authority/queue/credential/admission gates.

## Evidence and reversal

Primary references retrieved 2026-10-22: [NetBSD localtime(3)](https://man.netbsd.org/localtime.3)
for gmtime_r/localtime_r and caller-owned results; [tzset(3)](https://man.netbsd.org/tzset.3)
for initialization, OS/TZ rules and fallback. The _r function alone need not initialize
local conversion rules, so the startup call is explicit. These docs establish API
contracts, not Faction measurements. OS/libc/zone data remain trusted; arbitrary
per-board zone names, hot reload and live switching are deferred. Confidence in
conversion/matching depends on actual host/NetBSD UTC/year/DST fixtures and PTYs;
see plan 0047 and the evidence record for executed outcomes.

Costs: at most nine immutable theme snapshots globally, a bounded index per shared
client, one small clock FFI and no new external package/helper/schema. This is not
a new resident-memory/250-user support claim. Roll back with matching old config
and CLI flags/manual packs; data needs no migration. No historical art rights are
implied by the original functional templates. Native glyph/pixel/art acceptance,
custom grids, long art and later hardening stay separate.

Executed validation: 162 Rust tests on host/native, nine seasonal workflows per
OS, eight shared restricted SSH/native controls (including system-local startup),
and 22 shared regressions pass. See [the evidence](../evidence/seasonal-themes-netbsd/README.md).
The compact card pause was checked on the final candidate. The saved playground
stays off; no actual emulator/font or fresh memory/capacity result follows.
