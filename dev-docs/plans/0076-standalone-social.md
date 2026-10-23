# 0076: Standalone visit availability and polite paging

Issue: [#137](https://github.com/gonz0w/faction-bbs/issues/137)
Status: completed implementation and software validation; #137 remains open
for optional idle notification UX

## Scope

After merging checked #173, bring local login and the inherited standalone worker
to visit-scoped explicit availability, the common themed Lounge panel and safe
menu-return/explicit-refresh badge. Reuse one optional lease and the existing
Board connection. Drive silent heartbeat checks through every waiting input path;
no background thread/helper, extra SQLite connection, schema or budget change.
Chat exit parts channels/reset send state; logout/error/disconnect closes the
visit. Preserve plain/classic access, current approval/epoch/quiet/block checks,
drafts, terminal restoration and no unsolicited popups or forced joins.

Standalone adapters do not gain shared-only file transfer or Lua/legacy doors.
Optional idle notifications and hardening remain deferred; keep #137 open for
remaining notification UX. Preserve all old result/source identities. Only owned
marked NetBSD software clones; saved owner playground stays off/intact.

## Progress

- [x] Check and squash-merge exact #173 with no unresolved reviews.
- [x] Freeze prior executed/harness inputs and whole result as snapshot 43.
- [x] Implement a synchronous visit owner and heartbeat-aware waiting inputs.
- [x] Reuse the common Lounge, safe menu badge and current privacy reads.
- [x] Validate local/worker classic/plain, real idle/editor waits, privacy,
  disconnect/reconnect, revocation and input/terminal restoration.
- [x] Run host/native workspace, terminal and recovery regressions; pin evidence.
- [x] Remove owned fixture/private inputs/vendor; preserve saved playground.
- [x] Complete offline/archive checks and focused PR.

## Implementation findings

Private Visit borrowing keeps the public login/menu/authentication interfaces
unchanged while allowing the existing core/common presenters to use their Board
APIs. Authentication and identity run before any lease exists; every long-lived
member input wait uses the same synchronous heartbeat owner. The Lounge checks
configuration at action/redraw boundaries, avoiding idle database query polling.

A final lifecycle review found that a Busy result while parting chat could have
been treated as a recoverable menu action, retaining channel joins under a live
visit. Cleanup failure now exits the visit. An actual deliberately held SQLite
write lock confirms process exit/terminal restoration; stale authority still uses
the existing expiry when the locked database prevents deletion. This is a focused
correctness fix, not a new hardening or abuse-testing phase.

The parity journey registers members and performs actual in-BBS approval. Its
full-screen worker and plain canonical prompt hold unfinished input during pages;
only menu return or explicit ? refresh displays a current signal. Quick development
preflight is explicitly excluded from sustained evidence. Existing shared service
sources are unchanged and their earlier live observations remain independently
frozen, rather than being presented as fresh shared-service execution.


## Final software evidence

351 Rust tests per OS, strict host clippy and maintained terminal regressions pass.
The twelve authenticated/approved standalone controls pass per OS, with real
parallel waits of 66.069 seconds on the host and 66.103 on NetBSD. All three
final heartbeat ages were seven seconds on each OS. Private unsent full-screen
text and a plain canonical room-ID prefix remain intact when paged. Both plain
worker/local access and current disable/revocation/EOF/cleanup contention pass.
Twenty complete-recovery controls per OS and 130 host Python tests pass.

The new record pins 320 selected native inputs and 42 host harnesses, plus 190
unchanged shared/core/runtime inputs and thirteen unchanged access/dependency/
credential inputs. Native sources match final current bytes and installed Rust
binaries match the clean final native build. Normal budgets/schema 17 remain.
The owned VM, private inputs and temporary vendor are removed; saved owner
playground disk/login/keys remain stopped/intact. All 67 offline commands are
required, including guard 57, and repeated from the final Git source archive
before publication. No physical hardware, new capacity or hardening claim.
See [the evidence](../evidence/standalone-social-netbsd/README.md).
