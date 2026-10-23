# Personal catch-up

The optional **U:Catch up** desk brings a returning member back to their local
board: the sysop's notice, unread rooms and direct replies, favorite rooms, eligible
new files and upcoming or in-progress gatherings. It preserves the normal login,
identity art, welcome and main menu sequence. It adds no public feed or service.
Enable it through **S:Sysop controls → U:Catch-up settings**. It starts disabled.
Trusted theme packs can place `action = "catchup"` with their own hotkey/label;
otherwise the host inserts U (or a free key). The menu remains unavailable when
the host feature is disabled, including when a theme declares it.

## Find your way back

Commands are followed by Enter in both ANSI and plain terminals:

| Command | Action |
| --- | --- |
| M / M rooms | Next enabled section / select an enabled section by name. |
| N / P | Next/previous bounded page in the current section. |
| R | Refresh the file window and current section from live authorized data. |
| J | Enter a displayed stable room/event ID or opaque file object ID. |
| S | Manage your own room subscriptions. |
| C | Review an explicit saved-visit checkpoint; S saves, C cancels. |
| Q | Main menu. |

Sections are `notice`, `replies`, `rooms`, `files`, `gatherings`. Sysops choose
order or omit modules, name the desk, select cards/compact rows and explicitly
preview/save a local notice (240 printable ASCII bytes, four <=72-column lines).
A single `-` in the section-order setting disables every module. Colors/section
clears inherit the current trusted theme; plain terminals emit no ANSI.

Subscriptions accept `+ID`, `-ID`, `all`, `fav`, `none`: adding a room selects
favorites-only mode; `all` keeps the saved favorites but includes every room;
`fav` uses them again; `none` clears them and selects the empty favorites view.
At most sixteen favorites are saved per member. IDs come from Rooms or the
catch-up list. Subscriptions filter both room and reply sections. Other members
cannot change them. Source jumps are deliberate and recheck current approval,
feature policy and source access; no cached preview grants authority.

## Previews and saved visits

**Viewing, counting, refreshing or saving a visit never marks a room post read.**
Existing room unread markers remain the sole read ledger. A room jump opens the
existing new-message reader: only a complete message, with the shared service's
successful output-drain acknowledgment, advances it. Partial reads remain unread.

There is no automatic last-login or last-seen record. Before the first explicit
save, the desk says **Last saved visit: not set**. C → S records your UTC saved
visit and the file-publication cutoff captured on opening/refresh (subscription changes also refresh the window). Publications
that arrive after that captured cutoff remain new next time. Subscription saves
persist preferences but do not invent a saved-visit time. Concurrent sessions use
revision checks; a stale save refreshes instead of overwriting another session.
Busy writes retain the review for explicit retry. Preferences/checkpoints are
ordinary sysop-readable local data included in private owner backups, not mail.

The file section lists currently eligible published catalog entries since that
checkpoint, not download receipts. Files become eligible in publication order,
including uploads approved after the window opened. Existing files backfilled
on upgrade show **legacy publication date unknown**. A grant to an older private
file after a checkpoint does not make it newly published; use the full Files
catalog for older material. Access removal, hiding/removal or a disabled feature
is checked again on every frame/jump. File publication sequence/high-water counts
are not displayed. Shared hosts with file capability open the existing catalog
information/reference UI; an explicit transfer still uses the existing restricted
SFTP/native Files workflow. Local/worker sessions report Files unavailable.

## Bounded work and deliberate limits

Room/reply pages return four rooms with one lookahead and inspect at most 101 next
unread posts per candidate room. Unread totals cap at **100+**; direct replies mean
non-self replies to your own posts **within the next 100 unread posts per room**.
Zero-reply rooms remain visible; this is not an exhaustive historical reply search.
No second reply ledger or per-session message history is stored.

File pages show two authorized objects plus one lookahead; gatherings show four
plus one. Gatherings use stable ID order, hide cancelled/hidden/ended events and
revoked organizers, and show only title/UTC time before opening the existing
agenda detail. Private destinations and attendee names are not a catch-up badge.
Page history holds at most 32 cursors. Frames remain <=8 KiB with existing queue/
drain/session/credential limits; there is no new helper, scheduler or registry
package. This batch makes no new 250-member capacity claim.

Private mail has **no server-side section, decryption or unread count**. The
[device-side mail plan](private-mail.md) and #99 retain that boundary. These are
local modules, not cross-board ranking, federation, telemetry or a network inbox.

## Upgrade, recovery and validation

Schema 13 adds catch-up policy, own preferences/favorites, explicit checkpoints and
file publication cursors. Stop/drain, preserve and verify a complete DB/blob/config/
key backup before the first new-owner open. Exact schema-2..12 exports remain
read-only verifiable/restorable; opening restored older data migrates it to 13.
Roll back with matching older binaries and their complete backup in fresh paths,
never by relabeling the schema or deleting live locks. Logical deletion does not
erase old snapshots/WAL/storage. See [export/restore](export-restore.md).

[Plan 0066](../dev-docs/plans/0066-personal-catchup.md),
[ADR 0061](../dev-docs/decisions/0061-personal-catchup.md) and
[NetBSD software evidence](../dev-docs/evidence/catchup-netbsd/README.md) document
the implementation and validation. The saved owner playground stays off.
