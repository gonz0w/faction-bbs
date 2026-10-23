# 0069: Deliberate native travel preferences

Issue: #159. Status: implemented/validated; focused PR handoff pending.

## Scope

First usable increment: explicit local native-client ordinary/compact text choice
and per-file payload ceiling before connecting. Local Boards travel panel with
review/apply/cancel, preserved highlight, classic 40x18 layout; fixed --compact and
--file-budget BYTES for direct profile visits. Preferences stay in process memory,
never remote output/profile paths or automatic travel history. Zero defers all files;
normal default remains existing 16 MiB. No DB/server schema change.

Compact uses the existing SSH TERM=faction-compact/classic fallback, no new remote command,
route, network authority, weaker host verification or automatic downgrade. Disable
optional board graphics for this visit. Do not strip arbitrary server content into
new trusted controls or change room unread/send semantics. File ceiling applies
to known payload bytes before upload reservation/download grant/SFTP, preserving
existing confirmation, size/digest binding, quotas and cleanup. It is not an exact
SSH/IP wire or session budget; protocol overhead/retries are extra. No automatic
resume/retry. Existing native buffers/deadlines stay fixed.

Offline packets/reply synchronization (#145), non-IP delivery, byte-rate shaping
and named link-profile validation remain follow-on scope under #159; do not close
the parent. This batch adds no Tor/Reticulum/delivery claim, registry package,
service/helper or server-default changes. Keep private mail device-side, owner
playground stopped/preserved and hardening later.

## Progress

- [x] Merge passing #166 and preserve its actual sources/result.
- [x] Implement/review local travel choice and pre-transfer size gate.
- [x] Validate local UI, cancellation, malformed sizes/limits and native NetBSD SSH.
- [x] Update README/docs/ADR/evidence/CI and prepare focused follow-on PR.

## Discoveries

`TERM=dumb` correctly omits native Files advertisement. The final implementation
uses explicit `faction-compact` as a presentation/protocol hint: the updated service
renders plain text but advertises the existing connection-bound native Files
protocol. Ordinary dumb SSH stays escape-free and all file authority still comes
from current approval/grants. Worker/local presentation recognizes the hint too.
No route or authentication negotiation changes. The 16-MiB defaults are unchanged.

Final room comparison uses real article composition and the actual returned post
ID. History completion intentionally retains its existing read acknowledgment;
the harness must not assume deleted SQLite IDs are reused or demand no unread
write from a complete normal reader. Frozen pack README links belong to their
original tree; repository policy now skips only those historical Markdown links,
while no-JS/newline/conflict and exact-hash guards remain. A negative test proves
JS inside snapshots is still rejected. Python suite is now 130 tests.

## Final validation

304 Rust tests per OS, 130 Python, six local travel PTYs per OS, nine actual
restricted SSH/host-native/NetBSD-native/SFTP controls and seventeen existing
complete-recovery controls per OS (native distinct UID CLI). Exactly-at-limit
50-byte synthetic transfers preserve checksums/pending review; over-limit/zero
ceilings create no grant or reservation. Complete normal read acknowledgments,
private-area denial, cancellation and pinned host mismatch remain. Raw dumb SSH
Files emits no escape bytes. Full host terminal CI/prior community/art/editor/vault
regressions and unchanged reviewed registry inventories pass.

The new record pins 272 matched native execution inputs, 29 host harness/review
inputs and thirteen unchanged access/credential/dependency inputs, with current
built/installed binary hashes and a native client literal check. Server/CLI changes
are limited to explicit compact-native presentation/Files advertisement. Schema 14
and all credential/session/helper/output budgets remain. Owned clone/cache and test
credentials removed; owner playground remains stopped/preserved. No wire-budget,
offline, new-route, throughput, physical-radio or capacity claim. #159 remains open.
