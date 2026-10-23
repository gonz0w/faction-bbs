# Custom menu layouts and richer bounded ANSI import — #25

Owner, 2026-10-22: merge checked #123, then custom menus and richer ANSI art;
classic fallback remains, hardening later. #123 merged as fbfe245.

- [x] Verify exact-head check/review conversations and merge #123.
- [x] Preserve 133 executed seasonal inputs and its whole result record in
  `sources/pre25-layouts`; seventeen snapshots / twenty-nine historical identities.
- [x] Add explicit two-column/five-row entry coordinates, predictable arrows and
  Tab order, classic ordered fallback, bounded two-cell redraw and shared focus.
- [x] Import bounded SAUCE00 ANSI/CP437 with explicit non-blink iCE backgrounds;
  keep existing 80x16/16 KiB identity bounds and reject unsupported formats/controls.
- [x] Ship an opt-in original grid/art template with provenance and previews.
- [x] Validate host and native NetBSD software, including actual shared SSH UI;
  retain normal admission and output limits; no hardware/250-user/new RSS claim.
- [x] Update README, customization/decision/evidence documentation and CI.
- [x] Update issue, open and attach a focused PR against the existing feature base.

Design: sysop entry array remains hotkey/plain/Tab order. A grid declares all entry
coordinates; narrow terminals render that array vertically. Shared focus is an
entry index and must restore directly, independently of arrow movement. The same
validated theme/actions remain authoritative; layout is presentation only.

Primary art reference retrieved 2026-10-22:
https://www.acid.org/info/sauce/sauce.htm (SAUCE revision 00.5). Use only its
Character/ANSi record and non-blink bit. Metadata never grants artwork rights;
keep explicit theme provenance. Font/letter-spacing/aspect hints are not terminal
capability assertions. Larger/animated/long art and final scene identity stay open.

Validation and discoveries will be recorded below; no results assumed.

## Validation and discoveries

167 Rust tests pass on host/native NetBSD, 111 host Python, strict Clippy/fmt,
unchanged root/four-experiment no-JS inventories, active shared-UI Clippy/test.
Four local/worker custom controls per OS and four actual restricted SSH/native
custom controls pass. Nine seasonal controls per OS, eight actual shared calendar
controls and 22 shared reader/Unicode/graphics/revocation controls pass final code.
All six old previews reproduce unchanged; new grid/login SVGs are reviewed as
reference-cell artifacts. No terminal/font/pixel acceptance or new capacity claim.

Shared reconstruction previously restored focus by replaying Down. That cannot
restore array indexes in a spatial grid; restore the validated enabled index
instead. Tab remains a distinct ordered navigation key, while list/native transfer/
Lua door menus preserve their previous arrow behavior. Grid errors fail atomically.
The first SSH fixture submitted Done as a raw key; actual settings uses a line
command. Correct the fixture to send Enter, then rerun the whole shared matrix.

The [new evidence](../evidence/custom-menus-netbsd/README.md) compares 130 selected
final guest inputs and pins seven harnesses/two SVGs. Prior seasonal results remain
unchanged behind the seventeenth frozen snapshot. The disposable clone/cache are
removed and original playground remains off. Final art, long/animated art, arbitrary
canvas layouts and actual terminal/font acceptance remain #25/#13/#29 follow-ups.

Final review found that empty SGR reset cleared the current style but retained the
iCE intensity flag for subsequent background changes. Reset both states and add a
known-output assertion; rerun final host/native tests and the whole actual shared
matrix before collecting exact-source evidence. No old measured record changes.

## Handoff

[PR #124](https://github.com/gonz0w/faction-bbs/pull/124) is open and attached against
feat/103-articles-and-room-reader. #25 remains open for the stated art/long-form
follow-ups. Final code and regenerated native evidence pass clean Git-archive
repository/source/menu/seasonal/custom-menu checks. Current-head GitHub checks
remain the merge gate; no authorization to merge this new PR is assumed.
