# Authenticated native attachment software evidence

Issues #279/#280 connect the optional native attachment APIs and production console
to the existing binary TLS carrier. The final coalesced source snapshot is commit
`65952eac8a3b11a4aab2d0bd1ac0e922f64c3380`.

Actual production consoles passed **20 controls per OS**, ten each in classic and
Unicode modes, on macOS and fresh NetBSD 11 ARM64. Each mode used a 1 MiB managed
attachment, sequential independently approved device sessions at one home, a
separate encrypted-mail relay, authenticated receipt/fetch, byte verification and
explicit private no-overwrite save. Invalid private profiles, foreign peers,
provider outage, wrong pin/key, current revocation and cancellation were exercised.
Cancellation after possible durable progress is not a guaranteed lost Commit reply.
Storage commit and encrypted-mail release remain separate operations.

[Host](host.json) and [native](native.json) records distinguish source snapshots,
commands that stopped early, individual controls, final production journeys and
setup prerequisites. Counts across feature profiles overlap. Existing subprocess
helper entries are ignored as direct harness entries and exercised by their parent
recovery controls; they are not reported as new passing tests.

## Failure history and actual corrections

[TDD](tdd.json) preserves missing-interface and behavioral RED/GREEN observations,
including the same-home service guard and contiguous frame write. It also retains
fixture failures: macOS pending-TCP setup returned ECONNRESET, a long Unix socket
path exceeded SUN_LEN, an early harness stopped before its accepted deadline, and
a stale action-time fixture correctly encountered the store's rollback guard.
Those failures are not relabeled as complete passing runs.

Eight legitimate RPCs did not naturally saturate the macOS write path. Separate
socket-BIO characterization and a composed TLS/worker test establish different
facts. The latter deliberately injects bounded synthetic raw bytes through an
owned CLOEXEC socket duplicate after a real positive request. It observes actual
backpressure, an entered request with no result while the current-thread timer
advances, then Cancelled and a joined worker with the unread peer alive. This is
connection corruption for testing, not legitimate protocol traffic or Store proof.

The actual unchanged NetBSD producer reached only 442368 of 1048592 ciphertext
bytes after 60.104629 seconds. The correction combines the existing length prefix
and payload in one bounded write buffer. Wire bytes, flush, payload/attempt limits,
clocks and cancellation remain. Delayed-ACK/Nagle causality is an inference; one
buffer does not promise one TCP packet. Final production uses the original
60-second/65-call limits.

## Sources, identities and limits

[Inputs](inputs.json) pins four snapshots with 447/447/447/448 selected inputs and
93 development variants. Its 498 unique source references reuse 395 existing
public objects and add 103 exact objects. Selection is broader than execution;
these source captures do not imply that every selected file ran. Earlier provider
records retain their original bytes and claims.

[Identities](identities.json), [costs](costs.json), [routing](routing.json) and
[teardown](teardown.json) separate executable/library/tool identities, build and
runtime envelopes, failed observations, verified retention and actual cleanup.
Requested agent roles/models are recorded separately from unavailable backend
telemetry. No workflow quality/cost benchmark or human security audit is claimed.

Provider bounds remain one session, eight attempts, five-second shared TCP/TLS
admission, ten-second cycle, 8192-byte binary payload and 16384-byte IQ; the combined
binary frame is at most 8196 bytes. Authority retains 32 bindings/homes/tombstones
and eight handles. SDK calls keep four chunks and upload's ten seconds includes
mail release. Normal BBS budgets remain eight sessions, five-second admission,
two credential permits/kernel slots, three helpers and five-second drain.

This remains explicit, default-off, same-home software integration. Transport
certificate possession and registry approval do not prove OMEMO private-key
possession, enroll a production device or grant BBS membership. No XMPP server/SASL,
custom IQ carrier, delegated foreign-home transport, production provider deployment,
independent modern-client interoperability, physical hardware or 250-user support
is established. Ordinary mail and classic BBS/file access remain independent.

See [reproduction](REPRODUCE.md). The recorded-file guard verifies saved bytes and
selected membership; it does not rerun a VM or attest the interpretation of results.
