# Original LORD through the NetBSD SSH doorway

Date: 2026-10-21. Issues [#78](https://github.com/gonz0w/faction-bbs/issues/78)
and [#79](https://github.com/gonz0w/faction-bbs/issues/79). Exact sanitized inputs,
checks, aggregate counts and samples: [JSON record](netbsd-original-lord.json).
The record checker verifies saved evidence; it does not rerun gameplay.

## Scope and environment

Two separate, private NetBSD 11 ARM64 software VMs, each two vCPUs / 1 GiB:
ordinary eight-session maintained Faction shared backend/base OpenSSH on one;
original LORD 4.06 under its own unprivileged game UID on the other. Native Rust
1.96 workspace: **78 tests pass**. Product binaries are non-setuid and original
BBS process/thread/credential limits and normal defaults remain unchanged.
Physical hardware, other ports, production hosting and combined SBC capacity are
not measured here. The signed NetBSD ARM64 release was checked with the published
release-key fingerprint; no automatic NTP/allocator/sysctl tuning was applied.

[Research 0042](../research/0042-original-classic-games.md) records original
artifacts, included terms, exact hashes, rejected candidates and TWGS deferral.
Original LORD.EXE/OVR/art remain unchanged and outside Git. Runtime reports 4.06,
compiled March 14, 2000. LORD 4.07 failed its own version checks in this environment;
it was not patched. The small dosdoor emulator compiled on Linux but did not boot
the selected configuration successfully. Neither failure establishes universal
incompatibility.

Stock pkgsrc DOSBox 0.74.3nb4 sends output but never enables inherited-socket RX.
The separately built [one-line upstream receive fix](../compatibility/README.md)
enables keyboard input, using headless SDL and COM1/internal serial. No FOSSIL
claim. Original START.BAT receives node number and **NORIP without a slash**, as
the original manual specifies. LORDCFG supplies node-specific DOORSYS/open/reset
COM settings. Synthetic aliases are distinct; dropfiles contain an unused password
placeholder and never BBS credentials or automatic BBS identity.

The parser fix discards LORD's observed legacy RIP probe and accepts its bounded
255-left cursor return. It does not enable RIP or forward terminal queries. The
previous parser/complete fixture record remain pinned under
[evidence](../evidence/classic-doors-netbsd/pr80/sources.json); old measurements
were not rehashed to current code.

## Actual game and recovery checks

The [SSH oracle](../../tools/smoke_original_lord_ssh.py) uses real SSH clients,
local pending/approved/revoked BBS accounts, independent synthetic game tokens and
the unchanged original executable. It checks original ANSI/CP437, creates an
original character, deposits 100 fictional gold into Ye Olde Bank and checks the
original stats display. It quits/saves normally, then checks the same balance
after a fresh SSH/BBS/game login and after local escape, resize and revocation/
reapproval. A second game identity gets its own node/dropfile, while a competing
login cannot overwrite the active identity. A killed emulator must be reaped and
the saved character must reopen. Local chat/logout/export remain separate from
the original game. **All 12 checks pass: seven launches / seven reaped, one denied token and one
busy-game refusal.** Final exact counts are in the JSON.

The original title screens rotate and can place their pause caption before the
last ANSI bytes. Early test-driver attempts sent a key during drawing or RIP
detection, then timed out. The final oracle waits for the actual caption (or the uncaptioned title
ending in the author line) and its remaining serial tail. Another diagnostic exposed NetBSD's full-path process
name: basename-only `pgrep -x dosbox` missed a live child. Startup orphan refusal,
drain and failure tests now match the executable path. These unsuccessful driver
attempts are not counted as passing final-oracle checks.

DOSBox rejects DOS region locking. The fixture therefore holds one global OS
lock through launch, play, termination and reaping: **one original game at a
time**, including across the two sequential identities. This is a demonstrated
restriction, not safe concurrent multiplayer support.

## Resource and operational limits

The JSON contains active fixture/DOSBox RSS and VSZ snapshots, not high-water
marks:

| Game-host process | RSS | VSZ |
| --- | ---: | ---: |
| Patched DOSBox / original LORD | 63,952 KiB | 98,712 KiB |
| Private Python fixture | 19,460 KiB | 47,328 KiB |
 Python and SDL/emulator libraries are external test-host costs. Do not
interpret these samples as a peak allowance, CPU budget, maximum sessions or
proof that the BBS and game fit together on a 1 GiB SBC. The emulator's configured
hard limits include 192 MiB data, 4 MiB stack, 48 descriptors, 16 processes,
600 CPU seconds and zero core; the fixture also bounds sessions, input/output and
wait/TERM/KILL cleanup. These are not an arbitrary native-code sandbox.

The private game backup is owned/restored separately from Faction. The recorded
backup check compares a drained original game's files with a private restored
copy; it is not a crash-consistent or online-game backup claim. Faction exports
contain no original game state. Maintain separate protection and backups for the
legacy hop and game host.

The Python host is disposable validation tooling. A maintained host, crash/restart
supervision, account/token management and hobby-sysop installation are tracked by
[#81](https://github.com/gonz0w/faction-bbs/issues/81). No original executable/art,
private transcript, deployment endpoint, key or saved character is published.
Private VMs, forwards and inputs are removed after collecting sanitized evidence.
TWGS was installed in a separate Linux/Wine experiment; its mandatory publisher
activation is owner-deferred. No activation submission or TradeWars gameplay.
