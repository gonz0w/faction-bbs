# Bounded resource-collection software observations

This record preserves controlled observations for [issue #326](https://github.com/gonz0w/faction-bbs/issues/326). The [protocol contract](../../delegation-verifier.md#composed-carrier-preview) describes the explicit bounded service-purpose 2 wait. GitHub owns work status; this folder contains executed source and observations, not a second tracker.

One unchanged synthetic maximum-file workload completed all **1,048,592 bytes in 52.329 seconds**, including final digest and deadline assertions within the original 60 seconds. It made 65 Status and 257 Get requests over 65 cycles. Both provider loops joined and both homes released their charged ownership; drain took 2.978 ms. This is one controlled result with fresh generated private PKI, not historical Linux causation or a general reliability guarantee.

The same frozen 20 ms dispatch envelope previously stopped at 524,288 bytes with the original pacing. A depth-4 pacing candidate stopped at 942,080 bytes and was rejected. Its failure remains part of the record. The accepted structural candidate leaves the original outer depth-3 pacing and all deadline, attempt and resource bounds intact. Foreground, harness and controller durations are separate clock scopes; overlapping timing counters must not be added.

The [results](results.json) identify the actual source objects, executable and raw private-log hashes. Sources retain the project [BSD-3-Clause license](../../../LICENSE). Generated certificates, keys, tokens, Store data, endpoints and executables remain private. Old failed aggregates and unavailable preconditions are distinguished from passes. The first broad host aggregate had 155 passes and two failures; a subsequent controlled test established a binding-regeneration fixture defect class. It does not reconstruct that aggregate's missing failed Store state. A later instrumented run localized a router Frame to its follow-on read after a client response. The trusted fixture dropped TCP without an explicit TLS close. Its corrected variant sends close notification and passes all eight original case/request assertions. Server truncation and error semantics remain unchanged; this is not proof of an OpenSSL defect or the exact historical I/O cause.

## Source identity and limits

The source objects are immutable inputs. They are not current agent instructions or current product modules. The baseline is commit `c8cd988ff027ea603686d4ee618d2dbc752773b8`; only the listed execution-source overrides differ. The frozen workload and delay hook are preserved byte for byte. The selected public objects are a subset of the privately retained attempted inputs and captures; preservation of an old source is not a fresh execution.

The initial wire RED used `wire-red-tests.rs` with `baseline-delivery.rs`. The finished-result contention RED used `published-result-red-tests.rs` with `published-result-red-delivery.rs`. The successful maximum workload used `performance-delivery.rs`, `performance-tests.rs`, `frozen-delay-loop.rs` and `frozen-maximum-workload.rs` in their respective provider module paths. Fifteen focused controls used `fifteen-controls-tests.rs`; only the later expiry setup changed before its separate passing rerun. Binding-helper RED/GREEN objects preserve the later test-only correction.

The host used Rust 1.96 with locked offline Cargo, two compiler jobs and one test thread. The frozen-workload and provider-profile executables linked installed `libssl.4`/`libcrypto.4`; their certificate fixtures used the existing OpenSSL 3 CLI. The separate integrated client, fixture and production operator checks below used matched OpenSSL 3.6.3. These are distinct observed environments, not a new target dependency adoption. No fresh native, capacity, deployment or independent interoperability claim follows from this record.

## Reproduction

Use a separate marked disposable checkout of the baseline with its qualified offline dependencies. Restore the selected exact source objects to `crates/faction-provider/src/delivery.rs`, `publication_tests.rs`, `operator/composed_loop.rs` and `operator/composed_loop_tests.rs`. The maximum workload uses all four performance objects named above. Build the test executable with:

```sh
cargo test --manifest-path crates/faction-provider/Cargo.toml --features operator,delegation-verifier --locked --offline --lib --no-run
```

Run only `authority::services::publication::tests::composed_loop_tests::frozen_dispatch_envelope_maximum_fetch_with_original_foreground_deadline` with `--exact --nocapture --test-threads=1`. The workload legitimately publishes its object before the fixed fetch, uses the maintained composed loop and checks cleanup. Preserve the executable, inputs and failing results before changing anything. Fresh private fixtures reproduce the envelope, not an identical old fixture. Keep captures private and use only authorized loopback test execution. The artifact is not a deployment or a supported benchmark promise.

## Framing and fixture controls

A scoped test-only observer records fixed phase, framing part, outcome and optional I/O ErrorKind. It holds no payload, identity or error string. Twelve actual writer/TLS/scope/privacy controls passed after the missing-interface RED. An abrupt close produced Frame; an orderly close produced a clean boundary EOF. The one instrumented router run failed at the follow-on read on its first request; the original exact-response assertions were not reached. Its actual request and post-operation Store were retained privately before judgment.

The fixture correction sends the first close-notify step once after receiving a response. It preserves the original cancellation/deadline and request counts, and neither waits for another peer message nor alters production destruction. Thirteen frame controls, including cancelled and expired close refusal, passed. One corrected router run then reached every original response-byte, ownership, pending-work and callback assertion across both certificate variants and four requests. No separate drain duration was measured for that router case. Earlier failures and their unavailable evidence remain separate.

## Full host validation

Release Clippy and full test commands passed for all three provider profiles. The profile results below used the host's default OpenSSL 4. These do not replace the separate integrated OpenSSL 3 results.

| Provider features | Library passes | Other target passes | Documentation tests |
| --- | ---: | ---: | ---: |
| delegation-verifier | 165 | 90 | 38 |
| delegation-verifier, durable-policy | 201 | 118 | 38 |
| operator, delegation-verifier | 202 | 143 | 38 |

Some existing pending-TCP controls reported an unavailable host precondition. Their harness passes are not pending-TCP runtime proof. The initial durable aggregate's three create-new capture collisions were setup failures: each subsequent command received isolated marked capture and temporary directories. Earlier capture bytes and the failed aggregate remain retained. A test-only named return alias resolved the observed Clippy complexity error without suppression or changed assertions.

The client native adapter correctly rejected the default major-4 headers. The remaining integrated checks used the already-required installed OpenSSL 3.6.3, with explicit matching headers, libraries, pkg-config and no vendoring. No guard, source dependency or system installation changed. Client Clippy/build and 274 all-target tests passed, with eight existing ignored helpers. The saved client, fixture and production operator link family 3 only. Twenty-one Cargo-reported executable artifacts were retained; non-SSL consumers were classified separately.

One maintained console probe passed all 16 checks across classic and Unicode modes, using two independent maintained operator processes and at most one active client. Each mode transferred the exact 1,048,576-byte plaintext / 1,048,592-byte ciphertext and completed 65 fetch calls. Classic upload/fetch observations were 7.931 / 16.354 seconds; Unicode observations were 7.878 / 16.310 seconds. These scopes differ from the earlier unoptimized frozen-delay workload. Children were reaped and private fixtures retained. The separate reviewer accepted the frozen host code; native, CI and final publication gates are outside these observations.
