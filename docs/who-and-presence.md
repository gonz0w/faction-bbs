# Who, group commands and the Lounge

**Who is here** on the main menu shows visible logged-in members, their broad
current activity and their separate availability/public status. In group chat,
`/who` shows the same board-wide caller list. The **Lounge** is a distinct social
desk with its own name, invitations, answers, privacy controls and page-the-sysop.
It is not renamed or replaced by Who.

| Interface | Purpose |
| --- | --- |
| Main-menu Who | Caller list: handle, Where, availability and public status. |
| Group `/who` | Slash-command entry to the same board-wide caller list. |
| Lounge | Social roster, invitation/answer workflow and personal privacy controls. |

All three use one visibility/availability model. `/available optional status`,
`/busy optional status`, `/away optional status` and `/hide` apply everywhere.
Who accepts those commands as well as `/privacy`, `/mute`, `/quiet`, `/block` and
`/unblock`; use the Lounge for invitations and paging. Hidden members are absent
from every social roster and its pagination/counts. Existing mutual blocks apply
to all three. Busy and away remain visible while not accepting page invitations.
Each new visit starts hidden, and changing your location never opts you in or
changes your selected availability.

The sysop enables the existing social feature with `/social on` and disables it
with `/social off`; this governs all three interfaces. `/social name LABEL` names
the Lounge only. Theme menu action `who` can have its own label, description and
hotkey. The automatic Who action prefers W, selecting a free letter when W is
already used (for example by Community wall). Plain menus also accept the word
`who`. Who uses R to refresh, N for another page of up to eight eligible members,
and Q to return. Classic/plain access works without graphics.

## Where means a broad current screen

Activity is host-derived: Menu, Rooms, Chat, Doors, Files, Lounge, Who, Settings,
Community, Gallery, Library or Directory. Room reading/composing stays **Rooms**,
including unfinished drafts. Private channels still report only **Chat**; games
report only **Doors**. No private room/channel names, subjects, draft text, game
detail, IPs or native encrypted-mail activity is exposed. A member's chosen public
status remains their explicit text, independent of location. Existing channel
`/names` participation is separate from this social visibility control.

Screens are current samples on explicit refresh/action/resize, not a live history
or delivery promise. Activity may briefly lag under database contention. No Who
poll pushes output over a draft, game, transfer or terminal probe. Existing
optional idle page notifications remain restricted to an empty main menu.

## Operator upgrade and recovery

Schema 18 adds one current activity row per existing connection lease, bounded by
the existing 256-lease cap. It has no event history, extra database connection,
helper/thread, listener or new registry dependency. Existing normal eight-session,
5000-ms admission, credential, helper and output budgets remain. Ordinary logout
cascades activity cleanup; abrupt termination uses existing lease expiry/cleanup,
and stale/revoked/hidden activity is excluded from every public read.

Take a stopped complete DB/blob/config/key backup before upgrade. Current owners
migrate schema 17 to 18; older supported snapshots remain read-only verifiable.
Complete export/restore strips all live leases/presence/invitations/activity;
restoring never resumes visibility. No mixed-version owner or in-place downgrade
is supported; rollback uses the matching old build and a complete older backup
into fresh paths. This adds no new hardening or capacity claim.

See [social controls](social-lounge.md), [ADR 0073](../dev-docs/decisions/0073-universal-presence-and-who.md)
and [plan 0078](../dev-docs/plans/0078-who-universal-presence.md).
