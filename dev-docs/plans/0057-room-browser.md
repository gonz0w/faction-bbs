# 0057: Named room navigation and selected-room actions

Issues #21/#12/#25. Status: implemented and software validated; ready for review.

After checked #132 (head ebb0f3d, merge a7514b6), replace the ANSI R room dump with
a bounded themed browser and room card. Choose a named room with arrows/hotkeys,
then New/History/Post without entering a room ID. Keep chronological reader,
complete-message unread acknowledgement and manual N/H/P/plain workflows.

## Steps

- [x] Merge checked #132 and preserve 178 prior inputs/room-list fixtures plus its
  whole result in snapshot 24, sources/pre21-room-browser.
- [x] Shared inert room presenter/cursor: four rooms/page, bounded first 100,
  current unread counts, room cards, next-unread navigation and classic geometry.
- [x] Local/worker and shared ANSI routes; current approval and fresh room rows
  before every action. Selected IDs are hints, never authority. No retained room
  lists/history per connection or new schema/dependency/helper/default change.
- [x] Reuse existing reader/composer with selected room, unchanged retry/unread/
  visibility/Unicode rules; completed actions return through existing main pause.
- [x] Update old R-list fixture expectations, add real host/native room journeys,
  unread/cancel/revoke/resize/pending checks and regression tests.
- [x] Docs/README/decision/evidence, owned VM cleanup and focused PR. Hardening later.

## Validation and limits

Use 40x18/80x24, native Unicode/classic/plain routes, page/selection stability,
empty/new room cards, posting/reading selected IDs, unread counts and approval.
Plain R listing and main N/H/P numeric commands stay compatible. Browser Q/resize
returns directly to main; reader/composer uses the existing pause/return. Preserve
one encoded-frame/output budget, five-second drain and normal eight sessions/two
credential permits/three owner helpers/four LWPs. Software VM only; no new capacity,
room federation, Reddit threads, mail or hardening adoption is claimed.

## Outcomes

[Actual software evidence](../evidence/room-browser-netbsd/README.md): 194 Rust per
OS / 118 Python, ten browser PTY controls per OS, eleven real restricted-SSH/native
controls with 100 rooms, 22 shared-reader regressions and old menu/art/directory
checks. Final native workspace outputs were cleaned/rebuilt after the SSH test
caught a stale incremental artifact; fresh synthetic state was used and no failing
attempt was relabeled as pass. 169 native inputs matched; nine host harnesses and
32 unchanged core/build/frontend inputs are separately recorded.

One browser member / 100 rooms sampled owner 5,248 KiB RSS / four LWPs; minimum VM
free 363.62 MiB, not an incremental/new capacity claim. The owned clone/cache are
removed and the saved playground is off. Plain/numeric commands remain available;
favorites/search/persistent room context, moderation and hardening remain later.
