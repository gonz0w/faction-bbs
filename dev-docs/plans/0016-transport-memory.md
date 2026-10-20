# 0016: Remaining transport memory

Issue: [#58](https://github.com/gonz0w/faction-bbs/issues/58).
Status: bounded comparison merged as PR #59; plan 0017 records the later repeatable-headroom result. PR #57 merged at `1826376` on 2026-10-20.

## Question and constraints

Reduce the remaining complete SSH transport costs on the existing two-core /
1-GiB NetBSD 11 ARM64 software fixture. Keep base OpenSSH, terminal/owner UID
separation, actual descriptor handoff, current approval/epochs/grants, hidden
input, two kernel credential slots and unchanged Argon2. Normal services remain
eight workers. The shared prototype is isolated and still lacks #55 UI parity.
No physical hardware, new SSH/crypto stack, external service or weaker guard.

## Execution

- [x] Verify required CI and merge #57; read #58 and existing evidence.
- [x] Attribute authenticated SSH, handed-off prompt and approved menu/chat
  states using complete whole-VM counters and every mandatory process.
- [x] Probe actual NetBSD libc allocator before tuning; compare small changes
  before a separately reviewed frontend/runtime candidate if justified.
- [x] Save each actual guard decision, offered/admitted counts, batch overshoot,
  limits, repeated fresh-boot results and source/native binary identities.
- [x] Exercise unchanged credential work with a useful high-load cohort,
  authorization/SSH/descriptor denial oracles, failures, cleanup and fresh login.
- [x] Prepare bounded findings/decision, current docs, backlog/PR handoff and
  host/native checks; remove this task's private VM inputs. Final CI and review
  status are recorded on the focused GitHub PR after publication.

## Evidence rules and handoff

Historical 029/030 source pins stay intact. New versioned tools/experiment/results
hold new measurements. RSS sums and virtual mappings are not private charge;
sampled KDF maxima are lower bounds. A short 250-menu/chat result alone cannot
pass broad #28 active-mix/repeat/soak/appliance or #55 parity/adoption gates.
The 128-MiB clean-file/free guard and process/file reserves are unchanged; record
batch effects rather than imply a continuous reserve. Never count rejected or
never-offered users. Never unlink live credential or pidfile locks.

## 2026-10-20 intermediate findings

The corrected shared Rust variant resets the request sequence for each new chat
lease; unique markers exposed the original 055 repeat-entry defect. Two default
fresh-boot trials passed 200 useful members and stopped at 224 approved menus.
The first compact allocator trial passed 200 and stopped at 248, about 0.88 MiB
below the unchanged 128-MiB guard. At that intermediate point no 250 workload had completed; later outcomes follow
below.
A separately reviewed fixed-buffer C handoff-only frontend is justified for the
next experiment by measured remaining prompt/transport costs. Rust core and base
OpenSSH stay; no production frontend or service default changes. Native denial,
TTY/descriptor, approval/grant and failure checks must pass before load evidence.

## Recorded outcome and handoff

Two fresh-boot default trials passed 200 menu/chat members, then stopped at 224
approved menus; both minimal-frontend/compact trials completed short 250-member stages;
allocator-only compact stopped twice at 248. All
mandatory access/SSH costs, exact guard decisions and batch overshoot are saved.
Fresh unique delivery exposed 055's new-lease sequence bug; the separately
versioned variant resets it without changing normal services or old frozen
records. Both profiles use identical corrected source. High-load wrong-password
cleanup and two real credentials pass while 198/248 members stay in chat.

See [research 0030](../research/0030-transport-memory.md), results 031 and
[ADR 0016](../decisions/0016-native-allocator-comparison.md). Native regressions,
host checks, complete source/binary provenance and private-VM cleanup are saved.
#58 remains open: one 250-member workload passed the final guard and the repeat
finished 4.10 MiB short. Required CI must pass at the PR's final head; CI checks source/saved evidence,
not a VM rerun. Keep #55 parity/fairness/scoped configuration and #28 active-mix/
soak/admission/resource work open. No normal eight-worker/default/schema/KDF
change, new registry feature, embedded SSH or physical-device claim. Historical
execution hashes must never be refreshed without matching versioned execution.

Next #58 slice: investigate repeat-to-repeat anonymous/wired/kernel variation
and a bounded margin improvement without lowering the guard, changing SSH or
weakening credentials; save actual live-process RLIMITs and qualify warm/cold
credential work. The minimal candidate still requires separate production review.
Do not modify frozen sources or refresh hashes without matched versioned evidence.
