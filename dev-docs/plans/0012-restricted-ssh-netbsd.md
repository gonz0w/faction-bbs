# 0012: Restricted SSH and native NetBSD application validation

Issues: [#23](https://github.com/gonz0w/faction-bbs/issues/23), #18/#19/#20/#30/#35.
Status: published for review in [PR #52](https://github.com/gonz0w/faction-bbs/pull/52), 2026-10-20. Owner requested merge #51 and continue this slice.

## Goal and boundaries

#51 merged as 6a0460a61a60fa985c41b4ef0649059ff26f3d76. Build actual SSH access to
current accounts/rooms/chat without exposing the owner console. OpenSSH authenticates
transport admission to a dedicated OS identity; Faction separately logs in/registers
and enforces current approval. Only owner-local tools bootstrap/recover/export.
No HTTP, public listener, overlay/provider or physical SBC required.

## Progress

- [x] Read current requirements, workflow, prior NetBSD evidence and issue #23.
- [x] Verify checks/head and merge #51; branch from current main.
- [x] Build database-inaccessible terminal frontend, Unix descriptor handoff/broker
  and separate member worker with no owner command parser.
- [x] Validate registration/login/approval/revocation, menus/chat, shell/exec/SFTP/
  forwarding denial, bounded startup/session work and disconnect handling.
- [x] Retrieve/verify/boot a fresh disposable NetBSD 11 ARM64 VM; native Rust build,
  complete app tests, all local PTYs/recovery plus real restricted SSH oracles.
- [x] Record primary-source decision, current setup/threat limits and sanitized
  evidence; update README/backlog and publish/attach PR.
- Required CI is tracked on the PR current head and must pass before merge.

## Design to falsify

Use a separate OS account for the OpenSSH frontend, with no database access.
Admit a single already-open, kernel-owned terminal descriptor over a protected Unix
socket; no arbitrary path/command/domain-RPC supplied by client. A trusted local
broker launches fixed member-worker executable/config with cleared environment and
that terminal as stdio. Change shared UI to duplicate stdin rather than require a
controlling /dev/tty. This reuses current ANSI menus/chat without embedding SSH or
adding a serialization/parser dependency. Owner console stays a separate binary.

Socket directory/group permissions and kernel terminal ownership gate local handoff;
they are not authenticated member identity. Faction password + approval are still
required. Initial fixed supervisor caps: eight concurrent workers, burst eight /
one new launch per five seconds, two-second handoff budget, 30-minute total session.
This is a bounded development gateway, not a 100/250-member capacity claim or full
process sandbox. Measure native compatibility before support claims.

Use only generated private VM/member data and temporary loopback-bound forwards.
Never modify host SSH configuration. Stop guest/QEMU and delete credentials/disks
after evidence extraction. Keep unperformed fault/resource/hardware gates explicit.

## Commands and outcomes

Rust 1.96.0: developer-host release fmt/Clippy (`-D warnings`), all 54 Rust tests,
all 23 Python tests, repo links/policy/archive and selected no-JS dependency source
scan passed. Four local real PTY workflows pass, including an inherited-descriptor
worker with no controlling /dev/tty. No new registry package; existing rustix fs/net
and cc 1.9.0 are used and inventoried.

Fresh native NetBSD 11 ARM64, 2 vCPU/1024 MiB QEMU/HVF: complete offline release
build and all 54 Rust tests passed under the non-root backend owner. Native test
mix includes a positive CMSG ownership/actual-data transfer regression. All four
local PTY flows passed natively; 20 real application SSH oracles passed. Sanitized
results/source hashes are in [report 0024](../research/0024-restricted-ssh-netbsd.md).
Guest source hashes were checked against the final developer sources. Temporary
keys/raw logs/disks/guest/forwards were removed; host SSH was never configured.

Discoveries:

- Pinned libc 0.2.191 NetBSD ARM64 CMSG alignment differed from native headers
  (12 vs 16 payload offset; 16 vs 32 one-right space). Real positive SSH failed
  before community access. The small faction-unix native-header C helper and
  documented Rust ownership wrapper fix that path without a registry fork.
- Native malformed 1/2/80-right probes preserve a real broker's fstat row count
  (6 before/6 after). The socket-data positive test prevents negative-only false passes.
- NetBSD ps heading suppression requires separate -o options. Exact process-name
  matching includes paths/trailing formatting; use a verified PID and require a
  live worker count before testing disappearance. Both corrections are in the driver.
- An empty command argument in the SSH CLI requests a shell channel; use a real
  blank exec request for the denial test. No claim of an independent empty-wire-
  exec/agent/X11/tunnel attack oracle is made.
- Two-member sample: about 3.8 MiB RSS per worker and 2 MiB per frontend, plus broker;
  shared pages/OpenSSH/OS/KDF peaks are excluded, so no whole-host/capacity claim.
- Broker supervisor stacks are bounded at 128 KiB and thread creation failures
  release the slot/connection. Default eight-worker cap is intentionally retained.

Remaining work: maintained rc.d lifecycle, complete output/broker-crash/signal and
load/resource accounting, account/key lifecycle/fairness, broader confinement and
physical SBC support. Native compatibility is demonstrated; no production release
or higher session count is authorized by this result. Record final PR/check status
on GitHub; broad linked issues stay open.

## Review handoff

PR #52 is attached to the project chat. Issues #18/#20/#23/#31/#35 have progress
and remaining-gate comments; none is closed by this slice. Branch
`codex/23-restricted-ssh-netbsd` starts from merged #51. Saved native evidence is
tied to 37 relevant source/config/lock/harness files, verified against the guest
before cleanup. Host-only docs/handoff changes do not alter that native source set.
No public deployment or automatic merge of #52 is implied. Next bound service
lifecycle/output/crash and complete host resource/load tests before raising the
eight-worker cap, while retaining the first-core/then-Lua sequencing.
