# Gatherings and scheduled chat nights

An optional local agenda helps members meet for terminal hangouts, project help,
reading groups and sysop Q&A. Events and responses stay on this board; there is
no external calendar, mail delivery or federation.

## Set up the agenda

Choose **S:Sysop controls → G:Gatherings**. E toggles availability; the default is
disabled. Enabling adds **G:Gatherings** (or another free hotkey) to the main menu.
A trusted theme can declare `action = "gatherings"` and choose its own key/label;
it cannot enable the service or grant membership. Arrows/Enter and main hotkeys
work as usual. Inside the agenda, commands are followed by Enter.

| Setting | Default | Control |
| --- | --- | --- |
| Availability | Disabled | E |
| Name | Gatherings | N; 1–24 printable ASCII bytes |
| Presentation | Cards | L toggles compact |
| Organizers | Sysops only | O allows approved members |
| Approved-login reminder | Off | R |
| Aggregate attendance counts | All approved members | V restricts to organizer/sysops |
| Moderation | Local sysop | M; stable event ID and hide/show/remove |

Plain and narrow terminals retain text commands; the agenda inherits the session's
trusted palette. Disabling hides the action and prevents agenda access, preserving
stored events until re-enabled. Settings are local policy, separate from a theme.

## Browse, create and attend

The list shows four events/page: N/P page, A toggles upcoming/recent past, V opens
an exact stable event ID and Q returns to main. C starts an event if allowed.

Enter a title, **UTC `YYYY-MM-DD HH:MM`**, duration in minutes and optional local
place (`room:1`, `#lobby`, or `-` for none). Enter description lines, then `.` to
preview. `/undo`, `/clear` and `/cancel` work in description input; `..` enters a
literal dot. S explicitly saves, E revisits fields and C abandons the draft.
Preview has no side effects. Unsaved edits remain only in the connection; no
autosave/checkpoint is provided.

Events are one-off: no recurrence, local-time conversion or daylight-saving
adjustment. UTC remains UTC when clocks change. A new start must be in the future,
within 366 days; duration is 1–1440 minutes. An ongoing event can retain its start
while its organizer edits other details. Ended or cancelled events cannot be edited.

The detail card shows title, organizer, UTC range, accessible place, paged notes,
your response and permitted counts. N pages notes; R chooses going/maybe/clear,
then S explicitly saves the response. Cancellation leaves the previous response.
No attendee-name list is exposed. Counts exclude members whose approval was revoked;
those stored rows still count against the event quota until removed/expired.

**Responses are plaintext local board data, readable by the operator and included
in private backups.** Counts-only display is a community visibility rule, not
confidentiality from the host. This notice appears before response save. No message
is sent to rooms, chat or private mail and no room unread cursor is changed by
agenda/RSVP actions.

J deliberately opens the accessible local room history or joins the named local
chat channel. The destination does not grant room/channel access: current approval,
event state and private-channel grants are checked again. Inaccessible private
channel names are omitted from other members' cards. The normal room reader/chat
and their return-to-main behavior apply. No remote endpoint or executable is accepted.

## Changes, cancellation and moderation

Organizers edit their own event with E and cancel with C plus exact `CANCEL`.
Changing its start, duration or place clears all responses in the same transaction;
title/notes-only edits retain them. A reviewed response must match the current event
revision. Concurrent organizer/moderator edits refuse stale saves; reopen the event
to review current data. Busy saves preserve unsaved text for a deliberate retry.

Cancelled events remain visibly cancelled until removed/expired and cannot accept
new responses or jumps. Members may clear their own response while the event remains
accessible. Sysops can hide/show events without rewriting authorship; an author's
edit cannot clear that moderation gate. Permanent removal requires exact `DELETE`
and removes the event and all its response rows. It is not secure erasure from old
backups or storage media. Revoked organizers' events disappear from foreign views;
sysops may inspect/moderate them, but cannot attend/jump through inactive events.

At most 64 events are retained, with eight future events per ordinary organizer.
An event accepts at most 250 stored responses. Titles are <=64 printable ASCII
bytes; notes <=480 bytes/eight lines, each <=72 columns. Ended events older than
30 days are removed lazily on agenda operations. No background scheduler is added.
Creation retries reuse a retained request ID; permanent removal/expiry removes
that retry history. These storage limits are not supported concurrency claims.

## Login reminder and recovery

If enabled, approved login shows the earliest visible, uncancelled event ongoing
or starting within the next 24 hours, with title and explicit UTC time. It does
not reveal private destinations, attendee identities or hidden events. The welcome
pause keeps it readable even in compact/plain mode. It is not a push notification
or a separate catch-up inbox.

Schema **11** includes policy, events and RSVP rows in owner export/verify/restore
and complete catalog/blob bundles. Stop/drain the owner and verify a complete
pre-upgrade backup before first open. Old schema-2..10 exports remain read-only
verifiable/restorable; the new owner migrates restored data on first open. Rollback
uses matching old binaries and a complete pre-upgrade backup in fresh paths;
never relabel `user_version` or unlink live lock files.

See [ADR 0059](../dev-docs/decisions/0059-local-gatherings.md),
[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/136) and
[software validation](../dev-docs/evidence/gatherings-netbsd/README.md).
