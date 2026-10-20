# 0018: Bounded shared UI and ANSI themes candidate

Status: proposed implementation candidate; production adoption deferred.
Date: 2026-10-20. Authority: owner requested full shared UI/themes and preserved
250-member headroom after merging #60; PR review accepts this experiment.
Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55), related #20/#28.
Supersedes: none. Evidence: [research 0032](../research/0032-shared-ui-themes.md).

## Question and mandatory gates

Extend the measured shared owner to the existing room/chat/sysop/theme behavior
without importing caller identities, adding per-member workers or trading away
password/SSH security. Required approval, epochs, grants, safe terminal handling,
bounded work/output/credential lifetime and honest native memory accounting are
mandatory. Full maintained-service confinement, hostile admission, clock policy
and long active workloads remain unknown and block production adoption.

## Alternatives and evidence

The existing worker path supplies rollback and functional reference but its prior
1-GiB measurement stopped admission below 250. Shipping the small historical probe
fails UI parity. Wrapping blocking consoles in per-user threads/processes restores
costs the shared experiment is testing. The selected candidate instead reuses the
same unmodified core/terminal libraries through explicit nonblocking prompt states.
No popularity scoring or unmeasured numeric winner is asserted.

The [native report](../research/0032-shared-ui-themes.md), retrieved/generated
2026-10-20, is primary executed evidence. Final 27 native UI/fault oracles using SSH and direct descriptor/PTY probes pass with
the candidate policy; root native 58 tests, 20 SSH, 13 lifecycle and 20 direct
frontend checks pass. Three fresh-boot 250-member full-UI runs pass the unchanged sampled guard; the lowest stress headroom is 135.09 MiB. Completed-stage margins are 180.39–180.51 MiB. Source/binary identity, full SSH costs, restored host settings and cleanup are recorded.
Confidence is high only for those specific functional observations. Existing
product approval/ANSI/season requirements are owner decisions, not capacity data.

## Decision, tradeoffs and reversal

Implement an isolated unique Cargo package and separate build directory. Share one
validated theme/season snapshot, lazy capped output and fixed credential workers.
Rotate and bound work rather than execute a pasted stream of database actions.
Preserve read-after-flush and exact post retry semantics. Keep the normal broker's
eight-worker defaults; no automatic candidate/allocator/NTP installation occurs.

Tradeoffs: the shared owner increases failure scope; explicit prompt states need
parity tests as local features evolve; rate limits constrain paste throughput;
manual season changes require restart. The minimal C frontend retains manual
memory-safety and duplicated-contract review costs. Guest-wide host policies need
scoping before maintained deployment. No new registry dependency is required.

Reversal cost is low for application selection: stop/reap the candidate, restore
normal service, keep the identical schema/private database. Host-policy rollback
is separate and must restore exact original settings. Historical executed inputs
remain byte-identical; future root changes still need real source snapshots rather
than refreshing old hashes to unexecuted code.

## Validation and reopening

A prototype/native workload is required because documentation cannot establish
TTY safety, permission parity or whole-VM memory. See the report and saved ledger
for complete methods/results and uncertainty. Keep the unchanged 128-MiB predicate,
include all SSH costs and preserve any failed trials. Reopen for failed UI/permission
or memory results, source/binary drift, a new dependency or broader workload evidence.
Normal-service adoption, confinement and host policy remain #55/#20; capacity,
soak, complete budgets and authenticated-transport admission remain #28.
