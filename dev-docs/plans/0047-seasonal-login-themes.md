# Seasonal theme scheduling and login screens (#25)

Owner 2026-10-22: merge checked #122 and continue #25; classic fallback remains,
hardening #106–#111 stays later. Base: existing article feature stack.

## Outcome and design

Preload a base and at most eight validated annual seasonal overrides at startup.
A schedule has explicit UTC/system-local timezone, start-inclusive/end-exclusive
MM-DD windows (including year wrapping), overlap priorities and no tied overlap.
Manual season overrides the calendar. Disabled/unknown clock selects the base;
invalid explicit shared-service configuration fails before binding. Local console
warns and preserves its valid base. No polling of pack files or mid-session switch:
pick after approved login and pin an index through identity/menu/editor/chat/doors.
Shared sessions borrow immutable global themes, never copy art per member.

UTC/system-local date conversion uses the existing narrow native bridge and OS
zone rules; don't spawn date helpers or mutate process TZ/host settings. Pure
calendar matching stays in the terminal crate. Sysop-installed original seasonal
login templates exercise ANSI and compact ASCII fallback through one common
bounded identity presenter. These are original functional templates, not final
accepted scene identity art or redistribution of historical work.

## Steps

- [x] Merge #122 after required check and review-thread verification.
- [x] Preserve the 122 distinct previous application/harness/preview inputs and
  original menu record before editing; keep all older identities fixed.
- [x] Implement bounded schedule/date model, native clock bridge and common login presenter.
- [x] Integrate local/worker/shared sessions and dry-run preview/config workflows.
- [x] Ship disabled calendar plus original autumn/winter login examples and docs.
- [x] Validate date/priority/leap/disable/manual/error behavior and real host/native
  login/session coherence/approval/classic fallback using software fixtures.
- [x] Record exact results and update README/AGENTS/changelog.
- [x] Publish [PR #123](https://github.com/gonz0w/faction-bbs/pull/123) and record its link/progress on #25.

## Constraints and validation

Retain eight/5000-ms normal admission, two credential permits/kernel slots,
fixed owner helpers and 32-KiB/five-second output. No new crypto, schema, external
package, member authority or untrusted terminal-control allowance. New clock FFI
has explicit scalar/output-pointer lifetime and no retained pointers or TZ changes.
Exercise deterministic dates as pure/test/preview inputs; never set guest clock
or add member-supplied dates. No physical SBC gate or new 250-user claim. Use only
owned disposable VM/PTYs; saved playground stays off. Keep actual emulator/font
and final art acceptance distinct from cell/software validation.

## Actual outcomes

162 Rust tests pass on both host macOS ARM64 (1.98.0) and native NetBSD 11 ARM64
(1.96.0), with 110 host Python tests. Nine calendar/local/worker controls pass per
OS, including actual libc UTC/year/DST cases on new authoring processes. Eight
actual restricted SSH/native shared-calendar controls and twenty-two shared
reader/Unicode/graphics/revocation controls pass on the final candidate. Native
standard terminal regressions, strict host Clippy/rustfmt, root/four-experiment
no-JS inventories and active shared-UI Clippy/test pass. External package inventory
stays fixed; five lock files add only the existing internal Unix dependency edge.

Original login previews measure ANSI 604/657 bytes at 80 columns and 183/187 at
40; plain previews 144/148 bytes. Selected 125 guest source inputs, five host
harness sources, executed native fixture and three generated SVGs are pinned in
[evidence](../evidence/seasonal-themes-netbsd/README.md). These are bounded software
observations, not new resident-memory/CPU/250-user/emulator/pixel certification.
The owned clone/cache were removed; saved playground stays off and unchanged.

## Discoveries and continuation

- OS docs require explicit local timezone initialization; the narrow bridge warms
  it once before shared credential helpers without changing TZ or a clock.
- The first shared fixture expected the CLI registration prompt. The real shared
  UI uses Password (hidden); the corrected complete matrix passes. No product
  registration change or credential transcript was needed.
- A compact local ANSI identity could flash away before the menu cleared it.
  Add the Enter pause only for fitting identity cards, then rerun affected host/
  native local and final shared checks. Plain mode remains a line-mode fallback.
- Calendar files and all enabled art preload before binding. Editing a live file
  affects neither current nor later logins until restart. Next-date selection
  uses immutable themes, not hot reload; no mid-draft/calendar timer was added.
- The shipped disabled calendar plus manual override remain the recovery path;
  system mode trusts OS zone rules, including libc UTC fallback for unavailable
  rules. Named per-board zones, grids, long art and final scene-art acceptance
  remain separate #25/#13 work. Hardening stays later.
