# Rust authorization / separate Lua worker probe

Issue #10/#11/#35. Disposable research; not a BBS, production plugin sandbox or
NetBSD Lua support claim. See the [report](../../dev-docs/research/0010-extensible-ansi-platform.md)
and [exact saved evidence](../../dev-docs/research/results/010-lua-boundary.json).

## Run or inspect

```sh
python3 tools/probe_lua_boundary.py --check
python3 tools/probe_lua_boundary.py --run --lua-prefix "$LUA_PREFIX"
```

Run from repository root. `--check` needs Python 3.10+ only and verifies saved
source/oracle provenance without executing a worker. `--run` additionally needs
cc, rustc, and a Lua development prefix containing `include/lua/lua.h` (or
`include/lua.h`), `lib/liblua.dylib` or `lib/liblua.so`, and `bin/lua`. The worker
supports Lua 5.4/5.5 C signatures; only Lua 5.5.1 on Darwin ARM64 was run here.
Temporary binaries are removed automatically; no network, keys or real board data.
Native crash fixtures disable core dumps before triggering abort.

Rust uses only std; our Rust source forbids unsafe code. The isolated C worker
links installed Lua (MIT) through its C API. No crate, JS/TS tool, browser or
third-party game source is introduced. Installed compiler/OS/Lua provenance,
vulnerability review and distribution policy remain broader #31 work; this is not
a claim that the whole host/toolchain has been audited.

## Predeclared oracles and limits

Six dispatch cases: menu allowed; room post allowed; pending denied; revoked
before dispatch denied; missing post grant denied; unavailable OS/I/O/module/
loader/debug/coroutine/pcall APIs remain absent. Six negative worker cases:
allocator growth, infinite Lua loop, control bytes, long output, too many events,
attempted OS API. Finally native child crash, native stalled child, healthy worker
after failures and forged/control-byte protocol input. All 16 must pass.

Per worker: 1 MiB tracked Lua allocation cap, hook every 1000 VM instructions
failing after 20 ticks, eight emitted requests, 120 printable ASCII body bytes,
4096 source bytes. Parent kills/reaps a child after two seconds. Seed zero makes
the disposable Lua state reproducible; this is not a production hash/randomness
policy. No standard libraries are loaded. General game APIs and ANSI rendering
are deliberately absent; menu output is a proposal, never raw terminal bytes.

`MENU|body` and `ROOM_POST|body` proposals contain no authority or member ID.
The host checks synthetic current status and per-action grants, validates the
complete batch and rejects unknown verbs/control bytes. If the worker fails,
all its proposals are discarded; no partial synthetic mutation is accepted.
No database, real registration or transactional revoke race exists in this probe.

The allocator metric excludes process memory, native library/code/stacks, parent
and allocator overhead. Memory/instruction limits cover selected VM behavior;
the parent deadline covers the injected native stall. There is no OS sandbox,
separate restricted UID, descriptor/filesystem/network jail, process-wide memory
bound, global worker/concurrency limit, NetBSD runtime result or actual game.
Native compromise can still use the account's OS privileges. Production doors
need those controls plus real room/role capabilities and lifecycle/versioning.
