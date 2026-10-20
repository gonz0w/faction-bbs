# 0008: First local identity and room implementation

Date: 2026-10-19. Status: published for review in [PR #48](https://github.com/gonz0w/faction-bbs/pull/48). Issues: #6/#9/#12/#21/#30/#35.

## Authorized outcome and scope

Owner asked to begin building after merging research. Deliver a reusable Rust core
and a local, interactive operator/member console: one-time sysop bootstrap,
password authentication, pending registration, approve/revoke, current authorization,
durable local rooms/posts/replies/unread cursors and retry-safe posting. This is
the first implementation increment, not the complete core or a network deployment.
Rust/NetBSD direction and rusqlite/bundled SQLite build compatibility are accepted
inputs. Record narrow application/credential/schema decisions with primary sources.

No listener, SSH gateway, HTTP, chat federation, Lua, file sharing, games, public
publishing or funds. Do not substitute generated block art for the art direction.
The first console is an explicitly plain-text development surface; full arrow/ANSI
theme behavior remains the next UI increment and must not be claimed implemented.

## Invariants and predeclared tests

- Private owner-only state directory/database; reject symlinks, unsafe permissions,
  foreign/future schema versions and repeated bootstrap; initial schema version 1,
  with explicit upgrade migrations still required before supporting retained data.
- Random salts/OS entropy, bounded passwords and Argon2id PHC hashes; never store
  or log plaintext or accept credentials in command arguments. Hide password input.
- Opaque board-bound session handles; current status/role/credential epoch checked
  inside each action transaction. Revoke/reapprove or recovery cannot revive an
  old session. Test cross-board handles and independent DB connections.
- Sysop-only management; pending/revoked members cannot list/read/post/mark-read.
  Last sysop/self-lockout policy explicit. Local-owner recovery stays a separate
  operator path, never a member menu/transport command.
- Room posts chronological; replies remain in the same room. Parameter-bound SQL,
  bounded text/pages, monotonic unread cursor and request-ID/body reconciliation.
  Check duplicate/mismatched retries, revoked retries, concurrent writes and reopen.
- Meaningful Rust integration tests plus a disposable PTY CLI smoke; no committed
  generated credentials, databases, hashes or real data. Runtime fixtures outside
  the checkout; dependency/license/no-JS source inventory before PR.

## Work sequence

1. Pin/review dependencies; introduce small Cargo workspace and secure store/schema.
2. Implement auth/operator/domain APIs and regression tests.
3. Add bounded local console and reproducible private smoke workflow.
4. Record decisions/threat boundaries, accurate runnable instructions and stage;
   add Rust/dependency checks to shell-only CI, update existing issue evidence.
5. Run relevant checks, publish a focused PR, attach it and report actual outcomes.
   Physical SBC testing is not required; new NetBSD runtime evidence is reported
   only if actually obtained. No implicit merge of the new feature PR.

## Progress

- [x] Read accepted scope and relevant identity/domain/abuse issues; branch created.
- [x] Implement and verify the first runnable local increment.
- [x] Update docs/decisions/dependency evidence/CI.
- [x] Publish/attach PR #48. The linked PR carries the authoritative CI result;
  local evidence is recorded below. No implicit merge.

## Evidence and handoff

- Rust 1.96.0 on ARM64 Darwin: 14 core integration cases + two console unit cases
  passed; release build, rustfmt and strict Clippy passed. Real PTY smoke passed
  with generated disposable credentials/state and no listener.
- Selected source graphs: 29 registry packages for NetBSD ARM64, 31 for Linux
  x86-64. Package/license/feature inventory reviewed; no prohibited source filenames
  or manifests found. Cargo.lock pins checksums; no vulnerability audit claimed.
- Repository checks and all 23 existing Python regression cases passed. CI repeats
  existing evidence/source checks plus Rust 1.96.0 application/dependency/PTYS.
- No new native NetBSD application or physical SBC run. No remote listener, full
  ANSI/themes/chat, private-room ACLs, export/restore, migration, total-storage
  quotas, server admission or 100/250-member capacity claim. These are explicit
  next tasks, not blockers to the tested local increment.
- The local CLI includes filesystem-owner bootstrap/recovery and must never be
  configured as an SSH forced command. Next work needs safe ANSI/theme/session
  surfaces and a distinct restricted adapter, then local chat/integrated NetBSD
  identity/recovery/load evidence under #35. Lua follows the useful local core.
