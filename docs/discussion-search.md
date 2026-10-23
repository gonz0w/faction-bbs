# Find discussions and save bookmarks

The optional **B:Find & save** desk helps members find an older conversation and
keep a useful post for later. Enable/name it under **S:Sysop → B:Find settings**;
it starts disabled. Sysops choose cards or compact rows. Trusted themes can place
`action = "find"` with a free key/label; the enabled host inserts B otherwise.
The disabled host capability stays unavailable even when a theme declares it.
Login, board art and welcome order remain unchanged; ANSI and plain terminals
use the same commands followed by Enter.

| Command | Action |
| --- | --- |
| F | Start a literal text search; `/cancel` abandons its form. |
| B | Your own bookmarked posts. |
| N / P | Next/previous bounded search scan or bookmark page. |
| R | Refresh from the current newest post, including later arrivals. |
| J | Open an exact stable post ID in a read-only source view. |
| A / D | Add/remove a bookmark by post ID, then explicitly S:save or C:cancel. |
| X | Explicit copyable bookmark-reference export. |
| Q | Main menu. |

## Search and source views

Enter 1–64 UTF-8 bytes, optional numeric room ID, optional exact author handle and
S:subject, B:body or A:both. Text is literal: `%`, `_`, quotes and SQL-like text
have no pattern/query-language meaning. ASCII letters match without case; other
Unicode scalars match exactly. This is not Unicode linguistic folding, stemming,
ranking or a full-text query language. Results use newest post ID first. An opening
high-water cursor keeps later arrivals out until refresh or a new search.

Each action fetches at most **101 candidate IDs**, inspects at most **100 posts**
and returns at most **four hits**. A cooperative **20-ms iteration deadline** can
end a scan earlier; this is not a hard wall-clock promise for filesystem/SQLite
calls. N continues older than the last inspected item, including after an empty
scan. **No matching items in this scan** does not mean the whole board was searched.
P retains up to 32 prior cursors. Exact room/author candidate indexes avoid an
unbounded text filter before LIMIT; combined filters inspect a bounded room slice.
No global total, stored query history, background indexer or duplicated content
index is built. Richer FTS/search acceleration remains follow-on work under #9.

Current approval and local source visibility apply to every result, snippet,
bookmark caption and source view. Body matching happens **after current per-reader
file-link projection**: an inaccessible private filename/object token cannot be
used as a search oracle, and snippets show the same unavailable placeholder as
normal room reading. All current rooms are approved-member local discussions;
this adds no private-room ACL, remote feed or federation capability.

J opens the exact selected post and re-fetches it on every page. Source controls
are N/P:page, A:bookmark, Q:results. It reuses the existing chronological message
presenter with inert subjects/body and classic/Unicode capability rules. Changed
file projections restart the source page safely; a deleted source cannot silently
be replaced by the next post. **Search, source viewing, bookmark changes and export
never mark a post read, even after rendering the whole source.** Use the normal
room New reader for durable unread progress; its existing complete-message/output-
drain acknowledgment remains unchanged.

## Your bookmarks and reference export

At most **128 post references per member**. Explicit review/save makes a bookmark;
repeating Save does not duplicate it. Other members cannot list/edit your references.
Own removal requires the same visible review. Busy saves keep the review for
explicit retry. References retain no copied titles, snippets, message bodies or
queries; each page fetches current source data. Post deletion cascades references;
there is no hidden history or automatic expiry. Other item types, notes, import and
cross-board bookmark synchronization are later scope.

X emits an explicit **paged reference export** for copying from your terminal.
Each page has a header and up to four records; N continues and P returns. Save the
headers with the records, and use a fresh export after edits for a consistent set.
This is not a transactional whole-account export or a server-side file download.
No member-selected host path, credential, session capability, mail key or raw SQL
is included. The format is ASCII with LF lines and tab-separated records:

```text
FACTION-BOOKMARKS/1 board=PUBLIC_NAMESPACE_HEX
ROOM_ID<TAB>POST_ID<TAB><ARTICLE_ID@faction.invalid><TAB>SUBJECT_UTF8_HEX
```

The namespace is the existing public local file-reference namespace, independent
of private session authority; it and local article identities survive complete
recovery. The final column is lowercase hex of the exact UTF-8 subject bytes,
so Unicode/control-safe display does not lose the authored title. References grant
no access, prove no signature/home identity and do not resolve other boards by
network. The first export covers discussion posts only.

Bookmarks/settings are sysop-readable local plaintext in owner snapshots. No
private-mail search, count, decryption or server indexing is introduced; private
mail remains member-device work under #90/#99. No automatic query retention,
new service/helper/registry dependency or changed resource default is introduced.
Frames stay <=8 KiB; current output queue/drain and credential/session limits stay.
No new 250-member capacity or physical-hardware claim follows.

## Upgrade and validation

Schema 14 adds optional Find policy, the author/post B-tree and own reference rows.
Stop/drain and verify a complete pre-upgrade DB/blob/config/key backup before the
first new-owner open. Exact schema-2..13 exports remain read-only valid; restored
older boards migrate to 14 when opened by the new build. Use matching old binaries
and a complete older backup in fresh paths for rollback. Logical removal does not
erase old backups/WAL/storage. See [export/restore](export-restore.md).

[Plan 0067](../dev-docs/plans/0067-discussion-search-and-bookmarks.md),
[ADR 0062](../dev-docs/decisions/0062-discussion-search-and-bookmarks.md) and
[NetBSD software evidence](../dev-docs/evidence/discussion-netbsd/README.md)
record the implementation and checks. The saved owner playground stays off.
