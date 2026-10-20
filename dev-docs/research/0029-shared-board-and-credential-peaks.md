# 0029: Shared board ownership and persistent credential peaks

Date: 2026-10-20. Issues [#55](https://github.com/gonz0w/faction-bbs/issues/55),
[#56](https://github.com/gonz0w/faction-bbs/issues/56).
Decision: [ADR 0015](../decisions/0015-shared-board-readiness-experiment.md).

## Outcome

The final controlled comparison admitted **224 approved distinct-account SSH menu
sessions with the shared prototype versus 168 with workers**, before the same
memory guard stopped admission. Both completed the **100-member menu/chat stage**;
neither completed the planned 250 stage. A preceding shared trial admitted 232,
which demonstrates variability rather than a supported capacity. Normal service
still defaults to eight workers. Shared UI/operational adoption and 250-user
capacity remain open under #55/#28.

A persistent native sampler captured real credential spikes. Calibration measured
known touched 19/38-MiB allocations exactly and a zero no-work delta. Actual
credential samples observed rises of 23.62–35.70 MiB for successful bursts and
33.93 MiB for failed checks; those are sampled lower bounds, never instantaneous
upper bounds. Unchanged Argon2, two kernel slots, eight admitted prompts, hidden
input, explicit saturation, release and fresh-login recovery all passed (#56).

Evidence: [comparison/provenance JSON](results/029-netbsd-shared-board.json),
[complete credential timing/counter matrices](results/030-netbsd-credential-peaks.json).
The latter references the former by SHA-256. All 50 frozen execution inputs match
current source; 41 native-source copies were compared to the actual guest build/
execution copies. Eight native binary identities/sizes are recorded. Private VM,
disks, generated keys/credentials, raw logs and loopback forwards were removed;
the developer's SSH and unrelated VMs were untouched.

## Method and scope

Fresh signed/hash-verified NetBSD 11.0 ARM64 software VM, QEMU/HVF, two vCPUs,
1024 MiB configured RAM; NetBSD reports about 1006 MiB usable physical RAM and
UVM about 972 MiB managed. Native Rust 1.96.0, GCC 12.5.0, Python 3.13.14; base
OpenSSH and its restricted configuration/frontend are unchanged. Manifest/key
verification used HTTPS retrieval, without independent web-of-trust validation.
This is not physical SBC, another NetBSD port or appliance evidence.

Each model starts after an offline synthetic-fixture reset and guest reboot to
avoid retained kernel/cache history. Dataset: sysop plus 1000 approved accounts,
1001 independently salted hashes, 50 rooms, 100000 1024-byte posts. Separate real
SSH transports use distinct BBS accounts; the SSH admission key and terminal OS
identity are intentionally shared, with separate BBS authentication/approval.
No reused logical accounts or rejected offers count as connected members.

Five-second seeded baseline, sequential/staggered approved admission, then about
10 seconds each of menu and local chat idle per completed stage; one authored
chat delivery to every recipient. Credential work is unchanged Argon2id v19,
m=19456 KiB,t=2,p=1. Experimental local launch burst is eight/refill 100 ms,
connection cap 250. Normal gateway defaults are unaffected. Only the implemented
UI comparison surface is exercised; this is not the full #28 active workload,
10-minute warmup, repeated acceptance benchmark or 24-hour soak.

Kernel maxproc/maxfiles are explicitly 2048/16384 during comparison, restored to
1044/3404 afterwards. Observed post-experiment owner launch defaults include 1024
soft descriptors/processes and an 8192-KiB stack; they are labeled separately from
observed run kernel limits. The guard checks before each group of eight admissions:
free + max(0, clean file - wired) >=128 MiB, zero swap, process reserve 64 and
file reserve 128. A batch can cross that threshold before the next check; saved
pre-stop counters show actual headroom. Inactive anonymous pages are not available
memory. No OOM or swap occurred in this fixture.

## Matched 100-member results

All memory columns are MiB. Anonymous memory is whole-VM, not application USS/PSS.
Kernel counters overlap; do not add every VM column. CPU is a short whole-guest
observation expressed as percent of one core (two-core ceiling 200%), not a
throughput or hardware guarantee.

| Model | Baseline anon | Chat anon | Anon increase | Chat kernel pool | Chat CPU | All-recipient delivery |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Workers | 25.34 | 383.07 | 357.73 | 100.29 | 6.90% | 0.347 s |
| Shared owner | 33.59 | 309.62 | 276.02 | 80.45 | 4.69% | 0.107 s |

Baselines and repeated admission limits vary, so treat the consistent direction
of savings more confidently than an exact per-user allowance. Full dedicated
OpenSSH process costs and frontend costs are retained in each ledger; RSS sums
repeat shared pages and are never labeled private charge or simultaneous peaks.

| Final model | Approved menus admitted | Completed menu/chat stage | Pre-stop anon | Pre-stop pool | Stop/reap time |
| --- | ---: | ---: | ---: | ---: | ---: |
| Workers | 168 | 100 | 617.25 | 144.34 | 6.893 s |
| Shared owner | 224 | 100 | 633.27 | 135.81 | 1.287 s |

The 224-user shared stop retained 224 frontends, 449 dedicated SSH processes and
one shared owner; the worker stop retained 168 members, 168 frontends, 337 SSH
processes and one broker. Both snapshots had 695 total guest processes. This
correlation motivates isolating remaining per-transport/process overhead; it does
not identify exact private charge. The shared owner's single-process RSS was
about 5.61 MiB at stop, but mandatory helpers/SSH still belong in the whole-system
budget. Proposed full-application budgets have not been demonstrated or accepted.

Both active-load stops reaped backend/member/frontends and a fresh login/logout
passed. Exploratory same-candidate runs reached 168/232; the final counter-retaining
ledger reached 168/224. The initial host harness had a function-name collision
after a 100-user pass/cleanup; it was corrected and the complete comparison rerun.
Fresh-boot dedicated SSH readiness was checked after the fixture's private NIC
received its address; automatic appliance boot remains a separate gate.

## Shared-owner boundary checks

The focused candidate passed 16 native oracles: pending registration/login,
active revocation, fresh reapproval, private grant removal, outsider denial,
regrant requiring a new join, inability to borrow another connection's session,
arrows/hotkeys/core post, frontend crash isolation, forged control/owner-command
denial, actual non-reading guest PTY backpressure, fresh login after it, another
SSH member remaining usable, backend crash closure, durable-post/integrity
recovery and fresh login/logout. The stalled PTY closed in about five seconds.
The final extra synthetic account/post are explicitly counted separately from
the matched dataset. Normal gateway regression also passed 58 native Rust tests,
five terminal workflows, seven local lifecycle oracles, 20 actual SSH oracles and
13 service/load/failure oracles; positive native CMSG and escape denials remain.

The [standalone prototype](../../experiments/055-shared-board/README.md) owns each
terminal, opaque core session and chat lease together. Internal non-reused
connection generations bind async auth results; members cannot submit an identity
capability or owner RPC. The terminal UID cannot read private SQLite. Two fixed
credential workers retain the core kernel budget. Bounded output/pages complete
core work before terminal writes. This reduces duplication but creates a larger
backend failure domain. DB busy/fairness, hostile high-rate input, complete
theme/season/plain/resize/editor/room/sysop parity and operational adoption remain
gates. A synthetic boundary pass is not a production isolation claim.

## Persistent credential calibration and bursts

One native process stays alive over one SSH channel per trial. It emits explicit
monotonic start/sample/stop markers and query-start/query-finish timestamps.
Each trial retained all 300 samples; compact JSON matrices include explicit
column paths and every sanitized VM/process/SSH sample, not just selected peaks.
Calibration holds known page-touched allocations for 200 ms and compares samples
fully inside that interval with prior samples from the same guest clock.

| Known touched allocation | Observed anon increase | Fully inside samples | Median cadence |
| --- | ---: | ---: | ---: |
| 0 MiB | 0.00 MiB | 11 | 21.33 ms |
| 19 MiB | 19.00 MiB | 10 | 21.41 ms |
| 38 MiB | 38.00 MiB | 10 | 21.51 ms |

The collector requests 10 ms; measured median cadence is about 21–22 ms, maximum
gap about 31.35 ms across these trials. Median query work is about 0.12–0.13 ms
and maximum below 0.28 ms; that measures sysctl/query duration, not complete
collector/SSH CPU overhead. Guest stream duration and host elapsed time are both
recorded; host time includes SSH startup/transport and the clocks are not assumed
to share an epoch. Absolute scheduling skips missed intervals rather than emitting
catch-up samples. Kernel timer resolution/scheduling remains visible; the native
calibration provides sensitivity evidence rather than assuming requested cadence.

Eight distinct approved accounts are already at hidden password prompts before
each actual burst. The collector does not inspect/acquire credential locks.
The first trial has two explicit external kernel-slot holders; release is tested
before incorrect and correct credentials. Each sum below is eight outcomes.

| Case | Successful | Invalid credentials | Busy | Sampled anon rise from first sample |
| --- | ---: | ---: | ---: | ---: |
| both_slots_held | 0 | 0 | 8 | 0.15 MiB |
| wrong_password | 0 | 2 | 6 | 33.93 MiB |
| valid_password | 2 | 0 | 6 | 23.62 MiB |
| valid_password_repeat | 2 | 0 | 6 | 35.70 MiB |

Successful/wrong credential outcomes arrived about 36–41 ms after host submission;
sampling therefore covers only a few instants of each short operation. The
23.62/35.70-MiB successful observations and 33.93-MiB failed-check observation are
lower bounds on whole-VM transient rises. Control noise, sampling gaps, scheduling,
overlapping lifetimes and page accounting prevent an instantaneous upper bound.
Two kernel locks bound cooperating KDF concurrency; their 38-MiB aggregate array
allowance is not a whole-service memory bound. This trial's two successes do not
mean cumulative successes are capped at two; #54 observed four cumulative
successes under the same two-slot policy. Lifetime high-water RSS sums are not
simultaneous peaks. Fresh approved login/logout after slot release and hidden
input checks passed; Argon2 was neither weakened nor slowed for measurement.

## Decision, limits and reproduction

Continue shared ownership as an isolated measured candidate; retain the normal
eight-worker path. Compare trimmed workers and shared RPC/UI workers against the
same requirements if full UI/fairness recovery becomes too costly. Do not replace
base OpenSSH or accept a new crypto dependency from this memory result. Attribute
and reduce remaining per-transport costs, preserve all admission/approval/terminal
denials, close UI/operational parity and full workload/repeat/soak gates, then
rerun guarded 250 before changing defaults. Hardware tests gate later SBC claims.

Read [ADR 0015](../decisions/0015-shared-board-readiness-experiment.md),
[shared instructions](../../experiments/055-shared-board/README.md),
[collector instructions](../../experiments/056-credential-peaks/README.md) and
the [fresh native SSH fixture](0024-restricted-ssh-netbsd.md). Build normal workspace,
the standalone candidate and native collector/calibrator in a fresh signed guest.
Seed `profile_seed` through stdin with a generated private secret. Stop/reap all
backend work, run `reset_fixture.py --offline-disposable-fixture` as its owner,
then reboot before each model; do not delete live lock inodes. It only accepts the
marked fixed synthetic fixture and validates counts/integrity. Freeze sources and
binary identity before executing the new host drivers, keep private keys/logs
outside the checkout, restore limits and remove the guest afterwards. A 250 plan
is not permission to continue beyond a failed resource guard.

Saved checks (no VM rerun):

```sh
python3 tools/check_shared_evidence.py
python3 -m unittest discover -s tools -p 'test_*.py'
python3 tools/check_shared_dependencies.py
cargo clippy --manifest-path experiments/055-shared-board/Cargo.toml --all-targets --release --locked -- -D warnings
```

Source references retrieved 2026-10-20:
[NetBSD poll(2)](https://man.netbsd.org/poll.2),
[NetBSD nanosleep(2)](https://man.netbsd.org/nanosleep.2),
[OpenSSH sshd_config(5)](https://man.openbsd.org/sshd_config).
Runtime results above are our native observations, with the declared scope and
falsifiable adoption gates; no public service, blockchain action or external
publication of member data was performed.
