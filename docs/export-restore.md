# Private local export and restore

Updated 2026-10-20. These are filesystem-owner tools for the local development
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
foreign or older-schema source fails instead of creating/upgrading a board. Only
current schema 2 is supported by this first format. For schema 1 upgrades, stop
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
| Future file areas, mail, door games, SSH/host/private-network keys | Not implemented in this store; not part of this format. |

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
