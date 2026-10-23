# 0067: Discussion search and personal bookmarks

Issue: [#142](https://github.com/gonz0w/faction-bbs/issues/142)
Status: implemented and validated; ready for PR review. Owner: merge #164 and continue useful product work.

## Goal and decisions

Default-disabled Find desk: local literal subject/body search, room/exact-author
filters, bounded newest-first scans, inert snippets, exact read-only source views,
own bookmarks and deliberate paged reference export. Sysop name/layout/enabling;
ANSI/plain/local/worker/shared paths. No private-mail indexing or new listener.

Build on research 0009 without adopting full-text indexing prematurely: indexed
post/room/author candidate scans, <=101 candidates and <=100 inspected per action,
four hits, bounded cooperative time checks. Apply current file-link projection
before body matching/snippets. No stored queries, ranker or duplicated content index.
Capture the starting post high water; explicit refresh includes new posts. Previous
holds <=32 cursors. Exact source views reuse Message presentation and never change
unread, including after full rendering; normal room reading/drain acknowledgment
stays unchanged. Every view/action rechecks current approval/policy/source data.

Schema 14 adds optional policy, an author/post B-tree and at most 128 own post
references. No cached bookmark titles/bodies; fetch current data. Post deletion
cascades references; no hidden retention or automatic last-seen tracking. Paged
member export emits versioned local reference records with a public board namespace,
article identity and hex UTF-8 subject, never credentials/host paths/private mail.
It is an explicit copyable reference export, not a full account backup/import.
Other item types and richer search indexes remain follow-on scope.

## Progress

- [x] Merge checked #164; read #142/#9 and preserve preceding measured inputs.
- [x] Add core schema/search/bookmarks/export and old-schema recovery validation.
- [x] Add common local/worker/shared Find UI and classic/source-view controls.
- [x] Validate queries/visibility/unread/retention and actual NetBSD terminals.
- [x] Update README/docs/ADR/AGENTS/changelog/evidence and open focused PR.

## Gates

Snapshot 34 preserves 248 preceding inputs and the whole catch-up result. Keep all
47 saved historical guards/identities unchanged. No new registry dependency,
listener/helper, credential/default/session/queue/drain change or capacity claim.
Only an owned marked software clone; saved playground stays off, hardening later.
Schema 14 requires stopped/drained verified complete backup and matching-binary
fresh-path rollback; exact schema-2..13 snapshots remain read-only valid.


## Validation and findings

292 Rust tests per OS /129 Python tests pass. Eight local/worker Find PTYs per OS
and eleven actual restricted SSH/host-native/NetBSD-native/plain controls pass.
Seventeen complete-recovery controls per OS include native actual CLI with distinct
non-root owner/file identities. The final clean native build matches 250 selected
inputs, with 23 host harnesses and fifteen unchanged access/credential inputs pinned;
installed faction/service/sysop hashes match it and the new bookmark-save literal.
Existing local social/editor/reader/menu/art/vault/picker regressions and full host
terminal CI commands pass. Two stale schema-13 assertions in the older writer/seed
harnesses were updated to 14; final clean native validation passes without changing
product startup/permissions.

The sparse-query control follows scan cursors across a long no-match interval and
finds the older hit without including a later arrival until refresh. Filters are
literal, not SQL/regex/FTS. Body matching first applies live file projection; revoked
private filenames/object tokens do not match, and a held source restarts with the
unavailable placeholder. Deleted exact sources never silently show a successor.
The read-only source is deliberately separate from normal New reading, whose actual
shared complete-output/drain acknowledgment still advances the existing ledger.

Own exports round-trip Unicode subjects via hex bytes and preserve public local
namespace/article identities. They are paged copyable references, not full account
exports, network resolution or member-selected host files. Bookmarks are ordinary
owner-readable references with no copied text/query history; FK deletion and exact
schema-13 recovery are tested. No private-mail content/count/index is inspected.

Snapshot 34 and all 47 historical result identities remain fixed; the new current
record adds guard 48. Owned VM/cache/test credentials are removed, saved playground
off/preserved. No new capacity/physical hardware/hardening or mail/network claim.
