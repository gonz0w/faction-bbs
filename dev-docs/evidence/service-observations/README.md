# Registered-service observations and durable policy evidence

Issues [#308](https://github.com/gonz0w/faction-bbs/issues/308) and
[#309](https://github.com/gonz0w/faction-bbs/issues/309) add default-off real mTLS
service sessions, exact recipient-policy observations and restart-safe public
registrations. These are local library primitives; they grant no device or store
authority and do not implement an issuer or relay. See the
[API and upgrade contract](../../service-observations.md).

## Observed checks — 2026-10-30

| Provider selection | Host | NetBSD |
| --- | ---: | ---: |
| Default | 49 | 49 |
| `blob-iq` | 58 | 58 |
| `preparation` | 53 | Not separately run |
| `operator` | 84 | 84 |
| `delegation-records` | 66 | 66 |
| `delegation-records,durable-policy` | 86 | 86 |
| `delegation-services` | 73 | 73 |
| `delegation-services,durable-policy` | 100 | 100 |
| `delegation-services,delegation-records,durable-policy` | 119 | 119 |
| `durable-policy` without services | 67 | Not separately run |

Counts include overlapping regressions and compile-fail doctests, not distinct
user journeys. Native operator/record compositions include the actual no-service
v2 refusal and preparation/recovery cases. The new controls exercise separate
service/device authority, exact SAN/pin and policy, two independently configured
CA/home realms, reciprocal admission/refusal/release, original deadlines,
cancellation, context invalidation, aggregate credential/history bounds and real
durable apply/drop/reopen. Supported builds accept valid active-service and
tombstone-only v2 fixtures; unsupported builds refuse the same bytes unchanged.
Version 1 read/init remains unchanged; explicit updates retain typed history.

All ten host profiles pass strict Clippy. 628 Python tests pass, including 44
provider contracts; current provider/codec guards and historical evidence checks
pass. Complete selected Linux/NetBSD/macOS archives, extracted trees, features,
licenses and no-JS sources were checked before selected compilation. No dependency,
lock, ordinary client/operator feature selection or independent codec CI step changed.

Fresh host client integration passes 259 tests with eight unchanged, deliberately
ignored subprocess-helper entries used by parent tests. Twenty actual classic and
Unicode 1 MiB attachment-console controls pass with the service feature disabled.
This is a host client regression, not a fresh NetBSD production-client journey or
independent modern-client interoperability result.

## Build and runtime scope

Fresh NetBSD 11 ARM64 uses Rust/Cargo 1.96.0 and base OpenSSL 3.5.7. Default,
record and new service profiles compile and run with two CPUs/about 1 GiB and two
compiler jobs. Following prior resource evidence, broader existing XMPP/IQ and
operator dependencies compile separately at two CPUs/2 GiB; their tests then run
back at 1 GiB. This batch does not claim those dependencies were freshly built
at 1 GiB. These are software envelopes, not peak-memory or supported capacity.

Every native profile captures actual product/test/library artifacts before its
runtime and verifies exact source/artifact hashes afterward. Main operator bytes
are included. No-product-recompilation checks inspect Cargo logs and artifact
identities; compile-fail doctests intentionally invoke rustdoc/rustc on negative
snippets. They do not mean there was no compiler activity of any kind.

The provider-only host matrix used the workstation's selected OpenSSL 4.1.0.
The native-mail client keeps its existing major-3 guard: the first build refused
that default, then unchanged code passed with the already installed OpenSSL 3.6.3
selected explicitly. No host package was installed and no guard was relaxed.
Primary NetBSD uses 3.5.7; this is no adoption of OpenSSL 4 for native mail.

## Test-first and retained observations

Actual REDs precede the feature/source guard, missing service API, shared-lane
cleanup correction and durable service reopen implementation. The cleanup test
exposed admission becoming free before its tracked socket was cleared; the
corrected ordering preserves the new connection's stop tracking. Persistence RED
used actual disk publication and mTLS after reopen, rather than a snapshot-only
assertion. Final GREEN keeps those substantive assertions.

[Inputs](inputs.json) pin the immutable tested source commit; use its Git objects
when later maintained files change. The [results](results.json), [host provider binaries](host-provider-binaries.json),
[host client binaries](host-client-binaries.json), [native artifacts](native-binaries.json)
and [tool identities](tools.json) bind the observed files. Raw commands, sources,
logs and individual exits are privately retained. One early interim nine-test GREEN
executable was not independently hashed/copied before later compilation; its exact
source/log/exit remain, and every final qualification executable is retained. No
source capture gap or complete intermediate-executable retention is claimed.

The first client runtime's overly long temporary directory exceeded macOS
`SUN_LEN` in an existing socket fixture. The original failure is retained; the
same binaries/assertions pass under a shorter owned 0700 root. A CI-inspection
command also used the wrong job name before the correct unchanged-step comparison.
Neither is a product RED or a waived test. Intermediate compiler/guard corrections
and all meaningful original REDs remain distinguishable from final results.

[Private retention](retention.json) and [cleanup](cleanup.json) record verified
archive identities and owned teardown. The disposable VM/controllers/forward,
clone/kernel and marked scratch roots are removed; all 86 saved owner assets
remain exact and six owner images are closed. The playground was never started.
Raw credentials/captures remain private. No physical SBC, 250-user, broad hardening,
OAuth/RFC 7662 backchannel, store grant, callback executor, deployment or complete
[#300](https://github.com/gonz0w/faction-bbs/issues/300)/
[#301](https://github.com/gonz0w/faction-bbs/issues/301) outcome is claimed.
