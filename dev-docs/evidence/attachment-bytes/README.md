# Attachment byte proof evidence

Issue [#250](https://github.com/gonz0w/faction-bbs/issues/250) records one qualified
macOS ARM64 laboratory run. The [research report](../../research/0059-attachment-byte-proof.md)
explains the results and limits; the [experiment](../../../experiments/250-attachment-bytes/README.md)
describes the interfaces. This is historical file-byte encoding and authentication
quarantine evidence, not an attachment product, modern XMPP profile, NetBSD,
streaming, private-storage or independent-primitive qualification.

The original [result](results.json) has two known-answer cases, eight reciprocal
complete inputs, 47 rejection groups and 40 chunk observations. Of the rejections,
43 are actual authentication failures on both sides and four are short-tag framing
refusals. The unchanged reference had 14 successful chunk decryptions and 26
`ValueError` failures. Both classes remain visible. Separate review inspected the
original eight regular ciphertext objects and ran the 20-test native-enabled suite
(16 stdlib tests plus four native contract groups); it did not repeat integration.

The [qualification](qualification/qualification.json), original input manifests,
command records and logs retain their observed identities. The manifests under
`qualification/` and `proof/` describe the original private execution layout;
they do not claim deleted input paths remain locally available. The
[source map](source-map.json) resolves repeated project-source snapshots to four
immutable objects. Upstream inputs are identified by verified immutable revisions,
artifact/blob hashes and license locators. Wheels, native binaries, unchanged GPL
reference source/API responses, full upstream build trees and the large public
vector dataset are excluded. Two long successful external-build logs have clearly
labeled hash-bound head/tail summaries. Other selected observations are unchanged
bytes; the preserved earlier summary explains a later metadata-only clarification.

All keys, IVs and inputs are public test fixtures. Publication selection checks
exclude compiled/key-store/private-key artifacts and the original reference bytes;
this is not a scan of real member secrets. Fixture teardown is recorded separately
only after actual removal. The generic recorded-file check verifies the final
selected bytes and folder inventory; it does not rerun the original dependency or
crypto proof. CI also runs the stdlib contracts without fetching crypto inputs.
