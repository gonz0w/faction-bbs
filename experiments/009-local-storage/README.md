# BBS-009 SQLite recovery and export probe

Python stdlib research driver, not the selected product runtime. Uses synthetic data
and independent temporary databases/backups/exports. No real accounts, secrets or
network service. Ten 100,000-post databases are created sequentially and removed.

```sh
python3 experiments/009-local-storage/probe.py --run
python3 experiments/009-local-storage/probe.py --check
```

Requires Python SQLite with FTS5 for fresh runs. Snapshot/process-kill tests do not
simulate power loss, filesystem corruption, full disk or physical media guarantees.
The record-export fixture lacks hostile-input limits/encryption/attachments and is
not a supported interchange format. Generic SQL-dump restore failed during research;
the final probe rebuilds derived indexes from explicit record exports.

See [options, actual results and unresolved tests](../../dev-docs/research/0009-local-storage-comparison.md).
