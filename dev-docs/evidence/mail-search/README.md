# Local search of retained encrypted mail

Issue [#242](https://github.com/gonz0w/faction-bbs/issues/242) adds literal local
search to the unlocked global and correspondent inboxes. The
[member instructions](../../../docs/local-mail-inbox.md) describe its controls;
the [technical contract](../../local-mail-inbox.md) describes matching, query
lifetime and original-record selection.

The selected [results](results.json) and [file manifest](manifest.json) record
software observations and their source identities. The generic
[recorded-file checker](../../../tools/check_recorded_files.py) verifies pinned
file hashes and exact selected-folder membership. It does not rerun native
controls or independently prove privacy. Separate review and the actual
controls and source tests provide the evidence for those conclusions.

## Observations

Host and NetBSD 11 ARM64 controls exercised classic and Unicode local search:
two runs on each system, with seven controls per run. Two same-SSH search
runs per system each passed four controls. Existing inbox, correspondent and
reply controls in both display modes, Files controls and default-client controls
also passed on both systems. Host default, native and network tests, the focused
probe tests and integrity-checker tests passed. These are software observations,
not physical hardware, capacity, graphical or production qualification.

Each of the four final same-SSH originals received a separate audit of its source,
binary, inputs, markers, capture, result and strict GIO reconstruction. The
only deliberate canonical read-bit change was canonical history index 4
(displayed record 5). Pure-search ciphertext
preservation is an assertion in the executed source, supported by inspection of
the logical before and after state. Board database, store and log absence is the
source-bound original scan observation; it is not a fresh scan of a historical
filesystem. Native binary identities come from the preserved native hash receipt.

The selected native safe files were scanned against 333 actual generated private
values with zero hits. Fixed public synthetic query literals are identified by
source provenance separately from generated private values; the board I/O
checks cover both classes. The recorded VM and controller stopped, four forwards
closed, and the six saved owner files retained their hash, size and modification
time. Cleanup of private owned roots is recorded separately.

## Preservation and limits

Selected source variants, failed runs and observations preserve their executed
identities. Pre-implementation controls observed missing S in a real PTY and
missing model APIs. Later actual REDs exposed a 40-cell hint that exceeded the 39-cell limit and
search scratch reallocation; the corrected source retains those limits. A probe oracle initially
rejected a 43-cell prelude; its correction scopes the width check to the last
inbox after the observed failure. Setup, configuration and admission failures
are recorded separately from behavioral failures. An abandoned, unexecuted
shared-marker candidate is not qualified by a later RED; the final publication-only inventory
API had its own actual RED.

The existing evidence records and registry remain unchanged. Search adds no
persistent query or plaintext index, background refresh, credential lease,
protocol or cryptographic change. Broader communications remain under
[issue #99](https://github.com/gonz0w/faction-bbs/issues/99).
