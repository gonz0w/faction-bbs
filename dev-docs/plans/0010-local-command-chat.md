# 0010: BBS-local command chat

Date: 2026-10-19. Status: merged in [PR #50](https://github.com/gonz0w/faction-bbs/pull/50) on 2026-10-20; continuation is [plan 0011](0011-local-export-restore.md). Issues: #26/#6/#9/#12/#21/#29/#35.
Owner authorized merge #49 and continuation. #49 merged as
d96bf48e0de0a62343184084a3b040bd12d4ea37; main was clean and checks passed.

## Outcome and bounds

Deliver simultaneous local-console chat, not an IRC wire server or federation.
Reuse local approval/session authority and SQLite. Define board/private channels,
sysop create/allow/deny/kick, joined-session presence and bounded since-join history.
Use at most 32 channels, 256 live chat leases, 8 joins/lease, 64 retained messages/
channel, 512-byte messages, and 16-row/2-KiB pages. These are application caps,
not supported host capacity. No per-recipient queued copies; slow readers see an
explicit history-gap signal. Leases expire after 60 seconds without heartbeat;
normal exit removes presence. All channels remain local with no sharing flag.

Replay uses monotonic channel sequences. Per-lease monotonic submission sequence
keeps one bounded retry receipt; exact latest retry reconciles, stale/out-of-order
or changed retries fail. No unsent-draft persistence or post-exit retry promise.
Joining starts a new history floor; regrant/rejoin cannot expose earlier messages.
Add a transactional schema-1 -> schema-2 migration preserving existing identity,
room content/cursors; later backup/export remains separate and explicit.

Commands: /help /list /join /part /names /history /poll /me /quit, ordinary text,
sysop /create /allow /deny /kick. Account handle is the display identity; no /nick,
DMs, channel moderators, general IRC interoperability or private-channel encryption.
Local raw TTY supports live arrivals with bounded ASCII line editing and prompt
redraw; plain fallback uses Enter and /poll. No listener/SSH/HTTP/Lua/financial calls.

## Tests declared before implementation

- Pending/revoked/cross-board/expired leases deny list/join/read/send/names/admin.
  Core rechecks approval/role/epoch/channel grant/join inside action transactions.
- Two independent connections and real simultaneous PTYs exchange messages;
  private channel names/content cannot leak to outsiders or pre-join cursors.
- Revoke/reapprove, deny/regrant and kick remove access; old session/join state
  cannot revive implicitly. Never redirect an unsent private draft to another channel.
- Chronology, latest retry reconciliation/conflict, concurrent sends, bounded
  retention/pages, explicit lag gap, lease/join/channel caps and presence cleanup.
- Schema migration preserves existing posts/unread/auth, repeats safely, rejects
  foreign/future versions, and can migrate with independent openers.
- Live input context: draft retained across arrivals; arrows/control tails never
  become text/commands; CRLF once, overflow discards the whole line, safe output,
  resize/disconnect/normal exit cleanup. Existing ANSI/core tests remain green.
- No new dependency required; source inventory unchanged. Docs/ADR/README and
  existing issues updated with actual evidence and remaining NetBSD/server work.

## Progress

- [x] Merge #49 and read current product/security/roadmap/workflow/issues.
- [x] Core schema/chat actions, command grammar and tests.
- [x] Local chat UI, bounded editor and simultaneous PTYs.
- [x] Current docs/decision/source/checks.
- [x] Publish and attach PR #50; its checks carry the authoritative CI result. Broad issues remain open.

No physical hardware gate. New NetBSD application execution remains pending;
no implicit merge of the next PR. The owner-capable CLI must never become an SSH
forced command; real restricted transport identity/confinement is separate work.

## Evidence and continuation

- Rust 1.96.0 / ARM64 Darwin: 40 tests pass (11 chat, 14 existing core, two CLI,
  13 presentation). Includes a late-DDL migration rollback, concurrent migration/
  sends, name pagination, role changes without epoch change, grant/history-floor
  checks, receipts, retention and lease caps/expiry.
- Existing console PTY and new simultaneous chat PTYs pass. Chat exercises live
  delivery while typing, no private history/draft leakage, allow/deny/regrant/kick,
  revoke, control strings, resize, plain commands and complete presence cleanup.
  A macOS harness cleanup hang was corrected by closing inherited descriptors and
  draining/closing PTY masters before waiting; no transcript/credential is printed.
- Repository checks, 23 Python regressions, rustfmt, strict Clippy and unchanged
  selected dependency inventories pass locally. No new dependency or JS introduced.
- Built-in menu now has eight entries, 736-byte initial render / 137-byte first
  focus redraw. Existing custom themes may omit the optional chat action; base
  example includes it, and seasonal overrides still cannot remap commands.
- All channels are local; no IRC wire server/federation/listener. ASCII live
  editor and explicit plain /poll are documented. No native NetBSD application,
  server capacity, hardware, crash/export/restore or production output-limit claim.
- Next useful work: backup/export/restore and remaining account/room lifecycle,
  distinct restricted SSH adapter and native NetBSD application/PTYS, integrated
  resource/failure evidence. Lua/isolated doors remain after the useful local core.
