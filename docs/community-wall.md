# Community wall

The wall is a local, approved-member hangout: greetings, jokes, shoutouts and tiny
original ASCII contributions. It is separate from discussion-room articles, chat,
private mail and notifications. It never federates or publishes to a news provider.
This is a working hobby-preview feature, not a new capacity or hardened-host claim.

## Enable and personalize

As an approved sysop choose **S:Sysop controls**, then **W:Wall settings**.
The wall starts disabled. Enable it to add **W:Community wall** to the main menu;
a custom theme may assign `action = "wall"` another free key, label and description.
A conflicting W automatically chooses another free letter for the inserted entry.
The wall uses the session's trusted theme palette/section styling, with its own
sysop-defined name of 1–24 printable ASCII characters. Themes do not grant access.
Changes to wall policy apply to already connected members on their next action.
A menu drawn before enable/disable may be refreshed by returning to main.

Settings include:

| Setting | Default | Bounds |
| --- | --- | --- |
| Enabled | No | Sysop enables/disables |
| Name | Community wall | 1–24 printable ASCII bytes |
| Notes per edition | 64 | 1–128 |
| Note bytes, including newlines | 240 | 32–240 |
| Retention | 30 days | 1–365 days |
| Posting/editing cooldown | 30 seconds | 1–3600 seconds per member |
| Weekly rotation | Off | UTC Monday editions |
| Retained previous editions | 0 | 0–12, only in weekly mode |

Use **L:Limits** to enter `notes bytes days cooldown`, for example `64 240 30 30`.
Use **N:Name**, **A:Archive count**, **E:Enable** or **D:Disable** separately.
Changing weekly mode requires typing `YES` and **clears every note and archive**;
a different answer cancels that change. It preserves cooldown receipts.
Disabling hides the wall and refuses its operations without erasing notes.
Lowering size/count/retention limits removes notes outside the new policy.

## Leave a signal

All wall commands are a letter followed by Enter, in both ANSI and plain sessions.
The main menu retains arrows/Enter and immediate hotkeys. A compact optional menu
with more than eleven entries scrolls its lightbar window so every action and the
recovery footer remain reachable at 40×18. Plain menus show every entry.

The wall displays one note at a time in chronological order. **N/Enter** advances,
**B** goes back, **C** returns to the current edition, **A** lists retained weekly
editions and **Q** returns to main. Each note shows its stable ID, author, UTC
posting time and whether it was edited. Paging clamps at the first/last note.

**P** starts an unsaved local note: at most four lines, each at most 72 ASCII
characters, within the configured byte limit. Ordinary spaces preserve tiny art;
terminal escapes, controls, tabs and non-ASCII text are refused. ASCII is the
initial wall format; Unicode room editing remains available separately.
A line containing `.` reviews; `..` contributes a literal period. `/undo` removes
the last completed line; `/cancel` discards this composition. The fourth completed
line enters review automatically. In review **S** publishes, **E** keeps editing,
and **C** cancels. Nothing posts on preview, resize or disconnect. Unsaved wall
text has no checkpoint/recovery. A resized shared terminal keeps completed text,
clears unfinished input and uses classic output when geometry no longer fits.

A busy Send leaves the same body and request key available in review: retry S.
A cooldown error also leaves the reviewed text available. Editing resets that
member's cooldown. It is not an IP/NAT penalty or an SSH admission setting.

## Editing, moderation, archives and deletion

**E** replaces your own current-edition note, retaining author/original posting
time and marking it edited. Use `/undo` to remove old lines or append new ones,
then explicitly review/send. An intervening edit causes a conflict instead of
silently overwriting it. **X** deletes the displayed note: members can delete their
own notes; an approved sysop can remove any note, including an archived note.
Deleting an archived note returns to the current edition.
Sysops do not impersonate authors by editing other members' notes.

Actions address the displayed stable ID, so concurrent deletion cannot make X
remove a different note that moved into its page position. Current board identity,
member approval/epoch and role are checked again for every action and screen.
Pending/revoked members cannot enter, recover or send wall text.

Weekly editions advance lazily on the next wall operation, using UTC Mondays.
Moving the system clock backwards does not reopen an earlier edition. Archives
are kept **only** when the sysop explicitly selects a nonzero retained-edition
count; the independent retention-day limit still applies. Archives are read-only
for composition/editing. Deletion remains available. If an edition expires while
viewed, the UI returns to current. A review cannot silently post into a different
edition after rotation. Nonweekly mode has no archived editions.

Oldest notes beyond the per-edition limit are pruned. Each member retains only one
small cooldown/recent-request receipt without deleted message text. A retained,
unchanged post accepts a same-key retry; a deleted latest request cannot recreate
its note. This is a bounded retry window, not an eternal idempotency ledger.
Notes, their receipts and archives stay in this sysop's database and owner backups;
they are plaintext local community data. There is no notification, room read-marker,
mail, account-profile, member export or secure-erasure behavior. WAL, earlier
snapshots and backups can retain logically deleted/expired text.

## Upgrade and recovery

Schema 9 adds wall settings, notes and receipts. Stop/drain the service and make a
verified [complete backup](backup-checks.md) before opening an older board with the
new binary. No in-place downgrade. Restore the pre-upgrade bundle to a fresh path
with the previous binary for rollback. Owner exports/bundles include wall state;
schema-2..8 exports remain read-only verifiable/restorable and upgrade on open.
Theme packs are still separate operator-managed files.

Software tests cover the native NetBSD implementation, real restricted SSH and
native terminal clients, ANSI/plain/compact/worker input, rotation/archives,
authority changes, retry, moderation and backup. See the
[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/134) for current results.
No new listener/helper/package from a registry, session/default/credential limit,
private-mail decryption, physical SBC or 250-member support claim is introduced.
