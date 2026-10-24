# 0056: Local member-owned pages and .plan history

Status: accepted for the hobby preview. Issue #135, continuing #133 after the
owner-authorized checked #150 merge. 2026-10-22.

## Decision

Use a separate optional local social domain rather than OS home-directory files,
a public Finger endpoint, room articles or home-identity claims. Four optional
ASCII fields and explicitly shared/hidden dated `.plan` updates create a place
for people/projects without collecting mandatory real-world identity.

Core transactions own board/member/epoch/role checks, whole-page publication,
field masks, current-approved-author visibility, immutable update dates, own
revision-checked edits/delete and sysop hide/removal policy. A sysop cannot edit
as another author. Hide changes the page revision and persists across author
saves. Own content deletion clears every field/update but retains a minimal empty
moderation record, preventing delete/recreate from clearing the sysop gate. Foreign views/lists/history omit unpublished, moderated and revoked-author
pages and all hidden update bodies/counts. Own views retain management access.

Schema 10 stores one policy, one page/member and 1–16 retained <=480-byte/eight-line
updates per member. Four fields total <=720 bytes; all lines <=72 printable ASCII
columns. Maximum authored payload is 8,400 bytes/member with sixteen updates,
excluding SQLite/index/catalog/metadata overhead. No total memory/capacity claim.
Day retention is 7–365, lazy on history operations and applied on policy changes.
Dated text stays immutable; own deletion targets the displayed stable ID. Retained
same-key update retry is idempotent, without a forever ledger after pruning.

A new faction-pages workspace crate holds only bounded interaction and trusted
presentation with existing core/terminal dependencies. No I/O/network/host
capability or new registry package. Local/inherited-worker/shared adapters use the
same workflow. Preview, explicit Save, visibility and whole-page deletion stay
clear; unsaved page text has no autosave/checkpoint. Plain/compact fallback and
current frame/output/default/credential/helper limits remain. Optional M action
permits a custom theme key/label; enablement comes from sysop policy.

## Alternatives and consequences

An actual `.plan` file/Finger service would introduce host path/account and public
protocol authority unnecessarily. Reusing room posts would conflate discussion,
unread markers and prospective NNTP publication with member-controlled local
visibility. Reusing encrypted mail would misrepresent plaintext profile privacy.
No member-page private text may become a private-message fallback.

ASCII first preserves tiny original signatures on classic terminals and avoids
unreviewed terminal controls; Unicode room composing/rendering is unchanged.
Native encrypted mail/roaming and richer future page formats remain separate.
Hidden social fields remain plaintext/sysop-readable, not end-to-end encrypted.
Owner exports and complete bundles preserve hidden pages/updates as local data.
Stopped complete pre-upgrade backup and fresh-path rollback are required. Logical
deletion cannot promise removal from earlier WAL, exports or backups.

## Evidence and reversal

See [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/135) and the
[member/sysop guide](../../docs/member-pages.md). Validate separate members,
current revocation, ownership/conflicts, hidden-body/count filtering, configured
layouts/fields, history retention, preview/retry/delete and actual native recovery.
Snapshot 28 freezes 202 prior wall execution/harness/compatibility inputs and its
whole result. Keep forty-one historical identities fixed. Disable the feature to
stop local use; removal of schema 10 requires fresh-path pre-upgrade restoration,
rather than a live downgrade. No host/hardening/federation adoption follows.
