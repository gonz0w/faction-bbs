# Local directory submission software evidence

[Issue #184](https://github.com/gonz0w/faction-bbs/issues/184) adds the
[submission desk](../../../docs/other-boards.md) and schema-19 local overlay.
[The immutable result](results.json) binds source stages, original structured
observations and teardown. The owner-managed curator pack is unchanged.

Host and native NetBSD each passed 411 Rust workspace tests, including 15 core
lifecycle/recovery tests, six shared-desk tests and four overlay tests. Strict
native formatting and Clippy pass. Both platforms passed 14 new real local/worker
PTY controls, 11 existing directory controls and existing owner export/launcher
checks. Each passed 21 complete-backup controls; NetBSD used distinct non-root
board/file UIDs and actual owner CLI execution.

Six actual restricted SSH/native-client controls cover cancellation, submission
without publication, exact sysop approval, expiry/removal, plain access and removal
while another caller holds a compact card. The old selection key is consumed;
a changed directory cannot silently substitute another endpoint. Tests keep normal
eight-session/5000-ms admission and existing output/helper budgets.

## Execution and retained failures

NetBSD 11 ARM64 used Rust/Cargo 1.96, two cores and 2 GiB for compilation and the
unit suite. The same owned overlay rebooted with 1 GiB for runtime checks; observed
physical memory was 1,054,990,336 bytes. The host used Rust/Cargo 1.98. This is
software validation, with no physical SBC, capacity, audit or hardening claim.
All 705 inputs in the final native capture matched uploaded bytes. Original
source objects and executable hashes survive; raw binaries do not.

The old preserved #220 binary passed login/directory entry but failed the new S
entry PTY. Core and presentation missing-interface tests ran red before behavior;
private-draft visibility and configuration ordering have additional observed red
runs. Agent archives preserve candidate source and setup/compile failures separately.
The first full native test capture omitted existing gallery/library assets: nine
gallery tests failed on missing files, then the same unweakened suite passed with
those original assets. Product source did not change for this correction.

Lead harness corrections retain an empty expectation, a wrapped compact notice,
the shared welcome continuation and an already-running test SSH daemon. A first
native distinct-UID backup fixture could not traverse the private build directory;
using its root-installed executables passed. None is relabeled as a product RED.
The actual Git CRLF byte test failed before the new evidence-folder attributes and
passed after them. Source-capture stages include unexecuted captures as such;
their presence does not establish a successful runtime observation.

## Locality and verification

Only public project/driver sources and fixed-schema counts, checks and digests are
exported. Raw terminal logs, settings, credentials, keys, board state, Cargo caches
and overlays stay private and are removed by the recorded owned finalizer. Public
fixtures intentionally contain synthetic/example values. The marker scan covers
available generated settings and key blocks; it does not claim to recover every
short-lived PTY password. Those harnesses separately assert no password echo and
export only fixed check names/counts. No private content or endpoint is published.

The teardown record verifies stopped services, closed forwards, removed owned
fixtures and unchanged full saved-owner disk/kernel hashes. The saved playground
remains stopped. Earlier directory, launcher, Who, unread and inbox records retain
their whole original hashes. No historical measurement is refreshed to schema 19.

Run `python3 tools/check_directory_submissions_evidence.py` to validate the saved
record and source bytes. It does not start NetBSD. Required PR CI reruns current
workspace/feature/repository tests separately from these saved observations.

The later [CI schema-oracle correction](ci-schema-oracles.md) retains two old
expectations and their host red/green checks separately. It changes no product
code or original native observation.
