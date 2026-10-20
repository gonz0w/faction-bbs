# Installed shared-service validation (#55)

This directory holds versioned NetBSD collectors for the maintained
`faction-service`/`faction-tty` process names. Historical collectors remain unchanged.
Rust application code lives in `crates/faction-service`; native terminal-only code
lives in `native/faction-tty`. This is validation, not another service candidate.

Use a disposable, private NetBSD 11 ARM64 VM, two vCPUs / 1 GiB, with the explicit
profiling marker, isolated administration/member keys and independently seeded
synthetic board. The existing seed/reset/probe helpers under `055-shared-ui` remain
versioned inputs; owner credentials/transcripts/VM images must not be committed.

Build/test the root workspace with Rust 1.96, then build the frontend using
`python3 tools/build_terminal_frontend.py --output target/release/faction-tty`.
Install the actual binaries, root-owned service config/rc.d scripts and restricted
SSH template using the [operator guide](../../docs/service-lifecycle.md). Compile
these collectors and the existing credential calibration / allocator / host probes
natively with `cc -std=c11 -O2 -Wall -Wextra -Werror`.

- `tools/check_integrated_lifecycle.py`: real default-eight admission, ninth
  denial after token refill, strict configuration, expiry, stop/restart/HUP,
  persistent lock identity and worker rollback.
- `tools/check_integrated_oracles.py`: 27 complete UI/permission/terminal/fault
  checks through installed rc.d; hostile host-side writes are nonblocking, and
  queued attack input is discarded before fresh useful login.
- `frontend_probe.py`: 20 direct native FD/terminal/argument/filesystem checks
  against installed `faction-tty`, alongside real SSH denial tests.
- `tools/check_service_host.py` / `check_service_scaling.py`: actual rc.d
  start/status/stop, explicit root-owned 250-session config, distinct approved
  SSH members at 32/100/200/250, full UI at 250, real credentials, known touched
  memory controls, complete timing/counters/live resource limits and fresh-login
  recovery. Requires the explicit experimental allocator/pageable-time fixture.

Drivers require `--disposable-fixture`, host/ports, key/known-host paths and an
output path. UI/scaling drivers also require a private `--secret-file`; scaling
uses `--model shared --allocator compact`. These are fixture inputs, not a public
endpoint or deployment recipe. No script installs host tuning automatically.

The unchanged batched guard and complete sample matrices are mandatory. Sampling
does not prove an unsampled worst case or sustained capacity. Restore/read back
host settings after reboot, reset/integrity-check synthetic state offline, and
remove the private VM/keys/disks/logs/forwards after collecting sanitized results.
The strict saved-evidence checker is `tools/check_integrated_evidence.py`; CI does
not rerun a VM. Production confinement, active mixes/soak/admission and maintained
image/timekeeping remain open.
