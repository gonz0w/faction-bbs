# Retrieval issuer and HTTP carrier evidence

This record covers issues [#311](https://github.com/gonz0w/faction-bbs/issues/311)
and [#312](https://github.com/gonz0w/faction-bbs/issues/312) at source commit
`e66ba011e7dfb63c9e04bb02dede238dd5806f64`. The library-only default-off issuer
and one-operation HTTP carrier are described in the [issuer guide](../../delegation-issuer.md).
[Inputs](inputs.json), [results](results.json), [TDD captures](tdd.json),
[tools](tools.json) and [binary identities](binaries.json) distinguish source,
execution, environments and limits. See [reproduction](REPRODUCE.md).

## Software results

All 14 provider profiles passed on the macOS host and NetBSD 11 ARM64; host
strict Clippy passed for each. Counts overlap across profiles and include
compile-fail doctests, so they are not independent totals.

| Provider feature profile | Host passes | NetBSD passes |
| --- | ---: | ---: |
| `blob-iq` | 58 | 58 |
| `default` | 49 | 49 |
| `delegation-issuer` | 112 | 112 |
| `delegation-issuer,durable-policy` | 147 | 147 |
| `delegation-records` | 66 | 66 |
| `delegation-records,durable-policy` | 87 | 87 |
| `delegation-services` | 73 | 73 |
| `delegation-services,delegation-records,durable-policy` | 120 | 120 |
| `delegation-services,durable-policy` | 101 | 101 |
| `delegation-wire` | 128 | 128 |
| `delegation-wire,durable-policy` | 163 | 163 |
| `durable-policy` | 68 | 68 |
| `operator` | 85 | 85 |
| `preparation` | 53 | 53 |

The standalone codec passed 23 checks on each platform. The selected source gate
covered Linux, NetBSD and macOS graphs before compilation; the root lockfile stayed
unchanged. The maintained tooling suite passed 641 checks. Host Rust/Cargo 1.96
used explicitly selected OpenSSL 3.6.3; NetBSD used base OpenSSL 3.5.7. NetBSD runtime
used two virtual CPUs and 1 GiB. The first 12 smaller provider profiles and codec
were built at 1 GiB; the broader `blob-iq` and `operator` dependency builds used
2 GiB, then their existing binaries ran at 1 GiB. This is not a claim that a fresh
1 GiB compile failed. Compile-fail rustdoc snippets still invoke rustc during
runtime test commands.

The native source map contains 388 exact files; 449 artifact files were verified.
The earlier `a9bd905` native source and 204-artifact archive were preserved before
corrected runs. Source maps, executable hashes, commands and raw logs bind each
result; see the JSON records. [Cleanup](cleanup.json) records the stopped VM,
closed owned forward, six closed owner images and 86 unchanged saved assets.
[Retention](retention.json) identifies verified private archives; none of their
credentials or raw private transcripts are published.

Fresh host regressions passed 259 device-provider client checks, with eight existing
ignored helper tests invoked by their parents, 50 default-client checks and strict
device-provider Clippy. Twenty real classic/Unicode console controls transferred
1 MiB per mode; the private fixture was removed and children reaped. This batch
did not run a fresh NetBSD production-client console journey.

## Test-first and interpretation

The [TDD catalog](tdd.json) separates observed behavioral failures from missing
interfaces, compiler, lint, fixture and oracle failures. The corrected home
association guard has actual behavioral RED captures 100 and 102 and GREEN 104
and 106. The initial orchestration helper version exists only in its creation
transcript; no original independent file/hash is claimed. A separate codec fixture
staging helper failed before Cargo because `chown` was absent from PATH; initial
subprocess stdout/stderr was not retained. Its original wrapper was preserved and
the path corrected. This was not a product RED. A later archive verifier initially
skipped 24 hardlink headers; the retained archive was then checked by resolving
every in-prefix target and verifying all 10,912 indexed file paths, sizes and
content digests before teardown. That was a verification-helper failure, not a
product RED or lost data.

Actual mutual TLS tests exercise the consuming HTTP flow. A private post-handshake
TLS response BIO gate controls scheduling for policy, cancellation and deadline
checks; it does not show kernel backpressure on a small TLS response. The separate
plaintext `DeadlineIo` fixture observes kernel write retry/cancellation. Precise
clock tests use synthetic observations. Device record checks retain whole-second
wall high-water marks; precise issuer rollback observations begin at the first
issuer check, including a failed attempt. Unobserved clock changes are not detected;
fixed monotonic record and token deadlines remain binding.

The carrier is TLS 1.3 only and a restricted Faction profile, not proof of RFC 7662
TLS 1.2 support or general OAuth conformance. These results do not establish a
remote Store grant, sender verifier, outbound callback, native relay, complete
cross-home delivery, deployment, independent interoperability, physical hardware
or supported capacity. Requested role mappings are recorded in [tools](tools.json);
native role discovery and backend model telemetry were unavailable or unverified.
