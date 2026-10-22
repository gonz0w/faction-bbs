# 0037: Persistent local post metadata without implicit publication

Status: accepted local increment, 2026-10-21. Issue #103; plan 0039.

## Decision and constraints

Add schema-6 post_metadata keyed to the existing local post ID, with a unique
public random reference, generated subject, origin and nullable creation time.
Legacy records have origin legacy/unknown original time. New posts receive local
origin and the host's actual Unix seconds. Insert metadata in the same transaction
as a new post; reconcile retries before generating new identity/time. No body,
reply, unread, file namespace, role or credential changes.

Use SQLite's existing bounded migration machinery and cache, with one transactional
INSERT SELECT backfill. randomblob identifiers are public unique random
references, not secret capabilities or cryptographic identity. Keep authentication
randomness unchanged. The local proto-article mapping uses the reserved
faction.invalid domain; no DNS/host address/home proof or outbound feed is implied.
Future injection, configured From/Path/group policy and duplicate-ID handling are
separate #103/#101/#104 work.

Keep the existing owner-open upgrade model. Require stopped/drained admission,
verified complete backup and coordinated binary upgrade as documented in
[post metadata](../../docs/post-metadata.md). This is not a live-upgrade or
in-place-downgrade promise. Old read-only exports never migrate their source.
Canonical snapshot validation accepts old formats and exactly validates the new
table/data; private restore authority rotates while public references survive.

## Alternatives and tradeoffs

Putting fields directly on posts would expand every existing Post query/caller;
a separate keyed table preserves the current body API and adds one approved
metadata lookup per rendered screen. A sidecar file would complicate atomicity
and complete backups. Timestamping old posts at migration would fabricate history;
retain unknown dates. Require author-entered subjects later, keeping today's
one-line BBS posting flow useful. Generated labels are explicit UI metadata.

The new table/index increases per-post disk and backup size; no fresh capacity or
whole-resource guarantee follows. The SQL backfill avoids an unbounded Rust vector
of all historical IDs. ID collision or invalid clock/metadata causes transaction
failure, not a partially committed post. A public identifier is not authorization.

## Evidence and reversal

Primary [SQLite core functions](https://www.sqlite.org/lang_corefunc.html) and
[date functions](https://www.sqlite.org/lang_datefunc.html), retrieved 2026-10-21,
document randomblob and UTC Unix time/formatting; they are reused through the
already reviewed bundled SQLite, with no new dependency. Subject/date presentation
is inert and bounded. Native/host and migration/export/restore controls are the
software gates; plan 0039 records actual outcomes and remaining gaps.

Preserve the original #105 inputs/results in sources/pre103-metadata before these
changes. Rollback requires the verified old backup/old binaries in fresh paths;
it cannot recover post-backup data automatically. External copies/signatures would
raise reversal cost and are not enabled in this increment.
