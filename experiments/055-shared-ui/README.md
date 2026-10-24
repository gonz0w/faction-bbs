# Shared UI and ANSI themes candidate (#55)

This opt-in Rust shared owner extends the frozen #58 headroom backend with the
existing local UI features. It is never installed by normal services. The normal
gateway remains eight per-session workers; this experiment has a fixed 250-client
cap, burst eight / 100-ms admission refill and two credential operations. See
[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/55),
[research 0032](../../dev-docs/research/0032-shared-ui-themes.md) and
[ADR 0018](../../dev-docs/decisions/0018-shared-ui-themes.md).

## Build and private fixture

Use Rust 1.96, the reviewed root dependency inventory and an isolated target:

```sh
CARGO_TARGET_DIR=/tmp/faction-ui-target cargo build --manifest-path experiments/055-shared-ui/Cargo.toml --release --locked
python3 tools/check_ui_dependencies.py
```

The owner UID must be non-root and different from the terminal UID. Use the existing
protected descriptor endpoint and separate terminal identity from the
[restricted SSH guide](../../docs/restricted-ssh.md). Stop/reap the normal broker
before binding its endpoint. The terminal UID cannot read/open the private board.
Only an operator starts this process, for example with owner-selected arguments:

```text
shared-bbs --socket SOCKET --terminal-uid TERMINAL_UID --state-dir PRIVATE_STATE --theme OWNED_PACK --season OWNED_OVERRIDE
```

The first six arguments retain the explicit order above; optional `--theme`,
`--season`, `--plain` or `--ansi` follow. ANSI is the experiment default, subject
to the received TERM and actual TTY geometry. Plain input is used for dumb/empty
TERM or unsuitable dimensions. Theme options are processed in order; select base
before season. Unlike the local console fallback, an invalid explicit startup
pack/override fails before binding. Omitted packs use the built-in text theme.
No terminal command can select filesystem paths, load scripts or invoke owner
bootstrap/recovery/export/restore. BBS sysop actions still require the core role.

The shared owner validates packs and CP437 art once at startup. All members use
one coherent snapshot; restart selects a new manual season. No calendar scheduling,
hot reload, new scene art or third-party rights are implied. Fixture text/glyphs are
original BSD-3-Clause test data, not approved identity artwork or an art catalog.

## Behavior and explicit bounds

Approved login precedes identity, welcome and menu. Registration confirms the
hidden password and remains pending. Arrows/Enter and displayed case-insensitive
hotkeys use `faction-terminal`; disabled doors/sysop labels never grant actions.
Menus preserve custom ordering and use the same core actions in plain mode.

Room/account lists page ten of the core's first hundred rows. Selected room pages,
unread/history cursors, same-room replies and retained submission IDs implement the
local behavior without a blocking stdin loop. A read cursor advances only after
the output page drains to the TTY. Posting stays one bounded 1024-byte UTF-8 line (non-ASCII display is escaped).
The original local core still supplies all authorization and durable writes.

Chat supports list/join/part/names/history/poll/me, literal slash, quit, and the
role-checked create/allow/deny/kick commands. A bounded ASCII tail editor preserves
unsent text across ANSI delivery; grants/epochs are rechecked by the core. Channel
denial discards the selected draft and requires explicit rejoin. A new lease resets
its request sequence. Plain chat fetches on explicit commands; heartbeat remains
periodic. Ctrl-C/D leaves chat; at other prompts it terminates the connection.

Menu/chat resize discards the draft and falls back to plain for that action; the
next menu can regain ANSI. Unknown chat controls use the existing quarantining
decoder; menu/editor transitions flush typeahead. Normal responsive ANSI cleanup
restores SGR/cursor and saved termios; a blocked TTY is best effort and fatal
termination may need terminal recovery.

- One shared Board and two fixed credential workers; no unbounded task/thread
  spawning or caller-supplied member/session/role capability.
- Two permit lifetimes cover queued work, execution and unconsumed results; the
  unchanged two kernel credential slots also cover independent owner processes.
- Round-robin work starts rotate by 16; a sweep permits 16 input, geometry or
  background work steps. Input reads at most 64 bytes / 20 ms per terminal and
  submits at most one action. Arrow movement performs no SQLite queries. This is
  bounded scheduling, not a hard realtime guarantee for SQLite or the OS.
- Output starts at 1 KiB, grows lazily to at most 32 KiB **including CRLF encoding**
  and canonical safe art; no transaction waits behind terminal writes. Each
  nonempty output batch has the unchanged five-second deadline. Idle prompts have
  120 seconds; connection lifetime is 1800 seconds.
- Chat input uses the existing 768-byte editor, posts 1024, passwords 128. Invalid
  UTF-8 secrets stay in zeroizing storage. Control/descriptor protocol is unchanged.

## Native validation and reproduction

Follow the signed disposable NetBSD VM procedure in the #58 experiment and
research reports; no physical SBC is required. Build root tools/collectors and this
candidate with separate Cargo target directories. Install the candidate at the
fixture's `shared-bbs` path and the existing analyzed minimal frontend. Record the
installed executable hash as well as source hashes; do not reuse same-name builds.

Run `tools/check_ui_oracles.py --help`, then `tools/check_ui_host.py --help` against
the explicitly marked private fixture using private SSH keys/known_hosts and the
protected synthetic secret file. They do not provision real systems. The latter
checks NTP pageable mappings and all base services before/after staged
32/100/200/250 workloads, plus real member/owner credentials, full UI work and
known 0/19/38-MiB controls. Whole-VM counters include admin overhead; dedicated
SSH roles are all counted. Sampled peaks are lower bounds, not continuous reserves.

`reset_fixture.py --offline-disposable-fixture` is owner-only, validates the
marker, stopped endpoint and known synthetic rows before restoring the exact seed.
Never use it on an operator board. Keep credential lock inodes in place.

The guest-wide allocator/NTP policy remains research-only, with exact rollback.
Production confinement, authenticated transport admission, active workload mixes,
soak, appliance updates/timekeeping and normal-service adoption remain #20/#28/#55.
Run `python3 tools/check_ui_evidence.py` to validate saved evidence, not rerun a VM.
