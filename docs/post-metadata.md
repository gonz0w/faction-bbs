# Persistent discussion metadata and schema-6 upgrade

Each room post now has a persistent public article identifier and a generated
subject label. Members can now [author subjects or accept reply defaults](room-articles.md).
Newly created posts record the host's current UTC Unix time.
The reader shows the subject and UTC time alongside the existing author/reply
context. Post order continues to follow local IDs, including when the host clock
moves backward; timestamps are not trusted time attestations.

Existing posts migrate with generated `Legacy message #ID` labels and **unknown
original dates**. Their bodies, local IDs, replies and unread positions stay intact.
These legacy labels are derived metadata, not recovered authored subjects.
New subjects are authored or defaulted during posting; wider header/MIME and
independent news-software interoperability remain #103.

IDs use `<opaque-local-id@faction.invalid>` for this local proto-article stage.
The reserved domain is not a deliverable mailbox or a home authority. IDs are
public references, never credentials, signatures, membership or posting permission.
They carry no username, host address, private rotating board identity or file
namespace. They remain stable on retries/reopen and schema-6 export/restore.
Migrating an older export that never contained these IDs assigns new ones.
Publication and real peering identities/routes remain gated. The
[local projection](room-articles.md) uses reserved aliases and strict ID-conflict
comparison without a remote import or publication path.

## Upgrade procedure

Metadata is schema 6. Follow the existing owner-open migration model, with a
stopped-service upgrade and verified backup **before the first new binary opens
an existing board**. Mixed versions and online upgrades are not supported.

1. Stop member/file admission and drain active SSH/SFTP transfers, doors and the
   board backend using the [service lifecycle](service-lifecycle.md) and
   [file-area backup](file-areas.md) procedures. Keep the operator recovery path.
2. Save a complete owner-private backup: DB export plus file blobs, themes,
   configuration and required keys. A SQLite export alone contains no file blobs.
   Read-only export/verify do not upgrade the live source. Verify the export with
   the matching tool and retain the previous binaries/configuration.
3. Upgrade all application binaries together. The first owner open transactionally
   creates/populates `post_metadata` and advances version 5 to 6. Legacy dates stay
   NULL. Do not manually relabel `user_version` or mix old writers with new code.
4. Verify a new schema-6 export and inspect room bodies, replies, unread state,
   file references and new-post subject/time before restarting admission. New
   posts and their metadata commit in one transaction; retries reuse metadata.
5. Keep the pre-upgrade backup until the sysop accepts the new installation.

The new migration stage rolls back on error. Older schema migrations keep their
existing staged behavior. If migration or verification fails, keep admission
stopped and inspect the error through the local operator path; do not delete DB,
WAL/SHM, credential slots, file locks or owner pidfile locks to force a start.

## Restore and rollback

New binaries verify schema-2 through schema-6 exports using exact checked-in
catalogs and record validation. Export includes metadata when present; schema-6
restore preserves public article IDs and file namespace but rotates private
session authority. Older exports stay unchanged and gain legacy metadata when
the restored board is first opened. Missing/invalid metadata in a schema-6 export
is rejected before publication. An unknown catalog/trigger is not executed.

Rollback is a restore of the verified pre-upgrade complete backup into **new**
state/store paths, with the matching previous binaries/configuration. Follow the
existing no-overwrite restore procedures; never edit a live DB back to version 5.
Posts/files created after that backup are not part of it. There is no lossless
in-place downgrade or automatic publication/replication recovery in this increment.

Current software checks cover atomic new-post/metadata failure, migration
conflict/rollback, old read-only export, new/old restore, stable IDs, current
approval and unchanged unread/reply data. They do not establish physical power-loss,
supported hardware, a new 250-user budget or complete production hardening.
