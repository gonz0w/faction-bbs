# Shared connection lounge software evidence

[Results](results.json) pin the primary shared SSH/native service on a marked
NetBSD 11.0 ARM64 software VM (two cores/1 GiB). Normal eight-session/5000-ms
admission and original credential/helper/output budgets remain unchanged.

The new journey exercises explicit availability and bodyless invitations outside
chat, one retained connection lease with no channel joins after chat exit, safe
menu badge refresh, composer and Lua game input preservation, current privacy
controls/resize, opted-in sysop paging, escape-free plain access, disconnect,
revocation and sysop disable. Three measured waits of at least 66 seconds cover
an idle main menu, room composer and Lua game input; each checks a current silent
heartbeat. Cooldown/quiet controls separately use explicit synthetic clock aging.
No page interrupts a draft/game or forces a channel join.

The existing local/worker chat-only journeys and complete backup/restore also
pass. Legacy standalone availability outside chat and optional idle notifications
remain open #137 work. Notifications in this batch appear during safe menu
return/refresh or an explicit Lounge visit. Schema remains 17.

Final selected inputs match the clean native build/runtime; installed Rust and
optional C/Lua door binaries match the tested builds. The door uses NetBSD's base
Lua library with no new package. Snapshot 42 and [its manifest](../social-baseline-manifest.json)
preserve 316 previous input files and the entire unchanged social record. All 55
historical result identities remain fixed.

The owned VM, private test credentials and temporary vendor are removed after
validation. The saved owner playground stays stopped with disk/login/keys intact.
No physical SBC, 250-user capacity, GUI graphics, private-message body, federation,
public endpoint, secure-erasure or new hardening claim. See
[plan 0075](../../plans/0075-boardwide-social.md) and
[ADR 0070](../../decisions/0070-connection-scoped-social.md).
