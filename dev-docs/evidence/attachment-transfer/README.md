# Automatic encrypted attachment transfer: host software evidence

Observed on 2026-10-28 for [#260](https://github.com/gonz0w/faction-bbs/issues/260)
and [#261](https://github.com/gonz0w/faction-bbs/issues/261), from base
`c48ac5a564bad850b5c48def9ce51e97c756824f`. These are immutable observations,
not live assignments or a capacity, security-audit or production-support claim.
The [storage and transfer contract](../../private-attachment-storage.md) describes
what the optional library implements and what the embedding home must enforce.

## What actually ran

[Results](results.json) distinguish successful test groups from the original failed
commands. The initial broad network run passed 112 library and 68 binary tests,
then stopped at an unchanged test's symlinked inherited temporary directory. The
canonical `/private/tmp` integration-only rerun passed all 22 tests, including
queue-failure/orphan, exact-retry, historical verification and wrong-key controls.
Other checks passed: 16 store tests, 14 focused OMEMO tests, one fixture privacy
helper, 64 default client/files tests and 484 Python tooling tests. Ignored child
helper entries are recorded separately; the store death-test parent actually
invokes its child at five publication points and kills/reaps each one.

The [real TLS journey](tls-result.json) transferred 1 MiB of plaintext using actual
member `Home`/`Session`, the strict IQ codec and the Rust store handler. It took 64
bounded upload calls after a lost durable chunk reply and 64 successful fetch
calls after a partially accepted fetch was stopped by current revocation. The
sender was offline during fetch. Typed uninvited/stale-frame preflight controls
made no connection; a composed two-session timeout sent no additional message.
Device-side auditing found the actual private names, subject/body, key, IV,
plaintext hash and selected plaintext pattern absent from server files and IQ
payloads. Verification and explicit no-overwrite saving succeeded. Generated TLS
keys, credentials, runtime vaults and transcripts were removed with the fixture.

The 173.991-second journey includes certificates, debug client processes, repeated
vault unlocking, TLS setup and fixture work. It is not a per-member latency or
throughput budget. The 11.054-second deadline observation includes unlocking
outside the production operation's cooperative ten-second timeout. Byte counters
are measured at the test homes receiving client requests; they are not exported
member-side parser telemetry. The existing parser still enforces its per-Session
physical-input/event/stanza limits. One completed upload can use two sequential
Sessions under the same outer timeout; no transport cap was increased.

Separate store failure/death controls cover snapshot publication. Integrated
queue-failure/reopen characterization covers committed ciphertext with a refused
mail queue; this is distinct from killing a client between those publications.
An unqueued intent can leave a bounded opaque orphan without its lost key. Exact
retry applies to durably queued intent, not that incomplete preparation. Known-ID
retirement and backup inspection remain explicit; there is no inventory UI.

## Source, test-first history and review

[Integrated inputs](integrated-inputs.json) pin 23 final source/test/documents.
[Executed TLS inputs](tls-inputs.json) pin thirteen exact source inputs and the
actual binary digest. Their bytes are retained under `executed-tls/`; the final
fixture was subsequently formatted without changing its tokens. `baseline/`
retains the pre-edit client exports, home adapter and descriptor. Older historical
records and their existing resolvers remain unchanged.

Observed REDs include missing store/transfer/wire APIs, private-URN rejection,
Sync sending a managed wire prematurely, expired data reviving after clock rollback,
distinct missing/denied wire replies and a missing whole-call deadline helper.
Fixture helpers also have actual missing-interface REDs before implementation.
The first TLS attempts failed at archive initialization order and inherited XML
namespace handling. These failures and corrected variants remain retained; none
is counted as a successful transport observation. Tests/assertions were preserved.

The separate Temper reviewer examined source, contracts, exact input identities,
original failures/passes and the TLS journey; [review facts](review.json) record
its scope. This is a second examination, not independent assurance. Formatting,
scoped native/network/files Clippy, unchanged dependency/license/no-JS inventories,
repository links and historical-source checks passed locally. Hosted integration
checks are a separate PR gate; this record does not predict their outcome.

## Private retention and cleanup

[Retention facts](private-retention.json) identify an owner-private archive outside
Git containing the exact original source and execution captures. Public source and
bounded facts are available here; a public clone cannot inspect that private raw
history. No raw logs, runtime keys, vaults, private endpoints or transcripts are
published. The archive has private modes and its regular member bytes were verified.
[Cleanup facts](cleanup.json) record marked ownership, empty descriptor checks and
removal of the three disposable source/test/build roots. The owner playground was
not started or changed.

This proves a bounded software library/handler path between owned test homes.
Stock-provider integration/deployment, cryptographic device attestation,
member-facing attachment menus, independent modern-client interoperability and
fresh NetBSD qualification remain outside this observation. Original parent
[#88](https://github.com/gonz0w/faction-bbs/issues/88) remains open.
