# Backup checks and the Recovery panel

Updated 2026-10-22. `faction-sysop` now has a **D:Data** panel for historical
checks of an operator-selected complete file-area bundle. It does not open a
backup, read the live database, run a privileged command or start a restore.

## Check a complete bundle

Use the existing [complete backup workflow](file-areas.md#complete-backup-and-fresh-restore)
for a stopped/drained backup. The helper now also supports **verify**, which checks
an existing catalog, manifest and every file byte without creating restore paths
or acquiring/changing live PID/transfer locks. A verify-only check may run while
the board is operating because it reads the selected private bundle, not the live
DB/store. Keep the bundle closed/unchanged while checking it.

As an authorized root operator, using the board owner's actual UID and paths:

```sh
python3.13 tools/file_area_backup.py verify --owner-uid OWNER_UID --bundle /home/factiond/backups/bundle-001 --faction /usr/local/libexec/faction
```

This runs the existing Rust catalog verifier under the non-root owner and checks
all manifest lengths/SHA256 values against the catalog and saved objects. It
rejects missing/corrupt/extra objects and unsafe files. No file UID is needed for
verify-only. Backup and restore still require distinct non-root owner/file UIDs;
backup keeps its stopped/drained locks and restore still needs fresh paths.
Neither a valid SHA256 nor structural verification proves a sender's authenticity.

## Explicitly record the observation

Publication is optional and defaults off. Add `--record-status --board PATH` to
**backup** or **verify** to write a small private check receipt:

```sh
python3.13 tools/file_area_backup.py verify --owner-uid OWNER_UID --bundle /home/factiond/backups/bundle-001 --board /var/db/faction --record-status
faction-sysop --state-dir /var/db/faction --view data --ansi
```

A successful recorded backup also rechecks the completed bundle before marking
it verified. With file-area data installed, a standalone `faction export` contains
only the catalog and cannot claim a complete bundle check. It does not write a
receipt. Restore never copies telemetry into the recovered board. The new option
is refused for restore; a restore rehearsal is a separate operator action.

The private `backup.status` contains only version, attempt/result timestamps and
last-success catalog/object byte/count totals. It contains no names, paths,
handles, message content, hashes, passwords or keys. The dashboard reads at most
512 bytes using the same owner/root-private file boundary as service telemetry.
The writer atomically replaces a synced mode-0600 owner-owned file. No schema,
listener, owner helper, host configuration or BBS/member privilege changes.
Receipt publication is independent of `operator_status`: the common **Board:**
health line describes service snapshots, while the Data panel gives check ages.

## Read the panel honestly

- **VERIFIED** records the selected artifact check as of the shown time. The
  displayed age keeps increasing; there is no invented backup schedule/deadline.
- **FAILED** is the latest explicitly recorded attempt. Earlier last-verified
  time/totals remain visible; failure does not relabel them as current success.
- **UNRECORDED** means no receipt exists. **UNAVAILABLE** means it cannot be read
  safely or its version/fields are invalid. Future timestamps say **CLOCK UNKNOWN**.
- Pausing the dashboard also pauses receipt reads. Normal modest refresh, themes,
  compact/plain fallback and optional Unicode host graphs remain available.

The operator explicitly associates the chosen bundle with `--board`. This is not
an automatic comparison with the live board's identity or latest transactions.
A receipt does not prove the artifact still exists or is unchanged, was copied
off-host, includes all installation data, or has restored successfully. Themes,
OS/SSH configuration, remote door saves and member-device mail keys stay outside
this bundle. Repeat verify and rehearse a fresh restore when needed; maintain
separate protected copies and retention policy. Device mail recovery stays local.

Receipt failures return an error; they do not affect the BBS or remove an earlier
receipt. An interrupted writer can leave `.backup.status.next`; after all backup/
check writers stop, inspect that one owned staging file before removing it.
Never unlink live PID/credential/transfer locks to recover telemetry.

## Validation and remaining scope

[Plan 0054](../dev-docs/plans/0054-backup-checks-and-recovery-panel.md) records
actual developer and native NetBSD software checks. Native validation exercises
root CLI commands with distinct non-root owner/file UIDs, corruption refusal,
last-success retention, a real fresh restore/login and the installed Rust panel.
Developer API fixtures use one test UID and do not establish role separation.
#95 peer telemetry still needs real adapter receipts; host applies/firewall and
hardening remain later. No physical SBC or new 250-user capacity claim.
