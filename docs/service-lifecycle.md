# NetBSD shared terminal service

Updated 2026-10-20. The maintained `faction-service` Rust binary runs the full
member UI in one board-owner process. The NetBSD rc.d path selects it by default;
services remain disabled until deliberately enabled. This is a development gateway,
with eight sessions by default. It is not a supported appliance release.

## Install and configure

Complete [restricted SSH setup](restricted-ssh.md), private owner bootstrap and
native builds first. Keep administrative SSH separate. On the target NetBSD host:

```sh
install -d -o root -g wheel -m 755 /etc/faction
faction_terminal_uid=$(id -u factionterm)
sed "s/@FACTION_TERMINAL_UID@/$faction_terminal_uid/" deploy/netbsd/service.toml.in > /etc/faction/service.toml
chown root:wheel /etc/faction/service.toml
chmod 644 /etc/faction/service.toml
su -m factiond -c '/usr/local/libexec/faction-service --check-config /etc/faction/service.toml'
install -o root -g wheel -m 555 deploy/netbsd/faction_broker deploy/netbsd/faction_sshd /etc/rc.d/
touch /var/log/faction-broker.log
chown root:wheel /var/log/faction-broker.log
chmod 600 /var/log/faction-broker.log
```

Preserve existing configuration/logs during updates. Set `faction_broker=YES` and
`faction_sshd=YES` in the corresponding root-owned `/etc/rc.conf.d/` files, or in
`/etc/rc.conf`. Both default to NO. `faction_broker_backend=shared` is the default
selector; its configuration path is fixed at `/etc/faction/service.toml`.

```sh
/etc/rc.d/faction_broker start
/etc/rc.d/faction_broker status
/etc/rc.d/faction_sshd start
# Stop admission before ending current sessions:
/etc/rc.d/faction_sshd stop
/etc/rc.d/faction_broker stop
```

Scripts recreate the protected volatile socket directory. They do not initialize
or restore state, admit keys, expose a public interface, adjust kernel limits, or
change allocator/timekeeping settings. The database must already exist. Owner
bootstrap/recovery/export/restore remain separate `faction` commands.

## Configuration contract

[The template](../deploy/netbsd/service.toml.in) lists all settings. Version 1 and
the nonzero terminal UID are required. The service rejects unknown keys, wrong
types, unsafe files, relative/traversing/control-containing paths and invalid
themes. Configuration is at most 8 KiB, regular, singly linked, root/owner-owned,
and not group/world writable; a symlink or FIFO fails. Parent directories and the
board owner/root remain trusted. Keep `/etc/faction` root-owned and nonwritable by
members. Mode 0644 exposes only nonsecret settings; never put passwords in this file.

| Setting | Default / allowed value |
| --- | --- |
| `socket` | `/var/run/faction/terminal.sock`; version 1 requires this fixed path. |
| `state_dir` | `/var/db/faction`; existing private board. |
| `max_sessions` | 8; 1–250. |
| `launch_ms` | 5000; 100–60000, burst eight. |
| `session_seconds` | 1800; 1–1800. Individual prompts also have a 120-second limit. |
| `ansi` | true; terminal capability/size still controls plain fallback. |
| `theme`, `season` | Optional absolute paths; validate once at startup, base then manual override. |
| `experimental_capacity` | false; must be explicitly true for more than eight sessions or admission faster than five seconds. |

`--check-config FILE` validates configuration/theme without opening the board or
listener. Run it as `factiond` to check real access. `--config FILE` runs the
service in the foreground as the non-root board owner. The service accepts no
member-selected paths or owner-console commands. Version 1 fixes the socket path so rc.d status/stop and the SSH template
always refer to the same lock and endpoint. Custom endpoints/multiple boards
require a separately designed instance configuration.

Restart after configuration, pack or binary changes. Theme/season data is shared
across sessions, with no per-menu disk reads. ANSI menus support arrows/Enter and
the displayed hotkeys; plain/resize recovery, rooms/replies/retry IDs, local chat
and role-checked sysop controls use the existing core. Doors remain disabled.

## Lifecycle and rollback

`restart` ends existing sessions and starts the selected backend. TERM/INT/HUP
stop admission, finish the bounded credential jobs, close leases/TTYs and exit;
HUP does not reload. Nonempty output batches have a five-second deadline. The rc.d
stop command waits up to ten seconds and reports a failed stop without automatic
KILL. A stalled kernel/scheduler can exceed ordinary measured timings.

`terminal.pid` is a persistent kernel-lock inode. Never remove or replace it
while either backend may be running. Two credential lock inodes are persistent
too; never unlink them while any board process is active. A new process recovers
only an owned disconnected socket after obtaining the lock. Duplicate starts and
unsafe/live socket replacement fail. Backend death closes frontend control
connections; the frontend restores terminal state. Abrupt termination can leave
presence until the existing 60-second lease expiry. No automatic restart loop is
installed. Logs contain aggregate setup/readiness/errors, not passwords/content.

To roll back, stop admission and the backend, set `faction_broker_backend=workers`,
then start the backend and dedicated SSH daemon. Keep installed `faction-broker`
and `faction-member`; they use the identical database schema. Worker configuration
uses root-owned `faction_broker_flags` for existing `--theme`, `--season` and
`--session-seconds` options. Clear those flags when selecting shared mode: shared
mode rejects them rather than silently ignoring limits. Status/stop identify the
actual known backend even if the selector was edited while it was running.

The maintained native `faction-tty` frontend works with both backends. The Rust
`faction-ssh` entry point remains available for developer-host/worker comparisons;
switching frontends requires editing/testing the root-owned ForceCommand and
restarting the dedicated SSH daemon. Stop/reap sessions before replacing binaries.

## Resource scope

An explicit 250-session setting is research configuration, not a capacity promise.
The service does not enforce the experimental whole-host memory guard. Mandatory
SSH processes/frontends and pre-admission transports still cost memory; eight
application slots do not cap authenticated idle SSH transports. Complete budgets,
active mixes, soak, hostile admission and stronger process confinement remain open.

The existing 250-member VM results require an explicit experimental compact
allocator and pageable-NTP image profile. Do not apply that profile to an existing
host implicitly. Maintained image/update/rollback and timekeeping under pressure
remain #20; wider workloads and admission remain #28. See [integration research](../dev-docs/research/0033-shared-service-integration.md)
for actual service-path measurements. Physical SBC claims await later hardware.
