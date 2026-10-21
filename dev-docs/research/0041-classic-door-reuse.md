# Reuse-first classic BBS doors

Date/retrieval: 2026-10-21. Issues [#34](https://github.com/gonz0w/faction-bbs/issues/34)
and [#77](https://github.com/gonz0w/faction-bbs/issues/77).
Owner accepted **both original servers and isolated DOS binaries**. Rewriting or
porting is a fallback. Lua examples are original small games, not classic ports.

## Inventory and acquisition status

This is a bounded first inventory, not a catalog of supported titles. Unknown
artifact/manual/license facts remain unknown; #34 stays open. No proprietary
executable was acquired, run or redistributed, and no purchase/vendor contact or
public game account was created.

| Title/variant | Primary source/version observed | Runtime/CPU and interface | Terms/acquisition | Candidate action |
| --- | --- | --- | --- | --- |
| TradeWars Game Server | [EIS](https://eisonline.classictw.com/) describes a standalone original game server; [downloads](https://eisonline.classictw.com/downloads/) lists TWGS 2.20b dated 2012-03-05 | Separate original server; exact supported OS/CPU, Telnet negotiation/login and concurrency require its manual and a real host. No native NetBSD claim | Registration link exists; no redistribution grant reviewed. Linked EXE/revision pages returned HTTP 403 through the research tool, so no artifact/hash was verified | First server-path title under [#79](https://github.com/gonz0w/faction-bbs/issues/79); validate an operator-controlled lawful instance |
| Original DOS TradeWars | Same family, but not interchangeable with TWGS | Exact DOS release, CPU, COM/FOSSIL/dropfile and multi-node locking unverified | No original DOS artifact/license acquired; do not infer terms from TWGS | Inventory a specific lawful original DOS build under #34 before testing |
| Legend of the Red Dragon | [Gameport listing](https://www.gameport.com/bbs/lord.html) identifies 4.07 (2006-06-07), BBS/standalone variants | Listing is insufficient to establish the exact binary/CPU, node/dropfile or FOSSIL contract | Commercial registration listing; license/download not acquired. Linked developer domain redirected to a parked domain, so listing alone does not establish current artifact availability | Original DOS-host candidate, subject to acquisition/manual verification; [#78](https://github.com/gonz0w/faction-bbs/issues/78) |
| LORD II: The New World | [Publisher listing](https://www.gameport.com/bbs/lord2.html), version not stated | [Publisher order page](https://www.gameport.com/cgi-bin/orderform.cgi?product_id=000098) labels the platform DosDoorGames. Specific CPU, COM/FOSSIL/dropfiles and node behavior unverified | Commercial registration listing, not permission to bundle. No artifact or completed purchase | Second DOS candidate after the first title/host is proven |

Old publisher pages and availability links may be stale. A title is eligible only
after recording the actual lawful artifact hash, included license/manual and
required host/interface. Generic archives and a familiar game name are insufficient.
No replacement implementation has been selected for any title.

## Host comparison

| Path | Reuse and footprint | Main unknowns | Disposition |
| --- | --- | --- | --- |
| Original standalone game server | Preserve its game/universe; Faction needs one bounded TCP connection per active player and no local game runtime | Actual title's host requirements/licensing, protected legacy link, terminal quirks, separate login and concurrency | Implement the optional fixed-destination connector first; real TWGS test remains #79 |
| NetBSD DOSBox compatibility host | Preserve DOS binary; pkgsrc lists [DOSBox 0.74.3nb4](https://cdn.netbsd.org/pub/pkgsrc/current/pkgsrc/emulators/dosbox/index.html), including NetBSD ARM64 packages. It is a DOS/CPU emulator with SDL dependencies, not a tiny door service by itself | Original game's FOSSIL/dropfile/node model; headless operation, virtual serial bridge, guest filesystem isolation and measured per-game memory/CPU | Accepted research path, not validated title support |
| Separate DOS compatibility VM/Linux host | Keep Faction/NetBSD small; original games and compatibility dependencies are owned by that host | Extra host operations, isolation/update/backups and per-node quotas | Compare if direct NetBSD emulation is impractical; not a mandatory dependency |
| Future port/rewrite | Could remove an old runtime after game semantics are understood | Source rights, large fidelity/maintenance cost and altered gameplay | Defer until unchanged reuse is shown infeasible |

DOSBox's [serial documentation](https://www.dosbox.com/wiki/Configuration:SerialPort)
lists modem and null-modem facilities, including listen-port/Telnet parameters.
Those are useful building blocks, **not a FOSSIL driver or complete DOS-door
integration proof**. [DOSEMU2's own README](https://raw.githubusercontent.com/dosemu2/dosemu2/master/README.md)
describes Linux hosting; do not present it as a native NetBSD option. Every DOS
route needs a separately owned game filesystem and per-session/node metadata;
never launch it under the BBS owner/frontend/Lua evaluator or relax their limits.

## Representative experiment

First question: can an approved member select one fixed original-server-shaped
endpoint, review it, use ANSI/CP437/keyboard, and return while local rooms/chat and
approval remain intact? Use the maintained ordinary eight-session NetBSD service,
an owned synthetic server and private software VM; no third-party account action.
[tools/smoke_classic_doors_ssh.py](../../tools/smoke_classic_doors_ssh.py) covers
classic-only and Lua co-existence, no connection on cancel, Telnet negotiation,
control filtering, keyboard, EOF, refusal, oversized controls, revocation, resize/
plain fallback, score isolation and ordinary reboot configuration. Actual exact
inputs/results belong in the [validation index](../validation/README.md).

This connector experiment deliberately cannot prove real TradeWars/LORD game
compatibility. The next real-title experiment must use the acquired original
version and demonstrate actual login/turns, saved progress after reconnect,
concurrent players/node locking where supported, disconnect cleanup and return.
The DOS experiment additionally proves exact COM/FOSSIL/dropfile handling, per-node
isolation and measured emulator/host resources. Do not expand either into a full
legacy BBS replacement, pass local BBS credentials as dropfiles, or adopt JS-based
host dependencies. Stop after one verified original title per path, then expand.

## Protocol sources and confidence

The connector uses [RFC 854](https://www.rfc-editor.org/rfc/rfc854.txt),
[RFC 1091](https://www.rfc-editor.org/rfc/rfc1091.txt) and
[RFC 1073](https://www.rfc-editor.org/rfc/rfc1073.txt) as primary Telnet/NVT,
terminal-type and window-size references. It is a deliberately limited client,
not an implementation of every Telnet option/terminal dialect. Exact behavior is
bounded and tested in [classic.rs](../../crates/faction-service/src/classic.rs).

Owner intent is confirmed. Source listings and facilities above are documented;
artifact legality/availability, real title compatibility and native emulator
resource use remain unmeasured. Existing synthetic connector results may support
its narrow implementation, never a supported-game or physical SBC claim.
