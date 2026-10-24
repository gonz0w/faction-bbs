# 0061: Explicit personal catch-up

Status: accepted for the hobby preview, 2026-10-23. Issue: #138.

## Context and decision

Returning members need one useful local view without a second read ledger or
mandatory visit tracking. Default-disable a configurable menu desk with notice,
reply/room previews, own favorites, eligible published files and gatherings.
Preserve login/art/welcome order and use authorized jumps to the existing readers,
file catalog and agenda. Never server-decrypt/count member private mail.

Schema 13 stores policy, explicit own subscriptions (sixteen favorites), revision-
checked file cutoff/UTC save time and ordered publication rows. Opening/refreshing
is read-only and creates no member activity row. Only C → S saves a visit;
subscription writes preserve the old cutoff/time. Capture an upper file watermark
for the visit: publishing while browsing cannot be skipped by its later save.
Use AUTOINCREMENT publication IDs with exact owned publication triggers, not upload
row order or timestamps. Old published files backfill with unknown dates; deleting
the highest publication does not lower the watermark. Never display watermark/
publication IDs as private-area activity hints; file source keys are opaque IDs.

Use existing unread markers and bounded queries: four rooms plus lookahead, 101
next unread posts/room, capped 100+ unread and direct non-self replies in the next
100 unread. Favorites filter both. Two eligible files plus lookahead; four events
plus lookahead in ID order. At most 32 back cursors, no materialized feed/history.
Current core approval/role/grants/source/module visibility applies per frame and
jump. A preview never acknowledges read or download. Existing full-message/drained
output acknowledgment is the only shared room progress path.

A common faction-catchup workflow owns bounded presentation and explicit actions;
local/worker/service adapters own I/O. Core/terminal dependencies only, no new
registry package/helper/listener or default-budget change. Sysops choose ASCII
name/notice, card/compact layout and a unique ordered subset of local modules.
Host capability controls the menu even if a trusted theme declares an action.
Local/worker sessions have no file capability; shared catalog routes preserve
existing restricted byte-transfer boundaries. No implicit transfer is launched.

## Alternatives and consequences

Automatic login checkpoints hide activity changes and create mandatory tracking;
explicit saving makes intent and retention visible. A complete cross-room reply
search or materialized feed costs another unbounded scan/ledger; capped previews
state their scope. Newly granting an older private file does not change its
publication order; the full catalog stays available. Stable event ID order avoids
unstable paging while organisers change times. No ranking, subscriptions to foreign
content, mail badge, new message transport or automatic notification is implied.

Preferences are owner-readable local data in exports. Upgrades require a stopped/
drained verified complete backup. Exact schema-2..12 exports stay read-only valid;
matching older binaries and a complete older backup in fresh paths provide rollback.
Snapshot comparisons must include the entire bounded schema: the new tables exposed
a 65-row truncation that could hide extra views. Read 129 with an explicit 128-object
ceiling before exact comparison; negative snapshot controls reject extras.

## Validation and reversal

See [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/138) and
[software evidence](../evidence/catchup-netbsd/README.md). Snapshot 33 preserves 238
preceding inputs and the whole poll result before edits; historical identities
stay fixed. Validate current sources separately on an owned software VM and keep
the saved playground off. Disable the desk or restore a complete old backup with
matching binaries in fresh paths. Hardening, physical hardware and new capacity
claims remain outside this batch.
