# Local sysop console

Updated 2026-10-24. The read-only local operator console is available as
`faction-sysop`. It is separate from member SSH and from the in-BBS sysop menu.
Log into the operating system normally, then run the command as the board's Unix
owner or an already-authorized root operator. A BBS sysop role grants no OS access.

## Enable aggregate status

The normal installer/default stays disabled. After building the locked workspace,
install `target/release/faction-sysop` as a normal executable in your chosen local
operator path (for example `/usr/local/bin/faction-sysop`). Never setuid/setgid it
or use it as a member SSH forced command. This preview refuses a changed effective
UID/GID and never changes getty, account shells, host settings or the firewall.

Using the existing [stopped service workflow](service-lifecycle.md), set this
explicit option in the root-managed shared-service configuration:

```toml
operator_status = true
```

Start the shared backend normally. It publishes `operator.status` in its existing
private `state_dir`, atomically replacing a mode-0600 regular file every five
seconds. No new listener, owner helper, database schema or member privilege is
added. Worker rollback does not produce these snapshots. The normal eight-session,
two-credential-permit and output/drain limits remain unchanged.

Then, from your OS login shell:

```sh
faction-sysop --state-dir /var/db/faction --ansi
faction-sysop --state-dir /var/db/faction --theme /var/db/faction/themes/base --ansi
faction-sysop --state-dir /var/db/faction --plain --once
faction-sysop --state-dir /var/db/faction --view host --plain --once
```

Use your actual configured state/theme directories. The template defaults are
examples; the console does not discover or alter the service configuration. Theme
ownership/validation follows the existing [theme contract](theme-packs.md). Select
the same pack explicitly for matching board identity and semantic colors. The
console uses text panels rather than replaying login art or running theme code.

## Panels and controls

| Panel | Contents |
| --- | --- |
| B: Board | Connected/capacity, credential permits, queued output, member/pending totals, room/post/channel/retained-chat totals, published/pending files, active transfers, completed games, active doors, service PID/uptime and connected history. |
| H: Host | CPU count/busy percentage, 1/5/15-minute load, RAM used/free/physical capacity, board-filesystem available/total space, non-loopback network RX/TX rates and bounded CPU/RAM/network traces. Federation adapters remain unconfigured; recovery checks are in D:Data. |
| D: Data | Historical selected-bundle check, latest attempt and last successful check age/totals; see [backup checks](backup-checks.md). |
| I: Info | Local authority, controls and read-only scope. |

Arrows/Tab and B/H/D/I switch panels. P pauses/resumes local sampling. R redraws
without bypassing the sampling limit. Q or Ctrl-C/D exits back to the OS shell;
terminal settings are restored and ANSI exits place the shell below the dashboard. Unknown terminal sequences use the existing
bounded decoder; after Escape, Enter recovers input. No shell-command field or
host/member actions exist.

Default refresh is five seconds; `--refresh 2..60` selects a bounded interval.
Each connected/CPU/RAM/network history retains at most 60 samples, showing the most recent tail that fits the width. Its classic ASCII trace uses
`.` for zero and `:`/`*`/`#`/`@` for increasing usage bands. Add `--unicode` alongside `--ansi` for optional UTF-8 block bars/sparklines when the operator knows the font supports them; plain/piped output stays ASCII. CPU/RAM traces use a fixed 0–100% scale; network RX+TX is relative to the observed window peak. Missing observations are gaps. Pausing stops local
snapshot reads/host queries; it does not disable an explicitly enabled service
publisher. ANSI mode changes only modified rows and handles resize; a right margin
prevents wrapping. At 40x18 the panels still fit. Plain mode emits no ANSI controls,
including on exit. Piped output is a single plain snapshot unless a TTY is present;
`--once` also gives a single snapshot on a terminal. The displayed dimensions are
actual TTY geometry (or the declared 80x24 formatting width for a pipe).

## Reading health honestly

FRESH means a running-marked snapshot is at most 15 seconds old. The separate
**Owner:** line observes the optional publisher's kernel lease; neither indicator
alone proves service responsiveness or end-to-end delivery. STALE, STOPPED, UNAVAILABLE and CLOCK UNKNOWN are
distinct; unavailable reads keep only explicitly labeled last-known values. A
future timestamp never becomes a fresh sample. Graceful shutdown writes a final
stopped snapshot; a crash leaves the old sample to age. Samples use the host clock;
this is not a clock-synchronization service.

When publication is enabled, the owner holds an exclusive lock on the persistent,
mode-0600 `operator.lease` inode for the publisher's lifetime. Each start writes a
new opaque instance ID and publishes it with the PID in a version-2 snapshot. The
console reads at most 64 bytes and tries a nonblocking shared lock through a
separate read-only descriptor. It creates no lease file, launches no helper,
sends no signal and opens no service connection.

| Owner indicator | Observation |
| --- | --- |
| LEASE HELD / current instance | The kernel lock is held and its PID/instance match the sample. |
| NO LEASE / publisher inactive | No exclusive publisher lock was observed. This does not prove that every BBS process is stopped. |
| DIFFERENT / last-known sample | The held lease belongs to another PID or startup instance, even if the PID has been reused. |
| UNVERIFIED / legacy sample | A readable v1 snapshot has no instance binding. |
| UNAVAILABLE | The current sample or safe lease observation could not be read. |

A held lease can coexist with stale data or a hung publisher. Keep sample age and
owner lifetime separate. Failed refreshes clear the previous owner observation;
last-known counts remain labeled. Connected history accepts only fresh snapshots
with a matching held lease. A paused dashboard freezes all its local observations.
Federation is explicitly **not configured**; no invented peer collector or health
promise is added. Output is labeled idle/buffered occupancy, not delivery health.

Queued bytes measure the retained encoded output buffer, including an already
written prefix until that buffer drains. This is buffer occupancy, not a count of
unsent or end-to-end delivered network bytes.

Counts are aggregate-only. The per-start instance ID is telemetry metadata, not
an authentication token. No handle, message body, password/hash, file name, address
or device key appears in the status protocol. The dashboard never opens
SQLite or takes the BBS writer lock. Owner SQL uses a zero busy wait plus a
50,000-instruction/five-millisecond progress budget and restores the connection's
previous busy timeout/hooks; interrupted aggregates are unavailable, not zero.
This is a query progress budget, not a hard real-time guarantee for storage I/O.

The console reads only an owner-private directory and a bounded, single-link,
owner-owned regular snapshot, rejecting symlinks/unsafe modes/oversized or unknown
frames. Unix permissions provide local operator authorization; this does not give
an untrusted process with the same Unix owner a new sandbox. Stronger containment
remains in the later hardening work.

Host sampling uses filesystem `statvfs` and a small native read-only OS bridge,
with no command launched on refresh. NetBSD reads `hw.ncpu`, `hw.physmem64`,
`kern.cp_time`, `vm.uvmexp2`, load averages and routing-interface counters; macOS
uses its corresponding sysctl/Mach APIs. The [NetBSD sysctl reference](https://man.netbsd.org/sysctl.7)
and installed NetBSD 11 headers define those counters (checked 2026-10-22).
Unsupported collectors remain unavailable. Native output is 64 integer slots;
network enumeration uses one fixed 64-KiB buffer and at most 16 non-loopback
interfaces. Oversized/changing tables fail that metric independently. No names,
addresses, routing data, user commands or kernel pointers leave the bridge.

CPU busy percentage uses non-idle tick deltas across all CPUs. RAM **used** is
physical capacity minus free pages, including kernel reservations/cache; it is not
an application working set or an estimate of reclaimable memory. Network rates
sum non-loopback interface-byte deltas; tunnels/bridges can count traffic more than
once. These are host interface observations, not unique BBS payload throughput.
CPU/network require two valid samples. First sample, resumed sampling, counter
reset or a changed interface/CPU scope warms up; missing metrics say unavailable.
A one-shot report therefore gives capacities/load and warming interval counters.

## Optional operator-login launch and recovery

Start manually first. An operator may choose a convenience line in their **own**
interactive login profile after testing it:

```sh
if [ -t 0 ] && [ -t 1 ]; then
    /usr/local/bin/faction-sysop --state-dir /var/db/faction --ansi
fi
```

This returns to the authenticated shell on exit; do not use `exec`, replace getty,
autologin root or change the member account. Keep another normal OS login/recovery
route. No profile changes are installed automatically. A VM console can use plain
output when ANSI or font support is limited; no Ghostty bitmap graphics are needed.

To disable publication, stop/drain the shared service, set `operator_status=false`
or remove the optional key, and restart. Remove only the stale status/temp files
from the private state directory while the publisher is stopped if desired.
A leftover `.operator.status.next` after an interrupted publication prevents a
new publication rather than overwriting an unknown temporary inode; inspect/remove
that one owner-owned file during stopped recovery. Never remove a live lock file.
Never unlink or replace `operator.lease` while a publisher may hold it. It is a
persistent lock inode, not a stale temporary file. If acquisition fails, optional
publication stays unavailable and the board continues; inspect the private file
under stopped recovery before retrying. Old binaries may still read v1 samples;
old dashboards reject v2 instead of treating an unknown identity as current.

Status and lease identity are disposable telemetry, excluded from board export/restore; authoritative
DB/blob/config/key backups follow the existing [backup guide](export-restore.md).

## Validation and remaining work

See the [initial operator evidence](../dev-docs/evidence/operator-netbsd/README.md),
[host-stat/console evidence](../dev-docs/evidence/operator-host-netbsd/README.md)
and [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/95). Software
validation uses an owned two-core/1-GiB NetBSD 11 ARM64 VM. Normal OS-password
login, manual dashboard launch, panels, clean exit/termios restoration and logout
are exercised through its real virtual serial/getty console. Actual shared-owner,
restricted member SSH, owner PTY resize and stop/restart checks remain separate.
No getty replacement, profile change, autologin or physical hardware claim.

[Issue #95](https://github.com/gonz0w/faction-bbs/issues/95) records the current
publisher-lifetime and stale-instance validation. Software checks are not broader
owner artwork approval or a production availability guarantee.
The [Recovery panel](backup-checks.md) now reads explicitly recorded complete-bundle
checks; their historical timestamps are separate from the **Board:** service age.
Collectors cannot invent a peer protocol or a successful restore receipt. #98 host
configuration applies/rollback, #97 firewall and hardening remain separate.
No new capacity claim; the saved playground stays off.
