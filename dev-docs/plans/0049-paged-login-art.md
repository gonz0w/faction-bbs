# Long-form ANSI login art and paging — #25

Owner, 2026-10-22: merge checked #124, then long art/login paging; classic
fallback stays, hardening later. #124 merged as d659311.

- [x] Verify exact-head check/review threads; merge #124.
- [x] Freeze executed custom-menu source/result bytes before edits.
- [x] Add bounded long-art parsing and lazy 16-row canonical page rendering.
- [x] Integrate after-approval Enter-next / S-skip login flow in local/worker/shared;
  recheck approval, preserve classic/compact recovery and bounded output/typeahead.
- [x] Add original long-art template and deterministic page previews.
- [x] Validate host/native NetBSD PTYs, actual restricted SSH/native flow,
  paging/skip/resize/revocation, unchanged prior themes and normal limits.
- [x] Update docs, README, evidence, CI and #25; open/attach focused PR.

Direction: opt-in long assets, maximum 80x256 cells and 64 KiB total input;
fixed 16-row pages, <=16 KiB canonical art per page. Shared catalog owns assets,
member state is only a page index. Existing short Art::parse contract remains
80x16 / 16 KiB; a separate bounded import path serves explicit long identity art.
Plain/compact sessions use the existing fitting ASCII card or board name.
No animation, archival art redistribution, new dependencies, capacity or final
terminal/font/artwork claim. Native software proof suffices; playground stays off.

## Discoveries

Keep short parse at 16 rows and long explicit: do not silently widen old asset
acceptance. Preserve a minimum short canvas for J=2 background semantics, trim
long unused extent after startup and validate every page's expansion. Per-page
rendering must restart styles from cells and translate absolute art rows to the
local 1–16 viewport. Tests cover the 206-row reference extent, maximum page, bad
later controls/amplification, overrides and untouched compact/plain behavior.

Shared identity must set its own state; the old authentication path forced the
single welcome state after drawing. Page enum state plus current approval checks
now handles Enter/skip; existing backpressure already reads only after drain and
flushes coalesced input. Resize recovery waits for empty output before clearing.
Local/worker use the same presenter and recheck through a callback at boundaries.
The preview allows 17 KiB for 16 KiB canonical art plus framing/prompt overhead.

## Validation

Final code passes 171 Rust tests per OS, 112 host Python, strict host Clippy/fmt,
unchanged product/four-experiment no-JS inventories and active shared-UI Clippy/
credential-permit test. Eight local/worker paging controls per OS and eight actual
restricted SSH/native paging controls pass: page order, skip, compact/resize,
preloaded coherence, current revocation and bad later-page preflight. Four local
custom-menu/nine seasonal controls per OS, eight actual shared calendar controls
and 22 shared reader/Unicode/graphics/revocation regressions pass. All eight older
previews remain identical; three long-art page previews reproduce and their
known-cell renders were inspected. No emulator/font/final art acceptance claim.

The new paging record compares 135 selected native inputs and pins nine harnesses
and three new SVGs. Eighteen frozen snapshots/thirty historical record identities
remain unchanged. No new RSS/CPU/250-user/hardware or host-hardening measurement.

## Handoff

[PR #125](https://github.com/gonz0w/faction-bbs/pull/125) is open and attached against
feat/103-articles-and-room-reader; #25 progress is logged. Final native sources
match, all thirty saved-evidence guards pass and a clean Git archive passes
repository/frozen-source/seasonal/custom-menu/paging checks. The owned clone and
private cache are removed; saved playground stays off. Current-head GitHub checks
remain the merge gate. #25 stays open for the stated art/format acceptance work;
hardening remains later. No authorization to merge this new PR is assumed.
