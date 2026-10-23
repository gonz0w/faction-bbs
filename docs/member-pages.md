# Member pages and dated .plan updates

An optional local social directory gives members a small page: a bio, interests,
original ASCII signature, current project and dated `.plan` updates. Everything is
scoped to this BBS and its currently approved members. No real name, location,
external identity or additional registration field is required.

A page is application data, **not** an OS home directory, shell file, Finger
service, home-identity proof, private-mail inbox or federation export. Hidden fields
are local visibility controls. All stored text is plaintext readable by the board
operator and included in private owner backups; use device-encrypted private mail
for the separate server-blind communication requirement.

## Sysop setup and presentation

Choose **S:Sysop controls → M:Member pages**. The feature starts disabled. E enables
it, D disables it; Q returns to main. Enabling inserts **M:Member pages** or another
free letter if M is already assigned. A trusted theme may declare
`action = "member_pages"` with its own key, label and description. A theme cannot
enable the feature or grant access.

The settings are:

| Setting | Default | Choices |
| --- | --- | --- |
| Label | Member pages | 1–24 printable ASCII bytes |
| Layout | Card | Card or compact; L toggles |
| Available fields | All four | F: field numbers, all/none |
| Retained .plan updates per member | 8 | H: 1–16 updates |
| .plan retention | 90 days | H: 7–365 days |
| Sysop permanent removal | Off | R: explicit YES/NO |

Field numbers are 1 bio, 2 interests, 3 signature, 4 project. For example,
`1 3` allows bio/signature; `all` or `none` selects all or no fields. Disabled fields are hidden from
other members and cannot be newly authored; existing values remain owner-readable
and may be cleared. Re-enabling a field restores its member-selected visibility.
Members should hide or delete a value themselves for an enduring visibility choice.
H accepts two numbers, such as `8 90`. Reduced quotas/retention prune old updates
lazily during history operations and immediately when applying settings. These
limits bound retained notes, not whole-process/database size or supported load.

The UI inherits the login session's palette. Card layout includes a page heading
and update time; compact layout keeps a short handle/label heading. Plain sessions
emit no ANSI controls. Main-menu arrows/hotkeys remain; directory/editor commands
are letters/numbers followed by Enter. Larger optional main menus retain their
bounded scrolling lightbar and classic fallback.

## Find people

The Member pages list shows up to eight shared local page handles at a time.
N/B page through the first 100; F looks up any exact handle beyond that listing.
V views your own page; E edits it. Q returns to main.

Only deliberately saved, published pages belonging to approved members appear.
Unpublished and sysop-hidden pages do not appear in lists or foreign lookups.
Approval revocation also removes the author from these views; an already-open
page is rechecked at the next action. Reapproval permits the retained page again.
There is no last-seen, online status, ranking or activity-tracking directory.

A page shows one field or dated update at a time. N/B moves between fields and
updates, and Enter shows the next body page when text wraps. Fields and newest-first
updates are fetched under current permission on each screen. Visiting/counting a
page never changes room unread cursors. The owner can view hidden fields/updates;
other members receive only currently shared content and no hidden update count.

## Write your page

In E:Edit my page choose 1–4 to replace an available field:

| Field | Maximum bytes | Maximum lines |
| --- | --- | --- |
| Bio | 240 | 4 |
| Interests | 120 | 2 |
| ASCII signature | 240 | 4 |
| Current project | 120 | 2 |

Each line is at most 72 printable ASCII characters. Controls, tabs, terminal
escapes and non-ASCII input are refused; Unicode discussion-room writing is a
separate capability. Ordinary spaces/newlines preserve tiny original ASCII art.
No member text is trusted graphics or executable content.

A line `.` opens preview; `..` enters a literal period, `/undo` removes the last
completed line, and `/cancel` leaves the field unchanged. Reviewing an empty
replacement clears that field on Save. Preview **S:Save** is the only persistence
step; E returns to editing and C cancels the change. Enter pages through a wrapped
preview. Unsaved text has no checkpoint/autosave and is lost on disconnect.
Shared resize preserves completed text and clears unfinished input, falling back
to classic output when necessary. Nothing is silently posted by resize or preview.

V changes your shared field selection, followed by review/Save. H toggles publication
of the whole page, also followed by review/Save. Saved pages and fields start
shared with approved local members unless you choose otherwise. Hiding the page
also hides its entire update history from other members. Your own view/editor
still lets you manage retained hidden text. Hidden does not mean encrypted.

Concurrent edits use a saved revision: an older editor cannot overwrite a newer
page or undo a sysop hide. A conflict keeps the attempted review available; cancel
and reopen the current page before trying a revised change. Busy errors also keep
the review available for retry. Sysop hide remains in force when an author edits.

## Dated .plan history and removal

After saving a page, P in its editor starts a dated update of up to 480 ASCII bytes,
eight lines and 72 columns per line. Preview is paged; H in preview toggles shared
versus hidden before S saves. Updates get a UTC posting time and stable ID. Text
is immutable: add a newer update or delete the old one instead of rewriting its
original date. Current page publication/moderation still gates shared updates.

X in your page's update view deletes the displayed stable ID, so a concurrent
removal cannot redirect X to another update. You cannot remove someone else's
update. Retention is newest-first, with a per-member count/day policy. Same-key
retry is idempotent while the matching update is retained; there is no permanent
receipt after pruning/deletion. A successful Send must not be retried with a new
key. The terminal workflow keeps its key when a busy database refuses the save.

D in your editor asks for the exact word `DELETE`. It removes your page text and all
its retained updates atomically, with a revision check; another answer cancels.
When a sysop has hidden the page, a minimal empty moderation record remains so
deleting/recreating content cannot undo that hide. Only the sysop can lift it.
Logical deletion/expiry does not erase earlier snapshots, backups or WAL bytes.

## Sysop moderation

S in the Member pages list is sysop-only. Enter `handle hide` or `handle show` to
change the moderation gate while preserving the member's text/authorship. A member
can still manage their hidden page, but editing cannot clear that moderation gate.
No sysop edit-as-author UI is provided.

`handle remove` deletes the page/history only when the sysop explicitly enabled
permanent-removal policy. Removal is not an account ban; an approved author can
create a new page. Use the persistent hide gate or local membership revocation
when continued publication should be refused. No host permissions follow from
this in-BBS moderation role.

## Upgrade and recovery

Schema 10 adds page policy, pages and dated updates to validated owner exports and
[complete file-area bundles](backup-checks.md), including hidden content. Stop/drain
and make a verified complete pre-upgrade backup before opening an old board with
this binary. Older schema-2..9 exports remain read-only verifiable/restorable;
opening restores upgrades them. Rollback restores a pre-upgrade backup to a fresh
path with the earlier binary; never drop new tables in a running board.

See [plan 0061](../dev-docs/plans/0061-member-pages-and-plans.md) for software test
results. Default eight sessions, credential/helper/output limits and private-mail
boundaries remain. No new listener, registry dependency, host operation, physical
SBC, memory/capacity, protocol interoperability or hardened-host claim is made.
