# Native NetBSD authority and TLS qualification

Software observations for [#274](https://github.com/gonz0w/faction-bbs/issues/274),
2026-10-28. The isolated [experiment](../../../experiments/270-provider-auth/README.md)
ran on NetBSD 11.0 evbarm/aarch64 with native Rust/Cargo 1.96.0 and base OpenSSL
3.5.7. It remains outside the product workspace and ordinary SSH/home-mail defaults.

## Observed result

All **23 tests passed serially and with default parallelism**: 12 authority tests,
the ten original TLS functions and one new TCP refusal-observer regression. The
harness reported 53.85 / 53.88 seconds; whole commands took 53.992 / 54.030 seconds.
The current store suite passed 18 tests; its one ignored subprocess-helper entry
is invoked by its parent. Native source/license/no-JS checks, ten policy tests,
formatting and Clippy passed. [Results](results.json) contains complete test names,
commands, timings, configured bounds and observed TLS stages.

Runtime used two CPUs and 1,054,990,336 observed bytes of physical memory. The
original dependency build used two CPUs, 2,128,732,160 bytes and at most two compiler
jobs. The observer correction reused those dependencies and compiled in the
1-GiB guest. These are distinct envelopes, not peak-memory or capacity measurements.
The native dependency graph selected 38 registry packages; the qualified
Linux/NetBSD/macOS union contained 39 existing locked packages. The empty guest
cache was populated offline from qualified existing sources; no package was installed
or upgraded. [Reproduction](REPRODUCE.md) describes the owned guest procedure.

Actual socket stalls observed approximately 5001 ms for handshake admission and
10035 ms for frame expiry in the serial run, with zero dispatch. Controlled late
post-BIO scheduling separately observed 5742 / 5717 ms before refusal; those cases
are deliberate delays, not verifier-performance measurements or hard real-time
bounds. Delayed TCP plus stalled TLS expired at 5003 ms using the original shared
five-second admission budget. The ten-second cycle, eight attempts, 8192-byte
encoded frames and existing store bounds remain. Filesystem deadlines are
cooperative and cannot preempt blocked I/O.

## Preserved failure and test-only correction

The original unchanged source failed natively: 12 authority passes and ten TLS
failures, all at the final post-drop reconnect assertion. Later variants inside
those functions were not all reached. Neither that run nor its parallel replay
established TLS acceptance. [History](history.json) preserves their exact results,
binary identity, diagnostic distinctions and genuine observer RED/GREEN.

A separate plain-TCP diagnostic observed a real live connection and byte transfer,
then apparent `connect_timeout` success after listener closure with immediate
`SO_ERROR = ECONNREFUSED` and no connected peer. NetBSD documents checking
`SO_ERROR` after writable connection completion. [NetBSD connect(2)](https://man.netbsd.org/connect.2)
Its `sowritable`/`sodopoll` sources allow write-readiness for a pending socket error;
upstream Rust 1.96.0 checks that error on the relevant path only when poll reports
`POLLERR` or `POLLHUP`. This is consistent with the observation; exact installed
kernel/std source equivalence and a syscall trace were not established.
[NetBSD socketvar.h, revision 1.171](https://github.com/NetBSD/src/blob/netbsd-11/sys/sys/socketvar.h#L462),
[NetBSD uipc_socket.c, revision 1.314](https://github.com/NetBSD/src/blob/netbsd-11/sys/kern/uipc_socket.c#L2427),
[Rust 1.96.0 Unix sockets](https://raw.githubusercontent.com/rust-lang/rust/1.96.0/library/std/src/sys/net/connection/socket/unix.rs)

The focused regression first executed the old predicate: live and retained-clone
controls passed, then the final-drop assertion failed. The corrected observer
keeps one 100-ms attempt and requires actual `ConnectionRefused`, directly or via
immediate `take_error()`. Live success, timeout, unrelated errors and failed error
queries do not establish refusal. No sleep, retry, assertion removal, deadline
increase or production TLS/authority/store change obtained GREEN.

## Source, identity and cleanup

The accepted base is `39bfd2c` plus one explicitly hashed test-only override,
**not unchanged base source**. [Inputs](inputs.json) binds all 35 selected inputs:
34 reuse the existing frozen source package, and the corrected test file is
[snapshotted here](sources/tls_tests.rs). The focused
[RED variant](sources/observer-red.rs) is retained separately. The original
[host package](../provider-authority/README.md) and its 22-test observations remain
unchanged. [Identities](identities.json) records the final ELF binary, tools,
headers, loader and resolved library files; these are observed identities, not a
hermetic build or whole-loader attestation. [Lead checks](lead-host.json) record
the corrected host replay separately. The manifest pins 47 references and the
exact selected-folder membership; its unit guard verifies saved bytes only.

[Teardown](teardown.json) records removed guest fixtures, stopped/reaped VM and
controller, closed admin forward, verified private retention and disposable-root
removal. All 86 saved owner assets stayed unchanged and six images remained
closed. Synthetic TLS keys were generated in memory and no private keys, raw
transcripts or real endpoints are published. The owner playground stays stopped.

This software result does not adopt a provider, production enrollment/recovery,
own-home delegation, OMEMO private-key possession, independent modern-client
interoperability, hardware support, a security audit or 250-user capacity. Those
original requirements remain under [#88](https://github.com/gonz0w/faction-bbs/issues/88).
Classic/default-off entry points and normal approval/resource limits are unchanged.
