# Other BBSes directory: actual NetBSD software checks

2026-10-22, #27/#21. [Sanitized results](results.json) record a fresh owned
2-core/1-GiB NetBSD 11 ARM64 software clone. See [plan 0055](../../plans/0055-local-other-board-directory.md),
[ADR 0050](../../decisions/0050-curated-local-other-board-directory.md) and
[operator/member workflow](../../../docs/other-boards.md). The saved playground
stayed off; the disposable clone and temporary vendor cache were removed.

## Executed validation

- 188 Rust tests per OS, native Rust 1.96.0, host Clippy with warnings denied,
  formatting and 118 host Python tests pass. Strict pack/token/date/file checks,
  maximum catalog paging and Files+Contacts full-grid fallback are covered.
- Eleven local/worker PTY controls per OS: default disabled, pending denial,
  offline check, highlight/hotkeys/pages/cards/main return, compact/plain,
  inherited worker, resize, revocation, bad endpoint and a listening loopback
  trap that receives no connection when its endpoint is displayed.
- Twelve actual maintained shared-service controls use restricted base OpenSSH
  and the native client: disabled/default, 32-listing arrows/hotkeys/pages/cards,
  native return, compact paging, resize, resource samples, pending registration,
  current approval/revocation, plain output, OLD review dates, invalid/unsafe pack
  preflight and removal/empty directory after stopped restart. No remote login,
  automatic connection, profile import or federation is exercised or claimed.
- Regressions: eight themed operator UI controls and eleven backup/check/restore
  controls per OS, ten shared operator controls, four serial/getty controls,
  seven actual restricted SSH/native artwork routes, eighteen native artwork
  previews and seven local/worker artwork routes. Native backup commands use
  distinct non-root owner/file UIDs and installed binaries under actual root CLI.
- All four standalone experiment manifests pass locked all-target compile checks.
  The shared-UI shim treats Contacts as unavailable, like Terminal; these checks
  do not mean that old experiment implements the new directory or was remeasured.
  Local OpenSSH offline config expansion accepts the displayed IPv6 argument form.

The shared fixture needed two harness corrections before its passing run: shared
compact/plain login still has an Enter pause, and registration uses the maintained
hidden-password/pending prompts. Failed attempts produced no passing result and
were kept private. These were fixture expectations, with no runtime relaxation.

## Separate resource observations

With the maximum 32 listings loaded and one approved directory member, twelve
half-second samples show **5,312 KiB owner RSS / 26,100 KiB VSZ / four LWPs**;
minimum whole-VM free is **352.36 MiB**. The separate default-disabled directory
operator regression samples **2,440 KiB dashboard RSS / one LWP**, **5,196 KiB
owner RSS / four LWPs**. The runs differ in workload/history and are not an
incremental-memory estimate. VM free includes native-build/cache effects.

The immutable startup catalog is shared by Arc; members retain a pointer and
selection/card/offset cursor. Pack <=32 KiB / 32 entries, four listings per page,
card frame <=8 KiB. Normal eight sessions, two credential permits, three owner
helpers/four LWPs, 32-KiB queued output and five-second drain remain unchanged.
No new registry dependency, per-member connector, owner helper or listener.
These are short software observations, not peaks/soaks, physical SBC, all NetBSD
ports, current 250-member capacity, visual acceptance or hardening certification.

## Reproduce and preserve

Use a new owned marked disposable clone, reviewed locked/vendor sources and an
unprivileged native builder; never the saved/live board. Run workspace release
tests/builds and `tools/smoke_directory.py --help` for both local binaries. Install
normal owner/service/TTY/sysop binaries and use
`tools/smoke_shared_directory.py --help` with private fixture argv/password/profile
files. It requires `/etc/faction-disposable-test`, the synthetic profile-board path
and a fresh dedicated directory fixture path. It stops/restarts only that fixture.
Run operator, distinct-UID backup, serial and artwork regressions separately.

165 selected native build/runtime/validation source hashes match the actual copied
tree. Seventeen relevant host harness/compatibility inputs and installed/host
binary identities are pinned separately; this is not every host test/build input.
The standalone shim is compile-only, not a native executed source. Verify owned
QEMU PID and disk identity, stop it, wait for closed disk handles and remove only
its task-owned clone/cache; confirm the saved original has no open handle.
Private credentials, serial transcripts and admin argv never enter this record.

Snapshot 22, `sources/pre27-directory`, and
`operator-recovery-baseline-manifest.json` freeze 173 distinct previous execution/
harness/build inputs and the complete original recovery record. All 22 snapshots
and 35 historical result identities remain unchanged. Preserve this new source/
harness set and whole result before future edits; never refresh old hashes.
