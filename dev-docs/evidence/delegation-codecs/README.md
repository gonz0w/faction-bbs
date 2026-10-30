# Isolated delegation codec qualification

Issues [#303](https://github.com/gonz0w/faction-bbs/issues/303) and
[#304](https://github.com/gonz0w/faction-bbs/issues/304), observed 2026-10-30 from
baseline `734fe15df15621a39a7d01cc6f24d0abff3ed1fd`. The standalone
[codec contract](../../delegation-codecs.md) describes the implemented syntax.
[Results and identities](results.json) record source and actual executable hashes;
[retention](retention.json) pins the private archive and its precise limits.

## Actual checks

| Layer | Observed result |
| --- | --- |
| Source qualification | Ten selected registry crates, 318 source files; Linux, NetBSD and macOS graphs match. Exact archives/extracted trees, licenses, features and complete selected build/test/CI trees checked before compilation. |
| Product isolation | Root default/native-mail/network-mail/device-provider and provider default/operator graphs exclude this crate on all three targets. |
| Host Rust | 15 integration tests and two compile-fail doctests pass; strict Clippy and formatting pass with Rust 1.96.0. |
| Source/CI contracts | 11 Python contracts pass, including independent validation of the actual workflow. |
| Fresh NetBSD compilation | Rust/Cargo 1.96.0, 15 integration tests and two compile-fail doctests pass with the same eight Rust/manifest/lock/test input hashes as the host. |

The native clone used two CPUs and 1,054,990,336 bytes of physical memory with two
build jobs. The native run first compared its actual Cargo metadata and all ten
selected archives/extracted trees with the qualified target inventory. Only then
were build scripts and codec code compiled. The native log records a fresh build
of all ten dependencies and the library, not reused host/native test binaries.
Post-build archive/source checks also passed. Five inactive cached packages were
transported only for offline Cargo resolution; they are not selected build inputs.
All transported source trees were scanned; no upstream CI or dependency tests ran.

The tests use independent fixed form/JSON/HTTP byte fixtures, round trips and
hostile input. They cover strict full-message CRLF framing, Content-Length and
body/header/field/depth bounds; duplicate form keys after decoding; malformed
percent/UTF-8; nested and escaped duplicate JSON keys; wrong types, overflows and
purpose-specific fields; canonical certificate-thumbprint syntax; fixed refusals;
and absent token Debug/Display. Passing syntax checks grants no authority.

## Test-first chronology and corrections

The actual missing-checker interface failure preceded the new source guard.
Further isolation and missing-CI-validator REDs preceded their implementations.
The Rust test first failed against the empty crate because the codec APIs did not
exist. Initial positive and negative cases then passed on the first implementation.

Separate review found that httparse's defaults accepted mixed bare-LF headers and
leading empty lines. The new otherwise-valid request/response suite failed on its first mixed-LF
request before the explicit CRLF correction; its remaining variants were then
exercised by the complete GREEN run. Other form and nested-confirmation negative
fixtures were strengthened to start with valid references, so duplicate/encoding
refusals are causal rather than masked by an unrelated invalid scope/thumbprint.
These characterization corrections do not claim another producer defect.

Review also found a CI-validator gap: valid `if: false` or
`continue-on-error: true` keys after the run block bypassed its substring check.
Those mutations produced actual REDs before whole-step boundary validation.
A further test of the actual repository workflow under a temporary skip mutation
failed, then passed with the real unskipped workflow. Ordinary required unittest
discovery runs that actual-workflow assertion independently of the guarded step;
a step cannot silently skip its own only validator.

Captured prior producers, tests, workflow variants and logs are retained privately.
The derived execution-stage map identifies 27 stages and all 25 retained logs;
it is later curation, not an original execution log. It flags that the isolation
seven-test GREEN and several policy/formatting observations lack separate logs.
Original inline edit/snapshot/control runners remain tool-history observations,
not retained script files. The temporary skip-mutated actual-workflow YAML was
removed; its exact file is not retained or reconstructed. Original wrappers did
not capture every individual command exit status. Do not infer those statuses
from a later wrapper command's exit.

The final lead verification separately records individual command exits, logs,
source/tool identities and unchanged before/after inputs for the source gate,
11 Python contracts, formatting, Clippy, 15 wire tests/two doctests, repository and
860-reference evidence checks. These are fresh final checks, not replacements
for the earlier observations. Unchanged shared scanner sources remain resolvable
at the baseline Git revision and were compared with current bytes. No missing
artifact or failing run was reconstructed, and old GREEN results keep their own
input identities.

## CI and historical sources

One straight-line step was added after checkout in the existing required
Documentation and policy job. It warms locked metadata for the standalone codec,
root device-provider and standalone provider-operator graphs on the three targets,
then runs the offline source gate before formatting, Clippy and tests. Those
supersets cover all six inspected ordinary graph profiles, including fresh caches.
Existing required needs/results, commands, two-compiler-job setting and
15-minute job budget remain unchanged. Literal-step checks are not a general YAML,
shell or CI trust proof; separate source review remains required.

The earlier workflow bytes already reside in immutable source object
`dev-docs/evidence/sources/offline-provider-preparation/b443cd53fc41d6758330c95e6a8db8ec7292ad5a6c32284c0d20254854bcb9df`.
Its 42,203 bytes match the pre-change workflow; all 860 historical references
passed without changing an old manifest, record, measurement or shared checker.

## Cleanup and limits

The private native source/build archive retains actual sources, test binaries,
compiler identities and logs. Codec/compiler processes and copied BBS services
were absent before shutdown. The disposable VM and serial controller stopped;
its one loopback administrative forward closed and the owned overlay/kernel copy
were removed. All 86 saved owner assets were rehashed identically; all six saved
images were closed. The owner's playground was never started.

This is library/source/software validation only. The codec has no product/provider
edge, listener, network runtime, TLS verifier, issuer, token generator/registry,
Access or Session conversion, device key handling or store authority. Neither an
active JSON variant nor a matching thumbprint proves approval or possession.
There is no current-clock, invitation, revocation or delegated resource check.
No actual two-home delivery, interoperability, security audit, physical SBC or
supported-capacity claim follows. The remaining authority and relay outcomes stay
in [#300](https://github.com/gonz0w/faction-bbs/issues/300) and
[#301](https://github.com/gonz0w/faction-bbs/issues/301).
