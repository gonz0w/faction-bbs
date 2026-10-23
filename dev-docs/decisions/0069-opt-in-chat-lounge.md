# 0069: Opt-in chat availability and bodyless page invitations

Status: accepted for this bounded implementation, 2026-10-23
Issue: [#137](https://github.com/gonz0w/faction-bbs/issues/137)

## Decision and scope

Reuse the existing current-approved chat lease/heartbeat/cleanup boundary for
explicit available/busy/away/hidden social availability across local chat channels.
Default disabled by the sysop and hidden for every chat entry. A common inert
command presenter serves local/worker/shared terminals. Presence has no channel
name, membership list, private message body, device key or OS authority.

A page is a fixed invitation from one opted-in available member to another,
answered accept/decline without changing either chat channel or active editor.
Requests/replies expire after 60 seconds. Keep at most eight records per lease,
128 total, and one attempt per account per 30 seconds. Unknown/hidden/blocked/
quiet/unavailable recipients have the same generic attempt outcome; listings
never expose hidden counts or private-channel participation. Sysop paging requires
explicit operator opt-in under the same rules, without a privileged bypass.

Schema 17 stores durable mute, UTC quiet-hour and 16-handle block preferences,
and separate transient presence/invitations/cooldowns. Export drops all live
chat authority and social rate timestamps, preserving only configuration and
personal control intent. Current approval/epoch and both participants' preferences
are checked on action/read, including held requests. Stale clock/lease states
fail closed. No background daemon, added listener/helper/package or new budget.

## Alternatives and tradeoffs

Implicit global presence would reveal visits to private areas. Delivering custom
page bodies would become the server-readable private-mail shortcut the owner
rejected. Forced channel switches could redirect drafts or bypass channel grants.
Durable page history/last-seen logs are unnecessary for a minimal social signal.
Keep those out of this feature. Existing /names remains explicitly channel-scoped
chat presence; social opt-in changes no existing channel membership semantics.

This increment does not advertise users in rooms/doors or notify outside chat;
board-wide lease ownership and safe notifications remain #137 follow-on work.
Pages are explicitly polled with /pages in both live and plain chat. Existing
live channel messages retain their draft-preserving asynchronous presentation.
See plan 0074 for actual software validation; no capacity/hardware/security claim.

## Reversal and retention

Disable the lounge to remove live presence and requests without deleting personal
controls. Closing chat removes its live presence/pages; a stale crashed lease is
invisible after 60 seconds and reclaimed on later write/open. Expired rows may
remain at rest until cleanup; this is not secure erasure from WAL/backups.
Restore requires new live joins/opt-in. Take a stopped complete backup before
schema 17; older binaries cannot open it and rollback uses a pre-upgrade backup.
