# 0015: Shared board ownership and credential peaks

Issues: [#55](https://github.com/gonz0w/faction-bbs/issues/55),
[#56](https://github.com/gonz0w/faction-bbs/issues/56).
Status: implementation/native validation complete; [PR #57](https://github.com/gonz0w/faction-bbs/pull/57) open for review. #54 merged at `26ae82f`.

## Goal and constraints

Compare a bounded shared-board/readiness terminal prototype with the current
per-member process model on a fresh NetBSD 11 ARM64 VM, two cores/1 GiB. Capture
credential peaks with one persistent native collector, known-allocation controls
and the unchanged two-slot Argon2 path. Preserve OpenSSH, separate terminal UID,
private database, current core approval/epoch/grants, durability and bounded work.
Normal gateway remains eight workers; experiments are not an appliance release.

## Progress

- [x] Verify passing #54 and merge it; read requirements and issues.
- [x] Build isolated shared-board prototype and persistent collector/calibration.
- [x] Validate authorization, failure/slow-reader isolation and terminal behavior.
- [x] Run matched native loads and credential calibration/bursts; freeze inputs.
- [x] Publish sanitized evidence, ADR, current docs, issue updates and focused PR.

## Design to test

Reuse the existing descriptor handoff and unmodified OpenSSH frontend. A bounded
readiness loop owns each terminal, opaque core session and chat lease together;
members send terminal input, never serialized member IDs/session capabilities.
Two fixed credential workers open the same board and return results tagged with
an internal connection generation, discarded after disconnect. No owner RPC.
One shared database handle services bounded actions; no transaction spans output.
This deliberately limited experiment does not replace the production member UI.
Compare alternatives (trim worker caches, shared RPC with per-user UI workers,
readiness terminals, replacement SSH) before an adoption decision.

## Validation and continuation

Run existing host checks and native application/SSH escape oracles. Run isolated
prototype tests and real SSH approval/revocation/reapproval, slow-reader,
disconnect/crash/cleanup/fresh-login checks. Matched distinct-account 100/250
offers stop on the existing conservative memory/process/file guard; failed offers
are not connected members. Record dataset, warmup, OS limits and actual outcomes.
Persistent collector uses guest monotonic start/sample/stop timestamps; calibrate
no work and known touched allocations before eight admitted password prompts.
Do not confuse cumulative successes or lifetime RSS sums with concurrent peaks.
Remove only this task's private VM, disks, keys, raw logs and forwards.

## Recorded results and handoff

2026-10-20: final frozen-source fresh-boot comparison admitted 168 worker menus
and 224 shared menus before the same batched headroom guard; both passed the
matched 100-member menu/chat stage, neither passed 250. Earlier shared trial
reached 232; report variability. Final pre-stop counters, all mandatory SSH/
frontend roles, cleanup times (6.893/1.287 s) and fresh login are saved. Baseline
and absolute memory vary; use the report's recorded counters rather than private
RSS attribution. Keep full #28 workload/repeat/soak and #55 adoption gates open.

Persistent 300-sample native streams captured exact touched 19/38-MiB controls and
zero no-work delta. Median cadence 21–22 ms despite 10-ms request; max gap 31.35 ms.
Actual successful credential rises 23.62/35.70 MiB and failed-check rise 33.93 MiB
are sampled lower bounds. Two held kernel slots rejected all eight; release,
hidden input and fresh login passed, with unchanged Argon2. No success-count<=2
assumption or lifetime-RSS-sum-as-peak claim. See research 0029/results 029/030.

Native regressions passed 58 Rust, five terminal workflows, seven local lifecycle,
20 SSH and 13 service/failure checks; isolated backend passed 16 additional
approval/epoch/private-grant/forged-control/PTY-stall/crash/durability/recovery
oracles. Fifty execution-source hashes match, 41 native copies verified; eight
binary identities saved. All private guest/disks/keys/credentials/raw logs and
forwards removed, kernel limits restored, other VMs/host SSH untouched.

Host Rust/Clippy, 31 Python tests, repository/archive checks and unchanged reviewed
39-NetBSD/41-Linux registry graphs pass. The isolated package has its own lockfile
and explicit CI format/build/Clippy/dependency/evidence checks; CI does not rerun
the guest. No normal schema/default/core crate behavior changed.

Next: retain #55 for complete theme/season/plain/resize/editor/room/sysop parity,
fairness and whole-host capacity work. Attribute remaining per-transport costs
without replacing base OpenSSH or weakening admission/credentials. #56's bounded
calibrated ledger is ready for closure with the PR; full production/hardware
capacity remains outside it. Never refresh frozen native hashes without matched
execution; old temporary builders can overwrite newer sources.

PR #57 published and attached. Required Linux CI passed at implementation head
`17b8138`; final documentation handoff also requires current-head CI. #55/#56 and
broader #28/#20 have concrete result comments. #56 closes on PR merge; #55 remains
open for parity/adoption. [#58](https://github.com/gonz0w/faction-bbs/issues/58)
captures remaining per-transport attribution/reduction toward guarded 250, keeping
base OpenSSH, full denial oracles, credential policy and whole-host accounting.
The current PR is not merged by this task; only requested #54 was merged.
