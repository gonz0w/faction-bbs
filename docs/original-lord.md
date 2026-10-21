# Original LORD on NetBSD

Faction can connect an approved SSH member to unchanged **Legend of the Red
Dragon 4.06** running in a separate NetBSD ARM64 game VM. This is an original-game
software proof, with a reproducible private test fixture, rather than a rewritten
Lua game. No game executable, original artwork, registration code or game data is
bundled. See [what was tested](../dev-docs/validation/netbsd-original-lord.md).

The BBS's normal setup remains the [NetBSD quickstart](quickstart-netbsd.md).
Configure a protected game host with [classic server doors](classic-game-doors.md).
A member selects Doors, reviews the destination and connects, then supplies a
**separate game login/token**. Faction sends no automatic BBS identity/password.
Original character names, equipment, fictional gold and saved progress belong to
that game host. They do not appear in local Lua leaderboards or BBS exports.

The tested host permits **one active original LORD game at a time**. Other game
logins get a busy message and return to the board. Stock DOSBox lacks the DOS
record locking needed to claim safe simultaneous nodes. Two independent game
identities use separate dropfiles even though play is serialized. The unchanged
shareware game stops progression at level six; licensing belongs to its publisher.

## Reproduce the private software experiment

Use two separately owned, protected disposable NetBSD 11 ARM64 VMs: one ordinary
BBS and one game host. Each test VM had two vCPUs and 1 GiB RAM; this is not a
minimum requirement or a combined SBC capacity claim. The fixture deliberately
requires a root-owned `/etc/faction-disposable-test` marker and an explicit flag.
Do not install that marker or run fixture tools on a real board.

1. Obtain the specific [original archive and read its included terms](../dev-docs/research/0042-original-classic-games.md).
   Extract its inner LORD.ZIP privately, preserving filenames and dates. Keep the
   original distribution and license outside the public checkout.
2. On the game VM, install pkgsrc Python 3.13/SDL development interfaces and build
   [the exact patched DOSBox](../dev-docs/compatibility/README.md). Stock 0.74.3nb4's
   inherited receive path failed this experiment. No X server is needed for play.
3. Give the game its own unprivileged UID/group and private home/DOS drive; the
   oracle uses `/home/factionclassic/c/lord`. Prepare LORD.DAT through original
   LORDCFG. Configure nodes 1 and 2 for DOORSYS, internal COM, COM from dropfile,
   open/reset COM enabled and paths `C:\NODE1\` / `C:\NODE2\`. Keep START.BAT;
   the fixture enters the game directory before calling it. A temporary private
   display may help with original setup; runtime uses SDL dummy video/audio.
4. Review [the disposable host fixture](../tools/fixtures/original-games/lord_host.py).
   Its private, owned mode-0600 JSON config selects the installed DOSBox, protected
   bind/port and exactly two synthetic accounts. Each has `node` 1 or 2,
   `game_name`, and `token_sha256` of an independently generated 32-byte random
   token. Do not reuse a BBS password or include real personal data. The original
   executable/overlay hashes are checked before startup.
5. Start the fixture as the game UID with `--config` and `--disposable-fixture`.
   Its `runtime/counts.json` contains aggregate test counters. It refuses another
   host instance or an existing game process. Keep the entire VM private: no
   automatic firewall, public bind, host tuning or BBS confinement change occurs.
6. With a fresh ordinary BBS fixture, run
   [the real SSH oracle](../tools/smoke_original_lord_ssh.py). Supply private
   `--admin-argv`, `--member-argv`, `--game-admin-argv`, `--game-tokens` JSON files,
   a protected literal `--game-endpoint`, `--output` and `--disposable-fixture`.
   Optional `--private-debug-dir` retains generated test credentials/transcripts
   **outside Git** for local diagnostics; remove them during teardown.

The fixture is not an installation/account service for a real community. A
maintained launcher needs reviewed supervision/restart recovery, account/token
lifecycle, upgrades and backup operations ([#81](https://github.com/gonz0w/faction-bbs/issues/81)). It is not automatically enabled by
Faction, and must be drained before its owner stops it. Do not claim the process
limits or DOSBox secure mode isolate arbitrary hostile native code.

**Ctrl-] returns to Faction.** Original game quit saves/returns normally. Resize,
BBS revocation and failure close the game connection; the tested fixture reaps
its child. Use a protected legacy hop and retain separate private game backups.
No game state is recovered from a Faction database export.

Original **TWGS 2.20b** has been acquired/installed separately, but its mandatory
publisher activation is deliberately deferred by the owner. No TradeWars gameplay,
universe persistence or concurrency support is claimed yet; see
[#79](https://github.com/gonz0w/faction-bbs/issues/79).
