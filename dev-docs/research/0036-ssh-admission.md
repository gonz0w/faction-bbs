# 0036: SSH admission, abuse costs and shared-service headroom

Date: 2026-10-20. Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55),
related #19/#20/#28. Experimental implementation; no supported release.

## Question and comparison

Can we bound authenticated member SSH connections, including connections that
never open a BBS terminal, while keeping
the existing 250-member resource guard? [ADR 0022](../decisions/0022-ssh-admission.md)
records the experiment choice and alternatives. The ordinary listener, core,
shared UI, approval/grants, credentials, schema and isolation profile are preserved.

Primary sources retrieved 2026-10-20:

- [NetBSD 11 sshd(8)](https://man.netbsd.org/NetBSD-11.0/sshd.8), inetd mode `-i`.
- [NetBSD 11 sshd_config(5)](https://man.netbsd.org/NetBSD-11.0/sshd_config.5),
  MaxStartups, MaxSessions, LoginGraceTime, UnusedConnectionTimeout and forwarding.
- [NetBSD 11 inetd manual source](https://github.com/NetBSD/src/blob/36cf5108df513943b2914535a5e7c2f86fa35fd7/usr.sbin/inetd/inetd.8),
  `accept_max` versus `service_max`/`ip_max`.
- [inetd implementation](https://github.com/NetBSD/src/blob/36cf5108df513943b2914535a5e7c2f86fa35fd7/usr.sbin/inetd/inetd.c)
  and [rate limiter](https://github.com/NetBSD/src/blob/36cf5108df513943b2914535a5e7c2f86fa35fd7/usr.sbin/inetd/ratelimit.c),
  fixed accept-child accounting versus ten-minute close after aggregate rate exhaustion.
- [Base sshd config sender](https://github.com/NetBSD/src/blob/36cf5108df513943b2914535a5e7c2f86fa35fd7/crypto/external/bsd/openssh/dist/sshd.c),
  `send_rexec_state()` in inetd mode; and
  [monitor lifecycle](https://github.com/NetBSD/src/blob/36cf5108df513943b2914535a5e7c2f86fa35fd7/crypto/external/bsd/openssh/dist/monitor.c).
- [NetBSD process exit](https://github.com/NetBSD/src/blob/36cf5108df513943b2914535a5e7c2f86fa35fd7/sys/kern/kern_exit.c),
  user vmspace release and zombie wait/reparent accounting.

MaxStartups is a preauthentication bound; MaxSessions is per transport. Neither
stops an invitation holder opening many `ssh -N` connections. The base inetd cap
is credible reuse, but the documented ten-minute rate blackout differs from the
desired short refill. No per-IP table/ban is selected: private gateways/NAT can
represent many legitimate members. No stateful firewall replacement, modified
OpenSSH, internal sshd-session launch protocol or replacement SSH implementation
is introduced.

## Candidate selection and actual boundary

The first fixed-table native listener capped TCP launches before base `sshd -i`.
All three native runs completed functional 250-member work, but stress minima
were **129.35 / 125.50 / 126.04 MiB**. Two fail the unchanged 128-MiB predicate.
Reject this candidate for the target rather than relax SSH, credentials, the guard
or account away its extra processes. Code, source versions and complete failures
remain preserved; it is not the selected deployment.

The selected experiment reuses **AuthorizedKeysCommand** under dedicated
`factionauth`, with a fixed process-birth registry and global reservation/token
limits. The helper copies bounded root-managed key lines; OpenSSH retains parsing,
signature verification and options. No fallback AuthorizedKeysFile/certificate
path is permitted by the actual rc preflight. The helper adds no resident process,
proxy, SSH parser or crypto change. See the [operator contract](../../docs/experimental-admission.md).

Defaults remain eight reservations/burst eight/5,000-ms refill; the explicit
250-member experiment uses a 250-ms key-admission refill with unchanged 100-ms
application refill. Repeated lookups by one monitor reuse a slot; real-root UID
and kernel birth remain stable while OpenSSH temporarily changes effective UID
for command output. Listener restart cannot erase old live monitors. Checksummed
synchronized state fails closed; missing state requires quiescent initialization.
Native NetBSD boot cleanup removed the runtime directory, motivating that explicit
initializer rather than an unconditional recreate/reset.

This cap occurs **at key lookup**, with base MaxStartups/penalties/timeouts retained.
It does not bound all TCP transports at eight or enforce a global pre-fork launch
rate. Eight extra preauthentication connections, shared-NAT penalties, CPU/fork
flooding and repeated key-query amplification remain separate adoption gates.
Default process visibility is required without automatic lowering of host privacy.

## Native instrumentation finding

The first load attempt stopped at a live-limit ledger error with 32 approved
members. Base OpenSSH's inetd-mode config sender had exited but remained a zombie
under its connection monitor. NetBSD rejects a live-rlimit query for that process.
Source inspection and a one-connection native positive control confirmed the
process shape; this was an instrumentation failure, not a failed memory predicate.

The corrected rejected-candidate collector records these zombies explicitly and
queries rlimits only for live processes. Whole-host VM/kernel/process counters still
include them. Its transport metric group contains the launcher, both live SSH
processes per connection **and each completed config-sender entry**: 751 entries
at 250, with 250 explicit zombie entries in each live-limit ledger. Kernel-reported
RSS/high-water fields may retain exit accounting for zombies; summed transport
RSS includes those reported values and repeated shared mappings. It is not a
current physical or unique memory budget. The UVM-based guard remains unchanged.
The diagnostic result and exact differing driver/collector bytes are preserved.
The selected base-daemon/key-hook path retains 501 SSH entries at 250, zero
config-sender zombies and every transient key helper in descendant accounting.

## Validation and resource accounting

Native NetBSD 11 ARM64, official signed/hash-verified image, QEMU/HVF 2 vCPUs,
configured 1 GiB, no swap; software only. The same explicit global/sealed compact
allocator and pageable-NTP study profile is used, with every base service kept.
No service start applies those settings. Dataset: 1,000 distinct approved member
hashes plus sysop, 50 rooms and 100,000 approximately 1-KiB seed posts.

Guard: **free + max(0, clean_file - wired) >= 128 MiB**, zero swap,
processes + 64 < maxproc, files + 128 < maxfiles. All application ceilings and
unchanged two-permit/two-kernel-slot Argon2 settings remain. The ledger includes the base listener and every SSH descendant, including
transient helper work; no mandatory transport process is excluded.

Selected final native results:

| Fresh boot | Minimum stress headroom | Final pre-stop headroom | Loaded backend stop |
| --- | ---: | ---: | ---: |
| 1 | 130.01 MiB | 175.23 MiB | 1.487 s |
| 2 | 130.09 MiB | 175.22 MiB | 1.565 s |
| 3 | 129.34 MiB | 174.66 MiB | 1.317 s |

Every sampled guard passes, zero swap; 501 base SSH entries, 250 frontends and one
owner are counted. Both extra key lookups at a full 250-cohort are denied without
creating another reservation; their SSH preauthentication work still occurs and
is included in whole-host accounting. The field `launches` counts reservations,
not TCP accepts or forks. No rejected offer counts as a connected member.

14 native key/state/missing-state/rate/restart/config checks, six real SSH
channel/restart checks, 29 full UI, 31 sealed frontend, 31 ordinary frontend,
20 ordinary SSH, 13 worker lifecycle, six native PTY flows and 63 native Rust tests
pass. Silent-preauth controls admit eight, drop excess offers and return to the
single listener after grace deadlines; base source-penalty history is reset only
in the disposable fixture before subsequent tests. Native analyzer and ELF
PIE/RELRO/immediate-binding/no-RWE checks pass.

[Complete result 037](results/037-netbsd-admission.json) retains every selected
and rejected stream/ledger, actual source/binary identities, primary-source pins,
diagnostic bytes, settings restoration and task-private cleanup. Rejected-candidate
collector binary identities were not captured before replacement; its exact
executed sources/compiler commands and actual service/listener identities remain.
Do not present a reconstruction as an executed binary hash. Current selected
collector identities are captured directly.

Original allocator/NTP/sysctl/firewall/profile privileges are restored/read back
following reboot; stopped seed integrity/counts are verified. Private VM, generated
keys/passwords, raw logs/disks and loopback forwards are removed. Existing nine
historical records/two complete snapshots and records 035/036 remain unchanged.

## Scope and next gates

Independent key-authority/helper/loader review, IPv6 with a genuinely shared cap, sustained
connection/CPU/SYN/backlog pressure, reserved admission/fairness, robust state
recovery/service management, image/timekeeping policy and broader #28 active/soak
budgets remain. A capped hostile peer can still deny availability. No claim of a
complete sandbox, supported 250-member capacity or physical SBC readiness follows.
No Lua/door process inherits this root launcher or gains an exec/network exception.
