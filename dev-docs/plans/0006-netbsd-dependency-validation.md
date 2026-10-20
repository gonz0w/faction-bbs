# 0006: Verify the chosen Rust/NetBSD direction

Issues: #4/#9/#18/#23/#31; follows PR #39, which remains dependent on #38.
Status: complete research deliverable; PR #40 open for review. Date: 2026-10-19.

## Owner decision and outcome

The owner explicitly chose Rust and NetBSD as the primary target on 2026-10-19.
Record that acceptance without treating prior unvalidated backend/hardware claims as
passed. Verify a concrete SSH, SQLite and readiness-networking path on NetBSD, with
dated primary sources, exact versions/licenses/dependencies and executable evidence.
Do not promise every NetBSD architecture or adopt a JavaScript dependency.

## Predeclared validation

Attempt a disposable local NetBSD 11.0 ARM64 VM (same CPU family as the host and SBC
interest), 1 GiB RAM / one or two vCPUs, official image/kernel with SHA-512 integrity
checks against the release manifest. Use private temp disk/cache/keys, no public
listeners or real member data. Any host forwarding binds loopback explicitly.
No bridged network, host directory sharing, or persistent public service.

1. Compile/run Rust std and pinned Mio readiness sockets on NetBSD; test TCP/Unix
   connections, bounded synthetic command input, pending/revoked flag checks and a
   stalled peer. This remains a synthetic adapter probe, not authentication evidence.
2. Compile/run an explicit Rust/SQLite binding candidate with FTS5, WAL/FULL, foreign
   keys, parameter-bound reads/posts, bounded writer contention and export/restore
   tests. Record libc/native library versions and selected dependency source/license
   inventory. A Python-only DB test does not verify a Rust binding.
3. Test the shipped OpenSSH restricted-command gateway: key auth, forced BBS probe,
   PTY, shell/exec/subsystem/forwarding escape denial, unapproved/revoked synthetic
   actions and cleanup. Distinguish transport confinement from real account approval.
   Compare embedded Rust SSH feasibility/dependency costs before selecting a backend.

Record pass/fail/unavailable by architecture/VM/revision. Use measured results only for
compatibility here: emulation/VM overhead is not an SBC capacity result. Keep exact
commands and sanitized evidence in an issue-scoped experiment; never commit keys,
actual endpoints, raw login logs, disk images, hostnames or generated credentials.
Stop/clean up the VM and forwards after validation; no deployment is authorized.

## Progress

- [x] Read current requirements, issues and pending PR state; start a dependent branch.
- [x] Retrieve/verify official VM inputs and boot the bounded local guest. Signed global manifest matched image/kernel; HVF vmt probe required a documented userconf workaround.
- [x] Run dependency/build/runtime and negative gateway probes, record limitations. Both native DB variants passed; nine SSH oracles passed. Base SQLite shadowing was found and corrected with explicit pkgsrc runtime path. Embedded SSH remains a manifest-only comparison, explicitly unverified.
- [x] Update accepted direction, research/ADRs/docs and checks. Native bundled/pkgsrc runs and all nine SSH oracles passed; 14 selected crate sources passed the reproducible policy scan. All repository/provenance checks and 23 existing tests passed. GitHub evidence added to #4/#9/#18/#23/#31; publication tracked below.

## Cleanup and remaining work

Guest cleanly shut down; QEMU stopped, loopback forward verified closed, disposable
disks and temporary SSH credentials removed. No public service remains.

The embedded russh comparator received manifest review only; full transitive audit,
native build and equivalent resource/security tests remain open. No real account
registration/mapping, private-overlay/firewall test, physical SBC test or complete
100/250-member load test was done. These are explicit follow-ups in the report,
not reasons to reinterpret the narrow passes as production readiness.

[PR #40](https://github.com/gonz0w/faction-bbs/pull/40) targets the PR #39 branch
while #38/#39 remain open; no implicit merges. Initial Documentation and policy
CI passed: [run](https://github.com/gonz0w/faction-bbs/actions/runs/39356382214).

Next work is the report’s real-identity/local approval slice and complete-session
cost comparison, followed by named physical SBC and appliance update/restore tests.
