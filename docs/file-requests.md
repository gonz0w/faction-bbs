# File request desk

The optional **K:Library shelves → R:file requests** is a local community wanted
list, paired with the [curated shelves](library-shelves.md). Approved members can
request permitted documentation/shareware, supply an already-published local file,
and close/delete their own requests. Sysops can close/delete any request. Local
request text, authorship and provenance are ordinary operator-readable board data;
this is separate from device-encrypted private mail.

## Enable and present

The owner-reviewed `library.toml` now accepts optional `requests = true` and
`requests_name = "Wanted files"` (1–24 printable ASCII bytes). Older manifests keep
requests disabled. The existing `enabled`, `name`, `compact` and trusted theme
`action = "library"` control overall availability, the main title/card density and
main menu placement. A requests-only pack can use `shelf = []` and `book = []`:

```toml
version = 1
enabled = true
name = "Community library"
compact = false
requests = true
requests_name = "Wanted files"
shelf = []
book = []
```

Check/install it using the [library instructions](library-shelves.md). The same
fixed pack works in local/worker `--library PACK` and shared-service `library`.
Pack changes need a stopped/drained restart; disabling hides the UI and keeps data
for a later restart. This is a presentation capability, not a file grant.

## Ask, fulfill, finish

C starts a title (64 ASCII bytes) and one-line description (128 bytes). Review and
type YES to post after the final review page; Q cancels at review, and blank cancels text-entry steps. Requests
are immutable after posting; close/delete a mistaken request and post another.
Creation retries use a member-scoped nonce and reject changed content.

V selects a stable numeric request ID. Four newest requests appear per page; N/P
pages and R refreshes the list. A detail screen wraps the entire title, description,
rights and provenance in eight-row pages with N/P, including narrow/plain terminals.
Q returns through detail/list/library/main. No preview acknowledges room unread.

U offers fulfillment of an open request. Enter a 32-character local object ID or
its complete canonical `faction-file:BOARD/OBJECT` reference, then declared rights
(96 ASCII bytes) and provenance (128 bytes). Review with N/P and type YES after the final page. Any currently
approved member may fulfill, but the requester must still be approved and both
members must currently have permission to the **published** file in the same write
transaction. Pending, removed, foreign, missing and inaccessible sources cannot
fulfill. No new bytes, grants, quota changes or publication operations are created.

Fulfilled requests retain their status if an object is later withdrawn. Each
viewer sees the offer, token, rights/provenance and transfer action only while
currently eligible for the file. Otherwise a generic unavailable indication appears;
the original request remains readable. A later grant can make it eligible again.
Previously viewed/downloaded text cannot be recalled. A sysop's normal file access
remains broader than an ordinary member's; tests exercise both.

L shows the current copyable link. Shared-service F uses the existing Files lobby
and native recent-link list; members still explicitly choose and confirm a transfer.
Native compact visits and payload ceilings remain effective. Local/worker consoles
provide links without a transfer adapter; ordinary SSH retains manual SFTP.

X closes and Z deletes, each requiring YES, for the author or current sysop only.
Close preserves any fulfillment; delete removes the live record and frees its
retained slot. It does not erase backups or storage remnants. Exact revision checks
reject stale confirmations; reopen the request to review current state. Busy writes
keep the review/nonce for explicit retry. Revoked/expired/foreign sessions lose all
access, and requests from a revoked author disappear for ordinary other members;
sysops retain moderation access.

## Limits, rights and recovery

At most **256 retained requests per board**, **8 open per member**, four cards and
64 previous list cursors bound storage/UI work. Close releases the open quota;
explicit deletion releases the board quota. Detail paging retains at most 32
cursors. Request input is at most 128 bytes and frames at most 8 KiB. All text is
printable ASCII; complete file-reference tokens are accepted only in the dedicated
fulfillment field, not request/rights/provenance prose. No listener, helper, registry
dependency, background retrieval or per-member file cache is added. Normal eight
sessions, five-second admission/drain and existing credential/output limits remain.

A request or a rights declaration does not authorize piracy or prove a license.
Review actual distribution permission before the existing upload/publish workflow.
There is no automatic Internet fetch, archive extraction, selected host path,
resale/public syndication or permanent publishing. Shelf rights remain separately
owner-curated. File bytes and quotas stay under [file areas](file-areas.md).

**Schema 15** atomically adds the request table/index on owner open. Before upgrade,
stop/drain all owners/workers and verify a complete matching database/blob backup,
plus external packs/configuration/keys. Export/verify/restore supports exact schemas
2–15; verification never migrates older artifacts. First owner open migrates a
restored older board. Schema-15 snapshots retain all requests and fulfillment
metadata, including currently unavailable references; broken/deleted sources remain
unavailable after restore. Current permissions are rechecked after restore.

Do not mix old/new owners, relabel `user_version` or downgrade in place. Roll back
using the matching older binaries and complete older backup in fresh paths. Packs
are external configuration and must be preserved separately. Database restoration
rolls approval/grants back too; re-review before enabling services.

See [ADR 0066](../dev-docs/decisions/0066-file-request-desk.md),
[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/140) and
[software evidence](../dev-docs/evidence/file-requests-netbsd/README.md).
