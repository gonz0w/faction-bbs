# 0054: Full-screen room editing and explicit server checkpoints

Status: accepted implementation direction for #141.
Date: 2026-10-22. Follows ADR 0053; owner explicitly requested full-screen editing,
saved drafts and classic fallback after checked #148.

Use one bounded text model for the line desk and ANSI editor. Arrow/cell-aware
caret movement and paragraph editing need no external editor process. Preview and
quote remain inert and do not acknowledge unread content. Full-screen Send asks
for the literal SEND confirmation; ordinary one-line/.send shortcuts remain.
Output uses existing trusted theme controls, <=8-KiB frames and the unchanged
32-KiB shared queue/five-second drain. Keep eight-session/two-credential/three-helper
defaults and no new registry dependency. Preserve partial input decoding across
fitting resizes; transition to line input behind an Enter recovery barrier.

Select explicit **server-side local-room** checkpoints over native-only storage
for this slice: ordinary SSH users need reconnect recovery too. The owner already
selected member-side encryption for private mail; this does not weaken it. Save
announces plaintext/sysop readability. Four rooms/member, one revision/room,
1024 bytes / 32 lines, 30-day recovery and lazy per-member expiry keep work/data
bounded. Current approval and local room/parent context remain authoritative.
Compare native device room-draft storage later without adopting a second format
or remote-controlled local paths/unlocks now.

Schema 8 stores member/room/epoch, authored/default subject, body, parent, save time
and an opaque revision/request ID. Replacement/deletion compare the current revision.
Status/owner recovery clears affected rows. An old editor cannot overwrite/delete
a newer checkpoint; its own deliberate post is still allowed. Matching send cleanup
commits atomically with the post and existing same-key retry, so a lost acknowledgment
does not leave that saved revision as a second send opportunity.

Owner exports/bundles include private room drafts; exact schema/data validation
accepts older exports without modifying them. Stopped complete backup before
upgrade, fresh-path rollback and membership review on restore remain required.
Lazy expiry/deletion/WAL/backups do not provide secure erasure. No autosave,
member export, private-mail draft store or federated draft synchronization.
See [the product contract](../../docs/writing-desk.md).

Validation: unit ownership/revision/quota/expiry/cell/input tests; local/worker,
actual restricted SSH and host/native NetBSD clients; plain, resize, reconnect,
stale reply, approval, busy retry, schema-7 upgrade and export/restore. Preserve
old actual inputs/results before edits and pin new final clean-build sources.
No physical SBC, measured memory delta, new 250 capacity or hardening adoption.

Reversal requires the documented stopped backup/old binaries: schema 8 is not an
in-place downgrade. The optional full-screen surface can be disabled/avoided while
keeping line editing and stored posts/checkpoints available to compatible binaries.
