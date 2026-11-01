# Own-home encrypted attachment software evidence

This record covers [#321](https://github.com/gonz0w/faction-bbs/issues/321) and
[#322](https://github.com/gonz0w/faction-bbs/issues/322): an explicitly selected,
default-off submit/close/collect carrier and native attachment route through each
device's configured home. Current authorization, exact full principals, original
attempt/deadline accounting and the physical reservation survive the handoff.
Encrypted mail is released only after the authenticated exact committed result.

Selected runtime, build and helper inputs match
[`9fddcb151c2b94116f849408305f5c9a35ac5fe9`](https://github.com/gonz0w/faction-bbs/commit/9fddcb151c2b94116f849408305f5c9a35ac5fe9).
The broad private 9,358-file capture includes unowned nonruntime policy changes
excluded from that commit; its inventory SHA-256 is not a Git revision. The
[source record](source-provenance.json) preserves that distinction. Later evidence,
prose and registration changes do not refresh the earlier runtime observations.

## Observed software results

| Check | Developer host | NetBSD 11 ARM64 VM |
| --- | --- | --- |
| Current combined provider suites | 357 reported passes, including docs | 319 non-doc + 38 doc = 357 passes |
| Current own-home client suites | 265 passes, eight existing helper ignores | 266 passes, eight existing helper ignores |
| Classic and Unicode maximum file | Both passed | Both passed at 1 GiB / two CPUs |

The host's pending-TCP platform preconditions were unavailable in two provider
checks; reported harness passes do not establish those runtime observations.
The native client count includes its platform-specific actual pending-TCP case.
Earlier feature-matrix results retain their original source/configuration boundary
in [carrier components](carrier-host-components.json); counts overlap.

Each maximum-file journey used 1,048,576 plaintext bytes and 1,048,592 ciphertext
bytes, with 65 upload and 65 fetch calls. Source removal followed canonical
confirmed progress. The recipient was offline at send, later accepted encrypted
mail, fetched through its own home, verified exact bytes and explicitly saved a
new private 0600 file; overwrite was refused. Host and native timings and original
limits are in [host results](results.json) and [native results](native-results.json).
Two independently configured provider homes had separate keys, policy, runtime
and stores. One opaque test delivery/archive queue supplied encrypted mail;
this is not deployed XMPP server-to-server federation.

Compilation and runtime are separate: after retained 1 GiB compiler failures,
the native client and test ELFs were built serially at 2 GiB. The fixture was
restored and independently observed at 1 GiB before qualification. Original SSH
readiness failures, the client temporary-socket-path failure and its setup-only
short-directory correction remain in the native record. No memory, deadline,
request, chunk or output cap was raised to pass runtime checks.

## Recovery and independence

The interrupted-upload episode removed the source, stopped and reopened the same
policy, resumed persisted ciphertext through fresh authorization, committed and
delivered encrypted mail. Its later fetch halted unexpectedly at 32 KiB; that
episode remains failed and its cause is unestablished. One separate explicit
manual continuation resumed the retained partial fetch, verified 131,072 plaintext
bytes and completed private save/no-overwrite checks. This is not an uninterrupted
episode or an explanation/fix for the earlier halt. Exact fresh Commit retries
are tested; a precisely lost Commit wire reply was not observed.

Ordinary same-home encrypted text passed with the optional providers stopped.
The [native outage proof](bbs-outage.json) passed eight controls: independent
registration and sysop approval, preserved encrypted mail/cursor/pending state,
real restricted SFTP upload/review/download, exact 131,328-byte comparison and
terminal/privacy checks. The optional provider stayed stopped before, between
and after the journey. The selected profile was deliberately lazy and unused.

[Diagnostic recovery](diagnostic-recovery.json) preserves the failed preflight,
platform-format correction and later guarded manual startup. No failed setup is
rewritten as an uninterrupted pass. The final temporary snapshot, owned VM,
controller and forward are closed; 86 saved owner assets are unchanged. A prior
failed base image is intentionally retained privately for diagnosis and replay.
A consistent logical SQLite backup does not preserve original WAL byte identity.

## Privacy, retention and verification

Actual selected provider stores, logs and related fixture files were checked for
selected plaintext, names, device-derived descriptor keys and passwords. Hidden
exact token values were unavailable; token field-pattern checks are narrower.
See [privacy and retention](privacy-retention.json), [core TDD](core-tdd.json) and
[host TDD](tdd-host-results.json). Actual failures and unavailable beforeimages
remain explicit; private-corpus hashes do not imply public replay or a security
audit. Source, binary and raw-result identities stay attached to their observations.

Private keys, vaults, payloads, private endpoints, binaries and raw fixture corpora
are retained privately and are not published here. Home workers receive no member
vault, OMEMO private key or content key through these interfaces. Public identity,
routing, size and timing metadata remain visible. Trusted-home compromise,
retention/lost-key and recipient-copy limits remain in the
[member guide](../../../docs/private-attachments.md).

[Reproduction](REPRODUCE.md) describes the maintained commands and resource phases.
The manifest checker validates saved bytes and exact folder membership; it does
not rerun Rust, a VM, cryptography or a live service. This software preview supplies
no deployed provider, independent-client interoperability, physical SBC,
250-member support, hardening or security-audit claim. Earlier evidence is unchanged.
