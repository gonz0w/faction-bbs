# Experimental NetBSD filesystem and IP policy

This opt-in development profile adds a minimal chroot, noexec mounts and an
explicit host packet policy around the maintained shared UI. It is validated
only on the recorded NetBSD 11 ARM64 software VM. It introduces a **setuid root
frontend startup boundary** and remains experimental pending independent review.
Ordinary service installation and worker rollback keep their existing binaries.
There is no supported production sandbox or capacity claim.

## What runs with privilege

The trusted ELF loader and linked runtime libraries map the program and dependencies first. A small
[C initializer](../native/faction-isolation/startup.c) then runs before application
main or Rust configuration/theme/database parsing. It accepts only a bounded,
root-owned identity policy; checks disabled user mounts, root-owned ancestors,
stdio and caller identity; closes inherited descriptors; and sanitizes the
environment. The service requires a private root-owned regular log. The frontend
requires the terminal account, its dedicated primary group, one owned stdio TTY,
SSH transport context and a bounded TERM. Exec/subsystem requests are rejected.

It enters `/var/chroot/faction`, changes cwd to `/`, checks mount flags, drops
supplementary groups and real/effective/saved privileged IDs, and verifies that
root IDs cannot be regained. Only then does application main execute. The
[existing kernel envelope and peer checks](process-confinement.md) still apply.
No extra resident supervisor, per-user Rust worker or thread is added. The
recorded service links base libpthread, libgcc_s, libc and libm; the frontend
links libc. These libraries and the ELF loader execute outside the new boundary
and are part of its trust, with hashes retained in the evidence.

The profile service is root-owned mode 0700 at
`/usr/local/libexec/faction-isolated/faction-service`; the profile frontend is
root-owned **4755** at `/usr/local/libexec/faction-isolated/faction-tty`.
Never setuid the ordinary frontend or any Rust executable. Root loader/runtime libraries and
this initializer are part of the trusted computing base. This startup design
does not retroactively protect code executed before the initializer.

## Filesystem view

Both processes use one common nullfs view. Separate nullfs aliases could not
exchange the Unix socket in the native probe; sharing the mount preserves the
existing descriptor handoff. The backing tree is hidden beneath root-only
`/var/lib/faction-isolation`. The common root is read-only/noexec/nosuid/nodev;
only the exact run directory and selected board directory are writable nullfs
mounts, also noexec/nosuid/nodev. Config/theme copies are owner-only read-only
files/directories. The frontend cannot read them or the owner-only board.
No shell, runtime libraries, devices, procfs or host account database are placed
inside. Libraries were mapped before entry. The existing TTY/log descriptors
remain explicit authority; the owner still has the board's SQLite and locks.

Noexec blocks pathname and descriptor exec of the tested valid ELF. It does not
prevent already-running code, memory corruption, executable memory or interpreter
logic embedded in a process. Chroot does not supply process, network or same-UID
namespaces. The profile does not make compromised owner code safe for board data.
Lua/doors need a separate launcher/UID and capabilities; never relax member caps.

## Packet policy and operator control

The explicit experimental NPF candidate blocks the dedicated IPC **group and
both application users**, then passes other traffic. BPF JIT is disabled for
this candidate. The group must contain only the two application accounts. A
UID-only candidate failed the native terminal TCP gate and was rejected; retain
the combined GID/UID policy and fixed-group checks.

This is a narrow host policy, **not a complete host firewall or network
namespace**. Native probes cover IPv4/IPv6 loopback TCP connects, outbound and
unsolicited inbound UDP, plus denied raw/datagram ICMP sockets. Positive root
controls establish working listeners and packet delivery; denied UDP payloads
must never arrive. Other address families/protocols, same-UID peers, process
visibility/tracing, remote-interface coverage and hostile code remain gates.
The SSH transport runs outside the chroot and is retained in memory accounting.

`prepare_isolation.py --configure` writes a candidate but never loads/enables NPF.
An operator must review it and explicitly select the host policy and boot order.
**Do not replace an existing firewall with this development candidate.** Custom
firewall composition has not been validated. Start preflight requires the exact
canonical candidate active, no saved flows, unchanged IDs/config, safe sources
and exact mount sources/flags. It fails rather than installing a different rule
set. Stopping/changing the firewall after startup is a privileged host change;
there is no continuous application firewall monitor.

## Disposable research setup

Use a marked, private, disposable NetBSD 11 ARM64 VM following the
[existing development prerequisites](service-lifecycle.md), with root-owned theme
source packs and a dedicated `factiond`/`factionterm` IPC group. Python 3.13 is
needed for root setup/start preflight only, not as a resident server. The existing
board and run directories retain their private owner permissions and persistent
credential/pid locks. Export a stopped board before any separate deployment test.

1. Build with `python3 tools/build_isolation.py --output-dir OUT --target-dir TARGET`
   on NetBSD using the existing frozen vendor/toolchain setup. This builds only;
   it never installs privileges, policy or mounts. Native object content hashes
   enter Cargo's link argument so changing the initializer forces a relink.
2. Root-review/install the profile binaries with the modes above, the setup tool
   mode 0555 under the same directory, and `deploy/netbsd/faction_isolated` mode
   0555 as `/etc/rc.d/faction_isolated`. Keep the normal binaries untouched.
3. Stop the ordinary backend and explicitly select only one backend. Configure
   the existing strict `/etc/faction/service.toml`; run the installed setup tool
   with `--configure` while stopped. It creates bounded protected identity policy,
   config/theme snapshots and the minimal mount layout. A changed config/theme
   needs an explicit stopped reconfiguration; changing board path needs teardown.
4. Review the generated `/etc/faction/npf-isolation.conf` and explicitly arrange
   active NPF before service startup. In the isolated test VM only, the experiment
   selected `npf=YES` and `npf_rules=/etc/faction/npf-isolation.conf` in its NPF rc
   configuration. This is not an instruction to replace a real operator policy.
5. Explicitly select `faction_isolated=YES` and the profile frontend in the
   dedicated SSH ForceCommand, keeping socket path/owner UID arguments and all
   existing SSH restrictions. Start/status/stop use `faction_isolated`; base SSH
   still needs its backend started first. Its normal rc dependency names the
   ordinary backend, so this experimental selection needs deliberate boot order.

The rc script runs Python preflight with an empty environment and isolated mode;
the tool rejects optimized Python that would disable validation assertions.
Start preflight may recreate the selected nullfs mounts after reboot, only after
validating policy/config. It never tunes allocator, timekeeping, sysctls or
firewall activation. The 250-member settings remain explicitly experimental;
ordinary defaults remain eight/five seconds. Research memory profiles are
separate operator actions, including a matching allocator setting in the common
root; native readbacks verify it in both sealed roles.

## Stop, recovery and removal

Stop dedicated SSH and the isolated backend before changing selection. Preserve
pidfile/credential lock inodes; never unlink a live lock. Status/stop match the
profile executable. Stop remains bounded and restart uses the same database.
To return to the ordinary path, select its original frontend/backend and use the
existing worker/shared rollback procedure. No schema migration is introduced.

After all users of the view stop, unmount board, run and finally root in that
order; never force-unmount a live board. Busy mounts require investigation or a
controlled reboot with the profile disabled. Remove the profile's setuid bit
before removing its private files. Restore only settings explicitly selected for
the experiment, including NPF boot selection, allocator and NTP. Do not disable
an operator firewall or remove unrelated host configuration. See
[research and complete evidence](../dev-docs/research/0035-os-isolation.md).
