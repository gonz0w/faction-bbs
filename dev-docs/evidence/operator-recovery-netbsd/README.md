# Actual complete-bundle checks and Recovery panel on NetBSD

2026-10-22, #95/#18/#98. [Sanitized results](results.json) record an owned disposable
two-core/1-GiB NetBSD 11 ARM64 software clone. The saved playground stayed off.
See [ADR 0049](../../decisions/0049-historical-backup-check-receipts.md),
[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/95) and
[operator workflow](../../../docs/backup-checks.md).

## Executed checks

- 184 Rust tests per host/native OS; native release uses Rust 1.96.0. Host Clippy
  denies warnings, formatting and 118 Python tests pass. Receipt tests cover
  failure/last-success separation, bounded numeric protocol, busy/failed atomic
  publication, unsafe aliases and private permissions.
- Eleven actual bundle/check/restore controls per OS. Developer APIs use one
  fixture UID; native root commands use distinct non-root board/file identities,
  with actual Rust verification/export/restore dropped to the owner. The installed
  binaries are used; no builder-directory permissions are weakened.
- Checks exercise held owner-lock refusal, opt-in default, exact file bytes,
  verify-only with unchanged artifact/destinations, a connected local session,
  producer/consumer receipt agreement, corrupt recheck/last-success retention,
  no destination reservation on bad restore, valid fresh restore/login excluding
  telemetry, explicit recorded backup, unsafe/missing and clock-unknown states.
  Verify-only needs no file-service UID.
- Eight themed operator UI controls per OS, ten real shared-owner/member SSH/
  lifecycle/resource controls, four serial/getty login/launch/exit controls and
  seven actual shared SSH/native artwork regressions pass. Native artwork also
  passes eighteen previews and seven local/worker login routes. D:Data participates
  in keyboard/serial checks; full recorded rows fit the 40x18 unit geometry check.

## Resources and claims

Twelve half-second observations with one connected member and default-palette
console: **2,436 KiB dashboard RSS / one LWP**, **5,156 KiB shared-owner RSS / four
LWPs**; minimum sampled whole-VM free memory **333.37 MiB** after native build/cache
activity. The separate UI smoke loads Nightwire. These are short observations,
not incremental memory deltas, complete peaks/soaks or 250-member support claims.
The prior host-stat measurements remain unchanged, not silently relabeled.

The receipt is at most 512 bytes and numeric-only. The dashboard does not open
SQLite/backups/blobs or spawn an action/helper. Existing service defaults, two
credential permits, fixed owner helpers and drain/output limits stay unchanged.
Root backup/restore remain explicit existing owner operations; they never run
from a dashboard command field. No services or public listeners are auto-started.

A check applies to an operator-selected artifact at a historical time. It does
not authenticate provenance, match the current live board identity/transactions,
revalidate continued presence/content on refresh, prove off-host retention, include
all host/device/theme data or guarantee recovery. The actual fresh-restore test
is evidence for this fixture, not a universal restore promise. Peer health and
independent service-liveness proof remain uninstrumented; hardening stays later.

## Reproduction and preservation

1. Use a fresh owned software clone, locked/reviewed sources and an unprivileged
   native builder. Run workspace release tests/builds and operator/artwork UI
   regressions, then install the current normal owner/service/TTY/sysop binaries.
2. Run `tools/smoke_backup_recovery.py --help`. Developer runs call shared APIs with
   a single UID. Root runs require `/etc/faction-disposable-test`, explicit
   `--disposable-fixture` and distinct non-root `--owner-uid`/`--files-uid`. Use
   installed binaries accessible to the owner, not private builder output paths.
   The helper makes fresh temporary synthetic boards/stores and removes them;
   never point it at a saved/live board.
3. Run the existing marked shared operator fixture, serial/getty and shared art
   regressions separately. Keep passwords, private argv, serial transcripts and
   keys outside Git. Record per-process resources separately from whole-VM free.
4. Compare all **159 selected native input hashes** to the actual copied tree and
   pin **fourteen host harness/test inputs** plus binary identities. These are
   selected runtime/validation inputs, not a claim to pin every host test source.
5. Verify owned QEMU PID/disk identity, shut down that clone, wait for exit/closed
   disk handles, remove only its temporary clone/vendor cache and confirm no open
   handle to the saved original. Teardown flags report actual completed cleanup.

The twenty-first snapshot `sources/pre95-recovery` freezes 162 distinct original
host-stat/backup inputs and the complete original host-stat result before edits.
All historical hashes stay unchanged. Preserve this new measured source/harness
set and entire result before future edits; never refresh old hashes.
