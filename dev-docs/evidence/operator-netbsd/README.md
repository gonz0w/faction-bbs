# Local operator console on NetBSD

2026-10-22, issues #95/#18/#98. [Sanitized results](results.json) record a fresh
software-only two-core/1-GiB NetBSD 11 ARM64 clone of the privately saved playground.
The saved playground stayed off. No physical SBC or fresh 250-member claim.

## Actual checks

- 179 Rust tests pass per host/native OS; native locked release builds use Rust
  1.96.0. Host workspace Clippy with warnings denied and formatting pass.
- Six themed operator-UI controls per OS cover plain one-shot, stale/future/stopped/
  missing data, unsafe snapshot modes, refresh bounds, panels/arrows/pause, actual
  40x18 resize, terminal restoration and plain Ctrl-C exit with zero ANSI controls.
- Ten actual native shared-operator controls cover default-disabled publication,
  enabled rc startup, denial under the real member UID, fixed host collectors,
  approved restricted SSH and aggregate counts, OS-owner PTY panels/SSH resize,
  separate resources, exit/logout, graceful stopped status and restart replacement.
- Existing original-art preview/login and restricted SSH/native artwork checks
  remain separate regressions against the new runtime. No new art/font/pixel claim.
- SQL/privacy/file/export tests prove budget cancellation restores ordinary
  queries/timeouts, numeric aggregate output excludes names/passwords, unsafe
  snapshots fail, publication failure retains the earlier sample, and export stays
  independent of disposable telemetry. Schema/backup format stay unchanged.

## Resources and scope

Twelve half-second observations with one connected member and a plain-default
palette operator UI sampled **2,396 KiB dashboard RSS**, **5,144 KiB owner RSS**,
one dashboard LWP and the original four owner LWPs. Minimum sampled whole-VM free
memory was **334.23 MiB**, after native build/cache activity. Values are short
observations, not limits, incremental memory deltas, a soak, cache-normalized
benchmarks or support promises. Text counts update every five seconds. The
separate themed UI smoke verifies Nightwire loading; this resource sample used
the default palette, not a claim of every theme's memory cost.

The occasional fixed sysctl child is outside the owner and exits per sample.
Per-process polling does not establish that transient child's peak RSS; whole-VM
free pages were sampled alongside both processes. No allocator/kernel/NTP/session
retuning was applied. Resource ceilings and defaults remain unchanged. No new
resident owner helper/listener, retained member history or external registry
package is introduced.

Host observations report actual CPU count/load, physical RAM **capacity** and
board-filesystem space. Used/free RAM, CPU utilization, network, peer, backup and
service liveness collectors are not invented. This is a first read-only slice;
#95 remains open. No host-action/firewall/hardening or VM-getty autolaunch claim.
The operator test uses an authenticated OS-owner PTY through the clone's private
administrative SSH endpoint, distinct from the restricted BBS member endpoint.

## Reproduction and preservation

1. On an owned disposable clone with reviewed locked dependencies, copy the source
   tree, build/test the workspace and terminal examples as an unprivileged builder.
   Run `tools/smoke_operator.py` against the native `faction-sysop` with a private
   temporary state and owned Nightwire pack. Never migrate/boot the saved original.
2. Prepare synthetic approved membership and restricted SSH using the existing
   ordinary eight-session service fixture. Keep credentials/known-hosts/admin argv
   outside Git. Mark the clone `/etc/faction-disposable-test`.
3. `tools/smoke_shared_operator.py --help` takes explicit private admin/member argv,
   password and output paths plus `--disposable-fixture`. It enables telemetry only
   in this marked fixture, exercises owner/member access, collects numeric status
   and resources, and stops the backend in cleanup. Use a **fresh** default fixture
   for its default-disabled assertion. Never reinitialize an already live board.
4. Run existing original-art native/SSH regression helpers separately. Collect
   source hashes by comparing the exact selected host files to copied native files
   and record installed binary identities. Preserve execution and harness bytes.
5. Shut down only the owned clone after verifying its QEMU PID/disk identity, wait
   for exit, remove its temporary overlay/reference caches and confirm the saved
   original image has no open handle. Never run an old collector/cleanup script
   that would rewrite historical results.

The nineteenth source snapshot freezes both original Nightwire records and all
176 distinct measured inputs before these runtime edits. Their whole result hashes
remain unchanged. Preserve this new record and measured inputs before future
changes; never refresh hashes to relabel old measurements as current execution.
