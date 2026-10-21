# Running original BBS games

Date/retrieval: 2026-10-21. Issues [#78](https://github.com/gonz0w/faction-bbs/issues/78),
[#79](https://github.com/gonz0w/faction-bbs/issues/79), parent
[#34](https://github.com/gonz0w/faction-bbs/issues/34). Follows the
[initial inventory](0041-classic-door-reuse.md) and merged PR #80.

Question: can Faction members play unchanged original game software through the
maintained SSH doorway, with separate game identities/state and bounded failure?
Rust/NetBSD and the ordinary eight-session BBS stay unchanged. No game executes
under factiond, factionterm, factiongames or the Lua evaluator.

## Acquired artifacts and terms observed

Original included manuals/licenses are the primary authority for these specific
copies. Preservation mirrors and version labels are not publisher signatures.
Artifacts/artwork, generated game data, keys and private transcripts stay outside
Git. No purchase, keygen, registration bypass, public game account or redistribution.

| Artifact | Source and SHA256 | Observed terms/status |
| --- | --- | --- |
| LORD 4.06 shareware | [SLBBS archive](https://slbbs.org/files/doorgame/lord406.zip), `ae46fbd7a5f8d5ed92ae3f54ef9ad1df5efa886284ba1df9b087aae8ae3690cb` | Included LORD.DOC permits unchanged shareware sharing and describes evaluation play until level six; registration enables further progress and restricted options. Runtime reports version 4.06, compiled March 14, 2000. Tested without registration, UPDATE.EXE or PCMICRO.COM. |
| LORD 4.07 shareware | [SLBBS archive](https://slbbs.org/files/doorgame/lord407.zip), `edeae2fd82d6adaca693fc752c5045f1b63103f25187296097625c80b3d2591f` | Original manual describes level-limited evaluation. A [second mirror](https://flisterz.throwbackbbs.com/filelist/BBS_Doors/lord407.zip) has identical bytes. LORDCFG rejects the version; START reports internal error 38019. Restoring ZIP dates/default path did not resolve it. No executable patched; use verified 4.06 for this experiment. |
| TWGS 2.20b | [publisher download listing](https://www.eisonline.com/downloads/), linked [TWGS220B.EXE](https://wiki.classictw.com/filearchive/apps/TWGS220B.EXE), `4a26a37892202dc93f55bc348f02d53bd0fa82a2f59f618db3d2cd6eb329baa5` | Original installer license permits free starter operation with one connection and one game slot; expansion requires registration. The unmodified installer/server launches under a private Wine environment. Gameplay is gated by EIS activation; the owner has deferred activation. |

The TWGS artifact is 3,594,040 bytes, a PE32 i386 Windows installer with a nested
setup/MSI. A normal terminal HTTP client succeeded where the earlier research
browser returned 403. The included activation privacy notice describes identifying
the operator and using supplied contact information for a customer record/news.
Optional identity/location/email fields exist; the technical activation metadata
is not documented. The owner selected leaving activation pending and continuing DOS classics. No activation was submitted, no universe/turns were tested and
no original TWGS compatibility claim follows from installation. #79 remains open.
The Wine experiment used a separate Linux x86 VM, Wine 8.0, no Gecko/Mono download
and disabled mshtml/JScript components. Wine is not a Faction dependency or an
adopted NetBSD TWGS host. Original DOS TradeWars remains a separate inventory item.

## Host results and decision

| Candidate | Actual observation | Decision/limit |
| --- | --- | --- |
| dosdoor C emulator, commit `77ebafa910db59cdfe4ccb055044375241b37a91` | Source contains no JS/TS/manifests. Unprivileged Linux build succeeds; startup fails with low-RAM mapping/serial diagnostics. Upstream advertises Linux/macOS, not NetBSD. | Interesting small-host candidate, not validated title support. Do not relax OS security or BBS process limits to force it to work. |
| Stock pkgsrc DOSBox 0.74.3nb4, native NetBSD ARM64 | Headless original LORD sends its title through an inherited socket, but incoming keys do not reach the UART. | Source inspection identifies an uninitialized inherited receive state. Stock package is not the tested working configuration. |
| Upstream DOSBox 0.74-3 plus one-line receive fix, native NetBSD ARM64 | Unchanged LORD's COM1/internal serial path works with SDL dummy video/audio. See [patch/build provenance](../compatibility/README.md) and [exact validation](../validation/netbsd-original-lord.md). | Separate optional game-only VM/UID. One game globally; not a production launcher or supported platform/title matrix. |
| Separate Linux DOSBox host | Original 4.06 serial title/character setup works using DTR-aware null-modem mode. | Useful fallback, but native NetBSD proof avoids making Linux mandatory for this title. |

DOSBox's [serial implementation](https://svn.code.sf.net/p/dosbox/code-0/dosbox/tags/RELEASE_0_74_3/src/hardware/serialport/nullmodem.cpp)
sets up inherited transmit/polling without transitioning from disconnected receive
state. The patch adds that transition, without modifying LORD. Its
[DOS implementation](https://svn.code.sf.net/p/dosbox/code-0/dosbox/tags/RELEASE_0_74_3/src/dos/dos.cpp)
rejects region locking. Consequently, host serialization is a correctness rule:
there is no concurrent-DOS-node support claim. Future emulators/ports need their
own locking, lifecycle and footprint evidence before raising that limit.

The original LORD interface uses the included START.BAT, node-specific text
configuration, complete CRLF 52-line DOOR.SYS, COM1 at 38400, internal COM routines
and 80-column ANSI/CP437. No external FOSSIL driver was needed in this tested
configuration; this does not establish general FOSSIL support. The
[DOOR.SYS reference](https://wiki.synchro.net/ref:door.sys) and
[WWIV's original LORD setup](https://docs.wwivbbs.org/en/wwiv52/lord/)
help describe the interface; actual artifacts/runtime remain decisive.

## Boundaries and remaining work

The [private fixture](../../tools/fixtures/original-games/lord_host.py) has two
fixed synthetic game accounts with separate random 256-bit tokens, never BBS
passwords. Only game aliases/node metadata enter DOOR.SYS; its password field is
an unused placeholder. A global lock spans child startup/play/reaping. Two bounded
auth connections, fixed command/drive, secure-mode DOSBox, a private inherited
socket, process/resource limits, deadlines and cleanup support the software oracle.
Python is validation tooling, not a selected Faction game-host runtime.

This does not establish a hostile native-code sandbox, production account
management/supervision, a public Telnet service, a CPU budget or 250-user capacity.
The game VM's OS/emulator/native libraries remain trusted. Its state/backup belongs
to its operator; it never enters Faction's local arcade scores or exports.
Revoking BBS membership closes that BBS session, not the separate game account.
Protected networking remains necessary for the unencrypted legacy hop.

Confidence: high for the exact tested original title/configuration and bounded
software observations; unknown for physical SBCs, other NetBSD architectures,
other original releases and multiplayer record locking. Reversal is cheap: remove
the optional endpoint; the BBS database/build/dependencies and normal limits stay.
A maintained optional launcher/account/supervision path is separate follow-up work;
original-server reuse and original DOS reuse both stay ahead of rewriting games.
The maintained host/sysop follow-up is [#81](https://github.com/gonz0w/faction-bbs/issues/81).
