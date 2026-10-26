# Manual roaming owner CLI software evidence

Issue [#228](https://github.com/gonz0w/faction-bbs/issues/228), 2026-10-26.
The [immutable record](results.json) covers the filesystem-owner commands in the
[operator guide](../../../docs/roaming-authentication.md). It does not install a
home issuer, provision transport accounts or establish production roaming.

## Observed behavior

| Layer | Actual result |
| --- | --- |
| Host CLI | 18 tests: eight existing units, one metadata-display unit and nine owner integrations. |
| Host core | All 206 tests pass, including three existing-state/WAL tests and 15 roaming controls. Strict core/CLI Clippy and formatting pass. |
| Native build and tests | NetBSD11 ARM64, Rust1.96.0; 18 CLI tests and 28 focused core tests (three existing-state, 15 roaming, ten snapshot). Compiler/tests use two cores and 2 GiB. |
| Native runtime | 14 controls at two cores and 1 GiB: local ID lookup, explicit association/consent, owner PTY, private export/root install, member-command denial, current service startup, kernel UID/owned TTY entry, live CLI revocation, disabled/stale/revoked/restored rejection and default-off classic entry. |
| Native classic and recovery | Real console and export/restore PTYs plus 21 complete-bundle recovery controls with distinct owner/file UIDs. |
| Host regressions | Real local ANSI console and inherited member-worker PTYs. A 381-test Python suite passed before the later guard extension; all five final focused evidence-guard tests pass separately. |

Native compilation took 14.250 seconds. The complete CLI test command took
12.018 seconds; focused core tests took 10.522 seconds. Runtime controls took
1.180 seconds; classic, export/restore and complete recovery commands took
1.145, 0.363 and 0.673 seconds. A service-owner RSS sample was 5908 KiB.
These are small local software-fixture observations, not whole-host memory,
throughput, connected-member capacity, WAN latency or physical SBC measurements.
The runtime reported 1,054,990,336 bytes of physical memory; compiler/tests reported
2,128,732,160 bytes. These scopes must remain separate.

Normal eight sessions/5000-ms launch admission, two credential permits/kernel
slots, three owner helpers and five-second drain remain unchanged. The optional
roaming caps remain two issuers, four enabled bindings and 32 immutable UID
records. Owner inspection retains its 32768-byte output cap; the strict service
manifest remains at most four entries and 8192 bytes.

## Test-first corrections and source identity

Actual RED precedes the missing owner interface, maximal quoted/backslash metadata,
WAL-effective schema handling and local member-ID lookup. The final implementation
uses bounded graphical ASCII fields without expanding valid metadata, and a narrow
core opener reads effective SQLite app/schema state without initialization or
migration. Tests retain approval, generations, UID history, private new-only output,
failed writes, scoped live revocation and independent password authority checks.
The evidence checker has separate missing-interface and missing/failed-control RED.

The 15 source stages retain exact executed host test variants, three native
contexts and named orchestration sources. The final **725-input source context**
was independently hashed inside the guest. It includes tools/assets beyond the
commands actually executed; it is not a claim that every file ran on NetBSD.
The 998 content references reuse prior immutable objects where bytes match.
Original manifests remain unchanged. Eight incidental Finder `.DS_Store` blobs
from broad host directory copies are explicitly excluded from publication; they
are not executable inputs and are absent from the native context.

Two supplemental independent reviewer probes ran inline. Their original tool-call
history, result facts and before/after source hashes remain, but standalone scripts
were not saved before execution. Captured executable regression RED files are the
primary reproducible evidence; no retrospective script reconstruction is claimed.

The first native context predates the two review corrections; the second predates
the read-only local-ID command. Their observations are preserved, not relabeled as
final execution. Retained setup failures distinguish a classic-controller input
mistake, wrong ownership of the disposable theme copy and a missing collector-log
assumption. Fixture fingerprint/key reuse mistakes in early host tests also remain
visible. Assertions, admission limits and password costs were not weakened.

## Boundaries and cleanup

The service proof uses observed kernel UID and owned TTY descriptors. It does not
repeat certificate minting, CA qualification or SSH-wire trials from earlier
issues. Local lookup does not prove home identity, grant approval or import roles.
Root independently reviews/installs transport policy; exported metadata cannot
validate its accounts, CA/principal policy or reserved UIDs. Mail keys/plaintext
remain on member devices. No new dependency, schema, member RPC or resident helper
was added.

The owned VM/controller stopped, its overlay and four loopback forwards were
removed, and all marked private source/cache/board fixtures were removed. Full
saved disk/kernel hashes and saved metadata/sizes/timestamps remained unchanged.
The owner's saved playground stayed stopped and intact. No physical hardware,
production deployment, online enrollment/issuance or new capacity support is claimed.

Validate offline with:

```sh
python3 tools/check_roaming_owner_evidence.py
python3 -m unittest discover -s tools -p test_roaming_owner_evidence.py
python3 tools/check_evidence_sources.py
python3 tools/check_repository.py
```
