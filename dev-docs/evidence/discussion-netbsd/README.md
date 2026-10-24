# Discussion discovery: NetBSD software validation

Issue #142, [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/142),
[ADR 0062](../../decisions/0062-discussion-search-and-bookmarks.md).

Owned synthetic boards use host Rust 1.98.0 and a separate NetBSD 11.0 AArch64
software VM (two cores/1 GiB, Rust 1.96.0). The saved owner's playground remains
stopped. Final clean native workspace and host builds pass **292 Rust tests per
OS**, plus **129 Python tests**. Eight local/worker Find PTYs per OS cover optional
named policy, literal subject/body/room/author filters, Unicode input/classic
presentation, explicit bookmark cancel/repeated save/remove/export, compact exact
source paging, no unread changes, disabled paths and pending denial.

Eleven actual restricted SSH/host-native/NetBSD-native/plain controls cover actual
post composition, Unicode filters, exact read-only full sources, separate members'
bookmarks, copyable export, Busy-review retry, resize of completed query/source
state, private-file projection before matching/snippets/bookmark/source rendering,
revoked grants, deletion while viewing, cascade cleanup, active member revocation
and disabled policy. A normal complete room-read/drain still advances unread while
all Find activity leaves it unchanged.

Core tests cover indexed bounded/sparse scans with no skipped older hit, opening
high-water behavior, literal SQL-like input, Unicode matching, exact author/room
filters, 128-bookmark quota/idempotency, no foreign bookmark access, stale source
removal, current file projection, lossless hex-subject reference export, Busy writes,
disabled policy and exact schema-13 export/restore/migration. Source viewers reuse
the inert Message presenter and re-fetch exact current content. The 20-ms iteration
guard is cooperative, not a new hard filesystem/SQLite latency claim.

Complete recovery passes **17 controls per OS**, including actual native CLI use
with distinct non-root owner/file UIDs. Find policy and own reference identities
survive alongside prior local community/file data. Existing social/editor/browser/
reader/menu/operator/art and local vault/picker regressions pass; full host terminal
CI commands and unchanged reviewed dependency inventories pass. Two older smoke
assertions still expected schema 13; the corrected final clean native run expects
14 and passes. Product startup/permissions were not changed for the fixtures.

`results.json` pins **250 matched native execution inputs**, **23 host harness
inputs**, fifteen unchanged dependency/access/credential inputs and built/installed
binary hashes. The installed service's new bookmark-save literal is checked. No
new registry package or default-budget change: eight sessions/5000-ms admission,
two credential permits, three owner helpers, <=8-KiB frames, 32-KiB queue/five-second
drain remain. Candidate IDs cap at 101, inspected items 100, hits four, query 64
bytes, snippet 120 scalars, prior cursors 32 and own bookmarks 128.

Snapshot 34 freezes 248 preceding inputs and the whole catch-up record before
edits. All 47 historical identities stay fixed. The new saved guard is
`python3 tools/check_discussion_evidence.py`; the suite now has 48 evidence guards.
The owned VM, private test credentials and temporary vendor are removed; saved
owner playground stays off with disk/login/keys preserved.

These are software controls, not a new 250-member/physical SBC, hardening, network
mail/roaming, real Ghostty pixels or production acceptance claim. No server-mail
search, automatic query retention or whole-account export is added. Schema 14
requires a stopped/drained complete backup and matching-binary fresh-path rollback;
exact schema-2..13 exports remain read-only verifiable/restorable.
