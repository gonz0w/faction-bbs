# Durable manual archive observations

Issue [#211](https://github.com/gonz0w/faction-bbs/issues/211) adds one explicitly
requested own-home archive page to the optional native mail client. The
[member guide](../../../docs/archive-mail.md),
[technical contract](../../native-mail-archive.md) and
[ADR 0081](../../decisions/0081-durable-mail-archive.md) define its limits.

The source ledgers preserve exact executed inputs. Objects use SHA-256 names;
each ledger retains the original file name and verifies the object bytes.
They are historical inputs, not current documentation or executable assignments.
The pre-211 [baseline manifest](../home-mail-archive-baseline-manifest.json)
preserves 295 earlier inputs and four whole records. Earlier measurements and
result digests have not been refreshed to describe this change.

## Interpretation

The selected profile requests at most four MAM2/RSM results. An explicit local
schema upgrade adds an encrypted home/device/epoch-scoped cursor. An accepted
message, native state and its cursor publish atomically. Recognized duplicates and
own sent copies do not create another message or retire pending ciphertext.
Archive-enabled sending omits presence and live receive; receiving uses the
separate deliberate page action. The server's IDs, delay, ordering, count and
completion flags do not authenticate content or establish complete history.

The NetBSD build uses a two-core, 2 GiB disposable compiler VM. Some dependencies
were already cached; elapsed build phases are not a fresh complete-build memory
claim. The same overlay is rebooted with 1 GiB before runtime tests. Runtime
observations exclude a compiler-capacity or supported-user-count conclusion.

Deterministic Rust subprocess controls exercise before/after-publication failure
for incoming messages, duplicates and own sent copies. A separate two-home test
kills only its own live catch-up child after observing canonical publication.
That observation does not identify an exact fsync instruction boundary.

Only generated fixture identities and private markers are used. Public records
contain control names, hashes and counts, not keys, passwords, bodies, opaque
cursor values or private terminal transcripts. Server scans inspect actual home
data and original/restarted logs for the private-content marker. They support
that bounded observation, not a security audit or proof that every possible
plaintext leak is absent.

Home operators may omit, expire, reorder or truncate retained messages. There is
no complete-history, read-receipt, background-sync, roaming, attachment, physical
hardware, full-conformance or capacity claim. This optional member-device work
does not change ordinary BBS resource limits or enable server-side decryption.

## Recorded checks

The digest-pinned [result](results.json) records 139 client/OMEMO Rust tests per
OS, with two intentional subprocess helpers invoked by their parent controls.
Host and native runs passed the offline subject/body/reply, page continuation,
reopen and pending-ID checks. Each also passed four real live-SIGKILL/resume
controls, six archive PTYs and the existing home/checkpoint/vault/picker flows.
The native transport passed 25 Rust and 20 TLS controls. The host Python suite
passed 286 tests; the ten evidence/CI guard tests passed again after correcting
the measured-cost schema. Required PR CI is separate from these saved observations.

The actual native [empty-page UI cycle](netbsd/native-manual-archive-ui-cost.json)
peaked at **70,748 KiB RSS**, with 0.35 seconds wall, 0.07 user and 0.01 system.
It includes checkpoint unlock and one verified TLS/account/empty-MAM-page cycle;
it excludes pending sends, compiler and server fixtures. It does not measure a
nonempty page's receive/decrypt cost. The raw cost object is unchanged. Its NetBSD
platform context in the aggregate comes from the separately saved runtime record.

The first native polling observer attempt remains
[failed](netbsd/native-first-observer-failure-summary.json). Read-only inspection
found all three new messages durable and pending IDs intact, but the exact child
exit was not retained. A fresh profile and a tested NetBSD kqueue directory
observer passed the separate live-kill control in one attempt. Directory events
only wake the observer; an actual changed canonical inode, live child and SIGKILL
exit are still required. Rust product bytes and time bounds did not change.

Rejected TLS fixture profiles and the initial partial compiler candidate are
preserved separately in the source/build ledgers. The successful two-home fixture
uses the previously qualified RSA2048/SAN/serverAuth profile without relaxing
trust, name or pin checks. Read-only retention barriers ensure that deliberately
offline sends have reached the test home's archive before exact page assertions;
the barrier is a measured fixture input, not a client loop or delivery receipt.

Separate post-recovery server scans found the private-content marker absent from
87 host-observation files and 105 native-observation files, covering both data
trees and original/restarted logs. The [teardown proof](teardown.json) confirms
that the marked owned VM and generated private fixture inputs were removed,
loopback listeners closed, and the saved owner playground stayed stopped and
unchanged. No private actions, credentials or raw terminal transcripts are public.

Run `python3 tools/check_archive_mail_evidence.py` to verify the raw result digest,
source stages, actual controls, separate audit sources and teardown. This verifies
saved evidence; it does not repeat the software fixture or establish new support.
