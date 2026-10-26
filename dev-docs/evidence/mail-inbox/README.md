# Selectable local inbox software observations

[Issue #219](https://github.com/gonz0w/faction-bbs/issues/219) adds the
[local inbox](../../../docs/local-mail-inbox.md) to the existing unlocked mail
console. [The result](results.json) binds the actual source stages, fixed records,
platform observations and final cleanup. This is a finite child of #99.

Host and native NetBSD each passed 13 offline and 14 configured-online inbox PTY
controls, including classic/Unicode display. They cover empty/single/seven/full
64-record histories, six-row pages, arrow/plain keys, unread/unknown/sent states,
exact selected-message output, explicit durable acknowledgement, typeahead,
process death and canonical reopen. New model and real EOF/output-handler tests
retain the original checkpoint, unread and terminal-restoration assertions.

Each OS also ran classic and Unicode real SSH visits. A source-preserved variant
of the original 14-control journey replaces its selected history interaction with
I inbox navigation. All original approval, outage, pending/cursor/history,
restricted-SFTP, hostile-output, disconnect and private-marker assertions remain.
Three observed visits to original record 2 skip, acknowledge, then skip it; the
same client/SSH survives until deliberate disconnect. This is not arbitrary
interception of board drafts, games or transfers.

## Platforms and execution boundaries

NetBSD 11 ARM64 used Rust/Cargo 1.96, two cores and 2 GiB for compilation and unit
checks. The same owned overlay then rebooted with 1 GiB for runtime controls;
observed physical memory was 1,054,990,336 bytes. No fresh 1 GiB build is claimed.
Host versions are recorded separately: Rust/Cargo 1.98 and Python 3.14.7. The
project's selected Rust minimum and NetBSD target remain unchanged.

Both OSes passed native-only client graphs (85 outer tests) and network client
graphs (132), plus 32 separate native-crypto/storage regressions. Outer counts do
not double-count nested helpers. Three existing library helpers and one new
binary helper are intentionally ignored in the outer harness and exercised by
normal parent controls. Both strict Clippy graphs pass. The Python tool suite is checked again after
publication and in required PR CI; those final checks are separate gates.

## Retained failures and source identities

The actual old-client terminal test unlocked and failed specifically on the
missing I entry. Missing model APIs and a first-page navigation defect ran red
before fixes. Rejected fixture timestamps/forbidden control content, the compact
separator failure and tooling/publication corrections retain distinct sources.

The first native candidate exposed transient PENDIN terminal state. A narrowly
observed Darwin/NetBSD mask leaves every other flag, speed and control-character
assertion intact. The second candidate exposed the new backend fixture lowering
its shared test process's core limit, affecting an existing later guard. That
old guard passed alone on the same artifact. The final candidate isolates only
the new fixture in a subprocess; its full assertion body and the old guard remain
unchanged. Direct helper execution ran exactly one test and passed.

Each final native qualified tree has 695 captured public inputs; the unchanged
board is built from 693 inputs at base 60fecae. The final test-only correction is
an explicit one-file override of the previous frozen tree. A broader unexecuted
capture that also included a later Git-byte test change is not runtime evidence.
The result separately identifies old host/native candidates and exact executed
orchestration. Full original #216/#217 records keep their exact digests.

All exported fixed records preserve their original bytes as hash objects. A
native configuration observation without a final newline is published only as
its original object; it has no rewritten JSON mirror. Other maintained JSON views
are exact copies. Raw guest binaries were removed with the overlay; artifact
hashes and executed Rust/C/Cargo/Python sources remain available.

## Privacy, cleanup and verification

Only public source and fixed-schema counts/checks/digests are published. Four
actual private marker inventories include generated credentials, private message
markers, opaque state and both generated SSH key blocks. Literal/JSON-escaped
marker scans found zero matches. Known already-public lab constants are excluded
with their reason recorded. Raw settings, marker values, keys, mail stores and
terminal logs are removed with the owned fixtures. A marker test is a bounded
software observation, not an independent security audit.

The VM teardown record describes that earlier stage precisely: services,
controller, QEMU, overlay and all three forwards were removed while private host
roots remained for review. The final result separately records their removal.
Saved owner metadata and full disk/kernel hashes remain unchanged; the saved
playground stayed stopped. No normal service limit was reset or tuned.

Run `python3 tools/check_mail_inbox_evidence.py` to validate saved observations,
exact source/record bytes and cleanup. It does not start NetBSD. Background
notifications, credential leases, live private chat, roaming and attachments
remain separately scoped work under their GitHub issues.
