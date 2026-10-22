# Live host counters and sysop-console polish on NetBSD

2026-10-22, #95/#18/#98, [ADR 0048](../../decisions/0048-native-operator-host-counters.md).
[Sanitized results](results.json) cover an owned disposable two-core/1-GiB NetBSD
11 ARM64 software clone. The saved playground stayed off throughout.

## Executed validation

- 183 Rust tests per host/native OS; native release uses Rust 1.96.0. Host Clippy
  denies warnings; formatting and 115 Python tests pass.
- Eight themed operator UI controls per OS: plain/one-shot/ASCII-only Unicode
  override, stale/future/stopped/missing data, private-file denial, refresh bounds,
  panels/arrows/pause, real 40x18 resize, reserved-bottom-row exit/termios restore,
  interval readings and optional UTF-8 block graphs. Unsupported/oversized metrics
  remain unavailable. Deterministic Rust checks cover reset/zero/scope changes,
  exact CPU/network delta math, invalid FFI fields and bounded recent-history tail.
- Ten actual shared-operator controls include default-disabled publication,
  opt-in service start, real member-UID denial, live counts with approved member
  SSH, native host capacities/interval rates, owner PTY/resize, independent resource
  samples and graceful stop/restart. Aggregate owner limits remain unchanged.
- Four virtual serial/getty controls authenticate the normal OS owner with a
  synthetic clone-only password, launch the installed dashboard, switch panels,
  return to the reserved row with identical termios, then logout to getty. No
  autologin, login-profile/getty installation or physical-terminal claim.
- Existing artwork regression: eighteen native previews, seven local/worker
  logins and seven actual shared SSH/native login routes. No new pixel/font claim.

The native bridge uses no refresh subprocess: 64 scalar slots, one fixed 64-KiB
network buffer and sixteen interface entries. The VM has one non-loopback
interface; the Mac exceeds the fixed cap and correctly reports network unavailable.
Native runtime checks observe actual second-sample CPU/network values. RAM used
includes cache/kernel reservations; traffic sums interface counters and can count
tunnel/bridge packets more than once. First/reset/scope-change samples warm up.
Peer/backup collectors and independent process-liveness verification remain open.

## Resource observations

Twelve half-second samples with one member and default-palette dashboard:
**2,440 KiB dashboard RSS / one LWP**, **5,144 KiB owner RSS / four LWPs**;
minimum whole-VM free memory **332.66 MiB**. Native build/cache activity precedes
sampling. These are short observations, not hard ceilings, cache-normalized
comparisons, full peak/soak measurements or new 250-member capacity promises.
The themed smoke uses Nightwire separately. No session/credential, allocator,
clock, firewall or host tuning is applied.

## Reproduction and immutable inputs

Follow the [initial operator fixture procedure](../operator-netbsd/README.md)
using a fresh owned clone and current locked sources. `tools/smoke_operator.py`
runs locally and natively; `tools/smoke_shared_operator.py --help` documents the
marked shared fixture. After its stop/restart cleanup, start the fixture normally
for the console test. `tools/smoke_operator_serial.py --help` requires the owned
controller's private log/FIFO and a synthetic OS-owner login/password file. It
never boots an image, sets passwords, installs profiles or publishes a transcript.
Guest/host credentials, argv, keys and logs stay outside Git. Existing shared art
regressions run separately and stop the backend afterwards.

All **155 selected native source inputs** were hashed against the actual copied
execution tree; **nine harness inputs** are pinned separately, with installed
native and host binary identities. The twentieth snapshot `sources/pre95-hoststats`
freezes all 162 distinct original operator/build inputs and the whole original
result before runtime edits. Historical hashes are unchanged. Preserve these new
measured bytes and this complete record before future edits.

After validation, verify the owned QEMU PID/disk identity, shut down that clone,
wait for exit and closed image handles, remove only its temporary clone/vendor
cache, and confirm the saved original has no open handle. Teardown flags in the
record represent actual cleanup, not planned actions. No host-action/hardening,
public listener, member OS privilege or physical SBC support claim.
