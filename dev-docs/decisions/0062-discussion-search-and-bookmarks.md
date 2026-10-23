# 0062: Bounded discussion discovery and own references

Status: accepted for the hobby preview, 2026-10-23. Issue: #142.

## Context and decision

Members need to find older discussions and save useful posts without mandatory
query telemetry, a second read ledger or private-file metadata leaks. Build on
[storage research 0009](../research/0009-local-storage-comparison.md) using existing
SQLite/local authority. The research's optional FTS direction remains a candidate;
this first usable desk uses explicit indexed candidate scans rather than a new
content index or an unbounded text predicate before LIMIT.

Default-disabled host-optional B:Find & save, with sysop heading/cards/compact and
trusted-theme positioning. One shared faction-find workflow, core/terminal only,
for local/worker/shared adapters. No new registry package, helper, listener, service
or resource defaults. UTF-8 literal subject/body query (64 bytes), optional room and
exact-author filters, ASCII-insensitive/exact-other-Unicode matching. No ranking,
stemming, remote search or private-mail indexing.

Fetch <=101 candidate IDs using the existing primary/room index or a new author/id
B-tree, inspect <=100 items and return <=4 hits, newest first. Combined filters use
a bounded room slice. A 20-ms cooperative iteration deadline stops further items;
SQLite/filesystem calls have no new hard wall-clock guarantee. Resume strictly
older than the last inspected item. Empty scans may still have more candidates;
show this honestly instead of an unbounded total. Capture a starting high-water
cursor and retain <=32 previous cursors. No persistent query or materialized feed.

Current file-reference projection precedes body matching/snippets, not merely
result formatting. Every source/bookmark/result frame re-fetches current approved
local data. Exact source views use the existing inert Message presenter and no
unread writes, including complete output. Changed projections reset page offsets;
a removed source does not open its successor. Normal room New reading and its
complete-message/drain acknowledgment remain untouched.

Schema 14 adds Find policy, the author B-tree and at most 128 member-owned post
references. Unique (member,post) makes repeated Save idempotent; explicit review
and retry survive Busy. No copied title/body/query. A post FK cascades deletion;
SQLite maintains/rebuilds the ordinary B-tree during normal writes/recovery. No
text index needs reconciliation. Bookmarks have no hidden expiry, are operator-
readable local data and travel in complete owner backups.

An explicit paged member reference export uses the existing public board namespace,
local room/post/article IDs and hex UTF-8 subject bytes. Copyable pages contain no
host path or credential and grant no authority. This is deliberately narrower than
full account export/import, network resolution or cross-board synchronization.
Other item types, notes and accelerated search remain separate work.

## Alternatives and consequences

FTS5 can improve large-board retrieval, but its tokenizer/ranking/maintenance and
permission-aware file projection need measured integration and reconciliation.
A basic WHERE body LIKE ... LIMIT can scan the whole board before finding hits.
Bounded candidate scanning has predictable work and honest scan pagination, at the
cost of more Next actions for sparse queries. Author/room narrowing is useful now.
An ordinary author/post index adds storage/write cost without duplicating text.

Opening a found post through an unread cursor could mark skipped messages read.
The exact read-only source view avoids that ambiguity and retains the one existing
ledger. A bookmark is a local personal reference, not private encrypted content.
No member-side mail key or ciphertext store is inspected, even for a search badge.

## Validation and reversal

See [plan 0067](../plans/0067-discussion-search-and-bookmarks.md),
[guide](../../docs/discussion-search.md) and
[software evidence](../evidence/discussion-netbsd/README.md). Snapshot 34 preserves
248 preceding inputs and the whole catch-up record before edits; historical hashes
stay fixed. Schema-2..13 exports stay exact/read-only valid. Stop/drain and verify
a complete backup before upgrade; matching old binaries and a complete old backup
in fresh paths provide rollback. Disable the feature without deleting references.
The saved playground stays off; hardening and new capacity work stay later.
