# Your board's Lua arcade

The shared NetBSD backend can now open an optional Lua-driven Doors lobby and two
original terminal games. Choose **[D]oors**, then use arrows/Enter or the displayed
hotkeys. The host draws the screen with your BBS theme; Lua supplies text and
choices. Plain terminals use hotkeys followed by Enter. Ctrl-C returns to the BBS.

**Signal Hunt** gives you seven guesses to lock onto a pirate radio frequency.
**Neon Salvage** is a five-sector run: safe salvage or risky hacking, then dock
and count your points. Scores are local to that play session and have no monetary
value. Disconnect, timeout, resize or returning to the lobby discards the game.
A broken/unavailable door returns to the BBS; rooms and chat remain usable.

This first Lua menu is the arcade lobby. The main BBS menu remains declarative.
Local owner consoles and worker rollback retain their existing menus with Doors
unavailable. Native/DOS games such as TradeWars are future compatibility work;
this runner executes text Lua scripts only.

## Add the optional service

Start from the [ordinary NetBSD board](quickstart-netbsd.md), with the first sysop
initialized. NetBSD 11's **base Lua 5.4 headers/library** suffice; no extra Lua
package is required on the tested ARM64 image. From the source checkout:

```sh
python3 tools/build_lua_doors.py --output target/release/faction-doors
```

As root, announce maintenance and stop member SSH and the backend. Run the first
optional installation from the built checkout:

```sh
/etc/rc.d/faction_sshd onestop
/etc/rc.d/faction_broker onestop
python3 tools/install_lua_doors.py
/etc/rc.d/faction_doors onestart
/etc/rc.d/faction_broker onestart
/etc/rc.d/faction_sshd onestart
```

The helper creates a separate **factiongames** user/group, installs root-owned
scripts and a non-setuid evaluator, and adds `doors_uid` to the root-owned service
configuration. Its primary group is separate from the BBS owner/terminal group;
it cannot read the private board directory. The setgid IPC directory lets the
BBS owner connect; kernel peer UID checks admit only the configured owner, and
the owner verifies the game service UID. Games receive no BBS identity/password,
room data, session token or terminal descriptor.

Installation leaves the game service disabled and starts no listener. After a
successful play-through, opt into boot startup:

```sh
printf 'faction_doors=YES\n' > /etc/rc.conf.d/faction_doors
```

The optional service starts before the backend. It uses a local Unix socket,
not a network listener. Do not admit the game account through another SSH daemon
or reuse it for other services. No firewall, setuid launcher, experimental loader
or host-tuning profile is installed.

## Make the arcade your own

Edit `/usr/local/share/faction/doors/doors.lua` as root. Keep installed scripts
root-owned and not writable by group/other. Changes apply on the next game turn;
coordinate script/state-format changes with players. Add a text `.lua` file with
a lowercase ASCII slug (letters/digits/hyphens, at most 32 characters), then add a
choice to the lobby. For example:

```lua
bbs.title("Midnight Arcade")
bbs.text("Welcome to the sysop's after-hours corner.")
bbs.option("H", "Find the pirate broadcast", "game:signal-hunt")
bbs.option("N", "Run the neon salvage route", "game:neon-salvage")
bbs.option("B", "Back to the board", "back")
```

The `[D]` label, colors and welcome still belong to your normal theme. Change the
Doors label there if using an older pack that says "not installed". Lua cannot
send arbitrary ANSI/control bytes. All examples are original project source
under BSD-3-Clause; historical scene art and legacy games are not bundled.

## Small screen API, version 1

Each turn starts a fresh Lua state with `state` (the previous returned string) and
`input` (this turn's submitted input). On entry both are empty. Only the string
library, `tonumber`, and this `bbs` table are available:

| Call | Purpose and limit |
| --- | --- |
| `bbs.title(text)` | One screen title, at most 64 printable ASCII bytes |
| `bbs.text(text)` | Up to 16 lines, each at most 120 printable ASCII bytes |
| `bbs.option(key, label, target)` | Up to 8 distinct uppercase-letter hotkeys; label up to 48 bytes |
| `bbs.prompt(text)` | One line-input prompt, up to 120 bytes; use instead of options |
| `bbs.state(text)` | Return up to 1024 bytes for this connection's next turn |
| `bbs.random(max)` | An OS-random integer from 1 through max; max at most 1,000,000 |

Targets are `back`, `lobby`, `game:slug` (fresh game), and `input:value` (next turn
of the current game). Line input is at most 128 bytes. Field values exclude
control characters and the protocol separator `|`. A screen must have a title,
state (empty by default), and either choices or a prompt. The host validates the
entire response and current local approval before displaying it or acting on a
choice. No script target can approve an account or invoke an owner operation.

## Boundaries, updates and disablement

Only sysop-installed Lua is supported. One evaluator child runs at a time outside
the BBS process, with a 1-MiB Lua allocator cap, 100,000-instruction hook budget,
16-KiB source cap, 4-KiB response cap and 250-ms parent deadline. The child data segment is
capped at 16 MiB, stack at NetBSD's 8-MiB envelope, CPU at one second, descriptors
at eight, and core/locked memory/new processes/additional NetBSD threads at zero.
These are separate game budgets; the BBS's existing credential and process/thread
limits stay unchanged. The backend uses nonblocking requests with a two-second
response deadline and adds no game thread or fork.

The child has only null stdio and a write-only reply pipe. No `os`, `io`, module
loader, bytecode loader, debug API or coroutine API is exposed. Failed/partial
results are discarded; the parent kills/reaps overdue work. The Lua implementation,
C wrapper and OS remain trusted. Separate process/UID and restricted APIs are not
a hostile-native-code or all-family network sandbox. Member-uploaded Lua and
arbitrary native executables are outside this version's boundary.

To update, stop the board and door service, preserve your edited Lua files, build
and install the new evaluator/scripts, then start doors before the board. The
first-install helper refuses an existing installation; it is not an upgrader.
Keep the root-owned before-doors configuration copy for reference. Back up your
Lua files separately from the private database snapshot.

To disable, stop member SSH/backend, stop `faction_doors`, set its rc setting to
`NO`, and remove `doors_uid` from `/etc/faction/service.toml`. Restart the ordinary
backend and SSH; Doors becomes unavailable. Keep live pidfile/credential locks
in place. Startup diagnostics live in `/var/log/faction-doors.log`; game bodies, inputs
and scores are not logged by default.
