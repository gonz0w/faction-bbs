# Delegated retrieval software evidence

This record covers the default-off publication observation transport and private
retrieval verifier at source `05ce1933712e3ae9ddf138b472005097d3fa0405`.
The [inputs](inputs.json), [results](results.json), [test-first record](tdd.json),
[binary identities](binaries.json) and [tools](tools.json) retain source and
execution details. [Reproduce](REPRODUCE.md) describes the current command authority.
This software evidence is not deployment, independent interoperability, physical
hardware or capacity qualification. GitHub CI and merge remain separate workflow gates.

## Scope

Actual independent-home component tests exercise device-derived intent and relay
issuance, closure of the incoming session, fresh authenticated introspection and
private exact GetChunk, EOF and recipient Status against committed ciphertext.
The object was seeded only by a test-only exact binding/sender/setup Access through
normal Begin, PutChunk and Commit and a Store reopen; that Access is never used for
retrieval. The resource home has only its local sender device, and its ordinary
session still refuses foreign publication. This is not product cross-home upload,
native relay or mail release. [#300](https://github.com/gonz0w/faction-bbs/issues/300)
and [#301](https://github.com/gonz0w/faction-bbs/issues/301) remain open.

Current policy permits a resource-only or issuer-only configuration, not both in
one policy. Device keys and plaintext remain on the device. TLS/BIO scheduling
controls do not prove kernel saturation. The original deadlines, cancellation,
current policy, physical reservation and ordinary admission controls are distinct
from synthetic-clock tests.

## Host and NetBSD results

All 16 provider profiles passed on host and NetBSD, with matching counts below;
the standalone codec passed 26 on each. Host profiles also passed strict Clippy.
Counts include the selected tests and doctests, not a capacity measure.

| Provider profile | Host | NetBSD |
| --- | ---: | ---: |
| `blob-iq` | 58 | 58 |
| `default` | 49 | 49 |
| `delegation-issuer` | 112 | 112 |
| `delegation-issuer,durable-policy` | 149 | 149 |
| `delegation-records` | 66 | 66 |
| `delegation-records,durable-policy` | 89 | 89 |
| `delegation-services` | 73 | 73 |
| `delegation-services,delegation-records,durable-policy` | 122 | 122 |
| `delegation-services,durable-policy` | 103 | 103 |
| `delegation-verifier` | 194 | 194 |
| `delegation-verifier,durable-policy` | 236 | 236 |
| `delegation-wire` | 151 | 151 |
| `delegation-wire,durable-policy` | 188 | 188 |
| `durable-policy` | 70 | 70 |
| `operator` | 87 | 87 |
| `preparation` | 53 | 53 |

The NetBSD ARM64 software guest used two CPUs and 1 GiB at runtime. Fourteen small
profiles and the codec were built at 1 GiB; blob-iq and operator were compiled at
2 GiB and their exact binaries ran at 1 GiB. No product recompilation ran during
provider runtimes; rustdoc still invokes rustc. Independent verification covered
401 source files, 143 selected registry archives and extraction, and 480 artifacts.
The owned test TMPDIR was empty; no copied BBS, compiler or test processes
remained.
These are software envelopes, not physical resource or supported capacity claims.

The historical `3c4caa2` wire runs reused cached `9cd481b` artifacts and are not
fresh wire builds or v4 evidence. Before the corrected `05ce193` builds, four
changed source files had mtimes advanced with bytes verified unchanged; all seven
provider targets rebuilt in each wire and verifier profile. The earlier results and
artifacts remain distinct. The test-first record also discloses the early fixture
map omission and the administrative read-only verification of root-owned historical
metadata; product tests remained unprivileged.

Host client checks at `3c4caa2` passed 259 selected client and 50 default tests,
plus 20 classic and Unicode console transfers of 1 MiB per mode. They were not
rerun at `05ce193`: the captured feature/dependency comparison excludes all four
changed files, the other 817 tracked inputs and retained binary were unchanged,
and separate review accepted applicability. No fresh NetBSD production-client
console result is claimed. The host console probe data fixture was removed and its children reaped; capture
and build roots are retained separately for preservation.

Source gates, 642 tooling checks, pinned Temper manifest and 56/13 guards passed.
A separate nonauthor source and documentation review cleared its findings; it is
not independent assurance. The clock and queued-Authority time corrections and
actual red/green controls are detailed in [tdd.json](tdd.json).

The [native verification](native-verification.json) also records verification of
11,583 regular archive paths and hardlinks. The disposable VM and controller
stopped, its owned forward closed and clone and kernel were removed. All 86 saved
owner assets remained byte-exact, six owner images were closed, and the owner
playground was never started.
