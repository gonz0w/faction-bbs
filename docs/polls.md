# Community polls

An optional local voting desk lets approved members answer a weekly question,
choose a gathering theme or give board feedback. Each poll has one question and
2–8 choices; each member has one single-choice ballot. Multiple selection and
grouped questionnaires remain follow-on work. No external service or federation.

## Sysop setup and presentation

Choose **S:Sysop controls → P:Polls**. Polls start disabled. E toggles availability,
N changes the 1–24 ASCII-byte label, L toggles card/compact listings and O allows
approved-member creators instead of sysops only. M moderates an exact poll ID;
Q returns to main. Disabling hides the action and prevents access, retaining data.
Turning off member creation also stops member edits; own close/cancel still works.

Enabling inserts **V:Polls**, or another free letter. A trusted theme may declare
`action = "polls"` and choose a free key, label and description. The host policy
still enables it; a theme cannot grant access. Main arrows/Enter and direct hotkeys
remain. Inside the voting desk, commands use Enter. Plain/narrow terminals use the
same ASCII commands and text-cell result bars; no graphics protocol is required.

## Make and browse polls

The list shows four polls/page: N/P page, A toggles open/finished and V opens an
exact stable ID. C creates if allowed; Q returns to main.

Enter a question, then choices one per line. `.` finishes choices; `/clear` and
`/undo` help replace them when editing. Choose result visibility: A always, V after
your own vote, or C after close. Enter notes, then `.` previews. `/clear`, `/undo`
and `/cancel` work while entering notes; `..` enters a literal dot. `/cancel` at
any form field abandons it. Blank question/result input keeps the current value.

Review the question and choice pages with N/P; D previews notes with its own paging.
S explicitly saves, E revisits fields and C abandons. Preview/counting does not
write ballots or change room unread cursors. Unsaved forms are connection-local,
with no checkpoint or autosave. A Busy save keeps its form for deliberate retry.
A retained create request ID prevents duplicate same-content creation; removal
ends that retained retry history. Current creation policy still applies.

Questions are <=96 printable ASCII bytes, each choice <=48, with 2–8 distinct
choices (ignoring surrounding spaces/case). Notes are <=240 bytes/four lines, each
<=72 columns. There are at most 32 retained polls, four open polls per ordinary
creator and 250 retained ballot rows/poll. Closed/cancelled polls retain data until
sysop removal; there is no automatic closing, expiry or background scheduler.

## Vote and read results

The card shows the creator, open/closed/cancelled state, your own ballot and a
page of choices. N/P changes choice pages; D opens notes. V enters a choice number
(or zero to withdraw), followed by a review and explicit S save or C cancel.
Only open, visible polls with an approved creator accept changes. Repeated votes
update the same row; changing a vote replaces it and withdrawing removes it.

**Ballots are local plaintext readable by the operator and included in private
backups. This is not a secret ballot or an election system.** The confirmation
states that scope. Member screens expose no voter-name list, only the own choice
and permitted aggregate counts. Device-encrypted private mail remains separate.

| Result policy | Counts and bars |
| --- | --- |
| Always | Visible while open and after close |
| After your vote | Visible only while your own ballot is retained |
| After close | Visible after explicit close, including to nonvoters |

The display policy applies to creators and in-BBS sysops too; the owner can still
read the database. Hidden/cancelled/inactive-creator polls show no totals. Counts
exclude currently unapproved voters, while their retained rows still consume quota.
The bars use `#` cells and counts such as `3/5`; totals are currently approved
ballots, not all registered accounts or proof of human uniqueness. Rounding a bar
does not change the numeric count. There is no cross-board ranking or publication.

## Edit, finish and moderate

Only the creator edits or finishes a poll. E edits wording/choices/result policy
before **any** ballot has ever been cast. The first ballot permanently locks these
fields, even if all ballots are withdrawn later; create a new poll for a revised
question. F plus exact `CLOSE` closes it; C plus exact `CANCEL` cancels it. Both
are final and prevent further ballot changes, including withdrawal. Cancellation
suppresses results. Polls cannot be reopened.

Revisions protect edits, moderation and reviewed ballots. If another connection
changes the poll or closes it before Save, reopen and review current data. A ballot
cast before an edit transaction locks wording instead; neither race silently moves
a ballot to new wording. Concurrent vote changes by the same member serialize and
the last confirmed change wins. Voting itself does not change the wording revision.

Sysops hide/show/remove by stable ID without rewriting another creator's text.
An author's edit cannot clear the moderation gate. Exact `DELETE` removes the poll,
choices and ballots; another answer cancels removal. Hidden/revoked creators vanish
from foreign views, and authorization/visibility is checked on every frame/action.
Deletion is not secure erasure from media, old backups or operator copies.

## Upgrade and recovery

Schema **12** includes poll policy, choices, ballots, states and the permanent
wording lock in owner export/verify/restore and complete catalog/blob bundles.
Stop/drain and verify a complete pre-upgrade backup before first new-owner open.
Schema-2..11 exports remain exactly read-only verifiable/restorable; the new owner
migrates restored data on open. Rollback uses matching old binaries and a complete
pre-upgrade backup in fresh paths. Never relabel the schema or unlink live locks.

See [ADR 0060](../dev-docs/decisions/0060-local-community-polls.md),
[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/139) and
[software validation](../dev-docs/evidence/polls-netbsd/README.md).
