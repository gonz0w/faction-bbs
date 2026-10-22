# Custom menu layouts and richer bounded ANSI import — #25

Owner, 2026-10-22: merge checked #123, then custom menus and richer ANSI art;
classic fallback remains, hardening later. #123 merged as fbfe245.

- [x] Verify exact-head check/review conversations and merge #123.
- [x] Preserve 133 executed seasonal inputs and its whole result record in
  `sources/pre25-layouts`; seventeen snapshots / twenty-nine historical identities.
- [ ] Add explicit two-column/five-row entry coordinates, predictable arrows and
  Tab order, classic ordered fallback, bounded two-cell redraw and shared focus.
- [ ] Import bounded SAUCE00 ANSI/CP437 with explicit non-blink iCE backgrounds;
  keep existing 80x16/16 KiB identity bounds and reject unsupported formats/controls.
- [ ] Ship an opt-in original grid/art template with provenance and previews.
- [ ] Validate host and native NetBSD software, including actual shared SSH UI;
  retain normal admission and output limits; no hardware/250-user/new RSS claim.
- [ ] Update README, customization/decision/evidence documentation, CI and issue;
  open and attach a focused PR against the existing feature base.

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
