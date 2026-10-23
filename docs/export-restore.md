# Private local export and restore

Updated 2026-10-23. These are filesystem-owner tools for the local development
core, not member commands or a network backup service. Build with
[the local development instructions](local-development.md). Keep the owner-capable
executable and state out of member SSH shells/forced commands.

## Export, verify, rehearse

Use existing, trusted local parents. The export parent must be owned by the current
OS user with no group/world permissions; each file is created as `0600`. Do not
use a shared `/tmp` directory directly or put private data in the checkout.

```sh
mkdir -m 700 "$HOME/.faction-backups"
./target/release/faction --state-dir "$HOME/.faction-dev" export "$HOME/.faction-backups/board-001.sqlite3"
./target/release/faction verify-export "$HOME/.faction-backups/board-001.sqlite3"
./target/release/faction --state-dir "$HOME/.faction-rehearsal" restore "$HOME/.faction-backups/board-001.sqlite3"
./target/release/faction --state-dir "$HOME/.faction-rehearsal" login --plain
```

`mkdir` is a one-time setup; select a new export filename for each snapshot. The
restore directory **must not exist**, including an empty directory. Its immediate
parent must also be owned/private (normally your private home directory); if your
home is group/world accessible, create a dedicated `0700` parent and place the
restored directory inside it. Do not loosen permissions to make checks pass.

Export can run while local consoles are connected. SQLite's backup API captures
committed database state, including WAL content, in a separate private staging
file. It does not end source sessions or run a migration. A missing, uninitialized,
foreign or unsupported-schema source fails instead of creating/upgrading a board.
Schemas 2 through 14 are supported by format 1. Schema 3 includes arcade personal bests
and completed-run counts; no in-flight game state/run capabilities are exported.
Schema 4 adds file-area metadata/policy, excluding every transfer grant; file
bytes are outside SQLite. Use [complete file-area backups](file-areas.md#complete-backup-and-fresh-restore)
when files are installed. Schema 5 additionally preserves the stable **public**
file namespace and room/chat references; this does not preserve session authority.
Schema 6 additionally preserves article IDs, generated subjects, timestamp/origin
metadata and unknown legacy dates. Schema 7 preserves these fields and adds
[byte-bounded UTF-8 subjects](unicode-subjects.md). Schema-6 verification still
requires ASCII; schema-7 verification accepts valid UTF-8 without controls.
Verification is read-only; restored schema-2..13 boards upgrade to schema 14 when
opened by the new build. Follow the
[stopped-service backup/upgrade](post-metadata.md) procedure. For schema 1 upgrades, stop
all old consoles and preserve a complete private offline state-directory copy
before the existing transactional upgrade; this exporter cannot replace that copy.

The resulting format-1 export is one standalone, compact SQLite file, with no WAL,
SHM or journal needed. Keep it closed/unchanged. `verify-export` opens it read-only
and checks its format/schema, SQLite integrity, foreign keys and core record rules;
it prints only aggregate counts. It does not create a board or prove provenance.
Future/unknown formats, extra schema objects, unsupported credential parameters,
unsafe permissions, symlinks, sidecars and malformed/inconsistent data fail.

## What is included

| Data | Treatment |
| --- | --- |
| Local members, roles, approval/revocation, epochs | Preserved as of the snapshot. |
| Password hashes/salts | Included unchanged; existing passwords still authenticate after restore. Plaintext passwords are never requested by these commands. |
| Room IDs, titles/slugs, posts, replies, author/request IDs | Preserved; identical submission retries still reconcile after restore. |
| Unread positions | Preserved. |
| Board/private chat channels, grants, sequences and latest retained messages | Preserved; still local, with no federation/publication. |
| Chat leases, joins, presence and latest per-lease retry receipts | Removed; restore starts with no live chat entries. |
| In-memory session-binding identity | Restored store receives a new random nonce. Old source capabilities cannot transplant, even if restoring older account epochs. |
| Theme packs, identity art, seasonal packs and their permissions/provenance | External to state; **not included**. Copy/review separately and select explicitly at login. |
| Arcade bests/run counts; file catalog, policy and public reference namespace | Preserved; no active game runs or transfer grants. |
| File bytes | External; use the complete catalog/blob bundle. |
| Mail, remote game saves, SSH/host/private-network/device keys | Not part of this format. |

New logins and chat joins are required after restore. Private-channel grants survive,
but the existing since-join history floor still prevents a new chat entry from
reading earlier retained history through the member API. The owner can inspect
the database; private chat is not encrypted from the operator.

Exports contain password hashes and private board content in plaintext storage.
Treat them like the live database, keep copies private and apply your own protected
storage/transfer and retention policy. There is no built-in encryption, signature,
MAC or authenticated sender check. Structural validation cannot detect a deliberate
edit that remains valid. Restore only your trusted operator snapshots, never an
untrusted downloaded database. No external upload/public service is involved.

Compaction excludes removed lease/receipt rows and old free-page content from the
published artifact; it does not erase the original store, old snapshots, filesystem
snapshots, storage-device remnants or private staging left by a killed process.

## Restore and cutover

1. Verify the export, restore into a fresh private path and log in locally. Review
   account statuses/roles, posts/replies, unread positions and separately copied themes.
2. Remember that restoring a snapshot rolls **data and policy** back. Members revoked
   after that snapshot may be approved again; later password resets are also lost.
   Apply the current approval/password policy before any eventual remote exposure.
3. Stop every console using the old board before cutover. Preserve its full private
   directory until satisfied; point subsequent owner commands at the restored path.
   Do not replace a live database, merge histories, or mix old WAL files into a restore.
4. Test another export and restore of the recovered store as part of your own rehearsal.

Publication never overwrites an existing file/directory or symlink. Export links a
fully written private staged file into a new name. Restore validates and prepares
the database before reserving the new directory; that directory is briefly empty
before the complete database is linked into it. **Do not open the destination
concurrently while restoring.** A competing opener/restore cannot be overwritten,
but this is not an atomic whole-directory switch. Files/directories are synced
before successful return. Same-filesystem hard links are required; staging is
created in the destination parent so no cross-filesystem move is needed.

Ordinary failures remove their private staging files and avoid publishing a partial
database. A late filesystem sync failure may report an error after a complete output
has been linked; inspect/verify that path and use a fresh name for a retry. A kill or
power loss can leave a private `.faction-stage-*` directory or an empty reserved
restore directory. After all relevant processes stop, inspect/remove only that
abandoned scratch data. The commands do not recursively delete existing paths.
Native NetBSD application export/restore tests now pass; filesystem/power-loss fault injection remains open.

## Initial limits and evidence

The initial tooling rejects files/logical databases above 256 MiB, copies 32 pages
per backup step, uses a 100-ms database busy timeout and a cooperative 30-second work
budget (copy checks plus SQLite progress callbacks). Busy/budget feedback means
retry when quieter/offline; it is not permission to copy a live main file manually.
Blocking filesystem I/O/sync is not a guaranteed wall-clock deadline. Allow space
for the staging copy and SQLite compaction/journal scratch, potentially several
times the database size. No full-store Rust buffer is used, but total RAM, temporary
disk, busy-board latency and SBC capacity have not been measured for these tools.

Developer-host, native NetBSD and CI checks include committed live-WAL roundtrips, source-session
continuity, durable IDs/replies/unread/status/grants, old-capability denial, removed
receipt bytes, malformed catalogs/data/credentials, unsafe paths/sidecars, sparse
over-size input, competing restores, failure cleanup and a real CLI/PTY rehearsal:

```sh
cargo test --workspace --locked --release
cargo build --workspace --locked --release
python3 tools/smoke_export_restore.py target/release/faction
```

Member-scoped logical/interchange exports, old/future format adapters, deletions,
encrypted/authenticated archives, scheduled/incremental backups, full installation
packaging and fault-injected power-loss/disk-full recovery
remain work under #9/#18/#19/#35. See [ADR 0011](../dev-docs/decisions/0011-private-export-restore.md).

Credential-budget lock files coordinate running local processes and contain no
passwords. They are not part of the snapshot artifact; a fresh restored board
creates new locks on credential work. Never unlink them from a live source board
or interpret their presence as approval/authentication state.

The optional [Recovery panel](backup-checks.md) reports explicit checks of complete
file-area bundles. A standalone catalog export is not a complete file backup and
does not generate such a receipt. `backup.status` is disposable local telemetry,
not authoritative board data and not part of export/fresh restore.

The optional [other-BBS directory](other-boards.md) is external reviewed host
configuration. Preserve its pack separately; it is not part of the DB export or
complete file-area bundle and never imports remote membership.

Schema-8 exports also include private [room draft checkpoints](writing-desk.md).
These are sysop-readable plaintext with member/room/revision binding; restored
checkpoints still require current local approval. Explicit send removes its saved
revision atomically, and expired checkpoints cannot be resumed. Logical deletion
does not erase older exports or WAL bytes. Keep the stopped complete upgrade and
fresh-path rollback procedure.

Schema 9 additionally includes the optional [local community wall](community-wall.md):
policy, current/explicitly retained notes and bounded per-member cooldown/retry receipts.
These are plaintext local community data; logical deletion is not secure erasure.

Schema 10 includes optional [local member pages](member-pages.md), policy and
bounded dated updates, including hidden content in plaintext owner snapshots.


Schema 11 also includes optional [gatherings](gatherings.md): local agenda policy,
one-off events (including hidden/cancelled rows) and member RSVP rows. They remain
plaintext owner data; a counts-only member view does not redact owner backups.
Old schema-2..10 snapshots remain read-only verifiable/restorable. Make and verify
a complete stopped pre-upgrade backup; first new-owner open migrates to schema 11.
Use matching older binaries plus the old complete bundle in fresh paths for rollback.


Schema 12 adds [poll policy/questions/choices/ballots](polls.md), including hidden/
finished polls and permanent wording locks, to private owner recovery. Result
visibility does not redact owner backups. Old schema-2..11 exports remain read-only
valid; new-owner open migrates restored data. Stop/drain and verify a complete
pre-upgrade backup; roll back only with its matching older binaries in fresh paths.


Schema 13 adds [catch-up policy, own subscriptions, explicit saved-visit checkpoints
and ordered file publications](catchup.md). These are plaintext local owner data;
opening/login/preview does not create a visit record or update unread markers.
Old schema-2..12 exports remain exact read-only valid. Snapshot schema comparisons
reject extra objects and an explicit 128-object ceiling prevents truncated checks.
Stop/drain and verify a complete pre-upgrade backup; matching-binary fresh-path
rollback remains required.


Schema 14 includes [Find policy and own discussion bookmark references](discussion-search.md).
The ordinary author/post B-tree is maintained by SQLite; no copied text index or
query history is added. Post deletion cascades references. Member-side copyable
paged reference exports are deliberately narrower than this owner recovery format.
Old schema-2..13 exports remain exact/read-only valid. Stop/drain, verify a complete
pre-upgrade backup and use matching-binary fresh-path rollback.


## Curated gallery packs

See [the gallery guide](scene-gallery.md) for the optional owner-managed startup
pack, current-approval file links, canonical ANSI/text editions and rights review.
Gallery packs are external host configuration, preserved separately from database
exports/bundles. No new schema, public service or raw uploaded graphics authority.


## Curated library shelves

[Library shelves](library-shelves.md) layer owner-reviewed notes/collections over
current local file publication/grants. Hidden sources expose no captions, shelf
names or tokens. Transfers/discussion use existing authorized workflows; browsing
alone never advances unread. Packs are external config to preserve separately
from complete DB/blob backups. Schema remains 14; no new transfer authority.
