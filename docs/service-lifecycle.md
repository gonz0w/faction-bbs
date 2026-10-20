# NetBSD gateway service lifecycle

Updated 2026-10-20. Development service path for the existing restricted SSH
gateway, tested on a disposable NetBSD 11 ARM64 VM. No supported appliance or
public installation is implied. Complete [restricted SSH setup](restricted-ssh.md)
and local owner bootstrap first; keep administrative SSH separate.

## Install and enable deliberately

On the target system as root, after building/testing and installing the four
binaries and dedicated SSH configuration, install the two scripts. For a fresh
installation, create the private operational log; preserve existing logs on updates.

```sh
install -o root -g wheel -m 555 deploy/netbsd/faction_broker deploy/netbsd/faction_sshd /etc/rc.d/
touch /var/log/faction-broker.log
chown root:wheel /var/log/faction-broker.log
chmod 600 /var/log/faction-broker.log
```

Set `faction_broker=YES` and `faction_sshd=YES` in root-owned `/etc/rc.conf` or the
corresponding `/etc/rc.conf.d/faction_broker` and `faction_sshd` files. Defaults are
NO. Scripts assume the named accounts/paths in the SSH guide. They never initialize
or restore a database, admit a key, edit firewall rules or broaden the listener.
Boot ordering places broker before the dedicated SSH instance; shutdown reverses
that order. The broker script recreates its volatile `/var/run/faction` directory
with owner/group `factiond:factionipc` and mode `0710`.

```sh
/etc/rc.d/faction_broker start
/etc/rc.d/faction_broker status
/etc/rc.d/faction_sshd start
/etc/rc.d/faction_sshd status
# Stop admission first, then active application workers:
/etc/rc.d/faction_sshd stop
/etc/rc.d/faction_broker stop
```

For a deliberate broker restart, `/etc/rc.d/faction_broker restart` ends current
sessions and starts the same fixed configuration. Apply pack changes or binary
updates with a restart. HUP also stops the broker; it does not reload. Root-owned
`faction_broker_flags` may specify `--theme /ABSOLUTE/PACK --season /ABSOLUTE/PACK`
and/or `--session-seconds 900`. It is privileged shell configuration, never member
input; quote paths as needed. Session duration is 1–1800 seconds (default 1800).
The existing 120-second individual prompt budgets are separate.

## Drain, failure and recovery

TERM/INT/HUP close admission, drain for up to five seconds, then kill/reap remaining
member workers and join supervisor threads. Pending receives have a two-second
budget once they hold the spawn lock; waiting threads check shutdown. The service
stop command checks for exit for ten seconds and reports failure instead of
silently sending KILL. Kernel I/O/scheduler stalls can exceed ordinary measured
timings; inspect a failed stop before forced recovery.

The `terminal.pid` file is also the single-instance kernel lock. It remains on
disk, empty after an orderly stop; do **not** remove or replace it while any broker
may be running. Duplicate starts are refused. After a crash, a new broker acquires
the released lock and recovers only an owned disconnected socket. It refuses
symlinks, ordinary files and live sockets. Parent paths and same-UID processes are
trusted; this is not a defense against a compromised board owner/root.

On broker death, real member workers detect their kernel-parent change every
200 ms and exit, including a blocked UI. The terminal frontend restores saved
terminal settings and exits when its control socket closes. Forced termination
may leave chat presence until the existing 60-second lease expiry/reclamation.
No automatic service restart loop is installed; inspect private operational logs,
restart deliberately, and verify an authenticated member can enter again.

Logs contain service readiness, generic failures and aggregate rejected-admission
counts, not handles, credentials or message text. Configure local log rotation for
`/var/log/faction-broker.log`; no remote telemetry is installed. An operator's own
additional SSH logging configuration remains separate.

Keep eight workers until broader measurements justify a change. OpenSSH processes
and pre-admission connections cost additional memory/CPU; its handshake settings
are not a complete authenticated connection cap. See [measured scope and remaining
gates](../dev-docs/research/0026-service-lifecycle-load.md). File sharing, mail, Lua
doors, federation, HTTP and physical-device support do not follow from this service.

The two per-board credential lock files are also persistent coordination inodes.
Stop every owner/member process before replacing/removing them. Ordinary service
restart and owner snapshot export do not remove them. Credential saturation fails
that login/registration attempt explicitly; see [the SSH guide](restricted-ssh.md).
Experimental scaling flags do not change normal service defaults or sysctl limits.

The [shared-owner/credential comparison](../dev-docs/research/0029-shared-board-and-credential-peaks.md)
is an opt-in standalone experiment, not a replacement for these rc.d scripts.
Normal service defaults and the locked pidfile/credential inode rules remain.
Its direct terminal-owner lifecycle and incomplete UI need explicit adoption
review; a larger experimental admission count does not change service capacity.

The [repeatable native headroom experiment](../dev-docs/research/0031-repeatable-headroom.md)
passes three short 250-member VM workloads, including high-load member/owner
credentials and known extra-memory controls under the unchanged guard. Its
pageable time-daemon policy retains NTP and all other base services; original
settings are restored after testing. Normal setup/rc.d defaults remain eight
workers. The shared UI/minimal frontend, scoped allocator/host policy, clock
precision under pressure and broader active-mix/soak/admission bounds need #55/
#20/#28 adoption work. Keep research profiles confined to marked disposable VMs.
