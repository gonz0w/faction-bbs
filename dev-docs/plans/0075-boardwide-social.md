# 0075: Connection-scoped availability and polite pages outside chat

Issue: [#137](https://github.com/gonz0w/faction-bbs/issues/137)
Status: completed implementation and software validation; #137 remains open for
standalone parity and optional idle notification UX

## Goal and scope

After checked/squash-merged #172, extend the primary shared SSH/native service to
one optional lease for the complete BBS visit. Keep a member's explicit lounge
availability alive while reading rooms, editing or playing. Leaving chat parts
all channels/resets its send context but retains the connection lease. Disconnect
ends it. Add a themed Lounge main-menu action, current-read panel, and generic
coalesced page badge on safe ordinary menu redraws. No popup over drafts/doors,
forced switch, custom private-message text or new process/listener/connection.

Classic/plain shared terminals get the same panel. Legacy local/worker rollback
keeps its existing chat-only scope; wider standalone visit ownership is not
silently claimed. Keep #137 open for adapter parity and idle-menu notifications.
Schema stays 17; normal budgets and later hardening remain unchanged.

## Progress

- [x] Merge exact checked #172 after resolved review threads.
- [x] Preserve 316 input files and the old result as snapshot 42; retain all 55
  historical result identities/hashes unchanged.
- [x] Implement leave-all/current advertisement/config APIs and shared visit lease.
- [x] Add gated themed Lounge action, re-fetching panel and safe menu badge.
- [x] Complete authority/history/lease/presentation tests and sustained outside-chat
  SSH/native room/editor/door checks with classic/plain fallback.
- [x] Complete native/host recovery and existing regressions; record final inputs.
- [x] Remove owned clone/credentials/vendor; preserve owner playground off/intact.
- [x] Finish maintained checks/archive verification and focused PR.

## Discoveries and validation

One existing opaque chat lease can represent a shared BBS connection without
joining any channel. Clearing channel joins/receipt separately avoids extra
SQLite connections, presence rows, helpers or a schema migration. The server
heartbeats it without producing output in all UI states. Badge data is fetched
only during safe menu rendering; the panel never retains old eligible names or
page text as notices across redraw/revocation. Plain input remains bounded ASCII.

Use current core epochs/approval/lease/quiet/block checks, strict clippy/release
workspace tests, common panel and native shared PTYs, maintained Rust/Python/
offline/source-archive checks. Only a marked owned NetBSD software clone; no
physical hardware, 250-member capacity or new hardening claim.

The maintained historical shared-UI prototype needed an explicit denied arm for
the newly added action enum variant. Its exact prior source was already preserved
in snapshot 42 and older snapshots; no historical hash was changed. The primary
service implements the capability, while the prototype/worker cannot enable it.

Final validation records 351 Rust tests per OS, strict host clippy, the full
maintained Rust/terminal workflow, 130 Python tests, eight actual shared controls
and twenty complete-recovery controls per OS. The three real menu/composer/Lua
waits were 66.011/66.009/66.018 seconds with heartbeat ages 6/3/0 seconds. The game
check holds a prefix and submits an out-of-range frequency that would become a
valid frequency if the prefix were lost. Classic/plain, revocation and feature
disable pass. Final evidence pins 317 matched native inputs/41 host harnesses and
installed Rust/C door hashes; the door uses base NetBSD Lua with no new package.

The live menu test caught the decoder ignoring standalone question marks. The
new refresh key is now decoded only in ground state, with a regression assertion
that control-sequence tails cannot invoke it. A resize harness assertion initially
included an earlier screen's text; the final test inspects only the newly produced
frame and confirms blocked invitation text stays absent. No runtime privacy
exception was added. All 66 offline commands, including guard 56, are required
before publication and repeated from the final Git source archive.

The owned software VM, private test inputs and temporary vendor were removed.
Saved owner playground disk/login/keys remain stopped and intact. Schema stays
17 and normal resource defaults stay unchanged. No new physical hardware,
250-member capacity or hardening claim. See
[the recorded evidence](../evidence/boardwide-social-netbsd/README.md).
