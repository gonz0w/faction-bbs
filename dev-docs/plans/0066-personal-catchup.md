# 0066: Personal catch-up desk

Issue: [#138](https://github.com/gonz0w/faction-bbs/issues/138)
Status: implemented and validated; ready for PR review. Owner: merge #163 and continue useful product work.

## Goal and scope

Optional U desk with room unread/direct-reply previews, member-controlled favorites,
new eligible published files, upcoming gatherings and a sysop notice; sysop name,
layout and module order/enabling. Explicit authorized stable-ID jumps reuse current
readers/file UI/gathering detail. Preserve plain/classic terminals and login art order.
No private-mail module/decryption/badge, external feed, ranking or mandatory tracking.

Visit baselines are explicitly saved checkpoints, not automatic last-login tracking.
The opening file publication upper bound is retained until refresh/save; saving
advances only its own local file cursor/time, never post unread cursors. Files
published later remain new next time. Existing unread is the sole post read ledger.
Room/reply pages scan five rooms and at most 101 unread posts/room, show capped
counts and replies only in the next 100 unread posts. Files have bounded catalog
pagination; preview does not mark files downloaded. At most sixteen favorites.

## Progress

- [x] Merge checked #163, read #138 and preserve prior executed source/result bytes.
- [x] Add schema-13 policy/preferences/publication cursor and bounded authorized queries.
- [x] Add common local/worker/service UI, menus and existing authorized jump routes.
- [x] Validate unread/privacy/checkpoints/ACL changes/old schemas and actual NetBSD UI.
- [x] Update README/docs/ADR/AGENTS/changelog and open a checked focused PR.

## Gates

Snapshot 33 preserves the preceding poll inputs and whole result; all 46 saved
guards stay valid. No new registry package/helper/listener or default-limit change.
Keep normal eight sessions/5000-ms admission, two credential permits, three helpers,
8-KiB frames, 32-KiB queue/five-second drain. Use only an owned disposable software
VM; saved owner playground stays off. Hardening stays later. Schema 13 requires
stopped/drained complete pre-upgrade backup and matching-binary fresh-path rollback.
Old schema-2..12 exports remain read-only verifiable/restorable.


## Validation and findings

280 Rust tests per OS /129 Python tests pass. Eight local/worker catch-up PTYs per
OS and eleven actual restricted SSH/host-native/NetBSD-native/plain controls pass.
Sixteen complete-recovery controls per OS include the actual native CLI with
distinct non-root owner/file UIDs. Current native sources match 240 selected inputs;
21 host harness inputs and fifteen unchanged access/credential inputs are pinned.
Installed faction/service/sysop hashes match the clean build and new save literal.
Existing social/editor/menu/art/room-reader/export/vault/picker regressions pass.

The schema-growth negative control found that the previous 65-row schema comparison
could silently omit an extra view/trigger. Read one beyond a fixed 128-object limit,
reject oversized catalogs and then compare the complete schema exactly. Current
negative controls and old-schema recovery pass. The shared harness follows the
existing main-menu plain recovery after resize; catch-up resize retains its own
completed state. Distinct-UID recovery invokes installed traversable binaries,
not a builder-private home path. These fixture corrections do not change product
startup/permission policies.

Publication baselines use approval/publication order, not upload row order: a pending
upload approved after opening stays new after saving that earlier window. Deleting
the maximum publication cannot lower its watermark. Previews/checkpoints leave
room unread unchanged; only a deliberately opened completed/drained source read
advances it. Private-file grant removal between preview and jump returns no stale
metadata. There is no fresh SFTP byte-transfer or mail/network claim.

The 46 historical saved guards remain fixed; the new result adds guard 47. Snapshot
33 preserves 238 preceding inputs and the whole poll record. The owned VM, private
test credentials and temporary vendor are removed; saved owner playground remains
off with disk/login/keys preserved. Hardening and new capacity work stay later.
