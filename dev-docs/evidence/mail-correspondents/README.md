# Retained encrypted-mail correspondents

Issue [#235](https://github.com/gonz0w/faction-bbs/issues/235) adds C inside the local
inbox. It groups retained records by exact stored account/device and reuses the
existing reader, explicit acknowledgement and current-peer reply checks. See the
[member instructions](../../../docs/local-mail-inbox.md) and
[technical contract](../../local-mail-inbox.md).

[results.json](results.json) binds actual software observations and executed inputs.
Run `python3 tools/check_correspondents_evidence.py` from the repository root.
It checks immutable bytes and recorded contracts, without rerunning private VMs.
The [earlier reply observations](../inbox-reply/results.json) remain unchanged.

## Observed results

| Layer | Actual result |
| --- | --- |
| Host workspace release tests | 473 passed, no ignored; formatting and strict all-target Clippy passed |
| Optional client | Default 42 passed; native 95 passed / 5 helper ignores; network 144 / 5 |
| NetBSD selected native tests | 277 passed / 5 helper ignores |
| Final Python tools | 436 passed, including 10 evidence-guard and 3 oracle tests |
| Local correspondent PTYs | 8 controls each, host/NetBSD × classic/Unicode |
| Same-SSH main-menu picker | 4 controls each, host/NetBSD × classic/Unicode |
| Existing global inbox | 14 controls on each OS, including its existing rendering controls |
| Existing replies | 11 controls in each display mode on each OS |
| Files/SFTP | 14 controls on each OS |
| Actual older/default native-v1 | 2 controls on each OS |

The five ignored Rust entries are subprocess helpers invoked by substantive tests:
four baseline helpers and the new correspondent interface helper. They are not
five unexecuted feature tests. The 702-input native build used two cores, two
compiler jobs and 2 GiB; runtime used two cores and 1,054,990,336 physical bytes.
Twelve frozen Python runtime inputs were separately matched to the guest. The
compiled production inputs stayed unchanged through final checks. Default and
network binaries have separate targets and identities.

Actual encrypted history interleaves accounts, devices, received and sent records.
Controls cover newest-retained ordering, six-row pages up to 64 groups, original
indices, group-scoped U/A, unknown read state, nested Back and source preservation
after reply. The last unread acknowledgement leaves that group empty. Navigation
is read-only, and current-peer reply review/refusal remains unchanged; grouping
adds no historical-key continuity or contact verification.

Four unique capture IDs bind pre-generated marker sets, trace start, original
trace bytes, scan requests and route results. Separate review independently
replayed strict per-PID/fd/direction GIO reconstruction, positive park/back controls
and canonical inspections. Actual private peer/subject/body/identity markers were
covered and absent from reconstructed board I/O. Canonical main-menu state changed
only at the explicitly acknowledged received record, index 4; reply cancellation
preserved pending intents and archive state. Extracted native captures match their
original private archive byte-for-byte.

Board database/store/log absence is the original source-bound scan observation
with exact marker/request binding. It is not an independently repeated scan of a
historical filesystem snapshot. Same-SSH PID and actual TCP-outage assertions are
recorded runtime controls with supporting journey/park/back observations; they are
not a packet-trace or physical-hardware qualification.

## Preservation and rejected candidates

The native/client/lead/lead-final selections retain their exact manifests, flat
source-stage maps, selected logs and source variants. SHA-256 reuse references
resolve unchanged committed objects in the preceding reply evidence instead of
copying another complete source tree. Raw recorded metadata stays unchanged even
where it names an original private scratch path or an earlier pending scan. These
are historical execution inputs, not current instructions or work tracking.
Private vaults, configurations, keys, terminal captures and raw private traces are
excluded. Review scripts and safe verdicts are separately selected.

Actual pre-implementation RED includes the C+Back interface assertion, missing
model interfaces, missing probe module, missing evidence module and absent live-CI
registration. A later evidence-quality RED coherently re-bound the mutated result
to its capture association and showed that a false canonical-acknowledgement flag
was accepted. The final guard rejects it; all 10 corrected guard tests pass.
The earlier 435-tool / 9-guard run remains immutable and distinct from final 436/10.

Setup and rejected candidates are recorded separately: sandbox/zero-test setup,
an E0502 borrow-check failure, fixture archive-before-enrollment, owned-VM process
and directory preparation, cache-path/guard refusals, offline registry metadata
completion and an owned Files-settings UID guard refusal. Corrections retained
assertions and normal limits; failed controls did not become behavioural RED.
An initial reviewer audit used a wrong unlock-prompt literal; both its source and
failure survive beside the corrected audit using the actual frozen constant.

The publication scanner's first poison fixture was a complete private PEM, which
correctly activated both value and key detectors and failed its one-hit assertion.
A single-line private value isolates the intended one-hit check. Later scanner
setup attempts assumed the wrong client manifest filename; their sources/logs are
retained. Successful scans retain membership/hash assertions and reject the poison
control. Final publication verdicts cover 467 actual generated private values,
with zero matches; only known public synthetic fixture literals are excluded.

Requested implementer/reviewer model settings are recorded separately from
unavailable observed runtime metadata. No runtime-routing or cost-saving claim
is made. Final cleanup records bind identity-checked owned VM/controller stop,
four closed forwards, unchanged hashes/sizes/mtimes of six saved playground files,
and removal of the four marked private roots after proof copying and review.
The owner playground remained stopped and intact.

These observations add no new protocol, schema, dependency, crypto, background
polling, credential lease, automatic send/read acknowledgement or production
support. Physical SBCs, 250-member capacity and graphical acceptance are outside
this proof. Broader communications #99, attachments #88 and deferred #231 retain
their separate issue scopes.
