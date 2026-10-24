# 0004: Native core comparison and first validation choice

Later direction: [ADR 0006](../decisions/0006-rust-and-primary-netbsd.md) accepts Rust and primary NetBSD. [Native NetBSD evidence](0023-netbsd-dependencies.md) adds pinned dependency/gateway checks; earlier comparative observations below remain historical evidence, not contrary current authority.

Status: bounded comparative research complete; recommend Rust/readiness I/O for the next validation slice. Production stack not accepted.
Date: 2026-10-19. Issue: [#4](https://github.com/gonz0w/faction-bbs/issues/4).
Owner: Codex research; maintainer reviews the implementation recommendation.
Stopping condition: compare viable mechanisms and native probes, reject an observed bad approach, choose the next experiment with reopening conditions. [Decision framework](decision-framework.md).

## Decision to make

Carry **hosted Rust with readiness-based I/O** into the first integrated read/post/chat experiment. Keep **Go as the measured fallback** and C for a comparison/reference or narrowly justified compatibility boundary. Mio 1.2.4 is the tested readiness adapter, not a final framework commitment. Do not start with assembly, a custom OS port or a large mandatory distributed runtime. This is a positive recommendation for the next slice, not a production capacity/security claim or a complete closure of #4.

Owner priorities include tiny working sets, modern or older technologies on merit, independent hobby operation on a small SBC or VM, no JS/TS, and privacy/security. MINIX is a candidate, not a requirement (owner clarification 2026-10-19). The accepted SSH/private-network direction is [ADR 0004](../decisions/0004-ssh-and-optional-private-network.md); no SSH backend is selected here. No candidate wins by nostalgia, familiarity or being newest.

## Predeclared experiment and actual environment

[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/4) defined 250 simultaneous native local Unix-stream connections with bounded commands, synthetic approval/revocation, connection cap, oversized/fragmented input and slow-client progress before comparison. Source and [raw results](results/004-native-sessions.json) are reproducible with:

```sh
python3 tools/measure_native_sessions.py --run
python3 tools/measure_native_sessions.py --check
```

`--run` needs cc, rustc, cargo, go, ps and initial public crate access. It compiles in private disposable temp directories; sockets never listen on IP interfaces. Local driver raises only its own FD limit and terminates every child/removes binaries. This host was Darwin/arm64, clang 21.0.0, Rust 1.98.0 and Go 1.27.0. It was not a 1 GiB SBC or VM. Five fresh processes per candidate; each opens 250 connections and runs 500 sequential commands across 25 active connections (300 reads, 100 posts, 100 chat acknowledgements). These are **synthetic responses**: no real accounts, password checks, durable posts, fan-out, terminal rendering, TCP or SSH. A peer may issue `ALLOW` in this isolated harness; this would be an authentication bypass in a product. Never expose these probes as a service.

C and the first Rust implementation scan nonblocking sessions every millisecond. Go uses a blocking goroutine per connection backed by its networking runtime. The follow-up Rust+Mio uses readiness notification without periodic polling. Thus this compares **configured implementation approaches**, not language speed in isolation. The follow-up was triggered by the scan loops' measured idle CPU failure, not by changing a threshold to favor Rust. Preliminary runs were repeated after provenance/initial-pending checks and the readiness variant were added; saved final runs were sequential with no concurrent storage probe.

RSS is sampled using local `ps`, whose installed manual defines 1024-byte units. Three zero/connected samples and five active samples/run; the table uses median of each run's largest active sample. This is **sampled process RSS**, not peak RSS, PSS or the complete application/cgroup charge proposed by #28. CPU comes from coarse cumulative process time over one idle second; a reported zero means below that observation resolution, not zero work. No confidence intervals or host isolation. The driver is separate and excluded. Build options and source SHA-256s are in the runner/raw files; C is optimized with warnings as errors, Rust optimized/stripped with panic abort, Go stripped with CGO disabled.

## Results and contrary evidence

| Candidate | Binary bytes on this host | Median sampled active RSS | Median synthetic READ | Idle CPU observation |
| --- | ---: | ---: | ---: | --- |
| C, millisecond scan | 34,272 | 4,080 KiB | 1.207 ms | ~6.97% of one core |
| Rust std, millisecond scan | 360,032 | 4,304 KiB | 1.327 ms | ~5.98% of one core |
| Go, network runtime | 2,052,258 | 6,688 KiB | 0.051 ms | Below coarse resolution |
| Rust + Mio readiness | 360,912 | 4,288 KiB | 0.040 ms | Below coarse resolution |

All seven scoped oracles passed in five runs/candidate: initially pending flag denial, revoked flag denial, fragmented input, unknown command, 250-connection cap, oversized-input close and progress despite a non-reading peer. The probe does not establish real authentication/authorization, global fan-out retention, exact peak bounds or adversarial resistance. The slow-peer test verifies another connection's progress, not reliable chat delivery. No candidate's observed RSS proves #28's complete encrypted-server target met.

**Reject periodic millisecond scans for the initial server approach:** both exceeded the proposed 3% connected-idle target here. Readiness-based Rust removed that observed cost without a large sampled working-set increase. **Go remains competitive:** it was simpler to express and fast in this fixture, with no third-party networking package. Its larger binary/RSS here do not make it unsuitable or establish a GC latency problem. **C remains compact:** it had the smallest binary; manual input/ownership reasoning adds a substantial audit burden for a network-facing core. The experiment did not discover a measured benefit that requires assembly.

## Alternatives and primary evidence

Sources retrieved 2026-10-19. Documented mechanisms are not Faction integration proof.

| Candidate | Mechanism and source | Fit and remaining costs |
| --- | --- | --- |
| Rust hosted std + small readiness adapter | [Rust ownership chapter](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html): compiler-checked ownership. [Mio maintainer API docs](https://docs.rs/mio/latest/mio/): readiness registration/polling. The exact 1.2.4 source was obtained and compiled through the locked Cargo package. | New application parsing/state code forbids `unsafe`; dependency/FFI/crypto internals still need review. Good observed footprint without GC tuning; more type/lifetime and dependency-review work than the small Go probe. |
| Go std networking | [Go GC guide](https://go.dev/doc/gc-guide): memory/CPU and memory-limit tradeoffs. No runtime tuning used in probe. | Straightforward concurrency and good observed idle/response behavior. Need complete-heap/stack/GC/burst measurements; do not assume pauses or unacceptable RAM from GC's existence. |
| C/POSIX | The authored [C source](../../experiments/004-native-sessions/session.c) plus prior [allocation probe](0028-small-host-budgets.md); [WG14 N1570 memory management](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf). | Small binary and broad OS integration. Manual memory/bounds/concurrency mistakes matter on untrusted input. A safe application core may still need isolated C libraries or legacy hosts. |
| Zig | [0.16.0 language reference, Memory/Lifetime/Build Modes](https://ziglang.org/documentation/0.16.0/); installed compiler 0.16.0 observed. | Explicit allocators and native portability are promising. No equivalent networking/storage probe completed; smaller ecosystem/API changes and lifetime review remain unknown, not a numerical loss. |
| Free Pascal | [FPC smart-link guide 7.3](https://www.freepascal.org/docs-html/current/prog/progse30.html), generated 2025-07-18. | Native code/BBS heritage merits a bounded compatibility experiment. Compiler not installed here; no runtime/SSH/DB measurements. Nostalgia does not settle the new core. |
| Selective ASM / specialized OS | [Prior sourced options](0028-small-host-budgets.md) and [appliance/access research](0018-appliance-and-access-direction.md). | No hotspot requires assembly yet. A specialized OS changes networking/drivers/update/isolation obligations; measure total host cost separately rather than rewriting the core around it. |

## Dependency and gate screen

The Rust std probe and readiness application use `#![forbid(unsafe_code)]`. This does **not** prohibit unsafe code in std/Mio/libc or prove security. The experimental [Cargo manifest](../../experiments/004-native-sessions/Cargo.toml) pins Mio `=1.2.4`; the [lockfile](../../experiments/004-native-sessions/Cargo.lock) records package checksums, including platform-conditional packages. Measured Darwin dependency set: Mio 1.2.4 (MIT), libc 0.2.191 and log 0.4.34 (MIT OR Apache-2.0). Selected downloaded package files were scanned for JS/TS suffixes/manifests and none found. No package source/binary is vendored. Windows/WASI lock entries were not compiled/audited; no such platform support is claimed. This basic license/file scan is not a vulnerability or unsafe-code audit. See #31 before production dependency acceptance.

For these local probes: G1 pass for authored sources and selected build dependencies; G2 **unknown for product** (synthetic flags only); G3/G4/G5/G6 pass only for local synthetic experiment design (no external data/services, no domain addresses or private telemetry); G7 unknown (no full BBS/ANSI session); G8 not applicable (no doors/extensions); G9 pass for authored BSD sources and recorded selected package licenses, with production notices/dependency audits still required. Production applicable gates remain unknown until integrated tests. No score can override them.

Framework weights S25/O20/R15/T15/M10/P10/X5 remain reference priorities. Compare RSS, CPU, response time, build/dependency burden and safety evidence separately. Use U for complete security, required SSH/storage charge, hardware portability and doors; **no numeric overall winner**. Reuse #28's resource anchors only on equivalent complete-server measurements. Sensitivity: greater maintainability/stdlib weight strengthens Go; mandatory MINIX would reopen Rust portability; a measured hard binary/working-set limit could strengthen C/Zig/Pascal. Under present priorities Rust is a justified next experiment, not a universal result.

## Recommendation, confidence and reversal

Proceed with **Rust/readiness + SQLite** in an integrated validation branch, with transport/session/domain/storage boundaries kept explicit. Let one bounded writer own database mutations so blocking persistence cannot stall networking; read pools/caches need measured caps. This worker shape is proposed, not implemented here. Keep the server usable without portal, overlay network, federation or doors. Do not adopt an async framework, embedded SSH package, scripting runtime or `no_std` requirement from this probe.

Confidence: high for exact probe results/oracles; moderate for choosing Rust as the next slice; low for full encrypted BBS memory/latency on the target SBC. Reversal is low now (disposable probes), moderate after core APIs, high after deployment/persistent formats. [ADR 0005](../decisions/0005-rust-sqlite-validation-slice.md) defines exit conditions. #4 remains open until equivalent integrated access/storage/security/deployment evidence supports a production choice. If those conditions fail, compare Go at the same semantics rather than optimizing away required protection.
