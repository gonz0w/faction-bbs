# Named room browser: actual NetBSD software validation

2026-10-22, #21/#12/#25. [Sanitized results](results.json) record an owned disposable
2-core/1-GiB NetBSD 11 ARM64 software VM, with host and actual NetBSD native clients.
See [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/21),
[ADR 0052](../../decisions/0052-named-room-browser.md) and
[member workflow](../../../docs/room-browser.md). The saved playground stayed off.

## Passing checks

- **194 Rust tests per OS**, native Rust 1.96.0; host Clippy with warnings denied,
  formatting and **118 Python tests**. New presentation checks cover first-100
  pages/actions, ID stability/removal, next-unread selection without read mutation,
  inert classic/native Unicode text and compact frame bounds.
- **Ten local/worker room-browser controls per OS**: pending denial, named arrows/
  hotkeys/cards/pages, selected posting without Room ID, next-unread/new reading,
  selected history, compact partial-unread retention, plain compatibility, worker
  route, resize and active revocation before composing. Posting clears stale card
  controls and explicitly labels room #2; the fixture verifies that destination.
- **Eleven actual shared restricted-SSH/native controls**: 100 rooms created through
  real sysop menus, last-page/card #100, selected-room posting, unread/drained new
  reading, host native Unicode subject composer/reader, actual NetBSD native history,
  compact partial-unread, plain R with no ANSI, resize, resource samples, pending
  denial and current revocation. These named controls combine related steps; the
  count is eleven, not a count of every individual assertion or SQL operation.
- **22 existing shared-reader UID/PTY controls**, including partial/full message
  acknowledgment, real wide/compact themes, current approval and Unicode-draft
  revocation; that suite is not an additional dedicated-SSH admission proof.
- Full local console and plain/ANSI reader regressions pass on both OSes. Directory
  PTYs pass eleven controls per OS; native launcher seven and operator UI eight.
  Artwork passes eighteen native previews, seven local/worker routes and seven
  actual restricted-SSH/native login routes. All four old standalone manifests
  pass locked all-target compile checks; no new standalone UI behavior is claimed.

## Native build correction and preservation

The first SSH posting test correctly rejected a stale native executable: copied
source hashes matched the composer polish, but an incremental native build had
retained the earlier artifact. That attempt produced no passing new result. Final
validation cleaned all ten workspace packages' release outputs, rebuilt/tested the
workspace, verified the installed service's destination-label literal and exact
binary match, then reran on fresh synthetic state. The failed-attempt board/logs
stayed private in the disposable clone until cleanup. Production configuration,
authorization and limits were never relaxed to make a test pass.

169 selected native build/runtime/validation input hashes match the actual copied
tree; nine relevant host harness inputs and installed/host binary hashes are
recorded separately. All **32 checked core/build/frontend inputs** remain unchanged,
including database/auth code, Cargo files and native terminal frontend. This is a
selected input set, not every VM/toolchain/host build file. Old R-list fixture
expectations were preserved before updating them for the browser.

## Resources and scope

After the plain-mode stopped restart, eight half-second observations with 100
rooms and one approved browser member show **5,248 KiB owner RSS / four LWPs**;
minimum sampled whole-VM free **363.62 MiB**. Native build/cache/restart history
influences this observation. It is not an incremental memory estimate, a peak/
soak, new 250-member capacity, physical SBC or all-NetBSD-port claim.

The presenter keeps only a selected ID/card flag per shared member, with transient
first-100 rows and four displayed rooms. A frame caps at 8 KiB; normal eight
sessions/five-second admission, two credential permits, three owner helpers/four
LWPs, 32-KiB queued output and five-second drain remain. No schema, dependency,
extra listener/helper, room federation, native mail or hardening adoption occurs.
Selection never marks read; existing complete-message and output-drain rules stay.

## Reproduce and clean up

Use an owned marked software clone, reviewed locked/vendor sources and an
unprivileged builder. Clean native workspace release artifacts when updating the
copied candidate; file hashes alone cannot prove which sources an old binary used.
Run workspace release tests/builds, `tools/smoke_room_browser.py --help` and existing
console/reader/directory/art checks. Install the final normal owner/service/TTY.

`tools/smoke_shared_room_browser.py --help` needs private admin/member argv,
synthetic credentials/profile and a guest-native argv. It requires the disposable
marker and fresh `/var/db/faction-profile-fixture` board; creates 99 rooms through
sysop UI, plus synthetic pending/approved/revoked membership; and stops/restarts
only the fixture for plain mode. The guest route executes the actual NetBSD-built
client under an unprivileged device account with pre-enrolled host keys. Run the
old shared-reader suite in its own fresh board and restore the synthetic profile
config before artwork checks. Never point these tools at a saved/live board.

Keep credentials, argv, keys, serial and failed transcripts private. Verify owned
QEMU PID/disk identity before stopping, wait for exited processes/closed disk
handles and remove only its task-owned clone/cache. Confirm no open handle to the
saved original; teardown flags report actual completed cleanup.

Snapshot 24, `sources/pre21-room-browser`, and
`board-launcher-baseline-manifest.json` preserve 178 previous source/room-list
fixture inputs and the whole original launcher record. All 24 snapshots/37 prior
result identities remain fixed. Preserve this new measured source/harness set and
whole result before future changes; never refresh historical hashes to new code.
