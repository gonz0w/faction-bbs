# 0028: Tiny working sets for a busy independent board

Status: research deliverable complete; budgets and measurement program proposed.
Date: 2026-10-19. Issue: [#28 / BBS-028](https://github.com/gonz0w/faction-bbs/issues/28).
Owner: Codex research; founder/maintainer accepts decisions through review.
Decision affected: runtime (#4), storage (#9), operations (#18), experiments (#11/#20).
Stopping condition: workload/resource ledger, reproducible measurements, a bounded allocation probe, and technology hypotheses with explicit unknowns. No production stack selected. Use the [decision framework](decision-framework.md).

## Question, owner input and scope

How can 100–250 connected members fit comfortably on a small board without treating all host RAM as an application allowance? Owner answers on 2026-10-19 set **1–2 cores, about 1 GiB RAM**, primarily modern private networks with occasional poor connections. The founder explicitly prioritizes scalable low-memory languages, assembly if useful, and research into unusual technologies/public services/blockchain. The 640K-era reference expresses the desired efficiency; it is not a measured historic comparison or a literal modern encrypted-server ceiling.

The hardware/load envelopes are owner input. All numerical application budgets below are **researcher-proposed falsifiable targets**. They are not measured Faction performance, deployment instructions or acceptance of weaker security. An account count differs from active connections; an open connection differs from an active draft, password derivation, transfer or game process.

## Representative controls

| Control | Proposed reproducible workload |
| --- | --- |
| H1 primary | Named Linux SBC or VM, one core, 1 GiB host RAM; no swap for comparison. Record exact CPU, kernel, architecture, storage/media/filesystem, power/governor, memory availability and unrelated services. One VM vCPU does not emulate SBC speed. |
| H2 sensitivity | Same RAM, two cores; same storage and workload. Test actual ARM64 SBC and x86-64 VM separately before portability/support claims. |
| C0 empty | Started, zero connections, five-minute warm-up then ten-minute sample. Include cold startup separately. |
| C100 normal | 100 connected approved members, 10 active; each active member one action per five seconds = 2 actions/s. |
| C250 busy | 250 connected approved members, 25 active; same schedule = 5 actions/s. |
| C250 chat | 250 recipients, one 256-byte chat record/s, independent from action mix for fan-out accounting. Stress 32 records/s for 60 seconds; one P2 slow reader, then stalled readers. |
| Active action mix | Repeat a deterministic 100-action cycle: 60 room reads, 10 1-KiB-body posts, 20 chat sends, 10 metadata listings. Evenly phase members; payloads/visibility permissions fixed across candidates. Do not accidentally superimpose the separate chat workload unless labeled. |
| Seed data | 1,000 synthetic accounts, 50 rooms, 100,000 posts (1,024-byte body + 256-byte logical metadata), 1,000,000 retained chat records at 256 logical bytes. Retention is a test input, not a product default. Files use an explicit separate quota. |
| Bursts/failures | 25 reconnect attempts over 10 seconds, capped concurrent password work; 10 simultaneous uploads plus slow readers; storage-full/read-only, process crash at write/ack boundaries, denied/revoked/pending user, malformed/oversized terminal input; optional two door sessions. |

Action-cycle bytes and timestamps must be published with each candidate benchmark; these controls alone are not an implemented load generator. Use the [#8 tasks/caps](0008-interaction-budget.md) and #3 journeys. Run baseline with portal, federation, optional adapters and doors **disabled**. Then add each independently so its overhead is visible.

## Proposed resource targets and ledger

MiB = 1,048,576 bytes. **Application charge** includes every mandatory server, access/crypto helper, database/cache, worker, and descendant process; not just the main executable. Under the Linux reference measurement, use one enclosing cgroup, including its charged file cache/socket memory. OS baseline and total host consumption are reported separately. Do not subtract helpers or shared caches to improve a headline.

| Scenario/metric | Initial comparison target | Rationale / status |
| --- | --- | --- |
| C0 warm application memory | <=8 MiB | Aggressive tiny-service target; unknown until secure access/storage prototype. Stretch: <=1 MiB standalone core, separately labeled and never substituted for complete charge. |
| C250 connected/quiescent memory | <=32 MiB | Forces per-session costs into view; target unknown, not 128 MiB of automatic session buffers. |
| C250 active steady memory | <=64 MiB | Includes mandatory processes/cache; stretch <=16 MiB if full required behavior/security remain. |
| Burst application peak | <=256 MiB | Separate bounded authentication/storage transient envelope. Memory-hard credential work must be sized securely, then admission-limited; never lower its security to claim the idle budget. |
| Whole host memory | <=768 MiB working use, report available/cache/swap separately | Leave headroom inside 1 GiB; actual OS costs unknown. Application and host accounting overlap, never add them blindly. |
| CPU C0 / C250 idle / active | <=1% / 3% / 50% of **one core** averaged over sample | Idle wakeups, per-connection timers and compute efficiency; record burst p95/max and throttling separately. No existing throughput result. |
| Server action latency | p95 <=50 ms reads/chat/listing, <=100 ms durable post | Excludes client link and user think time; measure permission checks, storage commit and queueing explicitly. Do not acknowledge durable success before defined commit. |
| Text task end-to-end | p95 <=1 second P2 ordinary page | See #8 payload/RTT limits; security handshake and initial approval are separate measurements. |
| Startup/shutdown | Ready <=2 seconds; clean shutdown <=5 seconds | Fixed seeded data, cold/warm separately; record recovery time after crash separately. No pretending recovery was clean shutdown. |
| Connections | 250 steady, 100 lower-load control | 251st arrival must be bounded/rejected politely or admitted within a separately defined reserve. Hold unauthenticated sessions under a smaller independent cap (test 16), rate-limit expensive work and reserve sysop recovery access. No unlimited sockets. |
| Required deployment components | Aim <=2 independently managed services, zero mandatory external services | Count access gateway, broker/database and schedulers if required, regardless of packaging. Report process/thread count and update/backup steps separately; one binary can still spawn expensive workers. |
| Optional door delta | Two isolated sessions, initial experimental cap <=128 MiB each | Host/application peak accounting includes enabled doors and security helpers. Per-title measured budgets may differ; exceeding this experiment profile means unsupported at that profile, not permission to remove isolation. |

The steady **payload ledger** is a starting decomposition, not an RSS forecast:

| Allocation hypothesis | Upper bound at 250 connected / 25 active |
| --- | ---: |
| Session control state, proposed 1 KiB each | 256,000 bytes |
| Input accumulator, 1 KiB each | 256,000 bytes |
| Queued output liability, 8 KiB each | 2,048,000 bytes |
| Drafts only for 25 active editors, 16 KiB each | 409,600 bytes |
| Shared content/render scratch pool, globally capped proposal | 2,097,152 bytes |
| Sum of these named pools (not total memory) | 5,066,752 bytes (~4.83 MiB) |
| Code/runtime, allocator slack, stacks, crypto, OS buffers, indexes/page cache, workers | **U**; measure and fit into complete-charge targets above |

A reference retains real payload; charge it to each session's queue liability **and** count physical shared storage once in global accounting. This intentionally double bounds retention without double counting measured total memory. Bound queue age and reference lifetime so one slow reader cannot retain old pools forever. Consider event-driven I/O/pooled work, explicit arenas, streaming files, fixed-size handles and paged results before instruction-level optimization. These are hypotheses; no concurrency model or allocator is selected.

At 250 recipients * 256 bytes * 1 record/s, fan-out is **512,000 application bit/s** aggregate. At 32 records/s it becomes **16,384,000 bit/s**, while each client needs 65,536 bit/s, already beyond P2's 64,000 before framing. A faster language cannot eliminate that bandwidth deficit. Test host uplinks of 10 and 100 Mbit/s separately; use bounded queues and explicit catch-up/gap policy, not growing buffers. This arithmetic is independent of the measured allocation probe.

### Storage growth and operational space

Logical seed bytes: posts 128,000,000 + chat 256,000,000 + accounts (1,000*4,096) 4,096,000 + rooms (50*2,048) 102,400 = **388,198,400 bytes (~370.21 MiB)**. These synthetic record sizes omit indexes, tombstones, alignment, WAL/journals and filesystem costs. Test amplification factors **2/3/5** (assumptions, not engine measurements): roughly 740/1,111/1,851 MiB before files/backups.

C250 action mix for six active hours/day yields 10,800 posts and 21,600 chat records/day: **19,353,600 logical bytes/day** from those records, before metadata changes/amplification/deletion. Measure bytes written and allocated on disk, not just file length. Retention, pruning and search indexes are local policy decisions in #9/#19; do not set a hidden default by using this workload.

For an initial storage-pressure experiment use a **16 GiB BBS data volume**, 8 GiB authorized file quota, 1 GiB journal/temp allowance, 256 MiB bounded in-flight upload space and 64 MiB rotated operational logs without private bodies. The remaining space covers the seeded DB, growth, overhead and reserve; it is finite, not a support claim. Ten uploads share the temporary limit; quotas must include staging and prevent disk exhaustion. Backup/restore staging requires separately provisioned capacity and a verified restore; keeping it on the same full/failing volume is not recovery. OS/compiler images are outside this data-volume figure and must still be counted in deployment storage.

## Actual bounded experiment: ownership before assembly

Question: how much requested live allocation differs when delivering identical authored queued data through independent copies versus one immutable pool and per-session references. Predeclared workload in [probe.c](../../experiments/028-session-buffers/probe.c): 250 sessions, 256 records of 256 bytes per session; success requires allocation success, all payloads/references touched, equal delivery checksum and repeated same allocation count. This deliberately uses 64 KiB queued **per session**, exceeding #8's proposed 8 KiB cap; it is a stress counterexample, not the chosen queue design.

```sh
python3 tools/measure_buffer_probe.py --run
python3 tools/measure_buffer_probe.py --check
```

The tool compiles with `cc -std=c11 -O2 -Wall -Wextra -Werror` in a temporary directory, runs five fresh processes per strategy, then removes the binary. [Raw observations](results/028-buffer-probe.json) record source SHA-256s, compiler version, OS/architecture, pointer size, requested allocation bytes and checksum, without local paths or hostnames. This session used Apple clang 21.0.0 / Darwin arm64, 8-byte pointers; it was **not H1/H2**.

| Strategy | Requested live allocation | Equal logical delivery checksum | Observations |
| --- | ---: | ---: | --- |
| Per-session copies + pointer table | 16,386,000 bytes (~15.63 MiB) | 2,047,293,750 | Five matching successful runs |
| Shared immutable 64 KiB pool + 64,000 references | 577,536 bytes (564 KiB) | 2,047,293,750 | Five matching successful runs |

The second strategy requests ~28.37x less memory in this workload. **This is counted, successful allocation, not RSS/peak/process memory.** Allocator metadata, runtime/code/stacks, authentication, server/session structures, storage, sockets and real reference lifetime costs are excluded. No language comparison, CPU-speed result, 250 live connections, secure networking or slow-reader recovery was measured. Failures for allocation exhaustion, mutable content, cross-member authorization and reference release still need a representative server. Shared data must never bypass per-recipient visibility decisions. C is a disposable probe language. The owner later selected Rust/primary NetBSD in [ADR 0006](../decisions/0006-rust-and-primary-netbsd.md); this allocation probe does not validate the complete application budget.

## Measurement contract for candidate prototypes

For each candidate, freeze source revision, compiler/dependency versions/build flags, hardware/OS/storage, negotiated terminal/crypto, policy/seed/workload, and excluded components. Publish sanitized raw observations and exact commands. Equivalent security, durability and task semantics are mandatory. Warm five minutes, sample ten minutes, repeat five fresh runs; record cold boot separately. Extend to 24-hour soak for leaks after functional correctness. Each candidate must supply an executable workload harness and metric collector before ranking; **those full-server harnesses are not implemented here**.

| Measurement | Reproducible procedure and failure oracle |
| --- | --- |
| Resource accounting | Enclose all mandatory processes in one Linux cgroup. Sample `memory.current`, `memory.stat`, `memory.swap.current`, `cpu.stat`, process/thread/FD counts each second; capture `memory.peak`, OOM/events, I/O counters and whole-host available memory. Record RSS/PSS as additional diagnostics, never compare RSS to cgroup total as if identical. CPU%=100*delta usage_usec/delta wall_usec, relative to one core. Separate driver/monitor overhead outside measured group. |
| Room read | Seed same visible records. Read 2 KiB pages at C100/C250, measure permission-to-enqueue and client completion p50/p95/p99, request/output/wire bytes. Pending/revoked readers see no data; large records continue without cursor loss. |
| Post/persistence | Submit 1 KiB posts with defined durable commit semantics; time validation/commit/ack separately. Kill at before-commit/after-commit/before-ack checkpoints, restart and reconcile. Fail on duplicate acknowledged request, lost durable post, exposed unapproved data or success reported on full/read-only disk. Specify flush guarantees and media before comparing engines. |
| Chat/backpressure | One record/s then 32/s to 250 recipients; include P2/stalled readers and 10-second interruption. Record per-recipient gap/latency, queues, global retained bytes, stalled peers and memory return. Fail on unbounded retention, unauthorized cross-room sharing or silent complete-delivery claims. |
| Rendering/session | Render identical #8 content at 80x24/40x20; measure CPU/allocations/bytes and round trips. Replay through two named real terminal clients with plain/minimal ANSI, malformed controls/Unicode, resize and art skip. Follow #3 checkpoints; revoked resume fails closed. |
| Login burst | Measure cryptographic handshake and credential checks at an explicitly justified security setting; record per-worker peak/latency and admission backlog. Failed/slow clients cannot monopolize memory or bypass approval. No weakened KDF/crypto to win the benchmark. |
| Files/search/storage | Stream 1 MiB and quota-sized files under concurrency, test temporary-space exhaustion; measure DB/index/WAL growth for seeded and daily data, compaction, pruning, backup/export and restore. Confirm private contents are absent from logs. |
| Optional doors | Repeat baseline with 0/1/2 isolated per-title sessions; measure total CPU/memory/process/FD/egress deltas, startup/return latency. Kill/hang door; enforce CPU/RAM/filesystem/network limits and restore menu. No credentials/content leak or surviving child tree. No title tested in this report. |
| Operations/recovery | Count required services, packages, config/listeners, privileges, updates and backup steps. Disable HTTP/public services/adapters, interrupt upstream availability and restart host. Core local journeys must remain useful. |

The kernel documents cgroup accounting of anonymous/file-cache/kernel/socket memory, hierarchical `memory.current`/`memory.peak`, `memory.high` throttling versus `memory.max` enforcement and CPU microsecond counters. Thus whole-group accounting is more relevant than one executable's headline RSS. [Linux cgroup v2 primary reference, Memory/CPU interfaces, retrieved 2026-10-19](https://docs.kernel.org/admin-guide/cgroup-v2.html). Kernel/version/capability differences must be recorded; this reference method does not mandate Linux as Faction's runtime OS. No cgroups were configured in this session.

## Compact technology candidates and tradeoffs

These are documented mechanisms plus research hypotheses, **not benchmark rankings or audited dependencies**. All sources retrieved 2026-10-19; moving docs must be pinned/rechecked in #4. Keep ordinary hosted implementations in the comparison so specialization earns its complexity.

| Option | Relevant primary evidence | Faction hypothesis, cost and falsification |
| --- | --- | --- |
| C with bounded pools/event I/O | [WG14 N1570 draft, 7.22.3 memory management](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf); explicit allocation/free interfaces. Probe above uses a small subset, not a C standard-selection ADR. | Direct ownership can reduce payload copies; manual lifetime/bounds safety, fuzzing and review remain major costs. Measure complete secure server; low requested buffers alone cannot settle safety or dependency fit. |
| Rust hosted, optionally `no_std` core | [Rust Book ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) describes compiler ownership checks without garbage collection; [Embedded Book no_std](https://doc.rust-lang.org/embedded-book/intro/no-std.html) separates platform-independent core from OS integration. | Compare ordinary `std` first, then bounded allocators/core separation if needed. `no_std` does not supply a filesystem, secure network or durability. Unsafe/FFI dependencies and real working set remain U. |
| Zig explicit allocators | [Zig 0.15.2 language reference, Memory/Choosing an Allocator/Lifetime](https://ziglang.org/documentation/0.15.2/) documents allocator parameters and fixed buffers. This is a named research version, not a claim of newest release. | Allocation failure paths are exposed and budgetable; pointer lifetimes remain programmer responsibility. Measure build/toolchain churn, library/security maintenance and same server workload before adoption. |
| Free Pascal | [FPC programmer guide 7.3, generated 2025-07-18](https://www.freepascal.org/docs-html/current/prog/progse30.html) documents smart linking to omit unused procedures. | Worth testing given BBS heritage and compact native code. Smaller linked binary does not establish active RAM, modern secure-I/O fit or safe extensions. Measure bounds/overflow handling and deployment dependencies. |
| Selective assembly | [GCC Extended Asm, operands/clobbers](https://gcc.gnu.org/onlinedocs/gcc/Extended-Asm.html) documents compiler integration requirements. | Reserve for a profiled hotspot or demonstrated size limit after compiler optimization. Require equivalent portable reference, fuzz/differential tests and ARM64/x86-64 ABI review. Revert if whole-workload gains disappear or safety/maintenance costs dominate. No custom crypto or assembly rewrite justified by present evidence. |
| Specialized libOS/unikernel | [Kuenzer et al., Unikraft, EuroSys 2021 / arXiv v1](https://arxiv.org/abs/2104.12721) investigates composable OS libraries and reports compact images/RAM on its applications. | Interesting whole-system footprint experiment after hosted baseline. Paper results are not Faction/SBC numbers. Measure **host + VMM + guest**, update/debug/backup path, hardware drivers, network/privacy and door isolation; image size alone cannot win. Full runtime/dependency no-JS audit remains required. Project architecture docs could not be opened in this session; use the primary paper and verify current interfaces later. |

### Blockchain, public storage and services

The [NISTIR 8202 overview (2018)](https://www.nist.gov/publications/blockchain-technology-overview) describes distributed, tamper-resistant ledgers and consensus models. **Inference:** consensus/history replication does not remove Faction's local rendering, authorization, active-session or storage work. A public ledger for accounts, private posts, approval or local chat fails G3/G5; mandatory remote consensus/availability fails G4. A permissioned ledger adds peers/operating obligations; no measured small-host benefit exists here. It is a comparison option only if a future shared-chat decision names a specific unsolved trust problem. No coin/token/economic system is proposed.

Optional encrypted backup/object storage, rendezvous/directory helpers, or public distribution of *explicitly publishable assets* may reduce local storage/egress or discovery work. These are roles, not a provider selection. They cannot replace local membership authority or turn off the board when a service is down. Evaluate outbound metadata, key ownership/restore, deletion/export, price/quotas, dependency provenance and a working local fallback in #18/#19/#27. No remote data transfer or service setup was performed.

IPFS is a concrete content-addressed distribution candidate, not automatically private hosting: its [official privacy documentation](https://docs.ipfs.tech/concepts/privacy-and-encryption/) explains publicly visible content/provider metadata and the need for additional privacy measures. A private network or encrypted backup role remains a separate design/prototype, with no automatic permission to publish local BBS content or federation beyond shared chat. Hashes can still reveal matching content/metadata; immutable publication makes reversal/deletion harder. Public-service offload must demonstrate total small-host benefit at equivalent privacy and offline usefulness before adoption.

## Gates, criteria and evidence confidence

For the **local synthetic experiments**: G1 pass (stdlib Python + authored C, no added dependency), G2 unknown (no authorization server); G3/G4/G5/G6 pass in experiment scope (synthetic content, no external connection/service or telemetry); G7 unknown (no real terminal/session); G8 not applicable (no door/extension); G9 pass for our authored sources under repo BSD-3-Clause, not an audit of candidate stacks. For **each production candidate** above G1/G2/G5/G7/G8/G9 remain U pending complete dependency, security and integration validation; G3/G4/G6 are design obligations, not language properties. Public-ledger/private-data and mandatory-central-service designs fail the named gates regardless of resource scores. Other applicable gates remain U, not implied passes.

Use framework weights S25/O20/R15/T15/M10/P10/X5, with resource evidence prominent through mandatory published metrics and owner targets. Do not turn preference for tiny binaries into a security exception. For a later complete comparison define R anchors against these targets: 0 workload cannot complete/bounds fail; 1 completes but exceeds a proposed steady/peak target; 2 meets all declared resource targets; 3 also meets <=32 MiB active; 4 also meets <=16 MiB active with equal correctness/security. Higher anchors require measured complete charge, not isolated core memory. Record tradeoffs instead of collapsing CPU/latency regressions into the RAM score. Other criterion anchors must be defined before scoring their experiments; all are presently U. Vary R 15–30 and S 25–40 (renormalize) after comparable evidence; no present total/ranking is defensible.

| Claim | Evidence kind | Confidence and limit |
| --- | --- | --- |
| 1–2 core / ~1 GiB host, 100–250 connected, tiny-memory priority | Owner conversation, 2026-10-19 | High intent, no technical feasibility proof. |
| Buffer ownership changes requested allocation by ~28.37x | Measured C source + raw 10 process results above | High for fixed workload, low for server RAM/throughput extrapolation. |
| Row/byte-bounded text keeps minimal ANSI overhead small | [#8 measured replay](0008-replay-results.md) | High fixture accounting; real network/terminal behavior unknown. |
| Proposed budgets achievable with required crypto/durability/250 users | Unknown | Low; full prototype and H1/H2 measurements required. |
| Compact mechanisms exist in candidate languages/libOS | Primary sources in option table | Moderate relevance; not measured Faction integration, license/dependency/security audit incomplete. |
| External ledger/service improves small-host resources at equal privacy | Inference/unknown | Low; no such benefit measured, and several forms violate gates. |

## Conclusion, reversal and next work

**Propose** the budgets and measurement contract as comparison inputs; prioritize ownership/bounded work before instruction-level tuning. Keep C/Rust/Zig/Free Pascal, selective assembly and a specialized-OS experiment in the #4 evaluation, without naming a winner. The weakest critical unknown is complete secure working set/latency on H1 with 250 connections, required storage and failure handling. Record failed targets openly and revise through review if evidence justifies it; do not claim a target met from these micro-experiments.

Reversal is low for these fixtures/targets, moderate for a language/runtime/server layout, and high for persistent formats or irreversible external publication. #4 should implement equivalent tiny slices (approved read/post/chat, secure access and selected durable write contract) in bounded candidates; #9 compares storage growth/restore at the same semantics. #11/#20 own full workload/failure harnesses, #18 deploy/measure total operational costs, #19 security/privacy, #34 door evidence. Merge of this report accepts research inputs only; production language, OS, database, protocol, blockchain or provider adoption requires separate reviewed evidence/ADR.
