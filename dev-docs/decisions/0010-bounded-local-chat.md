# ADR 0010: Bounded local command chat

Status: accepted for the local development increment. Date: 2026-10-19.
Issues: #26/#6/#9/#12/#21/#29/#35. Owner authorized merge #49 and continued building.
Question: how can two local sessions use IRC-style channels now, preserving
local authority/privacy, tiny bounded work and future transport independence?

## Constraints and alternatives

The core must remain useful without federation, HTTP, external services or JS.
Registration/transport admission never grants local approval. Avoid per-recipient
unbounded queues, accidental private-history disclosure, stale approval/grants,
and menus interpreting chat text. No physical hardware required.

| Decision | Alternatives | Choice and cost |
| --- | --- | --- |
| Local delivery | Process-local broadcast; external IRC service; shared bounded store | Reuse existing SQLite for cross-console channel history. No new daemon/dependency or per-recipient copies; serialized writes and local polling are explicit costs. This is not a production server event-loop choice. |
| History | Full archive; no replay; bounded replay | Latest 64 messages/channel, eligible only after a join floor. Rejoin/regrant starts fresh; slow-reader gaps are explicit. Idle messages can remain at rest; retention is not secure erasure. |
| Membership/presence | Persistent member join; transport address; scoped lease | Opaque board/member/epoch-bound random lease, 60-second expiry, 10-second console heartbeat. Normal exit removes joins. Clock rollback beyond five seconds fails closed; future server clocks/timers still need native/integrated evidence. |
| Authority | Cache grants; sysop-only channels; current board/private scopes | Every read/send/names/join rechecks approval/epoch, role, channel grant and current join inside the transaction. Sysops manage grants/kicks; no channel moderators/ban modes yet. Grant changes remove joins, never silently revive them. |
| Retry | Blind insert; unbounded receipt ledger; one latest receipt | Per-lease monotonic request number plus latest body/kind/channel/join floor and original message sequence. Exact latest retry reconciles; older/changed/skipped requests fail. Receipt storage stays bounded; post-exit recovery is not promised. |
| Presentation | Cooked async output over drafts; full terminal editor; bounded local editor | ANSI local chat redraws a bounded ASCII tail viewport and preserves drafts across arrivals. Plain fallback polls on command while heartbeating. Unicode interactive editing, network framing/admission and rich editing remain separate. |

No existing IRC-client protocol is implemented. Slash vocabulary is inspired by
IRC conventions; interoperability remains an explicit open product question in
#26. Account handles remain display identity. No federation flag or external
connection exists; every channel is local, including private channels.

## Implementation and failure boundaries

`faction-core::chat` owns channels/grants, leases/joins, bounded messages, command
grammar and retry reconciliation. It reuses the accepted SQLite/credential core.
`faction-terminal::LineEditor` owns bounded draft/input context, not permissions.
The local CLI owns TTY/polling and maps commands to checked actions; it retains
filesystem-owner recovery authority and must never become an SSH forced command.

Schema 1 -> 2 uses a single IMMEDIATE migration transaction, preserving existing
board/account/room/unread data. Foreign/future schemas fail. Concurrent openers
recheck the version under the lock. There is no downgrade/general migration or
backup/export promise; stop older consoles and take a full private snapshot first.

Caps: 32 channels, 256 live leases, 8 joins/lease, 64 messages/channel, 512-byte
bodies, 16-row/2-KiB domain pages, 32-name cursor pages and one receipt/lease.
They bound named structures, not total DB/host memory or supported member count.
Grants/accounts, SQLite free pages/WAL and the complete app/SSH working set still
need quotas, retention operations and measurement. No application body logging or
telemetry is added; message/receipt bodies are explicit plaintext local data.

Reads check current authority in a SQLite snapshot; writes use IMMEDIATE locking.
Already-authorized output may finish after a revoke commits. No DB transaction
remains open while writing TTY output. Private grant changes/kicks remove every
current target join; the next join establishes a new history floor. The receipt's
join floor also must match, preventing an old retry from reopening earlier scope.
The console clears selected channel/draft on denied actions, never redirects it.

Local live polling is ~250 ms, lease heartbeat ~10 seconds, TTY input readiness has
a 100-ms sleeping deadline. Pure reads do not refresh leases/write the DB. Future
network adapters need event notification, output bounds/timeouts, login admission,
rate limits and whole-process accounting before capacity/security claims. A slow
reader receives a gap after history advances; no private data leaves the board.

The ASCII editor caps a line at 768 bytes, discards whole overflowed lines, consumes
CSI/SS3/control strings using the existing bounded decoder, and never dispatches
menu hotkeys from chat text. Incoming messages redraw the draft viewport; terminal
resize outside supported geometry closes chat without submitting the draft.
Normal exit/error restores termios/cursor and closes the lease. Fatal termination
relies on lease expiry and may require local terminal recovery.

## Primary evidence and sources

Retrieved 2026-10-19:

- [RFC 2812](https://www.rfc-editor.org/rfc/rfc2812.html): channel/join/part/list/names vocabulary and the distinction between client protocol and this console's slash commands. No RFC compliance or external client support inferred.
- [SQLite isolation](https://www.sqlite.org/isolation.html) and [transaction behavior](https://www.sqlite.org/lang_transaction.html): read snapshots, serialized writers and IMMEDIATE lock behavior underpin action/revoke ordering and migration rollback. Prior native bundled SQLite evidence remains [#23](../research/0023-netbsd-dependencies.md); these new application paths have not run in that guest.
- Existing [menu decision](0009-safe-terminal-menus-and-themes.md), [local-core authority](0008-local-core-and-credentials.md) and [first slice](https://github.com/gonz0w/faction-bbs/issues/35): accepted owner inputs and boundaries, not proof of new behavior.

No dependency added; selected 39-package NetBSD ARM64 / 41-package Linux x86-64
source/license/feature inventories remain unchanged. No new native-code, JS, IRC
library, async runtime or outbound adapter is introduced.

## Results, confidence, next falsification and reversal

On ARM64 Darwin/Rust 1.96.0: 11 chat regressions passed, including independent
connections, concurrent sends, approval/role/epoch/grant changes, hidden private
channels, join floors, retry conflicts, retained gaps/caps, presence pagination,
expiry/rollback and atomic schema migration with a late DDL conflict/concurrent
openers. Existing 14 core/two CLI/13 presentation tests pass (40 total).
Simultaneous PTYs verify live arrivals while typing, preserved drafts, emotes,
private allow/deny/regrant/kick, no cross-channel draft publication, live account
revoke, control tails, resize/plain fallback and presence cleanup. The harness now
closes inherited descriptors and drains/closes PTYs before waits to avoid a Darwin
test-cleanup deadlock. This is harness evidence, not a production process supervisor.

Confidence is high for the tested local subset. Native NetBSD application, full
server/SSH resource use, crash/backup/restore, rich Unicode editing, rate limiting,
network slow-reader enforcement and IRC/federation interoperability remain open.
Linux CI repeats the suite on the PR; physical SBC work is not required now.

Any unapproved/private-channel action succeeding, old join/receipt scope reviving,
duplicate accepted request, hidden history/draft leaking, unbounded retained row
growth, silent lag loss, lost committed room data during migration, or failed
normal terminal/presence cleanup falsifies this slice. Next work should test real
restricted SSH identities and native NetBSD PTYs alongside export/restore and
application failure/resource accounting. Generic Lua/doors still follow useful core.

Reversal cost: transport/UI adapters are replaceable without changing approval or
room semantics. Chat schema is development-only version 2; replacing delivery/
retention needs an explicit migration. Migrated stores cannot be reopened by old
schema-1 binaries; rollback uses the pre-upgrade full snapshot. Future shared
channels require a separate explicit model, never turning local/private into a
mandatory federation service or exporting accounts/content by implication.
