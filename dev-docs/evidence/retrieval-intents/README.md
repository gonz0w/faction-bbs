# Session-derived retrieval intent software evidence

Issue [#306](https://github.com/gonz0w/faction-bbs/issues/306) adds an explicit,
default-off local authority primitive. An approved device TLS session can create
an opaque retrieval intent; normal session closure releases the connection while
the record keeps only its original bounded lifetime. It grants no store access.
See the [API contract](../../retrieval-intents.md).

## Observed results — 2026-10-30

| Selected provider profile | Host checks | NetBSD checks |
| --- | ---: | ---: |
| Default | 48 | 48 |
| `delegation-records` | 65 | 65 |
| `delegation-records,durable-policy` | 84 | 84 |
| `blob-iq` | 57 | 57 |
| `operator` | 82 | 82 |

Counts include repeated ordinary regressions and compile-fail doctests; they are
not distinct end-user journeys. The two new selections each pass ten privacy
compile-fail doctests. Actual mTLS controls cover normal closure/readmission,
aggregate eight-record capacity, exact tuple/action/offset/audience checks,
independent invitation/local-home checks, stale policy and durable reopen.
Private deterministic tests cover original deadlines, certificate/object and
registry expiry, clock rollback, nonresurrection and bounded pruning. Test clocks
and source constructors remain inaccessible to external callers.

625 Python tests pass, including 41 provider and 11 independent codec contracts.
Both new profiles pass Clippy, and formatting passes. Locked registry archives,
extracted trees, selected features, licenses and no-JS source checks pass for
Linux, NetBSD and macOS before selected compilation. No external dependency,
lockfile, default or client feature selection changed. The ordinary client graph
remains separate; this record does not claim a fresh production-client journey.
Required PR CI also runs the existing client and operator integration checks.

## Native build and runtime envelopes

Fresh NetBSD 11 ARM64 uses Rust/Cargo 1.96.0 and base OpenSSL 3.5.7. Default and
both new profiles compile and run at two CPUs, 1,054,990,336 bytes physical memory
and two compiler jobs. The existing `xmpp-parsers` dependency's broader blob-IQ
build was killed at 1 GiB with both two and one compiler jobs; UVM recorded out of
swap. Its unchanged blob-IQ/operator test binaries were then compiled at two
CPUs/2 GiB/two jobs with `--no-run`, followed by actual tests back at 1 GiB.
The latter tests check exact existing test/library artifact hashes before/after
and refuse compilation in their Cargo logs. These separate envelopes are not
peak-memory, physical SBC, 250-user or production-support claims.

## Source, failures and retention

[Inputs](inputs.json) distinguish pre-edit Git blobs, the host Rust execution map,
the final guard/tooling map and all 371 exact native source inputs. Only the CI
workflow differs between the host execution and final guard maps: six accidental
codec-step warmup lines failed existing tests and were removed. Its old bytes
remain privately preserved; no Rust source or old measurement was relabeled.
[Results](results.json) retain command/log identities and each profile outcome.
[Host binaries](host-binaries.json), [native artifacts](native-binaries.json) and
[tools](tools.json) bind actual retained files. The maintained operator executable
hash is a separate post-run observation, outside the pre/post test-artifact map;
no per-process executable attestation is claimed.

Primary sourceguard/API RED and final GREEN logs are retained. One intermediate
visibility-error raw log was overwritten: its diagnostic is a labeled tool
transcript copy, and recovered source is labeled derived, not an original snapshot.
Sandbox listener refusals, a missing verification-helper import and a private
runner indentation error remain separate failed observations. The public result
lists these limits explicitly; passing later runs do not erase them.

The [retention record](retention.json) identifies the private source/log/binary
archive. Native sources and all 139 selected registry archives were rechecked
after execution. The owned VM, controllers, forward, clone and scratch roots are
removed; all 86 saved owner assets remain exact and six owner images are closed.
The owner playground was never started. Credentials and raw captures remain
private. No token issuer, introspection, sender-current/object-commitment proof,
remote grant, relay, durable delegation registration or full cross-home delivery
is delivered here; parent [#300](https://github.com/gonz0w/faction-bbs/issues/300)
remains open.
