# ADR 0008: Local core, durable room actions and credentials

Status: accepted for the first development increment, not production adoption.
Date: 2026-10-19. Issues: #6/#9/#12/#21/#30/#35. Owner direction: begin building
after current research merges; Rust/primary NetBSD and software-first scope remain.

## Question and constraints

How can we execute approval-gated room behavior now without prematurely exposing
a network service or coupling local identity to SSH? Keep local ownership, no JS,
tiny bounded work, developer-host/VM testing and later adapters/extensions. This
increment must have actual authorization/persistence tests and runnable instructions.

## Alternatives and decision

| Question | Alternatives | Decision and tradeoff |
| --- | --- | --- |
| First execution surface | Full SSH server; synthetic probe; local console + reusable core | Real local console exercises persistent behavior now; SSH identity/confinement and network accounting remain open. The operator-capable console is never the future SSH entry point. |
| Persistence | Ad hoc files; external database; prior SQLite candidate | Pin rusqlite 0.40.1 with bundled SQLite. Existing native NetBSD/recovery evidence supports a small local transaction store; native C and a larger binary are accepted costs. No database service or async runtime required. |
| Credentials | Weak fast hash; home-grown crypto; SSH-only identity; established password KDF | RustCrypto Argon2 0.6.0, Argon2id v19, 19,456 KiB / 2 iterations / 1 lane / 32-byte output, random per-password salt and PHC storage. Security work intentionally costs about 19 MiB per active hash; never weaken it to meet idle budgets. Network login concurrency/rate limits must be designed before exposure. |
| Session authority | Cached approval; public member ID; opaque current session | Board-random identity + member ID + credential/status epoch, held in an opaque non-serializable Rust session. Every action reads current role/status/epoch inside its transaction. Status changes/recovery increment epoch; old sessions never revive. No persistent bearer credential or transport address in the core. |
| Room model | Nested/voted threads; chronological rooms | Chronological local messages, same-room reply reference, monotonic per-member unread cursor. All rooms are approved-member rooms initially; per-room ACLs are not claimed. |
| Duplicate submit | Blind insert; content deduplication; scoped retry ID | Unique author/request ID with exact room/body/reply reconciliation. Duplicate identical submissions return the original; changed content conflicts. Authorization precedes reconciliation. |

The root workspace contains `faction-core` and the separate `faction` local CLI.
Own Rust source forbids unsafe code. Dependencies may use unsafe/native code;
this is not an all-safe-code guarantee. The CLI uses existing rustix 1.1.5 safe
termios wrappers and zeroize 1.9.1 for bounded hidden local password input rather
than adding a password-input crate with unbounded collection. OS entropy uses
getrandom 0.4.3. CLI buffers are wiped on ordinary drop, which does not guarantee
erasure of every compiler/OS copy, fatal termination, swap or terminal history.

## Security and consistency boundaries

- Private effective-UID-owned state directory/regular database and existing
  sidecars; reject symlinks and group/world permissions. Trusted parent directories
  and same-UID operator processes are assumed. This is not a hostile-filesystem
  race defense or encryption at rest. The owner can already modify the database.
- Schema application ID/version are checked before WAL configuration; foreign or
  future databases are rejected. Version 1 is initial schema, not a general upgrade
  system. WAL + FULL synchronization, foreign keys, 100-ms busy wait, 2-MiB SQLite
  cache target and disabled mmap are local defaults, not measured process ceilings.
- Authorization and writes share an IMMEDIATE transaction. Once revoke commits,
  later actions deny stale epochs. An already-authorized read snapshot may finish;
  no retroactive withdrawal of output already delivered is promised. Database role
  changes are also checked even if a trusted operator omitted epoch invalidation.
- Separate local-owner recovery resets an existing sysop credential and expires
  old sessions. It is absent from the member menu. No multi-sysop promotion/status
  policy exists yet; member commands cannot disable the bootstrap sysop.
- Credentials are bounded (12 characters minimum, 128 bytes maximum at creation);
  generic invalid-credential messages do not imply timing-resistant account
  enumeration. Unknown accounts avoid the KDF today. No remote login admission,
  concurrency limiter, lockout or MFA exists. Do not expose this console remotely.
- Post bodies reject controls other than LF; the console additionally escapes all
  non-ASCII/control display text. This is a useful plain-text baseline, not an ANSI
  sanitizer/theme engine. Oversized command input terminates rather than executing
  a truncated command. Password echo is disabled before readiness is printed,
  restored on normal returns, and buffered input is flushed. Fatal signals can
  still require the local user to restore terminal settings.
- Domain pages cap at 20 posts and 2,048 charged bytes; bodies cap at 1,024 bytes.
  Wire escaping is extra. Account/room lists cap at 100; full pagination, total
  storage/member quotas, retention and moderation remain open. No capacity promise.
- No listeners, outbound calls, chat, publication, Lua, shell/door execution or
  financial capability. Backup/export/migrations and crash fault injection against
  this application remain required; older generic SQLite experiments do not close
  those application gates. Draft persistence after console exit is not implemented.

## Primary sources and dependency review

Retrieved 2026-10-19; versioned crate sources were fetched via Cargo and inspected.

- [OWASP password storage guidance](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html): memory-hard salted password storage and the 19-MiB/two-iteration/one-lane Argon2id baseline inform the initial parameters; SBC tuning remains measurement work.
- [RustCrypto Argon2](https://docs.rs/argon2/latest/argon2/): established implementation, PHC representation, salt/verification APIs and zeroize feature. Exact 0.6.0 source/manifests are pinned in Cargo.lock.
- [getrandom 0.4.3 platform behavior](https://docs.rs/getrandom/0.4.3/getrandom/): OS entropy abstraction including NetBSD; no browser/JS backend enabled.
- [rustix 1.1.5 tcgetattr](https://docs.rs/rustix/1.1.5/rustix/termios/fn.tcgetattr.html): safe terminal wrapper; paired tcsetattr and source types reviewed for hidden bounded input.
- [SQLite WAL](https://www.sqlite.org/wal.html): single-machine shared memory, sidecars, concurrent readers/single writer and backup implications. Prior Faction native/storage evidence is in [#23 research](../research/0023-netbsd-dependencies.md) and [#9 research](../research/0009-local-storage-comparison.md).
- [Selected package/version/license/feature inventory](../dependencies/README.md): MIT/Apache-compatible crate license declarations, bundled SQLite public-domain source notice, no prohibited filenames/manifests in the selected Linux/NetBSD source trees. Not a legal, vulnerability, generated-code or unsafe-code audit.

## Evidence, confidence and falsifiable next checks

High confidence in the narrow tested local behavior: 14 core integration cases,
two console unit cases and a real disposable PTY smoke passed on the ARM64 Darwin
developer host. These include independent connections, concurrent bootstrap/retry,
revoke/reapprove, cross-board sessions, persistence, owner recovery, salted hashes,
bounded/control-safe output and hidden password/echo restoration. CI repeats the
application tests on Linux/Rust 1.96.0; its result must be read on the PR.

NetBSD 11 ARM64 previously ran Mio/rusqlite/OpenSSH probes. **This new credential
and console implementation has not yet run in that guest.** Source inventory for
a NetBSD target is not a native build/run result. No physical hardware tested.

Next falsification: native NetBSD build/PTYS; application export/restore + crash
faults; multiple simultaneous members with real restricted SSH identity; safe
ANSI/menu tests; local chat/backpressure; hash concurrency and full 100/250-member
resource accounting. Any approval bypass, stale-session revival, duplicate durable
post, terminal control escape, or unrecoverable committed message fails the slice.

## Reversal cost

The CLI is replaceable without domain changes. SQLite schema migration/export and
password rehash-on-login need explicit future versions. A transport can use the
same checked actions but must not receive owner recovery authority. Changing the
store/KDF requires measured migration and recovery tests; no stable external API,
schema or on-disk compatibility promise is made for this development increment.
