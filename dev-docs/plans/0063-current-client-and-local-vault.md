# 0063: Consolidate the current client and trusted local vault

Issue: [#90](https://github.com/gonz0w/faction-bbs/issues/90), [PR #94](https://github.com/gonz0w/faction-bbs/pull/94)
Status: implemented; final PR checks pending

## Goal and constraints

Owner requested all remaining PRs merged and another useful batch after trying the
playground. Stop/preserve that board. Merge #153 → #105 → #102 in dependency order,
then reconcile #94 with the current client and merge it after checks. Give the
member a deliberate local-vault entry from the board launcher, before any SSH
connection. No server decrypt, new crypto design, network inbox, roaming proof,
SSO, schema/server-default or hardening changes. Keep #90/#91/#88 open.

## Progress

- [x] Stop/preserve the owner's private playground and merge checked #153.
- [x] Preserve both source baselines and whole historical results before reconciliation.
- [x] Resolve older vault/current menu, graphics, dependency and docs conflicts.
- [x] Add optional local picker vault entry, with no remote/SSH unlock authority.
- [x] Validate combined client/vault and existing product on macOS and native NetBSD.
- [ ] Complete checked stack merges, update status and keep owner playground off.

## Discoveries and decisions

Only squash merges are enabled; merge children into parents in order before main.
Thirty immutable snapshots now preserve 213 prior current runtime/harness inputs
and 93 original vault inputs, with forty-three unchanged historical record
identities. Exact original Git blobs match every vault executed digest; no hash
was refreshed. Existing source lookup resolves both revisions by recorded digest.
The reviewed dependency inventory is the additive union of the two existing sets:
seven crypto packages from #94 plus the current Unicode-width package. Scanner
confirms the integrated selected features/licenses/no-JS policy without changes.

The old private playground is explicitly stopped and retained. Use a separate
owned software clone for native checks; never point disposable cleanup at that
preview. Admission/credential/session and server/helper budgets stay unchanged.

## Validation and outcomes

Pending final product changes and tests. Initial integrated workspace check,
rustfmt, reviewed dependency scan, thirty-snapshot guard, member-page evidence and
original local-vault evidence pass. These are software/integrity checks, not new
capacity, independent crypto review or network-mail implementation claims.


macOS release checks so far: 241 Rust tests; original thirteen vault PTYs, eight
new local picker/vault controls, seven unchanged launcher and three native graphics
controls. A final highlight-preservation check caught M returning to the previous
visited board rather than the current highlight; preserve the selected index in
the Mail destination and verify it after locking. The native owned clone gets a
clean final client rebuild after that correction. The original vault inactivity
test also needed the current Terminal graphics=false field; no crypto behavior
changed. The seven original reviewed crypto packages and current Unicode table
retain their exact selected versions/licenses/features. #105 is now merged into
#102; parent checks run before main. #90 remains incomplete for actual network mail.


Final native software outcomes: 241 Rust tests on both hosts; thirteen original
vault and eight new picker/vault PTYs per OS; seven launcher and three graphics
controls per OS; fourteen actual shared member-page SSH/macOS-native/NetBSD-native/
plain regression controls. Both clean native workspace and final clean client
builds pass. New record pins 209 matched native inputs, thirteen host/review inputs
and unchanged non-client sources. See [current evidence](../evidence/client-vault-integration-netbsd/README.md).
The apparent final highlight failure was an over-consuming PTY assertion: the
footer wait had already consumed the new highlight. The corrected test inspects
the actual new screen bytes; both hosts pass. No runtime behavior was weakened.

#153, #105 and #102 are merged after their checks; main now has the complete
current product tree. #94 reconciles the original vault with that tree and adds
the deliberate local picker entry. Its new current evidence guard brings saved
guards to forty-four. Thirty snapshots preserve forty-three historical identities;
new evidence does not refresh them. The owner's stopped preview stays retained.
