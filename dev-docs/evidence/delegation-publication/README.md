# Composed policy and publication software evidence

This record describes component observations at runtime source `f59`. The
[reproduction guide](REPRODUCE.md) identifies the command authority and limits.
The default-off library composes issuer and resource policy in one home runtime
and adds an explicit publication association with durable v5 policy. See the
[verifier and publication contract](../../delegation-verifier.md) for the API,
shared limits and whole-second observation boundary.

The final component tests use actual authenticated sender sessions for publication,
reopen the durable Store, and retrieve through the recipient path in both directions.
The positive object is created without test-only SetupAccess. Its payload is
synthetic ciphertext; this is not a new cryptographic or independent-client
interoperability qualification. The physical continuation reservation remains
shared within the aggregate eight slots, with existing credential/history and
home/floor limits of 32. Keys and plaintext remain device-side. Classic access and
default-off behavior remain; no new dependency is introduced.

Host and NetBSD 11 AArch64 runs passed all 16 selected provider profiles and 27
codec tests; strict host Clippy passed. NetBSD used Rust 1.96 and OpenSSL 3.5.7.
All provider runtime tests ran with two CPUs and 1 GiB: 14 profiles compiled at
1 GiB, while blob-iq and operator were built at 2 GiB and tested at 1 GiB. Exact
selected source inputs numbered 811 for native and 824 for host, including seven
binary fixtures. No installation or source change was part of those runs. The
initial #317 checkpoint `04fc` also had a 249-test native 1 GiB observation.

Fresh host regressions passed 259 device-client and 50 default tests, plus 20
maintained classic and Unicode 1 MiB console controls. At 1 GiB, NetBSD passed
260 device-client tests (including one native-only pending TCP test), 50 default
tests and 20 classic and Unicode 1 MiB console controls. The production client
was the retained original Cargo-reported executable, not a recompilation; the
console fixture was also retained. See [results](results.json) and
[native verification](native-verification.json) for identities and observations.

The first native attempt had three OpenSSL fixture setup failures: the private
configuration and test Python alias were absent. A second attempt had six setup
failures after a default-test build overwrote the client alias. Both failed outputs
and helper bindings were retained; the first child OpenSSL stderr was unavailable.
The corrected run retained assertions, source, time and resource limits and excluded
fixture environment overrides from the production console. Raw executables,
failures and commands remain private. For the initial #317 RED 02, original
executable bytes are unavailable; its exact source, log and identity remain
retained. This is not a complete intermediate-binary archive.

[Privacy](privacy.json), [retention](retention.json) and [cleanup](cleanup.json)
are separate records. Independent review and CI remain separate workflow gates.

This is library component software evidence, not provider deployment, physical
SBC or capacity qualification. Maintained native/operator response carriers,
encrypted-mail release and the full offline console journey remain under
[issue #301](https://github.com/gonz0w/faction-bbs/issues/301). The existing
[retrieval verifier record](../delegation-verifier/README.md) remains historical;
its test-only object setup and results are unchanged.
