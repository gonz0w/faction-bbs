# Repeatable headroom experiment (#58)

**Marked disposable NetBSD 11 ARM64 software fixture only.**
The [report](../../dev-docs/research/0031-repeatable-headroom.md),
[ADR 0017](../../dev-docs/decisions/0017-repeatable-headroom.md) and
[plan](https://github.com/gonz0w/faction-bbs/issues/58) define the experiment.
Normal services remain eight workers; this is not an installer or support claim.

## Candidate and evidence

The standalone Rust `backend.rs` copies the corrected 058 backend and fixes a
credential rendezvous race: a second valid login could be rejected before a
worker started waiting. Two owned permits now span queued, active and unconsumed-
result work together. Both handoff capacity and aggregate outstanding work are
two. Permits survive client disconnect and release on result disposal, failed
handoff and shutdown. The deterministic lifetime regression covers these cases.
Two fixed workers, the cross-process kernel budget, Argon2, approval/epoch/grant
checks, connection generations and terminal/owner UIDs stay unchanged.

Final trials combine this candidate, the prior hardened minimal handoff frontend,
compact native allocation and NTP's supported pageable-memory option. NTP and
all Postfix/mDNS/inetd/logging/DHCP/cron services remain present. Three fresh boots
complete 250-member real SSH/menu/chat, member/owner credentials and calibrated
extra-memory work under the unchanged guard. All eight exploratory records,
including the prior busy rejection, stay separate from those final passes.
The original NTP config/default allocator/limits are restored before cleanup.
Clock precision under sustained pressure and scoped deployment remain #20/#55
gates; active mix/soak/budgets/transport-flood bounds remain #28.

## Sources and tools

| Source/tool | Purpose |
| --- | --- |
| `backend.rs` / standalone manifest/lock | Bounded credential lifetime and shared terminal candidate; Rust safety checks retained. |
| `live_limits.c` | Root-only read-only CTL_PROC soft/hard limits for every live frontend/owner/dedicated SSH process. |
| `allocator.c` | Effective native allocator options; no configuration writes. |
| `slab_probe.c` / `dss_probe.c` | Effective bin geometry / DSS preference for rejected allocator alternatives. |
| `host_memory.c` | Service RSS/virtual counters and ntpd locked virtual mappings; no PIDs/paths/addresses. |
| `tools/check_headroom_scaling.py` | Frozen admission/credential/known-allocation workload with exact guard decisions. |
| `tools/check_slab_scaling.py` / `check_dss_scaling.py` | Native effective-geometry/preference checks around that same workload. |
| `tools/check_host_scaling.py` | Verify unlocked time-daemon mappings and all base-service presence before/after the same workload. |
| `tools/check_chat_reentry.py` | Three fresh-marker actual SSH chat cycles; detects a stale same-name executable. |
| `tools/check_headroom_evidence.py` | Recheck saved complete sample matrices, limits, identities, guards and cleanup; no VM execution. |

All registry packages/features remain in the reviewed 39-NetBSD/41-Linux
inventory. No new runtime, crypto, SSH or JS/TS dependency is introduced.
Per-process RSS sums/virtual ranges are never private memory. Equal live-limit
rows are grouped with explicit counts; every sampled timing/counter row remains.

## Build and rerun protocol

Use Rust 1.96 and the checked-in lockfiles. **Keep each standalone manifest in a
separate trusted builder-owned target directory.** Shared output initially reused
a stale same-name executable; a clean release rebuild matched an isolated build,
and the actual repeated-chat regression distinguished it from the stale binary.

From the repository root, with Rust 1.96 in PATH:

```sh
cargo test --manifest-path experiments/058-headroom/Cargo.toml --target-dir target/headroom --release --locked
cargo build --manifest-path experiments/058-headroom/Cargo.toml --target-dir target/headroom --release --locked
python3 tools/check_headroom_dependencies.py
python3 tools/check_host_scaling.py --help
```

Native reruns require the signed private 2-vCPU/1-GiB fixture described in
[research 0024](../../dev-docs/research/0024-restricted-ssh-netbsd.md) and
[the prior transport experiment](../058-transport-memory/README.md): separate
owner/terminal UIDs, protected actual descriptor handoff, root-managed restricted
SSH listener and marked synthetic profile board. Keep host/private-network keys,
generated common fixture password, transcripts, disks and forwards private.

1. Build the normal workspace and `profile_seed` example, plus this manifest in
   its own target. Install the separately identified candidate at the fixture's
   protected `shared-bbs` path. Build the unchanged prior minimal frontend with
   its hardened flags/native descriptor bridge; preserve the original frontend.
2. Compile each C probe with native `cc -std=c11 -O2 -Wall -Wextra -Werror`.
   The fixture drivers expect root-managed libexec names `faction-live-limits`,
   `faction-headroom-allocator`, `faction-slab-probe`, `faction-dss-probe` and
   `faction-host-memory`, plus the unchanged 056 metrics/calibration helpers.
   Native positive FD, actual SSH denial, approval/grant, terminal and failure
   suites must pass before profiling. Never substitute the owner CLI as an SSH
   forced command. Record installed binary SHA-256 identities and three-cycle
   real chat re-entry, not just source hashes.
3. Seed the marked synthetic board: 1000 distinct approved profiles plus sysop,
   independently salted hashes, 50 rooms, 100000 1-KiB posts. Use the private
   password file through stdin/PTY, never password arguments or environment.
   Reset only the marked offline fixture before each fresh boot. Preserve live
   credential/pidfile lock inodes.
4. Compare effective native compact allocation (one arena, tcache off, zero
   dirty/muzzy decay, retain false) with default locked versus pageable NTP.
   Save the original NTP config/hash; the guest-only pageable variant appends
   `rlimit memlock -1`. Reboot and verify the running time daemon has no locked
   maps while all base services remain. For the rejected alternatives, use the
   exact retained/minimum-slab/DSS settings and effective readbacks in the ledger.
   Settings are guest-wide experimental inputs and are absent from normal setup.
5. Run `check_slab_scaling.py` for the matched compact baseline and
   `check_host_scaling.py` for each of three fresh pageable-time repeats, with
   `--disposable-fixture --model shared --allocator compact`, private host/port/
   identity/known-hosts/password-file arguments and default 250-session target.
   The driver saves the actual guard at each batch/stage, live limits, failures,
   credential/control streams, stop/reap and recovery. It restores temporary
   guest global limits in `finally`; it never adjusts developer-host sysctls.
6. Keep all attempts and source/binary sets, including failed exploratory work.
   Recheck original NTP/default allocator/limits restoration, SQLite integrity,
   zero leftover application/SSH children and fresh login. Stop/remove only this
   task's VM, disks, generated keys/passwords/raw logs and loopback forwards.

The guard stays free + max(0, clean file − wired) ≥ 128 MiB with zero swap and
strict process/file reserves. Batch checks and sampled peaks are observations,
not continuous guarantees or instantaneous upper bounds. Two successful password
checks alone do not establish overlap. Extra touched 0/19/38-MiB controls qualify
the sampler; fresh owner processes still share a warm host/cache. Complete UI,
fairness, clock/update policy and physical SBC support require later work.
