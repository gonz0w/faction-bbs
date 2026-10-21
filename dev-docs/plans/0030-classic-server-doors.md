# Classic server doors and reuse inventory

Issues: [#77](https://github.com/gonz0w/faction-bbs/issues/77),
[#34](https://github.com/gonz0w/faction-bbs/issues/34). Status: implementation/validation complete; PR checks pending.

Owner accepted both unchanged game servers and isolated original DOS binaries on
2026-10-21. Implement the server connector first; keep DOS hosting as a separate
bounded follow-up. Bundled Lua games are original testable examples, not ports.

## Steps

1. Record primary-source title/runtime/license/interface inventory and compare
   original server, compatibility host and future ports. No redistributed games.
2. Add disabled-by-default fixed sysop-configured endpoints (at most six), member
   destination confirmation and an eight-active-connection cap. No DNS, dynamic
   destination, credential/identity export or local game process launch.
3. Implement bounded nonblocking Telnet negotiation, CP437 text and basic ANSI
   filtering. Unknown terminal controls/queries/strings never reach the member
   terminal; Ctrl-] returns to BBS. EOF/error/resize/revocation cleans up. All
   existing backend process/thread/descriptor/credential limits stay unchanged.
4. Rehearse real menu/connect/keyboard/return with owned synthetic game servers,
   fragmented protocol/control fixtures and NetBSD SSH; record source hashes,
   cleanup and exact scope. This is connector evidence, not a TradeWars title claim.
5. Update README/docs/changelog/ADR, preserve earlier native inputs, open PR and
   pass CI. Per-title original DOS gameplay and real TWGS validation remain open.

## Boundary decision

The original game executes on the separately operated server/compatibility host.
Faction opens a fixed configured TCP connection and parses terminal traffic in
bounded safe Rust; it does not execute a local classic or give Lua network APIs.
No mandatory helper, currency, public deployment or game-server dependency. The
plaintext legacy hop must use loopback/a protected private path configured by the
sysop. The maintained experimental isolation profile is not adopted for this path.

## Progress and validation

NetBSD native: 77 Rust tests/release/native builds and 13 real SSH check groups
passed. Six owned-fixture connections all closed; ANSI/DUMB/NAWS negotiation and
zero leaked device-query responses were observed. Classic-only and Lua coexistence,
confirmation cancel, local escape/EOF/refusal/control failure, revocation, resize/
plain, local chat/scores and normal reboot were exercised. See
[the exact validation record](../validation/netbsd-classic-doors.md).

Developer fmt/Clippy/release/workspace tests, existing terminal regressions,
100 Python tooling tests and all 23 policy/model/evidence validators passed. Previous score
inputs were preserved; old records remain unchanged. Task VM cleanup is recorded.
Original DOS (#78), actual TWGS (#79), and remaining per-title #34 work stay open.
Keep the ordinary eight-member defaults and no new capacity/SBC claim.
