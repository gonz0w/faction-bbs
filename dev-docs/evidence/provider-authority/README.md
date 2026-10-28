# Local attachment credential-boundary proof

Software observations for [#270](https://github.com/gonz0w/faction-bbs/issues/270)
and [#271](https://github.com/gonz0w/faction-bbs/issues/271), 2026-10-28.
The standalone Rust experiment is outside the product workspace. It exercises
synthetic credential bindings and real TLS 1.3 client-key possession before the
existing ciphertext store. It does not deploy an XMPP provider or change SSH,
classic/default-off mail, device-key ownership or normal service limits.

## Observed result

On macOS 27.2 / Darwin 27.2 arm64, Rust 1.96.0 and explicitly selected OpenSSL
3.6.3, all **22 tests passed: 12 authority/core and 10 TLS test functions**.
Each function can exercise multiple cases. The author test harness reported 31.53 seconds (32.81 seconds for the whole command);
lead serial and default-parallel replays both passed, as did exact CI-style
formatting and Clippy. Eighteen existing file-store tests passed separately;
one existing subprocess-helper entry is ignored and invoked by its parent.
[Results](results.json) records exact commands, timings, binary and linked-library
identities, source counts, fixed bounds and limitations.

The developer host has 18 CPUs and 128 GiB physical memory; compilation used two
jobs and fixtures allowed one active TLS connection. This is not a small-host
footprint or capacity measurement. Source qualification checked the exact selected
archive contents, license declarations and no-JS trees for Linux (39 packages),
NetBSD (38) and macOS (38): 39 unique existing locked registry packages, with no
root-lock update. Those source graphs do not establish NetBSD execution.

Actual TLS tests cover exact sender upload/commit and committed recipient read;
missing/wrong/expired/future client credentials; unknown or disabled authority;
wrong server CA/name/pin; malformed, oversized and trailing frames; actor-data
forgery; live account/device/peer and generation/reset changes; current leaf
expiry; quotas, exact retry, conflicting duplicate and lost-response recovery.
Wrong key and copied certificate without key fail client configuration, **not a
claimed TLS handshake**. Initial refusals enter authority dispatch zero times.
`Report.dispatched` counts authority entries, including an entry that denies.

Socket stalls observed approximately 5000 ms admission and 10006 ms frame expiry.
Controlled post-BIO delays separately reproduce scheduling after TLS success:
client/server completion beyond five seconds now denies with zero dispatch.
Delayed TCP plus TLS uses one original five-second admission budget. These
controlled delays are not certificate-verifier performance measurements. The
accepted-connection cycle remains ten seconds, at most eight attempted operations
and 8192 bytes per encoded frame. BIO calls recompute the original deadline;
filesystem checks remain cooperative and cannot preempt blocked I/O.

## Test-first history and preserved inputs

[TDD](tdd.json) distinguishes actual missing-module and behavioral RED runs from
setup, timing-observer and lint failures. It includes trust-store isolation and
late-admission defects reproduced before corrections. No assertion, product
crypto, store quota, normal budget or timeout was weakened. The original
stalled-frame test observer raced the client's separate ten-second clock; only
the independent observation socket became twelve seconds so the unchanged server
bound could be measured. All rejected variants and raw logs remain private.

[Inputs](inputs.json) binds 35 source/config/dependency inputs copied under
`sources/`; [manifest](manifest.json) pins this whole published package. These
are original BSD-3-Clause project sources, not vendor source or runtime keys.
The matching test binary is retained privately, not published. Fresh proof runs
generate new synthetic keys; this is source/runtime evidence, not a replay of
identical TLS secret material, a hermetic build or whole-loader attestation.

[Teardown](teardown.json) records empty owned fixture roots, joined handlers,
refused reconnects and unopened saved playground images. No provider, public
listener, package/service/security setting, VM or real member was operated.

## Reproduce and interpret

Follow the [experiment instructions](../../../experiments/270-provider-auth/README.md)
using the locked source cache, Rust 1.96.0, an existing OpenSSL 3 development
installation and a Cargo target directory outside the source tree. First run
`RUSTUP_TOOLCHAIN=1.96.0 python3 tools/check_provider_auth_dependencies.py`, then
`cargo +1.96.0 test --manifest-path experiments/270-provider-auth/Cargo.toml --release --locked --offline -- --test-threads=1`.
The exact proven environment/commands are in results; no install or provider
activation is implied. CI qualifies sources before formatting/lint/tests.

Registry state is in memory: lifetime digest tombstones are bounded at 32 and do
not survive a new Authority instance. TLS-key possession, an approved public
identity mapping, OMEMO identity-key possession and hardware attestation remain
different claims. This proof does not implement production enrollment/recovery,
own-home delegation, independent modern-client interoperability, a new NetBSD
runtime qualification, a security audit or 250-user support. Those original
requirements remain under [#88](https://github.com/gonz0w/faction-bbs/issues/88).
