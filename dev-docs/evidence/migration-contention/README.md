# Concurrent migration test correction

Issues [#244](https://github.com/gonz0w/faction-bbs/issues/244) and
[#289](https://github.com/gonz0w/faction-bbs/issues/289) distinguish a permitted
SQLite `Busy` result from a migration/data failure. Production code, schemas,
CI, dependency versions, credentials and the 100-ms database busy timeout stay
unchanged. This is a test-only correction.

The original Linux CI failure and unchanged-head diagnostic pass remain separate.
The entire core source tree at base `6a211a5` matches the failed `585eacf` tree.
The failed job's retained log is complete; the diagnostic pass has successful job
metadata and a bounded, truncated log tail. Neither records the historical lock
phase, scheduling delay or duration, and the later pass did not resolve the issue.

An unchanged host baseline passed once. Equivalent local fixture/single-attempt
factorization retained that success. Real held-writer controls then produced two
REDs: releasing only after an actual `Busy` still failed the one-attempt success
oracle, and persistent contention made one call instead of the required three.
The actual conflicting-table non-Busy rollback and original concurrency checks
already passed. Their working behavior is not relabeled as RED.

Only the test-local retry policy changed for GREEN: three total attempts, only
on `Error::Busy`, returning every other error immediately. The same assertions
then passed: second actual open after writer release; exactly three blocked opens
with unchanged schema-one/data followed by separate recovery; one actual
conflicting-DDL `Storage` failure with rollback; and both original synchronized
openers with every prior success/data assertion plus direct identity comparison.
Exhaustion still fails the original success oracle.

The full host suites passed 14 chat, 17 core and 4 credential tests. Twenty
independent original migration reruns passed. These overlap the suites, do not
promise success under starvation, and are not a latency/capacity benchmark.
Strict release core Clippy and formatting passed. Per-version migration commits
are separate: a failed open may leave a valid committed prefix, so this record
makes no whole-upgrade rollback claim.

[Results and exact source/executable/tool/log identities](results.json) retain
baseline, factorized, RED and GREEN distinctions. The three exact measured test
variants are in `sources/`; the original source reuses its immutable earlier
object. Raw execution records, executables and caches are retained privately.
All children were reaped and marked fixture roots are empty; the private scratch
root moved into ignored retention. No VM, service, package or owner-playground
activation occurred. No fresh NetBSD, hardware or security-audit execution is
claimed for this test-only change; Linux CI is a separate delivery gate.

Reproduce after the default source/license/no-JS qualification with Rust 1.96,
locked dependencies and at most two compiler jobs. These are normalized public
recipes, not the retained private execution argv:

```sh
cargo +1.96.0 test -p faction-core --release --locked --offline --test chat migration_
cargo +1.96.0 test -p faction-core --release --locked --offline --test chat
cargo +1.96.0 test -p faction-core --release --locked --offline --test core
cargo +1.96.0 test -p faction-core --release --locked --offline --test credential_budget
cargo +1.96.0 clippy -p faction-core --all-targets --release --locked --offline -- -D warnings
```

Use the preserved RED source against its original one-attempt adapter when
reproducing that failure; current code cannot retroactively reproduce old RED.
For exact binary comparisons, build without running first and materialize/hash
Cargo-reported test executables before execution or overwrite. Source qualification
and compiler/runtime costs are distinct. The saved-byte guard checks the pinned
record and source objects only; it does not rerun a compiler or database test.

The publication selects known metadata fields and owned test-source bytes.
A scoped scan covers retained owner-specific path strings; generated fixture
secrets and in-memory/deleted values were unavailable for a runtime-value scan.
No raw private values or whole-runtime privacy guarantee is inferred.
