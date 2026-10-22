# Long-form ANSI login art and paging — #25

Owner, 2026-10-22: merge checked #124, then long art/login paging; classic
fallback stays, hardening later. #124 merged as d659311.

- [x] Verify exact-head check/review threads; merge #124.
- [x] Freeze executed custom-menu source/result bytes before edits.
- [ ] Add bounded long-art parsing and lazy 16-row canonical page rendering.
- [ ] Integrate after-approval Enter-next / S-skip login flow in local/worker/shared;
  recheck approval, preserve classic/compact recovery and bounded output/typeahead.
- [ ] Add original long-art template and deterministic page previews.
- [ ] Validate host/native NetBSD PTYs, actual restricted SSH/native flow,
  paging/skip/resize/revocation, unchanged prior themes and normal limits.
- [ ] Update docs, README, evidence, CI and #25; open/attach focused PR.

Direction: opt-in long assets, maximum 80x256 cells and 64 KiB total input;
fixed 16-row pages, <=16 KiB canonical art per page. Shared catalog owns assets,
member state is only a page index. Existing short Art::parse contract remains
80x16 / 16 KiB; a separate bounded import path serves explicit long identity art.
Plain/compact sessions use the existing fitting ASCII card or board name.
No animation, archival art redistribution, new dependencies, capacity or final
terminal/font/artwork claim. Native software proof suffices; playground stays off.
