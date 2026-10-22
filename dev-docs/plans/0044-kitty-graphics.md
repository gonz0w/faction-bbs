# 0044: Bounded inline Kitty graphics

Issue #117. Implemented and validated, ready for review, 2026-10-22. #119 merged at 7df112f.

- [x] Verify exact-head CI/reviews and merge #119; preserve 106 actual inputs and original result.
- [x] Review the primary Kitty protocol; implement trusted inline RGB only, positive bounded probes, streaming and cleanup.
- [x] Add a separate native member opt-in gate without general APC passthrough.
- [x] Validate bounded costs, refusal, malformed/late replies, interrupted chunks, fallback and native NetBSD software paths.
- [x] Update product documentation, evidence and decision; prepare focused PR.

Classic is default. No member graphics, PNG/compression/files/shared memory/network fetch, Sixel, host changes or new capacity claim. Keep 32-KiB/five-second output rules, normal eight-session admission and fixed helpers. Preserve thirteen snapshots and twenty-five historical result identities; use only a disposable software VM, keeping the saved playground off/private.

## Implementation notes

The encoder uses 3,000 raw bytes / 4,000 base64 bytes per nonfinal chunk; the
largest full command is 4,054 bytes. The original static asset is 38,400 bytes,
not per-member heap. Exact-command native regeneration needs local opt-in and a
local positive probe, plus order/rate/upload timeout checks. Query replies own
their input state, including a split command-ending LF on shared sessions.
Shared approval is rechecked for each drain-driven chunk. Best-effort cleanup
terminates partial APC and deletes only our image ID. Standalone reset writes
are bounded too; blocked-socket tests check deadline/nonblocking flag restoration.

During review, standalone rate state was moved outside the settings screen so
leaving/reopening it cannot reset the limit. Shared actual frame starts have an
independent rate guard. The preview clears the screen and uses a compact prompt
so a 40x18 display retains the image instead of scrolling it away. Final sources
are rebuilt/tested after these changes. The shared test preflight correctly
refused an unmarked clone; only the verified owned disposable VM was then marked.

## Final validation

151 Rust tests pass per host (macOS ARM64/native NetBSD 11 ARM64), seven
graphics and three native local-gate PTY controls per host, previous five modern
and five classic Unicode controls, reader/full-console regressions, nineteen
shared native UID/PTY controls, 104 Python tests and unchanged dependency/old
evidence guards. GNU screen returns no Kitty reply. 112 exact final build/run
inputs match native bytes; the clone is stopped/removed, saved playground off.
Real GUI/font/tmux/full native-to-member-SSH graphics certification stays #118.
