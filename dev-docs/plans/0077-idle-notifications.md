# 0077: Optional idle-menu page notifications

Issue: [#137](https://github.com/gonz0w/faction-bbs/issues/137)
Status: implementation and software validation complete

## Scope

After merging checked #174, add /notify on|off for the current visit, default off.
Only an idle main menu may show a quiet generic indication. Shared SSH/native,
local login and inherited worker ANSI/plain paths use one bounded presentation
state and current core eligibility. No interrupting chat/composer input, room
readers, games, transfer states or terminal probes; no bell, OS/desktop escape,
private-message body or automatic join. Plain main-menu input tracks its own
partial line without emitting ANSI. Keep focus and typeahead on automatic redraw.

Poll at most every two seconds only when opted in and safely idle; retain at most
eight current signal ID/state keys, never names/message text or queued events.
Re-check current approval/epoch/lease, feature state and both-party preferences.
Notifications reset on reconnect, require no durable schema change, helper/thread,
listener, SQLite connection or resource-budget increase. Keep schema 17. Complete
#137's ordinary supported-adapter scope if validation passes; hardening stays later.

## Progress

- [x] Check and squash-merge exact #174, with no unresolved reviews.
- [x] Preserve prior source/result identities in snapshot 44.
- [x] Implement bounded consent/coalescing and safe shared/local/worker menus.
- [x] Cover default off, explicit opt-in/out, deduplication/current privacy,
  expiry/disconnect/revocation and protected input/game/probe states.
- [x] Run host/native workspace and actual SSH/native/plain/worker journeys.
- [x] Complete maintained terminal/recovery evidence and source matching.
- [x] Remove owned clone/private inputs/vendor; preserve owner playground off/intact.
- [x] Prepare the reviewed source/evidence set for archive validation and a focused PR.

## Software validation

Strict host clippy, 354 Rust tests per OS, 130 host Python tests, twenty optional
idle local/worker ANSI/plain controls per OS and twelve actual shared SSH/native/
Lua controls pass. The clean native build runs on a marked NetBSD 11.0 ARM64 VM
with two cores/1 GiB. Actual member registration/approval, consent reset, current
privacy/revocation, quiet coalescing, partial input/escape/focus, unsent drafts,
Lua input and terminal probes are covered. No product-code correction was needed
after these tests. The new shared harness was corrected to follow the existing
direct terminal-settings return and separately recognize the native client's
final device-side terminal reset after its escape-free compact server frames.

Maintained host/native terminal, art, social and complete recovery regressions
also pass. Shared main-menu/composer/Lua waits measured 66.462/66.474/66.023
seconds with current heartbeat ages 7/4/1 seconds. The result pins 323 matched
native inputs, 44 host harness inputs and thirteen unchanged access/credential/
dependency inputs, plus final matching installed Rust/C door hashes. No new
package or schema. The owned VM/private inputs/vendor were removed; the saved
owner playground remains stopped with disk/login/keys preserved. Run all 68
maintained offline commands from the checkout and final Git source archive
before publishing the focused PR with #137 closure.
See [the current evidence](../evidence/idle-notifications-netbsd/README.md).
