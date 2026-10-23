# Personal catch-up: NetBSD software validation

Issue #138, [plan 0066](../../plans/0066-personal-catchup.md),
[ADR 0061](../../decisions/0061-personal-catchup.md).

Owned synthetic boards run on host Rust 1.98.0 and a separate NetBSD 11.0 AArch64
software VM (two cores/1 GiB, Rust 1.96.0). The saved owner's playground stays
stopped. Clean native workspace and host builds pass **280 Rust tests per OS**.
**129 Python tests** pass. Eight local/worker catch-up controls per OS cover optional policy/name/notice,
preview/save, no automatic login tracking, own favorites, explicit checkpoints,
existing room-read jumps, unavailable Files, plain/compact views and pending denial.

Eleven actual restricted SSH/host-native/NetBSD-native/plain controls validate no
preview unread acknowledgment, deliberate checkpoint cancel/save, own favorites,
direct replies from another member, partial versus complete drained room reading,
concurrent preference/checkpoint conflict, Busy explicit retry, resized catch-up,
a current gathering-detail jump, revoked private-file access after preview, public
file information/reference and active member revocation before checkpoint save.
This run checks catalog integration, not a fresh SFTP byte-transfer claim.

Core checks cover 100+ unread caps, bounded room pages, favorite quotas and stale
revisions, publication after the captured window (including an upload pending
before it opened), monotonic watermark after deleting the highest publication,
file ACL/hide/removal, current gathering visibility, disabled modules, Busy readers
and exact schema-12 export/restore/migration without invented dates. Existing
unread remains the only post ledger. Snapshot negative controls exposed the old
65-row schema truncation; the bounded exact comparison now rejects extra views/
triggers and catalogs above the explicit 128-object budget.

Complete recovery passes **16 controls per OS**, including actual native CLI runs
with distinct non-root owner/file UIDs. Policy, own preferences/favorites, saved
visit and ordered publications survive alongside prior local community/file data.
Existing social/editor/browser/menu/operator/art and local encrypted vault/picker
regressions pass. No mail decryption, automatic visit telemetry or new capacity
measurement is introduced. Normal eight sessions/5000-ms admission, two credential
permits, three helpers, <=8-KiB frames, 32-KiB queue/five-second drain stay unchanged.

`results.json` pins **240 matched native execution inputs**, **21 host harnesses**,
installed binary hashes and unchanged dependency/access/credential inputs. The
new service visit-save literal is checked in the installed binary. Snapshot 33
preserves 238 preceding inputs and the whole poll result before edits. Historical
hashes/records remain fixed. `python3 tools/check_catchup_evidence.py` verifies the
new saved record alongside the 46 earlier guards.

The owned disposable VM, test credentials and temporary vendor are removed after
validation; saved playground disk/login/keys remain stopped and preserved. This is
software evidence, not physical SBC/250-member capacity, production, federation,
network mail, roaming authentication, real Ghostty pixels or hardening acceptance.
Schema 13 needs a stopped/drained complete backup and matching-binary fresh-path
rollback. Exact schema-2..12 exports remain read-only verifiable/restorable.
