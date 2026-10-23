# Magazine workshop

The opt-in local workshop lets approved members submit original work, sysops
assemble issues, and contributors approve the exact complete edition before
local publication. Library K → W opens it in the shared service, local console
or worker. The existing [static magazine reader](board-magazine.md) remains
available for owner-curated packs.

## Enable and use

Copy or adapt [the workshop pack](../library/workshop/library.toml). Set the
existing library service path (or console/worker --library PACK) and restart
the owner to load the validated pack. The required library identity, compact
setting and shelf/book arrays remain; editorial=true enables W and
magazine_name supplies the issue identity. Existing shelves, request desk and
static magazines can coexist in the same bounded manifest. Disabling the
workshop hides its menus without deleting records.

Members choose S for their drafts/submissions, then C to compose. Provide title,
section, rights, provenance and printable ASCII body lines. .undo, .clear,
.review, .cancel and doubled leading dots work as displayed. N/P review every
page; S saves a private draft, T explicitly submits it to local sysop editors and
the selected contributor review cohort. Confirm YES. Drafts are readable by their
author and sysops; submissions are not publication consent. Reopen/edit a source
to submit a saved draft. This board storage is plaintext and is not encrypted
private mail. Never paste private-room material without every required author's
explicit permission.

Sysops use A for the submission queue and C to create a numbered draft issue.
They define its title, edition revision, edition label and schedule label.
Schedule is descriptive; publication is manual. Within the issue, S selects,
replaces or removes a slot by submission ID; E edits a copied proposal; H edits
its header. Author identity comes from the selected source account. Editors
cannot rewrite another member's original manuscript or approve for that member.
Sections are per-contribution labels; submit only original/licensed text artwork,
essays, interviews or reports within the bounds below.

Contributors use I to find their assigned reviews. The issue has a contents list,
credits/provenance and all proposed text. Read N/P through the end, then A and YES
to approve all of your selected contributions in that exact issue for this
board's approved-member audience. Any header, selection or proposal edit
invalidates every approval. A changed source must be explicitly selected again.
The sysop reviews the whole issue and types PUBLISH, then YES; every selected
author must still be approved and have approved the current submitted revision.
P always means previous page. Ordinary readers see only currently eligible
published issues, with the same themed, classic and escape-free plain reading.

## Revisions, withdrawal and retention

An edition is identified by board namespace, internal issue ID, issue number and
edition revision. Published article/header text cannot change. For another
revision, create a new draft edition and collect new approvals. Existing
manuscript edits do not rewrite a published copy.

An author can U revoke approval; an author or sysop can X withdraw a source.
Either withdraws affected published issues from ordinary readers. Sysops can X
withdraw an issue. Current account revocation hides affected issues and expires
its sessions; reapproval may restore visibility if consent/source was not
withdrawn. Review-cohort access is checked against current author/source state.
These are trusted local account approvals, not PGP signatures or proof against
a malicious sysop with database access.

Z deletes an unselected source or non-published issue after confirmation.
Selected source provenance is retained until referencing issues are deleted.
No silent expiry. Live deletion/withdrawal cannot erase exports, old downloads,
screenshots or backups; public/Gopher/NNTP redistribution is not authorized.

## Optional text edition and Files

The OS filesystem owner may export an eligible published edition:

~~~sh
faction --state-dir /path/to/stopped-or-live-board magazine-edition 1 /private/new-edition.txt
~~~

This validates a read-only supported schema-16/17 catalog and creates a new mode-0600
text file without overwrite/symlink-following. It does not initialize or migrate
the board, authenticate as a member, export private drafts, or publish bytes.
Use the existing reviewed Files upload/publication procedure to distribute that
file. Sysop O attaches an existing published related file; this separate curator
offer does not change approved article text or validate matching file contents.
L displays the current scoped link; F opens the existing native Files transfer
flow when available. Current file publication, board namespace, area grants,
confirmation and transfer budgets remain authoritative. A reader may read an
issue without access to its related file. Withdraw/remove Files independently
when required; withdrawing an issue does not revoke an already issued download
or automatically remove the related file.

## Bounds and upgrade

Retain at most 128 manuscripts, eight active per author, 32 issues and eight
selected contributions per issue. Each body: 4,096 bytes, 64 lines, 120 ASCII
columns per line. Exact issue text including metadata/contents is at most
32 KiB. Lists show four cards; text uses eight wrapped rows and a 32-position
rewind ring, with forward reading across the whole bounded issue. Resizing
requires a fresh complete review. Workshop frames remain below 8 KiB.
No raw ANSI/control/Unicode prose or embedded file-token authority is accepted;
use the related-file action. Larger/Unicode/art formats remain future work.

Stop owner sessions and take a [complete backup](export-restore.md) before the
forward-only catalog upgrade (current schema 17). Export/verify/restore accepts exact supported
schemas 2–17 read-only; ordinary owner startup migrates older restored boards.
Complete backups retain sources, proposals, consent and related-file metadata.
Static packs/themes remain external configuration and need separate preservation.
Restore matching catalog and file objects together; older binaries cannot open
the newer catalog. Normal eight-session, five-second admission, credential/helper/output
and transfer limits are unchanged. No new capacity, hardening, cryptographic
signing, federation or external publication claim.
