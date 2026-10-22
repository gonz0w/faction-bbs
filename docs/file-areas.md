# File areas and restricted SFTP

The optional shared-backend **[F]iles** lobby provides local areas, descriptions,
search, upload reservations and sysop review. Transfers use a separate OpenSSH
listener and a small Rust SFTP v3 adapter. Files stay on this board. No browser,
public download URL, file federation or new authentication system is required.
This is an early hobby preview; read the validation limitations below.

## Install on NetBSD

First complete [normal setup](quickstart-netbsd.md), initialize your sysop and
build the current workspace. Before upgrading an existing board, export a private
pre-upgrade complete backup: current schema 6 adds [post metadata](post-metadata.md)
and cannot be opened by an older binary. Schema-2/3/4/5 exports remain readable. Stop member SSH, wait for members to leave,
and stop the backend before replacing binaries or changing configuration.

As root, from your source checkout:

```sh
python3.13 tools/install_file_areas.py --binary target/release/faction-files
python3.13 tools/install_file_areas.py --binary target/release/faction-files --install
```

This is a first-install helper, not an upgrader. It creates **factionfiles** (private
store `/var/db/faction-files`) and **factionfileauth** (key lookup only), with separate
primary groups and supplementary `factionipc` socket access. Neither can read the
board database. The existing owner remains the catalog/policy authority. All
binaries remain non-setuid. Replace the shared service binary with the current
build while stopped; the ordinary installer does not upgrade it automatically.

The helper preserves `/etc/faction/service.before-files.toml`, adds the two file
UIDs before any TOML tables, writes `/etc/faction/files.toml` and installs a separate
`faction_files_sshd` rc service, **disabled**. Default listener is **127.0.0.1:2223**;
select a literal private interface explicitly with `--listen-address`. No firewall,
host tuning, public bind or service start is applied. Validate the host key through
your normal trusted channel. Do not reuse the experimental admission-helper UID.

```sh
/etc/rc.d/faction_broker onestart
/etc/rc.d/faction_sshd onestart
/etc/rc.d/faction_files_sshd onestart
```

The file listener requires the actual shared-owner PID and policy socket to be
ready. For boot activation, set `faction_files_sshd=YES` in its rc.conf.d file
only after testing your chosen interface. It runs after the shared backend.
Removing both `files_uid`/`files_auth_uid` from the stopped service configuration
disables the feature. Stop file admission and drain existing transfers first.
The local console/worker rollback do not provide the Files lobby.

## Member workflow

Use **F** or arrows/Enter to open Files; commands inside the lobby use Enter.
Sysops create areas before members can upload:

```text
create general members 10485760 Scene releases
create private private 10485760 Private exchange
allow 2 alice
areas
list 1
search 1 - release
info OBJECT_ID
upload 1 release.zip 12345 Description of the archive
```

`members` permits approved local members to list, upload and download; `private`
requires an explicit grant (sysops retain access). A grant covers both upload and
download in this first version. Paste a plain **Ed25519 public key** when prompted;
never paste a private key or password. The BBS displays a transfer ID for status
and troubleshooting. The matching private key stays on your own SFTP client.
There can be only one live grant per key; issue another after completion/failure.

On your device, use the separately configured file host/port:

```sh
sftp -i ~/.ssh/faction_transfer -P 2223 factionfiles@YOUR_BOARD
```

The virtual root exposes only the granted filename. Use `put local.zip release.zip`
for the upload above. For a published file, request `download OBJECT_ID` in Files,
paste the public key, reconnect and use `get release.zip`. Both batch and interactive
OpenSSH SFTP work for one transfer per connection. The helper closes after successful
file close; reconnect with a new grant for another file. Do not use preserve-mode,
resume, append, remote rename/remove or concurrent requests for other files.

`areas AFTER_ID`, `list AREA AFTER_OBJECT`, and `review AREA AFTER_OBJECT` page
through 20 rows. `search AREA - WORDS` starts a search; its next-page hint preserves
the query as `search AREA AFTER_OBJECT WORDS`. Search covers names/descriptions.
`back` returns to the ANSI menu. Names are 1–64 ASCII letters/digits/`._-`, with
no leading dot or `..`; descriptions are 512 printable UTF-8 bytes. Display names
never become disk paths. Archives are stored without extraction or execution.

## Review and cleanup

```text
review 1
info OBJECT_ID
download OBJECT_ID
publish OBJECT_ID
remove OBJECT_ID
deny 2 alice
```

Only a sysop can inspect metadata/download a **completed pending** upload for review,
publish it, hide it or grant/deny area access. Incomplete/reserved objects cannot
be downloaded. Review does not certify a file as safe. Current approval, epoch,
area access and object state are checked during key lookup, claim and each SFTP
request/chunk; an area-policy update invalidates that member's live grants.

Interrupted uploads remain private and charged. **Resume is unavailable in v1.**
Remove the reservation in the sysop lobby, then run:

```sh
su -m factionfiles -c '/usr/local/libexec/faction-files gc'
```

GC acquires both persistent transfer locks, deletes only catalog-removed opaque
objects/partials, syncs storage, then reclaims catalog quota. It refuses while a
transfer holds either lock. Never delete/reset `slot0` or `slot1` while anything
can use the store. A crash before catalog completion leaves the upload reserved;
remove/GC it and upload again. GC can reconcile the exact two-link private inode
pair left between no-clobber publication and partial unlink. Unknown/mismatched
aliases fail closed and require operator inspection, never automatic publication.

Preview caps: two active streams; 32 KiB chunks; 16 MiB/file; 256 MiB board bytes;
32 areas; 1,000 catalog objects; two outstanding grants and two incomplete uploads
per member; 64 grants globally. Removed objects remain charged until GC. Grants
last five minutes, helper wall time at most five minutes, packet-input deadline
30 seconds and output/policy deadlines three seconds. These are implementation
bounds, not supported-capacity claims or general SSH preauthentication limits.

## File references and integrated transfers

The optional [native client](native-client.md) handles upload/download from the Files
menu while the BBS session stays connected. Plain SSH retains the manual workflow.
`link OBJECT_ID` produces a stable `faction-file:NAMESPACE/OBJECT_ID` reference for a
published file you can access. Room posts and chat accept up to four such references
per message; sending and reading check current membership/area permissions. Readers
without access see `[file unavailable]` without private file metadata. A link grants
no permissions and never fetches another board. Typed prose about malformed/incomplete
references remains ordinary text. Metadata manually typed into a post is ordinary
post content; redaction applies to recognized references, not arbitrary prose.

Files `info` and `download` accept either local object IDs or full local references.
`cancel TRANSFER_ID` abandons your current ready/active grant; incomplete upload
reservations remain private/charged. The native client manages this automatically
when a confirmed transfer fails. Recent link numbers are local client convenience,
not server authority. Revoked access is rechecked when opening/downloading.

Schema 5 preserves the public namespace and stored tokens through backup/restore,
independently of the freshly rotated private session identity. Do not run restored
clones as independent federated boards with the same namespace; namespace reassignment
and cross-board file sharing are future design work.

## Complete backup and fresh restore

**`faction export` includes the catalog, not the file bytes.** It removes outstanding
transfer grants from the exported copy. Use the complete tool below for file data.
Stop both file/member admission, drain transfers and members, then stop the backend.
Stopping the SSH listener alone does not kill authenticated transfer children.

Choose a new bundle directory under an existing private factiond-owned parent
(such as `/home/factiond/backups`); create that parent as factiond with mode 700.
As root, use actual numeric UIDs from `id -u factiond` and `id -u factionfiles`:

```sh
python3.13 tools/file_area_backup.py backup --owner-uid OWNER_UID --files-uid FILES_UID --bundle /home/factiond/backups/bundle-001
python3.13 tools/file_area_backup.py restore --owner-uid OWNER_UID --files-uid FILES_UID --bundle /home/factiond/backups/bundle-001 --board /home/factiond/recovered-board --store /var/db/faction-files-recovered
```

Backup holds the existing owner PID lock and both transfer slots, copies a validated
standalone export plus all catalog-owned bytes and writes a SHA256/length manifest.
It refuses unknown entries, aliases, missing or corrupt published/pending data.
A bundle is private **plaintext**, including account hashes and private board data;
store it securely. It excludes themes, SSH host/private keys and system configuration.

A new `verify` operation reuses these complete checks without creating restore
paths or changing live locks. Add `--record-status --board PATH` to backup/verify
for the optional [historical Recovery panel](backup-checks.md); default publication
is off. File/board bytes and secrets stay out of telemetry.

Restore verifies the entire bundle before creating fresh destinations, validates
the DB with the Rust owner tool, creates a fresh board identity, restores exact
bytes with the correct file UID and carries no transfer grants. The new board's
parent must be private, writable and factiond-owned. It never replaces a running
board or starts services. An interrupted tool leaves private partial fresh paths
for explicit inspection/removal before retrying. Keep admission disabled while
switching to the recovered state/store; root must review paths/ownership/config,
then start and test the recovered board. Do not merge it into existing state.

## Validation and extensibility

See [implementation plan](../dev-docs/plans/0033-file-areas-sftp.md) and the linked
native evidence for actual checks. Software VM checks do not establish physical
SBC support or 250-user capacity. The owner still uses its existing three helper
threads, two credential permits/slots, crypto settings and eight-session default.
Two file streams/SSH processes run under separate identities; no owner fork or
file bytes traverse the policy socket. This is scoped helper/API isolation, not
an arbitrary hostile-native-code or all-family network sandbox.

Area policy is independent of SFTP. Future ZMODEM/native-client/other transports
can use bounded reservation/claim/check/chunk/complete semantics behind a distinct
adapter boundary. Never expose trusted adapter completion methods as member RPCs.
Files, mail and game storage/authority stay separate. No JS/TS dependency was added.
