# 0078: Distinct Who caller list and universal presence

Issue: [#176](https://github.com/gonz0w/faction-bbs/issues/176)
Status: implementation and software validation complete; follows #175

## Contract

Main-menu Who shows broad current location plus availability. Group `/who` is the
slash-command entry to the same board-wide caller list. The Lounge remains its
own availability/invitation/privacy desk. All three read one current lease-bound
visibility/availability state. Explicit per-visit opt-in, hidden reconnect, current
approval/epoch/blocks and no hidden counts remain. No private room/channel names,
draft text, game detail, IPs, last-seen history or native private-mail activity.
Existing `/names` channel participation remains a separate protocol concept.

Use a bounded host-derived activity enum and one current row per existing lease,
not user-written location strings or a retained event history. Schema 18 adds the
transient activity relation; export/restore drops all live authority/activity.
Validate schema-2..17 read-only compatibility, stopped complete upgrades and no
mixed-version/in-place downgrade. Keep ordinary session/credential/helper/output
limits; no new thread, SQLite connection/listener or registry package.

## Progress

- [x] Log the owner contract and preserve snapshot 45 (332 prior inputs/full record).
- [x] Implement common Who presentation, group command and theme action.
- [x] Publish current broad activity from shared/local/worker adapters.
- [x] Validate universal visibility/availability, privacy, lifecycle and recovery.
- [x] Run host/NetBSD software checks and pin final exact inputs/binaries.
- [x] Remove only owned fixtures; saved owner playground remains off/intact.
- [x] Prepare the reviewed source/evidence set for final archive checks and the focused PR.

## Sequencing

#175 remains a separate checked idle-notification PR. This branch starts at its
exact head; publish a dependent PR and merge in dependency order. Hardening stays
later. Update docs/README/AGENTS/CHANGELOG with actual behavior and test limits.


## Validation progress

Strict clippy and 359 host Rust tests pass. Twenty-one actual registered/approved
local/worker ANSI/plain controls verify Who, group and Lounge visibility/status,
current Menu/Who/Chat/Lounge/Rooms transitions, private unfinished draft input,
mutual blocks and fresh hidden reconnect. The maintained host Rust/terminal/art/
recovery workflow passes, including 21 complete recovery controls and 130 Python.
The initial native run exposed the old current-schema assertion in the writing
desk helper; current-schema seed/writing checks now expect 18 and the native
clean build/regressions were rerun with final matching sources. No product-code
correction was required for this helper failure. The new shared harness now waits
for explicit private-channel grant acknowledgments and checks the Who payload
separately from the caller's own legitimate channel prompt.

All twelve shared SSH/native/Lua/Files Who controls pass, plus existing shared
idle-notification and sustained availability regressions. Main-menu/composer/Lua
waits measured 66.460/66.430/66.460 seconds, heartbeat ages 7/5/2 seconds. The
current result pins 328 matched native execution inputs, 47 host harness inputs,
thirteen unchanged access/credential/dependency inputs and matching installed
Rust/C door hashes. Guard 59 preserves this record offline. Complete the 69
maintained offline commands in the checkout and committed Git source archive
before publishing the focused dependent PR. No capacity/hardware/hardening claim.
