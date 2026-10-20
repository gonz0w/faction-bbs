# Try the local core

Updated 2026-10-19. Early development software, not a network BBS release. This
console exercises real accounts, local sysop approval and durable Citadel-style
rooms. It opens no listener and needs no HTTP, SSH, remote service or JavaScript.
NetBSD remains the primary product target; this new application increment has
developer-host evidence only. Earlier NetBSD dependency probes are separate.

## Build and enter

Use a Unix development host with Rust 1.96+ and a native C compiler for bundled
SQLite. Use a real terminal: hidden passwords are read from `/dev/tty` and are never
accepted as command arguments or environment credentials. Python 3.10+ is needed
for repository/PTY checks. The compiler/toolchain must already be installed.

```sh
cargo build --workspace --locked --release
./target/release/faction --help
./target/release/faction --state-dir "$HOME/.faction-dev" init
./target/release/faction --state-dir "$HOME/.faction-dev" register
./target/release/faction --state-dir "$HOME/.faction-dev" login
```

Choose a sysop handle/password during `init`; run it once. Use another handle for
`register`. Handles are 3–24 ASCII letters/digits/underscores/hyphens, matched
without case. Passwords need at least 12 characters, at most 128 UTF-8 bytes, no
controls; input is hidden and rejects overflow. The initial room is `Lobby`.

A new account can authenticate only to see its pending status. Log in as sysop,
choose `S`, then `A`, and enter the member's handle to approve it. The member must
log in again. `S` then `V` revokes a member and invalidates its existing sessions;
reapproval does not revive those sessions. Only the bootstrap sysop exists in this
increment; member-facing commands cannot promote users or revoke that sysop.

Plain mode commands require Enter; [opt-in ANSI menus](theme-packs.md) add arrows/Enter and direct hotkeys. `R` lists rooms; `N` reads new messages; `H` reads history;
`C` enters [local command chat](local-chat.md). `P` posts a single line or a reply to a post ID in that room. The sysop can create
rooms with `S` then `C`. Pages are chronological, without Reddit-style nesting or
votes. After a page is flushed successfully to the terminal, its last post becomes
the unread cursor. This acknowledges local output, not proof the human read it.

The core accepts bodies up to 1,024 UTF-8 bytes; a page has at most 20 posts and a
2,048-byte domain payload allowance (body/author plus 64 bytes per post). Escaped
console output may be larger: this is not a wire-bandwidth result. Listings show
the first 100 rooms/accounts; sysop status commands still accept any exact handle.
Every approved member currently sees/posts to every local room. Private-room ACLs,
moderation, timestamps, quotas, search and full pagination remain later work.
Exact unread counting and on-disk growth have no demonstrated server-work budget yet.

Posting prints an OS-random submission ID. On database-busy feedback the console
can retry the same body/ID. The core reconciles identical retries to the original
durable post; reusing the ID with changed content fails. The console does not yet
persist an unsent draft across process exit or offer manual ID-based recovery.

Line and hidden-password prompts have a two-minute input budget in both owner
and member sessions. The SSH gateway adds its own documented admission/session
limits; these current development defaults are not yet sysop-configurable.

## Local state and operator recovery

The explicit state directory is created with mode `0700`; the database is `0600`.
Existing state must belong to the effective OS user, have no group/world access,
and use regular files rather than symlinks. Parent directories must already exist
and be trusted. Keep state outside the checkout. No content is encrypted at rest;
the filesystem owner can read/change it. There is no application telemetry or
operational message-body log; chat history/receipts are explicit local application
data with [documented retention](local-chat.md). Development terminal scrollback can still contain displayed posts.

If the sysop password is lost, the filesystem owner can run:

```sh
./target/release/faction --state-dir "$HOME/.faction-dev" recover
```

Enter the existing sysop handle, type `RECOVER`, then choose a new hidden password.
This expires earlier sysop sessions. **This executable has operator authority;
never expose it as a remote login, SSH forced command or member shell.** Members
of the future network service must not get direct access to this database or OS
account. The future restricted adapter needs a separate entry point and reviewed
identity mapping. Local possession of the private store already gives equivalent
operator authority; this command is not a remote recovery protocol.

[Private export/verify/restore](export-restore.md) now provides a live-WAL-aware,
versioned snapshot and restores to a fresh private directory. It preserves durable
accounts/rooms/chat state, removes live chat entries and renews session binding.
Read the guide for private paths, included credential hashes, external theme copies,
initial limits and required account-policy review before cutover. Existing targets
are never overwritten; exports are neither encrypted nor authenticated.

Schema 1 -> 2 migrates transactionally for chat; stop older consoles and take a
complete private offline state-directory copy before upgrading from schema 1.
The new exporter only accepts schema 2. Older binaries reject schema 2; rollback
requires that pre-upgrade copy. Do not copy only a live SQLite database file:
committed data may still be in its WAL. Foreign/future schema versions are rejected.
Use test data until broader upgrade, data lifecycle and deployment gates are met.

## Validate

```sh
python3 tools/check_repository.py
python3 -m unittest discover -s tools -p 'test_*.py'
cargo fmt --all -- --check
python3 tools/check_rust_dependencies.py
cargo clippy --workspace --all-targets --release --locked -- -D warnings
cargo test --workspace --locked --release
cargo build --workspace --locked --release
python3 tools/smoke_local_console.py target/release/faction
python3 tools/smoke_local_chat.py target/release/faction
python3 tools/smoke_export_restore.py target/release/faction
python3 tools/smoke_member_worker.py target/release/faction target/release/faction-member
```

The PTY smoke generates disposable passwords/private state outside the checkout
and checks real prompts, hidden/bounded password input, pending denial, approval,
post/unread persistence, active-session revocation and owner recovery. It prints
no credentials or transcripts. CI uses Rust 1.96.0 on Linux with shell/Python steps.
The [dependency inventory](../dev-docs/dependencies/README.md) covers selected
Linux/NetBSD crate-source graphs; it is not a vulnerability/security audit.

## Presentation and next increment

Plain text remains the default. [ANSI/theme packs](theme-packs.md) now add the
approved identity -> welcome -> vertical arrow/Enter/direct-hotkey menu flow to
the local console, with manual seasonal overrides and resize fallback. Room/member
text remains escaped; sysop art uses a bounded CP437/cell importer, never raw
passthrough. No historical art is bundled. Broader/long ANSI, SAUCE/iCE metadata,
grid layouts, calendar themes and visual review of actual artwork remain work.

[Local chat](local-chat.md) now has a bounded simultaneous-console baseline.
[Restricted SSH](restricted-ssh.md) now uses a separate frontend/broker/member worker;
the complete workspace and terminal/recovery flows passed natively on NetBSD 11 ARM64.
Managed [NetBSD service lifecycle](service-lifecycle.md) and bounded load/failure
checks now pass; account/room lifecycle, complete resource/capacity accounting and
maintained deployment remain next; Lua menus/isolated
doors follow the useful local core. See [scope](../dev-docs/first-local-slice.md),
[ADR 0008](../dev-docs/decisions/0008-local-core-and-credentials.md) and
[ADR 0009](../dev-docs/decisions/0009-safe-terminal-menus-and-themes.md).
