# NetBSD arcade scores rehearsal

Date: 2026-10-21. Issue [#75](https://github.com/gonz0w/faction-bbs/issues/75),
[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/75),
[ADR 0028](../decisions/0028-local-arcade-scores.md).
Exact inputs, installed binary hashes and sanitized outcomes are in
[the machine-readable record](netbsd-arcade-scores.json).

## Environment and result

A fresh signed-release NetBSD 11 ARM64 software VM, two vCPUs and 1 GiB RAM,
ran the ordinary eight-session shared service with base OpenSSH/base Lua 5.4.6.
Rust 1.96.0 and Python 3.13 came from pkgsrc. The private loopback-forwarded
fixture used separate build, BBS-owner, terminal and game identities. No tuning,
experimental isolation/admission profile or physical SBC was used. All 66
executed source/fixture inputs were compared with the candidate before cleanup.

- **72 native Rust tests** passed, including five new core score suites and one
  new result-frame suite; release workspace and native C frontends/evaluator built.
- **18 actual evaluator checks** passed: real Lua menus/guide, Signal Hunt win,
  loss, invalid-input and abandon scoring, completed/early-land Neon Salvage,
  duplicate/negative/active-score rejection, existing memory/instruction/control/
  API/partial-output checks, native-library wall timeout, recovery and cleanup.
- **17 real SSH check groups** passed: fresh optional install/approval, guide,
  arrow/Enter and hotkeys, both games, persisted best/count, repeated top-ten
  browsing without resubmission, replay/early landing, disconnect/new login,
  peer chat, service loss/recovery, revocation/reapproval, broken Lua, plain/resize,
  private owner export/verify/restore and unassisted boot with the same score/count.
- Strict native C `-Wall -Wextra -Werror -fanalyzer` passed. The terminal UID's
  direct game-socket request was rejected; the game UID could not read the board
  database. Installed product executables remained root-owned, non-setuid and
  installed scripts root-owned/non-writable by members.

The core tests additionally exercise twelve-player ordering/top-ten/ties,
reopen, lower-score retention, wrong-member/board/epoch and pending authority,
one-shot completion, busy retry, the 64-game limit, failed migration rollback,
schema-2 read-only backup compatibility, schema-3 score roundtrip and malformed
score imports. These run natively as part of the 72-test total.

## Reproduction

On a fresh marked disposable NetBSD fixture only, build the Rust workspace and
three native binaries, then use the ordinary installer. The SSH test owns its
synthetic account setup; it refuses an existing board. Private argv/known-host
files stay outside the checkout. See the scripts' `--help` for required arguments.

```sh
cargo test --workspace --release --locked
cargo build --workspace --release --locked
python3 tools/build_terminal_frontend.py --output target/release/faction-tty
python3 tools/build_service_daemon.py --output target/release/faction-daemon
python3 tools/build_lua_doors.py --output target/release/faction-doors
```

`tools/smoke_lua_worker.py` runs unprivileged against root-owned copies of
`doors/examples/*.lua` and `tools/fixtures/lua-doors/*.lua`.
`tools/smoke_lua_doors_ssh.py --disposable-fixture … --reboot` exercises the actual
installed SSH/backend/game services. Native C analysis uses base `cc` and
`-std=c11 -D_GNU_SOURCE -Wall -Wextra -Werror -fanalyzer -I/usr/include -fsyntax-only`.

On the macOS developer host, Rust formatting/Clippy with warnings denied, all
72 workspace tests, release builds, existing local console/chat/export/member/
broker/credential PTY regressions and 100 Python tooling tests passed. The Lua C
wrapper also compiled against installed Lua 5.5; its native runtime was tested on
NetBSD/Lua 5.4, not claimed for every developer-host/library combination. Repository
policy and all 23 model/evidence validators passed. CI covers Linux/Lua 5.4 too.

## Preservation and limits

The prior [first arcade record](netbsd-lua-arcade.json), original C override and
manifest are unchanged. Its eight now-changed execution inputs are preserved
under [pr74/sources.json](../evidence/lua-arcade-netbsd/pr74/sources.json), with the
original record hash; unchanged inputs still resolve from that dated record.
All earlier capacity records/source snapshots remain untouched.

The task VM was stopped and removed after capture; both forwards were closed and
private generated credentials, disks and inputs deleted. Unrelated VMs and paused
research were untouched. Public records contain no endpoint, credential or play
transcript. This is functional software evidence, not a new 250-member memory
measurement, physical SBC support, production sandbox or game-integrity attestation.
