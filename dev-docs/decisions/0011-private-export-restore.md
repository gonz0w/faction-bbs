# 0011: Private SQLite export and fresh-directory restore

Status: implemented local development choice; production/NetBSD gates remain
Date: 2026-10-20
Owner: requested by project owner: merge #50, continue with export and restore
Issues: [#9](https://github.com/gonz0w/faction-bbs/issues/9), [#18](https://github.com/gonz0w/faction-bbs/issues/18), [#19](https://github.com/gonz0w/faction-bbs/issues/19), [#35](https://github.com/gonz0w/faction-bbs/issues/35)
Supersedes: no prior ADR; implements the next ADR 0008/0010 recovery slice

## Question and boundaries

How can an operator recover the complete current local core without a service,
unbounded Rust document buffer, lost WAL commits, live-session resurrection or
overwriting an existing board? Stop at a tested owner-only export/verify/restore
workflow, not full installation backup, member data export or a remote operator API.
The current state is SQLite schema 2; external themes and future assets are separate.

## Gate screen and alternatives

| Choice | Privacy/local authority | Integrity/recovery and bounded work | Maintenance/portability | Decision |
| --- | --- | --- | --- | --- |
| Copy the live main file | Local, but no deliberate format | Fails: committed WAL data may be omitted | Simple but wrong while online | Reject |
| Stop all consoles and copy full private directory | Pass with trusted operator | Credible offline escape hatch; includes ephemeral state and requires downtime | Existing pre-upgrade procedure | Retain for schema-1 rollback copies |
| Generic SQL dump/execute restore | Local but executes imported statements | Earlier research showed application roundtrip failure; broad import authority | Engine-coupled SQL, migration/dump special cases | Reject for this increment |
| Stream logical records into a clean schema | Can omit secrets or scope by member | Credible future interchange; must specify types, relationships, IDs and migrations for every table | New format/parser/code; selected for later design, not needed to rehearse current recovery | Defer member/interchange format |
| Incremental online backup -> validate/sanitize/compact private stage -> no-clobber publication | Current owner only; no external network/provider, no JS/TS or additional crate | Current-format roundtrip and bounded-input oracles measured; power-loss/capacity unknown | Existing SQLite API/Unix hard links; native NetBSD application unknown | Implement local format 1 |

Mandatory approval/locality/no-JS/provenance gates remain; owner backup is explicitly
separate from member access and publication. No network/door isolation is introduced.
Capacity and full production failure recovery remain unknown, so no production
adoption score is inferred. Use qualitative comparison: preservation, no overwrite,
privacy of paths, streaming work and reversible cutover are the decision criteria;
there is insufficient whole-host data for weighted performance rankings.

## Format, failure handling and authority

- Use existing rusqlite 0.40.1 `backup`, `hooks`, `limits` features with bundled SQLite.
  Selected registry packages and licenses remain unchanged. Own code forbids unsafe.
- Format 1 is a compact standalone SQLite file with the exact schema-2 catalog plus
  one version marker table. Read-only source export never creates/migrates a board;
  SQLite backup captures a consistent committed snapshot including WAL state.
- Validate application/schema/format, exact bounded catalog, integrity/FKs, IDs,
  text/relationship/channel bounds and pinned Argon2id parameters before DML on a
  copied database. Extra triggers/views/virtual tables fail. Restore executes only
  checked-in SQL; no imported dump statements or extension loading are provided.
- Defensive/untrusted-schema SQLite settings, length/SQL/column/expression/trigger
  limits and progress callback reduce work/behavior. Trigger depth is eight rather
  than zero because SQLite FK cascades also use this machinery. Catalog equality
  still excludes application-supplied triggers. These are not a process sandbox or
  a complete hostile-SQLite parser security guarantee; use trusted operator exports.
- Clear chat sessions/joins/latest receipts, retain durable content and grants, then
  compact the stage. No receipt or free-page content is intended in the published
  artifact; a synthetic byte-marker regression covers this narrow claim.
- New restore rotates the internal board session-binding nonce, preserving durable
  record IDs and credentials. This nonce has no adopted external/federation identity
  meaning. All capabilities require fresh login/join. Restoring earlier data still
  restores earlier approvals/passwords: the operator must review policy at cutover.
- Private staging under the destination parent; complete files published with Unix
  hard links, which fail on existing names. Sync file/directory metadata. A new
  restore directory is reserved only after validation; a brief empty-directory
  interval exists, so do not open it concurrently. No atomic directory cutover claim.
- Never recursively delete destination paths. Normal errors clean stage files. Late
  sync errors can leave a complete published result; a kill can leave private stage
  or empty reserved directory. No fault-injected durability claim.
- Initial 256-MiB physical/logical cap, 32-page backup steps, 100-ms busy timeout and
  cooperative 30-second copy/SQL budget. No whole-file Rust buffer; two SQLite
  connections/reference schemas and VACUUM scratch still cost RAM/disk. Deadline
  does not bound blocked OS I/O. No small-host capacity guarantee.

## Evidence and confidence

Primary sources retrieved 2026-10-20:

- [SQLite online backup](https://www.sqlite.org/backup.html): WAL-aware incremental
  snapshots and concurrent modification behavior. Primary documentation; high for
  API semantics, not our filesystem durability/capacity.
- [SQLite VACUUM](https://www.sqlite.org/lang_vacuum.html): repacking, deleted-content
  cleanup, scratch space and explicit-ID caveats. Primary documentation. Our
  explicit durable integer IDs/retry relationships are checked by roundtrip.
- [SQLite defensive configuration](https://www.sqlite.org/c3ref/c_dbconfig_defensive.html)
  and [untrusted database guidance](https://www.sqlite.org/security.html): relevant
  defense settings; they do not authenticate a snapshot or replace a sandbox.
- [SQLite pragmas](https://www.sqlite.org/pragma.html): `integrity_check` does not
  replace `foreign_key_check`; run both plus domain validation.
- [NetBSD link(2)](https://man.netbsd.org/link.2): existing target fails and hard links
  require the same filesystem. Primary documented portability, not native app evidence.
- Pinned local rusqlite 0.40.1 backup/config/hooks/limits source inspected. Existing
  [dependency inventory](../dependencies/README.md) source/feature scan retained.

Measured: ten Rust recovery integration tests and a real CLI/live-PTY rehearsal on
the developer host. Tests cover live WAL + concurrent writes, durable record/retry
preservation, session nonce isolation, permissions/sidecars, malformed/extra schema,
FK/content/credential errors, source immutability, sparse oversize, concurrent restore
one-winner publication, busy-store failure/retry and discarded-receipt bytes. Full validation outcomes and
CI handoff are in [plan 0011](../plans/0011-local-export-restore.md). High confidence
in these narrow host oracles; native NetBSD, power-loss/disk-full injection, complete
host cost and remote-safe operator separation are untested.

## Tradeoffs, reversal and reopening

Snapshot includes plaintext private content and credential hashes; there is no
encryption, MAC/signature or protection against valid intentional edits. It is not
member-scoped/interchange portability. A structurally valid older snapshot can
reinstate previously revoked accounts or old passwords; review before exposure.
Themes/art are external. Future table/schema/KDF changes must version the format
or add explicit reviewed adapters; unknown input fails closed today.

Reversal cost is moderate: keep old binary/format readers with private snapshots,
or build a logical converter; current exports are inspectable with SQLite. No live
schema migration is added and original board remains untouched. Disk format grows
only in export artifacts. The internal nonce rotation has low current compatibility
cost, but must be revisited before any external board identity contract.

Next: native NetBSD application/recovery oracles, failure/space/load accounting,
member logical exports and data lifecycle, packaging/theme assets, protected
backup policy and a separate restricted SSH adapter. Broad issues stay open.
