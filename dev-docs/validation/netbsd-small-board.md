# NetBSD small-board installation and member journey

Date: 2026-10-21. Issue: [#71](https://github.com/gonz0w/faction-bbs/issues/71).
Actual observations: [sanitized execution record](netbsd-small-board.json).
User instructions: [NetBSD quickstart](../../docs/quickstart-netbsd.md).

## Executed scope

An official NetBSD 11 ARM64 disposable QEMU/HVF VM, two vCPUs / 1 GiB RAM,
ran a fresh ordinary installation of the existing Rust application at merged
`befe804f2a31ee6a500bfea929e80bdc8b4de649`, plus the current native startup helper,
installer and rc scripts identified by the JSON execution-source hashes.
No new application Rust code changed. Built binaries and compiler identity are
recorded. The helper compiled with strict warnings/hardening and passed native
GCC static analysis. Root execution and an arbitrary backend command were denied;
all installed product executables are root-owned 0755, with no setuid bits.

Both read-only preflight and explicit installation passed. An existing deployment
was refused; preflight created no account/state. A safe-loopback first installation
and a final clean installation on an explicitly selected private VM interface
left both services disabled. Administrative SSH and host network policy were not
modified by the installer. Earlier research data was archived privately before
fresh fixture setup, never silently rewritten.

The final ordinary SSH rehearsal passed **16 checks**: disabled-by-default install,
interactive sysop bootstrap, backend/dedicated SSH startup, private database/UID
boundary, pending registration before board identity/content, approval through
ANSI sysop controls, customized identity and arrow/Enter navigation, hotkey post,
peer durable read, two-member local chat, normal restart, completely unassisted
fresh boot and approved login, worker rollback, return to shared, private
export/verify, and credential non-echo. No diagnostic/manual start intervened in
the final boot check. Shared backend/eight-session defaults were retained.

The existing native `smoke_export_restore.py` also passed its real owner-console
live-WAL/PTY export/restore, approval, no-overwrite and failed-restore cleanup
rehearsal. The application uses the original two credential permits/kernel locks
and unchanged Argon2. This increment does not remeasure them as a capacity claim.

## Failures retained and corrected

The first SSH attempt used the host VM forward against the safe loopback listener;
remote forwarding cannot reach that guest-only address. The final fixture selected
one private guest interface explicitly, with no wildcard fallback.

An early arrow-key test incorrectly expected a complete menu redraw; the product
correctly redraws only changed rows. The test now observes the changed rows and
Enter's actual room-list action.

The first complete boot rehearsal exposed ordinary backend SIGHUP loss after boot
session teardown. The unprivileged startup helper detaches before backend exec.
A subsequent boot exposed an SSH/DHCP bind race. The first bounded-retry candidate
mistook the daemonizing parent's success for binding; a manual diagnostic start
during that candidate invalidated its apparent success. The final candidate
checks the actual listener pidfile and its completely unassisted boot passed.
A later export correctly refused an existing backup from that invalidated run;
the final fresh-install fixture had no previous backup. No overwrite behavior
was relaxed. Do not count failed/corrected fixture attempts as successful runs.

## Host profile and limits

Original host NTP was restored, the experimental global allocator link removed,
and experimental NPF boot policy, isolated setuid frontend, key-admission and
global-admission profiles disabled before this ordinary product validation.
The normal board does not require those research profiles. All old capacity
records retain their original measured sources and hashes; they did not measure
the new normal startup helper/rc scripts.

This is a small-board hobby preview setup/member/recovery check. It is not a
250-member test, independent security review, long soak, packaged upgrade manager,
physical SBC result or all-NetBSD-port support promise. Further #70/#55/#20
combined pressure/NPF/host investigation is deferred by owner direction.
Private VM inputs and raw paused research are not publication artifacts.

After validation, the task VM was stopped and removed, its two host forwards
closed, and its generated credentials/disks deleted. Only deferred synthetic
observations/source maps remain in a private owner-only temporary archive, with
the separate durable local Git research checkpoint. Unrelated VMs were untouched.
