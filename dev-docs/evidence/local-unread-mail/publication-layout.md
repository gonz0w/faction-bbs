# Original JSON bytes and maintained mirrors

The first complete software-observation record is preserved unchanged as
[332bf000…](source-objects/332bf000f982b81a4fc4b4add952fe0b95d085a4e03caeec65d06d15b8383086).
Its exact original qualification outputs included 28 valid JSON files without a
final newline. The repository requires final newlines in maintained JSON files.
That general check remains unchanged.

The [byte map](publication-byte-map.json) binds each original logical filename to
its exact archived object/hash and its maintained JSON-view hash. Each view adds
exactly one final newline. No other byte changes, parsed data changes or refreshed
measurement fingerprints are permitted. Original exported manifest fingerprints
refer to the archived raw bytes, not the maintained views.

The [current record](results.json) adds only publication provenance and the new
resolver's executed source/test stages. Software behavior, source/artifact
identities, test outcomes, resource observations and teardown facts are unchanged.
The validator resolves and verifies the entire original record and all three
original exported manifests through the byte map as well as checking the current
record. Three actual red-to-green tests reject substitution, changed data,
whitespace beyond the single newline and path escape. No directory-wide policy
exception or repeated native/software measurement was used for this repair.
