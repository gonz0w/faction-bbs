# 0052: Named room browsing with selected-room actions

Date: 2026-10-22. Status: selected; software validated. #21/#12/#25.

## Decision and alternatives

Use the main R action to open a shared themed ANSI browser/card presenter. Keep
plain R and main N/H/P numeric workflows intact. Four rooms per page, arrow/Tab/
hotkey selection, unread counts and next-unread navigation make ordinary reading
and posting useful without a separate terminal client or numeric-ID memorization.

Selecting a card then New/History/Post reuses the existing reader/composer with a
room parameter. It avoids a duplicate codec/unread/retry path or a stored current-
room preference/schema. Finished operations use the existing main-menu pause.
Persistent in-room navigation, search, favorites and room administration are
separate increments; do not expand this into new storage/federation behavior.

## Authority and limits

Fetch the existing first 100 current rooms on each command; keep only a selected
ID and card flag per connection, not row/message history. Reconcile IDs against
fresh rows before acting. Board approval/epoch, room existence and per-viewer
file permissions remain authoritative. Names/control sequences stay inert with
existing classic/native rendering. New/History use unchanged complete-message and
shared output-drain acknowledgement; selection alone never marks read.

Current schema/dependencies, eight-session/default admission, two credential
permits, three owner helpers/four LWPs and 32-KiB/five-second output limits stay.
A new frame caps at 8 KiB with four entries fitting 40x18. Native capability gates
and themed session selection remain unchanged; no room publication or IRC/XMPP.

## Validation and reversal

[Plan 0057](../plans/0057-room-browser.md) tests parser-free presentation, stable ID
selection, Unicode/classic clipping, paging/next-unread, selected composing/reading,
partial unread, pending/revoked denial and resize on host/NetBSD PTYs plus actual
shared SSH/native routes. Existing reader/menu/art regressions retain their scope;
old room-list fixture bytes/results are preserved before expectations change.

Revert UI routing or use plain/direct numeric commands; no migration or stored
current-room state requires rollback. No new capacity/physical/hardening claim.

[Actual NetBSD results](../evidence/room-browser-netbsd/README.md) pass the scoped
controls after an explicit clean native rebuild; current core/default boundaries
remain unchanged. Persistent room context/search/moderation remain follow-on.
