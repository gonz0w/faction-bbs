# Run original DOS and TradeWars games

Issues [#78](https://github.com/gonz0w/faction-bbs/issues/78) and
[#79](https://github.com/gonz0w/faction-bbs/issues/79). Follows merged #80.
Status: original LORD software proof complete; TWGS activation owner-deferred.

## Goal and constraints

Prove real original game play through Faction, not another synthetic connector.
Acquire lawful evaluation/original artifacts, pin hashes and read included manuals.
Keep proprietary games outside the public checkout. No purchases, keygens,
registration bypasses, public accounts or vendor messages. No JS/TS dependency.
Keep the ordinary NetBSD BBS/approval/credential/rlimit defaults unchanged; any
compatibility host is optional and separate from BBS/frontend/Lua identities.

## Steps

1. Inspect original LORD shareware and official TWGS artifacts/manuals/licenses;
   compare a small C DOS-door emulator, NetBSD DOSBox and a separate software host.
2. Prepare disposable private compatibility host(s) with dedicated game identities,
   literal loopback/protected endpoints, fixed launchers and bounded cleanup.
3. Run original DOS character creation/gameplay, save/reconnect/return and node
   separation/concurrency behavior supported by the original game. Do not infer
   DOSBox file locking or multiplayer support from a successful screen.
4. Run original TWGS server/game initialization, independent game login/turns,
   persistence/reconnect and permitted evaluation concurrency. Record original
   license/demo restrictions without bypassing them.
5. Connect from an approved member of the maintained NetBSD shared BBS over real
   SSH; verify keyboard/ANSI/CP437, local escape/EOF/resize/revocation and unchanged
   local data/limits. Fix concrete compatibility gaps exposed by original games.
6. Record exact original/host/source hashes, observed game actions and host costs;
   document reproducible optional setup and remaining limits. Preserve historical
   records and sources. Update README/backlog/changelog, open PR and pass checks.

## Discoveries / evidence

- #80 merged at e4261bc56c376e113ae8565bdec8897d4b328672.
- Original LORD 4.07 fails its own version checks in this setup; unchanged 4.06
  works. Exact archives/runtime hashes and original included terms are in
  [research 0042](../research/0042-original-classic-games.md). No original asset,
  executable, registration code or saved character is in Git.
- dosdoor builds on a separate Linux x86 VM but fails startup in this setup.
  Native NetBSD stock DOSBox emits original output but cannot receive inherited
  COM input. One explicit upstream receive-state fix enables input; original
  LORD remains unchanged. No X server is needed during play.
- DOS record locking is unavailable: serialize the original game globally, and
  test competing admission without overwriting active node identity. Two separate
  game identities/dropfiles are exercised sequentially. No concurrent DOS claim.
- Original ANSI exposes a legacy RIP probe and 255-left cursor return. Fix their
  narrow parser handling and preserve the previous parser/whole connector result.
- Real SSH gameplay creates a character, deposits 100 fictional bank gold,
  saves/quits and restores it through new SSH/BBS/game logins, local escape,
  terminal resize and local revocation/reapproval. Final repeatable counts,
  emulator-failure/peer/export checks and teardown belong in the
  [new record](../validation/netbsd-original-lord.md).
- Title art can put its pause caption before its final ANSI bytes: wait for the
  caption and output tail. Original NORIP takes no slash; it removes the RIP-only
  loading/banner phase. NetBSD's process lookup must match DOSBox's full path.
  Early diagnostic failures are not counted as passing oracle runs.
- Original TWGS installer/license/runtime were actually inspected on private
  Linux/Wine. Starter terms are one connection/one game slot; runtime requires
  EIS activation. **Owner explicitly deferred activation.** No activation request,
  vendor contact, public account, purchase or game turn. #79 stays open/deferred.
- Maintained optional game-host/account/supervision/sysop workflow is logged as
  [#81](https://github.com/gonz0w/faction-bbs/issues/81). Python is disposable
  validation tooling, not an adopted production runtime.
- Native updated workspace: 78 Rust tests. Host workspace tests and strict clippy
  pass. 57 native transferred product/build/install source hashes match; all 12 actual
  SSH checks pass, with seven launches/reaped. A drained 42-file game-only backup
  restores byte-for-byte to a separate private copy. All three task-owned VMs,
  forwards, private keys, original assets and synthetic saved data were removed.
  The record checker pins final inputs; no physical/capacity claim.
