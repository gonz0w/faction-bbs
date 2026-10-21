# 0028: A Lua arcade on the ordinary board

Issue: [#73](https://github.com/gonz0w/faction-bbs/issues/73), related #10/#34.
Status: implementation and native play validation complete; final checks. PR #72 merged at `8dc8c6a` before this work.

## Goal and constraints

The owner wants a fun, usable BBS, keeping further 250-user edge cases deferred.
Make [D]oors open a sysop Lua menu and two original games: Signal Hunt and Neon
Salvage. Preserve arrows/Enter, hotkeys, plain fallback, local approval, themes,
existing process ceilings and two credential permits/slots. No JS/TS, board data
or terminal access from Lua; no native legacy title or persistent economy claim.

## Progress

- [x] Merge verified #72; read extension requirements and existing Lua probe.
- [x] Build separate-process bounded Lua evaluator and original examples.
- [x] Connect nonblocking host requests and host-rendered game navigation.
- [x] Native NetBSD separate-UID install/play/failure validation.
- [x] Complete relevant tests/docs and prepare the focused PR.

## Implementation

Optional factiongames account/group, one evaluator daemon and one short-lived
child at a time. Root-owned source, fixed bounded local protocol, peer UID checks
both ways. A child gets no BBS DB/socket/TTY/credential descriptors. Only string,
number conversion and the small bbs screen/state/random API are loaded; no
OS/I/O/module/loader/debug/coroutine APIs. Lua 1-MiB allocation, source/output/
instruction limits and parent wall deadline. Failure discards partial output.

The backend retains RLIMIT_NPROC=0 and its existing helper/thread ceilings;
nonblocking per-turn requests do not add a thread or fork to the board. State is
bounded and held by its owning connection, not a persistent game server/session
identity. The host rechecks current local approval at action/result delivery and
validates all returned choices before rendering its own ANSI with sysop colors.

First Lua menu is the Doors lobby; core BBS navigation remains declarative. Native
legacy doors need a separate future launcher/compatibility boundary. Separate UID
and restricted Lua APIs do not claim hostile native-code or all-family network
confinement. Optional installation leaves services disabled and never installs
experimental loader/setuid/NPF/host-tuning profiles.

## Validation and discoveries

Developer host: evaluator strict compile against installed Lua 5.5 passed after
accounting for Darwin's missing closefrom and deprecated daemon entrypoint.
Pinned rustix uses portable socket/connect plus fcntl nonblock/CLOEXEC rather
than the guessed connect_unix/atomic flags. Service/terminal unit tests pass.
Native NetBSD and real play/failure checks remain in progress. Earlier evidence
is immutable; use its already-preserved actual source bytes rather than refresh
old measured hashes when maintained Rust files change.

Native NetBSD base Lua 5.4 builds without an extra Lua package. The corrected
first install also succeeds with /usr/local initially absent and a private build
user cwd. The 12 actual evaluator tests pass: both games/state/win, memory/loop/
controls/API/partial-result rejection, native C-library stall timeout/reap,
malformed requests, healthy recovery and orderly stop. The 11 actual SSH tests
pass: pending/approval, Lua lobby arrows/hotkeys, numeric Signal Hunt, a complete
five-sector Neon Salvage run, peer chat, service loss/recovery, revoke/reapprove,
broken script, resize/plain play and unassisted optional-service reboot.

Failed native candidates are retained as observations, not counted as passes:
64-MiB virtual-address cap caused allocation failure, and a 1-MiB stack reduction
returned EINVAL. Native data 16 MiB/current 8-MiB stack with the Lua allocator cap
works. An initial SSH test also caught the old Theme disabling Doors; the host-only
capability now covers both routes and has a regression test. No BBS ceiling or
credential cost was changed. See the [final validation record](../validation/netbsd-lua-arcade.md) and exact source/binary JSON.
