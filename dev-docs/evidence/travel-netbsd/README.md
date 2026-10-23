# Native travel software validation

Issue #159 (first interactive increment), [plan 0069](../../plans/0069-native-travel.md),
[ADR 0064](../../decisions/0064-native-travel-preferences.md).

The final result uses host Rust 1.98.0 and an owned separate NetBSD 11.0 AArch64
software VM, two cores/1 GiB, Rust 1.96.0. Physical devices are unnecessary for
this software batch. The saved owner playground remains stopped and preserved.

A clean offline native workspace and host tests pass 304 Rust tests per OS. Six
local travel PTYs per OS cover 40x18 layout, explicit apply/cancel, preserved board
highlight, invalid/atomic limits, CLI preferences, Ctrl-C, contradictory options,
no early connection and no profile/key/vault/history writes. An initial test
assertion inspected already-consumed prompt bytes; corrected final runs use the
actual returned highlight. No runtime behavior was changed to satisfy that test.

Actual native SSH/SFTP results, existing recovery/vault/terminal regressions and
exact execution source/binary pins are recorded in `results.json`. Ordinary and
compact visits preserve the existing local authorization and room content. Exact
payload-size ceilings concern files, not encrypted wire cost or link throughput.
No offline exchange, route change, new helper/listener or server-default change.
Schema remains 14. Future named link profiles and packet/reply work remain #159/#145.

Snapshot 36, [gallery baseline](../gallery-baseline-manifest.json), freezes 277
preceding actual inputs and the whole #166 gallery result. Keep 49 old evidence
identities/guards fixed. Normal eight-session/5000-ms admission, two credential
permits, three owner helpers, 32-KiB output/five-second drain remain unchanged.

Final validation passes **130 Python tests**, **nine actual SSH/native/SFTP
controls**, **17 existing recovery controls per OS** (native distinct-UID CLI),
full host Rust/terminal CI and prior community/editor/art/vault/picker regressions.
Exactly-at-ceiling 50-byte original synthetic payloads upload/download via the real
restricted NetBSD SFTP service with matching checksum and pending review. Over-limit
and zero budgets create no file grants/reservations; cancel creates no destination.
Private file access, existing complete-read acknowledgment and pinned host mismatch
remain enforced. Explicit compact-native advertisement works while ordinary dumb SSH
Files stays escape-free. This is not a throughput test or wire-cost measurement.

`results.json` pins **272 matched native execution inputs**, **29 host harness/
review inputs**, **thirteen unchanged dependency/access/credential inputs** and
built/installed binary hashes. Final service/CLI compact-hint changes are intentional;
the client literal is checked on the native built binary. The snapshot's original
Markdown remains exact; only historical link resolution is excluded from current-doc
checks, with no-JS/newline/conflict/hash checks retained and a negative test. The
owned clone/cache/test credentials are removed and saved owner playground remains
off/preserved. New offline guard 50 preserves 36 snapshots/49 prior results.
