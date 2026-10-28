# Durable attachment mail validation

This public record supports [#257](https://github.com/gonz0w/faction-bbs/issues/257) and [#258](https://github.com/gonz0w/faction-bbs/issues/258) from baseline `0880fbd0d047f30ee2361a32a276b8d18b954d5a`. The [contract](../../attachment-mail.md) defines explicit FACTCP05 attachment-message adoption and retained-history verification/save. Bulk ciphertext remains caller-supplied; parent [#88](https://github.com/gonz0w/faction-bbs/issues/88) stays open for private storage, authorization and automatic file transport.

## What is retained

`facts.json` contains generated fixed-schema observations: reviewed repository-relative source identities, commands, exit/count/timing values and hashes of original records. `sources/` preserves the 19 reviewed final implementation, test, tooling and documentation inputs. Prose snapshots use literal `.md.txt` files, preserving original relative links as data. The [pre-upgrade baseline](../attachment-mail-baseline-manifest.json) separately retains ten committed inputs for historical byte resolution; no old observation or checksum was refreshed.

The exact original local source variants, commands, logs, rejected candidates and separate-review records are retained in an owner-only immutable archive outside Git. Its identity, size, regular-file membership and access properties are recorded in the public facts. Raw local execution data was deliberately excluded from public release after automatic disclosure review. A clone cannot open those private historical originals. Current source/tests and the closing PR's hosted checks remain public and reproducible. Hashes identify retained bytes; they do not prove assertions or provide a fresh runtime test.

## Observed checks

Actual missing APIs established RED before schema/queue/bridge producers. The historical resolver also failed on a simulated live-source edit before the additional pinned baseline was registered, then passed all 13 tests. The first private name filter ran five private cases and zero public cases; the public schema and four transaction tests were rerun without that filter. Filtered entries are never counted as passes.

The expanded private selection passed 20 harness entries with two ignored core child helpers actually invoked by parents. This includes 12 new core behavior entries and eight existing save harness entries; two existing save entries are no-op helpers in an ordinary parent invocation, with their real child executions controlled separately. Old CP02/03/04 rejection, peer/device/full-identity retry binding, malformed whole-state fields, recovery and cursor/read preservation were exercised. Review found an unbounded crash-child wait; its new bounded helper had an actual missing-function RED, followed by a five-second kill/reap implementation and an observed-ready stalled-child test. An assertion-preserving test-style Clippy correction was separately replayed.

Existing checkpoint/archive/read/network-time regressions passed 39 entries with three controlled ignored helpers and 26 deliberately filtered entries. The full debug Cargo regression command observed 270.38 seconds and 76,529,664 bytes peak process RSS, including driver, tests and child overhead; this is not per-call allocation, throughput or BBS capacity evidence.

Bridge tests passed six native and six network-feature entries, plus two existing-home scripted Channel controls. They exercised real native encryption, durable queue/reopen, exact pending retry after failed write, live/archive duplicates, partial-page failure, receiver reopen and exact private saving. The home channel is scripted; no new live home service or two-home bulk transport is claimed. Separate peer-agent replay passed 14 actual entries with unchanged source: 11 public cases, exact home, exact poisoned-history refusal and exact stalled-child control. It is a second examination, not independent human assurance.

Default-feature tests, native/network all-target Clippy, final formatting, product dependency inventories and historical/repository checks passed. Full tooling initially found a stale 55-helper expectation after the accepted addition made 56; the corrected test requires exactly 56 plus new-helper membership and retains every read/hash/tamper/cache-size oracle. A restricted rerun could not bind an existing ephemeral loopback refusal-test socket. With normal test capability, 11 targeted and all 479 tooling tests passed; preview process launches remained mocked and asserted absent. Neither failure is a product RED or a skipped passing check.

The unchanged standalone research graph check was unavailable locally offline because an existing locked package was not cached. No package, lock or dependency graph was changed. The unchanged exact-head hosted native/network/core/terminal/INN and policy checks remain integration requirements; the closing PR records their actual outcomes.

## Qualification and cleanup

These are software observations on the recorded host and existing OpenSSL3 toolchain. No fresh NetBSD/SBC, modern-client profile, power-loss, secure-erasure or capacity qualification follows. No UI/CLI attachment entry, automatic blob fetch/upload/spool, public HTTP requirement, provider or host-policy change is introduced. Existing classic access, local approval and member-device-only keys remain.

`cleanup.json` records actual disposable-root teardown and the separately retained private archive. The owner playground stays stopped and intact. The existing generic `tools/check_recorded_files.py` can verify `manifest.json` using the digest pinned in the closing PR; it verifies these saved public bytes, not private archive access or cryptographic correctness.
