# 0070: Connection-scoped shared availability and safe lounge notifications

Status: accepted for this implementation, 2026-10-23
Issue: [#137](https://github.com/gonz0w/faction-bbs/issues/137)

## Decision

The primary shared SSH/native service owns one optional opaque lease for a BBS
connection, allocated when chat or the enabled Lounge first opens. It can exist
without a channel join. Heartbeat it in the existing bounded event loop across
menu/room/editor/door states, with no TTY output or new database connection,
helper, thread, registry dependency or schema. Explicit availability stays local
and opt-in; fresh connections start hidden.

Chat exit parts all channels, clears retry/context and its editor, retaining only
the connection's explicit social advertisement. Re-entry explicitly joins with a
new history floor and request sequence. Transport loss/logout/service shutdown
closes the lease and cascades live presence/signals. Existing current account
approval/epoch/clock/quiet/block checks remain authoritative.

The shared-only runtime-gated Lounge menu entry opens a common bounded panel.
It re-fetches current advertisement, eligible roster, own signals and privacy
controls on every redraw, including resize. Never retain eligible names/page
text in a generic cached notice. A generic star marks the Lounge entry during a
normal main-menu redraw if there is a current incoming pending request or an
explicit reply to the member's outgoing request.

## Alternatives and limits

One extra social lease/database per member would double live rows or add
descriptor/cache costs; reuse avoids that. A new background notifier could add
threads/queues and interrupt games, drafts or probes. No unsolicited screen output
is introduced. The badge appears at safe menu return/refresh; Question mark explicitly refreshes a
waiting menu. Expired requests are not resurrected as retained notifications.
Accepting is still a bodyless signal and never forces a join/screen switch.

The legacy standalone local console/worker keeps its prior chat-only lease scope.
Its menus do not claim a shared-session Lounge capability. Adapter parity and
idle-menu notification UX remain open #137 scope; plain/classic members using the
primary shared service get the full outside-chat panel and visit lifetime.
No OS-console access, custom private-message body, public last-seen tracking,
federation or server-side encrypted-mail fallback.

## Evidence and reversal

See plan 0075 and the current board-wide software evidence. Schema stays 17;
existing complete exports still remove all live leases/social state. Disable the
social feature to remove advertisements/signals without changing durable personal
controls. Rollback to the earlier shared build shortens availability to chat again.
Normal eight sessions, five-second admission, two credential permits, three
helpers, 32-KiB output/five-second drain and 8-KiB frames remain. No new capacity,
physical-hardware or hardening claim; preserve all prior result/source identities.
