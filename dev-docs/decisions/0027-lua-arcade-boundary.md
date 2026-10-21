# 0027: A separate-UID Lua arcade for the first doors

Date: 2026-10-21. Status: implemented; native runtime and SSH play validation passed.
Issue: [#73](https://github.com/gonz0w/faction-bbs/issues/73), related #10/#34.

## Question and decision

Deliver the owner's first fun Lua menu/game slice without granting the shared BBS
backend permission to spawn extensions or run untrusted scripts. Keep product-first
sequencing, eight sessions, current approval/epochs and two credential permits/
kernel slots. No new Rust registry dependency, JS/TS, HTTP or host tuning.

Choose an optional separate-UID C/Lua evaluator: one daemon, one short-lived child
per screen/turn, one global evaluation at a time. The shared Rust owner exchanges
small nonblocking frames and renders validated text/choices with its theme. A
runtime capability enables Doors only when configured; theme data cannot enable
an extension. First scripts are an editable Lua lobby and two original games.
State belongs to its terminal connection and never carries BBS authority.

Embedding Lua in the backend would widen its failure/authority domain and conflict
with its no-fork/thread ceilings. Spawning per-game processes from that backend
would require relaxing those ceilings. Reusing the research probe unchanged would
not provide real UID/descriptor separation or interactive games. A generic native/
DOS door launcher would expand this first slice; it stays under #34 separately.

## Contract and trust

Root installs text Lua in a protected directory. Kernel peer UID checks protect
the local socket both ways. The child receives source/state/input and only a
write-only reply pipe plus null stdio, not the listener, private log, database,
member identity/session/password or TTY. The loaded API is string/tonumber plus
screen/state/random calls; no domain write, shell, I/O, network/module/debug or
coroutine API. Lua parsing is text-only. The host validates the complete response
and rechecks current local approval before action and result display.

Limits: one child; Lua heap 1 MiB; 100,000 VM instructions; source 16 KiB;
response 4 KiB; state 1 KiB; child data 16 MiB/stack 8 MiB/CPU one second;
files eight; core/memlock/new processes/additional NetBSD threads zero. Parent
wall deadline 250 ms covers native library stalls; the owner request deadline is
two seconds. Child failure discards all output and is reaped. The backend's
original ceilings and credential work limits are unchanged.

Separate UID/process and restricted Lua APIs do not isolate hostile arbitrary
native code or all-family networking after native compromise. The Lua VM/C
wrapper and sysop-installed source remain trusted. Native legacy hosting,
member-submitted code, persistent scores/economies and bigger extension capacity
remain future work. Normal board operation is independent of the service.

## Evidence, sources and portability

Primary sources retrieved 2026-10-21:

- [Lua 5.5 manual](https://www.lua.org/manual/5.5/manual.html): allocator callback,
  text-only chunk mode, hook and library APIs. The newstate seed signature is
  conditional for 5.4/5.5. Installed Darwin 5.5 builds; NetBSD base 5.4 runs.
- [Lua 5.4 manual](https://www.lua.org/manual/5.4/manual.html): the selected native
  API contract. No base libraries/loaders are implicitly opened.
- [NetBSD getpeereid(3)](https://man.netbsd.org/getpeereid.3),
  [setrlimit(2)](https://man.netbsd.org/setrlimit.2), and
  [daemon(3)](https://man.netbsd.org/daemon.3): native identity/limit/startup
  primitives, also exercised in the actual VM.
- Native source headers: NetBSD 11 base `/usr/include/lua.h` reports version 504;
  no extra Lua package was needed. The ordinary first-install rehearsal exposed
  missing `/usr/local` traversability under umask 077 and a private build cwd;
  the helper now explicitly creates the root-owned parent and changes cwd.

Two initial child-budget candidates failed before any screen: the 64-MiB virtual
address cap prevented allocation on NetBSD, and lowering the inherited stack to
1 MiB returned EINVAL. The selected data cap and existing 8-MiB stack envelope
pass actual games and runtime/failure checks. Do not reinterpret those failed
candidates as passing or claim an RSS/whole-host capacity bound. An early SSH
attempt also found the old menu intentionally disabled Doors; explicit host-only
capability routing now covers both arrow and hotkey entry.

## Confidence, reversal and falsifiable tests

Validate real approved SSH play, both games, local peer chat after games, pending/
revoked denial, plain/resize fallback, unavailable/broken worker recovery and
unassisted optional-service boot. Exercise allocator/instruction/control/API
rejections, stalled native Lua library cancellation and healthy recovery. Keep
native source/build observations with the final record and existing historical
capacity bytes unchanged. No physical SBC or new 250-member claim follows.

Disable by stopping board/doors, removing doors_uid and disabling its rc setting;
restart the ordinary board. Reversal leaves the SQLite schema unchanged. Sysop
Lua changes apply next turn; source/state changes may require ending current
runs. The [operator guide](../../docs/lua-doors.md) describes installation and
manual update/rollback. A larger game/persistence API requires a new decision.
