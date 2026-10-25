# Native OMEMO candidate: host observations

[results.md](results.md) records issue #199's actual disposable host build and
unchanged upstream test execution. The [research report](../../research/0053-native-omemo-reuse.md)
explains decisions, source contracts, reproduction and limits. The result is a
candidate for further qualification, not adopted product cryptography.

The record pins Picomemo's source archive and inspected files, comparison-source
hashes, reference wheels, exact commands/outputs and host artifact sizes. No
third-party source, keys, synthetic private store, wheel or binary is redistributed.
Commands use `<owned-fixture>` in place of the discarded private temporary path.
Pip's unused browser bootstrap asset was excluded before the final runtime test;
no JS execution or product Python dependency is selected.

Tests include a Twomemo-origin first-message decrypt, not independently verified
bidirectional replies, SCE correctness or Faction device-trust enrollment. The
native client, vault, signer and server remain unchanged. There is no new NetBSD,
physical SBC, whole-client RAM, capacity, cryptographic audit or production claim.
