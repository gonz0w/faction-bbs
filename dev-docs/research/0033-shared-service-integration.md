# 0033: Maintained shared service on NetBSD

Date: 2026-10-20. Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55),
related #18/#20/#23/#28. Status: implementation and native software validation;
production capacity and physical SBC support remain unaccepted.

## Question and implementation

Can the full shared UI measured in [0032](0032-shared-ui-themes.md) become the
actual maintained NetBSD service path while retaining approval, SSH/TTY isolation,
worker rollback and the unchanged 250-member experimental headroom guard?

`crates/faction-service` is now a workspace member with a unique binary name and
strict bounded TOML configuration. It reuses the same core/terminal libraries and
the full nonblocking room/chat/sysop/theme implementation. Defaults remain eight
sessions, burst eight and one replenished launch token per five seconds. Higher
limits/faster admission require `experimental_capacity=true`; this is not capacity
certification. Fixed socket/lock identity keeps rc.d/SSH/status/stop coordinated.

The installed rc.d script selects shared mode by default, with services still
disabled until deliberately enabled. Worker rollback uses the same database schema.
Status/stop identify either actual known backend even after configuration selection
changes. Configuration/theme errors fail before listening; `--check-config` has
no listener/owner-operation side effects. The service refuses root/equal owner and
terminal identities. It never installs allocator, NTP or sysctl policy.

The tiny terminal handoff entry point is maintained as native NetBSD C in
`native/faction-tty`, built with a separate standard-library Python helper. It
links no database/KDF/domain code and holds the actual terminal/control descriptors.
Its native CMSG bridge is unchanged. Base OpenSSH and the separate terminal UID
remain mandatory. C introduces a manual bounds/lifetime review obligation; Rust
remains the core/server. This adopts a bounded native boundary, not a new general
application language or SSH implementation. No new registry dependency/features,
JavaScript/TypeScript, schema or weakened password costs are introduced.

## Provenance and repeatable fixture

Primary executed evidence, generated/retrieved 2026-10-20:
[results 034](results/034-netbsd-shared-service.json), the
[versioned service collectors](../../experiments/055-service-integration/README.md),
and [service operator contract](../../docs/service-lifecycle.md). Current-source
hashes are checked directly against installed/transferred native bytes and built
binary hashes. The current result must fail on changed service sources even when
a historical copy exists. Tests cover this rule and snapshot/manifest corruption.

Before changing root manifests/deployment scripts, actual historical bytes were
preserved in a [buildable 103-file snapshot](../evidence/README.md), anchored at
merged PR #61 (`09b506e`). Embedded SQL schemas are included from that revision.
All eight old result files remain byte-identical; old hashes were not refreshed
to new code. The snapshot workspace builds with a separate Cargo target directory.

The signed official NetBSD 11 ARM64 image/kernel and SHA-512 manifest were checked
afresh; key retrieval used HTTPS, not an independent web-of-trust verification.
The software VM has two vCPUs and 1 GiB configured RAM, about 1006 MiB physical /
972 MiB UVM-managed, no swap. This is not a physical Raspberry Pi or an all-port
NetBSD/Rust validation. Native Rust 1.96 / C compiler / Python / OpenSSH identities
are in the record; release builds use the locked reviewed dependency graph.

Each fresh boot uses the explicit experimental compact allocator and pageable-NTP
profile from [0031](0031-repeatable-headroom.md), retaining NTP, syslog, DHCP,
cron, inetd, Postfix and mDNS. Ordinary service installation does not apply it.
Profile/reset helpers require the disposable marker and private synthetic fixture.
The seed has 1000 independently salted approved members plus a sysop, 50 rooms
and 100000 1-KiB posts. Credentials/keys/transcripts/images remain private.

## Native checks and workload

The installed service passes 11 lifecycle/configuration checks: default-eight
admission/ninth denial after launch-token refill, duplicate refusal, bounded TERM
cleanup/persistent lock identity, status/stop after selector changes and worker
rollback, invalid/implicit-capacity settings, writable/hardlinked/symlink/FIFO files,
explicit 250 validation, invalid theme, session expiry, HUP/restart and side-effect-
free config validation. The full 27 shared UI/permission/descriptor/slow-reader/
frontend/backend-crash/recovery checks pass through rc.d. Busy-peer useful-chat
latency remains below two seconds; actual samples are recorded.

The maintained frontend passes GCC analyzer and PIE/RELRO/immediate-binding/no-RWE
LOAD checks, plus 20 direct argument/filesystem/FD/TTY-restoration oracles. Worker
rollback passes the existing 20 real application SSH escape/approval tests and
13 lifecycle/load/failure checks. Native root tests total 62 including four service
tests; the final service config refinement was rebuilt and its four tests rerun.
Local console/chat/snapshot/member/lock/broker PTY regressions also pass. Worker/
frontend baseline regressions precede the experimental host profile; shared UI
and all measured service runs use it. See the exact record rather than inferring
that every individual check ran under every policy.

Three fresh-boot service runs explicitly configure 250 sessions / 100-ms refill.
Every run starts/statuses/stops the installed rc.d path, rather than launching an
isolated experiment. Stages 32/100/200/250 use real distinct approved SSH members,
10-second menu/chat observations after one-second settling, approximately 110-ms
admission spacing and unchanged guard checks between batches of eight. All required
frontends and dedicated SSH processes are counted; rejected offers are not users.
At 200/250, two wrong credentials are cleaned up then two real replacement logins
complete with 198/248 members in chat. At 250, selected room browsing/post/read/
chat return runs with 249 peers in chat; two fresh owner consoles perform real
password verification and 0/19/38-MiB touched-page controls run under the full load.

The predicate remains `free + max(0, clean_file - wired) >= 128 MiB`, zero used
swap, 64 process slots and 128 file slots reserved. Profiling changes guest-wide
maxproc/maxfiles explicitly to 2048/16384 and restores the original 1044/3404;
live process limits and full numeric timing/counter matrices are retained.

Every stage, full-UI action, credential/control sample and final sampled guard passes.
Each final cohort has 250 native frontends, one shared owner, zero member workers
and 501 dedicated SSH processes; admin/probe helpers also count in whole-VM totals.

| Fresh boot | Final guard headroom (MiB) | Lowest stress headroom (MiB) | Owner final-menu RSS (MiB) | Loaded stop (s) |
| --- | ---: | ---: | ---: | ---: |
| 1 | 180.67 | 135.23 | 7.24 | 1.751 |
| 2 | 180.72 | 140.07 | 7.25 | 1.700 |
| 3 | 181.12 | 140.40 | 7.25 | 1.609 |

The owner alone is 7.24–7.25 MiB; this excludes mandatory frontend/SSH costs and
does not accept the proposed complete application budget. The lowest observed
stress margin is 135.23 MiB, only 7.23 MiB above the guard; leave wider workloads
and reserve behavior open rather than rounding that into supported capacity.


RSS sums repeat shared mappings and may exceed VM physical RAM. A small owner
RSS is not a complete application budget. Frontend/SSH process high-water counters
can include memory inherited before exec; they are not exact private frontend
peaks. No USS/PSS is inferred. Native stress sampling requests 10 ms and records
actual timestamps; observed peaks are lower bounds, not an unsampled maximum.
Successful credential pairs do not prove complete overlap or cold-cache worst
cases. These are short warm-cache runs, not a soak or sustained capacity test.

## Findings, recovery and remaining work

Native integration found that rc.subr needs command registration for `status`,
even with a custom status function. That was fixed and worker regressions repeated.
The process oracle needed full executable paths for NetBSD `pgrep -x`. The first
hostile-peer test blocked in the host's blocking PTY write; another attempt sent a
useful command behind intentionally flooded SSH input. The final harness uses
nonblocking hostile writes, keeps independent-peer useful-chat latency assertions,
discards attack transport backlog and verifies a fresh useful login. Those harness
attempts are recorded as development findings, not counted as application passes.
The application loop did not change to accommodate them.

Original allocator/NTP settings are restored/read back after reboot; fixture
reset verifies integrity, 1001 unique hashes, 50 rooms and 100000 posts. Persistent
credential inodes are never unlinked live. The private guest, keys, credentials,
raw logs, disk and forwards are removed after sanitized capture.

This advances #55 to maintained development-service integration. Its shared-owner
failure scope and trusted database access remain explicit. Stronger confinement/
frontend review, #18 maintained images/update/rollback, #20 clock-under-pressure/
broader interoperability, and #28 active mixes, worst-legal-theme/output retention,
soak, hostile/authenticated-transport admission and complete budgets remain open. Lua menus and isolated game
doors remain the next extension priority after the useful core. No HTTP portal,
federation, files/mail, blockchain service or supported 250-user release follows.
