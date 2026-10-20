# 0021: Filesystem, network and exec boundaries on NetBSD

Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55), related #19/#20/#23/#28.
Status: implementation/native evidence complete; final workflow checks and PR.
Three numeric-lookup boots pass; minimum stress 132.71 MiB. Two rejected
named-lookup controls remain preserved. #63 merged at `473feb4`.

## Goal and constraints

Add an explicit experimental isolation profile around the maintained service and
native frontend while retaining default eight, current credentials/authority/UI,
fixed helpers and the unchanged 250-member research guard. Base OpenSSH and all
frontends/base services remain counted. No extra resident supervisor, registry
dependency, external kernel module, schema or automatic allocator/NTP/sysctl changes.

Keep the normal service and rollback binaries intact. Profile binaries live under
`/usr/local/libexec/faction-isolated/` with unchanged basenames for attribution.
Use separate root-managed rc.d selection and SSH ForceCommand. Opt-in profile
installation/network policy must be explicit; no existing firewall is overwritten
by ordinary service start. A new privileged startup boundary needs negative tests
and must stay experimental pending broader review.

## Plan

- [x] Verify required CI/reviews, merge #63, read current constraints and #55.
- [x] Research base chroot/noexec/NPF and current external isolation alternatives.
- [x] Prove release-local filesystem/exec, GID/UID packet and privilege-drop behavior.
- [x] Build narrow pre-main C initialization and explicit installation/lifecycle.
- [x] Exercise actual sealed binaries: 31 frontend, 29 UI, 16 lifecycle checks;
  boundary controls, both allocator readbacks, native analyzer/ELF and bidirectional
  credential/pid lock alias checks pass. Final TCP accept-counter probe also passes.
- [x] Repeat three fresh-boot 250-member full-counter/ceiling/guard workloads.
- [x] Restore/check host settings and integrity; remove private VM material.
- [x] Prepare evidence/research/ADR/current docs and focused PR; require CI before merge.

## Candidate and findings

One shared minimal noexec/nodev/nosuid nullfs root carries the exact run and
selected state directories, plus owner-only read-only config/theme copies. The
frontend cannot read owner config or DB. Separate nullfs aliases failed Unix
socket connection. Native outside-root fexecve entry failed; the selected small
pre-main C initializer seals the root and drops real/effective/saved privileged
IDs before Rust main or application parsing. Only the profile frontend is setuid;
normal service/frontend binaries remain independent rollback. Loader/libc startup
is trusted and arbitrary-code/process/same-UID isolation remains unclaimed.

UID-only NPF failed a real terminal TCP gate. Combined dedicated-group/two-user
stateless NPF passes native IPv4/IPv6 loopback TCP/UDP controls; raw/datagram ICMP
sockets fail. Setup requires the exact active canonical policy and no states,
never applies/enables/replaces the firewall, and rejects non-dedicated groups.
Preflight uses an empty isolated Python environment and refuses optimization.
User mounts are disabled; native securelevel refused enabling them at runtime.

Two initial fresh-boot functional 250-member completions dipped to 126.36 and
123.47 MiB in the 38-MiB loaded control, below the unchanged 128-MiB guard. They
are rejected evidence, with five exact source overrides preserved. The final
initializer replaces one fixed named sysctl lookup with the equivalent numeric
MIB, avoiding libc name-resolution allocation. At matched 32-user chat, frontend
VSZ falls from about 11.37 to 7.34 MiB; never call this a physical RAM saving.
Final lifecycle/UI/security checks and three new clean-boot load gates pass, using the explicit compact host+sealed allocator, pageable NTP, all
base services and experimental kernel limits. Service setup never selects those
memory settings. Broader confinement/admission/capacity gates remain open.

Original 035 application/probe inputs currently remain unchanged: do not edit their
hashes or claim a new mode was measured by them. New profile build/deployment/
probe inputs and actual binary identities are pinned in complete result 036.
Native VM only; primary NetBSD 11 ARM64, signed official image, 2 vCPUs/1 GiB.
Continue software work without physical SBC hardware. Lua/doors require separate
UID/capabilities and launch, never relaxed member limits.
