# Native classic-server connector rehearsal

Date: 2026-10-21. Issue [#77](https://github.com/gonz0w/faction-bbs/issues/77),
[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/77),
[ADR 0029](../decisions/0029-classic-server-door.md).
The [machine-readable record](netbsd-classic-doors.json) pins 59 executed code/
fixture inputs and installed binaries. **The game server is an owned test fixture;
no real classic title was acquired, played or declared supported.**

## Actual environment and observations

Fresh signed-release NetBSD 11 ARM64 software VM, two vCPUs / 1 GiB RAM, Rust
1.96.0/pkgsrc Python 3.13, base Lua/OpenSSH and ordinary eight-session shared BBS.
The TCP fixture ran under a distinct unprivileged build UID on guest loopback.
Administrative/member SSH used private host-loopback forwards and serial-verified
host keys. No public server/account action, host tuning or experimental profile.

**77 native Rust tests** and workspace release/native C builds passed. Five new
suites cover fixed-config rejection/default disablement, fragmented Telnet option/
terminal-type/size negotiation, ANSI/CP437/control filtering across every split,
protocol/command limits and real TCP keyboard/EOF/eight-slot release. No new Cargo
dependency or database schema migration was needed.

**13 real SSH check groups** passed:

- Classic-only setup with no Lua; pending/unauthorized members cannot connect.
- Arrow/Enter selection, displayed confirmation and cancel: zero accepted sockets.
- Actual Telnet negotiation, ANSI/CP437, keyboard input and Ctrl-] escape, including
  discarding same-read menu typeahead. Clipboard/DCS payloads/device queries were
  absent from terminal output; server observed no device-query responses.
- Local chat between two members after game return.
- Remote EOF and a refused endpoint return to the healthy BBS.
- Oversized unterminated terminal controls close only the door.
- Revocation closes an idle remote session; new approval requires fresh login.
- Resize closes a game; plain mode connects/negotiates DUMB and returns.
- Adding Lua to existing classic-game TOML tables works; list/lobby navigation and
  local persistent scores coexist without limit changes. The Lua installer now
  prepends its root key before array tables.
- Remote game visits leave the local scoreboard unchanged; only the intentional
  local Lua run appears in owner export.
- Unassisted normal boot retains optional classic configuration and Lua lobby.
- BBS passwords remain hidden.

The fixture counted **six accepted and six closed connections**, ANSI and DUMB
terminal reports and NAWS responses, with zero terminal-query responses. Original
server concurrency, account creation/login, saved universe and title-specific
terminal behavior are still untested. The fixture does not restart at boot; the
boot check verifies installed configuration/menu/service readiness, not a real
external game server's availability.

## Reproduction and cleanup

Build the ordinary workspace/native frontends and Lua evaluator as documented in
[the NetBSD quickstart](../../docs/quickstart-netbsd.md) and
[Lua guide](../../docs/lua-doors.md). On a **fresh marked disposable fixture only**,
`tools/smoke_classic_doors_ssh.py --disposable-fixture … --reboot` accepts private
admin/member argv files, guest source/binary paths and a private result filename.
It owns synthetic accounts/config and starts
[the unprivileged owned server](../../tools/fixtures/classic-doors/server.py).
It refuses an existing board. No transcript, credentials or endpoints are published.

On the developer host, formatting, Clippy with warnings denied, all 77 workspace
Rust tests/release builds, existing console/chat/export/member/broker/credential
PTY regressions and 100 Python tests passed. Repository/model/evidence validators
preserve earlier records and executed sources. Linux CI exercises the same Rust
TCP/codec tests and the existing Lua evaluator suite.

After capture the task VM was stopped, both forwards closed and generated private
keys/disks/inputs removed. Unrelated VMs and paused research were untouched. The
previous [score record](netbsd-arcade-scores.json) is unchanged; its seven changed
inputs are preserved under
[pr76/sources.json](../evidence/arcade-scores-netbsd/pr76/sources.json).
The earlier first-arcade record/C override and capacity evidence remain unchanged.

This is connector functionality evidence on one software target. It establishes
no original-title compatibility, 250-member capacity/headroom, physical SBC,
complete terminal dialect, end-to-end Telnet encryption or parser-process sandbox.
