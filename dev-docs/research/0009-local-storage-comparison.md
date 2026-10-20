# 0009: Local storage, recovery and export

Later direction: [ADR 0006](../decisions/0006-rust-and-primary-netbsd.md) accepts Rust and primary NetBSD. [Native NetBSD evidence](0023-netbsd-dependencies.md) adds pinned dependency/gateway checks; earlier comparative observations below remain historical evidence, not contrary current authority.

Status: bounded research complete; recommend SQLite WAL for the next validation slice. Production persistence contract remains proposed.
Date: 2026-10-19. Issue: [#9](https://github.com/gonz0w/faction-bbs/issues/9).
Owner: Codex research; maintainer reviews adoption. Stopping condition: compare credible storage roles, run bounded recovery/export evidence, and specify remaining filesystem/security tests. [Decision framework](decision-framework.md).

## Recommended direction

Use **one local SQLite database per BBS**, WAL with `synchronous=FULL`, foreign keys enabled and bounded connection/cache/writer work for the first integrated experiment. Keep search a rebuildable derived index. Use a versioned logical record export with attachment manifests as the portability direction; do not treat a live database-file copy or arbitrary SQL dump as the complete backup/export contract. Large uploads/game assets remain local files with database metadata and quotas to validate in #22/#34. No message-room/account database replication or distributed ledger is needed for this local workload.

This selects a concrete experiment, not a production schema or proof against power failure. SQLite authorizes nobody: the application still enforces current local approval and per-room/file permissions. Future shared-chat delivery/history belongs to #17/#26; it does not replicate local accounts, rooms, files, mail or directory entries. #7's wider topology comparison can proceed separately because local authority already constrains this experiment; no federation model is chosen.

## Alternatives and comparison

Sources below retrieved 2026-10-19. No unsupported numerical ranking; weights S25/O20/R15/T15/M10/P10/X5 with security/durability/operations unknowns retained as U.

| Option | Benefits/evidence | Cost or trigger for reconsideration |
| --- | --- | --- |
| SQLite rollback journal | Embedded SQL, same schema/search/export possibilities; measured here as the simpler journal control. | A held read snapshot blocked writer commit in the probe; commits were slower here. Still viable for different filesystem/workload constraints. |
| SQLite WAL | Reader snapshots and short local writes coexist in the probe; no independent DB service. [SQLite appropriate uses](https://www.sqlite.org/whentouse.html) describes local application storage versus server DB roles. | Still one writer at a time. Requires checkpoint/WAL lifecycle and local filesystem semantics; do not put this shared-memory WAL design on a network filesystem. [WAL reference](https://www.sqlite.org/wal.html). |
| PostgreSQL service | SQL, transactions/concurrency control and established dump/restore facilities. [PostgreSQL 18 concurrency](https://www.postgresql.org/docs/18/mvcc.html), [dump/restore](https://www.postgresql.org/docs/18/backup-dump.html). | Additional independently operated service/update/backup/configuration obligations. No measured complete resource comparison here. Revisit for demonstrated high write contention/multi-process operational needs, not connection count alone. |
| Embedded key/value store, e.g. LMDB | Credible alternative for explicit key/index layouts and native local access. | Would need our own query/relationship/search/export layers. LMDB primary docs could not be retrieved in this session; no supported concurrency/performance claim or elimination based on unverified details. Measure before adopting. |
| Flat files / append-only records | Simple inspectable data; strong reference/export option. | Must implement crash-safe indexing, atomic multi-record changes, search, deletion/retention and migrations. No full custom storage engine justified by current evidence. |
| Public ledger/replicated account/content store | No demonstrated local resource advantage. | Private/local data publication violates locality/privacy; mandatory remote availability violates independent operation. Reject those roles. Optional external encrypted backups need a separate explicit policy/restore test. |

### Search, files, retention and export roles

[FTS5](https://www.sqlite.org/fts5.html) supplies SQLite text search mechanisms; use external-content indexes with explicit maintenance/rebuild, and permission-filter search results before exposing snippets/counts. The probe tests insertion/deletion synchronization for a single synthetic visibility domain, **not** cross-room search authorization. FTS is optional above basic room reading; private retention/deletion must also cover indexes, logs, journals, backups and exports according to local policy. Logical deletion is not forensic erasure.

For file areas, compare DB BLOBs against local files rather than buffering complete uploads: BLOBs simplify transaction/backup coupling but can enlarge DB/backup work; local files permit bounded streaming and separate quotas but require staged write, safe names/paths, metadata commit and orphan reconciliation. Proposed first-slice role: local file objects plus SQLite metadata, explicit manifest/hash/size in exports. No file implementation, path/permission validation, power-loss rename or attachment backup test was executed here; #22 must validate those before adoption. Game executables/save formats and redistribution remain per-title #34 concerns.

Physical backups should use a consistent snapshot/backup facility, not copy only a live main DB file. [SQLite backup API](https://www.sqlite.org/backup.html) documents the supported snapshot mechanism. Keep filesystem snapshot/file-object manifest coordination explicit for later attachments. A successful restore into a fresh directory with record counts/hashes and search rebuild is the acceptance condition; a backup command returning success alone is insufficient.

## Experiment, commands and controls

[Plan 0005](../plans/0005-runtime-storage-decisions.md) predeclared comparable journals, fixed seed/cache/sync, retries, contention, snapshots, process crashes, export, FTS deletion and backup/restore. Run:

```sh
python3 experiments/009-local-storage/probe.py --run
python3 experiments/009-local-storage/probe.py --check
python3 -m unittest discover -s tools -p 'test_*.py'
```

Python stdlib SQLite is the **driver**, not the product runtime choice. This session used Python 3.14.7 / SQLite **3.53.4** on Darwin arm64, not the system CLI's different SQLite build. No Rust binding/storage integration was tested. Each of ten independent temporary DBs seeded 1,000 synthetic members and 100,000 posts with 1,024-byte bodies in 50 rooms, including a room/id index and FTS5. Five runs/journal mode, same fixed seed. Cache suggestion `-2048` KiB per connection, mmap off, foreign keys on, FULL synchronous and `fullfsync=ON`; settings checked where meaningful. The 2 MiB cache suggestion is not a total memory bound. Record actual VFS/storage/compile options before transferring durability claims to another host.

Each run times 400 indexed two-post page reads and 100 individual guarded commits, then exercises eleven oracles. Writer lock timeout is 100 ms. Held reader snapshots test different journal behavior. Child writers are killed after an explicit before-commit or after-commit checkpoint; the reopened DB must pass `integrity_check` and show the expected transaction outcome. Crash checks are process kills on a live OS, **not power cuts or malicious/corrupt-disk fuzzing**. Generated binaries/databases/exports/backups are removed; [raw observations](results/009-local-storage.json) preserve source hash, versions, timings, sizes and named outcomes only.

## Actual results and useful failure

| Observation | DELETE journal | WAL |
| --- | ---: | ---: |
| Median indexed page read (pooled five runs) | 0.016 ms | 0.006 ms |
| Median individual FULL commit | 11.048 ms | 3.861 ms |
| Main DB logical bytes after workload | 142,995,456 | 142,995,456 |
| Retained sidecars at sampled point | None | 94,792-byte WAL + 32,768-byte SHM |
| Held reader during write | Commit returned busy until reader ended | Writer committed; reader retained prior snapshot |
| Eleven recovery/correctness oracles | Passed in five runs | Passed in five runs |

Timing distributions, actual file allocation (`st_blocks*512`) and all repeated samples are in raw JSON. Filesystem allocation exceeds logical file sizes here; do not substitute file length for disk use or extrapolate this host's results to SD cards. There were 100,103 final posts/run after the test transactions. Workload content is repetitive, FTS/storage growth is not representative of arbitrary community text, and the complete #28 account/chat/file seed was not used. These are not 250-concurrent-user DB timings; the active workload was sequential.

The eleven positive/negative oracles: same request/content returns same identity; mismatched request reuse rejects; revoked author denied before retry lookup; second writer fails within its configured busy policy; snapshot behavior as specified; killed precommit absent; killed postcommit retained; deleted sentinel absent from FTS; online backup restored with same domain-record digest; explicit export restored with same digest and rebuilt FTS; reopened/restored DB integrity checks pass. Small unit tests also check transaction rollback, parameter-bound injection-like text and revocation preceding duplicate lookup.

**Contrary evidence retained:** the first attempt used Python `iterdump()` and restoring that generic SQL dump failed with `no such table: search` for this external-content FTS setup. That experiment did not complete its export oracle. We replaced that path with an explicitly versioned JSON-lines export of members/posts, creating the schema and rebuilding search on import; all final runs passed. This does not prove SQLite CLI `.dump` or all FTS backups fail. It demonstrates that our portability contract must be deliberately defined/tested rather than assumed from a generic dump. The prototype export has no hostile-input limits/encryption/attachments/migration compatibility yet and is not a supported format.

## Gates, confidence, decision and reversal

Experiment G1 pass (Python stdlib/SQLite, no added JS); G2 partial scoped transaction checks pass, full authentication/per-room/search/file authorization **unknown**; G3/G4/G5/G6 pass for local synthetic design, while production retention/export/encryption remain unknown; G7 not applicable to this storage-only probe, full terminal integration unknown; G8 not applicable (no extensions); G9 authored source BSD and [SQLite public-domain dedication](https://www.sqlite.org/copyright.html) documented, production binding/dependency audit pending. Gate failures cannot be scored away. No untested DB receives a zero or measured ranking.

Confidence: high in exact scoped outcomes, moderate in SQLite WAL as the first integration candidate, low in actual SBC power-loss durability/secure storage resource cost. Require a pinned maintained SQLite build, review relevant fixes (including the documented [WAL-reset fix in 3.51.3](https://www.sqlite.org/releaselog/3_51_3.html)), and pin/audit the eventual Rust binding before production. Version age alone is not a vulnerability audit.

Propose SQLite WAL/FULL, bounded writer/reader work, derived search, versioned logical export and consistent physical backups for the first Rust integration. [ADR 0005](../decisions/0005-rust-sqlite-validation-slice.md) records scope. Reversal low now, moderate once the core storage interface is established, high after deployed schema/formats; export/restore must remain usable to switch engines. Reopen on measured writer backlog/latency, WAL growth, unsafe platform/VFS behavior, footprint failure or an unfixable binding/dependency issue. #9 remains open for file/storage/retention integration and comparison follow-through; no federation persistence model or release support is accepted here.
