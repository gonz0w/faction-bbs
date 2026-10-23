# 0067: Owner-reviewed local magazine editions in the shared library

Status: accepted for the reader/editor-pack preview · 2026-10-23 · Issue #143

Add a useful publication reader and original starter issue using the existing
owner-reviewed startup library pack and shared frame/feed adapter. A magazine is
an explicitly assembled local publication, independent of a room/file grant. Do
not infer consent from private-room access or automatically import posts. Keep
submission queues and authenticated contributor approvals open under #143.

Use bounded printable ASCII editions, explicit editor-declared rights/provenance
and local consent, stable issue/article slugs and owner-managed numbers/revisions.
Published issues require every article's declared board-local consent. Drafts and
withdrawn editions stay out of member lists/selection/counts; offline owner review
can see them. Declarations are editorial records, not authenticated signatures.

Keep content in one immutable Catalog Arc under the existing combined 32-KiB
manifest cap. Four cards/eight text rows, bounded cursors and existing approvals,
output/session/credential/helper limits remain. This requires no new DB migration,
registry dependency, transport, helper or raw terminal-control renderer.

Optional edition downloads recheck existing local file permissions and use native
Files/recent links/SFTP without creating new transfer authority. A private file
offer does not hide independently and deliberately published article content.
Discussion jumps use the existing reader; previews do not acknowledge unread.

Owner-only preview and new-file text export support a manual offline publication
workflow. Install/replace/withdraw through stopped/drained restarts, retaining old
pack revisions and permission records separately from DB/blob backups. Existing
snapshots/downloads remain retained; no recall/secure-erasure or public-distribution
promise. IDs/revisions are labels, not a persisted verified release ledger.

Alternatives: a new server-side editorial database and moderation queue would
expand this batch; reusing raw uploaded art would weaken existing terminal bounds;
a live mutable file reader would add path/race authority. The selected pack reader
provides useful ordinary BBS content while preserving those follow-on boundaries.
