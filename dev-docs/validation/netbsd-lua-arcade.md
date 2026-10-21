# Native NetBSD Lua arcade validation

Date: 2026-10-21. Issue: [#73](https://github.com/gonz0w/faction-bbs/issues/73).
[Sanitized observations and execution hashes](netbsd-lua-arcade.json).
[Operator/member guide](../../docs/lua-doors.md).

## Actual execution

The current maintained Rust workspace and optional C evaluator built natively on
an official signed/hash-checked NetBSD 11 ARM64 QEMU/HVF VM, two vCPUs / 1 GiB.
NetBSD base Lua is **5.4.6**; no additional Lua package, registry package, JS/TS,
HTTP service, experimental privileged profile, NPF policy or host tuning was
needed. The source map and installed binary hashes describe this increment;
all older capacity records remain tied to their preserved original sources.

**66 native Rust tests** and strict native C build/GCC analyzer passed. The new
terminal capability test proves that both hotkeys and arrows reach Doors only
when enabled by the host. Existing local/worker menus keep Doors disabled.

**12 evaluator runtime checks** passed: a real Lua lobby, Signal Hunt win/state
round trip, complete Neon Salvage run, instruction/allocator/control/API/partial
output rejection, stalled native string-library cancellation/reap, malformed
path/control requests, healthy next turn and orderly socket cleanup. The direct
runtime fixture uses one unprivileged test account; the actual integration below
separately proves the game/BBS/terminal UID boundary.

**11 real restricted-SSH setup/play/recovery checks** passed with two BBS accounts:
optional install disabled by default, game UID unable to read the private board,
pending-before-community denial and approval, Lua lobby arrow/Enter navigation,
Signal Hunt numeric input/exit, a complete five-sector Neon Salvage run, peer chat
after play, service loss/recovery, revoke/reapprove, broken Lua output discarded,
resize cancellation and plain hotkey/Enter play, and a completely unassisted boot
starting the optional service plus approved arcade login. The plain menu also
survives empty/invalid lines. Hidden BBS credentials were never echoed.

The group labels above combine related assertions; the exact named check lists
are in JSON. Native kernel identity checks refused a direct terminal-UID game
request, while the BBS owner successfully reached it; the backend also verifies
the game peer. Root-owned scripts and non-setuid binaries were read back.
The original backend no-fork/thread/descriptor ceilings, two credential workers/
permits/kernel locks and Argon2 parameters are unchanged. Game scripts get no
BBS DB/session/password/TTY/log/listener descriptors or domain-write API.

## Corrected failures

- Fresh ordinary installation from a private build user's checkout revealed
  missing `/usr/local` traversability under umask 077 and an inaccessible inherited
  cwd. The helper now explicitly installs the root-owned parent and changes cwd;
  a repeat with `/usr/local` initially absent passed.
- Initial child address/stack candidates failed before any screen. A 64-MiB
  address cap prevented native allocation; lowering the current stack to 1 MiB
  returned EINVAL. The final child uses a 16-MiB data-segment cap, existing 8-MiB
  stack envelope and separate 1-MiB Lua allocator budget. Neither failed candidate
  is counted as a passing play run or a whole-host memory result.
- The first SSH entry found the original theme intentionally disabled Doors.
  Host-only capability routing now enables the configured shared service; themes
  alone cannot activate execution. A regression covers both navigation routes.
- Plain-menu empty/invalid input originally retained the dispatch sentinel state;
  it now redisplays through the same membership-checked route and is exercised
  by the final plain-terminal play test.

## Scope

This is the first optional Lua lobby and two original solo games for the ordinary
hobby preview. There is no persistent score/economy, native/DOS/TradeWars hosting,
member-uploaded Lua, arbitrary native-code/all-family network sandbox, 250-player
extension capacity, physical SBC result or supported appliance release. Lua/C/OS
and root-installed source remain trusted. Further edge-case/combined-load work
stays deferred; the useful core remains independent of the arcade.

After validation the disposable VM/forwards and generated credentials/disks are
removed. No private endpoint, raw transcript, key or board database is published.

## Subsequent Linux cleanup fix

Linux FORTIFY compilation rejected the ignored `ftruncate` return during orderly
shutdown. Current code returns a failure status if clearing the private pidfile
fails; its Lua/runtime/API/limit paths are unchanged. The final cleanup-return
variant is checked by Linux CI; it was not rerun in the already-removed NetBSD VM.
The original native JSON/digests remain unchanged. Its exact tested C source is
preserved in [the native override](../evidence/lua-arcade-netbsd/main.c), pinned
with the unchanged record by [its manifest](../evidence/lua-arcade-netbsd/manifest.json).
Do not represent those native observations as a test of the later cleanup change.
