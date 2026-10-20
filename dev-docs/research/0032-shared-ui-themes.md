# Full shared UI and ANSI themes within the small-host envelope

Status: completed bounded implementation and software-VM measurement; adoption deferred.
Date: 2026-10-20. Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55),
related #20/#28. Decision: [ADR 0018](../decisions/0018-shared-ui-themes.md).
Owner: project maintainer; implementation/evidence subject to PR review.
Stopping condition: runnable isolated full-UI candidate, actual native oracles and
three matched fresh-boot measurements or a preserved contrary result.

## Question, gates and alternatives

Can the shared owner retain the existing member UI and sysop themes without
losing the #60 250-member headroom? Existing root crates and frozen experiments
remain byte-identical; a unique standalone package avoids stale same-name outputs
and keeps all historical evidence checkers truthful. No database/schema migration,
new dependencies, public service, weakened password costs or JS/TS are introduced.

| Alternative | Gate/result | Consequence |
| --- | --- | --- |
| Keep normal per-session workers | Existing functionality passes; prior 1-GiB study stopped at 168 | Remains default eight-worker development path; no 250 claim |
| Ship the small #58 probe UI | Fails member/theme parity | Preserve its measurements; extend separately |
| Share a full nonblocking Rust UI using existing core/terminal libraries | Native functional gates pass; memory measured below | Selected bounded experiment; no normal-service adoption |
| Thread/process the blocking console for every member | Reintroduces stacks/process costs; U at 250 | Defer; retaining the console is the rollback |
| Replace base SSH or weaken Argon2 | Outside authorized direction/security gate | Rejected for this increment |

No numeric popularity score is appropriate: UI/permission gates are mandatory,
and small-host accounting must be measured. Native software VM is the evidence
target; physical SBC absence does not block software building or establish a
hardware support claim.

## Implementation and measured protocol

See [candidate bounds/setup](../../experiments/055-shared-ui/README.md). The
shared owner drives explicit prompt states through the existing pure menu decoder,
theme/art validation and core APIs. One global immutable theme/season snapshot
serves all clients. Lists page ten rows; post/read/retry state is connection-owned;
read cursors update after TTY output drains. Chat leases and request sequences
remain connection-bound. Role/approval/epoch/grant decisions stay in the core.

Rotating sweeps permit 16 input/geometry/background work steps. Each terminal
reads up to 64 bytes every 20 ms and submits one action; movement and disabled
hotkeys do no SQLite work. Output starts at 1 KiB and can grow to 32 KiB including
CRLF and safe art, with the existing five-second nonempty-batch deadline. No
database transaction waits for terminal output. This bounds application work,
not SQLite lock latency or OS scheduling. Two credential permits retain their
queued/executing/unconsumed-result lifetime and the same two kernel slots.

The NetBSD 11 ARM64 QEMU/HVF VM has two vCPUs / configured 1 GiB and no swap.
Rust 1.96 and the same reviewed SQLite/SSH dependencies are used. Reuse the #60
experimental compact jemalloc and pageable-NTP policy, retaining every base
service. It remains a guest-wide research input with exact rollback, not an
installer recommendation or a timekeeping precision result.

Synthetic seed: 1000 distinct approved member accounts plus one sysop, each with
an independently salted Argon2 record; 50 rooms and 100000 1-KiB posts. Short
stages are 32/100/200/250 distinct real SSH logins with one-second settling,
roughly ten-second menu/chat observations and 110-ms paced admissions. The fixed
experimental cap/refill stays 250 / 100 ms; normal defaults stay eight / five
seconds. Full dedicated OpenSSH/frontend costs are included. Role RSS sums repeat
shared pages and are not USS/PSS; whole-VM counters include helper/admin overhead.

The guard is unchanged: free + max(0, clean file - wired) >= 128 MiB, zero swap,
64 spare process slots and 128 spare descriptors, with strict process/file
inequalities. Admission is checked between batches of eight; this is not a
continuously enforced reserve. All actual samples/decisions are retained.

At 200/250, replace two members using two wrong and two real passwords while the
other members remain in chat. At 250, one member lists rooms, posts durably, reads
a selected room and rejoins live chat while 249 peers stay there; a persistent
native stream samples this full-UI work. Two fresh owner consoles then use real
credentials with all 250 members in chat, followed by loaded known 0/19/38-MiB
touched-page controls. These controls are not cold Argon2 upper bounds. Successful
password counts do not prove two derivations overlapped. All sampled peaks are
lower bounds; requested 10-ms sampling has actual timing/query cost in the ledger.

## Actual results and confidence

The final candidate passes **27 native UI/permission/fault oracles** through actual SSH sessions and direct
guest descriptor/PTY probes: original
CP437 identity and ASCII fallback, custom reordered/hotkey menu, manual winter
palette/welcome, room paging/selected-room UTF-8 post/replies/unread, exact retry
after a real write lock, shared sysop approval/revocation/room/member controls,
private deny/regrant/kick without draft redirection, live draft preservation,
plain editing, resize/recovery, responsive cursor cleanup, busy-peer service,
forged control rejection, slow-reader deadline, frontend/backend crash, integrity
and fresh login. The functional tests run with the selected allocator/time policy.

Normal native application tests (58), local PTY/credential/lifecycle checks,
20 actual SSH oracles and 13 lifecycle/load/failure oracles pass; the unchanged
analyzed minimal frontend also passes 20 direct contract checks. The candidate
passes its deterministic two-permit lifetime regression (one native test), host
pinned fmt/Clippy/test/build and the existing 39-NetBSD/41-Linux reviewed registry
package graph without new features or JS/TS sources.

Three fresh-boot **250-member** runs pass every sampled guard. Completed-stage
headroom is **180.39–180.51 MiB**, with **135.09 MiB** lowest sampled stress
headroom under the unchanged 128-MiB predicate. All trials finish the full UI,
credential, calibrated-control, fresh-login and bounded cleanup checks.

| Fresh boot | Final menu headroom MiB | Lowest sampled stress MiB | Full-UI stream minimum MiB | Fresh-owner minimum MiB | Stop/reap seconds |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 180.39 | 139.80 | 181.77 | 154.18 | 1.435 |
| 2 | 180.51 | 135.09 | 181.68 | 135.09 | 1.221 |
| 3 | 180.44 | 135.27 | 182.08 | 135.27 | 1.288 |

The shared owner's final-menu RSS is 7.26–7.28 MiB,
but this excludes mandatory frontend/SSH costs; it does not accept the complete
application budget. Every 250-member steady observation includes 250 frontends,
one shared owner and at least 501 dedicated SSH processes. RSS repeats shared
pages. The final margin uses the completed-stage guard after all members return
to menus, not the earlier chat-idle sample.

See the [complete sanitized ledger](results/033-netbsd-shared-ui.json). All native
copy hashes match the executed repository inputs; all three trials use the same
installed binary as the final 27-oracle suite. All known-page controls are retained;
exact original NTP configuration and default allocator were restored and read back
after reboot. The private VM, keys, generated credentials, disks, raw logs and
forwards were removed. No normal eight-worker default was changed.


Test-development corrections: fixture room slugs must use legal hyphens; custom
menu ordering is actually rearranged rather than only relabeled. NetBSD PTY
hangup may precede control EOF, so the probe verifies both. Its total twelve-second
allowance includes filling the PTY under rate limiting; the application output
deadline remains five seconds. No failed measured cohort is omitted or relabeled.

## Reproduction, reversal and remaining gates

The candidate README gives build/fixture/driver commands. Preserve separate target
directories and installed binary hashes, full persistent sample matrices, native
copy readback hashes, all limits, known-page controls and exact cleanup. Validate saved
evidence with `python3 tools/check_ui_evidence.py`; that does not rerun a VM.

Confidence is high for the listed tests on this NetBSD software fixture, limited
for other workloads/architectures and unknown for production capacity. No physical
SBC, sustained active mix, long soak, hostile authenticated-transport flood,
full application budget, network latency SLA or clock-precision support is claimed.

Keep normal services unchanged. Stopping the candidate and restarting the normal
eight-worker broker is a low-cost application rollback: the core schema and private
state are unchanged. Guest-wide allocator/NTP policy requires separate #20 scoped
setup/rollback/timekeeping review. #55 owns normal-service adoption/confinement;
#28 owns active workload/soak/resource/admission gates. Lua menus/isolated doors
remain the first extension after the useful core. No extra platform dependency
or mandatory blockchain/payment service was introduced.
